use libc::{c_char, c_void, dlsym, size_t, RTLD_NEXT};
use std::collections::HashMap;
use std::sync::Mutex;

type MallocFn = unsafe extern "C" fn(size_t) -> *mut c_void;
type ReallocFn = unsafe extern "C" fn(ptr: *mut c_void, size: size_t) -> *mut c_void;
type FreeFn = unsafe extern "C" fn(ptr: *mut c_void);
type StrcpyFn = unsafe extern "C" fn(dest: *mut c_char, src: *const c_char) -> *mut c_char;

lazy_static::lazy_static! {
    static ref ALLOC_MAP: Mutex<HashMap<usize, usize>> = Mutex::new(HashMap::new());
    static ref MALLOC: Mutex<Option<MallocFn>> = Mutex::new(None);
    static ref REALLOC: Mutex<Option<ReallocFn>> = Mutex::new(None);
    static ref FREE: Mutex<Option<FreeFn>> = Mutex::new(None);
    static ref STRCPY: Mutex<Option<StrcpyFn>> = Mutex::new(None);
}

pub fn get_malloc_cstr() -> [i8; 7] {
    let fn_str = b"malloc";
    let mut buf = [0 as c_char; 7];
    for (i, &b) in fn_str.iter().enumerate() {
        buf[i] = b as c_char;
    }
    buf
}

pub fn get_realloc_str() -> [i8; 8] {
    let fn_str = b"realloc";
    let mut buf = [0 as c_char; 8];
    for (i, &b) in fn_str.iter().enumerate() {
        buf[i] = b as c_char;
    }
    buf
}

pub fn get_free_str() -> [i8; 5] {
    let fn_str = b"free";
    let mut buf = [0 as c_char; 5];
    for (i, &b) in fn_str.iter().enumerate() {
        buf[i] = b as c_char;
    }
    buf
}

pub fn get_strcpy_str() -> [i8; 7] {
    let fn_str = b"strcpy";
    let mut buf = [0 as c_char; 7];
    for (i, &b) in fn_str.iter().enumerate() {
        buf[i] = b as c_char;
    }
    buf
}

pub fn get_cmalloc() -> MallocFn {
    let mut cmalloc = MALLOC.lock().unwrap();
    if cmalloc.is_none() {
        let buf = get_malloc_cstr();
        let malloc_sym = unsafe { dlsym(RTLD_NEXT, buf.as_ptr()) };
        let malloc_fn =
            unsafe { std::mem::transmute::<*const (), MallocFn>(malloc_sym as *const ()) };
        *cmalloc = Some(malloc_fn);
    }
    cmalloc.unwrap()
}

pub fn get_crealloc() -> ReallocFn {
    let mut crealloc = REALLOC.lock().unwrap();
    if crealloc.is_none() {
        let buf = get_realloc_str();
        let realloc_sym = unsafe { dlsym(RTLD_NEXT, buf.as_ptr()) };
        let realloc_fn =
            unsafe { std::mem::transmute::<*const (), ReallocFn>(realloc_sym as *const ()) };
        *crealloc = Some(realloc_fn);
    }
    crealloc.unwrap()
}

pub fn get_cfree() -> FreeFn {
    let mut cfree = FREE.lock().unwrap();
    if cfree.is_none() {
        let buf = get_free_str();
        let free_sym = unsafe { dlsym(RTLD_NEXT, buf.as_ptr()) };
        let free_fn = unsafe { std::mem::transmute::<*const (), FreeFn>(free_sym as *const ()) };
        *cfree = Some(free_fn);
    }
    cfree.unwrap()
}

pub fn get_strcpy() -> StrcpyFn {
    let mut cstrcpy = STRCPY.lock().unwrap();
    if cstrcpy.is_none() {
        let buf = get_strcpy_str();
        let strcpy_sym = unsafe { dlsym(RTLD_NEXT, buf.as_ptr()) };
        let strcpy_fn =
            unsafe { std::mem::transmute::<*const (), StrcpyFn>(strcpy_sym as *const ()) };
        *cstrcpy = Some(strcpy_fn);
    }
    cstrcpy.unwrap()
}
