use crate::state::{
    init_candidate_lockset_global_var, init_candidate_lockset_lock_var, state_transition,
    update_lock_held, update_shared_mem,
};

#[no_mangle]
pub extern "C" fn __race_init_candidate_lockset_global_var(
    global_var_name: *const libc::c_char,
    global_var_decl_line: u32,
    global_var_id: i64,
) {
    init_candidate_lockset_global_var(global_var_name, global_var_decl_line, global_var_id);
}

#[no_mangle]
pub extern "C" fn __race_init_candidate_lockset_lock_var(
    global_var_id: i64,
    lock_var_name: *const libc::c_char,
    lock_var_decl_line: u32,
    lock_id: i64,
) {
    init_candidate_lockset_lock_var(global_var_id, lock_var_name, lock_var_decl_line, lock_id);
}

#[no_mangle]
pub extern "C" fn __race_update_lock_held(is_lock: i8, thread_id: i64, lock_id: i64) {
    update_lock_held(is_lock, thread_id, lock_id);
}

#[no_mangle]
pub extern "C" fn __race_update_shared_mem(
    is_write: i8,
    thread_id: i64,
    global_var_id: i64,
    line: i64,
) {
    update_shared_mem(thread_id, global_var_id);
    state_transition(is_write, thread_id, global_var_id, line);
}
