use crate::{
    cli::CLI,
    util::{create_dir, extract_filename},
};
use anyhow::{Context as AnyhowContext, Result};
use inkwell::context::Context;
use instrument::{llvm_intrinsic::read_ll, module::instrument_all};
use std::process::Command;

pub fn get_args() -> Result<(
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
)> {
    let cli = CLI::new();
    cli.get_args()
}

fn compile_to_ir(file: &str, compiler: &str, opt_level: &str, out_dir: &str) -> Result<String> {
    let output_file = format!("{}/{}.ll", out_dir, extract_filename(file));
    let status = Command::new(compiler)
        .arg("-Wno-everything")
        .arg(format!("-{}", opt_level))
        .arg("-g")
        .arg("-S")
        .arg("-emit-llvm")
        .arg(file)
        .arg("-o")
        .arg(&output_file)
        .status()
        .context("Failed to run clang")?;
    if !status.success() {
        anyhow::bail!("clang failed with status: {}", status);
    }
    Ok(output_file)
}

fn instrument(ir_file: &str, out_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mem_buf = read_ll(&ir_file)?;
    let context = Context::create();
    let module = context.create_module_from_ir(mem_buf)?;
    let builder = context.create_builder();
    instrument_all(&context, &module, &builder)?;
    let output_file = format!("{}/instrumented_{}", out_dir, extract_filename(ir_file));
    module.print_to_file(&output_file)?;
    Ok(output_file)
}

fn compile_to_bin(
    instrumented_file: &str,
    compiler: &str,
    coverage_runtime_lib_path: &str,
    coverage_runtime_lib_name: &str,
    asan_runtime_lib_path: &str,
    asan_runtime_lib_name: &str,
    fuzzer_runtime_lib_path: &str,
    fuzzer_runtime_lib_name: &str,
    symbolic_runtime_lib_path: &str,
    symbolic_runtime_lib_name: &str,
    race_runtime_lib_path: &str,
    race_runtime_lib_name: &str,
    out_dir: &str,
    out_bin: &str,
) -> Result<()> {
    let output_path = format!("{}/{}", out_dir, out_bin);
    let coverage_runtime_lib_name = coverage_runtime_lib_name
        .strip_suffix(".so")
        .unwrap_or(coverage_runtime_lib_name);
    let fuzzer_runtime_lib_name = fuzzer_runtime_lib_name
        .strip_suffix(".so")
        .unwrap_or(fuzzer_runtime_lib_name);
    let status = Command::new(compiler)
        .arg("-Wno-everything")
        .arg(instrumented_file)
        .arg(format!("-L{}", coverage_runtime_lib_path))
        .arg(format!("-l{}", coverage_runtime_lib_name))
        .arg(format!("-L{}", asan_runtime_lib_path))
        .arg(format!("-l{}", asan_runtime_lib_name))
        .arg(format!("-L{}", fuzzer_runtime_lib_path))
        .arg(format!("-l{}", fuzzer_runtime_lib_name))
        .arg(format!("-L{}", symbolic_runtime_lib_path))
        .arg(format!("-l{}", symbolic_runtime_lib_name))
        .arg(format!("-L{}", race_runtime_lib_path))
        .arg(format!("-l{}", race_runtime_lib_name))
        .arg("-o")
        .arg(output_path)
        .status()
        .expect("Failed to execute clang");
    if !status.success() {
        anyhow::bail!("clang failed with status: {}", status);
    }
    Ok(())
}

pub fn build(
    input_file: &str,
    out_dir: &str,
    out_bin: &str,
    compiler: &str,
    opt_level: &str,
    coverage_runtime_lib_path: &str,
    coverage_runtime_lib_name: &str,
    asan_runtime_lib_path: &str,
    asan_runtime_lib_name: &str,
    fuzzer_runtime_lib_path: &str,
    fuzzer_runtime_lib_name: &str,
    symbolic_runtime_lib_path: &str,
    symbolic_runtime_lib_name: &str,
    race_runtime_lib_path: &str,
    race_runtime_lib_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    create_dir(out_dir)?;
    let ir_file = compile_to_ir(&input_file, &compiler, &opt_level, &out_dir)?;
    println!("[+] compiled to IR ({})", ir_file);
    let instrumented_file = instrument(&ir_file, &out_dir)?;
    println!("[+] IR file instrumented ({})", instrumented_file);
    compile_to_bin(
        &instrumented_file,
        &compiler,
        &coverage_runtime_lib_path,
        &coverage_runtime_lib_name,
        &asan_runtime_lib_path,
        &asan_runtime_lib_name,
        &fuzzer_runtime_lib_path,
        &fuzzer_runtime_lib_name,
        &symbolic_runtime_lib_path,
        &symbolic_runtime_lib_name,
        &race_runtime_lib_path,
        &race_runtime_lib_name,
        &out_dir,
        &out_bin,
    )?;
    println!("[+] Binary created ({})", out_bin);
    println!(
        "[+] You can run LD_LIBRARY_PATH={} ./{}/{} ",
        coverage_runtime_lib_path, out_dir, out_bin
    );
    Ok(())
}
