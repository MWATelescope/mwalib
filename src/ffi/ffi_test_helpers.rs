// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::correlator_context::ffi::mwalib_correlator_context_new;
use crate::metafits_context::ffi::mwalib_metafits_context_new;
use crate::voltage_context::test::get_test_voltage_context;
use crate::{CorrelatorContext, MWAVersion, MetafitsContext, VoltageContext};
use libc::size_t;
use std::ffi::{c_char, CString};

//
// Helper methods for many tests
//
/// Create and return a metafits context based on a test metafits file. Used in many tests in the module.
///
///
/// # Arguments
///
/// * `mwa_version` - Enum telling mwalib the mwa_version it should be using to interpret the metafits file.
///
///
/// # Returns
///
/// * a raw pointer to an instantiated MetafitsContext for the test metafits and gpubox file
///
pub(crate) fn get_test_ffi_metafits_context(mwa_version: MWAVersion) -> *mut MetafitsContext {
    get_test_ffi_metafits_context_ext(
        mwa_version,
        String::from("test_files/1101503312_1_timestep/1101503312.metafits"),
    )
}

/// Create and return a metafits context based on a test metafits file. Used in many tests in the module.
///
///
/// # Arguments
///
/// * `mwa_version` - Enum telling mwalib the mwa_version it should be using to interpret the metafits file.
/// * `metafits_filename` - Filename of metafits to use.
///
/// # Returns
///
/// * a raw pointer to an instantiated MetafitsContext for the test metafits and gpubox file
///
pub(crate) fn get_test_ffi_metafits_context_ext(
    mwa_version: MWAVersion,
    metafits_filename: String,
) -> *mut MetafitsContext {
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let metafits_file = CString::new(metafits_filename).unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    unsafe {
        // Create a MetafitsContext
        let mut metafits_context_ptr: *mut MetafitsContext = std::ptr::null_mut();
        let retval = mwalib_metafits_context_new(
            metafits_file_ptr,
            mwa_version,
            &mut metafits_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_metafits_context_new
        assert_eq!(retval, 0, "mwalib_metafits_context_new_ext failure");

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
pub(crate) fn get_test_ffi_correlator_context_legacy() -> *mut CorrelatorContext {
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

        // Check we got valid MetafitsContext pointer
        let context_ptr = correlator_context_ptr.as_mut();
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
pub(crate) fn get_test_ffi_correlator_context_mwax() -> *mut CorrelatorContext {
    // This tests for a valid correlator context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let metafits_file =
        CString::new("test_files/1244973688_1_timestep/1244973688.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file =
        CString::new("test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits")
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
/// * 'mwa_version' - Enum of the type of MWA data
/// * 'oversampled' - bool - is this an oversampled observation (oversampled only applies to MWAXVCSv2)
///
///
/// # Returns
///
/// * a raw pointer to an instantiated VoltageContext for the test metafits and voltage file
///
pub(crate) fn get_test_ffi_voltage_context(
    mwa_version: MWAVersion,
    oversampled: bool,
) -> *mut VoltageContext {
    // This returns a a valid voltage context
    let context = get_test_voltage_context(mwa_version, oversampled);

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
pub(crate) fn ffi_boxed_slice_to_array<T>(ptr: *mut T, len: usize) -> Vec<T> {
    unsafe {
        let vec: Vec<T> = Vec::from_raw_parts(ptr, len, len);
        vec
    }
}
