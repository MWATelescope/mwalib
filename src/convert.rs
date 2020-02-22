use crate::misc::*;
use crate::rfinput::*;
use crate::*;
use std::fmt;

/// This "macro" flips the bits in an 8 bit number such that bit order abcdefgh becomes abghcdef
///	This is the reordering required to undo the order imposed by the fine-PFB hardware.
/// Not as confusing as it looks!  Take the left two bits and leave them where they are,
///	then 'or' the bottom 2 bit after shifting them left 4 positions,
///	then 'or' the middle 4 bits after shifting them right 2 positions
/// It is inlined so the compiler will effectively make this like a C macro rather than a function call.
#[inline(always)]
fn fine_pfb_reorder(x: usize) -> usize {
    ((x) & 0xc0) | (((x) & 0x03) << 4) | (((x) & 0x3c) >> 2)
}

/// Structure for storing where in the input visibilities to get the specified baseline
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
    pub fn new(
        baseline: usize,
        ant1: usize,
        ant2: usize,
        xx_index: usize,
        xx_conjugate: bool,
        xy_index: usize,
        xy_conjugate: bool,
        yx_index: usize,
        yx_conjugate: bool,
        yy_index: usize,
        yy_conjugate: bool,
    ) -> Result<mwalibLegacyConversionBaseline, ErrorKind> {
        Ok(mwalibLegacyConversionBaseline {
            baseline,
            ant1,
            ant2,
            xx_index,
            xx_conjugate,
            xy_index,
            xy_conjugate,
            yx_index,
            yx_conjugate,
            yy_index,
            yy_conjugate,
        })
    }
}

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

pub fn generate_conversion_array(
    rf_inputs: &mut Vec<mwalibRFInput>,
) -> Vec<mwalibLegacyConversionBaseline> {
    let mut row1st: usize;
    let mut row2nd: usize;
    let mut col_a: usize;
    let mut col_b: usize; // temp space for the rf_inputs that form the row and column of the 2x2 correlation matrix

    // Sort the rf_inputs by "Input / metafits" order
    rf_inputs.sort_by(|a, b| a.input.cmp(&b.input));

    // Ensure we have a 256 element array of rf_inputs
    assert_eq!(rf_inputs.len(), 256);

    // Pull subfile order out into seperate vector, ensuring it is sorted by input/metafits order
    let mut mwax_order: Vec<usize> = vec![0; 256];

    for index in 0..256 {
        mwax_order[index] = rf_inputs[index].subfile_order as usize;
    }
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
            full_matrix[((row1st << 8) | col_b)] = source_legacy_ndx; // Top right
            source_legacy_ndx += 1;

            if col_order != row_order {
                // Unless it's one of the 128 redundant outputs from the old correlator
                full_matrix[((row2nd << 8) | col_a)] = source_legacy_ndx; // Here is the Bottom Left.  NB the source index *isn't* incremented during the 'if'
            }

            source_legacy_ndx += 1; // because we need to increment the source location even if we're ignoring it.

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

            conversion_table.push(
                mwalibLegacyConversionBaseline::new(
                    baseline,
                    row_tile,
                    col_tile,
                    {
                        if xx < 0 {
                            -xx
                        } else {
                            xx
                        }
                    } as usize,
                    xx < 0,
                    {
                        if xy < 0 {
                            -xy
                        } else {
                            xy
                        }
                    } as usize,
                    xy < 0,
                    {
                        if yx < 0 {
                            -yx
                        } else {
                            yx
                        }
                    } as usize,
                    yx < 0,
                    {
                        if yy < 0 {
                            -yy
                        } else {
                            yy
                        }
                    } as usize,
                    yy < 0,
                )
                .unwrap(),
            );

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
pub fn convert_legacy_hdu(
    conversion_table: &Vec<mwalibLegacyConversionBaseline>,
    input_buffer: &[f32],
    output_buffer: &mut [f32],
    num_fine_channels: usize,
) {
    assert_eq!(input_buffer.len(), output_buffer.len());

    let num_baselines = conversion_table.len();

    // Striding for input array
    let floats_per_baseline_fine_channel = 8; // xx_r,xx_i,xy_r,xy_i,yx_r,yx_i,yy_r,yy_i

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
            let source_index = fine_chan_index * num_baselines * floats_per_baseline_fine_channel;
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
