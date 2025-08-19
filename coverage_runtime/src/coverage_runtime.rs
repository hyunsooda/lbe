use crate::{coverage_internal::*, mmap::init_shm, util::cstr_to_string};
use memmap2::MmapMut;
use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};

const COVERAGE_ENABLED: &str = "COVERAGE_ENABLED";
const COLOR_ENABLED: &str = "COLOR";

#[derive(Debug, Clone)]
pub struct SourceMapping {
    pub lines: Vec<u32>,
    pub brs: Vec<u32>,
    pub funcs: Vec<u32>,
}

// Represents a source location (file:line)
#[derive(Debug, Hash, Eq, PartialEq, Clone, PartialOrd, Ord)]
pub struct LineMapping {
    pub file: String,
    pub line: u32,
}

// Global coverage state
pub struct CoverageState {
    pub source_map: Mutex<HashMap<String, SourceMapping>>,
    pub location_map: Mutex<HashMap<LineMapping, usize>>,
    pub lines: Vec<AtomicUsize>,
    pub enabled: AtomicUsize,
    pub color: AtomicUsize,
}

// Global state (initialized lazily)
lazy_static::lazy_static! {
    pub static ref COVERAGE_STATE: Arc<RwLock<CoverageState>> = Arc::new(RwLock::new(CoverageState {
        source_map: Mutex::new(HashMap::new()),
        location_map: Mutex::new(HashMap::new()),
        lines: Vec::new(),
        enabled: AtomicUsize::new(1), // Enabled by default
        color: AtomicUsize::new(1),
    }));
}

pub struct CovMap {
    pub mem: Option<MmapMut>,
    pub size: Option<usize>,
}

lazy_static::lazy_static! {
    pub static ref SHM_COV: Arc<RwLock<CovMap>> = Arc::new(RwLock::new(CovMap{mem: None, size: None}));
}

pub fn init_cov_shm() {
    let mut shm_cov = SHM_COV.write().unwrap();
    let (shm, size) = init_shm("SHM_COV_ID", "SHM_COV_SIZE");
    shm_cov.mem = shm;
    shm_cov.size = size;
}

// ======== Public API ========

/// Initializes the coverage system
#[no_mangle]
pub extern "C" fn __cov_init() {
    init_cov_shm();

    // Register the exit handler
    let _ = std::panic::catch_unwind(|| unsafe {
        libc::atexit(__cov_report);
    });

    let state = COVERAGE_STATE.write().unwrap();

    // Check for environment variables
    if let Ok(val) = env::var(COVERAGE_ENABLED) {
        let enabled = val != "0";
        state
            .enabled
            .store(if enabled { 1 } else { 0 }, Ordering::SeqCst);
    }
    if let Ok(val) = env::var(COLOR_ENABLED) {
        let enabled = val != "0";
        state
            .color
            .store(if enabled { 1 } else { 0 }, Ordering::SeqCst);
    }
}

#[no_mangle]
pub extern "C" fn __cov_mapping_src(
    file_ptr: *const libc::c_char,
    funcs_ptr: *const u32,
    funcs_length: usize,
    brs_ptr: *const u32,
    brs_length: usize,
    lines_ptr: *const u32,
    lines_length: usize,
) {
    if file_ptr.is_null() || funcs_ptr.is_null() || brs_ptr.is_null() || lines_ptr.is_null() {
        return;
    }

    let filename = cstr_to_string(file_ptr);
    let (funcs, brs, lines) = unsafe {
        (
            std::slice::from_raw_parts(funcs_ptr, funcs_length),
            std::slice::from_raw_parts(brs_ptr, brs_length),
            std::slice::from_raw_parts(lines_ptr, lines_length),
        )
    };
    let src_map = SourceMapping {
        lines: lines.to_vec(),
        brs: brs.to_vec(),
        funcs: funcs.to_vec(),
    };
    COVERAGE_STATE
        .write()
        .unwrap()
        .source_map
        .lock()
        .unwrap()
        .insert(filename, src_map);
}

#[no_mangle]
pub extern "C" fn __cov_hit_batch(
    file_ptr: *const libc::c_char,
    lines_ptr: *const u32,
    length: usize,
) {
    if lines_ptr.is_null() {
        return;
    }

    // convert raw C pointer into rust slice
    let lines = unsafe { std::slice::from_raw_parts(lines_ptr, length) };
    for line in lines {
        __cov_record(file_ptr, *line);
    }
}

/// Records a hit at the given source location
#[no_mangle]
pub extern "C" fn __cov_record(file_ptr: *const libc::c_char, line: u32) -> usize {
    // Early return if coverage is disabled
    if COVERAGE_STATE
        .read()
        .unwrap()
        .enabled
        .load(Ordering::Relaxed)
        == 0
    {
        return 0;
    }

    let file = cstr_to_string(file_ptr);
    let loc = LineMapping { file, line };

    // Get or create counter for this location
    let lines_idx = {
        let state = COVERAGE_STATE.write().unwrap();
        let mut loc_map = state.location_map.lock().unwrap();
        if let Some(&idx) = loc_map.get(&loc) {
            idx
        } else {
            let idx = state.lines.len();
            loc_map.insert(loc, idx);
            idx
        }
    };
    let mut state = COVERAGE_STATE.write().unwrap();
    if lines_idx >= state.lines.len() {
        // Insert a new index
        state.lines.push(AtomicUsize::new(0));
    }
    // Increment the lines
    state.lines[lines_idx].fetch_add(1, Ordering::Relaxed)
}

/// Reset all coverage lines to zero
#[no_mangle]
pub extern "C" fn __cov_reset() {
    for counter in &COVERAGE_STATE.read().unwrap().lines {
        counter.store(0, Ordering::Relaxed);
    }
}

/// Report function registered with atexit
#[no_mangle]
pub extern "C" fn __cov_report() {
    if COVERAGE_STATE
        .read()
        .unwrap()
        .enabled
        .load(Ordering::Relaxed)
        == 0
    {
        return;
    }
    let color_enabled = COVERAGE_STATE.read().unwrap().color.load(Ordering::Relaxed);
    write_coverage_data(color_enabled);
    write_cov_shm();
}

// ======== External API for Fuzzer Integration ========

/// Gets the current coverage bitmap for fuzzer consumption
#[no_mangle]
pub extern "C" fn __cov_get_hit_map(bitmap: *mut u64, len: usize) -> usize {
    if bitmap.is_null() {
        return 0;
    }

    let counter_count = COVERAGE_STATE.read().unwrap().lines.len();
    let bitmap_size = std::cmp::min(counter_count, len);
    let lines = &COVERAGE_STATE.read().unwrap().lines;
    // Fill the bitmap with 1s for hit locations, 0s for unhit
    for i in 0..bitmap_size {
        let hits = lines[i].load(Ordering::Relaxed);
        unsafe { *bitmap.add(i) = hits as u64 }
    }
    bitmap_size
}
