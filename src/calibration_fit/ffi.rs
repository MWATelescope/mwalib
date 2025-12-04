// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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
    /// polynomial fit parameter for a more accurate gain correction solution for each of the N channels (normally 24) in this observation
    pub gain_polynomial_fit0: *mut f32,
    /// polynomial fit parameter for a more accurate gain correction solution for each of the N channels (normally 24) in this observation
    pub gain_polynomial_fit1: *mut f32,
    /// dimensionless quality parameter (0-1.0) for the phase fit
    pub phase_fit_quality: f32,
    /// dimensionless quality parameter (0-1.0) for the gain fit
    pub gain_fit_quality: f32,
}
