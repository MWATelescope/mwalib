// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Unit tests for ffi module
use super::*;
use std::ffi::CStr;

/// Test that we can get the version numbers from the built crate
#[test]
fn test_mwalib_version_major() {
    assert_eq!(
        mwalib_get_version_major(),
        built_info::PKG_VERSION_MAJOR.parse::<c_uint>().unwrap()
    );
}

#[test]
fn test_mwalib_version_minor() {
    assert_eq!(
        mwalib_get_version_minor(),
        built_info::PKG_VERSION_MINOR.parse::<c_uint>().unwrap()
    );
}

#[test]
fn test_mwalib_version_patch() {
    assert_eq!(
        mwalib_get_version_patch(),
        built_info::PKG_VERSION_PATCH.parse::<c_uint>().unwrap()
    );
}

//
// Simple tests of the error message helper
//

/// Helper: read C buffer as &str (up to the first NUL).
/// Panics if the bytes before NUL are not valid UTF-8.
fn c_buf_to_str(ptr: *const c_char) -> &'static str {
    // SAFETY: We only call this in tests after writing a proper NUL-terminated
    // sequence, so CStr::from_ptr is safe here.
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_str().expect("invalid UTF-8 in C buffer")
}

// CoPilot!
#[test]
fn test_one_byte_buffer_writes_only_nul() {
    let mut buf = [0xFFu8; 1];
    let ptr = buf.as_mut_ptr() as *mut c_char;

    let written = set_c_string("a", ptr, buf.len());
    assert_eq!(written, 0, "no room for content, only NUL");
    assert_eq!(buf[0], 0u8, "must write NUL terminator");
}

#[test]
fn test_exact_fit_writes_all_bytes_and_nul() {
    // "hello" = 5 bytes. Need len = 6 to fit "hello\0"
    let mut buf = [0u8; 6];
    let ptr = buf.as_mut_ptr() as *mut c_char;

    let written = set_c_string("hello", ptr, buf.len());
    assert_eq!(written, 5);
    assert_eq!(c_buf_to_str(ptr), "hello");
    assert_eq!(buf[5], 0u8, "NUL terminator at last byte");
}

#[test]
fn test_truncates_when_buffer_too_small() {
    // Buffer can hold 3 bytes + NUL
    let mut buf = [0u8; 4];
    let ptr = buf.as_mut_ptr() as *mut c_char;

    let written = set_c_string("abcdef", ptr, buf.len());
    assert_eq!(written, 3);
    assert_eq!(c_buf_to_str(ptr), "abc");
    assert_eq!(buf[3], 0u8, "NUL terminator at last byte");
}

#[test]
fn test_multi_byte_utf8_is_truncated_by_bytes_not_panicking() {
    // "ééé" where each 'é' is 2 bytes in UTF-8 → total 6 bytes.
    let s = "ééé"; // bytes: [0xC3,0xA9]*3
    assert_eq!(s.len(), 6);

    // Buffer length 5 → can write 4 bytes + NUL. That will result in
    // "é" (2 bytes) + partial second 'é' (2 of 2 bytes) retained safely in bytes,
    // but we slice by byte index; the resulting CStr must still be valid up to NUL.
    let mut buf = [0u8; 5];
    let ptr = buf.as_mut_ptr() as *mut c_char;

    let written = set_c_string(s, ptr, buf.len());
    assert_eq!(written, 4);
    // The first 4 bytes of "ééé" correspond to "é" + start of next "é",
    // but because we write raw bytes and terminate at NUL, CStr sees bytes until NUL.
    // To avoid invalid UTF-8 assertion here, we check bytes rather than &str.
    assert_eq!(&buf[..written], &s.as_bytes()[..written]);
    assert_eq!(buf[written], 0u8);
}

#[test]
fn test_interior_nul_in_source_is_preserved_and_cstr_stops_early() {
    // "ab\0cd" → CStr will read only "ab"
    let src = "ab\0cd";
    let mut buf = [0u8; 8];
    let ptr = buf.as_mut_ptr() as *mut c_char;

    let written = set_c_string(src, ptr, buf.len());
    assert_eq!(
        written, 5,
        "copied bytes including interior NUL and final NUL"
    );
    // Raw bytes check (including interior NUL and trailing NUL)
    assert_eq!(&buf[..written], b"ab\0cd");
    assert_eq!(buf[written], 0u8, "trailing NUL we added");

    // CStr view should stop at first NUL (after 'ab')
    assert_eq!(c_buf_to_str(ptr), "ab");
}

#[test]
fn test_large_buffer_no_overrun_and_correct_nul() {
    let mut buf = [0xAAu8; 64];
    let ptr = buf.as_mut_ptr() as *mut c_char;

    let s = "Rust FFI ✅";
    let written = set_c_string(s, ptr, buf.len());
    assert_eq!(written, s.len());
    assert_eq!(c_buf_to_str(ptr), s);
    assert_eq!(buf[written], 0u8);

    // Bytes after the NUL remain untouched (no zero-fill expected)
    assert_eq!(buf[written + 1], 0xAAu8);
}

#[test]
fn test_empty_string_writes_only_nul() {
    let mut buf = [0x77u8; 4];
    let ptr = buf.as_mut_ptr() as *mut c_char;

    let written = set_c_string("", ptr, buf.len());
    assert_eq!(written, 0);
    assert_eq!(buf[0], 0u8);
    // CStr should view empty string
    assert_eq!(c_buf_to_str(ptr), "");
}

#[test]
fn test_buffer_len_two_allows_one_byte_plus_nul() {
    let mut buf = [0u8; 2];
    let ptr = buf.as_mut_ptr() as *mut c_char;

    let written = set_c_string("XYZ", ptr, buf.len());
    assert_eq!(written, 1);
    assert_eq!(c_buf_to_str(ptr), "X");
    assert_eq!(buf[1], 0u8);
}
