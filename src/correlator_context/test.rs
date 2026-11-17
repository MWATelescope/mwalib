// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Unit tests for correlator context
use super::*;
use float_cmp::*;
use std::path::PathBuf;

#[test]
fn test_context_new_with_only_non000_batch() {
    // The _001 fits file contains 1 timestep too- it is at time index 1560938480 / marker 10 (so the 10th timestep - zero indexed!)
    let metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
    let gpuboxfiles =
        vec!["test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_001.fits"];

    // No gpubox files provided
    let context_result = CorrelatorContext::new(metafits_filename, &gpuboxfiles);

    assert!(context_result.is_ok());

    let context = context_result.unwrap();
    println!("{:?}", context.gpubox_time_map);
    let data_result = context.read_by_baseline(10, 10);
    assert!(data_result.is_ok(), "Error was {:?}", data_result);
}

#[test]
fn test_context_new_with_000_and_001_batch() {
    // The _001 fits file contains 1 timestep too- it is at time index 1560938480 / marker 10 (so the 10th timestep - zero indexed!)
    let metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
    let gpuboxfiles = vec![
        "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits",
        "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_001.fits",
    ];

    // No gpubox files provided
    let context_result = CorrelatorContext::new(metafits_filename, &gpuboxfiles);

    assert!(context_result.is_ok());

    let context = context_result.unwrap();
    println!("{:?}", context.gpubox_time_map);
    // Read from batch 0
    let data_result1 = context.read_by_baseline(0, 10);
    assert!(data_result1.is_ok(), "Error was {:?}", data_result1);
    // Read from batch 1
    let data_result2 = context.read_by_baseline(10, 10);
    assert!(data_result2.is_ok(), "Error was {:?}", data_result2);
    // Read from a timestep that does not exist
    let data_result3 = context.read_by_baseline(15, 10);
    assert!(matches!(
        data_result3.unwrap_err(),
        GpuboxError::NoDataForTimeStepCoarseChannel {
            timestep_index: _,
            coarse_chan_index: _
        }
    ));
}

#[test]
fn test_context_new_missing_gpubox_files() {
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let gpuboxfiles: Vec<PathBuf> = Vec::new();

    // No gpubox files provided
    let context = CorrelatorContext::new(metafits_filename, &gpuboxfiles);
    assert!(matches!(
        context.unwrap_err(),
        MwalibError::Gpubox(GpuboxError::NoGpuboxes)
    ));
}

#[test]
fn test_context_new_invalid_metafits() {
    let metafits_filename = "invalid.metafits";
    let filename = "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";
    let gpuboxfiles = vec![filename];

    // No gpubox files provided
    let context = CorrelatorContext::new(metafits_filename, &gpuboxfiles);

    assert!(context.is_err());
}

#[test]
fn test_context_legacy_v1() {
    // Open the test legacy file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let filename = "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![filename];
    let context = CorrelatorContext::new(metafits_filename, &gpuboxfiles)
        .expect("Failed to create mwalibContext");

    // Test the properties of the context object match what we expect
    // Correlator version:       v1 Legacy,
    assert_eq!(context.mwa_version, MWAVersion::CorrLegacy);

    // common UNIX start time:   1417468096,
    assert_eq!(context.common_start_unix_time_ms, 1_417_468_096_000);

    // common UNIX end time:     1417468098,
    assert_eq!(context.common_end_unix_time_ms, 1_417_468_098_000);

    // common duration:          2 s,
    assert_eq!(context.common_duration_ms, 2000);

    // common bandwidth:    1.28 MHz,
    assert_eq!(context.common_bandwidth_hz, 1_280_000);

    // num coarse channels,      1,
    assert_eq!(context.num_common_coarse_chans, 1);

    // common coarse chan indicies
    assert_eq!(context.common_coarse_chan_indices[0], 0);

    // common good timesteps
    assert_eq!(context.common_good_timestep_indices.len(), 0);

    // common good coarse chans
    assert_eq!(context.common_good_coarse_chan_indices.len(), 0);

    // provided timesteps
    assert_eq!(context.provided_timestep_indices.len(), 1);
    assert_eq!(context.provided_timestep_indices[0], 0);

    // provided coarse channels
    assert_eq!(context.provided_coarse_chan_indices.len(), 1);
    assert_eq!(context.provided_coarse_chan_indices[0], 0);

    // num timesteps:            1,
    assert_eq!(context.num_timesteps, 56);

    // timesteps:                [unix=1417468096.000],
    assert_eq!(context.timesteps[0].unix_time_ms, 1_417_468_096_000);

    // coarse channels:          [gpu=1 corr=0 rec=109 @ 139.520 MHz],
    assert_eq!(context.coarse_chans[0].corr_chan_number, 0);
    assert_eq!(context.coarse_chans[0].gpubox_number, 1);
    assert_eq!(context.coarse_chans[0].rec_chan_number, 109);
    assert_eq!(context.coarse_chans[0].chan_centre_hz, 139_520_000);

    // Check that antenna[0].tile and antenna[1].tile equal rf_input[0] & [1].tile and rf_input[2] & [3].tile respectively
    assert_eq!(
        context.metafits_context.antennas[0].tile_id,
        context.metafits_context.rf_inputs[0].tile_id
    );
    assert_eq!(
        context.metafits_context.antennas[0].tile_id,
        context.metafits_context.rf_inputs[1].tile_id
    );
    assert_eq!(
        context.metafits_context.antennas[1].tile_id,
        context.metafits_context.rf_inputs[2].tile_id
    );
    assert_eq!(
        context.metafits_context.antennas[1].tile_id,
        context.metafits_context.rf_inputs[3].tile_id
    );

    assert_eq!(context.metafits_context.metafits_coarse_chans.len(), 24);
    assert_eq!(context.metafits_context.metafits_timesteps.len(), 56);

    // Check for num of fine chan freqs
    assert_eq!(
        context.metafits_context.num_metafits_fine_chan_freqs,
        24 * 128
    );

    assert_eq!(
        context.metafits_context.num_metafits_fine_chan_freqs,
        context.metafits_context.metafits_fine_chan_freqs_hz.len()
    );

    assert_eq!(context.bscale, 0.5);
}

#[test]
fn test_context_mwax() {
    // Open the test mwax file
    let metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
    let filename = "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![filename];
    let context = CorrelatorContext::new(metafits_filename, &gpuboxfiles)
        .expect("Failed to create mwalibContext");

    // Test the properties of the context object match what we expect
    // Correlator version:       v2,
    assert_eq!(context.mwa_version, MWAVersion::CorrMWAXv2);

    // common UNIX start time:   1560938470,
    assert_eq!(context.common_start_unix_time_ms, 1_560_938_470_000);

    // common UNIX end time:     1560938471,
    assert_eq!(context.common_end_unix_time_ms, 1_560_938_471_000);

    // common duration:          1 s,
    assert_eq!(context.common_duration_ms, 1000);

    // observation bandwidth:    1.28 MHz,
    assert_eq!(context.common_bandwidth_hz, 1_280_000);

    // num common coarse channels,      1,
    assert_eq!(context.num_common_coarse_chans, 1);

    // common coarse chan indicies
    assert_eq!(context.common_coarse_chan_indices[0], 10);

    // num common timesteps,      1,
    assert_eq!(context.num_common_timesteps, 1);

    // common timestep indicies
    assert_eq!(context.common_timestep_indices[0], 0);

    // common good timesteps
    assert_eq!(context.common_good_timestep_indices.len(), 0);

    // common good coarse chans
    assert_eq!(context.common_good_coarse_chan_indices.len(), 0);

    // provided timesteps
    assert_eq!(context.provided_timestep_indices.len(), 1);
    assert_eq!(context.provided_timestep_indices[0], 0);

    // provided coarse channels
    assert_eq!(context.provided_coarse_chan_indices.len(), 1);
    assert_eq!(context.provided_coarse_chan_indices[0], 10);

    // num timesteps:            1,
    assert_eq!(context.num_timesteps, 120);

    // timesteps:                [unix=1560938470.000],
    assert_eq!(context.timesteps[0].unix_time_ms, 1_560_938_470_000);

    // coarse channels:          [gpu=114 corr=10 rec=114 @  MHz],
    assert_eq!(context.coarse_chans[10].corr_chan_number, 10);
    assert_eq!(context.coarse_chans[10].gpubox_number, 114);
    assert_eq!(context.coarse_chans[10].rec_chan_number, 114);
    assert_eq!(context.coarse_chans[10].chan_centre_hz, 145_920_000);

    // Check that antenna[0].tile and antenna[1].tile equal rf_input[0] & [1].tile and rf_input[2] & [3].tile respectively
    assert_eq!(
        context.metafits_context.antennas[0].tile_id,
        context.metafits_context.rf_inputs[0].tile_id
    );
    assert_eq!(
        context.metafits_context.antennas[0].tile_id,
        context.metafits_context.rf_inputs[1].tile_id
    );
    assert_eq!(
        context.metafits_context.antennas[1].tile_id,
        context.metafits_context.rf_inputs[2].tile_id
    );
    assert_eq!(
        context.metafits_context.antennas[1].tile_id,
        context.metafits_context.rf_inputs[3].tile_id
    );

    assert_eq!(context.metafits_context.metafits_coarse_chans.len(), 24);
    assert_eq!(context.metafits_context.metafits_timesteps.len(), 120);

    // weights is baselines x 4 visibility pols
    assert_eq!(
        context.num_timestep_coarse_chan_weight_floats,
        ((128 * 129) / 2) * 4
    );

    assert_eq!(context.bscale, 1.0);
}

#[test]
fn test_read_by_frequency_invalid_inputs() {
    let mwax_metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
    let mwax_filename = "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![mwax_filename];
    let context = CorrelatorContext::new(mwax_metafits_filename, &gpuboxfiles)
        .expect("Failed to create CorrelatorContext");

    // 99999 is invalid as a timestep for this observation
    let result_invalid_timestep = context.read_by_frequency(99999, 10);

    assert!(matches!(
        result_invalid_timestep.unwrap_err(),
        GpuboxError::InvalidTimeStepIndex(_)
    ));

    // 99999 is invalid as a coarse_chan for this observation
    let result_invalid_coarse_chan = context.read_by_frequency(0, 99999);

    assert!(matches!(
        result_invalid_coarse_chan.unwrap_err(),
        GpuboxError::InvalidCoarseChanIndex(_)
    ));

    // Do another test, this time with no data for the timestep/cc combo
    let result = context.read_by_frequency(10, 0);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(
            error,
            GpuboxError::NoDataForTimeStepCoarseChannel {
                timestep_index: _,
                coarse_chan_index: _
            }
        ),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_read_by_baseline_invalid_inputs() {
    let mwax_metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
    let mwax_filename = "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![mwax_filename];
    let context = CorrelatorContext::new(mwax_metafits_filename, &gpuboxfiles)
        .expect("Failed to create CorrelatorContext");

    // 99999 is invalid as a timestep for this observation
    let result_invalid_timestep = context.read_by_baseline(99999, 10);

    assert!(matches!(
        result_invalid_timestep.unwrap_err(),
        GpuboxError::InvalidTimeStepIndex(_)
    ));

    // 99999 is invalid as a coarse_chan for this observation
    let result_invalid_coarse_chan = context.read_by_baseline(0, 99999);

    assert!(matches!(
        result_invalid_coarse_chan.unwrap_err(),
        GpuboxError::InvalidCoarseChanIndex(_)
    ));

    // Do another test, this time with no data for the timestep/cc combo
    let result = context.read_by_baseline(10, 0);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(
            error,
            GpuboxError::NoDataForTimeStepCoarseChannel {
                timestep_index: _,
                coarse_chan_index: _
            }
        ),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_mwa_legacy_read() {
    // Open the test mwa file
    // a) using mwalib (by baseline) (data will be ordered [baseline][freq][pol][r][i])
    // b) using mwalib (by frequency) (data will be ordered [freq][baseline][pol][r][i])
    // Then check b) is the same as a) and
    let mwax_metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let mwax_filename =
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";

    //
    // Read the mwax file by frequency using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![mwax_filename];
    let context = CorrelatorContext::new(mwax_metafits_filename, &gpuboxfiles)
        .expect("Failed to create CorrelatorContext");

    // Read and convert first HDU by baseline
    let mwalib_hdu_data_by_bl: Vec<f32> = context.read_by_baseline(0, 0).expect("Error!");

    // Read and convert first HDU by frequency
    let mwalib_hdu_data_by_freq: Vec<f32> = context.read_by_frequency(0, 0).expect("Error!");

    let sum_freq: f64 = mwalib_hdu_data_by_freq
        .iter()
        .fold(0., |sum, x| sum + *x as f64);
    let sum_bl: f64 = mwalib_hdu_data_by_bl
        .iter()
        .fold(0., |sum, x| sum + *x as f64);

    // Check sums match
    assert!(approx_eq!(f64, sum_bl, sum_freq, F64Margin::default()));
}

#[test]
fn test_mwax_read() {
    // Open the test mwax file
    // a) directly using Fits  (data will be ordered [baseline][freq][pol][r][i])
    // b) using mwalib (by baseline) (data will be ordered the same as the raw fits file)
    // c) using mwalib (by frequency) (data will be ordered [freq][baseline][pol][r][i])
    // Then check b) is the same as a) and
    // that c) is the same size and sum as a) and b)
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
        .expect("Failed to create CorrelatorContext");

    // Read and convert first HDU by baseline
    let mwalib_hdu_data_by_bl: Vec<f32> = context.read_by_baseline(0, 10).expect("Error!");

    // Read and convert first HDU by frequency
    let mwalib_hdu_data_by_freq: Vec<f32> = context.read_by_frequency(0, 10).expect("Error!");

    // First assert that the data vectors are the same size
    assert_eq!(fits_hdu_data.len(), mwalib_hdu_data_by_bl.len());
    assert_eq!(fits_hdu_data.len(), mwalib_hdu_data_by_freq.len());

    // Check all 3 sum to the same value
    let sum_fits: f64 = fits_hdu_data.iter().fold(0., |sum, x| sum + *x as f64);
    let sum_freq: f64 = mwalib_hdu_data_by_freq
        .iter()
        .fold(0., |sum, x| sum + *x as f64);
    let sum_bl: f64 = mwalib_hdu_data_by_bl
        .iter()
        .fold(0., |sum, x| sum + *x as f64);

    // Check sums match
    assert_eq!(
        approx_eq!(f64, sum_fits, sum_freq, F64Margin::default()),
        approx_eq!(f64, sum_fits, sum_bl, F64Margin::default())
    );

    // Check this block of floats matches
    assert_eq!(fits_hdu_data, mwalib_hdu_data_by_bl);
}

#[test]
fn test_validate_first_hdu() {
    // Open the test mwax file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let filename = "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![filename];
    let context = CorrelatorContext::new(metafits_filename, &gpuboxfiles)
        .expect("Failed to create CorrelatorContext");

    let coarse_chan = context.coarse_chans[0].gpubox_number;
    let (batch_index, _) =
        context.gpubox_time_map[&context.timesteps[0].unix_time_ms][&coarse_chan];

    let mut fptr =
        fits_open!(&context.gpubox_batches[batch_index].gpubox_files[0].filename).unwrap();

    let result_valid = CorrelatorContext::validate_first_hdu(
        context.mwa_version,
        context.metafits_context.num_corr_fine_chans_per_coarse,
        context.metafits_context.num_baselines,
        context.metafits_context.num_visibility_pols,
        &mut fptr,
    );

    let result_invalid1 = CorrelatorContext::validate_first_hdu(
        context.mwa_version,
        context.metafits_context.num_corr_fine_chans_per_coarse + 1,
        context.metafits_context.num_baselines,
        context.metafits_context.num_visibility_pols,
        &mut fptr,
    );

    let result_invalid2 = CorrelatorContext::validate_first_hdu(
        context.mwa_version,
        context.metafits_context.num_corr_fine_chans_per_coarse,
        context.metafits_context.num_baselines + 1,
        context.metafits_context.num_visibility_pols,
        &mut fptr,
    );

    let result_invalid3 = CorrelatorContext::validate_first_hdu(
        context.mwa_version,
        context.metafits_context.num_corr_fine_chans_per_coarse,
        context.metafits_context.num_baselines,
        context.metafits_context.num_visibility_pols + 1,
        &mut fptr,
    );

    // This is valid
    assert!(result_valid.is_ok());

    assert!(result_invalid1.is_err());

    assert!(result_invalid2.is_err());

    assert!(result_invalid3.is_err());
}

#[test]
fn test_validate_hdu_axes_good() {
    let metafits_fine_chans_per_coarse = 128;
    let metafits_baselines = 8256;
    let visibility_pols = 4;
    let result_good1 = CorrelatorContext::validate_hdu_axes(
        MWAVersion::CorrOldLegacy,
        metafits_fine_chans_per_coarse,
        metafits_baselines,
        visibility_pols,
        8256 * 4 * 2,
        128,
    );

    assert!(result_good1.is_ok());

    let result_good2 = CorrelatorContext::validate_hdu_axes(
        MWAVersion::CorrLegacy,
        metafits_fine_chans_per_coarse,
        metafits_baselines,
        visibility_pols,
        8256 * 4 * 2,
        128,
    );

    assert!(result_good2.is_ok());

    let result_good3 = CorrelatorContext::validate_hdu_axes(
        MWAVersion::CorrMWAXv2,
        metafits_fine_chans_per_coarse,
        metafits_baselines,
        visibility_pols,
        128 * 4 * 2,
        8256,
    );

    assert!(result_good3.is_ok());
}

#[test]
fn test_validate_hdu_axes_naxis_mismatches_oldlegacy() {
    let metafits_fine_chans_per_coarse = 128;
    let metafits_baselines = 8256;
    let visibility_pols = 4;
    let values = 1;

    // Check for NAXIS1 mismatch
    let result_bad1 = CorrelatorContext::validate_hdu_axes(
        MWAVersion::CorrOldLegacy,
        metafits_fine_chans_per_coarse,
        metafits_baselines,
        visibility_pols,
        metafits_baselines * visibility_pols * values,
        128,
    );

    assert!(matches!(
        result_bad1.unwrap_err(),
        GpuboxError::LegacyNaxis1Mismatch {
            metafits_baselines: _,
            visibility_pols: _,
            naxis1: _,
            naxis2: _,
            calculated_naxis1: _
        }
    ));

    // Check for NAXIS2 mismatch
    let result_bad2 = CorrelatorContext::validate_hdu_axes(
        MWAVersion::CorrOldLegacy,
        metafits_fine_chans_per_coarse,
        metafits_baselines,
        visibility_pols,
        8256 * 4 * 2,
        129,
    );

    assert!(matches!(
        result_bad2.unwrap_err(),
        GpuboxError::LegacyNaxis2Mismatch {
            metafits_fine_chans_per_coarse: _,
            naxis2: _,
            calculated_naxis2: _
        }
    ));
}

#[test]
fn test_validate_hdu_axes_naxis_mismatches_legacy() {
    let metafits_fine_chans_per_coarse = 128;
    let metafits_baselines = 8256;
    let visibility_pols = 4;
    let values = 1;

    // Check for NAXIS1 mismatch
    let result_bad1 = CorrelatorContext::validate_hdu_axes(
        MWAVersion::CorrLegacy,
        metafits_fine_chans_per_coarse,
        metafits_baselines,
        visibility_pols,
        metafits_baselines * visibility_pols * values,
        128,
    );

    assert!(matches!(
        result_bad1.unwrap_err(),
        GpuboxError::LegacyNaxis1Mismatch {
            metafits_baselines: _,
            visibility_pols: _,
            naxis1: _,
            naxis2: _,
            calculated_naxis1: _
        }
    ));

    // Check for NAXIS2 mismatch
    let result_bad2 = CorrelatorContext::validate_hdu_axes(
        MWAVersion::CorrLegacy,
        metafits_fine_chans_per_coarse,
        metafits_baselines,
        visibility_pols,
        8256 * 4 * 2,
        129,
    );

    assert!(matches!(
        result_bad2.unwrap_err(),
        GpuboxError::LegacyNaxis2Mismatch {
            metafits_fine_chans_per_coarse: _,
            naxis2: _,
            calculated_naxis2: _
        }
    ));
}

#[test]
fn test_validate_hdu_axes_naxis_mismatches_v2() {
    let metafits_fine_chans_per_coarse = 128;
    let metafits_baselines = 8256;
    let visibility_pols = 4;
    let values = 2;

    // Check for NAXIS1 mismatch
    let result_bad1 = CorrelatorContext::validate_hdu_axes(
        MWAVersion::CorrMWAXv2,
        metafits_fine_chans_per_coarse,
        metafits_baselines,
        visibility_pols,
        metafits_fine_chans_per_coarse * visibility_pols,
        8256,
    );

    assert!(matches!(
        result_bad1.unwrap_err(),
        GpuboxError::MwaxNaxis1Mismatch {
            metafits_fine_chans_per_coarse: _,
            visibility_pols: _,
            naxis1: _,
            naxis2: _,
            calculated_naxis1: _
        }
    ));

    // Check for NAXIS2 mismatch
    let result_bad2 = CorrelatorContext::validate_hdu_axes(
        MWAVersion::CorrMWAXv2,
        metafits_fine_chans_per_coarse,
        metafits_baselines,
        visibility_pols,
        metafits_fine_chans_per_coarse * visibility_pols * values,
        8257,
    );

    assert!(matches!(
        result_bad2.unwrap_err(),
        GpuboxError::MwaxNaxis2Mismatch {
            metafits_baselines: _,
            naxis2: _,
            calculated_naxis2: _
        }
    ));
}

#[test]
fn test_read_by_baseline_into_buffer_mwax() {
    // Open the test mwa file
    // a) using mwalib (by baseline) (data will be ordered [baseline][freq][pol][r][i])
    // b) using mwalib (by frequency) (data will be ordered [freq][baseline][pol][r][i])
    // Then check b) is the same as a) and
    let mwax_metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
    let mwax_filename = "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

    //
    // Read the mwax file by frequency using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![mwax_filename];
    let context = CorrelatorContext::new(mwax_metafits_filename, &gpuboxfiles)
        .expect("Failed to create CorrelatorContext");

    // Read and convert first HDU by baseline
    let mwalib_hdu_data_by_bl: Vec<f32> = context.read_by_baseline(0, 10).expect("Error!");

    // Read and convert first HDU by frequency
    let mwalib_hdu_data_by_freq: Vec<f32> = context.read_by_frequency(0, 10).expect("Error!");

    // Read into buffer by baseline
    let mut mwalib_hdu_data_by_bl2: Vec<f32> = vec![0.; context.num_timestep_coarse_chan_floats];

    let result_read_bl_buffer =
        context.read_by_baseline_into_buffer(0, 10, &mut mwalib_hdu_data_by_bl2);

    assert!(result_read_bl_buffer.is_ok());

    let sum_freq: f64 = mwalib_hdu_data_by_freq
        .iter()
        .fold(0., |sum, x| sum + *x as f64);

    let sum_bl: f64 = mwalib_hdu_data_by_bl
        .iter()
        .fold(0., |sum, x| sum + *x as f64);

    let sum_bl2: f64 = mwalib_hdu_data_by_bl2
        .iter()
        .fold(0., |sum, x| sum + *x as f64);

    // Check sums are not 0
    assert!(!approx_eq!(f64, sum_freq, 0., F64Margin::default()));
    assert!(!approx_eq!(f64, sum_bl, 0., F64Margin::default()));
    assert!(!approx_eq!(f64, sum_bl2, 0., F64Margin::default()));

    // Check they all match each other
    assert!(approx_eq!(f64, sum_bl, sum_freq, F64Margin::default()));
    assert!(approx_eq!(f64, sum_bl, sum_bl2, F64Margin::default()));

    // Do another test, this time with no data for the timestep/cc combo
    let result = context.read_by_baseline_into_buffer(10, 0, &mut mwalib_hdu_data_by_bl2);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(
            error,
            GpuboxError::NoDataForTimeStepCoarseChannel {
                timestep_index: _,
                coarse_chan_index: _
            }
        ),
        "Error was {:?}",
        error
    );

    // Do another test, this time with extremely out of bound indices (timestep)
    let result = context.read_by_baseline_into_buffer(999, 10, &mut mwalib_hdu_data_by_bl2);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(error, GpuboxError::InvalidTimeStepIndex(119)),
        "Error was {:?}",
        error
    );

    // Do another test, this time with extremely out of bound indices (cc)
    let result = context.read_by_baseline_into_buffer(0, 999, &mut mwalib_hdu_data_by_bl2);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(error, GpuboxError::InvalidCoarseChanIndex(23)),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_read_by_frequency_into_buffer_mwax() {
    // Open the test mwa file
    // a) using mwalib (by baseline) (data will be ordered [baseline][freq][pol][r][i])
    // b) using mwalib (by frequency) (data will be ordered [freq][baseline][pol][r][i])
    // Then check b) is the same as a) and
    let mwax_metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
    let mwax_filename = "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

    //
    // Read the mwax file by frequency using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![mwax_filename];
    let context = CorrelatorContext::new(mwax_metafits_filename, &gpuboxfiles)
        .expect("Failed to create CorrelatorContext");

    // Read and convert first HDU by baseline
    let mwalib_hdu_data_by_bl: Vec<f32> = context.read_by_baseline(0, 10).expect("Error!");

    // Read and convert first HDU by frequency
    let mwalib_hdu_data_by_freq: Vec<f32> = context.read_by_frequency(0, 10).expect("Error!");

    // Read into buffer by baseline
    let mut mwalib_hdu_data_by_freq2: Vec<f32> = vec![0.; context.num_timestep_coarse_chan_floats];

    let result_read_bl_buffer =
        context.read_by_frequency_into_buffer(0, 10, &mut mwalib_hdu_data_by_freq2);

    assert!(result_read_bl_buffer.is_ok());

    let sum_bl: f64 = mwalib_hdu_data_by_bl
        .iter()
        .fold(0., |sum, x| sum + *x as f64);

    let sum_freq: f64 = mwalib_hdu_data_by_freq
        .iter()
        .fold(0., |sum, x| sum + *x as f64);

    let sum_freq2: f64 = mwalib_hdu_data_by_freq2
        .iter()
        .fold(0., |sum, x| sum + *x as f64);

    // Check sums are not 0
    assert!(!approx_eq!(f64, sum_bl, 0., F64Margin::default()));
    assert!(!approx_eq!(f64, sum_freq, 0., F64Margin::default()));
    assert!(!approx_eq!(f64, sum_freq2, 0., F64Margin::default()));

    // Check they all match each other
    assert!(approx_eq!(f64, sum_bl, sum_freq, F64Margin::default()));
    assert!(approx_eq!(f64, sum_freq, sum_freq2, F64Margin::default()));

    // Do another test, this time with no data for the timestep/cc combo
    let result = context.read_by_frequency_into_buffer(10, 0, &mut mwalib_hdu_data_by_freq2);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(
            error,
            GpuboxError::NoDataForTimeStepCoarseChannel {
                timestep_index: _,
                coarse_chan_index: _
            }
        ),
        "Error was {:?}",
        error
    );

    // Do another test, this time with extremely out of bound indices (timestep)
    let result = context.read_by_frequency_into_buffer(999, 10, &mut mwalib_hdu_data_by_freq2);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(error, GpuboxError::InvalidTimeStepIndex(119)),
        "Error was {:?}",
        error
    );

    // Do another test, this time with extremely out of bound indices (cc)
    let result = context.read_by_frequency_into_buffer(0, 999, &mut mwalib_hdu_data_by_freq2);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(error, GpuboxError::InvalidCoarseChanIndex(23)),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_read_by_baseline_into_buffer_legacy() {
    // Open the test mwa file
    // a) using mwalib (by baseline) (data will be ordered [baseline][freq][pol][r][i])
    // b) using mwalib (by frequency) (data will be ordered [freq][baseline][pol][r][i])
    // Then check b) is the same as a) and
    let mwax_metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let mwax_filename =
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";

    //
    // Read the legacy file by frequency using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![mwax_filename];
    let context = CorrelatorContext::new(mwax_metafits_filename, &gpuboxfiles)
        .expect("Failed to create CorrelatorContext");

    // Read and convert first HDU by baseline
    let mwalib_hdu_data_by_bl: Vec<f32> = context.read_by_baseline(0, 0).expect("Error!");

    // Read and convert first HDU by frequency
    let mwalib_hdu_data_by_freq: Vec<f32> = context.read_by_frequency(0, 0).expect("Error!");

    // Read into buffer by baseline
    let mut mwalib_hdu_data_by_bl2: Vec<f32> = vec![0.; context.num_timestep_coarse_chan_floats];

    let result_read_bl_buffer =
        context.read_by_baseline_into_buffer(0, 0, &mut mwalib_hdu_data_by_bl2);

    assert!(result_read_bl_buffer.is_ok());

    let sum_freq: f64 = mwalib_hdu_data_by_freq
        .iter()
        .fold(0., |sum, x| sum + *x as f64);

    let sum_bl: f64 = mwalib_hdu_data_by_bl
        .iter()
        .fold(0., |sum, x| sum + *x as f64);

    let sum_bl2: f64 = mwalib_hdu_data_by_bl2
        .iter()
        .fold(0., |sum, x| sum + *x as f64);

    // Check sums are not 0
    assert!(!approx_eq!(f64, sum_freq, 0., F64Margin::default()));
    assert!(!approx_eq!(f64, sum_bl, 0., F64Margin::default()));
    assert!(!approx_eq!(f64, sum_bl2, 0., F64Margin::default()));

    // Check they all match each other
    assert!(approx_eq!(f64, sum_bl, sum_freq, F64Margin::default()));
    assert!(approx_eq!(f64, sum_bl, sum_bl2, F64Margin::default()));

    // Do another test, this time with no data for the timestep/cc combo
    let result = context.read_by_baseline_into_buffer(10, 10, &mut mwalib_hdu_data_by_bl2);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(
            error,
            GpuboxError::NoDataForTimeStepCoarseChannel {
                timestep_index: _,
                coarse_chan_index: _
            }
        ),
        "Error was {:?}",
        error
    );

    // Do another test, this time with extremely out of bound indices (timestep)
    let result = context.read_by_baseline_into_buffer(999, 0, &mut mwalib_hdu_data_by_bl2);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(error, GpuboxError::InvalidTimeStepIndex(55)),
        "Error was {:?}",
        error
    );

    // Do another test, this time with extremely out of bound indices (cc)
    let result = context.read_by_baseline_into_buffer(0, 999, &mut mwalib_hdu_data_by_bl2);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(error, GpuboxError::InvalidCoarseChanIndex(23)),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_read_by_frequency_into_buffer_legacy() {
    // Open the test mwa file
    // a) using mwalib (by baseline) (data will be ordered [baseline][freq][pol][r][i])
    // b) using mwalib (by frequency) (data will be ordered [freq][baseline][pol][r][i])
    // Then check b) is the same as a) and
    let mwax_metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let mwax_filename =
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";

    //
    // Read the legacy file by frequency using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![mwax_filename];
    let context = CorrelatorContext::new(mwax_metafits_filename, &gpuboxfiles)
        .expect("Failed to create CorrelatorContext");

    // Read and convert first HDU by baseline
    let mwalib_hdu_data_by_bl: Vec<f32> = context.read_by_baseline(0, 0).expect("Error!");

    // Read and convert first HDU by frequency
    let mwalib_hdu_data_by_freq: Vec<f32> = context.read_by_frequency(0, 0).expect("Error!");

    // Read into buffer by baseline
    let mut mwalib_hdu_data_by_freq2: Vec<f32> = vec![0.; context.num_timestep_coarse_chan_floats];

    let result_read_bl_buffer =
        context.read_by_frequency_into_buffer(0, 0, &mut mwalib_hdu_data_by_freq2);

    assert!(result_read_bl_buffer.is_ok());

    let sum_bl: f64 = mwalib_hdu_data_by_bl
        .iter()
        .fold(0., |sum, x| sum + *x as f64);

    let sum_freq: f64 = mwalib_hdu_data_by_freq
        .iter()
        .fold(0., |sum, x| sum + *x as f64);

    let sum_freq2: f64 = mwalib_hdu_data_by_freq2
        .iter()
        .fold(0., |sum, x| sum + *x as f64);

    // Check sums are not 0
    assert!(!approx_eq!(f64, sum_bl, 0., F64Margin::default()));
    assert!(!approx_eq!(f64, sum_freq, 0., F64Margin::default()));
    assert!(!approx_eq!(f64, sum_freq2, 0., F64Margin::default()));

    // Check they all match each other
    assert!(approx_eq!(f64, sum_bl, sum_freq, F64Margin::default()));
    assert!(approx_eq!(f64, sum_freq, sum_freq2, F64Margin::default()));

    // Do another test, this time with no data for the timestep/cc combo
    let result = context.read_by_frequency_into_buffer(10, 10, &mut mwalib_hdu_data_by_freq2);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(
            error,
            GpuboxError::NoDataForTimeStepCoarseChannel {
                timestep_index: _,
                coarse_chan_index: _
            }
        ),
        "Error was {:?}",
        error
    );

    // Do another test, this time with extremely out of bound indices (timestep)
    let result = context.read_by_frequency_into_buffer(999, 0, &mut mwalib_hdu_data_by_freq2);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(error, GpuboxError::InvalidTimeStepIndex(55)),
        "Error was {:?}",
        error
    );

    // Do another test, this time with extremely out of bound indices (cc)
    let result = context.read_by_frequency_into_buffer(0, 999, &mut mwalib_hdu_data_by_freq2);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(error, GpuboxError::InvalidCoarseChanIndex(23)),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_get_fits_filename_and_batch_and_hdu() {
    let mwax_metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let mwax_filename =
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";

    //
    // Read the legacy file by frequency using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![mwax_filename];
    let context = CorrelatorContext::new(mwax_metafits_filename, &gpuboxfiles)
        .expect("Failed to create CorrelatorContext");

    // Valid
    let result = context.get_fits_filename_and_batch_and_hdu(0, 0);
    assert!(result.is_ok());

    // Invalid - out of bounds completely (time)
    let result = context.get_fits_filename_and_batch_and_hdu(999, 0);
    let error = result.unwrap_err();
    assert!(
        matches!(error, GpuboxError::InvalidTimeStepIndex(55)),
        "Error was {:?}",
        error
    );

    // Invalid - out of bounds completely (channel)
    let result = context.get_fits_filename_and_batch_and_hdu(0, 999);
    let error = result.unwrap_err();
    assert!(
        matches!(error, GpuboxError::InvalidCoarseChanIndex(23)),
        "Error was {:?}",
        error
    );

    // Invalid - out of bounds no data (time)
    let result = context.get_fits_filename_and_batch_and_hdu(10, 0);
    let error = result.unwrap_err();
    assert!(
        matches!(
            error,
            GpuboxError::NoDataForTimeStepCoarseChannel {
                timestep_index: _,
                coarse_chan_index: _
            }
        ),
        "Error was {:?}",
        error
    );

    // Invalid - out of bounds no data (channel)
    let result = context.get_fits_filename_and_batch_and_hdu(0, 10);
    let error = result.unwrap_err();
    assert!(
        matches!(
            error,
            GpuboxError::NoDataForTimeStepCoarseChannel {
                timestep_index: _,
                coarse_chan_index: _
            }
        ),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_context_legacy_v1_get_fine_chan_feqs_one_coarse_chan() {
    // Open the test legacy file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let filename = "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![filename];
    let context = CorrelatorContext::new(metafits_filename, &gpuboxfiles)
        .expect("Failed to create mwalibContext");

    // Get fine channel freqs
    let coarse_channels: Vec<usize> = vec![0];
    let fine_chan_freqs: Vec<f64> = context
        .get_fine_chan_freqs_hz_array(&coarse_channels)
        .unwrap();

    assert_eq!(fine_chan_freqs.len(), 128);
    assert!(approx_eq!(
        f64,
        fine_chan_freqs[0],
        138_880_000.0,
        F64Margin::default()
    ));
}

#[test]
fn test_context_legacy_v1_get_fine_chan_feqs_some_coarse_chans() {
    // Open the test legacy file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let filename = "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![filename];
    let context = CorrelatorContext::new(metafits_filename, &gpuboxfiles)
        .expect("Failed to create mwalibContext");

    // Get fine channel freqs
    let coarse_channels: Vec<usize> = vec![10, 20];
    let fine_chan_freqs: Vec<f64> = context
        .get_fine_chan_freqs_hz_array(&coarse_channels)
        .unwrap();

    assert_eq!(fine_chan_freqs.len(), 256);
    assert!(approx_eq!(
        f64,
        fine_chan_freqs[0],
        151_680_000.0,
        F64Margin::default()
    ));

    assert!(
        approx_eq!(
            f64,
            fine_chan_freqs[128],
            164_480_000.0,
            F64Margin::default()
        ),
        "calc value: {}",
        fine_chan_freqs[128]
    );
}

#[test]
fn test_read_weights_by_baseline_invalid_inputs() {
    let mwax_metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
    let mwax_filename = "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![mwax_filename];
    let context = CorrelatorContext::new(mwax_metafits_filename, &gpuboxfiles)
        .expect("Failed to create CorrelatorContext");

    // 99999 is invalid as a timestep for this observation
    let result_invalid_timestep = context.read_by_baseline(99999, 10);

    assert!(matches!(
        result_invalid_timestep.unwrap_err(),
        GpuboxError::InvalidTimeStepIndex(_)
    ));

    // 99999 is invalid as a coarse_chan for this observation
    let result_invalid_coarse_chan = context.read_by_baseline(0, 99999);

    assert!(matches!(
        result_invalid_coarse_chan.unwrap_err(),
        GpuboxError::InvalidCoarseChanIndex(_)
    ));

    // Do another test, this time with no data for the timestep/cc combo
    let result = context.read_weights_by_baseline(10, 0);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(
        matches!(
            error,
            GpuboxError::NoDataForTimeStepCoarseChannel {
                timestep_index: _,
                coarse_chan_index: _
            }
        ),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_read_weights_by_baseline_into_buffer_legacy() {
    // Buffer should end up with 1.0's since legacy does not have ewights HDUs. Only MWAX does.
    let mwax_metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let mwax_filename =
        "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";

    //
    // Read the legacy file by frequency using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![mwax_filename];
    let context = CorrelatorContext::new(mwax_metafits_filename, &gpuboxfiles)
        .expect("Failed to create CorrelatorContext");

    // Read into buffer by baseline
    let mut mwalib_hdu_weights_by_bl1: Vec<f32> =
        vec![0.; context.num_timestep_coarse_chan_weight_floats];

    let result_read_bl_buffer =
        context.read_weights_by_baseline_into_buffer(0, 0, &mut mwalib_hdu_weights_by_bl1);
    assert!(result_read_bl_buffer.is_ok());

    // Read by baseline
    let mwalib_hdu_weights_by_bl2_result = context.read_weights_by_baseline(0, 0);
    assert!(mwalib_hdu_weights_by_bl2_result.is_ok());
    // Unwrap to a vector
    let mwalib_hdu_weights_by_bl2 = mwalib_hdu_weights_by_bl2_result.unwrap();

    // Expected result
    let expected_weights_by_bl: Vec<f32> =
        vec![1.0; context.num_timestep_coarse_chan_weight_floats];

    // Check they all match each other
    assert_eq!(mwalib_hdu_weights_by_bl1, expected_weights_by_bl);
    assert_eq!(mwalib_hdu_weights_by_bl2, expected_weights_by_bl);
}

#[test]
fn test_read_weights_by_baseline_into_buffer_mwax() {
    // When this obs was created weights would all be 1.0 (but they are real unlike previous test for legacy MWA!)
    let mwax_metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
    let mwax_filename = "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

    //
    // Read the legacy file by frequency using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![mwax_filename];
    let context = CorrelatorContext::new(mwax_metafits_filename, &gpuboxfiles)
        .expect("Failed to create CorrelatorContext");

    assert_eq!(8256 * 4, context.num_timestep_coarse_chan_weight_floats);

    // Read into buffer by baseline
    let mut mwalib_hdu_weights_by_bl1: Vec<f32> =
        vec![0.; context.num_timestep_coarse_chan_weight_floats];

    let result_read_bl_buffer =
        context.read_weights_by_baseline_into_buffer(0, 10, &mut mwalib_hdu_weights_by_bl1);
    assert!(result_read_bl_buffer.is_ok());

    // Read by baseline
    let mwalib_hdu_weights_by_bl2_result = context.read_weights_by_baseline(0, 10);
    assert!(mwalib_hdu_weights_by_bl2_result.is_ok());
    // Unwrap to a vector
    let mwalib_hdu_weights_by_bl2 = mwalib_hdu_weights_by_bl2_result.unwrap();

    // Expected result
    let expected_weights_by_bl: Vec<f32> =
        vec![1.0; context.num_timestep_coarse_chan_weight_floats];

    // Check they all match each other
    assert_eq!(mwalib_hdu_weights_by_bl1, expected_weights_by_bl);
    assert_eq!(mwalib_hdu_weights_by_bl2, expected_weights_by_bl);
}

//
// Regression tests for reading visibilities and weights
//
// These are here to make it safe to change how data is read and converted. They check every element of the data
// (rather than just a sum), that reads can be done from many threads at once, and what happens when a buffer
// bigger than needed is supplied.
//

const READ_LEGACY_METAFITS: &str = "test_files/1101503312_1_timestep/1101503312.metafits";
const READ_LEGACY_GPUBOX: &str =
    "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";
const READ_MWAX_METAFITS: &str = "test_files/1244973688_1_timestep/1244973688.metafits";
const READ_MWAX_GPUBOX: &str =
    "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

/// The timestep and coarse channel which have data in the legacy test files
const READ_LEGACY_TIMESTEP: usize = 0;
const READ_LEGACY_COARSE_CHAN: usize = 0;

/// The timestep and coarse channel which have data in the mwax test files
const READ_MWAX_TIMESTEP: usize = 0;
const READ_MWAX_COARSE_CHAN: usize = 10;

/// Value used to fill buffers, so we can see which parts were overwritten
const READ_SENTINEL: f32 = -1234.5;

/// Golden values: hashes of the data read from the test files, as read by the current implementation.
/// If one of these changes, the data we return has changed (e.g. the order of the data).
const GOLDEN_LEGACY_BY_BASELINE: u64 = 0xfb91_f2cf_618a_041a;
const GOLDEN_LEGACY_BY_FREQUENCY: u64 = 0x41d4_7446_eb32_4baa;
const GOLDEN_LEGACY_WEIGHTS: u64 = 0x91e1_b2db_affb_4b25;
const GOLDEN_MWAX_BY_BASELINE: u64 = 0x4189_46e6_e994_2235;
const GOLDEN_MWAX_BY_FREQUENCY: u64 = 0x4eca_ef2a_747f_c0a9;
const GOLDEN_MWAX_WEIGHTS: u64 = 0x91e1_b2db_affb_4b25;

fn get_read_legacy_context() -> CorrelatorContext {
    CorrelatorContext::new(READ_LEGACY_METAFITS, &[READ_LEGACY_GPUBOX])
        .expect("Failed to create CorrelatorContext")
}

fn get_read_mwax_context() -> CorrelatorContext {
    CorrelatorContext::new(READ_MWAX_METAFITS, &[READ_MWAX_GPUBOX])
        .expect("Failed to create CorrelatorContext")
}

/// A hash (64 bit FNV-1a) of the bits of all of the floats. Changing the order of the floats changes the hash.
/// Zeros are made positive first, so the hash does not depend on the sign of a zero.
fn hash_floats(data: &[f32]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;

    for value in data {
        for byte in (*value + 0.0).to_le_bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    hash
}

#[test]
fn test_hash_floats() {
    // Order matters
    assert_ne!(hash_floats(&[1.0, 2.0]), hash_floats(&[2.0, 1.0]));
    // The sign of a zero does not
    assert_eq!(hash_floats(&[0.0, 1.0]), hash_floats(&[-0.0, 1.0]));
    // Empty is the offset basis of the hash
    assert_eq!(hash_floats(&[]), 0xcbf2_9ce4_8422_2325);
}

#[test]
fn test_read_golden_legacy() {
    let context = get_read_legacy_context();

    let by_baseline = context
        .read_by_baseline(READ_LEGACY_TIMESTEP, READ_LEGACY_COARSE_CHAN)
        .unwrap();
    let by_frequency = context
        .read_by_frequency(READ_LEGACY_TIMESTEP, READ_LEGACY_COARSE_CHAN)
        .unwrap();
    let weights = context
        .read_weights_by_baseline(READ_LEGACY_TIMESTEP, READ_LEGACY_COARSE_CHAN)
        .unwrap();

    assert_eq!(hash_floats(&by_baseline), GOLDEN_LEGACY_BY_BASELINE);
    assert_eq!(hash_floats(&by_frequency), GOLDEN_LEGACY_BY_FREQUENCY);
    assert_eq!(hash_floats(&weights), GOLDEN_LEGACY_WEIGHTS);
}

#[test]
fn test_read_golden_mwax() {
    let context = get_read_mwax_context();

    let by_baseline = context
        .read_by_baseline(READ_MWAX_TIMESTEP, READ_MWAX_COARSE_CHAN)
        .unwrap();
    let by_frequency = context
        .read_by_frequency(READ_MWAX_TIMESTEP, READ_MWAX_COARSE_CHAN)
        .unwrap();
    let weights = context
        .read_weights_by_baseline(READ_MWAX_TIMESTEP, READ_MWAX_COARSE_CHAN)
        .unwrap();

    assert_eq!(hash_floats(&by_baseline), GOLDEN_MWAX_BY_BASELINE);
    assert_eq!(hash_floats(&by_frequency), GOLDEN_MWAX_BY_FREQUENCY);
    assert_eq!(hash_floats(&weights), GOLDEN_MWAX_WEIGHTS);
}

/// Check that data read by frequency is exactly the data read by baseline, in a different order
fn check_by_frequency_is_transpose_of_by_baseline(
    context: &CorrelatorContext,
    timestep: usize,
    coarse_chan: usize,
) {
    let num_baselines = context.metafits_context.num_baselines;
    let num_fine_chans = context.metafits_context.num_corr_fine_chans_per_coarse;
    let floats_per_baseline_fine_chan = context.metafits_context.num_visibility_pols * 2;

    let by_baseline = context.read_by_baseline(timestep, coarse_chan).unwrap();
    let by_frequency = context.read_by_frequency(timestep, coarse_chan).unwrap();

    assert_eq!(by_baseline.len(), by_frequency.len());
    assert_eq!(
        by_baseline.len(),
        num_baselines * num_fine_chans * floats_per_baseline_fine_chan
    );

    for baseline in 0..num_baselines {
        for fine_chan in 0..num_fine_chans {
            let a = (baseline * num_fine_chans + fine_chan) * floats_per_baseline_fine_chan;
            let b = (fine_chan * num_baselines + baseline) * floats_per_baseline_fine_chan;

            assert_eq!(
                by_baseline[a..a + floats_per_baseline_fine_chan],
                by_frequency[b..b + floats_per_baseline_fine_chan],
                "baseline {baseline}, fine chan {fine_chan}"
            );
        }
    }
}

#[test]
fn test_read_by_frequency_is_transpose_of_by_baseline_legacy() {
    check_by_frequency_is_transpose_of_by_baseline(
        &get_read_legacy_context(),
        READ_LEGACY_TIMESTEP,
        READ_LEGACY_COARSE_CHAN,
    );
}

#[test]
fn test_read_by_frequency_is_transpose_of_by_baseline_mwax() {
    check_by_frequency_is_transpose_of_by_baseline(
        &get_read_mwax_context(),
        READ_MWAX_TIMESTEP,
        READ_MWAX_COARSE_CHAN,
    );
}

#[test]
fn test_read_mwax_by_baseline_is_the_raw_fits_data() {
    // For mwax, reading by baseline is not converted at all, so it is the same as reading the HDU directly
    let context = get_read_mwax_context();

    let mut fptr = fits_open!(&READ_MWAX_GPUBOX).unwrap();
    let hdu = fits_open_hdu!(&mut fptr, 1).unwrap();
    let raw: Vec<f32> = get_fits_image!(&mut fptr, &hdu).unwrap();

    let by_baseline = context
        .read_by_baseline(READ_MWAX_TIMESTEP, READ_MWAX_COARSE_CHAN)
        .unwrap();

    assert_eq!(raw, by_baseline);
}

#[test]
fn test_read_legacy_weights_are_all_ones() {
    let context = get_read_legacy_context();

    let weights = context
        .read_weights_by_baseline(READ_LEGACY_TIMESTEP, READ_LEGACY_COARSE_CHAN)
        .unwrap();

    assert_eq!(
        weights.len(),
        context.metafits_context.num_baselines * context.metafits_context.num_visibility_pols
    );
    assert!(weights.iter().all(|w| *w == 1.0));
}

#[test]
fn test_read_into_buffer_gives_the_same_data_as_read() {
    for (context, timestep, coarse_chan) in [
        (
            get_read_legacy_context(),
            READ_LEGACY_TIMESTEP,
            READ_LEGACY_COARSE_CHAN,
        ),
        (
            get_read_mwax_context(),
            READ_MWAX_TIMESTEP,
            READ_MWAX_COARSE_CHAN,
        ),
    ] {
        let mut buffer = vec![READ_SENTINEL; context.num_timestep_coarse_chan_floats];
        context
            .read_by_baseline_into_buffer(timestep, coarse_chan, &mut buffer)
            .unwrap();
        assert_eq!(
            buffer,
            context.read_by_baseline(timestep, coarse_chan).unwrap()
        );

        let mut buffer = vec![READ_SENTINEL; context.num_timestep_coarse_chan_floats];
        context
            .read_by_frequency_into_buffer(timestep, coarse_chan, &mut buffer)
            .unwrap();
        assert_eq!(
            buffer,
            context.read_by_frequency(timestep, coarse_chan).unwrap()
        );

        let mut buffer = vec![READ_SENTINEL; context.num_timestep_coarse_chan_weight_floats];
        context
            .read_weights_by_baseline_into_buffer(timestep, coarse_chan, &mut buffer)
            .unwrap();
        assert_eq!(
            buffer,
            context
                .read_weights_by_baseline(timestep, coarse_chan)
                .unwrap()
        );
    }
}

#[test]
fn test_read_into_a_bigger_buffer_leaves_the_rest_of_the_buffer_alone() {
    // A buffer which is bigger than needed is accepted, and only the first part of it is used.
    // (By baseline for mwax is not tested: for that case, the number of floats read is the length of the buffer.)
    let extra = 100;

    let legacy = get_read_legacy_context();
    let mwax = get_read_mwax_context();

    // (context, timestep, coarse chan, read by baseline?)
    for (context, timestep, coarse_chan, by_baseline) in [
        (&legacy, READ_LEGACY_TIMESTEP, READ_LEGACY_COARSE_CHAN, true),
        (
            &legacy,
            READ_LEGACY_TIMESTEP,
            READ_LEGACY_COARSE_CHAN,
            false,
        ),
        (&mwax, READ_MWAX_TIMESTEP, READ_MWAX_COARSE_CHAN, false),
    ] {
        let len = context.num_timestep_coarse_chan_floats;
        let mut buffer = vec![READ_SENTINEL; len + extra];

        if by_baseline {
            context
                .read_by_baseline_into_buffer(timestep, coarse_chan, &mut buffer)
                .unwrap();
        } else {
            context
                .read_by_frequency_into_buffer(timestep, coarse_chan, &mut buffer)
                .unwrap();
        }

        let expected = if by_baseline {
            context.read_by_baseline(timestep, coarse_chan).unwrap()
        } else {
            context.read_by_frequency(timestep, coarse_chan).unwrap()
        };

        assert_eq!(buffer[..len], expected[..]);
        assert!(buffer[len..].iter().all(|x| *x == READ_SENTINEL));
    }
}

#[test]
fn test_read_last_valid_indices() {
    let context = get_read_mwax_context();

    // The last valid indices are valid, but there is no data for them
    let last_timestep = context.num_timesteps - 1;
    let last_coarse_chan = context.num_coarse_chans - 1;

    assert!(matches!(
        context.read_by_baseline(last_timestep, READ_MWAX_COARSE_CHAN),
        Err(GpuboxError::NoDataForTimeStepCoarseChannel { .. })
    ));
    assert!(matches!(
        context.read_by_baseline(READ_MWAX_TIMESTEP, last_coarse_chan),
        Err(GpuboxError::NoDataForTimeStepCoarseChannel { .. })
    ));

    // One more than the last is not valid, and the error says what the last valid index is
    assert!(matches!(
        context.read_by_baseline(context.num_timesteps, READ_MWAX_COARSE_CHAN),
        Err(GpuboxError::InvalidTimeStepIndex(i)) if i == last_timestep
    ));
    assert!(matches!(
        context.read_by_baseline(READ_MWAX_TIMESTEP, context.num_coarse_chans),
        Err(GpuboxError::InvalidCoarseChanIndex(i)) if i == last_coarse_chan
    ));
}

#[test]
fn test_correlator_context_is_send_and_sync() {
    // We need to be able to share a context between threads to read from it in parallel
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<CorrelatorContext>();
}

#[test]
fn test_read_from_many_threads_at_once() {
    for (context, timestep, coarse_chan) in [
        (
            get_read_legacy_context(),
            READ_LEGACY_TIMESTEP,
            READ_LEGACY_COARSE_CHAN,
        ),
        (
            get_read_mwax_context(),
            READ_MWAX_TIMESTEP,
            READ_MWAX_COARSE_CHAN,
        ),
    ] {
        let expected_by_baseline = context.read_by_baseline(timestep, coarse_chan).unwrap();
        let expected_by_frequency = context.read_by_frequency(timestep, coarse_chan).unwrap();
        let expected_weights = context
            .read_weights_by_baseline(timestep, coarse_chan)
            .unwrap();

        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    scope.spawn(|| {
                        (
                            context.read_by_baseline(timestep, coarse_chan).unwrap(),
                            context.read_by_frequency(timestep, coarse_chan).unwrap(),
                            context
                                .read_weights_by_baseline(timestep, coarse_chan)
                                .unwrap(),
                        )
                    })
                })
                .collect();

            for handle in handles {
                let (by_baseline, by_frequency, weights) = handle.join().unwrap();

                assert_eq!(by_baseline, expected_by_baseline);
                assert_eq!(by_frequency, expected_by_frequency);
                assert_eq!(weights, expected_weights);
            }
        });
    }
}

/// Check the fine channel centre frequencies of every coarse channel
fn check_fine_chan_freqs(context: &CorrelatorContext) {
    let num_fine_chans = context.metafits_context.num_corr_fine_chans_per_coarse;
    let width_hz = context.metafits_context.corr_fine_chan_width_hz as f64;

    for (index, coarse_chan) in context.coarse_chans.iter().enumerate() {
        let freqs = context.get_fine_chan_freqs_hz_array(&[index]).unwrap();

        assert_eq!(freqs.len(), num_fine_chans);

        // Evenly spaced
        assert!(freqs.windows(2).all(|w| w[1] - w[0] == width_hz));

        // and inside the coarse channel
        assert!(freqs[0] >= coarse_chan.chan_start_hz as f64);
        assert!(freqs[num_fine_chans - 1] <= coarse_chan.chan_end_hz as f64);
    }

    // All of the channels, and in any order, is the same as the channels one at a time
    let indices: Vec<usize> = (0..context.num_coarse_chans).rev().collect();
    let all = context.get_fine_chan_freqs_hz_array(&indices).unwrap();
    let one_at_a_time: Vec<f64> = indices
        .iter()
        .flat_map(|i| context.get_fine_chan_freqs_hz_array(&[*i]).unwrap())
        .collect();

    assert_eq!(all, one_at_a_time);
    assert!(context
        .get_fine_chan_freqs_hz_array(&[])
        .unwrap()
        .is_empty());
}

#[test]
fn test_get_fine_chan_freqs_hz_array_every_coarse_chan_legacy() {
    check_fine_chan_freqs(&get_read_legacy_context());
}

#[test]
fn test_get_fine_chan_freqs_hz_array_every_coarse_chan_mwax() {
    check_fine_chan_freqs(&get_read_mwax_context());
}

#[test]
fn test_reads_that_share_the_temp_buffer_give_golden_data() {
    // The legacy by baseline read and both by frequency reads reuse one temp buffer per thread. Mix them, on one
    // thread, and check each result is still the golden one (stale data in the temp buffer would change a hash).
    let legacy = get_read_legacy_context();
    let mwax = get_read_mwax_context();

    for _ in 0..2 {
        let legacy_by_frequency = legacy
            .read_by_frequency(READ_LEGACY_TIMESTEP, READ_LEGACY_COARSE_CHAN)
            .unwrap();
        let mwax_by_frequency = mwax
            .read_by_frequency(READ_MWAX_TIMESTEP, READ_MWAX_COARSE_CHAN)
            .unwrap();
        let legacy_by_baseline = legacy
            .read_by_baseline(READ_LEGACY_TIMESTEP, READ_LEGACY_COARSE_CHAN)
            .unwrap();
        let mwax_by_baseline = mwax
            .read_by_baseline(READ_MWAX_TIMESTEP, READ_MWAX_COARSE_CHAN)
            .unwrap();

        assert_eq!(
            hash_floats(&legacy_by_frequency),
            GOLDEN_LEGACY_BY_FREQUENCY
        );
        assert_eq!(hash_floats(&mwax_by_frequency), GOLDEN_MWAX_BY_FREQUENCY);
        assert_eq!(hash_floats(&legacy_by_baseline), GOLDEN_LEGACY_BY_BASELINE);
        assert_eq!(hash_floats(&mwax_by_baseline), GOLDEN_MWAX_BY_BASELINE);
    }
}

#[test]
fn test_with_read_scratch_sizes_and_nesting() {
    const BIG: usize = 16;
    const SMALL: usize = 4;

    // The buffer has the size asked for, bigger or smaller than last time
    assert_eq!(with_read_scratch(BIG, |b| b.len()), BIG);
    assert_eq!(with_read_scratch(SMALL, |b| b.len()), SMALL);
    assert_eq!(with_read_scratch(BIG, |b| b.len()), BIG);

    // A nested call gets its own buffer, and does not panic
    let (outer, inner) = with_read_scratch(BIG, |outer| {
        outer.fill(READ_SENTINEL);
        let inner_len = with_read_scratch(SMALL, |inner| {
            inner.fill(0.0);
            inner.len()
        });
        (outer.iter().all(|v| *v == READ_SENTINEL), inner_len)
    });
    assert!(outer);
    assert_eq!(inner, SMALL);
}

//
// Phase 2 (safety) tests: errors instead of panics or partial reads
//

/// The three `_into_buffer` reads, by name, so a test can run each of them
const INTO_BUFFER_READS: [&str; 3] = [
    "read_by_baseline_into_buffer",
    "read_by_frequency_into_buffer",
    "read_weights_by_baseline_into_buffer",
];

/// Run one of the `_into_buffer` reads by name.
fn read_into_buffer_by_name(
    context: &CorrelatorContext,
    read: &str,
    timestep: usize,
    coarse_chan: usize,
    buffer: &mut [f32],
) -> Result<(), GpuboxError> {
    match read {
        "read_by_baseline_into_buffer" => {
            context.read_by_baseline_into_buffer(timestep, coarse_chan, buffer)
        }
        "read_by_frequency_into_buffer" => {
            context.read_by_frequency_into_buffer(timestep, coarse_chan, buffer)
        }
        "read_weights_by_baseline_into_buffer" => {
            context.read_weights_by_baseline_into_buffer(timestep, coarse_chan, buffer)
        }
        _ => panic!("unknown read {read}"),
    }
}

/// Number of floats the given `_into_buffer` read gives
fn floats_for_read(context: &CorrelatorContext, read: &str) -> usize {
    if read == "read_weights_by_baseline_into_buffer" {
        context.num_timestep_coarse_chan_weight_floats
    } else {
        context.num_timestep_coarse_chan_floats
    }
}

#[test]
fn test_read_into_a_buffer_that_is_too_small_is_an_error() {
    for (context, timestep, coarse_chan) in [
        (
            get_read_legacy_context(),
            READ_LEGACY_TIMESTEP,
            READ_LEGACY_COARSE_CHAN,
        ),
        (
            get_read_mwax_context(),
            READ_MWAX_TIMESTEP,
            READ_MWAX_COARSE_CHAN,
        ),
    ] {
        for read in INTO_BUFFER_READS {
            let expected = floats_for_read(&context, read);

            for got in [0, 1, expected - 1] {
                let mut buffer = vec![READ_SENTINEL; got];

                let result =
                    read_into_buffer_by_name(&context, read, timestep, coarse_chan, &mut buffer);

                assert!(
                    matches!(
                        result,
                        Err(GpuboxError::BufferTooSmall { expected: e, got: g }) if e == expected && g == got
                    ),
                    "{read} with {got} floats: {result:?}"
                );
                // Nothing was written
                assert!(buffer.iter().all(|x| *x == READ_SENTINEL), "{read}");
            }
        }
    }
}

#[test]
fn test_read_into_a_bigger_buffer_uses_only_the_first_part_for_every_read() {
    // Like test_read_into_a_bigger_buffer_leaves_the_rest_of_the_buffer_alone, but for every read, including MWAX by
    // baseline (which used to read as many floats as the buffer had) and the weights.
    let extra = 100;

    for (context, timestep, coarse_chan) in [
        (
            get_read_legacy_context(),
            READ_LEGACY_TIMESTEP,
            READ_LEGACY_COARSE_CHAN,
        ),
        (
            get_read_mwax_context(),
            READ_MWAX_TIMESTEP,
            READ_MWAX_COARSE_CHAN,
        ),
    ] {
        for read in INTO_BUFFER_READS {
            let len = floats_for_read(&context, read);
            let mut exact = vec![READ_SENTINEL; len];
            let mut bigger = vec![READ_SENTINEL; len + extra];

            read_into_buffer_by_name(&context, read, timestep, coarse_chan, &mut exact).unwrap();
            read_into_buffer_by_name(&context, read, timestep, coarse_chan, &mut bigger).unwrap();

            assert_eq!(bigger[..len], exact[..], "{read}");
            assert!(bigger[len..].iter().all(|x| *x == READ_SENTINEL), "{read}");
        }
    }
}

#[test]
fn test_get_fine_chan_freqs_hz_array_invalid_index_is_an_error() {
    for context in [get_read_legacy_context(), get_read_mwax_context()] {
        let num_coarse_chans = context.coarse_chans.len();

        for indices in [vec![num_coarse_chans], vec![0, num_coarse_chans + 10]] {
            let result = context.get_fine_chan_freqs_hz_array(&indices);

            assert!(
                matches!(result, Err(GpuboxError::InvalidCoarseChanIndex(max)) if max == num_coarse_chans - 1),
                "{indices:?}: {result:?}"
            );
        }
    }
}

#[test]
fn test_check_legacy_tile_count() {
    assert!(CorrelatorContext::check_legacy_tile_count(convert::LEGACY_NUM_TILES).is_ok());

    for num_tiles in [0, 120, 256] {
        assert!(matches!(
            CorrelatorContext::check_legacy_tile_count(num_tiles),
            Err(GpuboxError::UnsupportedLegacyTileCount { expected, num_tiles: n })
                if expected == convert::LEGACY_NUM_TILES && n == num_tiles
        ));
    }
}

#[test]
fn test_validate_first_weights_hdu() {
    let context = get_read_mwax_context();
    let num_baselines = context.metafits_context.num_baselines;
    let num_pols = context.metafits_context.num_visibility_pols;

    let mut fptr = fits_open!(&READ_MWAX_GPUBOX).unwrap();
    assert!(
        CorrelatorContext::validate_first_weights_hdu(num_baselines, num_pols, &mut fptr).is_ok()
    );

    // The same file against the wrong number of baselines or pols
    for (baselines, pols) in [(num_baselines + 1, num_pols), (num_baselines, num_pols + 1)] {
        let result = CorrelatorContext::validate_first_weights_hdu(baselines, pols, &mut fptr);

        assert!(
            matches!(
                result,
                Err(GpuboxError::MwaxWeightsNaxisMismatch {
                    naxis1,
                    naxis2,
                    expected_naxis1,
                    expected_naxis2,
                    ..
                }) if naxis1 == num_pols && naxis2 == num_baselines && expected_naxis1 == pols && expected_naxis2 == baselines
            ),
            "{result:?}"
        );
    }
}

#[test]
fn test_read_float_image_reads_the_given_hdu_even_if_the_current_hdu_moved() {
    // cfitsio reads from its "current" HDU. Open the visibilities HDU, then move cfitsio to the weights HDU, then
    // read with the visibilities HDU: the visibilities must be read, not the weights.
    let context = get_read_mwax_context();
    let expected = context
        .read_by_baseline(READ_MWAX_TIMESTEP, READ_MWAX_COARSE_CHAN)
        .unwrap();

    let mut fptr = fits_open!(&READ_MWAX_GPUBOX).unwrap();
    let visibilities_hdu = fits_open_hdu!(&mut fptr, 1).unwrap();
    let _weights_hdu = fits_open_hdu!(&mut fptr, MWAX_FIRST_WEIGHTS_HDU_INDEX).unwrap();

    let mut buffer = vec![READ_SENTINEL; context.num_timestep_coarse_chan_floats];
    get_fits_float_image_into_buffer!(&mut fptr, &visibilities_hdu, &mut buffer).unwrap();

    assert_eq!(buffer, expected);
}
