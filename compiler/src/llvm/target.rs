use llvm_sys::target_machine::*;

use super::components::Module;

macro_rules! c_str_mut {
    ($s:expr) => (
        format!("{}\0", $s).as_mut_ptr() as *mut i8
    );
}

pub struct TargetMachine {
    pub tm: LLVMTargetMachineRef,
}

impl TargetMachine {
    pub fn new(target_triple: *const i8) -> Result<Self, String> {
        let mut target = std::ptr::null_mut();
        let mut err_msg_ptr = std::ptr::null_mut();
        unsafe {
            LLVMGetTargetFromTriple(target_triple, &mut target, &mut err_msg_ptr);
            if target.is_null() {
                assert!(!err_msg_ptr.is_null());

                let err_msg_cstr = std::ffi::CStr::from_ptr(err_msg_ptr as *const _);
                let err_msg = std::str::from_utf8(err_msg_cstr.to_bytes()).unwrap();
                return Err(err_msg.to_owned());
            }
        }

        let cpu = std::ffi::CString::new("generic").unwrap();
        let features = std::ffi::CString::new("").unwrap();

        let target_machine;
        unsafe {
            target_machine = LLVMCreateTargetMachine(
                target,
                target_triple,
                cpu.as_ptr() as *const _,
                features.as_ptr() as *const _,
                LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
                LLVMRelocMode::LLVMRelocPIC,
                LLVMCodeModel::LLVMCodeModelDefault,
            );
        }

        Ok(TargetMachine { tm: target_machine })
    }

    pub fn write_to_object_file(&self, module: &Module) {
        let mut obj_error = c_str_mut!("Writing object file failed.");
        let result = unsafe { LLVMTargetMachineEmitToFile(
            self.tm,
            module.get_module(),
            c_str_mut!("out.o"),
            LLVMCodeGenFileType::LLVMObjectFile,
            &mut obj_error,
        ) };

        if result != 0 {
            unsafe {
                panic!("obj_error: {:?}", std::ffi::CStr::from_ptr(obj_error as *const _));
            }
        }
    }

    pub fn dispose(&mut self) {
        unsafe {
            LLVMDisposeTargetMachine(self.tm);
        }
    }
}
