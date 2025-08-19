use crate::{
    inkwell_intrinsic::{
        build_asan_init_redzone, build_asan_mem_check, can_skip_instrument, get_ptr_operand,
        module_verify, set_filename,
    },
    module::InstrumentModule,
};
use anyhow::Result;
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicType, IntType},
    values::{AnyValue, GlobalValue, InstructionOpcode, InstructionValue, IntValue, PointerValue},
    AddressSpace,
};
use std::collections::{HashMap, HashSet};

const REDZONE_SIZE: u32 = 32;

fn build_memcheck<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    filename_str_ptr: Option<GlobalValue<'ctx>>,
    instr: &InstructionValue<'ctx>,
    ptr: PointerValue<'ctx>,
    access_size: IntValue<'ctx>,
) -> Result<()> {
    // the index value must be integer type when using it as array index (RHS) value (e.g., int val = arr[idx] + 1)
    builder.position_before(&instr);
    build_asan_mem_check(
        context,
        module,
        builder,
        &filename_str_ptr.unwrap(),
        ptr,
        access_size,
    )?;
    Ok(())
}

fn handle_load<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    filename_str_ptr: Option<GlobalValue<'ctx>>,
    instr: &InstructionValue<'ctx>,
) -> Result<()> {
    let ptr = get_ptr_operand(&instr, 0);
    if instr.get_type().is_int_type() {
        let access_size = instr.get_type().into_int_type().size_of();
        build_memcheck(
            context,
            module,
            builder,
            filename_str_ptr,
            &instr,
            ptr,
            access_size,
        )?;
    }
    Ok(())
}

fn handle_store<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    filename_str_ptr: Option<GlobalValue<'ctx>>,
    instr: &InstructionValue<'ctx>,
) -> Result<()> {
    let offset = instr.get_operand(0).unwrap().left().unwrap();
    let ptr = get_ptr_operand(&instr, 1);

    // Do not consider pointer arithmetic (e.g., arr[idx] = value, where
    // value is pointer)
    if offset.is_pointer_value() {
        return Ok(());
    }

    // pointer value is used as LHS in `store` instruction. (e.g., arr[idx] = value)
    // any type of access size is valid (i.e., `= value`)
    let access_size = offset.get_type().size_of().unwrap();
    build_memcheck(
        context,
        module,
        builder,
        filename_str_ptr,
        &instr,
        ptr,
        access_size,
    )?;
    Ok(())
}

fn handle_alloca<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    replaced_alloca: &mut HashMap<PointerValue<'ctx>, (PointerValue<'ctx>, IntType<'ctx>)>,
    instr: &InstructionValue<'ctx>,
) -> Result<()> {
    // 1. allocate [ redzone | usable | redzone ]
    let static_arr_kind = instr.get_allocated_type().unwrap();
    if !static_arr_kind.is_array_type() {
        return Ok(());
    }
    let static_arr = static_arr_kind.into_array_type();
    let arr_len = static_arr.len();
    let arr_typ = static_arr.get_element_type();
    let elem_typ = arr_typ.into_int_type();
    let elem_typ_byte = elem_typ.get_bit_width() / 8;
    let rz_size = REDZONE_SIZE / elem_typ_byte;

    builder.position_before(&instr);
    let new_alloca = builder.build_alloca(elem_typ.array_type(rz_size + arr_len + rz_size), "")?;
    let new_alloca_instr = new_alloca.as_instruction().unwrap();
    new_alloca_instr
        .set_alignment(instr.get_alignment().unwrap())
        .unwrap();
    instr.replace_all_uses_with(&new_alloca_instr);
    instr.erase_from_basic_block();

    let next_instr_of_new_alloc = new_alloca_instr.get_next_instruction().unwrap();
    builder.position_before(&next_instr_of_new_alloc);

    // 2. mark redzones and set shadow memory
    build_asan_init_redzone(
        context,
        module,
        builder,
        new_alloca,
        static_arr.size_of().unwrap(),
    )?;

    // 3. add redzone size to allocated pointer to correctly set the usable pointer
    let new_alloca_ptr = builder.build_alloca(context.ptr_type(AddressSpace::default()), "")?;
    let zero_offset = context.i64_type().const_int(0, false);
    let new_alloca_start_ptr = unsafe {
        builder.build_in_bounds_gep(
            elem_typ.array_type(rz_size + arr_len + rz_size),
            new_alloca,
            &[zero_offset, zero_offset],
            "",
        )?
    };
    let rz_offset = context
        .i64_type()
        .const_int((REDZONE_SIZE / elem_typ_byte).into(), false);
    let usable_ptr =
        unsafe { builder.build_gep(elem_typ, new_alloca_start_ptr, &[rz_offset], "")? };
    builder.build_store(new_alloca_ptr, usable_ptr)?;

    replaced_alloca.insert(new_alloca, (new_alloca_ptr, elem_typ));
    Ok(())
}

fn handle_call<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    replaced_alloca: &HashMap<PointerValue<'ctx>, (PointerValue<'ctx>, IntType<'ctx>)>,
    instr: &InstructionValue<'ctx>,
) -> Result<()> {
    // Inkwell seems to hold a subtle bug, where metadata instruction crashes to get operand
    if !instr
        .print_to_string()
        .to_str()
        .unwrap()
        .contains("llvm.dbg.declare")
    {
        for (idx, operand) in instr.get_operands().into_iter().enumerate() {
            if operand.is_none() {
                return Ok(());
            }
            let operand = operand.unwrap();
            if operand.left().is_none() {
                return Ok(());
            }
            let operand = operand.left().unwrap();
            if operand.is_pointer_value() {
                if let Some((replaced_alloca_ptr, _)) =
                    replaced_alloca.get(&operand.into_pointer_value())
                {
                    builder.position_before(&instr);
                    let loaded_ptr = builder.build_load(
                        context.ptr_type(AddressSpace::default()),
                        *replaced_alloca_ptr,
                        "",
                    )?;
                    instr.set_operand(idx.try_into().unwrap(), loaded_ptr);
                }
            }
        }
    }
    Ok(())
}

fn handle_gep<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    replaced_alloca: &HashMap<PointerValue<'ctx>, (PointerValue<'ctx>, IntType<'ctx>)>,
    instr: &InstructionValue<'ctx>,
) -> Result<()> {
    let target_ptr = instr
        .get_operand(0)
        .unwrap()
        .left()
        .unwrap()
        .into_pointer_value();
    if let Some((replaced_alloca_ptr, replaced_alloca_typ)) = replaced_alloca.get(&target_ptr) {
        builder.position_before(&instr);
        // 1. add `load` instruction for pointer
        let loaded_ptr = builder.build_load(
            context.ptr_type(AddressSpace::default()),
            *replaced_alloca_ptr,
            "",
        )?;
        // 2. install `gep` instruction and remove older one
        let idx = instr
            .get_operand(2)
            .unwrap()
            .left()
            .unwrap()
            .into_int_value();
        let gep = unsafe {
            builder.build_gep(
                // replaced_alloca_typ.unwrap(),
                *replaced_alloca_typ,
                loaded_ptr.into_pointer_value(),
                &[idx],
                "",
            )?
        };
        instr.replace_all_uses_with(&gep.as_instruction().unwrap());
        instr.erase_from_basic_block();
    }
    Ok(())
}

#[derive(Default)]
pub struct ASANModule {}

impl InstrumentModule for ASANModule {
    fn instrument<'ctx>(
        &self,
        context: &'ctx Context,
        module: &Module<'ctx>,
        builder: &Builder<'ctx>,
    ) -> Result<()> {
        let mut filename_str_ptr = None;
        let funcs: Vec<_> = module.get_functions().collect();
        for func in funcs {
            // Skip funcs without bodies or those we've added
            if can_skip_instrument(&func) {
                continue;
            }
            set_filename(module, builder, &mut filename_str_ptr, &func)?;
            let mut replaced_alloca = HashMap::new();
            let mut instrumented_blks = HashSet::new();
            for basic_blk in func.get_basic_blocks() {
                if instrumented_blks.contains(&basic_blk) {
                    continue;
                }
                for instr in basic_blk.get_instructions() {
                    match instr.get_opcode() {
                        // install asan check
                        InstructionOpcode::Load => {
                            handle_load(context, module, builder, filename_str_ptr, &instr)?;
                        }
                        InstructionOpcode::Store => {
                            handle_store(context, module, builder, filename_str_ptr, &instr)?;
                        }
                        InstructionOpcode::Alloca => {
                            handle_alloca(context, module, builder, &mut replaced_alloca, &instr)?;
                        }
                        // Replace all uses of origin static object with newly allocated object's pointer
                        InstructionOpcode::Call => {
                            handle_call(context, builder, &replaced_alloca, &instr)?;
                        }
                        InstructionOpcode::GetElementPtr => {
                            handle_gep(context, builder, &replaced_alloca, &instr)?;
                        }
                        _ => {}
                    }
                }
                instrumented_blks.insert(basic_blk);
            }
        }
        // Verify instrumented IRs
        module_verify(module)
    }
}
