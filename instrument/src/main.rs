use anyhow::Result;
use inkwell::context::Context;
use instrument::{
    asan::ASANModule,
    cli::{get_args, ModuleTyp},
    coverage::CoverageModule,
    fuzz::FuzzModule,
    llvm_intrinsic::read_ll,
    module::{instrument, instrument_all},
    race::RaceModule,
    symbolic::SymbolicModule,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (input_filename, output_filename, module_typ) = get_args()?;
    let mem_buf = read_ll(&input_filename)?;
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module_from_ir(mem_buf)?;

    match module_typ {
        ModuleTyp::Coverage => {
            let cov_module = CoverageModule::default();
            instrument(&cov_module, &context, &module, &builder)?;
        }
        ModuleTyp::Asan => {
            let asan_module = ASANModule::default();
            instrument(&asan_module, &context, &module, &builder)?;
        }
        ModuleTyp::Fuzz => {
            let fuzz_module = FuzzModule::default();
            instrument(&fuzz_module, &context, &module, &builder)?;
        }
        ModuleTyp::Symbolic => {
            let symbolic_module = SymbolicModule::default();
            instrument(&symbolic_module, &context, &module, &builder)?;
        }
        ModuleTyp::Race => {
            let race_module = RaceModule::default();
            instrument(&race_module, &context, &module, &builder)?;
        }
        ModuleTyp::All => {
            instrument_all(&context, &module, &builder)?;
        }
    }

    module.print_to_file(&output_filename)?;
    Ok(())
}
