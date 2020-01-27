/*!
This module exists purely for other languages to interface with mwalib.

It's very difficult to provide errors to external callers, as rust's concept of
ownership means that any strings made by rust must also be deallocated by
rust. For now, the caller must use these interfaces correctly, and the
correctness of mwalib is verified by using rust directly (and some testing via
C).

TODO: Add examples.
 */

use std::ffi::*;
use std::slice;
use std::ptr;

use libc::{c_char, size_t};

use crate::types::*;

/// Free a rust-allocated CString.
///
/// mwalib uses error strings to detail the caller with anything that went
/// wrong. Non-rust languages cannot deallocate these strings; so, call this
/// function with the pointer to do that.
pub unsafe extern "C" fn mwalib_free_rust_cstring(rust_cstring: *mut c_char) {
    // Don't do anything if the pointer is null.
    if rust_cstring.is_null() {
        return;
    }
    CString::from_raw(rust_cstring);
}

/// Create an `mwalibObsContext` struct.
#[no_mangle]
pub unsafe extern "C" fn mwalibObsContext_new(
    metafits: *mut c_char,
    gpuboxes: *mut *mut c_char,
    gpubox_count: size_t,
) -> *mut mwalibObsContext {
    let m = CStr::from_ptr(metafits).to_str().unwrap().to_string();
    let gpubox_slice = slice::from_raw_parts(gpuboxes, gpubox_count);
    let mut gpubox_files = Vec::with_capacity(gpubox_count);
    for g in gpubox_slice {
        let s = CStr::from_ptr(*g).to_str().unwrap();
        gpubox_files.push(s.to_string())
    }
    let context = match mwalibObsContext::new(&m, &gpubox_files) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
    };
    Box::into_raw(Box::new(context))
}

/// Display an `mwalibObsContext` struct.
#[no_mangle]
pub unsafe extern "C" fn mwalibObsContext_display(ptr: *mut mwalibObsContext) {
    if ptr.is_null() {
        eprintln!("mwalibObsContext_display: Warning: null pointer passed in");
        return;
    }
    let context = *Box::from_raw(ptr);
    println!("{}", context);
    ptr::write(ptr, context);
}

/// Free a previously-allocated `mwalibObsContext` struct.
#[no_mangle]
pub unsafe extern "C" fn mwalibObsContext_free(ptr: *mut mwalibObsContext) {
    if ptr.is_null() {
        eprintln!("mwalibObsContext_free: Warning: null pointer passed in");
        return;
    }
    Box::from_raw(ptr);
}
