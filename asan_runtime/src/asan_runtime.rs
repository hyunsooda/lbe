use crate::asan_hook::{
    convert_to_shadow_idx, CLEAN_BYTE_MARKER, MALLOC_REENTERED, SHADOW_MEMORY, SHADOW_SIZE,
};
use std::backtrace::Backtrace;
use std::env;

pub fn cstr_to_string(file_ptr: *const libc::c_char) -> String {
    if file_ptr.is_null() {
        return "".to_string();
    }
    unsafe {
        std::ffi::CStr::from_ptr(file_ptr)
            .to_string_lossy()
            .into_owned()
    }
}
const ASAN_TEST_ENABLED: &str = "ASAN_UNIT_TEST_ENABLED";
const EXIT_CODE: i32 = 99;

#[no_mangle]
pub extern "C" fn __asan_mem_check(file_ptr: *const libc::c_char, addr: usize, access_size: usize) {
    if file_ptr.is_null() || addr == 0 {
        return;
    }

    let filename = cstr_to_string(file_ptr);
    let shadow_idx = convert_to_shadow_idx(addr);
    let shadow_val = SHADOW_MEMORY.lock().unwrap()[shadow_idx % SHADOW_SIZE];
    if shadow_val != CLEAN_BYTE_MARKER && ((addr & 0x07) + access_size) as i8 > shadow_val {
        report_asan_violated(&filename, addr);
    }
}

fn report_asan_violated(filename: &str, addr: usize) {
    if is_test_enabled() {
        eprintln!("[ASAN] invalid memory access detected at {}", filename);
    } else {
        eprintln!(
            "[ASAN] invalid memory access detected at {}: 0x{:x}",
            filename, addr
        );
    }
    // print backtrace
    MALLOC_REENTERED.with(|re_enter| {
        *re_enter.lock().unwrap() = true;
        let bt = Backtrace::force_capture();
        if is_test_enabled() {
            eprintln!("{}", trim_runtime_bt(bt.to_string()));
        } else {
            eprintln!("{bt}");
        }
        *re_enter.lock().unwrap() = false;
    });
    if !is_test_enabled() {
        unsafe {
            libc::_exit(EXIT_CODE);
        }
    }
}

fn is_test_enabled() -> bool {
    matches!(env::var(ASAN_TEST_ENABLED), Ok(val) if val == "1")
}

fn trim_runtime_bt(bt: String) -> String {
    bt.lines()
        .skip_while(|line| !line.contains("__asan_mem_check"))
        // remove filepath line from `strcpy` test case
        .filter(|line| !line.contains("asan_hook.rs"))
        .skip(2)
        .collect::<Vec<_>>()
        .join("\n")
}
