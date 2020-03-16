// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for coverting Legacy MWA data into a sensible ordering/format.

Major contributor: Brian Crosse (Curtin Institute for Radio Astronomy)

*/
use crate::misc::*;
use crate::rfinput::*;
use std::fmt;

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
#[allow(non_camel_case_types)]
pub struct mwalibLegacyConversionBaseline {
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

impl mwalibLegacyConversionBaseline {
    /// Create a new populated mwalibLegacyConversionBaseline which represents the conversion table
    /// to work out where in the input data we should pull data from, for the given baseline/ant1/ant2.
    ///
    ///
    /// # Arguments
    ///
    /// See `mwalibLegacyConversionBaseline` struct.
    ///
    ///
    /// # Returns
    ///
    /// * Returns a Result containing a populated mwalibLegacyConversionBaseline if Ok.
    ///
    pub fn new(
        baseline: usize,
        ant1: usize,
        ant2: usize,
        xx: i32,
        xy: i32,
        yx: i32,
        yy: i32,
    ) -> Self {
        Self {
            baseline,
            ant1,
            ant2,
            xx_index: xx.abs() as usize,
            xx_conjugate: xx < 0,
            xy_index: xy.abs() as usize,
            xy_conjugate: xy < 0,
            yx_index: yx.abs() as usize,
            yx_conjugate: yx < 0,
            yy_index: yy.abs() as usize,
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
#[cfg_attr(tarpaulin, skip)]
impl fmt::Debug for mwalibLegacyConversionBaseline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}v{} {} {} {} {}",
            self.baseline,
            self.ant1,
            self.ant2,
            self.xx_index,
            self.xy_index,
            self.yx_index,
            self.yy_index
        )
    }
}

/// Generates a full matrix mapping pfb inputs to MWAX format. Used to generate conversion table
/// which is vector of `mwalibLegacyConversionBaseline` structs.
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
/// * `rf_inputs` - A vector containing all of the `mwalibRFInput`s from the metafits.
///
///
/// # Returns
///
/// * A Vector of `mwalibLegacyConversionBaseline`s which tell us, for a specific output baseline, where in the input HDU
/// to get data from (and whether it needs to be conjugated).
///
pub fn generate_conversion_array(
    rf_inputs: &mut Vec<mwalibRFInput>,
) -> Vec<mwalibLegacyConversionBaseline> {
    // Sort the rf_inputs by "Input / metafits" order
    rf_inputs.sort_by(|a, b| a.input.cmp(&b.input));

    // Ensure we have a 256 element array of rf_inputs
    // This is an OK assumption since we only use this for Legacy and OldLegacy MWA data which always must have 128 tiles
    // which is 256 rf inputs.
    assert_eq!(rf_inputs.len(), 256);

    // Create a vector which contains all the mwax_orders, sorted by "input" from the metafits
    let mut mwax_order: Vec<usize> = vec![0; 256];
    for index in 0..256 {
        mwax_order[index] = rf_inputs[index].subfile_order as usize;
    }

    // Generate the full matrix
    let full_matrix: Vec<i32> = generate_full_matrix(mwax_order);
    // Now step through the 256 x 256 square, but in the order of the wanted triangular output!
    // Each step, we need to pick up the source position index that we stored in the 256 x 256 square.
    let (mut xx, mut xy, mut yx, mut yy): (i32, i32, i32, i32); // Indexes to the polarisations for this pair of tiles
    let mut baseline: usize = 0;

    // Create an output vector so we can lookup where to get data from the legacy HDU, given a baseline/ant1/ant2
    let baseline_count = get_baseline_count(128);

    let mut conversion_table: Vec<mwalibLegacyConversionBaseline> =
        Vec::with_capacity(baseline_count as usize);

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

            conversion_table.push(mwalibLegacyConversionBaseline::new(
                baseline, row_tile, col_tile, xx, xy, yx, yy,
            ));

            /* Handy debug to print out the output lookup for each baseline/pol
            println!(
                "{}:{},{}  {}:{}:{}:{}",
                baseline, row_tile, col_tile, xx, xy, yx, yy
            );*/

            baseline += 1;
        }
    }

    // Ensure we processed all baselines
    assert_eq!(baseline, baseline_count as usize);
    assert_eq!(conversion_table.len(), baseline_count as usize);

    conversion_table
}

/// Using the precalculated conversion table, reorder the legacy visibilities into our preferred output order
/// [time][baseline][freq][pol] in a standard triangle of 0,0 .. 0,N 1,1..1,N baseline order.
/// # Arguments
///
/// * `conversion_table` - A vector containing all of the `mwalibLegacyConversionBaseline`s we have pre-calculated.
///
/// * `input_buffer` - Float vector read from legacy MWA HDUs.
///
/// * `output_buffer` - Float vector to write converted data into.
///
/// * `num_fine_channels` - Number of file channles in this observation.
///
///
/// # Returns
///
/// * Nothing
///
/// # TODO
/// Better error handling by returning a Result with associated Errors. Right now it just panics.
///
pub fn convert_legacy_hdu(
    conversion_table: &[mwalibLegacyConversionBaseline],
    input_buffer: &[f32],
    output_buffer: &mut [f32],
    num_fine_channels: usize,
) {
    // Note: hardcoded values are safe here because they are only for the case where we are using the
    // legacy correlator which ALWAYS has 128 tiles
    let num_baselines = get_baseline_count(128);
    assert_eq!(num_fine_channels, 128);
    assert_eq!(conversion_table.len(), num_baselines);

    // Striding for input array
    let floats_per_baseline_fine_channel = 8; // xx_r,xx_i,xy_r,xy_i,yx_r,yx_i,yy_r,yy_i
    let floats_per_fine_channel = num_baselines * floats_per_baseline_fine_channel; // All floats for all baselines and 1 fine channel

    // Striding for output array
    let floats_per_baseline = floats_per_baseline_fine_channel * num_fine_channels;

    // Read from the input buffer and write into the temp buffer
    for fine_chan_index in 0..num_fine_channels {
        // convert one fine channel at a time
        for (baseline_index, baseline) in conversion_table.iter().enumerate() {
            // Input visibilities are in [fine_chan][baseline][pol][real][imag] order so we need to stride
            // through it.
            // source index =
            // (fine_chan_index * num_baselines * floats per baseline_chan) +
            // (baseline_index * floats per baseline_chan)

            // We need to work out where to start indexing the source data
            // Go "down" the fine channels as if they are rows
            // Go "across" the baselines as if they are columns
            let source_index = fine_chan_index * floats_per_fine_channel;
            // We need to work out where to start indexing the destination data
            // Go "down" the baselines as if they are rows
            // Go "across" the fine channels as if they are columns
            let destination_index = (baseline_index * floats_per_baseline)
                + (fine_chan_index * floats_per_baseline_fine_channel);
            // xx_r
            output_buffer[destination_index] = input_buffer[source_index + baseline.xx_index];
            // xx_i
            output_buffer[destination_index + 1] = if baseline.xx_conjugate {
                // We have to conjugate the visibility
                -input_buffer[source_index + baseline.xx_index + 1]
            } else {
                input_buffer[source_index + baseline.xx_index + 1]
            };

            // xy_r
            output_buffer[destination_index + 2] = input_buffer[source_index + baseline.xy_index];
            // xy_i
            output_buffer[destination_index + 3] = if baseline.xy_conjugate {
                // We have to conjugate the visibility
                -input_buffer[source_index + baseline.xy_index + 1]
            } else {
                input_buffer[source_index + baseline.xy_index + 1]
            };

            // yx_r
            output_buffer[destination_index + 4] = input_buffer[source_index + baseline.yx_index];
            // yx_i
            output_buffer[destination_index + 5] = if baseline.yx_conjugate {
                // We have to conjugate the visibility
                -input_buffer[source_index + baseline.yx_index + 1]
            } else {
                input_buffer[source_index + baseline.yx_index + 1]
            };

            // yy_r
            output_buffer[destination_index + 6] = input_buffer[source_index + baseline.yy_index];
            // yy_i
            output_buffer[destination_index + 7] = if baseline.yy_conjugate {
                // We have to conjugate the visibility
                -input_buffer[source_index + baseline.yy_index + 1]
            } else {
                input_buffer[source_index + baseline.yy_index + 1]
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{misc, mwalibContext};
    use csv::*;
    use float_cmp::*;

    #[test]
    fn test_fine_pfb_reorder() {
        // This hardcoded vector comes from cotter/pyuvdata/build_lfiles and represents the input to
        // output mapping for a single pfb. To cater for all 4 PFB's we need to loop through it 4 times
        let single_pfb_output_to_input: Vec<usize> = vec![
            0, 16, 32, 48, 1, 17, 33, 49, 2, 18, 34, 50, 3, 19, 35, 51, 4, 20, 36, 52, 5, 21, 37,
            53, 6, 22, 38, 54, 7, 23, 39, 55, 8, 24, 40, 56, 9, 25, 41, 57, 10, 26, 42, 58, 11, 27,
            43, 59, 12, 28, 44, 60, 13, 29, 45, 61, 14, 30, 46, 62, 15, 31, 47, 63,
        ];

        for pfb in 0..4 {
            for (i, pfb_output) in single_pfb_output_to_input.iter().enumerate() {
                let hardcoded = pfb_output + (64 * pfb);
                let calculated = fine_pfb_reorder(i + (64 * pfb));

                assert_eq!(
                    hardcoded, calculated,
                    "fine_pfb_reorder({}) did not equal expected hardcoded value {}",
                    hardcoded, calculated
                );
            }
        }
    }

    #[test]
    fn test_full_matrix() {
        // Use this as the input of mwax_orders, sorted by input (from metafits)
        // This was derived from an example metafits: test_files/1101503312.metafits
        // by sorting by "Input" and then using get_mwax_order(antenna, pol)
        let mwax_order: Vec<usize> = vec![
            151, 150, 149, 148, 147, 146, 145, 144, 159, 158, 157, 156, 155, 154, 153, 152, 231,
            230, 229, 228, 227, 226, 225, 224, 239, 238, 237, 236, 235, 234, 233, 232, 247, 246,
            245, 244, 243, 242, 241, 240, 255, 254, 253, 252, 251, 250, 249, 248, 103, 102, 101,
            100, 99, 98, 97, 96, 111, 110, 109, 108, 107, 106, 105, 104, 23, 22, 21, 20, 19, 18,
            17, 16, 31, 30, 29, 28, 27, 26, 25, 24, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10,
            9, 8, 135, 134, 133, 132, 131, 130, 129, 128, 143, 142, 141, 140, 139, 138, 137, 136,
            119, 118, 117, 116, 115, 114, 113, 112, 127, 126, 125, 124, 123, 122, 121, 120, 183,
            182, 181, 180, 179, 178, 177, 176, 191, 190, 189, 188, 187, 186, 185, 184, 167, 166,
            165, 164, 163, 162, 161, 160, 175, 174, 173, 172, 171, 170, 169, 168, 215, 214, 213,
            212, 211, 210, 209, 208, 223, 222, 221, 220, 219, 218, 217, 216, 199, 198, 197, 196,
            195, 194, 193, 192, 207, 206, 205, 204, 203, 202, 201, 200, 39, 38, 37, 36, 35, 34, 33,
            32, 47, 46, 45, 44, 43, 42, 41, 40, 55, 54, 53, 52, 51, 50, 49, 48, 63, 62, 61, 60, 59,
            58, 57, 56, 87, 86, 85, 84, 83, 82, 81, 80, 95, 94, 93, 92, 91, 90, 89, 88, 71, 70, 69,
            68, 67, 66, 65, 64, 79, 78, 77, 76, 75, 74, 73, 72,
        ];

        // Normally this is generated using the metafits, but we hardcode it above
        assert_eq!(mwax_order.len(), 256);

        // Generate the full_matrix
        let generated_full_matrix: Vec<i32> = generate_full_matrix(mwax_order);

        assert_eq!(generated_full_matrix.len(), (256 * 256));

        let mut csv_full_matrix: Vec<i32> = vec![0; 256 * 256];

        assert_eq!(csv_full_matrix.len(), (256 * 256));

        // Check the generated full matrix against one crafted by "hand", in csv format in test_files/1101503312_full_matrix.csv
        //
        // First read the csv file
        //
        // Build the CSV reader and iterate over each record.
        // The csv file contains 256 rows each containing 256 columns of signed integers
        let mut reader = ReaderBuilder::new()
            .has_headers(false)
            .from_path("test_files/1101503312_1_timestep/1101503312_full_matrix.csv")
            .unwrap();
        for (row_index, result) in reader.deserialize().enumerate() {
            // An error may occur, so abort the program in an unfriendly way.
            let record: Vec<i32> = result.expect("Failed to deserialize CSV");

            assert_eq!(record.len(), 256);
            assert!(row_index < 256, "row_index is out of bounds {}", row_index);

            // Now loop though all the columns in this row
            for (i, v) in record.iter().enumerate() {
                assert!(i < 256);
                let dest_index = (row_index * 256) + i;

                assert!(
                    dest_index < (256 * 256),
                    "dest_index is out of bounds {}",
                    dest_index
                );

                csv_full_matrix[dest_index] = *v;
            }
        }

        // Loop through every row and column and check if the generated matrix == csv matrix!
        for row in 0..256 {
            for col in 0..256 {
                let index = (row * 256) + col;
                assert_eq!(
                    csv_full_matrix[index], generated_full_matrix[index],
                    "on row {}, col {}",
                    row, col
                );
            }
        }
    }

    #[test]
    fn test_conversion_of_legacy_hdu() {
        // Open a context and load in a test metafits and gpubox file
        let metafits: String = String::from("test_files/1101503312_1_timestep/1101503312.metafits");
        let gpuboxfiles: Vec<String> = vec![String::from(
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
        )];
        let mut context =
            mwalibContext::new(&metafits, &gpuboxfiles).expect("Failed to create mwalibContext");

        // Read and convert first HDU
        let mwalib_hdu: Vec<f32> = context.read_by_baseline(0, 0).expect("Error!");

        // Check it
        // Vector is in:
        // [baseline][fine_channel][pol][r/i] order
        //
        assert_eq!(
            mwalib_hdu.len(),
            8256 * 128 * 8,
            "mwalib HDU vector length is wrong {}. Should be {}",
            mwalib_hdu.len(),
            8256 * 128 * 8
        );

        //
        // Next read the csv file
        //
        // Build the CSV reader and iterate over each record.
        // The csv file contains 1056768 (8256 baselines * 128 fine channels) rows
        // each containing each containing 8 floats:
        // XX real, XX imag, XY real, XY imag, YX real, YX imag, YY real, YY imag
        //
        let mut reader = ReaderBuilder::new()
            .has_headers(false)
            .from_path(
                "test_files/1101503312_1_timestep/1101503312_gpubox01_pyuvdata_1st_timestep.csv",
            )
            .expect("Failed to read CSV");

        let mut baseline = 0;
        let mut fine_chan = 0;
        let mut mwalib_sum_of_baseline: f64 = 0.;
        let mut pyuvdata_sum_of_baseline: f64 = 0.;
        let mut ant1: usize = 0;
        let mut ant2: usize = 0;

        for (row_index, result) in reader.deserialize().enumerate() {
            // Verify the baseline matches the antenna numbers
            match misc::get_antennas_from_baseline(baseline, 128) {
                Some(b) => {
                    assert_eq!(ant1, b.0);
                    assert_eq!(ant2, b.1);
                }
                None => panic!("baseline {} is not valid!", baseline),
            }
            let record: Vec<f32> = result.expect("Failed to deserialize CSV");
            assert!(
                row_index < 1_056_768,
                "row_index is out of bounds {}",
                row_index
            );

            // Determine where we should be in the mwalib array
            let mwalib_bl_index = baseline * (128 * 8);
            let mwalib_ch_index = fine_chan * 8;
            // Now loop though all the columns in this row
            for (i, v) in record.iter().enumerate() {
                assert!(i < 8);
                let mwalib_dest_index = mwalib_bl_index + mwalib_ch_index + i;

                let mwalib_value = mwalib_hdu[mwalib_dest_index] as f64;
                let pyuvdata_value = *v as f64;

                mwalib_sum_of_baseline += mwalib_value;
                pyuvdata_sum_of_baseline += pyuvdata_value;

                let pol = match i {
                    0 => "xx_r",
                    1 => "xx_i",
                    2 => "xy_r",
                    3 => "xy_i",
                    4 => "yx_r",
                    5 => "yx_i",
                    6 => "yy_r",
                    7 => "yy_i",
                    _ => "?",
                };

                assert!(approx_eq!(f64, mwalib_value, pyuvdata_value, F64Margin::default()), "baseline: {} ant1: {} v ant2: {} fine_chan: {} pol: {} mwalib_value: {} != pyuvdata_value: {} difference: {}", baseline, ant1, ant2, fine_chan, pol, mwalib_value, pyuvdata_value, mwalib_value - pyuvdata_value);
            }

            if fine_chan < 127 {
                fine_chan += 1;
            } else {
                // We are at the end of a baseline
                // Get value from mwa_lib
                let good: u8 = if approx_eq!(
                    f64,
                    mwalib_sum_of_baseline,
                    pyuvdata_sum_of_baseline,
                    F64Margin::default()
                ) {
                    // match
                    1
                } else {
                    // no match
                    0
                };

                assert_eq!(
                    good,
                    1,
                    "baseline: {} ant1: {} v ant2: {} mwalib_sum: {} != pyuvdata_sum: {} difference: {}",
                    baseline,
                    ant1,
                    ant2,
                    mwalib_sum_of_baseline,
                    pyuvdata_sum_of_baseline,
                    mwalib_sum_of_baseline - pyuvdata_sum_of_baseline
                );

                // Reset our sums
                mwalib_sum_of_baseline = 0.;
                pyuvdata_sum_of_baseline = 0.;

                // Reset counters
                fine_chan = 0;
                baseline += 1;
                if ant2 < 127 {
                    ant2 += 1;
                } else {
                    ant1 += 1;
                    ant2 = ant1;
                }
            }
        }
    }
}
