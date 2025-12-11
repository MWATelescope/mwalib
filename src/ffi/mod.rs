// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This module exists purely for other languages to interface with mwalib.

use crate::*;
use libc::{c_char, c_uint, size_t};
use std::ffi::*;
use std::mem;
use std::ptr;
use std::slice;

#[cfg(test)]
pub(crate) mod ffi_test_helpers;

#[cfg(test)]
mod test;

/// mwalib FFI interface return codes
pub const MWALIB_SUCCESS: i32 = 0;
pub const MWALIB_FAILURE: i32 = 1;
pub const MWALIB_NO_DATA_FOR_TIMESTEP_COARSECHAN: i32 = -1;

/// Generic helper function for all FFI modules to take an already allocated C string buffer
/// and populate it with a string. This is primarily used to pass error messages back to C from Rust.
///
/// # Arguments
///
/// * `in_message` - A Rust string holing the error message you want to pass back to C
///
/// * `error_buffer_ptr` - Pointer to a char* buffer which has already been allocated, for storing the error message.
///
/// * `error_buffer_len` - Length of char* buffer allocated by caller in C.
///
///
/// # Returns
///
/// * Nothing
///
///
/// # Safety
/// It is up to the caller to:
/// - Allocate `error_buffer_len` bytes as a `char*` on the heap
/// - Free `error_buffer_ptr` once finished with the buffer
///
pub(crate) fn set_c_string(
    in_message: &str,
    error_buffer_ptr: *mut c_char,
    error_buffer_len: size_t,
) {
    // Don't do anything if the pointer is null.
    if error_buffer_ptr.is_null() {
        return;
    }
    // Check that error buffer, minus 1 for nul terminator is still >=1
    if error_buffer_len < 2 {
        return;
    } // need at least 1 char + NUL

    // Trim it to error_buffer_len - 1 (must include room for null terminator)
    let max_bytes = error_buffer_len - 1;
    // Strip interior NULs to avoid CString failure
    let sanitized = in_message.replace('\0', "");
    let message = if sanitized.len() > max_bytes {
        &sanitized[..max_bytes]
    } else {
        &sanitized
    };

    // Convert to C string- panic if it can't.
    let error_message = CString::new(message).unwrap_or_else(|_| CString::new("").unwrap());

    // Add null terminator
    let error_message_bytes = error_message.as_bytes_with_nul();

    unsafe {
        let buf = slice::from_raw_parts_mut(error_buffer_ptr as *mut u8, error_buffer_len);
        buf[..error_message_bytes.len()].copy_from_slice(error_message_bytes);
    }
}

/// Return the MAJOR version number of mwalib
///
/// Uses the built crate in build.rs to generate at build time a built.rs module
///
/// # Arguments
///
/// * None
///
/// # Returns
///
/// * u16 representing the major version number of mwalib
///
#[no_mangle]
pub extern "C" fn mwalib_get_version_major() -> c_uint {
    built_info::PKG_VERSION_MAJOR.parse::<c_uint>().unwrap()
}

/// Return the MINOR version number of mwalib
///
/// Uses the built crate in build.rs to generate at build time a built.rs module
///
/// # Arguments
///
/// * None
///
/// # Returns
///
/// * u16 representing the minor version number of mwalib
///
#[no_mangle]
pub extern "C" fn mwalib_get_version_minor() -> c_uint {
    built_info::PKG_VERSION_MINOR.parse::<c_uint>().unwrap()
}

/// Return the PATCH version number of mwalib
///
/// Uses the built crate in build.rs to generate at build time a built.rs module
///
/// # Arguments
///
/// * None
///
/// # Returns
///
/// * u16 representing the patch version number of mwalib
///
#[no_mangle]
pub extern "C" fn mwalib_get_version_patch() -> c_uint {
    built_info::PKG_VERSION_PATCH.parse::<c_uint>().unwrap()
}

/// Free a rust-allocated CString.
///
/// mwalib uses error strings to detail the caller with anything that went
/// wrong. Non-rust languages cannot deallocate these strings; so, call this
/// function with the pointer to do that.
///
/// # Arguments
///
/// * `rust_cstring` - pointer to a `char*` of a Rust string
///
///
/// # Returns
///
/// * MWALIB_SUCCESS on success, non-zero on failure
///
/// # Safety
/// * rust_cstring must not have already been freed and must point to a Rust string.
#[no_mangle]
pub unsafe extern "C" fn mwalib_free_rust_cstring(rust_cstring: *mut c_char) -> i32 {
    // Don't do anything if the pointer is null.
    if rust_cstring.is_null() {
        return MWALIB_SUCCESS;
    }
    drop(CString::from_raw(rust_cstring));

    // return success
    MWALIB_SUCCESS
}

/// Boxes for FFI a rust-allocated vector of T. If the vector is 0 length, returns a null pointer.
/// This leaks the array of T to C.
/// At the end of use, you MUST call ffi_free_c_boxed_slice()
///
/// # Arguments
///
/// * `v` - Rust vector of T's
///
///
/// # Returns
///
/// * a raw pointer to the array of T's
///
pub(crate) fn ffi_array_to_boxed_slice<T>(v: Vec<T>) -> *mut T {
    if !v.is_empty() {
        // Prevent the slice from being destroyed (Leak the memory).
        // This is because we are using our ffi code to free the memory
        let mut boxed = v.into_boxed_slice();
        let ptr = boxed.as_mut_ptr();
        let _len = boxed.len();
        std::mem::forget(boxed);

        ptr
    } else {
        std::ptr::null_mut()
    }
}

/// Reconstitutes a pointer and number of elements into a slice
/// so it can be dropped/forgotten
/// Use this to free an object leaked to C via ffi_array_to_boxed_slice()
///
pub(crate) fn ffi_free_c_boxed_slice<T>(ptr_to_array: *mut T, len: usize) {
    if ptr_to_array.is_null() {
        return;
    }
    unsafe {
        // Rebuild a *mut [T] fat pointer, then drop the Box<[T]>
        let _ = Box::from_raw(ptr::slice_from_raw_parts_mut(ptr_to_array, len));
    }
}

pub(crate) fn rust_string_to_buf(rust_str: String) -> *mut c_char {
    let mut boxed: Box<[u8]> = rust_str.into_bytes().into_boxed_slice();
    let ptr_u8 = boxed.as_mut_ptr();
    //let len = boxed.len();

    // Leak ownership to C
    mem::forget(boxed);

    ptr_u8.cast::<c_char>()
}

pub(crate) fn free_c_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        let _ = CString::from_raw(ptr);
    }
}
