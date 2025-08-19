use coverage_runtime::coverage_internal::{cov_clear, make_cov};
use coverage_runtime::coverage_runtime::*;
use defer_lite::defer;
use serial_test::serial;
use std::ffi::CString;

#[test]
#[serial]
fn test_line_hit() {
    defer! { cov_clear(); };

    let cstr = CString::new("test.c").unwrap();
    let file_ptr = cstr.as_ptr();
    let lines = [10, 20, 30];
    __cov_hit_batch(file_ptr, lines.as_ptr(), lines.len());

    let mut bitmap = vec![0u64; lines.len()];
    let bitmap_ptr = bitmap.as_mut_ptr();
    assert_eq!(bitmap, vec![0, 0, 0]);

    __cov_get_hit_map(bitmap_ptr, lines.len());
    assert_eq!(bitmap, vec![1, 1, 1]);
}

#[test]
#[serial]
fn test_cov_report() {
    defer! { cov_clear(); };

    let cstr = CString::new("test.c").unwrap();
    let file_ptr = cstr.as_ptr();
    let hit_lines = [5, 7, 9, 10, 12];
    __cov_hit_batch(file_ptr, hit_lines.as_ptr(), hit_lines.len());

    let mut bitmap = vec![0u64; hit_lines.len()];
    let bitmap_ptr = bitmap.as_mut_ptr();
    __cov_get_hit_map(bitmap_ptr, hit_lines.len());

    let (funcs_lines, brs_lines, lines_lines) = (vec![5, 20], vec![11, 12], vec![5, 7, 8, 9, 10]);
    let (funcs_lines_ptr, brs_lines_ptr, lines_lines_ptr) = (
        funcs_lines.as_ptr(),
        brs_lines.as_ptr(),
        lines_lines.as_ptr(),
    );
    __cov_mapping_src(
        file_ptr,
        funcs_lines_ptr,
        funcs_lines.len(),
        brs_lines_ptr,
        brs_lines.len(),
        lines_lines_ptr,
        lines_lines.len(),
    );

    let cov_report = &make_cov()[0];
    assert_eq!(cov_report.funcs_hit_ratio, "50.00");
    assert_eq!(cov_report.brs_hit_ratio, "50.00");
    assert_eq!(cov_report.lines_hit_ratio, "80.00");
    assert_eq!(cov_report.uncovered_funcs, "20");
    assert_eq!(cov_report.uncovered_brs, "11(12:F)");
    assert_eq!(cov_report.uncovered_lines, "8,12");
}
