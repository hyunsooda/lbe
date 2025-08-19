use crate::{
    inkwell_intrinsic::{
        build_ctros, build_pthread_self, build_race_init, build_update_lock_held,
        build_update_shared_mem, can_skip_instrument, module_verify,
    },
    llvm_intrinsic::{cstr_to_str, get_instr_loc},
    module::InstrumentModule,
    names::{PTHREAD_MUTEX_LOCK, PTHREAD_MUTEX_UNLOCK},
};
use anyhow::Result;
use inkwell::values::{InstructionOpcode, InstructionValue, PointerValue};
use inkwell::{
    builder::Builder,
    context::Context,
    llvm_sys::LLVMValue,
    module::Module,
    values::{AsValueRef, BasicValueEnum, CallSiteValue, FunctionValue},
};
use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum LockTyp {
    Lock,
    UnLock,
}
impl LockTyp {
    pub fn i8(&self) -> i8 {
        match self {
            Self::Lock => 1,
            Self::UnLock => 0,
        }
    }

    pub fn from_i8(t: i8) -> Option<Self> {
        match t {
            1 => Some(Self::Lock),
            0 => Some(Self::UnLock),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum AccessOperation {
    Write,
    Read,
}
impl AccessOperation {
    pub fn i8(&self) -> i8 {
        match self {
            Self::Write => 1,
            Self::Read => 0,
        }
    }

    pub fn from_i8(t: i8) -> Option<Self> {
        match t {
            1 => Some(Self::Write),
            0 => Some(Self::Read),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lock<'ctx> {
    pub typ: LockTyp,
    pub lock: PointerValue<'ctx>,
}

impl<'ctx> Lock<'ctx> {
    fn new(typ: LockTyp, lock: PointerValue<'ctx>) -> Self {
        Self { typ, lock }
    }
}

impl<'ctx> PartialEq for Lock<'ctx> {
    fn eq(&self, other: &Self) -> bool {
        self.typ == other.typ && self.lock.as_value_ref() == other.lock.as_value_ref()
    }
}

impl<'ctx> Eq for Lock<'ctx> {}

impl<'ctx> Hash for Lock<'ctx> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.typ.hash(state);
        self.lock.as_value_ref().hash(state);
    }
}

#[derive(Default)]
pub struct RaceModule {}

impl InstrumentModule for RaceModule {
    fn instrument<'ctx>(
        &self,
        context: &'ctx Context,
        module: &Module<'ctx>,
        builder: &Builder<'ctx>,
    ) -> Result<()> {
        let global_int_vals = get_global_int_vals(module);

        let mut lock_candidate_set = HashMap::new();
        let mut live_locks = HashSet::new();
        let mut pthread_self_callsites = HashMap::new();
        let funcs: Vec<_> = module.get_functions().collect();
        for func in &funcs {
            // Skip funcs without bodies or those we've added
            if can_skip_instrument(&func) {
                continue;
            }
            for basic_blk in func.get_basic_blocks() {
                for instr in basic_blk.get_instructions() {
                    if let Some(lock) = get_lock(&instr) {
                        match lock.typ {
                            LockTyp::Lock => {
                                live_locks.insert(lock.clone());
                            }
                            LockTyp::UnLock => {
                                live_locks.remove(&lock);
                            }
                        }
                        // instrument for `lock_held(t)`
                        if let Some(next_instr) = instr.get_next_instruction() {
                            builder.position_before(&next_instr);
                        }
                        let thread_id = get_thread_id(
                            context,
                            module,
                            builder,
                            func,
                            &instr,
                            &mut pthread_self_callsites,
                        )?;
                        build_update_lock_held(context, module, builder, thread_id, lock)?;
                        continue;
                    }

                    if let InstructionOpcode::Load | InstructionOpcode::Store = instr.get_opcode() {
                        let (operand, access_op) = match instr.get_opcode() {
                            InstructionOpcode::Load => {
                                let operand = instr.get_operand(0).unwrap().left().unwrap();
                                (operand, AccessOperation::Read)
                            }
                            InstructionOpcode::Store => {
                                let operand = instr.get_operand(1).unwrap().left().unwrap();
                                (operand, AccessOperation::Write)
                            }
                            _ => unreachable!(),
                        };
                        let thread_id = get_thread_id(
                            context,
                            module,
                            builder,
                            func,
                            &instr,
                            &mut pthread_self_callsites,
                        )?;
                        handle_mem_access(
                            &instr,
                            &operand,
                            access_op,
                            &global_int_vals,
                            &mut lock_candidate_set,
                            &live_locks,
                            thread_id,
                            context,
                            module,
                            builder,
                        )?;
                    }
                }
            }
        }

        let constructor = build_race_init(context, module, builder, &lock_candidate_set)?;
        build_ctros(context, module, constructor)?;

        // Verify instrumented IRs
        module_verify(module)
    }
}

fn get_global_int_vals<'ctx>(module: &Module<'ctx>) -> HashSet<PointerValue<'ctx>> {
    let globals: HashSet<PointerValue<'ctx>> = module
        .get_globals()
        .filter(|global| global.get_value_type().is_int_type())
        .map(|global| global.as_pointer_value())
        .collect();
    globals
}

fn get_lock<'ctx>(instr: &InstructionValue<'ctx>) -> Option<Lock<'ctx>> {
    if instr.get_opcode() != InstructionOpcode::Call || instr.get_num_operands() != 2 {
        return None;
    }

    if let Some(operand) = instr.get_operand(1) {
        if let Some(func_name_val) = operand.left() {
            let func_name = cstr_to_str(func_name_val.get_name());

            let lock_typ = match func_name.as_str() {
                PTHREAD_MUTEX_LOCK => Some(LockTyp::Lock),
                PTHREAD_MUTEX_UNLOCK => Some(LockTyp::UnLock),
                _ => None,
            };
            if lock_typ.is_some() {
                let lock_operand = instr
                    .get_operand(0)
                    .unwrap()
                    .left()
                    .unwrap()
                    .into_pointer_value();
                return Some(Lock::new(lock_typ.unwrap(), lock_operand));
            }
        }
    }
    None
}

fn handle_mem_access<'ctx>(
    instr: &InstructionValue<'ctx>,
    operand: &BasicValueEnum<'ctx>,
    access_op: AccessOperation,
    global_int_vals: &HashSet<PointerValue<'ctx>>,
    lock_candidate_set: &mut HashMap<PointerValue<'ctx>, Vec<Lock<'ctx>>>,
    live_locks: &HashSet<Lock<'ctx>>,
    thread_id: CallSiteValue<'ctx>,
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
) -> Result<()> {
    if operand.is_pointer_value() {
        if let Some(global_var) = global_int_vals.get(&operand.into_pointer_value()) {
            lock_candidate_set.insert(
                operand.into_pointer_value(),
                live_locks.clone().into_iter().collect::<Vec<_>>(),
            );
            if let Some(next_instr) = instr.get_next_instruction() {
                builder.position_before(&next_instr);
            }
            let (line, _) = get_instr_loc(instr);
            build_update_shared_mem(
                context, module, builder, access_op, thread_id, global_var, line,
            )?;
        }
    }
    Ok(())
}

fn get_thread_id<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    func: &FunctionValue<'ctx>,
    instr: &InstructionValue<'ctx>,
    pthread_self_callsites: &mut HashMap<*mut LLVMValue, CallSiteValue<'ctx>>,
) -> Result<CallSiteValue<'ctx>> {
    let func_id = func.as_value_ref();
    let thread_id = match pthread_self_callsites.get(&func_id) {
        Some(thread_id) => *thread_id,
        None => {
            builder.position_before(&instr);
            let thread_id = build_pthread_self(context, module, builder)?;
            pthread_self_callsites.insert(func.as_value_ref(), thread_id);
            thread_id
        }
    };
    Ok(thread_id)
}
