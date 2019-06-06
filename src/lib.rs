extern crate llvm_sys;
extern crate libc;

use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::bit_reader::*;
use llvm_sys::linker::*;
use llvm_sys::LLVMLinkage;

use libc::{size_t};
use std::ptr;
use std::os::raw::{c_char};
use std::ffi::{CString, CStr};


macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    };
}

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


pub unsafe fn get_bc_module(bcname: &str) -> LLVMModuleRef {
    let path_str = format!(
        "{}/{}",
        "./data",
        bcname
    );
    let path = CString::new(path_str).unwrap().into_raw();
    
    let mut membuf = 0 as LLVMMemoryBufferRef;
    let mut msg = 0 as *mut c_char;

    let ret = LLVMCreateMemoryBufferWithContentsOfFile(
        path, &mut membuf as *mut LLVMMemoryBufferRef, &mut msg as *mut *mut c_char
    );
    if ret != 0 {
        println!("LLVMCreateMemoryBufferWithContentsOfFile: {}", to_string(msg));
        println!("{}", to_string(path));
        panic!();
    }

    let mut module = 0 as LLVMModuleRef;
    let ret = LLVMParseBitcode(
        membuf, &mut module as *mut LLVMModuleRef,
        &mut msg as *mut *mut c_char
    );
    if ret != 0 {
        println!("LLVMParseBitcode: {}", to_string(msg));
        panic!();
    }
    LLVMDisposeMemoryBuffer(membuf);

    return module;
}


pub unsafe fn look_module(module: LLVMModuleRef) {
    
    let sym = CString::new("Sleef_expd4_u10avx").unwrap().into_raw();

    let function = LLVMGetNamedFunction(module, sym as *const c_char);  // how to handle this error
        
    let name = LLVMGetValueName(function);
    println!("{}", to_string(name));
}


pub unsafe fn call_func_in(callee_mod: LLVMModuleRef) {

    let ret_void = false;

    // get builder
    let context = LLVMGetGlobalContext();
    let caller_mod = LLVMModuleCreateWithNameInContext(c_str!("caller") as *const c_char, context);
    let builder = LLVMCreateBuilderInContext(context);

    // set builder to caller function
    let void_type = LLVMVoidTypeInContext(context);
    let func_type = if ret_void {
        LLVMFunctionType(LLVMVoidTypeInContext(context), ptr::null_mut(), 0, 0)
    } else {
        LLVMFunctionType(LLVMDoubleTypeInContext(context), ptr::null_mut(), 0, 0)
    };
    let caller_func = LLVMAddFunction(caller_mod, c_str!("mainfunc"), func_type);
    let block = LLVMAppendBasicBlockInContext(context, caller_func, c_str!("e0"));
    LLVMPositionBuilderAtEnd(builder, block);

    // simd
    let width = 4;
    
    // declare callee
    let arg_type = LLVMVectorType(LLVMDoubleTypeInContext(context), width);
    let mut param_type = vec![arg_type];
    let func_type = LLVMFunctionType(
        arg_type, param_type.as_mut_ptr(), 1, 0
    );
    let callee_func = LLVMAddFunction(caller_mod,
                                      c_str!("Sleef_expd4_u10avx"),
                                      func_type);
    
    // build code
    let mut elem = Vec::new();
    //let mut elem = Vec<LLVMValueRef>::new();
    for i in 0..width {
        elem.push(
            LLVMConstReal(LLVMDoubleTypeInContext(context), i as f64)
        );
    }
    let vector = LLVMConstVector(elem.as_mut_ptr(), width);

    let a = LLVMBuildAlloca(builder, arg_type, c_str!("a")); 
    LLVMBuildStore(builder, vector, a);
    let mut a_val = LLVMBuildLoad(builder, a, c_str!("a_val"));

    let b = LLVMBuildCall(builder,
                          callee_func,
                          &mut a_val as *mut LLVMValueRef,
                          1,
                          c_str!("b"));
    if ret_void {
        LLVMBuildRetVoid(builder);
    } else {
        let idx = 1;
        let b_i = LLVMBuildExtractElement(
            builder, b, LLVMConstInt(LLVMInt32TypeInContext(context), idx, 1), c_str!("b_i")
        );
        LLVMBuildRet(builder, b_i);
    }
    
    // dispose builder
    LLVMDisposeBuilder(builder);

    println!("After link: caller={}, callee={}", caller_func as u64, callee_func as u64);    

    // link module
    let ret = LLVMLinkModules2(caller_mod, callee_mod);
    if ret != 0 {
        panic!("LLVMLinkModules2");
    }

    println!("Before link: caller={}, callee={}", caller_func as u64, callee_func as u64);    
    
    // add attribute
    let name = CString::new("alwaysinline").unwrap();
    let kind = LLVMGetEnumAttributeKindForName(name.as_ptr(), name.as_bytes().len());
    assert_ne!(kind, 0);
    let attr = LLVMCreateEnumAttribute(context, kind, 0);
    let callee_func_def = LLVMGetNamedFunction(caller_mod, c_str!("Sleef_expd4_u10avx"));
    LLVMAddAttributeAtIndex(callee_func_def, !0, attr);
    // linkage to private
    LLVMSetLinkage(callee_func_def, LLVMLinkage::LLVMPrivateLinkage);

    // verify
    use self::llvm_sys::analysis::LLVMVerifierFailureAction::*;
    use self::llvm_sys::analysis::LLVMVerifyModule;
    let mut error_str = ptr::null_mut();
    let result_code = LLVMVerifyModule(caller_mod, LLVMReturnStatusAction, &mut error_str);
    if result_code != 0 {
        panic!("LLVMVerifyModule ");
    }

    
    // dump
    LLVMDumpModule(caller_mod);

    // dispose module
    LLVMDisposeModule(caller_mod);
    //LLVMDisposeModule(callee_mod);  // already released in linking
}
   

pub unsafe fn demo() {
    let module = get_bc_module("weldsimddp_AVX.bc");
    //look_module(module);
    call_func_in(module);
}
