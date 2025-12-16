// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This module exists purely for other languages to interface with mwalib.

use crate::*;
use libc::{c_char, c_uint, size_t};
use std::{ffi::CString, slice};

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
/// * Number of bytes written including the NUL terminator
///
///
/// # Safety
/// It is up to the caller to:
/// - Allocate `error_buffer_len` bytes as a `char*` on the heap
/// - Free `error_buffer_ptr` (in C) once finished with the buffer
///
pub(crate) fn set_c_string(
    in_message: &str,
    error_buffer_ptr: *mut c_char,
    error_buffer_len: size_t,
) -> usize {
    if error_buffer_ptr.is_null() || error_buffer_len == 0 {
        return 0;
    }

    // Reserve space for NUL terminator
    let max_bytes = error_buffer_len - 1;
    let bytes = in_message.as_bytes();
    let write_len = bytes.len().min(max_bytes);

    unsafe {
        let buf = slice::from_raw_parts_mut(error_buffer_ptr as *mut u8, error_buffer_len);

        // Copy the string bytes
        buf[..write_len].copy_from_slice(&bytes[..write_len]);

        // Add NUL terminator
        buf[write_len] = 0u8;
    }

    write_len
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

//// Utility: Create a C String from a Rust string
#[inline]
pub fn ffi_create_c_string(rust_str: &str) -> *mut c_char {
    // Convert to CString (adds null terminator)
    return match CString::new(rust_str) {
        Ok(s) => s.into_raw(),
        Err(_) => {
            // If the string name contains an interior NUL, skip it (or handle differently)
            CString::new("").unwrap().into_raw()
        }
    };
}

/// Utility: free a Rust-allocated C string returned by this API.
#[inline]
pub fn ffi_free_rust_c_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr); // drop to free
        }
    }
}

/// Utility: Create a C Array from a Rust Vector
#[inline]
pub fn ffi_create_c_array<T>(rust_vec: Vec<T>) -> (*mut T, usize) {
    //let ptr = rust_vec.as_mut_ptr();
    //let len = rust_vec.len();
    //std::mem::forget(rust_vec); // Prevent Rust from freeing the Vec
    //(ptr, len)
    let boxed_slice: Box<[T]> = rust_vec.into_boxed_slice();
    let len = boxed_slice.len();
    let ptr = Box::into_raw(boxed_slice) as *mut T;
    (ptr, len)
}

/// Utility: free a Rust-allocated C struct returned by this API
#[inline]
pub fn ffi_free_c_array<T>(ptr: *mut T, len: usize) {
    if !ptr.is_null() {
        unsafe {
            let boxed = Box::from_raw(std::slice::from_raw_parts_mut(ptr, len));
            drop(boxed);
        }
    }
}
