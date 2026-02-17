// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::*;
use crate::{
    antenna::ffi::Antenna,
    ffi::{
        ffi_create_c_array,
        ffi_test_helpers::{
            ffi_boxed_slice_to_array, get_test_ffi_metafits_context, get_test_ffi_voltage_context,
        },
    },
    metafits_context::ffi::{
        mwalib_metafits_get_expected_volt_filename, mwalib_metafits_metadata_get, MetafitsMetadata,
    },
    voltage_context::{
        ffi::{
            mwalib_voltage_context_display, mwalib_voltage_context_free,
            mwalib_voltage_context_get_fine_chan_freqs_hz_array, mwalib_voltage_context_new,
            mwalib_voltage_context_read_file, mwalib_voltage_context_read_second,
            mwalib_voltage_metadata_free, mwalib_voltage_metadata_get, VoltageMetadata,
        },
        test::{
            get_index_for_location_in_test_voltage_file_legacy,
            get_index_for_location_in_test_voltage_file_mwaxv2,
            get_index_for_location_in_test_voltage_file_mwaxv2_os, get_test_voltage_files,
        },
    },
};
use float_cmp::{approx_eq, F64Margin};
use libc::size_t;
use std::ffi::{c_char, CStr, CString};

//
// VoltageContext Tests
//
#[test]
fn test_mwalib_voltage_context_new_valid_mwaxv2() {
    // This tests for a valid voltage context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let metafits_file = CString::new("test_files/1101503312_mwax_vcs/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    // Setup files
    let created_voltage_files = get_test_voltage_files(MWAVersion::VCSMWAXv2, false);
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
            str_slice.clone_into(&mut ret_error_message);
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
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let metafits_file =
        CString::new("test_files/1101503312_mwax_vcs/invalid_filename.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    // Setup files
    let created_voltage_files = get_test_voltage_files(MWAVersion::VCSMWAXv2, false);
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
            str_slice.clone_into(&mut ret_error_message);
        }

        // Check error message
        assert!(!ret_error_message.is_empty());
    }
}

#[test]
fn test_mwalib_metafits_metadata_get_from_voltage_context_valid() {
    // This tests for a valid metafits metadata returned given a voltage context
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

        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: *mut MetafitsMetadata = std::ptr::null_mut();
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
        let metafits_metadata = Box::from_raw(metafits_metadata_ptr);

        // We should get a valid obsid and no error message
        assert_eq!(metafits_metadata.obs_id, 1_101_503_312);

        //
        // Test antennas
        //
        let items: Vec<Antenna> =
            ffi_boxed_slice_to_array(metafits_metadata.antennas, metafits_metadata.num_ants);

        assert_eq!(items.len(), 1, "Array length is not correct");

        assert_eq!(items[0].tile_id, 11);
        assert_eq!(items[0].rfinput_y, 0);
        assert_eq!(items[0].rfinput_x, 1);

        // Note- don't try to do any free's here since, in order to test, we have had to reconstituded some of the arrays which will result in a double free
    }
}

#[test]
fn test_mwalib_metafits_get_expected_volt_filename() {
    // Create a MetafitsContext
    let mwa_version = MWAVersion::VCSLegacyRecombined;
    let metafits_context_ptr: *mut MetafitsContext = get_test_ffi_metafits_context(mwa_version);

    let error_message_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let filename_len: size_t = 32; // 31 + null terminator
    let filename = CString::new(" ".repeat(filename_len)).unwrap();
    let filename_ptr = filename.as_ptr() as *mut c_char;

    unsafe {
        let retval = mwalib_metafits_get_expected_volt_filename(
            metafits_context_ptr,
            3,
            1,
            filename_ptr,
            filename_len,
            error_message_ptr,
            error_message_len,
        );

        // Should be success
        assert_eq!(retval, 0);

        // Check the filename (NOTE it already has a nul terminator)
        assert_eq!(
            filename.as_bytes(),
            CString::new("1101503312_1101503315_ch110.dat")
                .unwrap()
                .as_bytes_with_nul()
        );
    }
}

#[test]
fn test_mwalib_voltage_context_display() {
    let voltage_context_ptr: *mut VoltageContext =
        get_test_ffi_voltage_context(MWAVersion::VCSLegacyRecombined, false);

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;
    let buf_len: size_t = 1280;
    let buf_message = CString::new(" ".repeat(buf_len)).unwrap();
    let buf_message_ptr = buf_message.as_ptr() as *mut c_char;

    unsafe {
        let retval = mwalib_voltage_context_display(
            voltage_context_ptr,
            buf_message_ptr,
            buf_len,
            error_message_ptr,
            error_len,
        );

        assert_eq!(retval, 0);

        // Check that the first few chars are "VoltageContext ("
        let output_str = CStr::from_ptr(buf_message_ptr)
            .to_str()
            .expect("Error converting C string");
        assert!(output_str.starts_with("VoltageContext ("));
    }
}

#[test]
fn test_mwalib_voltage_context_display_null_ptr() {
    let voltage_context_ptr: *mut VoltageContext = std::ptr::null_mut();

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;
    let buf_len: size_t = 1280;
    let buf_message = CString::new(" ".repeat(buf_len)).unwrap();
    let buf_message_ptr = buf_message.as_ptr() as *mut c_char;

    unsafe {
        let retval = mwalib_voltage_context_display(
            voltage_context_ptr,
            buf_message_ptr,
            buf_len,
            error_message_ptr,
            error_len,
        );

        assert_ne!(retval, 0);
    }
}

#[test]
fn test_mwalib_voltage_context_legacy_read_file_valid() {
    let voltage_context_ptr: *mut VoltageContext =
        get_test_ffi_voltage_context(MWAVersion::VCSLegacyRecombined, false);

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let timestep_index = 0;
    let coarse_chan_index = 14;

    // 2 pols x 128 fine chans x 1 tile * 10000 samples
    let buffer_len = 2 * 128 * 10000;

    unsafe {
        let in_buffer: Vec<i8> = vec![0; buffer_len];
        let (buffer_ptr, _) = ffi_create_c_array(in_buffer);

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
        let buffer: Vec<i8> = ffi_boxed_slice_to_array(buffer_ptr, buffer_len);

        // Check contents
        // Check for various values
        // sample: 0, fine_chan: 0, rfinput: 0
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_legacy(0, 0, 0)],
            0
        );

        // sample: 0, fine_chan: 0, rfinput: 1
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_legacy(0, 0, 1)],
            2
        );

        // sample: 0, fine_chan: 1, rfinput: 1
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_legacy(0, 1, 1)],
            5
        );

        // sample: 0, fine_chan: 127, rfinput: 0
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_legacy(0, 127, 0)],
            125
        );

        // sample: 10, fine_chan: 32, rfinput: 1
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_legacy(10, 32, 1)],
            -118
        );

        // sample: 9999, fine_chan: 127, rfinput: 1
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_legacy(9999, 127, 1)],
            -69
        );
    }
}

#[test]
fn test_mwalib_voltage_context_mwaxv2_read_file_valid() {
    // Read data from a critically sampled MWAXv2 VCS obs
    let voltage_context_ptr: *mut VoltageContext =
        get_test_ffi_voltage_context(MWAVersion::VCSMWAXv2, false);

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let timestep_index = 0;
    let coarse_chan_index = 14;

    // 2 pols x 1 fine chans x 1 tile * 64000 samples * 160 blocks * 2 bytes per sample
    let buffer_len = 2 * 64000 * 160 * 2;

    unsafe {
        let in_buffer: Vec<i8> = vec![0; buffer_len];
        let (buffer_ptr, _) = ffi_create_c_array(in_buffer);

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
        let buffer: Vec<i8> = ffi_boxed_slice_to_array(buffer_ptr, buffer_len);

        // Check for various values
        // block: 0, rfinput: 0, sample: 0, value: 0
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 0, 0)],
            0
        );

        // block: 0, rfinput: 0, sample: 1, value: 1
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 1, 1)],
            -3
        );

        // block: 0, rfinput: 0, sample: 255, value: 0
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 255, 0)],
            -2
        );

        // block: 0, rfinput: 0, sample: 256, value: 1
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 256, 1)],
            -1
        );

        // block: 1, rfinput: 0, sample: 2, value: 0
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2(1, 0, 2, 0)],
            9
        );

        // block: 159, rfinput: 1, sample: 63999, value: 1
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2(159, 1, 63999, 1)],
            -30
        );

        // block: 120, rfinput: 0, sample: 0, value: 0
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2(120, 0, 0, 0)],
            88
        );
    }
}

#[test]
fn test_mwalib_voltage_context_mwaxv2_read_os_file_valid() {
    // Read data from a oversampled MWAXv2 VCS obs
    let voltage_context_ptr: *mut VoltageContext =
        get_test_ffi_voltage_context(MWAVersion::VCSMWAXv2, true);

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let timestep_index = 0;
    let coarse_chan_index = 14;

    // 2 pols x 1 fine chans x 1 tile * 81920 samples * 160 blocks * 2 bytes per sample
    let buffer_len = 2 * 81920 * 160 * 2;

    unsafe {
        let in_buffer: Vec<i8> = vec![0; buffer_len];
        let (buffer_ptr, _) = ffi_create_c_array(in_buffer);

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
        let buffer: Vec<i8> = ffi_boxed_slice_to_array(buffer_ptr, buffer_len);

        // Check for various values
        // block: 0, rfinput: 0, sample: 0, value: 0
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 0, 0)],
            0
        );

        // block: 0, rfinput: 0, sample: 1, value: 1
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 1, 1)],
            -3
        );

        // block: 0, rfinput: 0, sample: 255, value: 0
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 255, 0)],
            -2
        );

        // block: 0, rfinput: 0, sample: 256, value: 1
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 256, 1)],
            -1
        );

        // block: 1, rfinput: 0, sample: 2, value: 0
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(1, 0, 2, 0)],
            9
        );

        // block: 159, rfinput: 1, sample: 63999, value: 1
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(159, 1, 63999, 1)],
            -30
        );

        // block: 159, rfinput: 1, sample: 81919, value: 1
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(159, 1, 81919, 1)],
            -30
        );

        // block: 120, rfinput: 0, sample: 0, value: 0
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(120, 0, 0, 0)],
            88
        );
    }
}

#[test]
fn test_mwalib_voltage_context_mwaxv2_read_second_valid() {
    // Read data from a oversampled MWAXv2 VCS obs
    let voltage_context_ptr: *mut VoltageContext =
        get_test_ffi_voltage_context(MWAVersion::VCSMWAXv2, false);

    let error_message_length: size_t = 128;
    let error_message = CString::new(" ".repeat(error_message_length)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *mut c_char;

    let gps_second_start: u64 = 1_101_503_318;
    let gps_second_count: usize = 4;
    let coarse_chan_index: usize = 14;

    // Get voltage metadata
    let mut voltage_metadata_ptr: *mut VoltageMetadata = std::ptr::null_mut();

    unsafe {
        let retval = mwalib_voltage_metadata_get(
            voltage_context_ptr,
            &mut voltage_metadata_ptr,
            error_message_ptr,
            error_message_length,
        );

        assert_eq!(retval, 0);

        assert!(
            !voltage_metadata_ptr.is_null(),
            "mwalib_voltage_metadata_get returned success but voltage_metadata_ptr is null"
        );
    }

    unsafe {
        // 2 pols x 1 fine chans x 1 tile * 81920 samples * 160 blocks * 2 bytes per sample
        let buffer_len = ((*voltage_metadata_ptr).voltage_block_size_bytes
            * (*voltage_metadata_ptr).num_voltage_blocks_per_second as u64
            * gps_second_count as u64) as usize;

        let in_buffer: Vec<i8> = vec![0; buffer_len];
        let (buffer_ptr, _) = ffi_create_c_array(in_buffer);

        let retval = mwalib_voltage_context_read_second(
            voltage_context_ptr,
            gps_second_start,
            gps_second_count,
            coarse_chan_index,
            buffer_ptr,
            buffer_len,
            error_message_ptr,
            error_message_length,
        );

        assert_eq!(retval, 0);

        // Reconstitute the buffer
        let buffer: Vec<i8> = ffi_boxed_slice_to_array(buffer_ptr, buffer_len);

        // Check values
        //
        // Second 1_101_503_318
        //
        // location in buffer: block: 0, rfinput: 0, sample: 0, value: 0
        // location in file0:  block: 120, rfinput: 0, sample: 0, value: 0
        //
        // (this is the 120th block / 7th second of the 8 second block in the FILE, but the first block of the buffer)
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 0, 0)],
            88
        );

        // Second 1_101_503_319
        // location in buffer: block: 20, rfinput: 0, sample: 0, value: 0
        // location in file0:  block: 140, rfinput: 0, sample: 0, value: 0
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2(20, 0, 0, 0)],
            -68
        );

        // Second 1_101_503_320 (now we are in a new data file so the values are incrememented by 2 from the first file)
        // location in buffer: block: 40+0, rfinput: 0, sample: 0, value: 0
        // location in file1:  block: 0, rfinput: 0, sample: 0, value: 0
        assert_eq!(
            buffer[2
                * (*voltage_metadata_ptr).voltage_block_size_bytes as usize
                * (*voltage_metadata_ptr).num_voltage_blocks_per_second
                + get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 0, 0)],
            2
        );

        // Second 1_101_503_321 (now we are in a new data file so the values are incrememented by 2 from the first file)
        // location in buffer: block: 40+20, rfinput: 0, sample: 0, value: 0
        // location in file1:  block: 20, rfinput: 0, sample: 0, value: 0
        assert_eq!(
            buffer[2
                * (*voltage_metadata_ptr).voltage_block_size_bytes as usize
                * (*voltage_metadata_ptr).num_voltage_blocks_per_second
                + get_index_for_location_in_test_voltage_file_mwaxv2(20, 0, 0, 0)],
            102
        );

        // Second 1_101_503_318 (this is the 7th block / 7th second of the 8 second block in the FILE, but the first block of the buffer)
        // location in buffer: block: 0, rfinput: 1, sample: 16750, value: 1
        // location in file0:  block: 120, rfinput: 1, sample: 16750, value: 1
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 1, 16750, 1)],
            -57,
        );

        // Second 1_101_503_319
        // location in buffer: block: 20, rfinput: 1, sample: 16750, value: 1
        // location in file0:  block: 140, rfinput: 1, sample: 16750, value: 1
        assert_eq!(
            buffer[get_index_for_location_in_test_voltage_file_mwaxv2(20, 1, 16750, 1)],
            99
        );

        // Second 1_101_503_320 (now we are in a new data file so the values are incrememented by 2 from the first file)
        // location in buffer: block: 40+0, rfinput: 1, sample: 16750, value: 1
        // location in file0:  block: 0, rfinput: 1, sample: 16750, value: 1
        assert_eq!(
            buffer[2
                * (*voltage_metadata_ptr).voltage_block_size_bytes as usize
                * (*voltage_metadata_ptr).num_voltage_blocks_per_second
                + get_index_for_location_in_test_voltage_file_mwaxv2(0, 1, 16750, 1)],
            29
        );

        // Second 1_101_503_321 (now we are in a new data file so the values are incrememented by 2 from the first file)
        // location in buffer: block: 40+0, rfinput: 1, sample: 16750, value: 1
        // location in file0:  block: 0, rfinput: 1, sample: 16750, value: 1
        assert_eq!(
            buffer[2
                * (*voltage_metadata_ptr).voltage_block_size_bytes as usize
                * (*voltage_metadata_ptr).num_voltage_blocks_per_second
                + get_index_for_location_in_test_voltage_file_mwaxv2(20, 1, 16750, 1)],
            -71
        );
    }
}

#[test]
fn test_mwalib_voltage_context_get_fine_chan_freqs_hz_array_valid() {
    let voltage_context_ptr: *mut VoltageContext =
        get_test_ffi_voltage_context(MWAVersion::VCSLegacyRecombined, false);

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

        let retval = mwalib_voltage_context_get_fine_chan_freqs_hz_array(
            voltage_context_ptr,
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
fn test_mwalib_voltage_context_get_fine_chan_freqs_hz_array_invalid_buffer_len() {
    let voltage_context_ptr: *mut VoltageContext =
        get_test_ffi_voltage_context(MWAVersion::VCSLegacyRecombined, false);

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

        let retval = mwalib_voltage_context_get_fine_chan_freqs_hz_array(
            voltage_context_ptr,
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

        let retval = mwalib_voltage_context_get_fine_chan_freqs_hz_array(
            voltage_context_ptr,
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
fn test_mwalib_voltage_context_get_fine_chan_freqs_hz_array_null_context() {
    let voltage_context_ptr: *mut VoltageContext = std::ptr::null_mut();

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

        let retval = mwalib_voltage_context_get_fine_chan_freqs_hz_array(
            voltage_context_ptr,
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
fn test_mwalib_voltage_context_get_fine_chan_freqs_hz_array_null_coarse_chans() {
    let voltage_context_ptr: *mut VoltageContext =
        get_test_ffi_voltage_context(MWAVersion::VCSLegacyRecombined, false);

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

        let retval = mwalib_voltage_context_get_fine_chan_freqs_hz_array(
            voltage_context_ptr,
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
fn test_mwalib_voltage_context_get_fine_chan_freqs_hz_array_null_buffer() {
    let voltage_context_ptr: *mut VoltageContext =
        get_test_ffi_voltage_context(MWAVersion::VCSLegacyRecombined, false);

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

        let retval = mwalib_voltage_context_get_fine_chan_freqs_hz_array(
            voltage_context_ptr,
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
