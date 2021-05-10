// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Unit tests for ffi module
*/
#[cfg(test)]
use super::*;
use float_cmp::*;
use voltage_context::test::get_test_voltage_context;

//
// Helper methods for many tests
//

/// Create and return a metafits context based on a test metafits file. Used in many tests in the module.
///
///
/// # Arguments
///
/// * None
///
///
/// # Returns
///
/// * a raw pointer to an instantiated MetafitsContext for the test metafits and gpubox file
///
#[cfg(test)]
fn get_test_ffi_metafits_context() -> *mut MetafitsContext {
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    unsafe {
        // Create a MetafitsContext
        let mut metafits_context_ptr: *mut MetafitsContext = std::ptr::null_mut();
        let retval = mwalib_metafits_context_new(
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

        context_ptr.unwrap()
    }
}

/// Create and return a correlator context ptr based on a test metafits and gpubox file. Used in many tests in the module.
///
///
/// # Arguments
///
/// * None
///
///
/// # Returns
///
/// * a raw pointer to an instantiated CorrelatorContext for the test metafits and gpubox file
///
#[cfg(test)]
fn get_test_ffi_correlator_context() -> *mut CorrelatorContext {
    // This tests for a valid correlator context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

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

        // Check we got valid MetafitsContext pointer
        let context_ptr = correlator_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        context_ptr.unwrap()
    }
}

/// Create and return a voltage context ptr based on a test metafits and voltage file. Used in many tests in the module.
///
///
/// # Arguments
///
/// * None
///
///
/// # Returns
///
/// * a raw pointer to an instantiated VoltageContext for the test metafits and voltage file
///
#[cfg(test)]
fn get_test_ffi_voltage_context(corr_version: CorrelatorVersion) -> *mut VoltageContext {
    // This returns a a valid voltage context
    let mut context = get_test_voltage_context(corr_version);

    //
    // In order for our smaller voltage files to work with this test we need to reset the voltage_block_size_bytes
    //
    context.voltage_block_size_bytes /= 128;

    Box::into_raw(Box::new(context))
}

/// Reconstructs a Vec<T> from FFI using a pointer to a rust-allocated array of *mut T.
///
///
/// # Arguments
///
/// * `ptr` - raw pointer pointing to an array of T
///
/// * 'len' - number of elements in the array
///
///
/// # Returns
///
/// * Array of T expressed as Vec<T>
///
#[cfg(test)]
fn ffi_boxed_slice_to_array<T>(ptr: *mut T, len: usize) -> Vec<T> {
    unsafe {
        let vec: Vec<T> = Vec::from_raw_parts(ptr, len, len);
        vec
    }
}

//
// Simple test of the error message helper
//
#[test]
fn test_set_error_message() {
    let buffer = CString::new("HELLO WORLD").unwrap();
    let buffer_ptr = buffer.as_ptr() as *mut u8;

    set_error_message("hello world", buffer_ptr, 12);

    assert_eq!(buffer, CString::new("hello world").unwrap());
}

#[test]
fn test_set_error_message_null_ptr() {
    let buffer_ptr: *mut u8 = std::ptr::null_mut();

    set_error_message("hello world", buffer_ptr, 12);
}

#[test]
fn test_set_error_message_buffer_len_too_small() {
    let buffer = CString::new("H").unwrap();
    let buffer_ptr = buffer.as_ptr() as *mut u8;

    set_error_message("hello world", buffer_ptr, 1);
}

#[test]
fn test_mwalib_free_rust_cstring() {
    let buffer = CString::new("HELLO WORLD").unwrap();
    let buffer_ptr = buffer.into_raw() as *mut i8;

    // into_raw will take garbage collection of the buffer away from rust, so
    // some ffi/C code can free it (like below)
    unsafe {
        assert_eq!(mwalib_free_rust_cstring(buffer_ptr), 0);
    }
}

#[test]
fn test_mwalib_free_rust_cstring_null_ptr() {
    let buffer_ptr: *mut i8 = std::ptr::null_mut();
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
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    unsafe {
        // Create a MetafitsContext
        let mut metafits_context_ptr: *mut MetafitsContext = std::ptr::null_mut();
        let retval = mwalib_metafits_context_new(
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
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/missing_file.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    unsafe {
        // Create a MetafitsContext
        let mut metafits_context_ptr: *mut MetafitsContext = std::ptr::null_mut();
        let retval = mwalib_metafits_context_new(
            metafits_file_ptr,
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
            ret_error_message = str_slice.to_owned();
        }

        // Check error message
        assert!(!ret_error_message.is_empty());
    }
}

#[test]
fn test_mwalib_metafits_context_display() {
    let metafits_context_ptr: *mut MetafitsContext = get_test_ffi_metafits_context();

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let retval =
            mwalib_metafits_context_display(metafits_context_ptr, error_message_ptr, error_len);

        assert_eq!(retval, 0);
    }
}

#[test]
fn test_mwalib_metafits_context_display_null_ptr() {
    let metafits_context_ptr: *mut MetafitsContext = std::ptr::null_mut();

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

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
    let error_message_ptr = error_message.as_ptr() as *const c_char;

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
    let error_message_ptr = error_message.as_ptr() as *const c_char;

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
            ret_error_message = str_slice.to_owned();
        }

        // Check error message
        assert!(!ret_error_message.is_empty());
    }
}

#[test]
fn test_mwalib_correlator_context_display() {
    let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context();

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

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
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let retval =
            mwalib_correlator_context_display(correlator_context_ptr, error_message_ptr, error_len);

        assert_ne!(retval, 0);
    }
}

#[test]
fn test_mwalib_correlator_context_legacy_read_by_baseline_valid() {
    let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

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
        assert_eq!(
            approx_eq!(f32, ret_buffer[0], 73189.0, F32Margin::default()),
            true,
            "Expected value was {}, should be {}",
            ret_buffer[0],
            73189.0
        );
        assert_eq!(
            approx_eq!(f32, ret_buffer[100], -1482.5, F32Margin::default()),
            true,
            "Expected value was {}, should be {}",
            ret_buffer[100],
            -1482.5
        );
        assert_eq!(
            approx_eq!(f32, ret_buffer[1016], 74300.5, F32Margin::default()),
            true,
            "Expected value was {}, should be {}",
            ret_buffer[1016],
            74300.5
        );
        assert_eq!(
            approx_eq!(f32, ret_buffer[8385552], -174.5, F32Margin::default()),
            true,
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
    let error_message_ptr = error_message.as_ptr() as *const c_char;

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
    let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

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
    let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

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
        assert_eq!(
            approx_eq!(f32, ret_buffer[0], 73189.0, F32Margin::default()),
            true,
            "Expected value was {}, should be {}",
            ret_buffer[0],
            73189.0
        );
        assert_eq!(
            approx_eq!(f32, ret_buffer[100], 112.0, F32Margin::default()),
            true,
            "Expected value was {}, should be {}",
            ret_buffer[100],
            -1844.5
        );
        assert_eq!(
            approx_eq!(f32, ret_buffer[1016], 205.5, F32Margin::default()),
            true,
            "Expected value was {}, should be {}",
            ret_buffer[1016],
            205.5
        );
        assert_eq!(
            approx_eq!(f32, ret_buffer[8385552], -178.0, F32Margin::default()),
            true,
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
    let error_message_ptr = error_message.as_ptr() as *const c_char;

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
    let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context();

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

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

        // Should get non zero return code
        assert_ne!(retval, 0);
    }
}

//
// VoltageContext Tests
//
#[test]
fn test_mwalib_voltage_context_new_valid_mwaxv2() {
    // This tests for a valid voltage context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    // Setup files
    let created_voltage_files =
        voltage_context::test::get_test_voltage_files(CorrelatorVersion::V2);
    let voltage_file = CString::new(created_voltage_files[0].clone()).unwrap();

    let voltage_files: Vec<*const c_char> = vec![voltage_file.as_ptr()];

    let voltage_files_ptr = voltage_files.as_ptr() as *mut *const c_char;

    unsafe {
        // Create a VoltageContext
        let mut voltage_context_ptr: *mut VoltageContext = std::ptr::null_mut();
        let retval = mwalib_voltage_context_new(
            metafits_file_ptr,
            voltage_files_ptr,
            1,
            &mut voltage_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_voltage_context_new
        let mut ret_error_message: String = String::new();

        if retval != 0 {
            let c_str: &CStr = CStr::from_ptr(error_message_ptr);
            let str_slice: &str = c_str.to_str().unwrap();
            ret_error_message = str_slice.to_owned();
        }
        assert_eq!(
            retval, 0,
            "mwalib_voltage_context_new failure {}",
            ret_error_message
        );

        // Check we got valid VoltageContext pointer
        let context_ptr = voltage_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Now ensure we can free the rust memory
        assert_eq!(mwalib_voltage_context_free(context_ptr.unwrap()), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_voltage_context_free(std::ptr::null_mut()), 0);
    }
}

#[test]
fn test_mwalib_voltage_context_new_invalid() {
    // This tests for a invalid voltage context (missing file)
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/invalid_filename.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    // Setup files
    let created_voltage_files =
        voltage_context::test::get_test_voltage_files(CorrelatorVersion::V2);
    let voltage_file = CString::new(created_voltage_files[0].clone()).unwrap();

    let voltage_files: Vec<*const c_char> = vec![voltage_file.as_ptr()];

    let voltage_files_ptr = voltage_files.as_ptr() as *mut *const c_char;

    unsafe {
        // Create a VoltageContext
        let mut voltage_context_ptr: *mut VoltageContext = std::ptr::null_mut();
        let retval = mwalib_voltage_context_new(
            metafits_file_ptr,
            voltage_files_ptr,
            1,
            &mut voltage_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return val
        assert_ne!(retval, 0);

        // Get Error message
        let mut ret_error_message: String = String::new();

        if retval != 0 {
            let c_str: &CStr = CStr::from_ptr(error_message_ptr);
            let str_slice: &str = c_str.to_str().unwrap();
            ret_error_message = str_slice.to_owned();
        }

        // Check error message
        assert!(!ret_error_message.is_empty());
    }
}

#[test]
fn test_mwalib_voltage_context_display() {
    let voltage_context_ptr: *mut VoltageContext =
        get_test_ffi_voltage_context(CorrelatorVersion::Legacy);

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let retval =
            mwalib_voltage_context_display(voltage_context_ptr, error_message_ptr, error_len);

        assert_eq!(retval, 0);
    }
}

#[test]
fn test_mwalib_voltage_context_display_null_ptr() {
    let voltage_context_ptr: *mut VoltageContext = std::ptr::null_mut();

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let retval =
            mwalib_voltage_context_display(voltage_context_ptr, error_message_ptr, error_len);

        assert_ne!(retval, 0);
    }
}

#[test]
fn test_mwalib_voltage_context_legacy_read_file_valid() {
    let voltage_context_ptr: *mut VoltageContext =
        get_test_ffi_voltage_context(CorrelatorVersion::Legacy);

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    let timestep_index = 0;
    let coarse_chan_index = 0;

    // 2 pols x 128 fine chans x 1 tile * 10000 samples
    let buffer_len = 2 * 128 * 10000;

    unsafe {
        let in_buffer: Vec<u8> = vec![0; buffer_len];
        let buffer_ptr: *mut u8 = ffi_array_to_boxed_slice(in_buffer);

        let retval = mwalib_voltage_context_read_file(
            voltage_context_ptr,
            timestep_index,
            coarse_chan_index,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        assert_eq!(retval, 0);

        // Reconstitute the buffer
        let buffer: Vec<u8> = ffi_boxed_slice_to_array(buffer_ptr, buffer_len);

        // Check contents
        // Check for various values
        // sample: 0, fine_chan: 0, rfinput: 0
        assert_eq!(
            buffer[voltage_context::test::get_index_for_location_in_test_voltage_file_legacy(
                0, 0, 0
            )],
            0
        );

        // sample: 0, fine_chan: 0, rfinput: 1
        assert_eq!(
            buffer[voltage_context::test::get_index_for_location_in_test_voltage_file_legacy(
                0, 0, 1
            )],
            2
        );

        // sample: 0, fine_chan: 1, rfinput: 1
        assert_eq!(
            buffer[voltage_context::test::get_index_for_location_in_test_voltage_file_legacy(
                0, 1, 1
            )],
            5
        );

        // sample: 0, fine_chan: 127, rfinput: 0
        assert_eq!(
            buffer[voltage_context::test::get_index_for_location_in_test_voltage_file_legacy(
                0, 127, 0
            )],
            125
        );

        // sample: 10, fine_chan: 32, rfinput: 1
        assert_eq!(
            buffer[voltage_context::test::get_index_for_location_in_test_voltage_file_legacy(
                10, 32, 1
            )],
            138
        );

        // sample: 9999, fine_chan: 127, rfinput: 1
        assert_eq!(
            buffer[voltage_context::test::get_index_for_location_in_test_voltage_file_legacy(
                9999, 127, 1
            )],
            187
        );
    }
}

#[test]
fn test_mwalib_voltage_context_mwaxv2_read_file_valid() {
    let voltage_context_ptr: *mut VoltageContext =
        get_test_ffi_voltage_context(CorrelatorVersion::V2);

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    let timestep_index = 0;
    let coarse_chan_index = 0;

    // 2 pols x 1 fine chans x 1 tile * 64000 samples * 160 blocks * 2 bytes per sample
    let buffer_len = 2 * 64000 * 160 * 2;

    unsafe {
        let in_buffer: Vec<u8> = vec![0; buffer_len];
        let buffer_ptr: *mut u8 = ffi_array_to_boxed_slice(in_buffer);

        let retval = mwalib_voltage_context_read_file(
            voltage_context_ptr,
            timestep_index,
            coarse_chan_index,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        assert_eq!(retval, 0);

        // Reconstitute the buffer
        let buffer: Vec<u8> = ffi_boxed_slice_to_array(buffer_ptr, buffer_len);

        // Check for various values
        // block: 0, rfinput: 0, sample: 0, value: 0
        assert_eq!(
            buffer[voltage_context::test::get_index_for_location_in_test_voltage_file_mwaxv2(
                0, 0, 0, 0
            )],
            0
        );

        // block: 0, rfinput: 0, sample: 1, value: 1
        assert_eq!(
            buffer[voltage_context::test::get_index_for_location_in_test_voltage_file_mwaxv2(
                0, 0, 1, 1
            )],
            253
        );

        // block: 0, rfinput: 0, sample: 255, value: 0
        assert_eq!(
            buffer[voltage_context::test::get_index_for_location_in_test_voltage_file_mwaxv2(
                0, 0, 255, 0
            )],
            254
        );

        // block: 0, rfinput: 0, sample: 256, value: 1
        assert_eq!(
            buffer[voltage_context::test::get_index_for_location_in_test_voltage_file_mwaxv2(
                0, 0, 256, 1
            )],
            255
        );

        // block: 1, rfinput: 0, sample: 2, value: 0
        assert_eq!(
            buffer[voltage_context::test::get_index_for_location_in_test_voltage_file_mwaxv2(
                1, 0, 2, 0
            )],
            9
        );

        // block: 159, rfinput: 1, sample: 63999, value: 1
        assert_eq!(
            buffer[voltage_context::test::get_index_for_location_in_test_voltage_file_mwaxv2(
                159, 1, 63999, 1
            )],
            226
        );

        // block: 120, rfinput: 0, sample: 0, value: 0
        assert_eq!(
            buffer[voltage_context::test::get_index_for_location_in_test_voltage_file_mwaxv2(
                120, 0, 0, 0
            )],
            88
        );
    }
}

//
// Metafits Metadata Tests
//
#[test]
fn test_mwalib_metafits_metadata_get_from_metafits_context_valid() {
    // This tests for a valid metafits context and metadata returned
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;
    // Create a MetafitsContext
    let metafits_context_ptr: *mut MetafitsContext = get_test_ffi_metafits_context();
    unsafe {
        // Check we got valid MetafitsContext pointer
        let context_ptr = metafits_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: &mut *mut MetafitsMetadata = &mut std::ptr::null_mut();
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
            ret_error_message = str_slice.to_owned();
        }
        assert_eq!(
            retval, 0,
            "mwalib_metafits_metadata_get failure {}",
            ret_error_message
        );

        // Get the mwalibMetadata struct from the pointer
        let metafits_metadata = Box::from_raw(*metafits_metadata_ptr);

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
fn test_mwalib_metafits_metadata_get_null_contexts() {
    // This tests for a null context passed to mwalib_metafits_metadata_get
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut metafits_metadata_ptr: &mut *mut MetafitsMetadata = &mut std::ptr::null_mut();
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
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        // Create a CorrelatorContext
        let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context();

        // Check we got valid MetafitsContext pointer
        let context_ptr = correlator_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: &mut *mut MetafitsMetadata = &mut std::ptr::null_mut();
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
        let metafits_metadata = Box::from_raw(*metafits_metadata_ptr);

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
fn test_mwalib_metafits_metadata_get_from_voltage_context_valid() {
    // This tests for a valid metafits metadata returned given a voltage context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        // Create a VoltageContext
        let voltage_context_ptr: *mut VoltageContext =
            get_test_ffi_voltage_context(CorrelatorVersion::Legacy);

        // Check we got valid MetafitsContext pointer
        let context_ptr = voltage_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: &mut *mut MetafitsMetadata = &mut std::ptr::null_mut();
        let retval = mwalib_metafits_metadata_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            voltage_context_ptr,
            &mut metafits_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        assert_eq!(
            retval, 0,
            "mwalib_metafits_metadata_get did not return success"
        );

        // Get the metafits metadata struct from the pointer
        let metafits_metadata = Box::from_raw(*metafits_metadata_ptr);

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
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        // Create a CorrelatorContext
        let correlator_context_ptr: *mut CorrelatorContext = get_test_ffi_correlator_context();

        // Check we got valid MetafitsContext pointer
        let context_ptr = correlator_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a CorrelatorMetadata struct
        let mut correlator_metadata_ptr: &mut *mut CorrelatorMetadata = &mut std::ptr::null_mut();
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
        let correlator_metadata = Box::from_raw(*correlator_metadata_ptr);

        // We should get a valid number of coarse channels and no error message
        assert_eq!(correlator_metadata.num_coarse_chans, 1);

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
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut correlator_metadata_ptr: &mut *mut CorrelatorMetadata = &mut std::ptr::null_mut();

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
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        // Create a VoltageContext
        let voltage_context_ptr: *mut VoltageContext =
            get_test_ffi_voltage_context(CorrelatorVersion::Legacy);

        // Check we got valid MetafitsContext pointer
        let context_ptr = voltage_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a VoltageMetadata struct
        let mut voltage_metadata_ptr: &mut *mut VoltageMetadata = &mut std::ptr::null_mut();
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
        let voltage_metadata = Box::from_raw(*voltage_metadata_ptr);

        // We should get a valid number of coarse channels and no error message
        assert_eq!(voltage_metadata.num_coarse_chans, 2);

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
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut voltage_metadata_ptr: &mut *mut VoltageMetadata = &mut std::ptr::null_mut();

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
fn test_mwalib_antennas_get_from_metafits_context_valid() {
    // This test populates antennas given a metafits context
    let index = 2; // valid  should be Tile013

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_metafits_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut Antenna = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_antennas_get(
            context,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_antennas_get did not return success");

        // reconstitute into a vector
        let item: Vec<Antenna> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 128, "Array length is not correct");
        assert_eq!(item[index].tile_id, 13);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let array_to_free = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_antennas_free(array_to_free, array_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_antennas_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_antennas_get_from_correlator_context_valid() {
    // This test populates antennas given a correlator context
    let index = 2; // valid  should be Tile013
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_correlator_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut Antenna = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_antennas_get(
            std::ptr::null_mut(),
            context,
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_antennas_get did not return success");

        // reconstitute into a vector
        let item: Vec<Antenna> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 128, "Array length is not correct");
        assert_eq!(item[index].tile_id, 13);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let array_to_free = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_antennas_free(array_to_free, array_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_antennas_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_antennas_get_from_voltage_context_valid_mwax() {
    // This test populates antennas given a voltage context
    let index = 2; // valid  should be Tile013
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_voltage_context(CorrelatorVersion::V2);

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut Antenna = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_antennas_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            context,
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_antennas_get did not return success");

        // reconstitute into a vector
        let item: Vec<Antenna> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 128, "Array length is not correct");
        assert_eq!(item[index].tile_id, 13);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let array_to_free = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_antennas_free(array_to_free, array_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_antennas_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_antennas_get_null_contexts() {
    // This tests for a null context passed into the _get method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut array_ptr: &mut *mut Antenna = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;
        let retval = mwalib_antennas_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer, non-zero retval and an error message
        assert_ne!(retval, 0);
        assert!(array_ptr.is_null());
        let expected_error: &str = &"mwalib_antennas_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

// Baselines
#[test]
fn test_mwalib_baselines_get_valid_using_metafits_context() {
    // This test populates baselines given a metafits context
    let index = 2; // valid  should be baseline (0,2)

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_metafits_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut Baseline = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_baselines_get(
            context,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_baselines_get did not return success");

        // reconstitute into a vector
        let item: Vec<Baseline> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 8256, "Array length is not correct");
        assert_eq!(item[index].ant1_index, 0);
        assert_eq!(item[index].ant2_index, 2);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let array_to_free = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_baselines_free(array_to_free, array_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_baselines_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_baselines_get_valid_using_correlator_context() {
    // This test populates baselines given a correlator context
    let index = 2; // valid  should be baseline (0,2)

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_correlator_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut Baseline = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_baselines_get(
            std::ptr::null_mut(),
            context,
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_baselines_get did not return success");

        // reconstitute into a vector
        let item: Vec<Baseline> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 8256, "Array length is not correct");
        assert_eq!(item[index].ant1_index, 0);
        assert_eq!(item[index].ant2_index, 2);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let array_to_free = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_baselines_free(array_to_free, array_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_baselines_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_baselines_get_valid_using_voltage_context() {
    // This test populates baselines given a voltage context
    let index = 2; // valid  should be baseline (0,2)

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_voltage_context(CorrelatorVersion::Legacy);

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut Baseline = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_baselines_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            context,
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_baselines_get did not return success");

        // reconstitute into a vector
        let item: Vec<Baseline> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 8256, "Array length is not correct");
        assert_eq!(item[index].ant1_index, 0);
        assert_eq!(item[index].ant2_index, 2);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let array_to_free = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_baselines_free(array_to_free, array_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_baselines_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_baselines_get_null_context() {
    // This tests for a null context passed into the _get method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut array_ptr: &mut *mut Baseline = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;
        let retval = mwalib_baselines_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer, non-zero retval and an error message
        assert_ne!(retval, 0);
        assert!(array_ptr.is_null());
        let expected_error: &str = &"mwalib_baselines_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

// Coarse Channels
#[test]
fn test_mwalib_correlator_coarse_channels_get_valid() {
    // This test populates coarse_chans given a correlator context
    let index = 0; // valid  should be receiver channel 109

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_correlator_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut CoarseChannel = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_correlator_coarse_channels_get(
            context,
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(
            retval, 0,
            "mwalib_correlator_coarse_channels_get did not return success"
        );

        // reconstitute into a vector
        let item: Vec<CoarseChannel> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 1, "Array length is not correct");
        assert_eq!(item[index].rec_chan_number, 109);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let array_to_free = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_coarse_channels_free(array_to_free, array_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_coarse_channels_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_correlator_coarse_channels_get_null_context() {
    // This tests for a null context passed into the _get method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut array_ptr: &mut *mut CoarseChannel = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;
        let retval = mwalib_correlator_coarse_channels_get(
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer, non-zero retval and an error message
        assert_ne!(retval, 0);
        assert!(array_ptr.is_null());
        let expected_error: &str = &"mwalib_correlator_coarse_channels_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

#[test]
fn test_mwalib_voltage_coarse_channels_get_valid() {
    // This test populates coarse_chans given a voltage context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_voltage_context(CorrelatorVersion::Legacy);

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut CoarseChannel = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_voltage_coarse_channels_get(
            context,
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(
            retval, 0,
            "mwalib_voltage_coarse_channels_get did not return success"
        );

        // reconstitute into a vector
        let item: Vec<CoarseChannel> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(
            array_len, 2,
            "Coarse channel array length is not correct- should be 2"
        );
        assert_eq!(item[0].rec_chan_number, 123);
        assert_eq!(item[1].rec_chan_number, 124);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let array_to_free = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_coarse_channels_free(array_to_free, array_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_coarse_channels_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_voltage_coarse_channels_get_null_context() {
    // This tests for a null context passed into the _get method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut array_ptr: &mut *mut CoarseChannel = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;
        let retval = mwalib_voltage_coarse_channels_get(
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer, non-zero retval and an error message
        assert_ne!(retval, 0);
        assert!(array_ptr.is_null());
        let expected_error: &str = &"mwalib_voltage_coarse_channels_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

// RF Input
#[test]
fn test_mwalib_rfinputs_get_from_metafits_context_valid() {
    // This test populates rfinputs given a metafits context
    let index = 2; // valid  should be Tile012(X)

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_metafits_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut Rfinput = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_rfinputs_get(
            context,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_rfinputs_get did not return success");

        // reconstitute into a vector
        let item: Vec<Rfinput> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 256, "Array length is not correct");

        assert_eq!(item[index].ant, 1);

        assert_eq!(
            CString::from_raw(item[index].tile_name),
            CString::new("Tile012").unwrap()
        );

        assert_eq!(
            CString::from_raw(item[index].pol),
            CString::new("X").unwrap()
        );
    }
}
#[test]
fn test_mwalib_rfinputs_free() {
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_metafits_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut Rfinput = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_rfinputs_get(
            context,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_rfinputs_get did not return success");

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_rfinputs_free(*array_ptr, array_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_rfinputs_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_rfinputs_get_from_correlator_context_valid() {
    // This test populates rfinputs given a correlator context
    let index = 2; // valid  should be Tile012(X)

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_correlator_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut Rfinput = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_rfinputs_get(
            std::ptr::null_mut(),
            context,
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_rfinputs_get did not return success");

        // reconstitute into a vector
        let item: Vec<Rfinput> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 256, "Array length is not correct");

        assert_eq!(item[index].ant, 1);

        assert_eq!(
            CString::from_raw(item[index].tile_name),
            CString::new("Tile012").unwrap()
        );

        assert_eq!(
            CString::from_raw(item[index].pol),
            CString::new("X").unwrap()
        );
    }
}

#[test]
fn test_mwalib_rfinputs_get_from_voltage_context_valid() {
    // This test populates rfinputs given a voltage context
    let index = 2; // valid  should be Tile012(X)

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_voltage_context(CorrelatorVersion::V2);

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut Rfinput = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_rfinputs_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            context,
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_rfinputs_get did not return success");

        // reconstitute into a vector
        let item: Vec<Rfinput> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 256, "Array length is not correct");

        assert_eq!(item[index].ant, 1);

        assert_eq!(
            CString::from_raw(item[index].tile_name),
            CString::new("Tile012").unwrap()
        );

        assert_eq!(
            CString::from_raw(item[index].pol),
            CString::new("X").unwrap()
        );
    }
}

#[test]
fn test_mwalib_rfinputs_get_null_contexts() {
    // This tests for a null context passed into the _get method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut array_ptr: &mut *mut Rfinput = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;
        let retval = mwalib_rfinputs_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer, non-zero retval and an error message
        assert_ne!(retval, 0);
        assert!(array_ptr.is_null());
        let expected_error: &str = &"mwalib_rfinputs_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

// Timesteps
#[test]
fn test_mwalib_correlator_timesteps_get_valid() {
    // This test populates timesteps given a correlator context
    let index = 0; // valid  should be timestep at unix_time 1417468096.0;

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_correlator_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut TimeStep = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_correlator_timesteps_get(
            context,
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_timesteps_get did not return success");

        // reconstitute into a vector
        let item: Vec<TimeStep> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 1, "Array length is not correct");
        assert_eq!(item[index].unix_time_ms, 1_417_468_096_000);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let array_to_free = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_timesteps_free(array_to_free, array_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_timesteps_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_correlator_timesteps_get_null_context() {
    // This tests for a null context passed into the _get method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut array_ptr: &mut *mut TimeStep = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;
        let retval = mwalib_correlator_timesteps_get(
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer, non-zero retval and an error message
        assert_ne!(retval, 0);
        assert!(array_ptr.is_null());
        let expected_error: &str = &"mwalib_correlator_timesteps_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

// Visibility Pols
#[test]
fn test_mwalib_visibility_pols_get_valid_using_metafits_context() {
    // This test populates visibility_pols given a metafits context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_metafits_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut VisibilityPol = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_visibility_pols_get(
            context,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(
            retval, 0,
            "mwalib_correlator_visibility_pols_get did not return success"
        );

        // reconstitute into a vector
        let item: Vec<VisibilityPol> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 4, "Array length is not correct");
        assert_eq!(
            CString::from_raw(item[0].polarisation),
            CString::new("XX").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[1].polarisation),
            CString::new("XY").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[2].polarisation),
            CString::new("YX").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[3].polarisation),
            CString::new("YY").unwrap()
        );
    }
}
#[test]
fn test_mwalib_visibility_pols_get_valid_using_correlator_context() {
    // This test populates visibility_pols given a correlator context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_correlator_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut VisibilityPol = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_visibility_pols_get(
            std::ptr::null_mut(),
            context,
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(
            retval, 0,
            "mwalib_correlator_visibility_pols_get did not return success"
        );

        // reconstitute into a vector
        let item: Vec<VisibilityPol> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 4, "Array length is not correct");
        assert_eq!(
            CString::from_raw(item[0].polarisation),
            CString::new("XX").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[1].polarisation),
            CString::new("XY").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[2].polarisation),
            CString::new("YX").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[3].polarisation),
            CString::new("YY").unwrap()
        );
    }
}

#[test]
fn test_mwalib_visibility_pols_get_valid_using_voltage_context() {
    // This test populates visibility_pols given a voltage context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_voltage_context(CorrelatorVersion::Legacy);

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut VisibilityPol = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_visibility_pols_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            context,
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(
            retval, 0,
            "mwalib_correlator_visibility_pols_get did not return success"
        );

        // reconstitute into a vector
        let item: Vec<VisibilityPol> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 4, "Array length is not correct");
        assert_eq!(
            CString::from_raw(item[0].polarisation),
            CString::new("XX").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[1].polarisation),
            CString::new("XY").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[2].polarisation),
            CString::new("YX").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[3].polarisation),
            CString::new("YY").unwrap()
        );
    }
}

#[test]
fn test_mwalib_visibility_pols_free() {
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_ffi_metafits_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut VisibilityPol = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_visibility_pols_get(
            context,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(
            retval, 0,
            "mwalib_correlator_visibility_pols_get did not return success"
        );

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_visibility_pols_free(*array_ptr, array_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_visibility_pols_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_visibilitypols_get_null_context() {
    // This tests for a null context passed into the _get method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut array_ptr: &mut *mut VisibilityPol = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;
        let retval = mwalib_visibility_pols_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer, non-zero retval and an error message
        assert_ne!(retval, 0);
        assert!(array_ptr.is_null());
        let expected_error: &str = &"mwalib_visibility_pols_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}
