// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::*;
use crate::{
    correlator_context::ffi::{
        mwalib_correlator_context_display, mwalib_correlator_context_free,
        mwalib_correlator_context_get_fine_chan_freqs_hz_array, mwalib_correlator_context_new,
        mwalib_correlator_context_read_by_baseline, mwalib_correlator_context_read_by_frequency,
        mwalib_correlator_context_read_weights_by_baseline, mwalib_correlator_metadata_free,
        mwalib_correlator_metadata_get, CorrelatorMetadata,
    },
    ffi::{
        ffi_create_c_array,
        ffi_test_helpers::{
            ffi_boxed_slice_to_array, get_test_ffi_correlator_context_legacy,
            get_test_ffi_correlator_context_mwax,
        },
    },
};
use float_cmp::{approx_eq, F32Margin, F64Margin};
use libc::size_t;
use std::ffi::{c_char, CStr, CString};

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
    let buf_len: size_t = 1280;
    let buf_message = CString::new(" ".repeat(buf_len)).unwrap();
    let buf_message_ptr = buf_message.as_ptr() as *mut c_char;

    unsafe {
        let retval = mwalib_correlator_context_display(
            correlator_context_ptr,
            buf_message_ptr,
            buf_len,
            error_message_ptr,
            error_len,
        );

        assert_eq!(retval, 0);

        // Check that the first few chars are "CorrelatorContext ("
        let output_str = CStr::from_ptr(buf_message_ptr)
            .to_str()
            .expect("Error converting C string");
        assert!(output_str.starts_with("CorrelatorContext ("));
    }
}

#[test]
fn test_mwalib_correlator_context_display_null_ptr() {
    let correlator_context_ptr: *mut CorrelatorContext = std::ptr::null_mut();

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;
    let buf_len: size_t = 1280;
    let buf_message = CString::new(" ".repeat(buf_len)).unwrap();
    let buf_message_ptr = buf_message.as_ptr() as *mut c_char;

    unsafe {
        let retval = mwalib_correlator_context_display(
            correlator_context_ptr,
            buf_message_ptr,
            buf_len,
            error_message_ptr,
            error_len,
        );

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
        let (buffer_ptr, _) = ffi_create_c_array(buffer);

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
        let (buffer_ptr, _) = ffi_create_c_array(buffer);

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
        let (buffer_ptr, _) = ffi_create_c_array(buffer);

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
        let (buffer_ptr, _) = ffi_create_c_array(buffer);

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
        let (buffer_ptr, _) = ffi_create_c_array(buffer);

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
        let (buffer_ptr, _) = ffi_create_c_array(buffer);

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
        let (chan_indicies_ptr, _) = ffi_create_c_array(chan_indicies);

        let buffer_len = 128;
        let buffer: Vec<f64> = vec![0.0; buffer_len];
        let (buffer_ptr, _) = ffi_create_c_array(buffer);

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
        let (chan_indicies_ptr, _) = ffi_create_c_array(chan_indicies);

        // Invalid buffer - too big
        let buffer_len = 129;
        let buffer: Vec<f64> = vec![0.0; buffer_len];
        let (buffer_ptr, _) = ffi_create_c_array(buffer);

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
        let (buffer_ptr, _) = ffi_create_c_array(buffer);

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
        let (chan_indicies_ptr, _) = ffi_create_c_array(chan_indicies);

        let buffer_len = 128;
        let buffer: Vec<f64> = vec![0.0; buffer_len];
        let (buffer_ptr, _) = ffi_create_c_array(buffer);

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
        let (buffer_ptr, _) = ffi_create_c_array(buffer);

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
        let (chan_indicies_ptr, _) = ffi_create_c_array(chan_indicies);

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
