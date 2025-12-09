// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::*;
use crate::{
    MWAVersion, MetafitsContext, ffi::ffi_test_helpers::{ffi_boxed_slice_to_array, get_test_ffi_metafits_context, get_test_ffi_metafits_context_ext}, metafits_context::ffi::{MetafitsMetadata, mwalib_metafits_metadata_get}};
use std::ffi::{c_char, CStr, CString};
use libc::size_t;

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