// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Unit tests for code dealing with conversion from legacy to mwax format

#[cfg(test)]
use super::*;
use crate::*;
use csv::*;
use float_cmp::*;

#[test]
fn test_fine_pfb_reorder() {
    // This hardcoded vector comes from cotter/pyuvdata/build_lfiles and represents the input to
    // output mapping for a single pfb. To cater for all 4 PFB's we need to loop through it 4 times
    let single_pfb_output_to_input: Vec<usize> = vec![
        0, 16, 32, 48, 1, 17, 33, 49, 2, 18, 34, 50, 3, 19, 35, 51, 4, 20, 36, 52, 5, 21, 37, 53,
        6, 22, 38, 54, 7, 23, 39, 55, 8, 24, 40, 56, 9, 25, 41, 57, 10, 26, 42, 58, 11, 27, 43, 59,
        12, 28, 44, 60, 13, 29, 45, 61, 14, 30, 46, 62, 15, 31, 47, 63,
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
fn test_legacy_conversion_baseline_debug() {
    let lcb = LegacyConversionBaseline {
        baseline: 1,
        ant1: 0,
        ant2: 1,
        xx_conjugate: false,
        xx_index: 2,
        xy_conjugate: false,
        xy_index: 3,
        yx_conjugate: false,
        yx_index: 4,
        yy_conjugate: false,
        yy_index: 5,
    };

    assert_eq!(format!("{:?}", lcb), "1,0,1,2,-2,3,-3,4,-4,5,-5\n");
}

#[test]
fn test_full_matrix() {
    // Use this as the input of mwax_orders, sorted by input (from metafits)
    // This was derived from an example metafits: test_files/1101503312.metafits
    // by sorting by "Input" and then using get_mwax_order(antenna, pol)
    let mwax_order: Vec<usize> = vec![
        151, 150, 149, 148, 147, 146, 145, 144, 159, 158, 157, 156, 155, 154, 153, 152, 231, 230,
        229, 228, 227, 226, 225, 224, 239, 238, 237, 236, 235, 234, 233, 232, 247, 246, 245, 244,
        243, 242, 241, 240, 255, 254, 253, 252, 251, 250, 249, 248, 103, 102, 101, 100, 99, 98, 97,
        96, 111, 110, 109, 108, 107, 106, 105, 104, 23, 22, 21, 20, 19, 18, 17, 16, 31, 30, 29, 28,
        27, 26, 25, 24, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 135, 134, 133, 132,
        131, 130, 129, 128, 143, 142, 141, 140, 139, 138, 137, 136, 119, 118, 117, 116, 115, 114,
        113, 112, 127, 126, 125, 124, 123, 122, 121, 120, 183, 182, 181, 180, 179, 178, 177, 176,
        191, 190, 189, 188, 187, 186, 185, 184, 167, 166, 165, 164, 163, 162, 161, 160, 175, 174,
        173, 172, 171, 170, 169, 168, 215, 214, 213, 212, 211, 210, 209, 208, 223, 222, 221, 220,
        219, 218, 217, 216, 199, 198, 197, 196, 195, 194, 193, 192, 207, 206, 205, 204, 203, 202,
        201, 200, 39, 38, 37, 36, 35, 34, 33, 32, 47, 46, 45, 44, 43, 42, 41, 40, 55, 54, 53, 52,
        51, 50, 49, 48, 63, 62, 61, 60, 59, 58, 57, 56, 87, 86, 85, 84, 83, 82, 81, 80, 95, 94, 93,
        92, 91, 90, 89, 88, 71, 70, 69, 68, 67, 66, 65, 64, 79, 78, 77, 76, 75, 74, 73, 72,
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
fn test_conversion_of_legacy_hdu_to_mwax_baseline_ordervs_pyuvdata() {
    // Open a context and load in a test metafits and gpubox file
    let metafits = "test_files/1101503312_1_timestep/1101503312.metafits";
    let gpuboxfiles =
        vec!["test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits"];
    let context =
        CorrelatorContext::new(metafits, &gpuboxfiles).expect("Failed to create CorrelatorContext");

    // Read and convert first HDU
    let mwalib_hdu: Vec<f32> = context.read_by_baseline(0, 0).expect("Error!");

    // Check it
    // Vector is in:
    // [baseline][fine_chan][pol][r/i] order
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
    // row 0 is bl 0, freq 0
    // row 1 is bl 0, freq 1
    // ...
    // row 127 is bl 0, freq 127
    // row 128 is bl 1, freq 0
    // ...
    // each containing each containing 8 floats:
    // XX real, XX imag, XY real, XY imag, YX real, YX imag, YY real, YY imag
    //
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_path(
            "test_files/1101503312_1_timestep/1101503312_gpubox01_pyuvdata_1st_timestep_by_bl.csv",
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

            // pyuvdata data needs to be conjugated because we operate in opposite triangles.
            let pyuvdata_value = if i % 2 != 0 {
                // conjugate since it is an imaginary value
                -*v as f64
            } else {
                // real value remains unchanged
                *v as f64
            };

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
            let good = u8::from(approx_eq!(
                f64,
                mwalib_sum_of_baseline,
                pyuvdata_sum_of_baseline,
                F64Margin::default()
            ));

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

#[test]
fn test_conversion_of_legacy_hdu_to_mwax_frequency_order_vs_pyuvdata() {
    // Open a context and load in a test metafits and gpubox file
    let metafits = "test_files/1101503312_1_timestep/1101503312.metafits";
    let gpuboxfiles =
        vec!["test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits"];
    let context =
        CorrelatorContext::new(metafits, &gpuboxfiles).expect("Failed to create mwalibContext");

    // Read and convert first HDU
    let mwalib_hdu: Vec<f32> = context.read_by_frequency(0, 0).expect("Error!");

    // Check it
    // Vector is in:
    // [fine_chan][baseline][pol][r/i] order
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
    // row 0 is freq 0, bl 0
    // row 1 is freq 0, bl 1
    // ...
    // row 8255 is freq 0, bl8255
    // row 8256 is freq 1, bl0
    // ...
    //
    // each containing each containing 8 floats:
    // XX real, XX imag, XY real, XY imag, YX real, YX imag, YY real, YY imag
    //
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_path(
            "test_files/1101503312_1_timestep/1101503312_gpubox01_pyuvdata_1st_timestep_by_freq.csv",
        )
        .expect("Failed to read CSV");

    let mut baseline = 0;
    let mut fine_chan = 0;
    let mut mwalib_sum_of_fine_chan: f64 = 0.;
    let mut pyuvdata_sum_of_fine_chan: f64 = 0.;
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

        // Ensure channel is <= num_fine_chans
        assert!(fine_chan <= context.metafits_context.num_corr_fine_chans_per_coarse);

        let record: Vec<f32> = result.expect("Failed to deserialize CSV");
        assert!(
            row_index < 1_056_768,
            "row_index is out of bounds {}",
            row_index
        );

        // Determine where we should be in the mwalib array
        let mwalib_ch_index = fine_chan * (8256 * 8);
        let mwalib_bl_index = baseline * 8;
        // Now loop though all the columns in this row
        for (i, v) in record.iter().enumerate() {
            assert!(i < 8);
            let mwalib_dest_index = mwalib_ch_index + mwalib_bl_index + i;

            let mwalib_value = mwalib_hdu[mwalib_dest_index] as f64;

            // pyuvdata data needs to be conjugated because we operate in opposite triangles.
            let pyuvdata_value = if i % 2 != 0 {
                // conjugate since it is an imaginary value
                -*v as f64
            } else {
                // real value remains unchanged
                *v as f64
            };

            mwalib_sum_of_fine_chan += mwalib_value;
            pyuvdata_sum_of_fine_chan += pyuvdata_value;

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

            assert!(approx_eq!(f64, mwalib_value, pyuvdata_value, F64Margin::default()), "mwalib_ch_index: {} mwalib_bl_index: {} fine_chan: {} baseline: {} ant1: {} v ant2: {} fine_chan: {} pol: {} mwalib_value: {} != pyuvdata_value: {} difference: {}", mwalib_ch_index, mwalib_bl_index, fine_chan, baseline, ant1, ant2, fine_chan, pol, mwalib_value, pyuvdata_value, mwalib_value - pyuvdata_value);
        }

        if baseline < 8255 {
            baseline += 1;

            if ant2 < 127 {
                ant2 += 1;
            } else {
                ant1 += 1;
                ant2 = ant1;
            }
        } else {
            // We are at the end of a fine channel
            // Get value from mwa_lib
            let good = u8::from(approx_eq!(
                f64,
                mwalib_sum_of_fine_chan,
                pyuvdata_sum_of_fine_chan,
                F64Margin::default()
            ));

            assert_eq!(
                good,
                1,
                "fine_chan: {} baseline: {} ant1: {} v ant2: {} mwalib_sum: {} != pyuvdata_sum: {} difference: {}",
                fine_chan,
                baseline,
                ant1,
                ant2,
                mwalib_sum_of_fine_chan,
                pyuvdata_sum_of_fine_chan,
                mwalib_sum_of_fine_chan - pyuvdata_sum_of_fine_chan
            );

            // Reset our sums
            mwalib_sum_of_fine_chan = 0.;
            pyuvdata_sum_of_fine_chan = 0.;

            // Reset counters
            baseline = 0;
            ant1 = 0;
            ant2 = 0;
            fine_chan += 1;
        }
    }
}

#[test]
fn test_conversion_of_legacy_hdu_to_mwax_baseline_order_vs_cotter() {
    // Open a context and load in a test metafits and gpubox file
    let metafits = "test_files/1101503312_1_timestep/1101503312.metafits";
    let gpuboxfiles =
        vec!["test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits"];
    let context =
        CorrelatorContext::new(metafits, &gpuboxfiles).expect("Failed to create CorrelatorContext");

    // Read and convert first HDU
    let mwalib_hdu: Vec<f32> = context.read_by_baseline(0, 0).expect("Error!");

    // Check it
    // Vector is in:
    // [baseline][fine_chan][pol][r/i] order
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
    // row 0 is bl 0, freq 0
    // row 1 is bl 0, freq 1
    // ...
    // row 127 is bl 0, freq 127
    // row 128 is bl 1, freq 0
    // ...
    // each containing each containing 8 floats:
    // XX real, XX imag, XY real, XY imag, YX real, YX imag, YY real, YY imag
    //
    // In this case, the CSV was generated by the python script in tools/comparison_tools/create_comparison_csvs.py
    // Cotter 4.5 (with cable delays commented out) was run with the following command line options to just create
    // a CASA measurement set of the visibilities with no corrections:
    //
    // $ cotter -nostats -noantennapruning -noflagautos -noflagdcchannels -norfi -nogeom -nosbgains -edgewidth 0 \
    //  -initflag 0 -sbpassband /path/to/sbpassbandfiles/10khz.txt -m /path/to/metafits.metafits \
    //  -o /path/to/output.ms /path/to/gpubox_files/*gpubox*.fits
    //
    // The create_comparison_csvs.py then dumps this out to a CSV file and it is this we have placed in the test_data dir.
    //
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_path(
            "test_files/1101503312_1_timestep/1101503312_gpubox01_cotter_1st_timestep_by_bl.csv",
        )
        .expect("Failed to read CSV");

    let mut baseline = 0;
    let mut fine_chan = 0;
    let mut mwalib_sum_of_baseline: f64 = 0.;
    let mut cotter_sum_of_baseline: f64 = 0.;
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

            // Cotter always sets a value of 0 in the real and imaginary values for the XY pol in an autocorrelation
            // mwalib *does* provide the value, so we will alter the test value provided by mwalib to reflect that
            let mwalib_value = if ant1 == ant2 && (i == 2 || i == 3) {
                0.
            } else {
                // Also cotter differs from mwalib in the following way:
                // Where ant1==ant2 we are conjugated with respect to cotter, so we must negate the imag values
                if ant1 == ant2 && i % 2 != 0 {
                    -mwalib_hdu[mwalib_dest_index] as f64
                } else {
                    mwalib_hdu[mwalib_dest_index] as f64
                }
            };

            let cotter_value = *v as f64;

            mwalib_sum_of_baseline += mwalib_value;
            cotter_sum_of_baseline += cotter_value;

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

            assert!(approx_eq!(f64, mwalib_value, cotter_value, F64Margin::default()), "baseline: {} ant1: {} v ant2: {} fine_chan: {} pol: {} mwalib_value: {} != cotter_value: {} difference: {}", baseline, ant1, ant2, fine_chan, pol, mwalib_value, cotter_value, mwalib_value - cotter_value);
        }

        if fine_chan < 127 {
            fine_chan += 1;
        } else {
            // We are at the end of a baseline
            // Get value from mwa_lib
            let good = u8::from(approx_eq!(
                f64,
                mwalib_sum_of_baseline,
                cotter_sum_of_baseline,
                F64Margin::default()
            ));

            assert_eq!(
                good,
                1,
                "baseline: {} ant1: {} v ant2: {} mwalib_sum: {} != cotter_sum: {} difference: {}",
                baseline,
                ant1,
                ant2,
                mwalib_sum_of_baseline,
                cotter_sum_of_baseline,
                mwalib_sum_of_baseline - cotter_sum_of_baseline
            );

            // Reset our sums
            mwalib_sum_of_baseline = 0.;
            cotter_sum_of_baseline = 0.;

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

#[test]
fn test_mwax_conversion_to_frequency_order() {
    // Open the test mwax file
    // a) directly using Fits  (data will be ordered [baseline][freq][pol][r][i])
    // b) using mwalib (by freq) (data will be ordered [freq][baseline][pol][r][i])
    // Then check b) is the same as a) modulo the order
    let mwax_metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
    let mwax_filename = "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

    //
    // Read the mwax file using FITS
    //
    let mut fptr = fits_open!(&mwax_filename).unwrap();
    let fits_hdu = fits_open_hdu!(&mut fptr, 1).unwrap();

    // Read data from fits hdu into vector
    let fits_hdu_data: Vec<f32> = get_fits_image!(&mut fptr, &fits_hdu).unwrap();

    //
    // Read the mwax file by frequency using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![mwax_filename];
    let context = CorrelatorContext::new(mwax_metafits_filename, &gpuboxfiles)
        .expect("Failed to create mwalibContext");

    // Read and convert first HDU
    let mwalib_hdu_data: Vec<f32> = context.read_by_frequency(0, 10).expect("Error!");

    // First assert that the data vectors are the same size
    assert_eq!(fits_hdu_data.len(), mwalib_hdu_data.len());

    let num_floats_per_baseline_fine_chan = context.metafits_context.num_visibility_pols * 2; // xx_r, xx_i, xy_r, ...

    // We will walk through the visibilities and compare them
    for b in 0..context.metafits_context.num_baselines {
        for f in 0..context.metafits_context.num_corr_fine_chans_per_coarse {
            // At this point we have 1 baseline and 1 fine channel which == (num_floats_per_baseline_fine_chan)
            // locate this block of data in both hdus
            let fits_index = (b
                * (context.metafits_context.num_corr_fine_chans_per_coarse
                    * num_floats_per_baseline_fine_chan))
                + (f * num_floats_per_baseline_fine_chan);
            let mwalib_index = (f
                * (context.metafits_context.num_baselines * num_floats_per_baseline_fine_chan))
                + (b * num_floats_per_baseline_fine_chan);

            // Check this block of floats matches
            assert_eq!(
                fits_hdu_data[fits_index..fits_index + num_floats_per_baseline_fine_chan],
                mwalib_hdu_data[mwalib_index..mwalib_index + num_floats_per_baseline_fine_chan]
            );
        }
    }
}

//
// Reference tests for the converters
//
// These compare every element of the output of each converter to a simple, obviously correct reference
// implementation, using an input where every float is unique. So any mistake in where a value ends up (not just
// in the sum of all values) is caught. They are here to make it safe to change how the converters are written.
//

const REFERENCE_LEGACY_METAFITS: &str = "test_files/1101503312_1_timestep/1101503312.metafits";
const REFERENCE_LEGACY_GPUBOX: &str =
    "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";
const REFERENCE_MWAX_METAFITS: &str = "test_files/1244973688_1_timestep/1244973688.metafits";
const REFERENCE_MWAX_GPUBOX: &str =
    "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

/// Floats per baseline per fine channel: xx_r,xx_i,xy_r,xy_i,yx_r,yx_i,yy_r,yy_i
const REFERENCE_FLOATS_PER_BASELINE_FINE_CHAN: usize = 8;

/// Returns a vector of `len` floats where the value of each float is its index. f32 holds integers exactly up to
/// 16,777,216, and the biggest input we use is 8256 * 128 * 8 = 8,454,144 floats, so every value is unique and exact.
fn unique_input(len: usize) -> Vec<f32> {
    (0..len).map(|i| i as f32).collect()
}

/// Reference implementation of the legacy conversion. Legacy input is [fine_chan][baseline][pol][r][i] but
/// where each baseline's pols are found, and if they need conjugating, comes from the conversion table.
/// Output is [baseline][fine_chan][pol][r][i], or [fine_chan][baseline][pol][r][i] if `by_frequency`.
fn reference_convert_legacy(
    table: &[LegacyConversionBaseline],
    input: &[f32],
    num_fine_chans: usize,
    by_frequency: bool,
) -> Vec<f32> {
    let num_baselines = table.len();
    let floats_per_fine_chan = num_baselines * REFERENCE_FLOATS_PER_BASELINE_FINE_CHAN;
    let mut output = vec![f32::NAN; num_fine_chans * floats_per_fine_chan];

    for fine_chan in 0..num_fine_chans {
        for (baseline, entry) in table.iter().enumerate() {
            let source_start = fine_chan * floats_per_fine_chan;
            let destination_start = if by_frequency {
                fine_chan * floats_per_fine_chan
                    + baseline * REFERENCE_FLOATS_PER_BASELINE_FINE_CHAN
            } else {
                baseline * num_fine_chans * REFERENCE_FLOATS_PER_BASELINE_FINE_CHAN
                    + fine_chan * REFERENCE_FLOATS_PER_BASELINE_FINE_CHAN
            };

            // (index of the real part in the input, whether the conversion table says to conjugate it)
            let pols = [
                (entry.xx_index, entry.xx_conjugate),
                (entry.xy_index, entry.xy_conjugate),
                (entry.yx_index, entry.yx_conjugate),
                (entry.yy_index, entry.yy_conjugate),
            ];

            for (pol, (index, conjugate)) in pols.iter().enumerate() {
                let real = input[source_start + index];
                let imag = input[source_start + index + 1];

                output[destination_start + pol * 2] = real;
                // The converters finish by taking the conjugate of everything (to get to the correct
                // triangle), so a visibility the table says to conjugate ends up with its imaginary part
                // unchanged and all others end up negated
                output[destination_start + pol * 2 + 1] = if *conjugate { imag } else { -imag };
            }
        }
    }

    output
}

/// Make a legacy context, and return the conversion table and the number of fine channels
fn get_legacy_conversion_table() -> (Vec<LegacyConversionBaseline>, usize) {
    let context = CorrelatorContext::new(REFERENCE_LEGACY_METAFITS, &[REFERENCE_LEGACY_GPUBOX])
        .expect("Failed to create CorrelatorContext");

    (
        context.legacy_conversion_table.clone(),
        context.metafits_context.num_corr_fine_chans_per_coarse,
    )
}

#[test]
fn test_reference_convert_legacy_hdu_to_mwax_baseline_order() {
    let (table, num_fine_chans) = get_legacy_conversion_table();
    assert_eq!(table.len(), 8256);

    let input =
        unique_input(num_fine_chans * table.len() * REFERENCE_FLOATS_PER_BASELINE_FINE_CHAN);
    let mut output = vec![f32::NAN; input.len()];

    convert_legacy_hdu_to_mwax_baseline_order(&table, &input, &mut output, num_fine_chans);

    let expected = reference_convert_legacy(&table, &input, num_fine_chans, false);

    assert_eq!(output.len(), expected.len());
    // Compare by value, so a zero does not need to have the same sign
    assert!(output.iter().zip(expected.iter()).all(|(a, b)| a == b));
}

#[test]
fn test_reference_convert_legacy_hdu_to_mwax_frequency_order() {
    let (table, num_fine_chans) = get_legacy_conversion_table();

    let input =
        unique_input(num_fine_chans * table.len() * REFERENCE_FLOATS_PER_BASELINE_FINE_CHAN);
    let mut output = vec![f32::NAN; input.len()];

    convert_legacy_hdu_to_mwax_frequency_order(&table, &input, &mut output, num_fine_chans);

    let expected = reference_convert_legacy(&table, &input, num_fine_chans, true);

    assert_eq!(output.len(), expected.len());
    assert!(output.iter().zip(expected.iter()).all(|(a, b)| a == b));
}

#[test]
fn test_reference_convert_legacy_baseline_and_frequency_order_are_transposes() {
    let (table, num_fine_chans) = get_legacy_conversion_table();
    let num_baselines = table.len();

    let input =
        unique_input(num_fine_chans * num_baselines * REFERENCE_FLOATS_PER_BASELINE_FINE_CHAN);
    let mut by_baseline = vec![f32::NAN; input.len()];
    let mut by_frequency = vec![f32::NAN; input.len()];

    convert_legacy_hdu_to_mwax_baseline_order(&table, &input, &mut by_baseline, num_fine_chans);
    convert_legacy_hdu_to_mwax_frequency_order(&table, &input, &mut by_frequency, num_fine_chans);

    for baseline in 0..num_baselines {
        for fine_chan in 0..num_fine_chans {
            let a =
                (baseline * num_fine_chans + fine_chan) * REFERENCE_FLOATS_PER_BASELINE_FINE_CHAN;
            let b =
                (fine_chan * num_baselines + baseline) * REFERENCE_FLOATS_PER_BASELINE_FINE_CHAN;

            assert_eq!(
                by_baseline[a..a + REFERENCE_FLOATS_PER_BASELINE_FINE_CHAN],
                by_frequency[b..b + REFERENCE_FLOATS_PER_BASELINE_FINE_CHAN]
            );
        }
    }
}

#[test]
fn test_reference_convert_legacy_only_writes_the_expected_part_of_a_bigger_output() {
    let (table, num_fine_chans) = get_legacy_conversion_table();

    let input_len = num_fine_chans * table.len() * REFERENCE_FLOATS_PER_BASELINE_FINE_CHAN;
    let input = unique_input(input_len);

    // The output has 10 extra floats at the end, which must not be touched
    let mut output = vec![-1234.5; input_len + 10];

    convert_legacy_hdu_to_mwax_baseline_order(&table, &input, &mut output, num_fine_chans);
    assert!(output[input_len..].iter().all(|x| *x == -1234.5));

    let mut output = vec![-1234.5; input_len + 10];

    convert_legacy_hdu_to_mwax_frequency_order(&table, &input, &mut output, num_fine_chans);
    assert!(output[input_len..].iter().all(|x| *x == -1234.5));
}

#[test]
fn test_reference_convert_mwax_hdu_to_frequency_order() {
    let context = CorrelatorContext::new(REFERENCE_MWAX_METAFITS, &[REFERENCE_MWAX_GPUBOX])
        .expect("Failed to create CorrelatorContext");
    let num_baselines = context.metafits_context.num_baselines;
    let num_fine_chans = context.metafits_context.num_corr_fine_chans_per_coarse;
    let num_pols = context.metafits_context.num_visibility_pols;
    let floats_per_baseline_fine_chan = num_pols * 2;

    let len = num_baselines * num_fine_chans * floats_per_baseline_fine_chan;
    let input = unique_input(len);
    let mut output = vec![f32::NAN; len];

    convert_mwax_hdu_to_frequency_order(
        &input,
        &mut output,
        num_baselines,
        num_fine_chans,
        num_pols,
    );

    // MWAX input is [baseline][fine_chan][pol][r][i], output is [fine_chan][baseline][pol][r][i]
    for baseline in 0..num_baselines {
        for fine_chan in 0..num_fine_chans {
            let source = (baseline * num_fine_chans + fine_chan) * floats_per_baseline_fine_chan;
            let destination =
                (fine_chan * num_baselines + baseline) * floats_per_baseline_fine_chan;

            assert_eq!(
                input[source..source + floats_per_baseline_fine_chan],
                output[destination..destination + floats_per_baseline_fine_chan]
            );
        }
    }
}

#[test]
fn test_reference_convert_mwax_hdu_to_frequency_order_only_writes_the_expected_part_of_a_bigger_output(
) {
    let num_baselines = 10;
    let num_fine_chans = 4;
    let num_pols = 4;
    let len = num_baselines * num_fine_chans * num_pols * 2;

    let input = unique_input(len);
    let mut output = vec![-1234.5; len + 10];

    convert_mwax_hdu_to_frequency_order(
        &input,
        &mut output,
        num_baselines,
        num_fine_chans,
        num_pols,
    );

    assert!(output[len..].iter().all(|x| *x == -1234.5));
    assert!(output[..len].iter().all(|x| *x != -1234.5));
}
