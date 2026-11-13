// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Thin wrappers that let the benchmarks in `benches/` call internal code.
//!
//! This module is only built with the `bench-internals` feature. It is not part of the public API and can change or
//! go away at any time. Do not use it outside the benchmarks.

use crate::convert::{self, LegacyConversionBaseline};
use crate::rfinput::Rfinput;

/// An opaque legacy conversion table, as `CorrelatorContext` builds it from the metafits rf inputs.
pub struct LegacyConversionTable(Vec<LegacyConversionBaseline>);

impl LegacyConversionTable {
    /// Build the legacy conversion table from the rf inputs of a legacy (128 tile) metafits.
    ///
    /// # Arguments
    ///
    /// * `rf_inputs` - The rf inputs of a legacy metafits (256 of them).
    ///
    /// # Returns
    ///
    /// * The conversion table.
    ///
    pub fn new(rf_inputs: &[Rfinput]) -> Self {
        Self(convert::generate_conversion_array(rf_inputs))
    }
}

/// Calls the internal `convert_legacy_hdu_to_mwax_baseline_order`.
///
/// # Arguments
///
/// * `table` - The legacy conversion table.
///
/// * `input_buffer` - Floats as read from a legacy HDU.
///
/// * `output_buffer` - Floats to write the converted data into.
///
/// * `num_fine_chans` - Number of fine channels per coarse channel.
///
pub fn convert_legacy_hdu_to_mwax_baseline_order(
    table: &LegacyConversionTable,
    input_buffer: &[f32],
    output_buffer: &mut [f32],
    num_fine_chans: usize,
) {
    convert::convert_legacy_hdu_to_mwax_baseline_order(
        &table.0,
        input_buffer,
        output_buffer,
        num_fine_chans,
    );
}

/// Calls the internal `convert_legacy_hdu_to_mwax_frequency_order`.
///
/// # Arguments
///
/// * `table` - The legacy conversion table.
///
/// * `input_buffer` - Floats as read from a legacy HDU.
///
/// * `output_buffer` - Floats to write the converted data into.
///
/// * `num_fine_chans` - Number of fine channels per coarse channel.
///
pub fn convert_legacy_hdu_to_mwax_frequency_order(
    table: &LegacyConversionTable,
    input_buffer: &[f32],
    output_buffer: &mut [f32],
    num_fine_chans: usize,
) {
    convert::convert_legacy_hdu_to_mwax_frequency_order(
        &table.0,
        input_buffer,
        output_buffer,
        num_fine_chans,
    );
}

/// Calls the internal `convert_mwax_hdu_to_frequency_order`.
///
/// # Arguments
///
/// * `input_buffer` - Floats as read from an MWAX HDU.
///
/// * `output_buffer` - Floats to write the converted data into.
///
/// * `num_baselines` - Number of baselines.
///
/// * `num_fine_chans` - Number of fine channels per coarse channel.
///
/// * `num_visibility_pols` - Number of visibility polarisations.
///
pub fn convert_mwax_hdu_to_frequency_order(
    input_buffer: &[f32],
    output_buffer: &mut [f32],
    num_baselines: usize,
    num_fine_chans: usize,
    num_visibility_pols: usize,
) {
    convert::convert_mwax_hdu_to_frequency_order(
        input_buffer,
        output_buffer,
        num_baselines,
        num_fine_chans,
        num_visibility_pols,
    );
}
