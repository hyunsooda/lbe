use crate::{asan_intrinsic::*, asan_runtime::__asan_mem_check};
use heapless::FnvIndexMap;
use libc::{c_char, c_void, size_t};
use std::ptr;
use std::sync::Mutex;
use std::ffi::CStr;

pub const ALLOC_STACK: u8 = 0x1;
pub const ALLOC_HEAP: u8 = 0x2;

pub const STACK_LEFT_REDZONE_MARKER: i8 = -0x10;
pub const STACK_RIGHT_REDZONE_MARKER: i8 = -0x11;
pub const HEAP_LEFT_REDZONE_MARKER: i8 = -0x20;
pub const HEAP_RIGHT_REDZONE_MARKER: i8 = -0x21;
pub const FREED_MARKER: i8 = -0x30;
pub const CLEAN_BYTE_MARKER: i8 = 0x00;

pub const SHADOW_SCALE: usize = 3;
pub const REDZONE_SIZE: usize = 32;
pub const SHADOW_SIZE: usize = 1 << 32; // mapping 4GB. We can extend to maximum system memory
                                        // (on-demand paging). Increase this value if collision is
                                        // found
const ALLOC_MAP_SIZE: usize = 1 << 15;

const ALLOC_MAP_INSERT_ERR_STR: &str = "alloc map insertion error (hint: increase heapless map)";

thread_local! {
    pub static MALLOC_REENTERED: Mutex<bool> = const { Mutex::new(false) }
}

lazy_static::lazy_static! {
    pub static ref SHADOW_MEMORY: Mutex<Vec<i8>> = Mutex::new(vec![0i8; SHADOW_SIZE]);
    pub static ref ALLOC_MAP: Mutex<FnvIndexMap<usize, usize, ALLOC_MAP_SIZE>> = Mutex::new(FnvIndexMap::new());
}

#[no_mangle]
pub unsafe extern "C" fn malloc(size: size_t) -> *mut c_void {
    let cmalloc = get_cmalloc();

    MALLOC_REENTERED.with(|re_enter| {
        let is_renter = re_enter.lock().unwrap();
        if *is_renter {
            cmalloc(size)
        } else {
            let total_size = size + REDZONE_SIZE * 2;
            let raw_ptr = cmalloc(total_size) as *mut u8;
            __asan_init_redzone(raw_ptr, size,  ALLOC_HEAP)
        }
    })
}

#[no_mangle]
pub unsafe extern "C" fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char {
    let cstrcpy = get_strcpy();
    let src_len = CStr::from_ptr(src).to_bytes().len();
    let temp_filename_ptr = c"libc::strcpy".as_ptr() as *const c_char;
    for i in 0..=src_len {
        __asan_mem_check(temp_filename_ptr, dest as usize + i, 1);
    }
    cstrcpy(dest, src)
}

#[no_mangle]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, size: size_t) -> *mut c_void {
    if ptr.is_null() {
        return malloc(size);
    }
    let mut alloc_map = ALLOC_MAP.lock().unwrap();
    if let Some(org_size) = alloc_map.remove(&(ptr as usize)) {
        // drop locked variables because the following code will require `free` call
        drop(alloc_map);
        poison_shadow_freed(ptr, size);
        let usable_ptr = malloc(size);
        ptr::copy(ptr, usable_ptr, org_size);
        free(ptr.sub(REDZONE_SIZE));
        usable_ptr
    } else {
        let crealloc = get_crealloc();
        crealloc(ptr, size)
    }
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let cfree = get_cfree();
    let mut alloc_map = ALLOC_MAP.lock().unwrap();
    if let Some(size) = alloc_map.remove(&(ptr as usize)) {
        poison_shadow_freed(ptr, size);
        // use this code block for debugging purpose
        // some of library may touch areas of `LEFT_ZONE` and `RIGHT_ZONE`
        // as a result, free-ing thoes unexpected area may cause undefined behavior (e.g., segfault)
        // {
            // unsafe {
            //     let mut start = ptr.sub(REDZONE_SIZE) as *const u8;
            //     for offset in 0..(REDZONE_SIZE) {
            //         let c = start.add(offset);
            //         if *c != 0u8 {
            //             dbg!("corrupted: {}", *c);
            //         }
            //     }
            //     start = ptr.add(size) as *const u8;
            //     for offset in 0..(REDZONE_SIZE) {
            //         let c = start.add(offset);
            //         if *c != 0u8 {
            //             dbg!("corrupted: {}", *c);
            //         }
            //     }
            // }
        // }
        cfree(ptr.sub(REDZONE_SIZE));
    } else {
        cfree(ptr);
    }
}

/// Poisons left and right redzones and make clean for usable region
fn poison_shadow_allocated(raw_ptr: usize, usable_size: usize, alloc_kind: u8) {
    // |----------------------|----------------------|----------------------|
    // ^        LEFT_RZ                USABLE        ^       RIGHT_RZ       ^
    //
    // precisely allocate shadow byte for each boundary(^)

    let usable_ptr = raw_ptr + REDZONE_SIZE;

    let (left_rz_marker, right_rz_marker) = match alloc_kind {
        1 => (STACK_LEFT_REDZONE_MARKER, STACK_RIGHT_REDZONE_MARKER),
        2 => (HEAP_LEFT_REDZONE_MARKER, HEAP_RIGHT_REDZONE_MARKER),
        _ => panic!("unknown alloc kind"),
    };

    // 1. Poison left redzone
    let left_start = raw_ptr;
    let left_end = usable_ptr;
    let shadow_left_start = convert_to_shadow_idx(left_start);
    let shadow_left_end = convert_to_shadow_idx(left_end);
    let mut shadow_mem = SHADOW_MEMORY.lock().unwrap();

    set_bounary_poison_byte(
        &mut shadow_mem,
        left_start,
        shadow_left_start,
        left_rz_marker,
    );
    for i in (shadow_left_start + 1)..shadow_left_end {
        write_shadow_mem(&mut shadow_mem, i, left_rz_marker);
    }

    // 2. Unpoison usable region
    let usable_start = usable_ptr;
    let usable_end = usable_ptr + usable_size;
    let shadow_usable_start = convert_to_shadow_idx(usable_start);
    let shadow_usable_end = convert_to_shadow_idx(usable_end);

    for i in shadow_usable_start..shadow_usable_end {
        write_shadow_mem(&mut shadow_mem, i, CLEAN_BYTE_MARKER);
    }
    set_bounary_poison_byte(
        &mut shadow_mem,
        usable_end,
        shadow_usable_end,
        right_rz_marker,
    );

    // 3. Poison right redzone
    let right_start = usable_end;
    let right_end = right_start + REDZONE_SIZE;
    let shadow_right_start = convert_to_shadow_idx(right_start);
    let shadow_right_end = convert_to_shadow_idx(right_end);

    for i in (shadow_right_start + 1)..shadow_right_end {
        write_shadow_mem(&mut shadow_mem, i, right_rz_marker);
    }
    set_bounary_poison_byte(
        &mut shadow_mem,
        right_end,
        shadow_right_end,
        right_rz_marker,
    );
}

fn set_bounary_poison_byte(
    shadow_mem: &mut [i8],
    start: usize,
    shadow_start: usize,
    rz_marker: i8,
) {
    let remaining = start & 0x07;
    if remaining != 0 {
        write_shadow_mem(shadow_mem, shadow_start, remaining as i8);
    } else {
        write_shadow_mem(shadow_mem, shadow_start, rz_marker);
    }
}

/// Make all regions as poisoned
fn poison_shadow_freed(usable_ptr: *mut c_void, size: usize) {
    let start = unsafe { usable_ptr.sub(REDZONE_SIZE) };
    let shadow_start = convert_to_shadow_idx(start as usize);
    let shadow_end =
        unsafe { convert_to_shadow_idx(start.add(REDZONE_SIZE + size + REDZONE_SIZE) as usize) };
    let mut shadow = SHADOW_MEMORY.lock().unwrap();
    set_bounary_poison_byte(&mut shadow, start as usize, shadow_start, FREED_MARKER);
    for i in (shadow_start + 1)..shadow_end {
        write_shadow_mem(&mut shadow, i, FREED_MARKER);
    }
    set_bounary_poison_byte(&mut shadow, start as usize, shadow_end, FREED_MARKER);
}

#[no_mangle]
pub unsafe extern "C" fn __asan_init_redzone(
    raw_ptr: *mut u8,
    usable_size: size_t,
    alloc_kind: u8,
) -> *mut c_void {
    if raw_ptr.is_null() {
        return ptr::null_mut();
    }

    let usable_ptr = unsafe { raw_ptr.add(REDZONE_SIZE) };

    // in a right mannor of implementation, this memset seems redundant
    // without this unused explicit memory initialization, it may trigger segfault
    // you may try comment out these two lines and use z3 library, then segfault will be triggered
    // when `free` system call is invoked.
    // exact reasoning has not been found yet
    // also, initialization with zero value only works emperically
    libc::memset(raw_ptr as *mut c_void, 0, REDZONE_SIZE);
    libc::memset(raw_ptr.add(REDZONE_SIZE + usable_size) as *mut c_void, 0, REDZONE_SIZE);

    // initialize shadow memory
    poison_shadow_allocated(raw_ptr as usize, usable_size, alloc_kind);
    // record allocated size
    ALLOC_MAP
        .lock()
        .unwrap()
        .insert(usable_ptr as usize, usable_size)
        .expect(ALLOC_MAP_INSERT_ERR_STR);

    // return usable region pointer
    usable_ptr as *mut c_void
}

pub fn convert_to_shadow_idx(addr: usize) -> usize {
    addr >> SHADOW_SCALE
}

fn write_shadow_mem(shadow_mem: &mut [i8], idx: usize, val: i8) {
    shadow_mem[idx % SHADOW_SIZE] = val;
}
