use anyhow::Result;
use inkwell::{builder::Builder, context::Context, module::Module};

use crate::{
    asan::ASANModule, coverage::CoverageModule, fuzz::FuzzModule, race::RaceModule,
    symbolic::SymbolicModule,
};

pub trait InstrumentModule {
    fn instrument<'ctx>(
        &self,
        context: &'ctx Context,
        module: &Module<'ctx>,
        builder: &Builder<'ctx>,
    ) -> Result<()>;
}

pub fn instrument<'ctx, M>(
    m: &M,
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
) -> Result<()>
where
    M: InstrumentModule,
{
    m.instrument(context, module, builder)
}

pub fn instrument_all<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
) -> Result<()> {
    let asan_module = ASANModule::default();
    let fuzz_module = FuzzModule::default();
    let cov_module = CoverageModule::default();
    let symbolic_module = SymbolicModule::default();
    let race_module = RaceModule::default();
    instrument(&race_module, &context, &module, &builder)?;
    instrument(&symbolic_module, &context, &module, &builder)?;
    instrument(&fuzz_module, &context, &module, &builder)?;
    instrument(&asan_module, &context, &module, &builder)?;
    instrument(&cov_module, &context, &module, &builder)?;
    Ok(())
}
