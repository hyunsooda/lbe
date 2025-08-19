use inkwell::{
    llvm_sys::{self},
    memory_buffer::MemoryBuffer,
    values::{AnyValue, InstructionOpcode, InstructionValue},
};
use llvm_sys::core::{LLVMGetDebugLocColumn, LLVMGetDebugLocFilename, LLVMGetDebugLocLine};
use std::ffi::CStr;
use std::path::Path;

pub fn read_ll(file_path: &str) -> Result<MemoryBuffer, Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    let mem_buf = MemoryBuffer::create_from_file(path)?;
    Ok(mem_buf)
}

pub fn get_instr_filename<'ctx, T: AnyValue<'ctx>>(instr: &'ctx T) -> Option<&'ctx str> {
    let mut length: libc::c_uint = 0;
    unsafe {
        let file_name_ptr = LLVMGetDebugLocFilename(instr.as_value_ref(), &mut length);
        if !file_name_ptr.is_null() {
            let file_name = CStr::from_ptr(file_name_ptr).to_str().unwrap();
            Some(file_name)
        } else {
            None
        }
    }
}

pub fn get_instr_loc<'ctx, T: AnyValue<'ctx>>(instr: &'ctx T) -> (u32, u32) {
    unsafe {
        let line = LLVMGetDebugLocLine(instr.as_value_ref());
        let col = LLVMGetDebugLocColumn(instr.as_value_ref());
        (line, col)
    }
}

pub fn cstr_to_str(cstr: &CStr) -> String {
    cstr.to_string_lossy().into_owned()
}

fn get_br_loc(instr: &mut InstructionValue) -> u32 {
    loop {
        let loc = get_instr_loc(instr).0;
        if loc != 0 {
            return loc;
        }
        match instr.get_next_instruction() {
            Some(next) => *instr = next,
            None => return 0,
        }
    }
}

pub fn record_br(brs_loc: &mut Vec<u32>, instr: &InstructionValue) {
    if instr.is_conditional() {
        if instr.get_opcode() == InstructionOpcode::Invoke {
            let (next_normal_label, exception_label) = (
                instr.get_operand(0).unwrap().right().unwrap(),
                instr.get_operand(1).unwrap().right().unwrap(),
            );
            if let Some(mut first_instr) = next_normal_label.get_first_instruction() {
                let loc = get_br_loc(&mut first_instr);
                brs_loc.push(loc);
            }
            if let Some(mut first_instr) = exception_label.get_first_instruction() {
                let loc = get_br_loc(&mut first_instr);
                brs_loc.push(loc);
            }
        }
        if instr.get_opcode() == InstructionOpcode::Br {
            let (tbr, fbr) = (
                instr.get_operand(2).unwrap().right().unwrap(),
                instr.get_operand(1).unwrap().right().unwrap(),
            );
            if let Some(mut first_instr) = tbr.get_first_instruction() {
                let loc = get_br_loc(&mut first_instr);
                brs_loc.push(loc);
            }
            if let Some(mut first_instr) = fbr.get_first_instruction() {
                let loc = get_br_loc(&mut first_instr);
                brs_loc.push(loc);
            }
        }
    }
}

pub fn record_switch(brs_loc: &mut Vec<u32>, instr: &InstructionValue) {
    if instr.is_terminator() {
        if instr.get_opcode() == InstructionOpcode::Switch {
            let mut labels = vec![];
            for i in (0..instr.get_num_operands()).step_by(2) {
                let label = instr.get_operand(i + 1).unwrap().right().unwrap();
                labels.push(label);
            }
            if labels.len() > 1 {
                labels.push(labels[0]);
                labels.remove(0);

                for i in (0..labels.len() - 1).step_by(2) {
                    if let Some(mut first_instr) = labels[i].get_first_instruction() {
                        let loc = get_br_loc(&mut first_instr);
                        brs_loc.push(loc);
                    }
                    if let Some(mut first_instr) = labels[i + 1].get_first_instruction() {
                        let loc = get_br_loc(&mut first_instr);
                        brs_loc.push(loc);
                    }
                }
            }
        }
    }
}
