// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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

/// # Safety
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

/// # Safety
/// TODO: What does the caller need to know?
/// Create an `mwalibContext` struct.
#[no_mangle]
pub unsafe extern "C" fn mwalibContext_new(
    metafits: *const c_char,
    gpuboxes: *mut *const c_char,
    gpubox_count: size_t,
) -> *mut mwalibContext {
    let m = CStr::from_ptr(metafits).to_str().unwrap().to_string();
    let gpubox_slice = slice::from_raw_parts(gpuboxes, gpubox_count);
    let mut gpubox_files = Vec::with_capacity(gpubox_count);
    for g in gpubox_slice {
        let s = CStr::from_ptr(*g).to_str().unwrap();
        gpubox_files.push(s.to_string())
    }
    let context = match mwalibContext::new(&m, &gpubox_files) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            exit(1)
        }
    };
    Box::into_raw(Box::new(context))
}

/// # Safety
/// TODO: What does the caller need to know?
/// Free a previously-allocated `mwalibContext` struct.
#[no_mangle]
pub unsafe extern "C" fn mwalibContext_free(ptr: *mut mwalibContext) {
    if ptr.is_null() {
        eprintln!("mwalibContext_free: Warning: null pointer passed in");
        return;
    }
    Box::from_raw(ptr);
}

/// # Safety
/// TODO: What does the caller need to know?
/// Display an `mwalibContext` struct.
#[no_mangle]
pub unsafe extern "C" fn mwalibContext_display(ptr: *const mwalibContext) {
    if ptr.is_null() {
        eprintln!("mwalibContext_display: Warning: null pointer passed in");
        return;
    }
    let context = &*ptr;
    println!("{}", context);
}

/// # Safety
/// Read MWA data.
///
/// `num_scans` is an input and output variable. The input `num_scans` asks
/// `mwalib` to read in that many scans, but the output `num_scans` tells the
/// caller how many scans were actually read. This is done because the number of
/// scans requested might be more than what is available.
///
/// `num_gpubox_files` and `gpubox_hdu_size` are output variables, allowing the
/// caller to know how to index the returned data.
#[no_mangle]
pub unsafe extern "C" fn mwalibContext_read(
    context_ptr: *mut mwalibContext,
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
        &mut *context_ptr
    };

    // Read data in.
    let data = match context.read(*num_scans as usize) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("{}", e);
            exit(1)
        }
    };

    // Overwrite the input/output pointers to variables, so the caller knows how
    // to index the data.
    ptr::write(num_scans, *Box::new(context.num_data_scans as i32));
    ptr::write(num_gpubox_files, *Box::new(context.num_gpubox_files as i32));
    ptr::write(gpubox_hdu_size, *Box::new(context.gpubox_hdu_size as i64));

    // If the data buffer is empty, then just return a null pointer. The caller
    // should see that `num_scans` is 0, and therefore there is no data.
    if data.is_empty() {
        return ptr::null_mut();
    }

    // Convert the vectors of `data` to C-compatible arrays.
    let mut scan_ptrs = Vec::with_capacity(context.num_data_scans);
    for scan in data {
        let mut gpubox_ptrs = Vec::with_capacity(context.num_gpubox_files);
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

/// # Safety
/// Free a previously-allocated float*** (designed for use after
/// `mwalibContext_read`).
///
/// Python can't free memory itself, so this is useful for Python (and perhaps
/// other languages).
#[no_mangle]
pub unsafe extern "C" fn free_float_buffer(
    float_buffer_ptr: *mut *mut *mut c_float,
    num_scans: *const c_int,
    num_gpubox_files: *const c_int,
    gpubox_hdu_size: *const c_longlong,
) {
    if float_buffer_ptr.is_null() {
        eprintln!("free_float_buffer: Warning: null pointer passed in");
        return;
    }

    let scans = Vec::from_raw_parts(float_buffer_ptr, *num_scans as usize, *num_scans as usize);
    for g in scans {
        let gpubox = Vec::from_raw_parts(g, *num_gpubox_files as usize, *num_gpubox_files as usize);
        for d in gpubox {
            let _ = Vec::from_raw_parts(d, *gpubox_hdu_size as usize, *gpubox_hdu_size as usize);
        }
    }
}
