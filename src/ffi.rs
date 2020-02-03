/*!
This module exists purely for other languages to interface with mwalib.

It's very difficult to provide errors to external callers, as rust's concept of
ownership means that any strings made by rust must also be deallocated by
rust. For now, the caller must use these interfaces correctly, and the
correctness of mwalib is verified by using rust directly (and some testing via
C).
 */

use std::ffi::*;
use std::process::exit;
use std::ptr;
use std::slice;

use libc::{c_char, c_float, c_int, c_longlong, size_t};

use crate::*;

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
    metafits: *const c_char,
    gpuboxes: *mut *const c_char,
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
            exit(1)
        }
    };
    Box::into_raw(Box::new(context))
}

/// Display an `mwalibObsContext` struct.
#[no_mangle]
pub unsafe extern "C" fn mwalibObsContext_display(ptr: *const mwalibObsContext) {
    if ptr.is_null() {
        eprintln!("mwalibObsContext_display: Warning: null pointer passed in");
        return;
    }
    let context = &*ptr;
    println!("{}", context);
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

/// Create an `mwalibBuffer` struct.
#[no_mangle]
pub unsafe extern "C" fn mwalibBuffer_new(
    context_ptr: *const mwalibObsContext,
    num_scans: size_t,
) -> *mut mwalibBuffer {
    if context_ptr.is_null() {
        eprintln!("mwalibBuffer_new: Error: null pointer passed in");
        exit(1);
    }
    let context = &*context_ptr;

    let buffer = match mwalibBuffer::new(&context, num_scans) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            exit(1)
        }
    };

    Box::into_raw(Box::new(buffer))
}

/// Read MWA data.
#[no_mangle]
pub unsafe extern "C" fn mwalibBuffer_read(
    context_ptr: *const mwalibObsContext,
    mwalib_buffer_ptr: *mut mwalibBuffer,
    num_scans: *mut c_int,
    num_gpubox_files: *mut c_int,
    gpubox_hdu_size: *mut c_longlong,
) -> *mut *mut *mut c_float {
    // Load the previously-initialised context and buffer structs. Exit if
    // either of these are null.
    let context = if context_ptr.is_null() {
        eprintln!("mwalibBuffer_read: Error: null pointer for \"context_ptr\" passed in");
        exit(1);
    } else {
        &*context_ptr
    };

    let buffer = if mwalib_buffer_ptr.is_null() {
        eprintln!("mwalibBuffer_read: Error: null pointer for \"mwalib_buffer_ptr\" passed in");
        exit(1);
    } else {
        &mut *mwalib_buffer_ptr
    };

    // Read data in.
    let data = match buffer.read(&context) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("{}", e);
            exit(1)
        }
    };

    // Overwrite the input/output pointers to variables, so the caller knows how
    // to index the data.
    ptr::write(num_scans, *Box::new(buffer.num_data_scans as i32));
    ptr::write(num_gpubox_files, *Box::new(buffer.num_gpubox_files as i32));
    ptr::write(gpubox_hdu_size, *Box::new(buffer.gpubox_hdu_size as i64));

    // If the data buffer is empty, then just return a null pointer. The caller
    // should see that `num_scans` is 0, and therefore there is no data.
    if data.is_empty() {
        return ptr::null_mut();
    }

    // Convert the vectors of `data_buffer` to C-compatible arrays.
    let mut scan_ptrs = Vec::with_capacity(buffer.num_data_scans);
    for scan in data {
        let mut gpubox_ptrs = Vec::with_capacity(buffer.num_gpubox_files);
        for mut gpubox in scan {
            // Ensure the vector -> array conversion doesn't have elements we
            // don't care about.
            gpubox.shrink_to_fit();
            // Get the pointers to the vector, and push into the upper layer. We
            // need to tell rust to forget about this vector, so it doesn't get
            // automatically deallocated. Deallocation should be handled by the
            // caller.

            // TODO: When Vec::into_raw_parts is stable, use that instead.
            gpubox_ptrs.push(gpubox.as_mut_ptr());
            std::mem::forget(gpubox);
        }
        scan_ptrs.push(gpubox_ptrs.as_mut_ptr());
        std::mem::forget(gpubox_ptrs);
    }
    let data_buffer_ptr = scan_ptrs.as_mut_ptr();
    std::mem::forget(scan_ptrs);

    data_buffer_ptr
}

/// Free an `mwalibBuffer` struct.
#[no_mangle]
pub unsafe extern "C" fn mwalibBuffer_free(mwalib_buffer_ptr: *mut mwalibBuffer) {
    if mwalib_buffer_ptr.is_null() {
        eprintln!("mwalibBuffer_free: Warning: null pointer for \"mwalib_buffer_ptr\" passed in");
    }
    Box::from_raw(mwalib_buffer_ptr);
}
