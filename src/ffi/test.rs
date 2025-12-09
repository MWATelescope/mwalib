// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Unit tests for ffi module
#[cfg(test)]
use super::*;
use crate::ffi::ffi_test_helpers::*;
use crate::voltage_context::ffi::*;
use antenna::ffi::Antenna;
use float_cmp::*;

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

//
// Metafits context Tests
//
#[test]
fn test_mwalib_metafits_context_new_valid() {
    // This tests for a valid metafitscontext
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    unsafe {
        // Create a MetafitsContext
        let mut metafits_context_ptr: *mut MetafitsContext = std::ptr::null_mut();
        let retval = mwalib_metafits_context_new(
            metafits_file_ptr,
            MWAVersion::CorrLegacy,
            &mut metafits_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_metafits_context_new
        assert_eq!(retval, 0, "mwalib_metafits_context_new failure");

        // Check we got valid MetafitsContext pointer
        let context_ptr = metafits_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Now ensure we can free the rust memory
        assert_eq!(mwalib_metafits_context_free(context_ptr.unwrap()), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_metafits_context_free(std::ptr::null_mut()), 0);
    }
}

#[test]
fn test_mwalib_metafits_context_new_invalid() {
    // This tests for an invalid metafitscontext (missing file)
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/missing_file.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    unsafe {
        // Create a MetafitsContext
        let mut metafits_context_ptr: *mut MetafitsContext = std::ptr::null_mut();
        let retval = mwalib_metafits_context_new(
            metafits_file_ptr,
            MWAVersion::CorrLegacy,
            &mut metafits_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_metafits_context_new
        assert_ne!(retval, 0);

        // get error message
        let mut ret_error_message: String = String::new();

        if retval != 0 {
            let c_str: &CStr = CStr::from_ptr(error_message_ptr);
            let str_slice: &str = c_str.to_str().unwrap();
            str_slice.clone_into(&mut ret_error_message);
        }

        // Check error message
        assert!(!ret_error_message.is_empty());
    }
}

#[test]
fn test_mwalib_metafits_context_display() {
    let metafits_context_ptr: *mut MetafitsContext =
        get_test_ffi_metafits_context(MWAVersion::CorrLegacy);

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    unsafe {
        let retval =
            mwalib_metafits_context_display(metafits_context_ptr, error_message_ptr, error_len);

        assert_eq!(retval, 0);
    }
}

#[test]
fn test_mwalib_metafits_context_new_guess_mwa_version() {
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    unsafe {
        // Create a MetafitsContext
        let mut metafits_context_ptr: *mut MetafitsContext = std::ptr::null_mut();
        let retval = mwalib_metafits_context_new2(
            metafits_file_ptr,
            &mut metafits_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_metafits_context_new
        assert_eq!(retval, 0, "mwalib_metafits_context_new failure");

        // Check we got valid MetafitsContext pointer
        let context_ptr = metafits_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        let metafits_context = context_ptr.unwrap();

        assert_eq!(
            metafits_context.mwa_version.unwrap(),
            MWAVersion::CorrLegacy
        );
    }
}

#[test]
fn test_mwalib_metafits_context_display_null_ptr() {
    let metafits_context_ptr: *mut MetafitsContext = std::ptr::null_mut();

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    unsafe {
        let retval =
            mwalib_metafits_context_display(metafits_context_ptr, error_message_ptr, error_len);

        assert_ne!(retval, 0);
    }
}

//
// CorrelatorContext Tests
//
#[test]
fn test_mwalib_correlator_context_new_valid() {
    // This tests for a valid correlator context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file =
        CString::new("test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits")
            .unwrap();
    let gpubox_files: Vec<*const c_char> = vec![gpubox_file.as_ptr()];

    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

    unsafe {
        // Create a CorrelatorContext
        let mut correlator_context_ptr: *mut CorrelatorContext = std::ptr::null_mut();
        let retval = mwalib_correlator_context_new(
            metafits_file_ptr,
            gpubox_files_ptr,
            1,
            &mut correlator_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_correlator_context_new
        assert_eq!(retval, 0, "mwalib_correlator_context_new failure");

        // Check we got valid CorrelatorContext pointer
        let context_ptr = correlator_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Now ensure we can free the rust memory
        assert_eq!(mwalib_correlator_context_free(context_ptr.unwrap()), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_correlator_context_free(std::ptr::null_mut()), 0);
    }
}

#[test]
fn test_mwalib_correlator_context_new_valid_free() {
    // This tests for a valid correlator context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file =
        CString::new("test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits")
            .unwrap();
    let gpubox_files: Vec<*const c_char> = vec![gpubox_file.as_ptr()];

    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

    unsafe {
        // Create a CorrelatorContext
        let mut correlator_context_ptr: *mut CorrelatorContext = std::ptr::null_mut();
        let retval = mwalib_correlator_context_new(
            metafits_file_ptr,
            gpubox_files_ptr,
            1,
            &mut correlator_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_correlator_context_new
        assert_eq!(retval, 0, "mwalib_correlator_context_new failure");

        // Check we got valid CorrelatorContext pointer
        let context_ptr = correlator_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Now ensure we can free the rust memory
        assert_eq!(mwalib_correlator_context_free(context_ptr.unwrap()), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_correlator_context_free(std::ptr::null_mut()), 0);
    }
}

#[test]
fn test_mwalib_correlator_context_new_invalid() {
    // This tests for a invalid correlator context (missing file)
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/invalid_filename.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file =
        CString::new("test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits")
            .unwrap();
    let gpubox_files: Vec<*const c_char> = vec![gpubox_file.as_ptr()];

    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

    unsafe {
        // Create a CorrelatorContext
        let mut correlator_context_ptr: *mut CorrelatorContext = std::ptr::null_mut();
        let retval = mwalib_correlator_context_new(
            metafits_file_ptr,
            gpubox_files_ptr,
            1,
            &mut correlator_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_correlator_context_new
        assert_ne!(retval, 0);

        // Get error message
        let mut ret_error_message: String = String::new();

        if retval != 0 {
            let c_str: &CStr = CStr::from_ptr(error_message_ptr);
            let str_slice: &str = c_str.to_str().unwrap();
            str_slice.clone_into(&mut ret_error_message);
        }

        // Check error message
        assert!(!ret_error_message.is_empty());
    }
}

#[test]
fn test_mwalib_correlator_context_display() {
    let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context_legacy();

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    unsafe {
        let retval =
            mwalib_correlator_context_display(correlator_context_ptr, error_message_ptr, error_len);

        assert_eq!(retval, 0);
    }
}

#[test]
fn test_mwalib_correlator_context_display_null_ptr() {
    let correlator_context_ptr: *mut CorrelatorContext = std::ptr::null_mut();

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    unsafe {
        let retval =
            mwalib_correlator_context_display(correlator_context_ptr, error_message_ptr, error_len);

        assert_ne!(retval, 0);
    }
}

#[test]
fn test_mwalib_correlator_context_legacy_read_by_baseline_valid() {
    let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context_legacy();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let timestep_index = 0;
    let coarse_chan_index = 0;

    let buffer_len = 8256 * 128 * 8;
    unsafe {
        let buffer: Vec<f32> = vec![0.0; buffer_len];
        let buffer_ptr: *mut f32 = ffi_array_to_boxed_slice(buffer);

        let retval = mwalib_correlator_context_read_by_baseline(
            correlator_context_ptr,
            timestep_index,
            coarse_chan_index,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        assert_eq!(retval, 0);

        // Reconstitute the buffer
        let ret_buffer: Vec<f32> = ffi_boxed_slice_to_array(buffer_ptr, buffer_len);
        assert!(
            approx_eq!(f32, ret_buffer[0], 73189.0, F32Margin::default()),
            "Expected value was {}, should be {}",
            ret_buffer[0],
            73189.0
        );
        assert!(
            approx_eq!(f32, ret_buffer[100], -1482.5, F32Margin::default()),
            "Expected value was {}, should be {}",
            ret_buffer[100],
            -1482.5
        );
        assert!(
            approx_eq!(f32, ret_buffer[1016], 74300.5, F32Margin::default()),
            "Expected value was {}, should be {}",
            ret_buffer[1016],
            74300.5
        );
        assert!(
            approx_eq!(f32, ret_buffer[8385552], -174.5, F32Margin::default()),
            "Expected value was {}, should be {}",
            ret_buffer[8385552],
            -174.5
        );
    }
}

#[test]
fn test_mwalib_correlator_context_legacy_read_by_baseline_null_context() {
    let correlator_context_ptr: *mut CorrelatorContext = std::ptr::null_mut();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let timestep_index = 0;
    let coarse_chan_index = 0;

    let buffer_len = 8256 * 128 * 8;
    unsafe {
        let buffer: Vec<f32> = vec![0.0; buffer_len];
        let buffer_ptr: *mut f32 = ffi_array_to_boxed_slice(buffer);

        let retval = mwalib_correlator_context_read_by_baseline(
            correlator_context_ptr,
            timestep_index,
            coarse_chan_index,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        // Should get a non-zero return code
        assert_ne!(retval, 0);
    }
}

#[test]
fn test_mwalib_correlator_context_legacy_read_by_baseline_null_buffer() {
    let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context_legacy();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let timestep_index = 0;
    let coarse_chan_index = 0;

    let buffer_len = 8256 * 128 * 8;
    unsafe {
        let buffer_ptr: *mut f32 = std::ptr::null_mut();

        let retval = mwalib_correlator_context_read_by_baseline(
            correlator_context_ptr,
            timestep_index,
            coarse_chan_index,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        // Should get non zero return code
        assert_ne!(retval, 0);
    }
}

#[test]
fn test_mwalib_correlator_context_legacy_read_by_frequency_valid() {
    let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context_legacy();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let timestep_index = 0;
    let coarse_chan_index = 0;

    let buffer_len = 8256 * 128 * 8;
    unsafe {
        let buffer: Vec<f32> = vec![0.0; buffer_len];
        let buffer_ptr: *mut f32 = ffi_array_to_boxed_slice(buffer);

        let retval = mwalib_correlator_context_read_by_frequency(
            correlator_context_ptr,
            timestep_index,
            coarse_chan_index,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        assert_eq!(retval, 0);

        // Reconstitute the buffer
        let ret_buffer: Vec<f32> = ffi_boxed_slice_to_array(buffer_ptr, buffer_len);
        assert!(
            approx_eq!(f32, ret_buffer[0], 73189.0, F32Margin::default()),
            "Expected value was {}, should be {}",
            ret_buffer[0],
            73189.0
        );
        assert!(
            approx_eq!(f32, ret_buffer[100], 112.0, F32Margin::default()),
            "Expected value was {}, should be {}",
            ret_buffer[100],
            -1844.5
        );
        assert!(
            approx_eq!(f32, ret_buffer[1016], 205.5, F32Margin::default()),
            "Expected value was {}, should be {}",
            ret_buffer[1016],
            205.5
        );
        assert!(
            approx_eq!(f32, ret_buffer[8385552], -178.0, F32Margin::default()),
            "Expected value was {}, should be {}",
            ret_buffer[8385552],
            -178.0
        );
    }
}

#[test]
fn test_mwalib_correlator_context_legacy_read_by_frequency_null_context() {
    let correlator_context_ptr: *mut CorrelatorContext = std::ptr::null_mut();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let timestep_index = 0;
    let coarse_chan_index = 0;

    let buffer_len = 8256 * 128 * 8;
    unsafe {
        let buffer: Vec<f32> = vec![0.0; buffer_len];
        let buffer_ptr: *mut f32 = ffi_array_to_boxed_slice(buffer);

        let retval = mwalib_correlator_context_read_by_frequency(
            correlator_context_ptr,
            timestep_index,
            coarse_chan_index,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        // Should get a non-zero return code
        assert_ne!(retval, 0);
    }
}

#[test]
fn test_mwalib_correlator_context_legacy_read_by_frequency_null_buffer() {
    let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context_legacy();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let timestep_index = 0;
    let coarse_chan_index = 0;

    let buffer_len = 8256 * 128 * 8;
    unsafe {
        let buffer_ptr: *mut f32 = std::ptr::null_mut();

        let retval = mwalib_correlator_context_read_by_frequency(
            correlator_context_ptr,
            timestep_index,
            coarse_chan_index,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        // Should get no zero return code
        assert_ne!(retval, 0);
    }
}

#[test]
fn test_mwalib_correlator_context_read_weights_by_baseline_valid() {
    let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context_mwax();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let timestep_index = 0;
    let coarse_chan_index = 10;

    let buffer_len = 8256 * 4;
    unsafe {
        let buffer: Vec<f32> = vec![0.0; buffer_len];
        let buffer_ptr: *mut f32 = ffi_array_to_boxed_slice(buffer);

        let retval = mwalib_correlator_context_read_weights_by_baseline(
            correlator_context_ptr,
            timestep_index,
            coarse_chan_index,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        assert_eq!(retval, 0);

        // Reconstitute the buffer
        let ret_buffer: Vec<f32> = ffi_boxed_slice_to_array(buffer_ptr, buffer_len);
        assert!(
            approx_eq!(f32, ret_buffer[0], 1.0, F32Margin::default()),
            "Expected value was {}, should be {}",
            ret_buffer[0],
            1.0
        );
        assert!(
            approx_eq!(f32, ret_buffer[100], 1.0, F32Margin::default()),
            "Expected value was {}, should be {}",
            ret_buffer[100],
            1.0
        );
        assert!(
            approx_eq!(f32, ret_buffer[1016], 1.0, F32Margin::default()),
            "Expected value was {}, should be {}",
            ret_buffer[1016],
            1.0
        );
        assert!(
            approx_eq!(f32, ret_buffer[32023], 1.0, F32Margin::default()),
            "Expected value was {}, should be {}",
            ret_buffer[32023],
            1.0
        );
    }
}

#[test]
fn test_mwalib_correlator_context_read_weights_by_baseline_null_context() {
    let correlator_context_ptr: *mut CorrelatorContext = std::ptr::null_mut();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let timestep_index = 0;
    let coarse_chan_index = 10;

    let buffer_len = 8256 * 4;
    unsafe {
        let buffer: Vec<f32> = vec![0.0; buffer_len];
        let buffer_ptr: *mut f32 = ffi_array_to_boxed_slice(buffer);

        let retval = mwalib_correlator_context_read_weights_by_baseline(
            correlator_context_ptr,
            timestep_index,
            coarse_chan_index,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        // Should get a non-zero return code
        assert_ne!(retval, 0);
    }
}

#[test]
fn test_mwalib_correlator_context_read_weights_by_baseline_null_buffer() {
    let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context_mwax();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let timestep_index = 0;
    let coarse_chan_index = 10;

    let buffer_len = 8256 * 4;
    unsafe {
        let buffer_ptr: *mut f32 = std::ptr::null_mut();

        let retval = mwalib_correlator_context_read_weights_by_baseline(
            correlator_context_ptr,
            timestep_index,
            coarse_chan_index,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        // Should get non zero return code
        assert_ne!(retval, 0);
    }
}

#[test]
fn test_mwalib_correlator_context_get_fine_chan_freqs_hz_array_valid() {
    let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context_legacy();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    unsafe {
        let chan_indices_len: usize = 1;
        let chan_indicies: Vec<usize> = vec![0];
        let chan_indicies_ptr: *mut usize = ffi_array_to_boxed_slice(chan_indicies);

        let buffer_len = 128;
        let buffer: Vec<f64> = vec![0.0; buffer_len];
        let buffer_ptr: *mut f64 = ffi_array_to_boxed_slice(buffer);

        let retval = mwalib_correlator_context_get_fine_chan_freqs_hz_array(
            correlator_context_ptr,
            chan_indicies_ptr,
            chan_indices_len,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        // Should get zero return code
        assert_eq!(retval, 0);

        // Reconstitute the buffer
        let ret_buffer: Vec<f64> = ffi_boxed_slice_to_array(buffer_ptr, buffer_len);

        // Check values
        assert_eq!(ret_buffer.len(), buffer_len);

        assert!(approx_eq!(
            f64,
            ret_buffer[0],
            138_880_000.0,
            F64Margin::default()
        ));
    }
}

#[test]
fn test_mwalib_correlator_context_get_fine_chan_freqs_hz_array_invalid_buffer_len() {
    let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context_legacy();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    unsafe {
        let chan_indices_len: usize = 1;
        let chan_indicies: Vec<usize> = vec![0];
        let chan_indicies_ptr: *mut usize = ffi_array_to_boxed_slice(chan_indicies);

        // Invalid buffer - too big
        let buffer_len = 129;
        let buffer: Vec<f64> = vec![0.0; buffer_len];
        let buffer_ptr: *mut f64 = ffi_array_to_boxed_slice(buffer);

        let retval = mwalib_correlator_context_get_fine_chan_freqs_hz_array(
            correlator_context_ptr,
            chan_indicies_ptr,
            chan_indices_len,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        // Should get non-zero return code
        assert_ne!(retval, 0);

        //
        // Invalid buffer - too small
        //
        let buffer_len = 127;
        let buffer: Vec<f64> = vec![0.0; buffer_len];
        let buffer_ptr: *mut f64 = ffi_array_to_boxed_slice(buffer);

        let retval = mwalib_correlator_context_get_fine_chan_freqs_hz_array(
            correlator_context_ptr,
            chan_indicies_ptr,
            chan_indices_len,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        // Should get non-zero return code
        assert_ne!(retval, 0);
    }
}

#[test]
fn test_mwalib_correlator_context_get_fine_chan_freqs_hz_array_null_context() {
    let correlator_context_ptr: *mut CorrelatorContext = std::ptr::null_mut();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    unsafe {
        // Null context
        let chan_indices_len: usize = 1;
        let chan_indicies: Vec<usize> = vec![0];
        let chan_indicies_ptr: *mut usize = ffi_array_to_boxed_slice(chan_indicies);

        let buffer_len = 128;
        let buffer: Vec<f64> = vec![0.0; buffer_len];
        let buffer_ptr: *mut f64 = ffi_array_to_boxed_slice(buffer);

        let retval = mwalib_correlator_context_get_fine_chan_freqs_hz_array(
            correlator_context_ptr,
            chan_indicies_ptr,
            chan_indices_len,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        // Should get non-zero return code
        assert_ne!(retval, 0);
    }
}

#[test]
fn test_mwalib_correlator_context_get_fine_chan_freqs_hz_array_null_coarse_chans() {
    let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context_legacy();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    unsafe {
        // Null coarse chans
        let chan_indices_len: usize = 1;
        let chan_indicies_ptr: *mut usize = std::ptr::null_mut();

        let buffer_len = 128;
        let buffer: Vec<f64> = vec![0.0; buffer_len];
        let buffer_ptr: *mut f64 = ffi_array_to_boxed_slice(buffer);

        let retval = mwalib_correlator_context_get_fine_chan_freqs_hz_array(
            correlator_context_ptr,
            chan_indicies_ptr,
            chan_indices_len,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        // Should get non-zero return code
        assert_ne!(retval, 0);
    }
}

#[test]
fn test_mwalib_correlator_context_get_fine_chan_freqs_hz_array_null_buffer() {
    let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context_legacy();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    unsafe {
        let chan_indices_len: usize = 1;
        let chan_indicies: Vec<usize> = vec![0];
        let chan_indicies_ptr: *mut usize = ffi_array_to_boxed_slice(chan_indicies);

        // Null buffer ptr
        let buffer_len = 128;
        let buffer_ptr: *mut f64 = std::ptr::null_mut();

        let retval = mwalib_correlator_context_get_fine_chan_freqs_hz_array(
            correlator_context_ptr,
            chan_indicies_ptr,
            chan_indices_len,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        // Should get non-zero return code
        assert_ne!(retval, 0);
    }
}

//
// Metafits Metadata Tests
//
#[test]
fn test_mwalib_metafits_metadata_get_from_metafits_context_get_and_free() {
    // This tests for a valid metafits context and metadata returned
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;
    // Create a MetafitsContext
    let metafits_context_ptr: *mut MetafitsContext =
        get_test_ffi_metafits_context(MWAVersion::CorrLegacy);
    unsafe {
        // Check we got valid MetafitsContext pointer
        let context_ptr = metafits_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: *mut MetafitsMetadata = std::ptr::null_mut();
        let retval = mwalib_metafits_metadata_get(
            metafits_context_ptr,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut metafits_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        let mut ret_error_message: String = String::new();

        if retval != 0 {
            let c_str: &CStr = CStr::from_ptr(error_message_ptr);
            let str_slice: &str = c_str.to_str().unwrap();
            str_slice.clone_into(&mut ret_error_message);
        }
        assert_eq!(
            retval, 0,
            "mwalib_metafits_metadata_get failure {}",
            ret_error_message
        );

        // Get the mwalibMetadata struct from the pointer
        let metafits_metadata = Box::from_raw(metafits_metadata_ptr);

        // Now ensure we can free the rust memory
        assert_eq!(
            mwalib_metafits_metadata_free(Box::into_raw(metafits_metadata)),
            0
        );

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_metafits_metadata_free(std::ptr::null_mut()), 0);
    }
}

#[test]
fn test_mwalib_metafits_metadata_get_from_metafits_context_valid() {
    // This tests for a valid metafits context and metadata returned
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;
    // Create a MetafitsContext
    let metafits_context_ptr: *mut MetafitsContext =
        get_test_ffi_metafits_context(MWAVersion::CorrLegacy);
    unsafe {
        // Check we got valid MetafitsContext pointer
        let context_ptr = metafits_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: *mut MetafitsMetadata = std::ptr::null_mut();
        let retval = mwalib_metafits_metadata_get(
            metafits_context_ptr,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut metafits_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        let mut ret_error_message: String = String::new();

        if retval != 0 {
            let c_str: &CStr = CStr::from_ptr(error_message_ptr);
            let str_slice: &str = c_str.to_str().unwrap();
            str_slice.clone_into(&mut ret_error_message);
        }
        assert_eq!(
            retval, 0,
            "mwalib_metafits_metadata_get failure {}",
            ret_error_message
        );

        // Get the mwalibMetadata struct from the pointer
        let metafits_metadata = Box::from_raw(metafits_metadata_ptr);

        // We should get a valid obsid and no error message
        assert_eq!(metafits_metadata.obs_id, 1_101_503_312);

        //
        // Test baselines by reconstituting into a vector we can test
        //
        let item: Vec<baseline::ffi::Baseline> =
            ffi_boxed_slice_to_array(metafits_metadata.baselines, metafits_metadata.num_baselines);

        // Test specific values
        assert_eq!(item.len(), 8256, "Array length is not correct");
        assert_eq!(item[2].ant1_index, 0);
        assert_eq!(item[2].ant2_index, 2);

        //
        // Test antennas
        //
        let item: Vec<Antenna> =
            ffi_boxed_slice_to_array(metafits_metadata.antennas, metafits_metadata.num_ants);

        assert_eq!(item.len(), 128, "Array length is not correct");
        assert_eq!(
            CString::from_raw(item[127].tile_name),
            CString::new("Tile168").unwrap()
        );
        assert_eq!(item[2].tile_id, 13);

        //
        // Test rf inputs
        //
        let item: Vec<rfinput::ffi::Rfinput> =
            ffi_boxed_slice_to_array(metafits_metadata.rf_inputs, metafits_metadata.num_rf_inputs);

        assert_eq!(item.len(), 256, "Array length is not correct");
        assert_eq!(item[2].ant, 1);
        assert_eq!(
            CString::from_raw(item[2].tile_name),
            CString::new("Tile012").unwrap()
        );
        assert_eq!(CString::from_raw(item[2].pol), CString::new("X").unwrap());

        assert_eq!(item[2].num_digital_gains, 24);
        let rfinput_digital_gains =
            ffi_boxed_slice_to_array(item[2].digital_gains, item[2].num_digital_gains);
        assert_eq!(item[2].num_digital_gains, rfinput_digital_gains.len());
        assert!(item[2].calib_delay.is_nan());
        assert_eq!(item[2].num_calib_gains, 24);
        // no signal chain corrections in this metafits, so it should = 256
        assert_eq!(
            item[0].signal_chain_corrections_index,
            MAX_RECEIVER_CHANNELS
        );

        assert!(approx_eq!(
            f64,
            rfinput_digital_gains[4],
            76. / 64.,
            F64Margin::default()
        ));

        assert_eq!(item[2].num_dipole_delays, 16);
        let rfinput_dipole_delays =
            ffi_boxed_slice_to_array(item[2].dipole_delays, item[2].num_dipole_delays);
        assert_eq!(item[2].num_dipole_delays, rfinput_dipole_delays.len());
        assert_eq!(rfinput_dipole_delays[0], 0);

        assert_eq!(item[2].num_dipole_gains, 16);
        let rfinput_dipole_gains =
            ffi_boxed_slice_to_array(item[2].dipole_gains, item[2].num_dipole_gains);
        assert_eq!(item[2].num_dipole_gains, rfinput_dipole_gains.len());
        assert!(approx_eq!(
            f64,
            rfinput_dipole_gains[0],
            1.0,
            F64Margin::default()
        ));
        assert_eq!(item[2].rec_type, ReceiverType::Unknown);

        //
        // Test metafits_coarse_channels
        //
        let item: Vec<coarse_channel::ffi::CoarseChannel> = ffi_boxed_slice_to_array(
            metafits_metadata.metafits_coarse_chans,
            metafits_metadata.num_metafits_coarse_chans,
        );
        assert_eq!(item.len(), 24, "Array length is not correct");
        assert_eq!(item[0].rec_chan_number, 109);

        //
        // Test metafits_timesteps
        //
        let item: Vec<timestep::ffi::TimeStep> = ffi_boxed_slice_to_array(
            metafits_metadata.metafits_timesteps,
            metafits_metadata.num_metafits_timesteps,
        );
        assert_eq!(item.len(), 56, "Array length is not correct");
        assert_eq!(item[0].unix_time_ms, 1_417_468_096_000);
        assert_eq!(item[55].unix_time_ms, 1_417_468_206_000);

        // Test oversample flag
        assert!(!metafits_metadata.oversampled);

        // test deripple
        assert!(!metafits_metadata.deripple_applied);
        assert_eq!(
            CString::from_raw(metafits_metadata.deripple_param),
            CString::new("").unwrap()
        );

        // test signal chain corrections (should be empty array)
        if metafits_metadata.num_signal_chain_corrections > 0 {
            let sig_chain_corrs = ffi_boxed_slice_to_array(
                metafits_metadata.signal_chain_corrections,
                metafits_metadata.num_signal_chain_corrections,
            );
            assert_eq!(
                sig_chain_corrs.len(),
                metafits_metadata.num_signal_chain_corrections
            );
        }

        // Note- don't try to do any free's here since, in order to test, we have had to reconstituded some of the arrays which will result in a double free
    }
}

#[test]
fn test_mwalib_metafits_metadata_get_from_metafits_context_legacy_vcs_valid() {
    // This tests for a valid metafits context and metadata returned
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;
    // Create a MetafitsContext
    let metafits_context_ptr: *mut MetafitsContext =
        get_test_ffi_metafits_context(MWAVersion::VCSLegacyRecombined);
    unsafe {
        // Check we got valid MetafitsContext pointer
        let context_ptr = metafits_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: *mut MetafitsMetadata = std::ptr::null_mut();
        let retval = mwalib_metafits_metadata_get(
            metafits_context_ptr,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut metafits_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        let mut ret_error_message: String = String::new();

        if retval != 0 {
            let c_str: &CStr = CStr::from_ptr(error_message_ptr);
            let str_slice: &str = c_str.to_str().unwrap();
            str_slice.clone_into(&mut ret_error_message);
        }
        assert_eq!(
            retval, 0,
            "mwalib_metafits_metadata_get failure {}",
            ret_error_message
        );

        // Get the mwalibMetadata struct from the pointer
        let metafits_metadata = Box::from_raw(metafits_metadata_ptr);

        //
        // Test antennas
        //
        let items: Vec<Antenna> =
            ffi_boxed_slice_to_array(metafits_metadata.antennas, metafits_metadata.num_ants);

        assert_eq!(items.len(), 128, "Array length is not correct");

        for item in items {
            if item.tile_id == 154 {
                assert_eq!(item.rfinput_y, 1);
            } else if item.tile_id == 104 {
                assert_eq!(item.rfinput_y, 0);
            }
        }
    }
}

#[test]
fn test_mwalib_metafits_metadata_get_null_contexts() {
    // This tests for a null context passed to mwalib_metafits_metadata_get
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    unsafe {
        let mut metafits_metadata_ptr: *mut MetafitsMetadata = std::ptr::null_mut();
        let ret_val = mwalib_metafits_metadata_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut metafits_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // We should get a non-zero return code
        assert_ne!(ret_val, 0);
    }
}

#[test]
fn test_mwalib_metafits_metadata_get_from_correlator_context_valid() {
    // This tests for a valid metafits metadata returned given a correlator context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    unsafe {
        // Create a CorrelatorContext
        let correlator_context_ptr: *mut CorrelatorContext =
            get_test_ffi_correlator_context_legacy();

        // Check we got valid MetafitsContext pointer
        let context_ptr = correlator_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: *mut MetafitsMetadata = std::ptr::null_mut();
        let retval = mwalib_metafits_metadata_get(
            std::ptr::null_mut(),
            correlator_context_ptr,
            std::ptr::null_mut(),
            &mut metafits_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        assert_eq!(
            retval, 0,
            "mwalib_metafits_metadata_get did not return success"
        );

        // Get the mwalibMetadata struct from the pointer
        let metafits_metadata = Box::from_raw(metafits_metadata_ptr);

        // We should get a valid obsid and no error message
        assert_eq!(metafits_metadata.obs_id, 1_101_503_312);

        // Now ensure we can free the rust memory
        assert_eq!(
            mwalib_metafits_metadata_free(Box::into_raw(metafits_metadata)),
            0
        );

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_metafits_metadata_free(std::ptr::null_mut()), 0);
    }
}



#[test]
fn test_mwalib_correlator_metadata_get_valid() {
    // This tests for a valid correlator metadata struct being instantiated
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    unsafe {
        // Create a CorrelatorContext
        let correlator_context_ptr: *mut CorrelatorContext =
            get_test_ffi_correlator_context_legacy();

        // Check we got valid MetafitsContext pointer
        let context_ptr = correlator_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a CorrelatorMetadata struct
        let mut correlator_metadata_ptr: *mut CorrelatorMetadata = std::ptr::null_mut();
        let retval = mwalib_correlator_metadata_get(
            correlator_context_ptr,
            &mut correlator_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        assert_eq!(
            retval, 0,
            "mwalib_correlator_metadata_get did not return success"
        );

        // Get the correlator metadata struct from the pointer
        let mut correlator_metadata = Box::from_raw(correlator_metadata_ptr);

        // We should get a valid number of coarse channels and no error message
        assert_eq!(correlator_metadata.num_coarse_chans, 24);

        // reconstitute into a vector
        let item: Vec<timestep::ffi::TimeStep> = ffi_boxed_slice_to_array(
            correlator_metadata.timesteps,
            correlator_metadata.num_timesteps,
        );

        // We should get a valid, populated array
        assert_eq!(
            correlator_metadata.num_timesteps, 56,
            "Array length is not correct"
        );
        assert_eq!(item[0].unix_time_ms, 1_417_468_096_000);

        // Check bscale
        assert_eq!(correlator_metadata.bscale, 0.5);

        // So that the next free works, we set the pointer to null (the ffi_boxed_slice_to_array effectively freed the timestep array memory - as far as C/FFI is concerned)
        correlator_metadata.timesteps = std::ptr::null_mut();

        // Now ensure we can free the rust memory
        assert_eq!(
            mwalib_correlator_metadata_free(Box::into_raw(correlator_metadata)),
            0
        );

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_correlator_metadata_free(std::ptr::null_mut()), 0);
    }
}

#[test]
fn test_mwalib_correlator_metadata_get_null_context() {
    // This tests for passing a null context to the mwalib_correlator_metadata_get() method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    unsafe {
        let mut correlator_metadata_ptr: *mut CorrelatorMetadata = std::ptr::null_mut();

        let context_ptr = std::ptr::null_mut();
        let ret_val = mwalib_correlator_metadata_get(
            context_ptr,
            &mut correlator_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // We should get a non-zero return code
        assert_ne!(ret_val, 0);
    }
}

#[test]
fn test_mwalib_voltage_metadata_get_valid() {
    // This tests for a valid voltage metadata struct being instantiated
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    unsafe {
        // Create a VoltageContext
        let voltage_context_ptr: *mut VoltageContext =
            get_test_ffi_voltage_context(MWAVersion::VCSLegacyRecombined, false);

        // Check we got valid MetafitsContext pointer
        let context_ptr = voltage_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a VoltageMetadata struct
        let mut voltage_metadata_ptr: *mut VoltageMetadata = std::ptr::null_mut();
        let retval = mwalib_voltage_metadata_get(
            voltage_context_ptr,
            &mut voltage_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        assert_eq!(
            retval, 0,
            "mwalib_voltage_metadata_get did not return success"
        );

        // Get the voltage metadata struct from the pointer
        let mut voltage_metadata = Box::from_raw(voltage_metadata_ptr);

        // We should get a valid number of coarse channels and no error message
        assert_eq!(voltage_metadata.num_coarse_chans, 24);

        // reconstitute into a vector
        let item: Vec<coarse_channel::ffi::CoarseChannel> = ffi_boxed_slice_to_array(
            voltage_metadata.coarse_chans,
            voltage_metadata.num_coarse_chans,
        );

        // We should have a valid, populated array
        assert_eq!(item[0].rec_chan_number, 109);
        assert_eq!(item[23].rec_chan_number, 132);

        // So that the next free works, we set the pointer to null (the ffi_boxed_slice_to_array effectively freed the coarse_chan array memory - as far as C/FFI is concerned)
        voltage_metadata.coarse_chans = std::ptr::null_mut();

        // Now ensure we can free the rust memory
        assert_eq!(
            mwalib_voltage_metadata_free(Box::into_raw(voltage_metadata)),
            0
        );

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_voltage_metadata_free(std::ptr::null_mut()), 0);
    }
}

#[test]
fn test_mwalib_voltage_metadata_get_null_context() {
    // This tests for passing a null context to the mwalib_voltage_metadata_get() method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    unsafe {
        let mut voltage_metadata_ptr: *mut VoltageMetadata = std::ptr::null_mut();

        let context_ptr = std::ptr::null_mut();
        let ret_val = mwalib_voltage_metadata_get(
            context_ptr,
            &mut voltage_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // We should get a non-zero return code
        assert_ne!(ret_val, 0);
    }
}

#[test]
fn test_calibration_hdu_in_metafits() {
    // This tests for a valid metafits context and metadata returned
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;
    // Create a MetafitsContext
    let metafits_context_ptr: *mut MetafitsContext = get_test_ffi_metafits_context_ext(
        MWAVersion::CorrLegacy,
        String::from("test_files/metafits_cal_sol/1111842752_metafits.fits"),
    );

    // Check we got valid MetafitsContext pointer
    let context_ptr = unsafe { metafits_context_ptr.as_mut() };
    assert!(context_ptr.is_some());

    let context = context_ptr.unwrap();
    assert_eq!(context.num_rf_inputs, 256);
    assert_eq!(context.num_ants, 128);

    assert_eq!(context.rf_inputs.len(), 256);
    assert_eq!(context.antennas.len(), 128);

    unsafe {
        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: *mut MetafitsMetadata = std::ptr::null_mut();
        let retval = mwalib_metafits_metadata_get(
            metafits_context_ptr,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut metafits_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        let mut ret_error_message: String = String::new();

        if retval != 0 {
            let c_str: &CStr = CStr::from_ptr(error_message_ptr);
            let str_slice: &str = c_str.to_str().unwrap();
            str_slice.clone_into(&mut ret_error_message);
        }

        assert_eq!(
            retval, 0,
            "mwalib_metafits_metadata_get failure {}",
            ret_error_message
        );

        // Get the mwalibMetadata struct from the pointer
        let metafits_metadata = Box::from_raw(metafits_metadata_ptr);

        // We should get a valid obsid and no error message
        assert_eq!(metafits_metadata.obs_id, 1_111_842_752);
        assert_eq!(metafits_metadata.best_cal_fit_id, 1720774022);
        assert_eq!(metafits_metadata.best_cal_obs_id, 1111842752);
        assert_eq!(
            CString::from_raw(metafits_metadata.best_cal_code_ver),
            CString::new("0.17.22").unwrap()
        );
        assert_eq!(
            CString::from_raw(metafits_metadata.best_cal_fit_timestamp),
            CString::new("2024-07-12T08:47:02.308203+00:00").unwrap()
        );
        assert_eq!(
            CString::from_raw(metafits_metadata.best_cal_creator),
            CString::new("calvin").unwrap()
        );
        assert_eq!(metafits_metadata.best_cal_fit_iters, 3);
        assert_eq!(metafits_metadata.best_cal_fit_iter_limit, 20);
    }
}

#[test]
fn test_calibration_hdu_not_in_metafits() {
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    // Create a MetafitsContext
    let metafits_context_ptr: *mut MetafitsContext =
        get_test_ffi_metafits_context(MWAVersion::CorrLegacy);

    unsafe {
        // Check we got valid MetafitsContext pointer
        let context_ptr = metafits_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: *mut MetafitsMetadata = std::ptr::null_mut();
        let retval = mwalib_metafits_metadata_get(
            metafits_context_ptr,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut metafits_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        let mut ret_error_message: String = String::new();

        if retval != 0 {
            let c_str: &CStr = CStr::from_ptr(error_message_ptr);
            let str_slice: &str = c_str.to_str().unwrap();
            str_slice.clone_into(&mut ret_error_message);
        }
        assert_eq!(
            retval, 0,
            "mwalib_metafits_metadata_get failure {}",
            ret_error_message
        );

        // Get the mwalibMetadata struct from the pointer
        let metafits_metadata = Box::from_raw(metafits_metadata_ptr);

        // We should get a valid obsid and no error message
        assert_eq!(metafits_metadata.obs_id, 1_101_503_312);
        assert_eq!(metafits_metadata.best_cal_fit_id, 0);
        assert_eq!(metafits_metadata.best_cal_obs_id, 0);
        assert_eq!(
            CString::from_raw(metafits_metadata.best_cal_code_ver),
            CString::new("").unwrap()
        );
        assert_eq!(
            CString::from_raw(metafits_metadata.best_cal_fit_timestamp),
            CString::new("").unwrap()
        );
        assert_eq!(
            CString::from_raw(metafits_metadata.best_cal_creator),
            CString::new("").unwrap()
        );
        assert_eq!(metafits_metadata.best_cal_fit_iters, 0);
        assert_eq!(metafits_metadata.best_cal_fit_iter_limit, 0);
    }
}

#[test]
fn test_signal_chain_hdu_in_metafits() {
    // This tests for a valid metafits context and metadata returned
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;
    // Create a MetafitsContext
    let metafits_context_ptr: *mut MetafitsContext = get_test_ffi_metafits_context_ext(
        MWAVersion::CorrLegacy,
        String::from("test_files/metafits_signal_chain_corr/1096952256_metafits.fits"),
    );

    // Check we got valid MetafitsContext pointer
    let context_ptr = unsafe { metafits_context_ptr.as_mut() };
    assert!(context_ptr.is_some());

    unsafe {
        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: *mut MetafitsMetadata = std::ptr::null_mut();
        let retval = mwalib_metafits_metadata_get(
            metafits_context_ptr,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut metafits_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        let mut ret_error_message: String = String::new();

        if retval != 0 {
            let c_str: &CStr = CStr::from_ptr(error_message_ptr);
            let str_slice: &str = c_str.to_str().unwrap();
            str_slice.clone_into(&mut ret_error_message);
        }

        assert_eq!(
            retval, 0,
            "mwalib_metafits_metadata_get failure {}",
            ret_error_message
        );

        // Get the mwalibMetadata struct from the pointer
        let metafits_metadata = Box::from_raw(metafits_metadata_ptr);

        // We should get a valid obsid and no error message
        assert_eq!(metafits_metadata.obs_id, 1_096_952_256);

        assert_eq!(metafits_metadata.num_signal_chain_corrections, 8);
        let sig_chain_corr = ffi_boxed_slice_to_array(
            metafits_metadata.signal_chain_corrections,
            metafits_metadata.num_signal_chain_corrections,
        );

        assert_eq!(
            sig_chain_corr.len(),
            metafits_metadata.num_signal_chain_corrections
        );

        // First row is:
        // RRI                0  0.16073910960211837 .. 0.7598147243238643
        assert_eq!(sig_chain_corr[0].receiver_type, ReceiverType::RRI);
        assert!(!sig_chain_corr[0].whitening_filter);
        let sig_chain_corr_0_corrections =
            ffi_boxed_slice_to_array(sig_chain_corr[0].corrections, MAX_RECEIVER_CHANNELS);
        assert_eq!(sig_chain_corr_0_corrections[0], 0.16073910960211837);
        assert_eq!(sig_chain_corr_0_corrections[255], 0.7598147243238643);

        // 4th row is:
        // NI                 1   0.0 .. 0.0
        assert_eq!(sig_chain_corr[3].receiver_type, ReceiverType::NI);
        assert!(sig_chain_corr[3].whitening_filter);
        let sig_chain_corr_3_corrections =
            ffi_boxed_slice_to_array(sig_chain_corr[3].corrections, MAX_RECEIVER_CHANNELS);
        assert_eq!(sig_chain_corr_3_corrections[0], 0.0);
        assert_eq!(sig_chain_corr_3_corrections[255], 0.0);
    }
}

#[test]
fn test_signal_chain_hdu_not_in_metafits() {
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    // Create a MetafitsContext
    let metafits_context_ptr: *mut MetafitsContext =
        get_test_ffi_metafits_context(MWAVersion::CorrLegacy);

    unsafe {
        // Check we got valid MetafitsContext pointer
        let context_ptr = metafits_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: *mut MetafitsMetadata = std::ptr::null_mut();
        let retval = mwalib_metafits_metadata_get(
            metafits_context_ptr,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut metafits_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        let mut ret_error_message: String = String::new();

        if retval != 0 {
            let c_str: &CStr = CStr::from_ptr(error_message_ptr);
            let str_slice: &str = c_str.to_str().unwrap();
            str_slice.clone_into(&mut ret_error_message);
        }
        assert_eq!(
            retval, 0,
            "mwalib_metafits_metadata_get failure {}",
            ret_error_message
        );

        // Get the mwalibMetadata struct from the pointer
        let metafits_metadata = Box::from_raw(metafits_metadata_ptr);

        // We should get a valid obsid and no error message
        assert_eq!(metafits_metadata.obs_id, 1_101_503_312);
        assert_eq!(metafits_metadata.num_signal_chain_corrections, 0)
    }
}
