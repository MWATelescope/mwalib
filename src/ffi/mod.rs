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
/// and populate it with a string. This is primarily used to pass messages back to C from Rust.
///
/// # Arguments
///
/// * `in_message` - A Rust string holing the message you want to pass back to C
///
/// * `buffer_ptr` - Pointer to a char* buffer which has already been allocated, for storing the message.
///
/// * `buffer_len` - Length of char* buffer allocated by caller in C.
///
///
/// # Returns
///
/// * Number of bytes written including the NUL terminator
///
///
/// # Safety
/// It is up to the caller to:
/// - Allocate `buffer_len` bytes as a `char*` on the heap
/// - Free `buffer_ptr` (in C) once finished with the buffer
///
pub(crate) fn set_c_string(in_message: &str, buffer_ptr: *mut c_char, buffer_len: size_t) -> usize {
    if buffer_ptr.is_null() || buffer_len == 0 {
        return 0;
    }

    // Reserve space for NUL terminator
    let max_bytes = buffer_len - 1;
    let bytes = in_message.as_bytes();
    let write_len = bytes.len().min(max_bytes);

    unsafe {
        let buf = slice::from_raw_parts_mut(buffer_ptr as *mut u8, buffer_len);

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

/// Utility: Create a C String from a Rust string
///
/// Take a reference to a Rust string and return a C string pointer, transferring ownership to the caller (C).
///
/// # Arguments
///
/// * `rust_cstring` - pointer to a `char*` of a Rust string
///
///
/// # Returns
///
/// * A mutable pointer to c_char. This is C owned, and if there was an error with the string, e.g. an
/// extra NUL terminator for some reason, then the function will return a NULL pointer.
///
/// # Safety
/// * It is up to the caller to free the string, using `ffi_free_rust_c_string`.
#[inline]
pub fn ffi_create_c_string(rust_str: &str) -> *mut c_char {
    // Convert to CString (adds null terminator)
    return match CString::new(rust_str) {
        Ok(s) => s.into_raw(),
        Err(_) => {
            // If the string name contains an interior NUL or some other error, return a NULL ptr
            std::ptr::null_mut()
        }
    };
}

/// Utility: free a Rust-allocated C string returned by this API.
///
/// # Arguments
///
/// * `c_string_ptr` - pointer to a C-owned `char*`
///
/// # Returns
///
/// * Nothing
///
/// # Safety
/// * c_string_ptr must not have already been freed and must point to a C string.
#[inline]
pub fn ffi_free_rust_c_string(c_string_ptr: *mut c_char) {
    if !c_string_ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(c_string_ptr); // drop to free
        }
    }
}

/// Utility: Create a C Array from a Rust Vector
///
/// # Arguments
///
/// * `rust_vec` - A Rust owned vector of type T
///
/// # Returns
///
/// * Pointer to a C owned array of type T*
///
/// * Length of the C owned array of type T*
///
/// # Safety
/// * You must call `ffi_free_c_array` passing the pointer and len to properly free the object.
#[inline]
pub fn ffi_create_c_array<T>(rust_vec: Vec<T>) -> (*mut T, usize) {
    let boxed_slice: Box<[T]> = rust_vec.into_boxed_slice();
    let len = boxed_slice.len();
    let ptr = Box::into_raw(boxed_slice) as *mut T;
    (ptr, len)
}

/// Utility: free a Rust-allocated C struct returned by this API
///
/// # Arguments
///
/// * `c_vec_ptr` - C pointer to array of T
///
/// * `c_vec_len` - Length of array
///
/// # Returns
///
/// * Nothing
///
/// # Safety
/// * c_vec_ptr must not have already been freed and must point to a populate T*.
/// * If the T contains members which also have vectors or other objects that need freeing, you'll need to do that first
/// before calling this.
#[inline]
pub fn ffi_free_c_array<T>(c_vec_ptr: *mut T, c_vec_len: usize) {
    if !c_vec_ptr.is_null() {
        unsafe {
            let boxed = Box::from_raw(std::slice::from_raw_parts_mut(c_vec_ptr, c_vec_len));
            drop(boxed);
        }
    }
}
