// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{read_cell_array_f32, read_cell_value, Rfinput};
use std::fmt;
pub mod error;

use fitsio::hdu::{FitsHdu, HduInfo};
use fitsio::FitsFile;
#[cfg(any(feature = "python", feature = "python-stubgen"))]
use pyo3::prelude::*;
#[cfg(feature = "python-stubgen")]
use pyo3_stub_gen_derive::gen_stub_pyclass;

pub mod ffi;

#[cfg(test)]
pub(crate) mod ffi_test;

///
/// Calibration Fits
/// This table is present in some metafits files, and if present, contains data from the calibration_solutions
/// database table with a calibration fit from the most recent fitted calibration observation with the same
/// frequency channel setttings as this observation.
///
#[cfg_attr(feature = "python-stubgen", gen_stub_pyclass)]
#[cfg_attr(
    any(feature = "python", feature = "python-stubgen"),
    pyclass(get_all, set_all)
)]
#[derive(Clone, Debug, PartialEq)]
pub struct CalibrationFit {
    /// rf_input (ant, pol)
    pub rf_input: Rfinput,

    /// The calibration offset, in metres, for that input,
    /// derived from the most recently processed calibrator
    /// observation with the same coarse channels.
    /// May be missing or all zeros in some metafits files.
    /// Used to generate the slope (versus frequency) for the phase correction.
    pub delay_metres: f32,
    /// Used, with the phase slope above to generate the phase correction for each fine channel, for this tile.
    pub intercept_metres: f32,
    /// The calibration gain multiplier (not in dB) for each of the N channels (normally 24) in this observation,
    /// for this input. Derived from the most recently processed calibrator observation with the same coarse
    /// channels. May be missing or all ones in some metafits files.
    pub gains: Vec<f32>,
    pub num_gains: usize,
    /// polynomial fit parameter for a more accurate gain correction solution for each of the N channels (normally 24) in this observation
    pub gain_polynomial_fit0: Vec<f32>,
    /// number of gain_polynomial_fit0 elements
    pub num_gain_polynomial_fit0: usize,
    /// polynomial fit parameter for a more accurate gain correction solution for each of the N channels (normally 24) in this observation
    pub gain_polynomial_fit1: Vec<f32>,
    /// number of gain_polynomial_fit1 elements
    pub num_gain_polynomial_fit1: usize,
    /// dimensionless quality parameter (0-1.0) for the phase fit
    pub phase_fit_quality: f32,
    /// dimensionless quality parameter (0-1.0) for the gain fit
    pub gain_fit_quality: f32,
}

/// Implements fmt::Display for CalibrationFit
///
/// # Arguments
///
/// * `f` - A fmt::Formatter
///
///
/// # Returns
///
/// * `fmt::Result` - Result of this method
///
///
impl fmt::Display for CalibrationFit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let gains: String = if !self.gains.is_empty() {
            format!("[{}..{}]", self.gains[0], self.gains[self.gains.len() - 1])
        } else {
            "[]".to_string()
        };

        let gain_polynomial_fit0: String = if !self.gain_polynomial_fit0.is_empty() {
            format!(
                "[{}..{}]",
                self.gain_polynomial_fit0[0],
                self.gain_polynomial_fit0[self.gain_polynomial_fit0.len() - 1]
            )
        } else {
            "[]".to_string()
        };

        let gain_polynomial_fit1: String = if !self.gain_polynomial_fit1.is_empty() {
            format!(
                "[{}..{}]",
                self.gain_polynomial_fit1[0],
                self.gain_polynomial_fit1[self.gain_polynomial_fit1.len() - 1]
            )
        } else {
            "[]".to_string()
        };

        write!(
            f,
            "Rfinput: {:?} Delay (m): {} Intercept (m): {}, Gains: {}, GainPolyFit0: {}, GainPolyFit1: {}, PhaseFitQuality: {}, GainFitQuality: {}",
            self.rf_input, self.delay_metres, self.intercept_metres, gains, gain_polynomial_fit0, gain_polynomial_fit1, self.phase_fit_quality, self.gain_fit_quality
        )
    }
}

/// Read the calibration fits FitsHdu and return a populated vector of `CalibrationFit`s
///
/// # Arguments
///
/// * `metafits_fptr` - reference to the FitsFile representing the metafits file.
///
/// * `calibdata_hdu` - The FitsHdu containing valid calibration fits data.
///
/// * `num_coarse_channels` - The number of coarse channels in the observation (usually 24).
///
/// # Returns
///
/// * Result containing a vector of calibration fits read from the calibdata_hdu HDU.
///
pub(crate) fn populate_calibration_fits(
    metafits_fptr: &mut FitsFile,
    calibdata_hdu: &FitsHdu,
    rf_inputs: &[Rfinput],
    num_coarse_channels: usize,
) -> Result<Vec<CalibrationFit>, error::CalibrationFitError> {
    // Find out how many rows there are in the table
    let rows = match &calibdata_hdu.info {
        HduInfo::TableInfo {
            column_descriptions: _,
            num_rows,
        } => *num_rows,
        _ => 0,
    };

    let mut cal_fit_vec: Vec<CalibrationFit> = Vec::new();

    for row in 0..rows {
        let antenna: u32 = read_cell_value::<u32>(metafits_fptr, calibdata_hdu, "Antenna", row)?;
        let pol: String = read_cell_value::<String>(metafits_fptr, calibdata_hdu, "Pol", row)?;

        // Get the rf_input based on the antenna and pol
        let rf_input: Rfinput = rf_inputs
            .iter()
            .find(|&r| r.ant == antenna && r.pol.to_string() == pol)
            .cloned()
            .ok_or(
                error::CalibrationFitError::NoRfInputFoundForCalibdataAntennaPol {
                    antenna: antenna as usize,
                    pol: pol.clone(),
                },
            )?;

        // older metafits files may have "Calib_Delay" for the delay column
        let delay_metres: f32 =
            match read_cell_value::<f32>(metafits_fptr, calibdata_hdu, "delay_m", row) {
                Ok(val) => val,
                Err(_) => read_cell_value::<f32>(metafits_fptr, calibdata_hdu, "Calib_Delay", row)?,
            };

        // Older metafits files may be missing the intercept column
        let intercept_metres: f32 =
            read_cell_value::<f32>(metafits_fptr, calibdata_hdu, "Intercept", row)
                .unwrap_or_default();

        let gains: Vec<f32> = match read_cell_array_f32(
            metafits_fptr,
            calibdata_hdu,
            "gains",
            row as i64,
            num_coarse_channels,
        ) {
            Ok(val) => val,
            Err(_) => read_cell_array_f32(
                metafits_fptr,
                calibdata_hdu,
                "Calib_Gains",
                row as i64,
                num_coarse_channels,
            )?,
        };

        // older metafits files may be missing the gain polynomial fit columns
        let gain_polynomial_fit0: Vec<f32> = read_cell_array_f32(
            metafits_fptr,
            calibdata_hdu,
            "gains_pol0",
            row as i64,
            num_coarse_channels,
        )
        .unwrap_or_default();

        // older metafits files may be missing the gain polynomial fit columns
        let gain_polynomial_fit1: Vec<f32> = read_cell_array_f32(
            metafits_fptr,
            calibdata_hdu,
            "gains_pol1",
            row as i64,
            num_coarse_channels,
        )
        .unwrap_or_default();

        // older metafits files may be missing the fit quality columns
        let phase_fit_quality: f32 =
            read_cell_value::<f32>(metafits_fptr, calibdata_hdu, "phase_fit_quality", row)
                .unwrap_or_default();

        // older metafits files may be missing the fit quality columns
        let gain_fit_quality: f32 =
            read_cell_value::<f32>(metafits_fptr, calibdata_hdu, "gain_fit_quality", row)
                .unwrap_or_default();

        cal_fit_vec.push(CalibrationFit {
            rf_input,
            delay_metres,
            intercept_metres,
            num_gains: gains.len(),
            gains,
            num_gain_polynomial_fit0: gain_polynomial_fit0.len(),
            gain_polynomial_fit0,
            num_gain_polynomial_fit1: gain_polynomial_fit1.len(),
            gain_polynomial_fit1,
            phase_fit_quality,
            gain_fit_quality,
        });
    }

    Ok(cal_fit_vec)
}
