// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::{
    ffi::ffi_test_helpers::{get_test_ffi_metafits_context, get_test_ffi_metafits_context_ext},
    metafits_context::ffi::{mwalib_metafits_metadata_get, MetafitsMetadata},
    MWAVersion, MetafitsContext,
};
use libc::size_t;
use std::ffi::{c_char, CStr, CString};

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
