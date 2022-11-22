// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Structs and helper methods for coverting Legacy MWA data into a sensible
//! ordering/format.
//!
//! Major contributor: Brian Crosse (Curtin Institute for Radio Astronomy)

use crate::misc::*;
use crate::rfinput::*;
use log::trace;
use std::fmt;

#[cfg(test)]
mod test;

/// This "macro" flips the bits in an 8 bit number such that bit order abcdefgh becomes abghcdef
/// This is the reordering required to undo the order imposed by the fine-PFB hardware.
/// Not as confusing as it looks!  Take the left two bits and leave them where they are,
/// then 'or' the bottom 2 bit after shifting them left 4 positions,
/// then 'or' the middle 4 bits after shifting them right 2 positions
/// It is inlined so the compiler will effectively make this like a C macro rather than a function call.
///
/// # Arguments
///
/// * `input` - Fine PFB input index.
///
///
/// # Returns
///
/// * The correctly reordered rf_input index.
fn fine_pfb_reorder(input: usize) -> usize {
    ((input) & 0xc0) | (((input) & 0x03) << 4) | (((input) & 0x3c) >> 2)
}

/// Structure for storing where in the input visibilities to get the specified baseline when converting
pub(crate) struct LegacyConversionBaseline {
    pub baseline: usize,    // baseline index
    pub ant1: usize,        // antenna1 index
    pub ant2: usize,        // antenna2 index
    pub xx_index: usize,    // index of where complex xx is in the input buffer
    pub xx_conjugate: bool, // if true, we need to conjugate this visibility
    pub xy_index: usize,    // index of where complex xx is in the input buffer
    pub xy_conjugate: bool, // if true, we need to conjugate this visibility
    pub yx_index: usize,    // index of where complex xx is in the input buffer
    pub yx_conjugate: bool, // if true, we need to conjugate this visibility
    pub yy_index: usize,    // index of where complex xx is in the input buffer
    pub yy_conjugate: bool, // if true, we need to conjugate this visibility
}

impl LegacyConversionBaseline {
    /// Create a new populated
    ///LegacyConversionBaseline which represents the conversion table
    /// to work out where in the input data we should pull data from, for the given baseline/ant1/ant2.
    ///
    ///
    /// # Arguments
    ///
    /// See `
    ///LegacyConversionBaseline` struct.
    ///
    ///
    /// # Returns
    ///
    /// * Returns a Result containing a populated
    ///LegacyConversionBaseline if Ok.
    ///
    fn new(baseline: usize, ant1: usize, ant2: usize, xx: i32, xy: i32, yx: i32, yy: i32) -> Self {
        Self {
            baseline,
            ant1,
            ant2,
            xx_index: xx.unsigned_abs() as usize,
            xx_conjugate: xx < 0,
            xy_index: xy.unsigned_abs() as usize,
            xy_conjugate: xy < 0,
            yx_index: yx.unsigned_abs() as usize,
            yx_conjugate: yx < 0,
            yy_index: yy.unsigned_abs() as usize,
            yy_conjugate: yy < 0,
        }
    }
}

/// Implements fmt::Debug for mwalibTimeStep struct
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
impl fmt::Debug for LegacyConversionBaseline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{},{},{},{},{},{},{},{},{},{},{}",
            self.baseline,
            self.ant1,
            self.ant2,
            self.xx_index, // xx_real
            /* Note since we conjugate ALL imaginaries at the end,
            we can simplify so that if the conjugate is True then we leave the sign alone (equivalent of -(-index)).
            if conjugate is false, we conjugate (equivalent of -index) */
            // xx_imag
            if self.xx_conjugate {
                self.xx_index as isize
            } else {
                -(self.xx_index as isize)
            },
            self.xy_index, // xy_real
            // xy_imag
            if self.xy_conjugate {
                self.xy_index as isize
            } else {
                -(self.xy_index as isize)
            },
            self.yx_index, // yx_real
            // yx_imag
            if self.yx_conjugate {
                self.yx_index as isize
            } else {
                -(self.yx_index as isize)
            },
            self.yy_index, // yy_real
            // yy_imag
            if self.yy_conjugate {
                self.yy_index as isize
            } else {
                -(self.yy_index as isize)
            }
        )
    }
}

/// Generates a full matrix mapping pfb inputs to MWAX format. Used to generate conversion table
/// which is vector of `
///LegacyConversionBaseline` structs.
///
///
/// # Arguments
///
/// * `mwax_order` - A vector containing the MWAX order of rf_inputs.
///
///
/// # Returns
///
/// * A Vector with one element per rf_input vs rf_input (256x256). Positive numbers represent the index of the
/// input HDU to get data from, negative numbers mean to take the complex conjugate of the data at the index of
/// the input HDU.
///
fn generate_full_matrix(mwax_order: Vec<usize>) -> Vec<i32> {
    let mut row1st: usize;
    let mut row2nd: usize;
    let mut col_a: usize;
    let mut col_b: usize; // temp space for the rf_inputs that form the row and column of the 2x2 correlation matrix
                          // Pull subfile order out into seperate vector, ensuring it is sorted by input/metafits order
    assert_eq!(mwax_order.len(), 256);

    // Create an array of 65536 ints.  Really a 2d array of 256 x 256 ints
    // set them all to -1 to say 'contains nothing useful'
    let mut full_matrix: Vec<i32> = vec![-1; 65536];
    // We're going to iterate through the legacy complex numbers in the order they appear.
    let mut source_legacy_ndx: i32 = 0;

    // Below loop is equivalent to: for ( col_order = 0 ; col_order < 256 ; col_order+=2 )
    for col_order in (0..256).step_by(2) {
        // For a while we'll be working with two inputs that appear as the columns in our 2x2 correlation square.  Let's look them up now.
        col_a = mwax_order[fine_pfb_reorder(col_order)]; // We need to access in *not* in metafits order, but the order the fine-PFB uses
        col_b = mwax_order[fine_pfb_reorder(col_order + 1)];

        // below for loop is equivalent to: for ( row_order = 0 ; row_order <= col_order ; row_order+=2 )
        for row_order in (0..=col_order).step_by(2) {
            // Right now, we need to know which two inputs appear as the rows in our 2x2 correlation square.  Let's look them up now.
            row1st = mwax_order[fine_pfb_reorder(row_order)];
            row2nd = mwax_order[fine_pfb_reorder(row_order + 1)];

            full_matrix[((row1st << 8) | col_a)] = source_legacy_ndx; // Top left complex number in the 2x2 correlation square
            source_legacy_ndx += 1;
            // Unless it's one of the 128 redundant outputs from the old correlator
            if col_order != row_order {
                full_matrix[((row2nd << 8) | col_a)] = source_legacy_ndx; // Bottom left
            }
            source_legacy_ndx += 1; // NB the source index *isn't* incremented during the 'if'
            full_matrix[((row1st << 8) | col_b)] = source_legacy_ndx; // Here is the Top right.
            source_legacy_ndx += 1;

            full_matrix[((row2nd << 8) | col_b)] = source_legacy_ndx; // Bottom Right complex number in the 2x2.
            source_legacy_ndx += 1;
        }
    }

    for row_order in 0..256 {
        // Now we want to fill in the pointers to conjugates where we don't have the value itself
        for col_order in 0..256 {
            // Go through every cell by row and column
            if full_matrix[(row_order << 8 | col_order)] == -1 {
                // If the entry is currently empty (represented by -1)
                full_matrix[(row_order << 8 | col_order)] =
                    -(full_matrix[(col_order << 8 | row_order)]); // copy the result from the inverse and negate it
            }
            // Useful debug
            // print!("{},", full_matrix[row_order << 8 | col_order]);
        }
    }

    full_matrix
}

/// This takes the rf_inputs from the metafis and generates the conversion array for use when we convert legacy HDUs.
///
/// # Arguments
///
/// * `rf_inputs` - A vector containing all of the `RFInput`s from the metafits.
///
///
/// # Returns
///
/// * A Vector of `
///LegacyConversionBaseline`s which tell us, for a specific output baseline, where in the input HDU
/// to get data from (and whether it needs to be conjugated).
///
pub(crate) fn generate_conversion_array(rf_inputs: &[Rfinput]) -> Vec<LegacyConversionBaseline> {
    // Ensure we have a 256 element array of rf_inputs
    // This is an OK assumption since we only use this for Legacy and OldLegacy MWA data which always must have 128 tiles
    // which is 256 rf inputs.
    assert_eq!(rf_inputs.len(), 256);

    // Create a vector which contains all the mwax_orders, sorted by "input" from the metafits
    let mut map = rf_inputs
        .iter()
        .map(|rf| (rf.input, rf.subfile_order))
        .collect::<Vec<_>>();
    map.sort_unstable();
    let mwax_order = map
        .into_iter()
        .map(|(_, subfile_order)| subfile_order as usize)
        .collect();

    // Generate the full matrix
    let full_matrix: Vec<i32> = generate_full_matrix(mwax_order);
    // Now step through the 256 x 256 square, but in the order of the wanted triangular output!
    // Each step, we need to pick up the source position index that we stored in the 256 x 256 square.
    let (mut xx, mut xy, mut yx, mut yy): (i32, i32, i32, i32); // Indexes to the polarisations for this pair of tiles
    let mut baseline: usize = 0;

    // Create an output vector so we can lookup where to get data from the legacy HDU, given a baseline/ant1/ant2
    let baseline_count = get_baseline_count(128);

    let mut conversion_table: Vec<LegacyConversionBaseline> = Vec::with_capacity(baseline_count);

    // Our row tile and column tile.  Now 2 pols each so only 128 in legacy obs
    for row_tile in 0..128 {
        for col_tile in row_tile..128 {
            // The following indicies are for the complex pair of values
            // To get the individual real or imaginary we need to multiply by 2
            // Therefore the imag value will be the index of the real, plus 1.
            xx = full_matrix[(row_tile * 2) << 8 | (col_tile * 2)] * 2;
            xy = full_matrix[(row_tile * 2) << 8 | (col_tile * 2 + 1)] * 2;
            yx = full_matrix[(row_tile * 2 + 1) << 8 | (col_tile * 2)] * 2;
            yy = full_matrix[(row_tile * 2 + 1) << 8 | (col_tile * 2 + 1)] * 2;

            conversion_table.push(LegacyConversionBaseline::new(
                baseline, row_tile, col_tile, xx, xy, yx, yy,
            ));

            baseline += 1;
        }
    }

    // Ensure we processed all baselines
    assert_eq!(baseline, baseline_count);
    assert_eq!(conversion_table.len(), baseline_count);

    trace!("legacy_conversion_table: {:?}", conversion_table);

    conversion_table
}

/// Using the precalculated conversion table, reorder the legacy visibilities into our preferred output order
/// [time][baseline][freq][pol] in a standard triangle of 0,0 .. 0,N 1,1..1,N baseline order.
/// # Arguments
///
/// * `conversion_table` - A vector containing all of the `
///LegacyConversionBaseline`s we have pre-calculated.
///
/// * `input_buffer` - Float vector read from legacy MWA HDUs.
///
/// * `output_buffer` - Float vector to write converted data into.
///
/// * `num_fine_chans` - Number of file channels in this observation.
///
///
/// # Returns
///
/// * Nothing
///
///
pub(crate) fn convert_legacy_hdu_to_mwax_baseline_order(
    conversion_table: &[LegacyConversionBaseline],
    input_buffer: &[f32],
    output_buffer: &mut [f32],
    num_fine_chans: usize,
) {
    // Note: hardcoded values are safe here because they are only for the case where we are using the
    // legacy correlator which ALWAYS has 128 tiles
    let num_baselines = get_baseline_count(128);

    // Striding for input array
    let floats_per_baseline_fine_chan = 8; // xx_r,xx_i,xy_r,xy_i,yx_r,yx_i,yy_r,yy_i
    let floats_per_fine_chan = num_baselines * floats_per_baseline_fine_chan; // All floats for all baselines and 1 fine channel

    // Striding for output array
    let floats_per_baseline = floats_per_baseline_fine_chan * num_fine_chans;

    assert!(input_buffer.len() >= num_fine_chans * floats_per_fine_chan);
    assert!(output_buffer.len() >= num_fine_chans * floats_per_fine_chan);

    // Read from the input buffer and write into the temp buffer
    // convert one fine channel at a time
    for fine_chan_index in 0..num_fine_chans {
        for (baseline_index, baseline) in conversion_table.iter().enumerate() {
            // Input visibilities are in [fine_chan][baseline][pol][real][imag] order so we need to stride
            // through it.

            // We need to work out where to start indexing the source data
            // Go "down" the fine channels as if they are rows
            // Go "across" the baselines as if they are columns
            let source_index = fine_chan_index * floats_per_fine_chan;
            // We need to work out where to start indexing the destination data
            // Go "down" the baselines as if they are rows
            // Go "across" the fine channels as if they are columns
            let destination_index = (baseline_index * floats_per_baseline)
                + (fine_chan_index * floats_per_baseline_fine_chan);

            unsafe {
                // xx_r
                *output_buffer.get_unchecked_mut(destination_index) =
                    *input_buffer.get_unchecked(source_index + baseline.xx_index);
                // xx_i
                *output_buffer.get_unchecked_mut(destination_index + 1) = if baseline.xx_conjugate {
                    // We have to conjugate the visibility
                    -input_buffer.get_unchecked(source_index + baseline.xx_index + 1)
                } else {
                    *input_buffer.get_unchecked(source_index + baseline.xx_index + 1)
                };

                // xy_r
                *output_buffer.get_unchecked_mut(destination_index + 2) =
                    *input_buffer.get_unchecked(source_index + baseline.xy_index);
                // xy_i
                *output_buffer.get_unchecked_mut(destination_index + 3) = if baseline.xy_conjugate {
                    // We have to conjugate the visibility
                    -input_buffer.get_unchecked(source_index + baseline.xy_index + 1)
                } else {
                    *input_buffer.get_unchecked(source_index + baseline.xy_index + 1)
                };

                // yx_r
                *output_buffer.get_unchecked_mut(destination_index + 4) =
                    *input_buffer.get_unchecked(source_index + baseline.yx_index);
                // yx_i
                *output_buffer.get_unchecked_mut(destination_index + 5) = if baseline.yx_conjugate {
                    // We have to conjugate the visibility
                    -input_buffer.get_unchecked(source_index + baseline.yx_index + 1)
                } else {
                    *input_buffer.get_unchecked(source_index + baseline.yx_index + 1)
                };

                // yy_r
                *output_buffer.get_unchecked_mut(destination_index + 6) =
                    *input_buffer.get_unchecked(source_index + baseline.yy_index);
                // yy_i
                *output_buffer.get_unchecked_mut(destination_index + 7) = if baseline.yy_conjugate {
                    // We have to conjugate the visibility
                    -input_buffer.get_unchecked(source_index + baseline.yy_index + 1)
                } else {
                    *input_buffer.get_unchecked(source_index + baseline.yy_index + 1)
                };

                // Finally take the conjugate so we are in the correct triangle
                *output_buffer.get_unchecked_mut(destination_index + 1) *= -1.0;
                *output_buffer.get_unchecked_mut(destination_index + 3) *= -1.0;
                *output_buffer.get_unchecked_mut(destination_index + 5) *= -1.0;
                *output_buffer.get_unchecked_mut(destination_index + 7) *= -1.0;
            }
        }
    }
}

/// Using the precalculated conversion table, reorder the legacy visibilities into our preferred output order
/// [time][freq][baseline][pol] in a standard triangle of 0,0 .. 0,N 1,1..1,N baseline order.
/// # Arguments
///
/// * `conversion_table` - A vector containing all of the `
///LegacyConversionBaseline`s we have pre-calculated.
///
/// * `input_buffer` - Float vector read from legacy MWA HDUs.
///
/// * `output_buffer` - Float vector to write converted data into.
///
/// * `num_fine_chans` - Number of file channels in this observation.
///
///
/// # Returns
///
/// * Nothing
///
///
pub(crate) fn convert_legacy_hdu_to_mwax_frequency_order(
    conversion_table: &[LegacyConversionBaseline],
    input_buffer: &[f32],
    output_buffer: &mut [f32],
    num_fine_chans: usize,
) {
    // Note: hardcoded values are safe here because they are only for the case where we are using the
    // legacy correlator which ALWAYS has 128 tiles
    let num_baselines = get_baseline_count(128);

    // Striding for input array
    let floats_per_baseline_fine_chan = 8; // xx_r,xx_i,xy_r,xy_i,yx_r,yx_i,yy_r,yy_i
    let floats_per_fine_chan = num_baselines * floats_per_baseline_fine_chan; // All floats for all baselines and 1 fine channel

    assert!(input_buffer.len() >= num_fine_chans * floats_per_fine_chan);
    assert!(output_buffer.len() >= num_fine_chans * floats_per_fine_chan);

    // Read from the input buffer and write into the temp buffer
    for fine_chan_index in 0..num_fine_chans {
        // convert one fine channel at a time
        for (baseline_index, baseline) in conversion_table.iter().enumerate() {
            // Input visibilities are in [fine_chan][baseline][pol][real][imag] order
            // We need to work out where to start indexing the source data
            // Go "down" the fine channels as if they are rows
            // Go "across" the baselines as if they are columns
            let source_index = fine_chan_index * floats_per_fine_chan;
            // Since the destination is also to be in [fine_chan][baseline][pol][real][imag] order
            // For the destination, we have to stride along each baseline for this channel
            let destination_index = source_index + (baseline_index * floats_per_baseline_fine_chan);

            unsafe {
                // xx_r
                *output_buffer.get_unchecked_mut(destination_index) =
                    *input_buffer.get_unchecked(source_index + baseline.xx_index);
                // xx_i
                *output_buffer.get_unchecked_mut(destination_index + 1) = if baseline.xx_conjugate {
                    // We have to conjugate the visibility
                    -input_buffer.get_unchecked(source_index + baseline.xx_index + 1)
                } else {
                    *input_buffer.get_unchecked(source_index + baseline.xx_index + 1)
                };

                // xy_r
                *output_buffer.get_unchecked_mut(destination_index + 2) =
                    *input_buffer.get_unchecked(source_index + baseline.xy_index);
                // xy_i
                *output_buffer.get_unchecked_mut(destination_index + 3) = if baseline.xy_conjugate {
                    // We have to conjugate the visibility
                    -input_buffer.get_unchecked(source_index + baseline.xy_index + 1)
                } else {
                    *input_buffer.get_unchecked(source_index + baseline.xy_index + 1)
                };

                // yx_r
                *output_buffer.get_unchecked_mut(destination_index + 4) =
                    *input_buffer.get_unchecked(source_index + baseline.yx_index);
                // yx_i
                *output_buffer.get_unchecked_mut(destination_index + 5) = if baseline.yx_conjugate {
                    // We have to conjugate the visibility
                    -input_buffer.get_unchecked(source_index + baseline.yx_index + 1)
                } else {
                    *input_buffer.get_unchecked(source_index + baseline.yx_index + 1)
                };

                // yy_r
                *output_buffer.get_unchecked_mut(destination_index + 6) =
                    *input_buffer.get_unchecked(source_index + baseline.yy_index);
                // yy_i
                *output_buffer.get_unchecked_mut(destination_index + 7) = if baseline.yy_conjugate {
                    // We have to conjugate the visibility
                    -input_buffer.get_unchecked(source_index + baseline.yy_index + 1)
                } else {
                    *input_buffer.get_unchecked(source_index + baseline.yy_index + 1)
                };

                // Finally take the conjugate so we are in the correct triangle
                *output_buffer.get_unchecked_mut(destination_index + 1) *= -1.0;
                *output_buffer.get_unchecked_mut(destination_index + 3) *= -1.0;
                *output_buffer.get_unchecked_mut(destination_index + 5) *= -1.0;
                *output_buffer.get_unchecked_mut(destination_index + 7) *= -1.0;
            }
        }
    }
}

/// Reorder correlator v2 (MWAX) visibilities into our preferred output order
/// [time][freq][baseline][pol]. The antennas/baselines are already in our preferred order.
/// # Arguments
///
/// * `input_buffer` - Float vector read from MWAX HDUs.
///
/// * `output_buffer` - Float vector to write converted data into.
///
/// * `num_baselines` - Number of baselines in this observation.
///
/// * `num_fine_chans` - Number of file channels in this observation.
///
///
/// # Returns
///
/// * Nothing
///
///
pub(crate) fn convert_mwax_hdu_to_frequency_order(
    input_buffer: &[f32],
    output_buffer: &mut [f32],
    num_baselines: usize,
    num_fine_chans: usize,
    num_visibility_pols: usize,
) {
    // Striding for input array
    let floats_per_baseline_fine_chan = num_visibility_pols * 2; // xx_r,xx_i,xy_r,xy_i,yx_r,yx_i,yy_r,yy_i
    let floats_per_baseline = num_fine_chans * floats_per_baseline_fine_chan; // All floats for 1 baseline and all fine channels
    let floats_per_fine_chan = num_baselines * floats_per_baseline_fine_chan; // All floats for all baselines and 1 fine channel

    assert!(input_buffer.len() >= num_fine_chans * floats_per_fine_chan);
    assert!(output_buffer.len() >= num_fine_chans * floats_per_fine_chan);

    // Read from the input buffer and write into the temp buffer
    for baseline_index in 0..num_baselines {
        // convert one baseline at a time
        for fine_chan_index in 0..num_fine_chans {
            // Input visibilities are in [baseline][fine_chan][pol][real][imag] order
            //
            // We need to work out where to start indexing the source data
            // Go "down" the baselines as if they are rows
            // Go "across" the fine_chans as if they are columns
            let source_index = (baseline_index * floats_per_baseline)
                + (fine_chan_index * floats_per_baseline_fine_chan);
            // The destination is to be in [fine_chan][baseline][pol][real][imag] order
            // For the destination, we have to stride along each fine channel for this baseline
            let destination_index = (fine_chan_index * floats_per_fine_chan)
                + (baseline_index * floats_per_baseline_fine_chan);
            // for each polarisation (r,i) => xx_r, xx_i, xy_r, xy_i, ... copy input to output
            // Copy source into dest
            output_buffer[destination_index..(floats_per_baseline_fine_chan + destination_index)]
                .clone_from_slice(
                    &input_buffer[source_index..(floats_per_baseline_fine_chan + source_index)],
                );
        }
    }
}
