use anyhow::Result;
use tools::compile::{build, get_args};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (
        input_file,
        out_dir,
        out_bin,
        compiler,
        opt_level,
        coverage_runtime_lib_path,
        coverage_runtime_lib_name,
        asan_runtime_lib_path,
        asan_runtime_lib_name,
        fuzzer_runtime_lib_path,
        fuzzer_runtime_lib_name,
        symbolic_runtime_lib_path,
        symbolic_runtime_lib_name,
        race_runtime_lib_path,
        race_runtime_lib_name,
    ) = get_args()?;
    build(
        &input_file,
        &out_dir,
        &out_bin,
        &compiler,
        &opt_level,
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
    )
}
