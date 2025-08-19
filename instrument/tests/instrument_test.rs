use inkwell::values::InstructionOpcode::Call;
use inkwell::{context::Context, values::CallSiteValue};
use instrument::{
    coverage::CoverageModule,
    module::instrument,
    names::{COV_HIT_BATCH, COV_MAPPING_SRC},
};
use instrument::{llvm_intrinsic::cstr_to_str, names::COV_INIT_ENTRY};
mod util;

#[test]
fn test_instrument_coverage() {
    let src = r#"
    #include <stdio.h>

    int add(int a, int b) {
        return a + b;
    }

    int main() {
        int a = 1;
        int b = 2;
        add(a, b);
    }
"#;

    let mem_buf = util::load_ir(&src);
    let context = Context::create();
    let module = context.create_module_from_ir(mem_buf).unwrap();
    let builder = context.create_builder();
    let cov_module = CoverageModule::default();
    instrument(&cov_module, &context, &module, &builder).unwrap();

    let mut src_mapping_call_found = 0;
    for func in module.get_functions() {
        for basic_blk in func.get_basic_blocks() {
            let mut cov_hit_batch_found = 0;
            for instr in basic_blk.get_instructions() {
                if instr.get_opcode() == Call {
                    if let Ok(callsite) = TryInto::<CallSiteValue>::try_into(instr) {
                        let fn_val = cstr_to_str(callsite.get_called_fn_value().get_name());
                        match fn_val.as_str() {
                            COV_HIT_BATCH => cov_hit_batch_found += 1,
                            COV_MAPPING_SRC => src_mapping_call_found += 1,
                            _ => {}
                        };
                    }
                }
            }
            // coverage hit   : one instrumentation should exist per basic block
            if cstr_to_str(basic_blk.get_name()) != COV_INIT_ENTRY {
                assert_eq!(cov_hit_batch_found, 1);
            }
        }
    }
    // source mapping : one instrumentation should exist per single file
    assert_eq!(src_mapping_call_found, 1);
}
