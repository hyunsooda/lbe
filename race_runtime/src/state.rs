use colored::Colorize;
use instrument::{
    names::RACE_GLOBAL_PREFIX,
    race::{AccessOperation, LockTyp},
};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    env,
    sync::{Arc, Mutex},
};

const RACE_TEST_ENABLED: &str = "RACE_UNIT_TEST_ENABLED";

pub struct GlobalVarMetadata {
    global_var_name: String,
    global_var_decl_line: u32,
}
impl GlobalVarMetadata {
    fn new(global_var_name: String, global_var_decl_line: u32) -> Self {
        Self {
            global_var_name,
            global_var_decl_line,
        }
    }
}

pub struct LockMetadata {
    lock_var_name: String,
    lock_var_decl_line: u32,
}
impl LockMetadata {
    fn new(lock_var_name: String, lock_var_decl_line: u32) -> Self {
        Self {
            lock_var_name,
            lock_var_decl_line,
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
pub struct Reported {
    thread_id: i64,
    global_var_id: i64,
    global_var_decl: u32,
    global_var_used: i64,
}
impl Reported {
    fn new(thread_id: i64, global_var_id: i64, global_var_decl: u32, global_var_used: i64) -> Self {
        Self {
            thread_id,
            global_var_id,
            global_var_decl,
            global_var_used,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum State {
    Virgin,
    Exclusive,
    Shared,
    SharedModified,
}

pub struct ThreadState {
    thread_ids: HashSet<i64>,
    state: State,
}
impl Default for ThreadState {
    fn default() -> Self {
        Self {
            thread_ids: HashSet::new(),
            state: State::Virgin,
        }
    }
}

lazy_static::lazy_static! {
    // <global var id, metadata>
    pub static ref global_var_metadata: Arc<Mutex<HashMap<i64, GlobalVarMetadata>>> = Arc::new(Mutex::new(HashMap::new()));
    // <lock  id, metadata>
    pub static ref lock_metadata: Arc<Mutex<HashMap<i64, LockMetadata>>> = Arc::new(Mutex::new(HashMap::new()));
    // <global var id, lockset>
    pub static ref init_lock_set: Arc<Mutex<HashMap<i64, BTreeSet<i64>>>> = Arc::new(Mutex::new(HashMap::new()));
    pub static ref lock_set: Arc<Mutex<HashMap<i64, BTreeSet<i64>>>> = Arc::new(Mutex::new(HashMap::new()));
    // <thread id, lockset>
    pub static ref lock_held: Arc<Mutex<HashMap<i64, BTreeSet<i64>>>> = Arc::new(Mutex::new(HashMap::new()));
    // <global var id, state>
    pub static ref state: Arc<Mutex<HashMap<i64, ThreadState>>> = Arc::new(Mutex::new(HashMap::new()));
    // <set>
    pub static ref reported: Arc<Mutex<HashSet<Reported>>> = Arc::new(Mutex::new(HashSet::new()));
}

fn cstr_to_string(ptr: *const libc::c_char) -> String {
    if ptr.is_null() {
        return "".to_string();
    }
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}
pub fn init_candidate_lockset_lock_var(
    global_var_id: i64,
    lock_var_name: *const libc::c_char,
    lock_var_decl_line: u32,
    lock_id: i64,
) {
    if lock_var_name.is_null() {
        return;
    }

    let lock_var_name = cstr_to_string(lock_var_name)
        .strip_prefix(RACE_GLOBAL_PREFIX)
        .unwrap()
        .to_string();
    lock_metadata.lock().unwrap().insert(
        lock_id,
        LockMetadata::new(lock_var_name, lock_var_decl_line),
    );
    lock_set
        .lock()
        .unwrap()
        .entry(global_var_id)
        .or_insert_with(BTreeSet::new)
        .insert(lock_id);

    init_lock_set
        .lock()
        .unwrap()
        .entry(global_var_id)
        .or_insert_with(BTreeSet::new)
        .insert(lock_id);
}

pub fn init_candidate_lockset_global_var(
    global_var_name: *const libc::c_char,
    global_var_decl_line: u32,
    global_var_id: i64,
) {
    if global_var_name.is_null() {
        return;
    }

    let global_var_name = cstr_to_string(global_var_name)
        .strip_prefix(RACE_GLOBAL_PREFIX)
        .unwrap()
        .to_string();
    global_var_metadata.lock().unwrap().insert(
        global_var_id,
        GlobalVarMetadata::new(global_var_name, global_var_decl_line),
    );
    // set init state
    // as we only consider global variables for target shared memory,
    // directly set `Virgin` state initially
    init_state(global_var_id);
}

pub fn update_lock_held(is_lock: i8, thread_id: i64, lock_id: i64) {
    if let Some(lock_typ) = LockTyp::from_i8(is_lock) {
        match lock_typ {
            LockTyp::Lock => {
                lock_held
                    .lock()
                    .unwrap()
                    .entry(thread_id)
                    .or_insert_with(BTreeSet::new)
                    .insert(lock_id);
            }
            LockTyp::UnLock => {
                if let Some(set) = lock_held.lock().unwrap().get_mut(&thread_id) {
                    set.remove(&lock_id);
                }
            }
        }
    }
}

pub fn update_shared_mem(thread_id: i64, global_var_id: i64) {
    // calc. C(v) = C(v) ∩ lock_held(t)
    match lock_held.lock().unwrap().get(&thread_id) {
        Some(holds) => {
            let mut new_lockset = BTreeSet::new();
            if let Some(set) = lock_set.lock().unwrap().get(&global_var_id) {
                for l in set {
                    if holds.contains(l) {
                        new_lockset.insert(*l);
                    }
                }
            }
            lock_set.lock().unwrap().insert(global_var_id, new_lockset);
        }
        None => {
            lock_set
                .lock()
                .unwrap()
                .insert(global_var_id, BTreeSet::new());
        }
    }
}

fn init_state(global_var_id: i64) {
    state
        .lock()
        .unwrap()
        .insert(global_var_id, ThreadState::default());
}

pub fn state_transition(is_write: i8, thread_id: i64, global_var_id: i64, line: i64) {
    if let Some(access_op) = AccessOperation::from_i8(is_write) {
        let mut s = state.lock().unwrap();
        match access_op {
            AccessOperation::Write => {
                if let Some(cur_state) = s.get_mut(&global_var_id) {
                    match cur_state.state {
                        State::Virgin => {
                            cur_state.state = State::Exclusive;
                            cur_state.thread_ids.insert(thread_id);
                        }
                        State::Exclusive => {
                            if !cur_state.thread_ids.contains(&thread_id) {
                                cur_state.state = State::SharedModified;
                                cur_state.thread_ids.insert(thread_id);
                            }
                        }
                        State::Shared => {
                            cur_state.state = State::SharedModified;
                            cur_state.thread_ids.insert(thread_id);
                        }
                        State::SharedModified => {
                            if let Some(set) = lock_set.lock().unwrap().get(&global_var_id) {
                                if set.is_empty() {
                                    report(thread_id, global_var_id, line);
                                }
                            }
                        }
                    }
                }
            }
            AccessOperation::Read => {
                if let Some(cur_state) = s.get_mut(&global_var_id) {
                    if cur_state.state == State::Exclusive {
                        if !cur_state.thread_ids.contains(&thread_id) {
                            cur_state.state = State::Shared;
                            cur_state.thread_ids.insert(thread_id);
                        }
                    }
                }
            }
        }
    }
}

pub fn report(thread_id: i64, global_var_id: i64, line: i64) {
    let gv_md = global_var_metadata.lock().unwrap();
    let md = gv_md.get(&global_var_id).unwrap();
    let report = Reported::new(thread_id, global_var_id, md.global_var_decl_line, line);

    let mut r = reported.lock().unwrap();
    if r.get(&report).is_none() {
        println!(
            "{}",
            format!(
                "[--------------------- Data race detected #{} ---------------------]",
                r.len()
            )
            .red()
            .bold()
        );
        if !is_test_enabled() {
            println!("thread id          = {}", thread_id);
        }
        println!("variable name      = {}", md.global_var_name);
        println!("variable decl      = {}", md.global_var_decl_line);
        println!("variable used line = {}", line);
        println!("[related locks]");

        if let Some(set) = init_lock_set.lock().unwrap().get(&global_var_id) {
            for lock_id in set {
                let lk_md = lock_metadata.lock().unwrap();
                let md = lk_md.get(lock_id).unwrap();
                println!("    - lock variable name = {}", md.lock_var_name);
                println!("    - lock variable decl = {}", md.lock_var_decl_line);
            }
        }
        println!("");
        r.insert(report);
    }
}

fn is_test_enabled() -> bool {
    matches!(env::var(RACE_TEST_ENABLED), Ok(val) if val == "1")
}
