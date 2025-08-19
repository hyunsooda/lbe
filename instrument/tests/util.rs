use defer_lite::defer;
use inkwell::memory_buffer::MemoryBuffer;
use instrument::llvm_intrinsic::read_ll;
use rand::{distr::Alphanumeric, Rng};
use std::fs;
use std::process::Command;

pub fn get_rand_filename() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(20) // You can adjust the length of the random string
        .map(char::from)
        .collect()
}

pub fn load_ir(code: &str) -> MemoryBuffer {
    // Write C code to a temporary file
    let c_file_path = format!("{}.c", get_rand_filename());
    let ir_file_path = format!("{}.ll", get_rand_filename());
    defer! {
        fs::remove_file(&c_file_path).unwrap();
        fs::remove_file(&ir_file_path).unwrap();
    };
    fs::write(&c_file_path, code).expect("Failed to write C file");

    // Compile C code to LLVM IR
    let output = Command::new("clang")
        .args([
            "-emit-llvm",
            "-g",
            "-O0",
            "-S",
            "-o",
            &ir_file_path,
            &c_file_path,
        ])
        .output()
        .expect("Failed to compile C to LLVM IR");
    assert!(
        output.status.success(),
        "Clang failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    read_ll(&ir_file_path).unwrap()
}
