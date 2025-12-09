// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::*;
use crate::{
    ffi::ffi_test_helpers::{ffi_boxed_slice_to_array, get_test_ffi_correlator_context_legacy, get_test_ffi_metafits_context}, metafits_context::ffi::{MetafitsMetadata, mwalib_metafits_context_display, mwalib_metafits_context_free, mwalib_metafits_context_new, mwalib_metafits_context_new2, mwalib_metafits_metadata_free, mwalib_metafits_metadata_get}};
use std::ffi::{c_char, CStr, CString};
use float_cmp::{F64Margin, approx_eq};
use libc::size_t;

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
        let item: Vec<antenna::ffi::Antenna> =
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
        let items: Vec<antenna::ffi::Antenna> =
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