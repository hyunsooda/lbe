use crate::symbolic::{select_id, ADDRS, CONSTRAINTS};

#[no_mangle]
pub extern "C" fn __symbolic_module_add_sym(
    id: i64,
    left_operand_kind: i8,
    left_operand_val: i64,
    right_operand_kind: i8,
    right_operand_val: i64,
    predicate: i8,
) {
    CONSTRAINTS.write().unwrap().add_constraint(
        id,
        left_operand_kind,
        left_operand_val,
        right_operand_kind,
        right_operand_val,
        predicate,
    );
}

#[no_mangle]
pub extern "C" fn __symbolic_make_prepare(ptr: *mut libc::c_void, addr: i64) {
    ADDRS.write().unwrap().insert(ptr as i32, addr);
}

#[no_mangle]
pub extern "C" fn __make_symbolic(typ_size: usize, ptr: *mut libc::c_void) {
    if !ptr.is_null() {
        let id = select_id();
        if id.is_none() {
            return;
        }
        let addr = {
            let addrs = ADDRS.read().unwrap();
            *addrs.get(&(ptr as i32)).unwrap()
        };

        let id = id.unwrap();
        if let Some(solutions) = CONSTRAINTS.read().unwrap().solve(id) {
            for (sym_addr, solution) in solutions {
                if sym_addr == addr {
                    match typ_size {
                        1 => unsafe {
                            std::ptr::write(ptr as *mut libc::c_char, solution as i8);
                        },
                        4 => unsafe {
                            std::ptr::write(ptr as *mut libc::c_int, solution as i32);
                        },
                        _ => unsafe {
                            std::ptr::write(ptr as *mut libc::c_long, solution);
                        },
                    }
                }
            }
        }
    }
}
