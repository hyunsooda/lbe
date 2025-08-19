use crate::inkwell_intrinsic::*;
use crate::llvm_intrinsic::*;
use crate::module::InstrumentModule;
use crate::names::*;
use anyhow::Result;
use inkwell::{builder::Builder, context::Context, module::Module, values::InstructionOpcode};
use std::collections::{BTreeSet, HashSet};

#[derive(Default)]
pub struct CoverageModule {}

impl InstrumentModule for CoverageModule {
    fn instrument<'ctx>(
        &self,
        context: &'ctx Context,
        module: &Module<'ctx>,
        builder: &Builder<'ctx>,
    ) -> Result<()> {
        let constructor = build_cov_init(context, module, builder)?;
        build_ctros(context, module, constructor)?;

        let module_filename = cstr_to_str(module.get_source_file_name());
        let funcs: Vec<_> = module.get_functions().collect();
        let mut filename_str_ptr = None;
        let mut funcs_loc = BTreeSet::new();
        let mut brs_loc = vec![];
        let mut lines_loc = BTreeSet::new();
        // Now instrument each function
        for func in funcs {
            // Skip funcs without bodies or those we've added
            if can_skip_instrument(&func) {
                continue;
            }
            set_filename(module, builder, &mut filename_str_ptr, &func)?;
            let mut func_loc_inserted = false;

            // Track basic blocks we've instrumented in this function
            let mut instrumented_blks = HashSet::new();
            // Get basic blocks and instrument them
            for basic_blk in func.get_basic_blocks() {
                if instrumented_blks.contains(&basic_blk) {
                    continue;
                }
                if let Some(first_instr) = basic_blk.get_first_instruction() {
                    let mut instrument_pos = first_instr;
                    // `landingpad` instruction must be located at first, thus we avoid inserting
                    // before it
                    if first_instr.get_opcode() == InstructionOpcode::LandingPad {
                        if let Some(second_instr) = first_instr.get_next_instruction() {
                            instrument_pos = second_instr;
                        }
                    }
                    builder.position_before(&instrument_pos);
                } else {
                    continue;
                }
                if let Some(instr_filename) = get_instr_filename(&func) {
                    if instr_filename != module_filename {
                        continue;
                    }
                }
                let mut lines = BTreeSet::new();
                if !func_loc_inserted {
                    let func_loc = get_instr_loc(&func).0;
                    lines_loc.insert(func_loc);
                    lines.insert(func_loc);
                    funcs_loc.insert(func_loc);
                    func_loc_inserted = true;
                }
                for instr in basic_blk.get_instructions() {
                    if let Some(instr_filename) = get_instr_filename(&instr) {
                        if instr_filename != module_filename {
                            continue;
                        }
                    }
                    let (line, _) = get_instr_loc(&instr);
                    lines.insert(line);
                    lines_loc.insert(line);
                    record_br(&mut brs_loc, &instr);
                    record_switch(&mut brs_loc, &instr);
                }
                let lines_len = lines.len();
                if lines_len > 0 {
                    let arr_ptr = build_i32_static_arr(
                        context,
                        builder,
                        &lines.into_iter().collect(),
                        COV_HIT_LINES_ARR,
                        COV_HIT_LINES_ARR_PTR,
                    )?;
                    build_cov_hit_batch(
                        context,
                        module,
                        builder,
                        &filename_str_ptr.unwrap(),
                        arr_ptr,
                        lines_len,
                    )?;
                    instrumented_blks.insert(basic_blk);
                }
            }
        }
        // install source mapping record into `__cov_init`
        let init_last_instr = get_cov_init_last_instr(module);
        builder.position_before(&init_last_instr);
        build_src_mapping_call(
            context,
            module,
            builder,
            &filename_str_ptr.unwrap(),
            &funcs_loc,
            &brs_loc,
            &lines_loc,
        )?;
        // Verify instrumented IRs
        module_verify(module)
    }
}
