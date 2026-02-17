// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    calibration_fit::{self, ffi},
    ffi::{ffi_create_c_array, ffi_free_c_array},
    MetafitsContext,
};

///
/// C Representation of a `CalibrationFit` struct
///
///
/// Calibration Fit table
///
#[repr(C)]
pub struct CalibrationFit {
    /// RF input index
    pub rf_input: usize,
    /// The calibration offset, in metres, for that input,
    /// derived from the most recently processed calibrator
    /// observation with the same coarse channels.
    /// May be missing or all zeros in some metafits files.
    /// Used to generate the slope (versus frequency) for the phase correction.
    pub delay_metres: f32,
    /// /// Used, with the phase slope above to generate the phase correction for each fine channel, for this tile.
    pub intercept_metres: f32,
    /// /// The calibration gain multiplier (not in dB) for each of the N channels (normally 24) in this observation,
    /// for this input. Derived from the most recently processed calibrator observation with the same coarse
    /// channels. May be missing or all ones in some metafits files.
    pub gains: *mut f32,
    /// number of gains elements
    pub num_gains: usize,
    /// polynomial fit parameter for a more accurate gain correction solution for each of the N channels (normally 24) in this observation
    pub gain_polynomial_fit0: *mut f32,
    /// number of gain_polynomial_fit0 elements
    pub num_gain_polynomial_fit0: usize,
    /// polynomial fit parameter for a more accurate gain correction solution for each of the N channels (normally 24) in this observation
    pub gain_polynomial_fit1: *mut f32,
    /// number of gain_polynomial_fit1 elements
    pub num_gain_polynomial_fit1: usize,
    /// dimensionless quality parameter (0-1.0) for the phase fit
    pub phase_fit_quality: f32,
    /// dimensionless quality parameter (0-1.0) for the gain fit
    pub gain_fit_quality: f32,
}

impl CalibrationFit {
    /// This function populates a C array (owned by Rust) of this class
    ///
    /// # Arguments
    ///
    /// * `metafits_context` - Reference to the populated Rust MetafitsContext
    ///    
    /// # Returns
    ///
    /// * Noting
    ///
    ///
    /// # Safety
    /// * The corresponding `ffi_destroy` function for this class must be called to free the memory
    ///
    pub fn populate_array(metafits_context: &MetafitsContext) -> (*mut ffi::CalibrationFit, usize) {
        let mut item_vec: Vec<ffi::CalibrationFit> = Vec::new();

        if let Some(v) = &metafits_context.calibration_fits {
            for item in v.iter() {
                let out_item = {
                    let calibration_fit::CalibrationFit {
                        rf_input: _,
                        delay_metres,
                        intercept_metres,
                        gains,
                        num_gains: _,
                        gain_polynomial_fit0,
                        num_gain_polynomial_fit0: _,
                        gain_polynomial_fit1,
                        num_gain_polynomial_fit1: _,
                        phase_fit_quality,
                        gain_fit_quality,
                    } = item;

                    let (gains_ptr, gains_len) = ffi_create_c_array(gains.clone());
                    let (gain_polynomial_fit0_ptr, gain_polynomial_fit0_len) =
                        ffi_create_c_array(gain_polynomial_fit0.clone());
                    let (gain_polynomial_fit1_ptr, gain_polynomial_fit1_len) =
                        ffi_create_c_array(gain_polynomial_fit1.clone());

                    calibration_fit::ffi::CalibrationFit {
                        rf_input: metafits_context
                            .rf_inputs
                            .iter()
                            .position(|x| x.ant == item.rf_input.ant && x.pol == item.rf_input.pol)
                            .unwrap(),
                        delay_metres: *delay_metres,
                        intercept_metres: *intercept_metres,
                        gains: gains_ptr,
                        num_gains: gains_len,
                        gain_polynomial_fit0: gain_polynomial_fit0_ptr,
                        num_gain_polynomial_fit0: gain_polynomial_fit0_len,
                        gain_polynomial_fit1: gain_polynomial_fit1_ptr,
                        num_gain_polynomial_fit1: gain_polynomial_fit1_len,
                        phase_fit_quality: *phase_fit_quality,
                        gain_fit_quality: *gain_fit_quality,
                    }
                };
                item_vec.push(out_item);
            }
        }

        ffi_create_c_array(item_vec)
    }

    /// This function frees an individual instance (owned by Rust) of this class
    ///
    /// # Arguments
    ///
    /// * `item` - the pointer to the instance of this class
    ///    
    /// # Returns
    ///
    /// * Nothing
    ///
    fn destroy_item(item: *mut ffi::CalibrationFit) {
        if item.is_null() {
            return;
        }

        let a = unsafe { &mut *item };

        // Free string if present
        // Now for each item we need to free anything on the heap
        if !a.gains.is_null() {
            ffi_free_c_array(a.gains, a.num_gains);
        }

        if !a.gain_polynomial_fit0.is_null() {
            ffi_free_c_array(a.gain_polynomial_fit0, a.num_gain_polynomial_fit0);
        }

        if !a.gain_polynomial_fit1.is_null() {
            ffi_free_c_array(a.gain_polynomial_fit1, a.num_gain_polynomial_fit1);
        }
    }

    /// This function frees all array items (owned by Rust) of this class
    ///
    /// # Arguments
    ///
    /// * `items_ptr` - the pointer to the array
    ///
    /// * `items_len` - elements in the array
    ///    
    /// # Returns
    ///
    /// * Nothing
    ///
    pub fn destroy_array(items_ptr: *mut ffi::CalibrationFit, items_len: usize) {
        let items_slice = unsafe { std::slice::from_raw_parts_mut(items_ptr, items_len) };
        for item in items_slice {
            Self::destroy_item(item);
        }
        // Now free the array itself by reconstructing the Vec and letting it drop
        ffi_free_c_array(items_ptr, items_len);
    }
}
