use instrument::names::RACE_GLOBAL_PREFIX;
use instrument::race::AccessOperation;
use race_runtime::runtime::{__race_init_candidate_lockset_global_var, __race_update_shared_mem};
use race_runtime::state::reported;
use std::ffi::CString;

#[test]
fn test_race() {
    let s = CString::new(format!("{}.v", RACE_GLOBAL_PREFIX)).unwrap();
    let global_var_name: *const libc::c_char = s.as_ptr();
    let global_var_decl: u32 = 100;
    let global_var_id: i64 = 200;
    __race_init_candidate_lockset_global_var(global_var_name, global_var_decl, global_var_id);

    let write = AccessOperation::Write;
    let thread_id1 = 300;
    let thread_id2 = 400;
    let line = 500;
    __race_update_shared_mem(write.i8(), thread_id1, global_var_id, line); // virgin -> exclusive
    __race_update_shared_mem(write.i8(), thread_id2, global_var_id, line); // exclusive ->
                                                                           // shared-modified
    assert!(reported.lock().unwrap().len() == 0);
    __race_update_shared_mem(write.i8(), thread_id2, global_var_id, line); // reported
    assert!(reported.lock().unwrap().len() == 1);
}
