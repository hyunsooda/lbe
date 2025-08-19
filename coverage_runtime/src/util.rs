use std::collections::HashSet;
use std::hash::Hash;

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

pub fn get_intersect<T: Eq + Clone + Hash>(values: &HashSet<T>, other: &HashSet<T>) -> HashSet<T> {
    values.intersection(other).cloned().collect::<HashSet<_>>()
}

pub fn get_symmetric_diff<T: Eq + Clone + Hash>(values: &HashSet<T>, other: &HashSet<T>) -> Vec<T> {
    values
        .symmetric_difference(other)
        .cloned()
        .collect::<Vec<_>>()
}
