use crate::{
    llvm_intrinsic::{cstr_to_str, get_instr_filename, get_instr_loc},
    names::*,
    race::{AccessOperation, Lock},
    symbolic::ConstraintSerialized,
};
use anyhow::Result;
use inkwell::{
    builder::Builder,
    context::Context,
    llvm_sys::{self},
    module::Module,
    values::{
        AsValueRef, BasicValueEnum::ArrayValue, CallSiteValue, FunctionValue, GlobalValue,
        InstructionValue, IntValue, PointerValue, StructValue,
    },
    AddressSpace,
};
use llvm_sys::core::{LLVMGetAggregateElement, LLVMGetNumOperands};
use std::collections::{BTreeSet, HashMap};

pub fn get_func<'ctx>(module: &Module<'ctx>, func_name: &str) -> Option<FunctionValue<'ctx>> {
    module.get_function(func_name)
}

pub fn get_or_build_global_string_ptr<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    filename_str: &str,
) -> Result<GlobalValue<'ctx>> {
    match module.get_global(filename_str) {
        Some(filename) => Ok(filename),
        None => Ok(builder.build_global_string_ptr(&filename_str, filename_str)?),
    }
}

pub fn get_ptr_operand<'ctx>(instr: &InstructionValue<'ctx>, idx: u32) -> PointerValue<'ctx> {
    instr
        .get_operand(idx)
        .unwrap()
        .left()
        .unwrap()
        .into_pointer_value()
}

pub fn set_filename<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    filename_val: &mut Option<GlobalValue<'ctx>>,
    func: &FunctionValue<'ctx>,
) -> Result<()> {
    if filename_val.is_none() {
        if let Some(filename_str) = get_instr_filename(func) {
            if let Some(first_basic_blk) = func.get_first_basic_block() {
                if let Some(first_instr) = first_basic_blk.get_first_instruction() {
                    // Do not create redundant global string if defined
                    builder.position_before(&first_instr);
                    *filename_val = Some(get_or_build_global_string_ptr(
                        module,
                        builder,
                        filename_str,
                    )?);
                }
            }
        }
    }
    Ok(())
}

pub fn build_i32_static_arr<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    vals: &Vec<u32>,
    alloca_name: &str,
    gep_name: &str,
) -> Result<PointerValue<'ctx>> {
    let arr_typ = context.i32_type().array_type(vals.len().try_into()?);
    let int_vals: Vec<_> = vals
        .iter()
        .map(|v| context.i32_type().const_int(*v as u64, false))
        .collect();
    let arr_val = context.i32_type().const_array(&int_vals);
    let arr_alloca = builder.build_alloca(arr_typ, alloca_name)?;
    builder.build_store(arr_alloca, arr_val)?;

    // Get a pointer to the first element
    let zero = context.i32_type().const_int(0, false);
    let array_ptr = unsafe {
        builder.build_gep(
            arr_typ,
            arr_alloca,
            &[zero, zero], // Get pointer to the first element
            gep_name,
        )
    }?;
    Ok(array_ptr)
}

pub fn convert_to_int_val<'ctx>(context: &'ctx Context, v: u32) -> IntValue<'ctx> {
    context.i64_type().const_int(v as u64, false)
}

fn get_src_mapping_func<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    match get_func(module, COV_MAPPING_SRC) {
        Some(func) => func,
        None => {
            let record_func_src_map_typ = context.void_type().fn_type(
                &[
                    context.ptr_type(AddressSpace::default()).into(),
                    context.ptr_type(AddressSpace::default()).into(),
                    context.i64_type().into(),
                    context.ptr_type(AddressSpace::default()).into(),
                    context.i64_type().into(),
                    context.ptr_type(AddressSpace::default()).into(),
                    context.i64_type().into(),
                ],
                false,
            );
            module.add_function(COV_MAPPING_SRC, record_func_src_map_typ, None)
        }
    }
}

fn get_cov_hit_batch_func<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    match get_func(module, COV_HIT_BATCH) {
        Some(func) => func,
        None => {
            let record_func_cov_lines_typ = context.i64_type().fn_type(
                &[
                    context.ptr_type(AddressSpace::default()).into(),
                    context.ptr_type(AddressSpace::default()).into(),
                    context.i64_type().into(),
                ],
                false,
            );
            module.add_function(COV_HIT_BATCH, record_func_cov_lines_typ, None)
        }
    }
}

fn get_asan_mem_check_func<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    match get_func(module, ASAN_MEM_CHECK) {
        Some(func) => func,
        None => {
            let asan_mem_check_func_typ = context.void_type().fn_type(
                &[
                    context.ptr_type(AddressSpace::default()).into(),
                    context.ptr_type(AddressSpace::default()).into(),
                    context.i64_type().into(),
                ],
                false,
            );
            module.add_function(ASAN_MEM_CHECK, asan_mem_check_func_typ, None)
        }
    }
}

fn get_asan_init_redzone<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    match get_func(module, ASAN_MEM_INIT_REDZONE) {
        Some(func) => func,
        None => {
            let asan_mem_init_redzone = context.void_type().fn_type(
                &[
                    context.ptr_type(AddressSpace::default()).into(),
                    context.i64_type().into(),
                    context.i8_type().into(),
                ],
                false,
            );
            module.add_function(ASAN_MEM_INIT_REDZONE, asan_mem_init_redzone, None)
        }
    }
}

fn get_trace_edge<'ctx>(context: &'ctx Context, module: &Module<'ctx>) -> FunctionValue<'ctx> {
    match get_func(module, FUZZER_TRACE_EDGE) {
        Some(func) => func,
        None => {
            let trace_edge = context
                .void_type()
                .fn_type(&[context.i64_type().into()], false);
            module.add_function(FUZZER_TRACE_EDGE, trace_edge, None)
        }
    }
}

fn get_pthread_self<'ctx>(context: &'ctx Context, module: &Module<'ctx>) -> FunctionValue<'ctx> {
    match get_func(module, PTHREAD_SELF) {
        Some(func) => func,
        None => {
            let pthread_self_type = context.i64_type().fn_type(&[], false);
            module.add_function(PTHREAD_SELF, pthread_self_type, None)
        }
    }
}

fn get_update_lock_held<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    match get_func(module, RACE_LOCK_HELD) {
        Some(func) => func,
        None => {
            let lock_held = context.void_type().fn_type(
                &[
                    context.i8_type().into(),
                    context.i64_type().into(),
                    context.i64_type().into(),
                ],
                false,
            );
            module.add_function(RACE_LOCK_HELD, lock_held, None)
        }
    }
}

fn get_updated_shared_mem<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    match get_func(module, RACE_UPDATE_SHARED_MEM) {
        Some(func) => func,
        None => {
            let update_shared_mem = context.void_type().fn_type(
                &[
                    context.i8_type().into(),
                    context.i64_type().into(),
                    context.i64_type().into(),
                    context.i64_type().into(),
                ],
                false,
            );
            module.add_function(RACE_UPDATE_SHARED_MEM, update_shared_mem, None)
        }
    }
}

fn get_sym_make_prep<'ctx>(context: &'ctx Context, module: &Module<'ctx>) -> FunctionValue<'ctx> {
    match get_func(module, SYMBOLIC_MAKE_PREPARE) {
        Some(func) => func,
        None => {
            let sym_make_prep = context.void_type().fn_type(
                &[
                    context.ptr_type(AddressSpace::default()).into(),
                    context.i64_type().into(),
                ],
                false,
            );
            module.add_function(SYMBOLIC_MAKE_PREPARE, sym_make_prep, None)
        }
    }
}

pub fn build_src_mapping_call<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    filename_str_ptr: &GlobalValue,
    funcs_loc: &BTreeSet<u32>,
    brs_loc: &Vec<u32>,
    lines_loc: &BTreeSet<u32>,
) -> Result<()> {
    let funcs_loc_vec: Vec<_> = funcs_loc.clone().into_iter().collect();
    let brs_loc_vec: Vec<_> = brs_loc.clone().into_iter().collect();
    let lines_loc_vec: Vec<_> = lines_loc.clone().into_iter().collect();

    let func_lines_ptr = build_i32_static_arr(
        context,
        builder,
        &funcs_loc_vec,
        COV_SRC_MAPPING_FUNC_LINES,
        COV_SRC_MAPPING_FUNC_LINES_PTR,
    )?;
    let brs_lines_ptr = build_i32_static_arr(
        context,
        builder,
        &brs_loc_vec,
        COV_SRC_MAPPING_BRS_LINES,
        COV_SRC_MAPPING_BRS_LINES_PTR,
    )?;
    let lines_lines_ptr = build_i32_static_arr(
        context,
        builder,
        &lines_loc_vec,
        COV_SRC_MAPPING_LINES_LINES,
        COV_SRC_MAPPING_LINES_LINES_PTR,
    )?;

    builder.build_call(
        get_src_mapping_func(context, module),
        &[
            filename_str_ptr.as_pointer_value().into(),
            func_lines_ptr.into(),
            convert_to_int_val(context, funcs_loc_vec.len().try_into()?).into(),
            brs_lines_ptr.into(),
            convert_to_int_val(context, brs_loc_vec.len().try_into()?).into(),
            lines_lines_ptr.into(),
            convert_to_int_val(context, lines_loc_vec.len().try_into()?).into(),
        ],
        "",
    )?;
    Ok(())
}

pub fn build_cov_hit_batch<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    filename_str_ptr: &GlobalValue,
    arr_ptr: PointerValue,
    arr_length: usize,
) -> Result<()> {
    let record_func_cov_lines = get_cov_hit_batch_func(context, module);
    builder.build_call(
        record_func_cov_lines,
        &[
            filename_str_ptr.as_pointer_value().into(),
            arr_ptr.into(),
            convert_to_int_val(context, arr_length.try_into()?).into(),
        ],
        "",
    )?;
    Ok(())
}

pub fn build_asan_mem_check<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    filename_str_ptr: &GlobalValue,
    ptr: PointerValue,
    access_size: IntValue<'ctx>,
) -> Result<()> {
    let asan_mem_check = get_asan_mem_check_func(context, module);
    builder.build_call(
        asan_mem_check,
        &[
            filename_str_ptr.as_pointer_value().into(),
            ptr.into(),
            access_size.into(),
        ],
        "",
    )?;
    Ok(())
}

pub fn build_asan_init_redzone<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    ptr: PointerValue,
    usable_size: IntValue<'ctx>,
) -> Result<()> {
    let asan_init_redzone = get_asan_init_redzone(context, module);
    // stack alloc kind is defined as `0x01` in asan runtime crate
    let stack_alloc = context.i8_type().const_int(1, false);
    builder.build_call(
        asan_init_redzone,
        &[ptr.into(), usable_size.into(), stack_alloc.into()],
        "",
    )?;
    Ok(())
}

pub fn build_trace_edge<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    cur_loc: IntValue<'ctx>,
) -> Result<()> {
    let trace_edge = get_trace_edge(context, module);
    builder.build_call(trace_edge, &[cur_loc.into()], "")?;
    Ok(())
}

pub fn build_pthread_self<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
) -> Result<CallSiteValue<'ctx>> {
    // instrument POSIX `pthread_self()` call
    let pthread_self_fn = get_pthread_self(context, module);
    let thread_id = builder.build_call(pthread_self_fn, &[], "")?;
    Ok(thread_id)
}

pub fn build_update_lock_held<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    thread_id: CallSiteValue<'ctx>,
    lock: Lock<'ctx>,
) -> Result<()> {
    // instrument `__race_update_lock_held()`
    let lock_held = get_update_lock_held(context, module);
    builder.build_call(
        lock_held,
        &[
            context
                .i8_type()
                .const_int(lock.typ.i8() as u64, true)
                .into(),
            thread_id.try_as_basic_value().left().unwrap().into(),
            context
                .i64_type()
                .const_int(lock.lock.as_value_ref() as u64, true)
                .into(),
        ],
        "",
    )?;
    Ok(())
}

pub fn build_update_shared_mem<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    access_op: AccessOperation,
    thread_id: CallSiteValue<'ctx>,
    global_var: &PointerValue<'ctx>,
    line: u32,
) -> Result<()> {
    let update_shared_mem = get_updated_shared_mem(context, module);
    builder.build_call(
        update_shared_mem,
        &[
            context
                .i8_type()
                .const_int(access_op.i8() as u64, true)
                .into(),
            thread_id.try_as_basic_value().left().unwrap().into(),
            context
                .i64_type()
                .const_int(global_var.as_value_ref() as u64, true)
                .into(),
            context.i64_type().const_int(line as u64, true).into(),
        ],
        "",
    )?;
    Ok(())
}

pub fn build_sym_make_prep<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    ptr: PointerValue,
) -> Result<()> {
    let sym_make_prep = get_sym_make_prep(context, module);
    builder.build_call(
        sym_make_prep,
        &[
            ptr.into(),
            context
                .i64_type()
                .const_int(ptr.as_value_ref() as u64, true)
                .into(),
        ],
        "",
    )?;
    Ok(())
}

pub fn build_cov_init<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
) -> Result<FunctionValue<'ctx>> {
    // __cov_init() - initialize coverage
    let init_func_typ = context.void_type().fn_type(&[], false);
    let init_fn = module.add_function(COV_INIT, init_func_typ, None);

    // Create a constructor function to initialize coverage
    let constructor = module.add_function(COV_MODULE_INIT, init_func_typ, None);
    let entry = context.append_basic_block(constructor, COV_INIT_ENTRY);
    builder.position_at_end(entry);
    builder.build_call(init_fn, &[], "")?;
    builder.build_return(None)?;
    Ok(constructor)
}

pub fn build_fuzzer_init<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
) -> Result<FunctionValue<'ctx>> {
    // __fuzzer_forkserver_init() - initialize fuzzer
    let init_func_typ = context.void_type().fn_type(&[], false);
    let init_fn = module.add_function(FUZZER_FORKSERVER_INIT, init_func_typ, None);

    // Create a constructor function to initialize fuzzer
    let constructor = module.add_function(FUZZER_MODULE_INIT, init_func_typ, None);
    let entry = context.append_basic_block(constructor, FUZZER_INIT_ENTRY);
    builder.position_at_end(entry);
    builder.build_call(init_fn, &[], "")?;
    builder.build_return(None)?;
    Ok(constructor)
}

pub fn build_symbolic_init<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    serialized_constraints: Vec<Vec<ConstraintSerialized>>,
) -> Result<FunctionValue<'ctx>> {
    // __symbolic_init(i8, i64, i8, i64, u8) - initialize symbolic executor
    let init_func_typ = context.void_type().fn_type(
        &[
            context.i64_type().into(),
            context.i8_type().into(),
            context.i64_type().into(),
            context.i8_type().into(),
            context.i64_type().into(),
            context.i8_type().into(),
        ],
        false,
    );
    let init_fn = module.add_function(SYMBOLIC_MODULE_ADD_SYM, init_func_typ, None);

    // Create a constructor function to initialize symbolic executor
    let constructor = module.add_function(SYMBOLIC_INIT, init_func_typ, None);
    let entry = context.append_basic_block(constructor, SYMBOLIC_INIT_ENTRY);
    builder.position_at_end(entry);
    for serialized_constraint in serialized_constraints {
        for constraint in serialized_constraint {
            builder.build_call(
                init_fn,
                &[
                    context
                        .i64_type()
                        .const_int(constraint.id as u64, true)
                        .into(),
                    context
                        .i8_type()
                        .const_int(constraint.left_operand_kind as u64, true)
                        .into(),
                    context
                        .i64_type()
                        .const_int(constraint.left_operand_val as u64, true)
                        .into(),
                    context
                        .i8_type()
                        .const_int(constraint.right_operand_kind as u64, true)
                        .into(),
                    context
                        .i64_type()
                        .const_int(constraint.right_operand_val as u64, true)
                        .into(),
                    context
                        .i8_type()
                        .const_int(constraint.predicate as u64, true)
                        .into(),
                ],
                "",
            )?;
        }
    }
    builder.build_return(None)?;
    Ok(constructor)
}

pub fn build_race_init<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    init_lock_candidate_set: &HashMap<PointerValue<'ctx>, Vec<Lock<'ctx>>>,
) -> Result<FunctionValue<'ctx>> {
    // __race_module_init() - initialize race module
    let init_func_typ = context.void_type().fn_type(&[], false);
    let init_func_typ_global_var = context.void_type().fn_type(
        &[
            // 1. global var name
            context.ptr_type(AddressSpace::default()).into(),
            // 2. global var decl line
            context.i64_type().into(),
            // 3. global var address
            context.i64_type().into(),
        ],
        false,
    );
    let init_func_typ_lock_var = context.void_type().fn_type(
        &[
            // 1. global var address
            context.i64_type().into(),
            // 2. lock var name
            context.ptr_type(AddressSpace::default()).into(),
            // 3. lock decl line
            context.i64_type().into(),
            // 4. lock address
            context.i64_type().into(),
        ],
        false,
    );

    let init_fn_global_var = module.add_function(
        RACE_INIT_CANDIDATE_LOCKSET_GLOBAL_VAR,
        init_func_typ_global_var,
        None,
    );
    let init_fn_lock_var = module.add_function(
        RACE_INIT_CANDIDATE_LOCKSET_LOCK_VAR,
        init_func_typ_lock_var,
        None,
    );

    // Create a constructor function to initialize race module
    let constructor = module.add_function(RACE_MODULE_INIT, init_func_typ, None);
    let entry = context.append_basic_block(constructor, RACE_INIT_ENTRY);
    builder.position_at_end(entry);

    for (global_var, locks) in init_lock_candidate_set {
        // initialize global variable
        let global_var_name = get_or_build_global_string_ptr(
            module,
            builder,
            &format!(
                "{}{}",
                RACE_GLOBAL_PREFIX,
                &cstr_to_str(global_var.get_name())
            ),
        )?;
        let (global_var_decl_line, _) = get_instr_loc(global_var);
        builder.build_call(
            init_fn_global_var,
            &[
                global_var_name.as_pointer_value().into(),
                convert_to_int_val(context, global_var_decl_line).into(),
                context
                    .i64_type()
                    .const_int(global_var.as_value_ref() as u64, true)
                    .into(),
            ],
            "",
        )?;

        // initialize lock variable
        for lock in locks {
            let lock_var_name = get_or_build_global_string_ptr(
                module,
                builder,
                &format!(
                    "{}{}",
                    RACE_GLOBAL_PREFIX,
                    &cstr_to_str(lock.lock.get_name())
                ),
            )?;
            let (lock_var_decl_line, _) = get_instr_loc(&lock.lock);
            builder.build_call(
                init_fn_lock_var,
                &[
                    context
                        .i64_type()
                        .const_int(global_var.as_value_ref() as u64, true)
                        .into(),
                    lock_var_name.as_pointer_value().into(),
                    convert_to_int_val(context, lock_var_decl_line).into(),
                    context
                        .i64_type()
                        .const_int(lock.lock.as_value_ref() as u64, true)
                        .into(),
                ],
                "",
            )?;
        }
    }
    builder.build_return(None)?;
    Ok(constructor)
}

pub fn build_ctros<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    constructor: FunctionValue<'ctx>,
) -> Result<()> {
    let priority = context.i32_type().const_int(u64::MAX, false); // Assign the
                                                                  // prioirty to
                                                                  // lowest value
    let cov_init_ptr = constructor.as_global_value().as_pointer_value();
    let null_ptr = context.ptr_type(AddressSpace::default()).const_null();

    // Create the constructor struct
    let ctor_typ = context.struct_type(
        &[
            context.i32_type().into(),
            cov_init_ptr.get_type().into(),
            null_ptr.get_type().into(),
        ],
        false,
    );
    let ctor_val =
        ctor_typ.const_named_struct(&[priority.into(), cov_init_ptr.into(), null_ptr.into()]);
    if let Some(global_ctors) = module.get_global(LLVM_GLOBAL_CTORS) {
        // If global_ctors already exists, get the current initializer
        if let Some(initializer) = global_ctors.get_initializer() {
            if let ArrayValue(init_arr) = initializer {
                let init_arr_ref = init_arr.as_value_ref();
                let mut ctors = vec![];
                unsafe {
                    for i in 0..LLVMGetNumOperands(init_arr_ref) {
                        let elem = LLVMGetAggregateElement(init_arr_ref, i.try_into()?);
                        ctors.push(StructValue::new(elem));
                    }
                    ctors.push(ctor_val);
                    global_ctors.delete();
                }
                let new_ctors_val = ctor_typ.const_array(&ctors);
                let new_global_ctors = module.add_global(
                    ctor_typ.array_type(ctors.len().try_into()?),
                    None,
                    LLVM_GLOBAL_CTORS,
                );
                new_global_ctors.set_initializer(&new_ctors_val);
                new_global_ctors.set_linkage(inkwell::module::Linkage::Appending);
            }
        }
    } else {
        // Create the global variable
        let new_global_ctors = module.add_global(ctor_typ.array_type(1), None, LLVM_GLOBAL_CTORS);
        let ctors_val = ctor_typ.const_array(&[ctor_val]);
        new_global_ctors.set_initializer(&ctors_val);
        new_global_ctors.set_linkage(inkwell::module::Linkage::Appending);
    }
    Ok(())
}

pub fn get_cov_init_last_instr<'ctx>(module: &Module<'ctx>) -> InstructionValue<'ctx> {
    let constructor = get_func(module, COV_MODULE_INIT).unwrap();
    constructor
        .get_last_basic_block()
        .unwrap()
        .get_terminator()
        .unwrap()
}

pub fn module_verify<'ctx>(module: &Module<'ctx>) -> Result<()> {
    match module.verify() {
        Err(_) => Err(anyhow::anyhow!("Module verification err")),
        _ => Ok(()),
    }
}

pub fn can_skip_instrument<'ctx>(func: &FunctionValue<'ctx>) -> bool {
    return func.count_basic_blocks() == 0
        || func.get_name().to_string_lossy().starts_with("__cov_")
        || func.get_name().to_string_lossy().starts_with("__asan_")
        || func.get_name().to_string_lossy().starts_with("__fuzzer_")
        || func.get_name().to_string_lossy().starts_with("__symbolic_")
        || func.get_name().to_string_lossy().starts_with("__race_");
}
