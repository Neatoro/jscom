use llvm_sys::core::*;
use llvm_sys::prelude::*;

macro_rules! c_str {
    ($s:expr) => (
        format!("{}\0", $s).as_ptr() as *const i8
    );
}

pub fn void_type(context: LLVMContextRef) -> LLVMTypeRef {
    return unsafe { LLVMVoidTypeInContext(context) };
}

pub fn int8_pointer_type() -> LLVMTypeRef {
    return unsafe { LLVMPointerType(LLVMInt8Type(), 0) };
}

pub fn gen_local_string(module: LLVMModuleRef, data: &str) -> LLVMValueRef {
    let cdata = c_str!(data);
    let len: u32 = (data.len() + 1) as u32;

    let gep: LLVMValueRef = unsafe {
        let glob: LLVMValueRef = LLVMAddGlobal(module, LLVMArrayType(LLVMInt8Type(), len), c_str!("string"));

        LLVMSetLinkage(glob, llvm_sys::LLVMLinkage::LLVMInternalLinkage);
        LLVMSetGlobalConstant(glob, 1);

        LLVMSetInitializer(glob, LLVMConstString(cdata, len, 1));

        llvm_sys::core::LLVMConstInBoundsGEP(
            glob,
            [
                LLVMConstInt(LLVMInt32Type(), 0, 0),
                LLVMConstInt(LLVMInt32Type(), 0, 0)
            ].as_mut_ptr(),
            2
        )
    };

    return gep;
}
