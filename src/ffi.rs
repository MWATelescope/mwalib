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
pub unsafe extern "C" fn mwalibContext_read_one_timestep_coarse_channel_bfp(
    context_ptr: *mut mwalibContext,
    timestep_index: *mut c_int,
    coarse_channel_index: *mut c_int,
) -> *mut c_float {
    // Load the previously-initialised context and buffer structs. Exit if
    // either of these are null.
    let context = if context_ptr.is_null() {
        eprintln!("mwalibBuffer_read: Error: null pointer for \"context_ptr\" passed in");
        exit(1);
    } else {
        &mut *context_ptr
    };

    // Read data in.
    let mut data = match context.read_one_timestep_coarse_channel_bfp(
        *timestep_index as usize,
        *coarse_channel_index as usize,
    ) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("{}", e);
            exit(1)
        }
    };
    // If the data buffer is empty, then just return a null pointer.
    if data.is_empty() {
        return ptr::null_mut();
    }

    // Convert the vector of `data` to C-compatible array.
    let data_buffer_ptr = data.as_mut_ptr();
    std::mem::forget(data);

    data_buffer_ptr
}

/// # Safety
/// Free a previously-allocated float* (designed for use after
/// `mwalibContext_read_one_timestep_coarse_channel_bfp`).
///
/// Python can't free memory itself, so this is useful for Python (and perhaps
/// other languages).
#[no_mangle]
pub unsafe extern "C" fn free_float_buffer(
    float_buffer_ptr: *mut c_float,
    gpubox_hdu_size: *const c_longlong,
) {
    if float_buffer_ptr.is_null() {
        eprintln!("free_float_buffer: Warning: null pointer passed in!");
        return;
    }

    let _ = Vec::from_raw_parts(
        float_buffer_ptr,
        *gpubox_hdu_size as usize,
        *gpubox_hdu_size as usize,
    );
}
