// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Unit tests for ffi module
#[cfg(test)]
use super::*;

/// Test that we can get the version numbers from the built crate
#[test]
pub fn test_mwalib_version_major() {
    assert_eq!(
        mwalib_get_version_major(),
        built_info::PKG_VERSION_MAJOR.parse::<c_uint>().unwrap()
    );
}

#[test]
pub fn test_mwalib_version_minor() {
    assert_eq!(
        mwalib_get_version_minor(),
        built_info::PKG_VERSION_MINOR.parse::<c_uint>().unwrap()
    );
}

#[test]
pub fn test_mwalib_version_patch() {
    assert_eq!(
        mwalib_get_version_patch(),
        built_info::PKG_VERSION_PATCH.parse::<c_uint>().unwrap()
    );
}

//
// Simple test of the error message helper
//
#[test]
fn test_set_error_message() {
    let buffer = CString::new("HELLO WORLD").unwrap();
    let buffer_ptr = buffer.as_ptr() as *mut c_char;

    set_c_string("hello world", buffer_ptr, 12);

    assert_eq!(buffer, CString::new("hello world").unwrap());
}

#[test]
fn test_set_error_message_null_ptr() {
    let buffer_ptr: *mut c_char = std::ptr::null_mut();

    set_c_string("hello world", buffer_ptr, 12);
}

#[test]
fn test_set_error_message_buffer_len_too_small() {
    let buffer = CString::new("H").unwrap();
    let buffer_ptr = buffer.as_ptr() as *mut c_char;

    set_c_string("hello world", buffer_ptr, 1);
}

#[test]
fn test_mwalib_free_rust_cstring() {
    let buffer = CString::new("HELLO WORLD").unwrap();
    let buffer_ptr = buffer.into_raw();

    // into_raw will take garbage collection of the buffer away from rust, so
    // some ffi/C code can free it (like below)
    unsafe {
        assert_eq!(mwalib_free_rust_cstring(buffer_ptr), 0);
    }
}

#[test]
fn test_mwalib_free_rust_cstring_null_ptr() {
    let buffer_ptr: *mut c_char = std::ptr::null_mut();
    unsafe {
        assert_eq!(mwalib_free_rust_cstring(buffer_ptr), 0);
    }
}
