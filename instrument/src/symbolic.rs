use crate::{
    inkwell_intrinsic::{
        build_ctros, build_sym_make_prep, build_symbolic_init, can_skip_instrument, module_verify,
        set_filename,
    },
    llvm_intrinsic::{cstr_to_str, get_instr_filename},
    module::InstrumentModule,
    names::SYMBOLIC_MAKE_VAR,
};
use anyhow::Result;
use inkwell::values::{AsValueRef, InstructionOpcode};
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    values::{BasicValueEnum, PointerValue},
    IntPredicate,
};
use std::collections::HashSet;
use std::{collections::HashMap, sync::Mutex};

lazy_static::lazy_static! {
    static ref CONSTRAINT_ID: Mutex<i64> = Mutex::new(0);
}

pub const VAR_KIND: i8 = 0;
pub const CONST_KIND: i8 = 1;

pub const PREDICATE_EQ: i8 = 0;
pub const PREDICATE_NE: i8 = 1;
pub const PREDICATE_SLT: i8 = 2;
pub const PREDICATE_SLE: i8 = 3;
pub const PREDICATE_SGT: i8 = 4;
pub const PREDICATE_SGE: i8 = 5;

#[derive(Debug, Clone)]
enum Operand<'ctx> {
    Var(PointerValue<'ctx>),
    Const(i64),
}
impl<'ctx> Operand<'ctx> {
    fn is_const(&self) -> bool {
        if let Self::Const(_) = self {
            return true;
        } else {
            return false;
        }
    }
}

#[derive(Debug, Clone)]
struct Constraint<'ctx> {
    left_operand: Operand<'ctx>,
    right_operand: Operand<'ctx>,
    predicate: IntPredicate,
}

impl<'ctx> Constraint<'ctx> {
    fn new(
        left_operand: Operand<'ctx>,
        right_operand: Operand<'ctx>,
        predicate: IntPredicate,
    ) -> Self {
        Self {
            left_operand,
            right_operand,
            predicate,
        }
    }

    fn predicate_to_i8(&self) -> i8 {
        match &self.predicate {
            IntPredicate::EQ => PREDICATE_EQ,
            IntPredicate::NE => PREDICATE_NE,
            IntPredicate::SLT => PREDICATE_SLT,
            IntPredicate::SLE => PREDICATE_SLE,
            IntPredicate::SGT => PREDICATE_SGT,
            IntPredicate::SGE => PREDICATE_SGE,
            _ => unreachable!("unsigned operation is not supported"),
        }
    }

    fn negate(&self) -> Self {
        let predicate = match self.predicate {
            IntPredicate::EQ => IntPredicate::NE,
            IntPredicate::NE => IntPredicate::EQ,
            IntPredicate::SLT => IntPredicate::SGE,
            IntPredicate::SLE => IntPredicate::SGT,
            IntPredicate::SGT => IntPredicate::SLE,
            IntPredicate::SGE => IntPredicate::SLT,
            _ => unreachable!("unsigned operation is not supported"),
        };
        Self::new(
            self.left_operand.clone(),
            self.right_operand.clone(),
            predicate,
        )
    }
}

#[derive(Debug)]
pub struct ConstraintSerialized {
    pub id: i64,
    pub left_operand_kind: i8,
    pub left_operand_val: i64,
    pub right_operand_kind: i8,
    pub right_operand_val: i64,
    pub predicate: i8,
}

impl ConstraintSerialized {
    pub fn new(
        id: i64,
        left_operand_kind: i8,
        left_operand_val: i64,
        right_operand_kind: i8,
        right_operand_val: i64,
        predicate: i8,
    ) -> Self {
        Self {
            id,
            left_operand_kind,
            left_operand_val,
            right_operand_kind,
            right_operand_val,
            predicate,
        }
    }
}

#[derive(Debug, Clone)]
struct State<'ctx> {
    id: i64,
    path_constraints: Vec<Constraint<'ctx>>,
    is_leaf: bool,
}

impl<'ctx> State<'ctx> {
    fn new() -> Self {
        Self {
            id: 0,
            path_constraints: vec![],
            is_leaf: false,
        }
    }

    fn fork(&self, constraint: Constraint<'ctx>) -> (Self, Self) {
        let mut id = CONSTRAINT_ID.lock().unwrap();
        let (tid, fid) = (*id + 1, *id + 2);
        *id += 2;
        let (mut copied_state_t, mut copied_state_f) = (self.clone(), self.clone());
        copied_state_t.path_constraints.push(constraint.clone());
        copied_state_f.path_constraints.push(constraint.negate());
        copied_state_t.id = tid;
        copied_state_f.id = fid;
        (copied_state_t, copied_state_f)
    }
}

#[derive(Default)]
pub struct SymbolicModule {}

impl InstrumentModule for SymbolicModule {
    fn instrument<'ctx>(
        &self,
        context: &'ctx Context,
        module: &Module<'ctx>,
        builder: &Builder<'ctx>,
    ) -> Result<()> {
        let sym_ptrs = build_sym_ptrs(context, module, builder)?;
        let module_filename = cstr_to_str(module.get_source_file_name());
        let mut serialized = vec![];
        let mut filename_str_ptr = None;
        let funcs: Vec<_> = module.get_functions().collect();
        for func in funcs {
            // Skip funcs without bodies or those we've added
            if can_skip_instrument(&func) {
                continue;
            }
            set_filename(module, builder, &mut filename_str_ptr, &func)?;
            let mut instrumented_blks = HashSet::new();

            let mut states: HashMap<i64, State> = HashMap::new();
            let mut bb_id = HashMap::new();
            if let Some(first_bb) = func.get_first_basic_block() {
                let first_bb_addr = first_bb.as_mut_ptr() as usize;
                bb_id.insert(first_bb_addr, 0);
                states.insert(0, State::new());
            }
            for basic_blk in func.get_basic_blocks() {
                if instrumented_blks.contains(&basic_blk) {
                    continue;
                }
                if let Some(first_instr) = basic_blk.get_first_instruction() {
                    // this guard prevents instrumentation of unknown code (e.g., C++)
                    if let Some(instr_filename) = get_instr_filename(&first_instr) {
                        if instr_filename != module_filename {
                            continue;
                        }
                    }
                }
                for instr in basic_blk.get_instructions() {
                    match instr.get_opcode() {
                        InstructionOpcode::Br => {
                            // if the current basic block has a state and the terminator is not
                            // conditional, we mark them as leaf
                            if !instr.is_conditional() {
                                let bb_addr = basic_blk.as_mut_ptr() as usize;
                                if let Some(id) = bb_id.get(&bb_addr) {
                                    if let Some(state) = states.get_mut(&id) {
                                        state.is_leaf = true;
                                    }
                                }
                            }
                        }
                        InstructionOpcode::ICmp => {
                            if let Some(br_instr) = instr.get_next_instruction() {
                                if br_instr.get_opcode() == InstructionOpcode::Br
                                    && br_instr.is_conditional()
                                {
                                    let (left_op, right_op) = (
                                        instr.get_operand(0).unwrap().left().unwrap(),
                                        instr.get_operand(1).unwrap().left().unwrap(),
                                    );
                                    let (tbr, fbr) = (
                                        br_instr.get_operand(2).unwrap().right().unwrap(),
                                        br_instr.get_operand(1).unwrap().right().unwrap(),
                                    );
                                    match (
                                        get_sym_ptr_operand(&sym_ptrs, left_op),
                                        get_sym_ptr_operand(&sym_ptrs, right_op),
                                    ) {
                                        (Some(left_operand), Some(right_operand)) => {
                                            // ignore if both operands are constant
                                            if left_operand.is_const() && right_operand.is_const() {
                                                continue;
                                            }
                                            let predicate = instr.get_icmp_predicate().unwrap();
                                            let constraint = Constraint::new(
                                                left_operand,
                                                right_operand,
                                                predicate,
                                            );
                                            let bb_addr = basic_blk.as_mut_ptr() as usize;
                                            if let Some(id) = bb_id.get(&bb_addr) {
                                                if let Some(state) = states.get(&id) {
                                                    let (state_t, state_f) = state.fork(constraint);
                                                    bb_id.insert(
                                                        tbr.as_mut_ptr() as usize,
                                                        state_t.id,
                                                    );
                                                    bb_id.insert(
                                                        fbr.as_mut_ptr() as usize,
                                                        state_f.id,
                                                    );
                                                    states.insert(state_t.id, state_t);
                                                    states.insert(state_f.id, state_f);
                                                }
                                            } else {
                                                let state = states.get(&0).unwrap();
                                                let (state_t, state_f) =
                                                    state.fork(constraint.clone());
                                                bb_id.insert(tbr.as_mut_ptr() as usize, state_t.id);
                                                bb_id.insert(fbr.as_mut_ptr() as usize, state_f.id);
                                                states.insert(state_t.id, state_t);
                                                states.insert(state_f.id, state_f);
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                instrumented_blks.insert(basic_blk);
            }
            if states.len() > 1 {
                serialized.push(serailize_constraints(&states));
            }
        }
        if !serialized.is_empty() {
            let constructor = build_symbolic_init(context, module, builder, serialized)?;
            build_ctros(context, module, constructor)?;
        }

        // Verify instrumented IRs
        module_verify(module)
    }
}

fn build_sym_ptrs<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
) -> Result<Vec<PointerValue<'ctx>>> {
    let mut symbolic_ptr_inits = HashSet::new();
    let funcs: Vec<_> = module.get_functions().collect();
    for func in funcs {
        if can_skip_instrument(&func) {
            continue;
        }
        for basic_blk in func.get_basic_blocks() {
            for instr in basic_blk.get_instructions() {
                if instr.get_opcode() == InstructionOpcode::Call && instr.get_num_operands() == 3 {
                    let func_ptr = instr.get_operand(2).unwrap().left().unwrap();
                    let func_str = cstr_to_str(func_ptr.get_name());
                    if func_str == SYMBOLIC_MAKE_VAR {
                        let var_ptr = instr
                            .get_operand(1)
                            .unwrap()
                            .left()
                            .unwrap()
                            .into_pointer_value();
                        symbolic_ptr_inits.insert(var_ptr);

                        // install <address, pointer value> mapping
                        builder.position_before(&instr);
                        build_sym_make_prep(context, module, builder, var_ptr)?;
                    }
                }
            }
        }
    }
    Ok(symbolic_ptr_inits.iter().cloned().collect())
}

fn get_sym_ptr_operand<'ctx>(
    sym_ptrs: &Vec<PointerValue<'ctx>>,
    op: BasicValueEnum<'ctx>,
) -> Option<Operand<'ctx>> {
    if op.is_int_value() {
        let op = op.into_int_value();
        if op.is_const() {
            return Some(Operand::Const(op.get_sign_extended_constant().unwrap()));
        } else {
            let mut instr = op.as_instruction().unwrap();
            loop {
                match instr.get_opcode() {
                    InstructionOpcode::Load => {
                        let ptr = instr.get_operand(0).unwrap().left().unwrap();
                        for sym_ptr in sym_ptrs {
                            if *sym_ptr == ptr {
                                return Some(Operand::Var(*sym_ptr));
                            }
                        }
                        return None;
                    }
                    _ => {
                        if let Some(prev_instr) = instr.get_previous_instruction() {
                            instr = prev_instr;
                        } else {
                            return None;
                        }
                    }
                }
            }
        }
    }
    None
}

fn serailize_constraints<'ctx>(states: &HashMap<i64, State>) -> Vec<ConstraintSerialized> {
    let mut constraints = vec![];
    for (_, state) in states {
        if state.is_leaf {
            for constraint in &state.path_constraints {
                let (left_kind, left_op) = match constraint.left_operand {
                    Operand::Var(var) => (VAR_KIND, var.as_value_ref() as i64),
                    Operand::Const(v) => (CONST_KIND, v),
                };
                let (right_kind, right_op) = match constraint.right_operand {
                    Operand::Var(var) => (VAR_KIND, var.as_value_ref() as i64),
                    Operand::Const(v) => (CONST_KIND, v),
                };
                let predicate = constraint.predicate_to_i8();
                constraints.push(ConstraintSerialized::new(
                    state.id, left_kind, left_op, right_kind, right_op, predicate,
                ));
            }
        }
    }
    constraints
}
