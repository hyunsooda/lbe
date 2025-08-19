use crate::coverage::EDGE_COVERAGE;

#[no_mangle]
pub extern "C" fn __fuzzer_trace_edge(cur_loc: i64) {
    EDGE_COVERAGE.lock().unwrap().trace_edge(cur_loc);
}

/// this exists for unit test
pub fn __init() {
    EDGE_COVERAGE.lock().unwrap().init();
}

#[no_mangle]
pub extern "C" fn __fuzzer_forkserver_init() {
    EDGE_COVERAGE.lock().unwrap().read_wakeup();
}
