use crate::{
    inkwell_intrinsic::{
        build_ctros, build_fuzzer_init, build_trace_edge, can_skip_instrument, module_verify,
    },
    llvm_intrinsic::{cstr_to_str, get_instr_filename},
    module::InstrumentModule,
};
use anyhow::Result;
use inkwell::values::{InstructionOpcode, IntValue};
use inkwell::{builder::Builder, context::Context, module::Module};
use std::collections::HashSet;
use uuid::Uuid;

fn get_rand_value<'ctx>(context: &'ctx Context) -> IntValue<'ctx> {
    let (lower, _) = Uuid::new_v4().as_u64_pair();
    let i64_typ = context.i64_type();
    i64_typ.const_int(lower, false)
}

#[derive(Default)]
pub struct FuzzModule {}

impl InstrumentModule for FuzzModule {
    fn instrument<'ctx>(
        &self,
        context: &'ctx Context,
        module: &Module<'ctx>,
        builder: &Builder<'ctx>,
    ) -> Result<()> {
        let constructor = build_fuzzer_init(context, module, builder)?;
        build_ctros(context, module, constructor)?;

        let module_filename = cstr_to_str(module.get_source_file_name());
        let funcs: Vec<_> = module.get_functions().collect();
        for func in funcs {
            // Skip funcs without bodies or those we've added
            if can_skip_instrument(&func) {
                continue;
            }
            let mut instrumented_blks = HashSet::new();
            for basic_blk in func.get_basic_blocks() {
                if instrumented_blks.contains(&basic_blk) {
                    continue;
                }
                if let Some(first_instr) = basic_blk.get_first_instruction() {
                    let mut instrument_pos = first_instr;
                    match first_instr.get_opcode() {
                        InstructionOpcode::LandingPad | InstructionOpcode::Phi => {
                            if let Some(second_instr) = first_instr.get_next_instruction() {
                                instrument_pos = second_instr;
                            }
                        }
                        _ => {}
                    }
                    if let Some(instr_filename) = get_instr_filename(&instrument_pos) {
                        if instr_filename != module_filename {
                            continue;
                        }
                    }
                    builder.position_before(&instrument_pos);
                    build_trace_edge(context, module, builder, get_rand_value(context))?;
                }
                instrumented_blks.insert(basic_blk);
            }
        }
        // Verify instrumented IRs
        module_verify(module)
    }
}
