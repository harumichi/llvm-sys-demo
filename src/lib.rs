extern crate llvm_sys;
extern crate libc;

use llvm_sys::core::*;
use llvm_sys::prelude::*;

use std::os::raw::{c_char};
use std::ffi::{CString, CStr};
use libc::{
    printf,
};


pub unsafe fn to_string(ptr: *const c_char) -> &'static str {
    CStr::from_ptr(ptr).to_str().unwrap()
}


pub unsafe fn use_llvm() {

    let context = LLVMContextCreate();
    
    let module = LLVMModuleCreateWithNameInContext(b"hoge\0".as_ptr() as *const _, context);
    let builder = LLVMCreateBuilderInContext(context);
    
        // Write here to build IR
    
    LLVMDumpModule(module);
    
    LLVMDisposeBuilder(builder);
    LLVMDisposeModule(module);
    LLVMContextDispose(context);
}


pub unsafe fn get_bitcode() {

    let path_str = format!(
        "{}/{}",
        "./data",
        "sleefsimddp_AVX.bc"
    );
    let path = CString::new(path_str).unwrap().into_raw();
    
    let mut membuf = 0 as LLVMMemoryBufferRef;
    let mut msg = 0 as *mut c_char;

    let ret = LLVMCreateMemoryBufferWithContentsOfFile(
        path, &mut membuf as *mut LLVMMemoryBufferRef, &mut msg as *mut *mut c_char
    );
    if ret != 0 {
        println!("path='{}', msg='{}'", to_string(path), to_string(msg));
        panic!("LLVMCreateMemoryBufferWithContentsOfFile");
    }


    
    
}
