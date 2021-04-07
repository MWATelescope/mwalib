// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Unit tests for correlator context
*/
#[cfg(test)]
use super::*;
use float_cmp::*;

#[test]
fn test_context_new_missing_gpubox_files() {
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let gpuboxfiles = Vec::new();

    // No gpubox files provided
    let context = CorrelatorContext::new(&metafits_filename, &gpuboxfiles);
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
    let context = CorrelatorContext::new(&metafits_filename, &gpuboxfiles);

    assert!(context.is_err());
}

#[test]
fn test_context_legacy_v1() {
    // Open the test mwax file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let filename = "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![filename];
    let context = CorrelatorContext::new(&metafits_filename, &gpuboxfiles)
        .expect("Failed to create mwalibContext");

    // Test the properties of the context object match what we expect
    // Correlator version:       v1 Legacy,
    assert_eq!(context.corr_version, CorrelatorVersion::Legacy);

    // Actual UNIX start time:   1417468096,
    assert_eq!(context.start_unix_time_ms, 1_417_468_096_000);

    // Actual UNIX end time:     1417468098,
    assert_eq!(context.end_unix_time_ms, 1_417_468_098_000);

    // Actual duration:          2 s,
    assert_eq!(context.duration_ms, 2000);

    // num timesteps:            1,
    assert_eq!(context.num_timesteps, 1);

    // timesteps:                [unix=1417468096.000],
    assert_eq!(context.timesteps[0].unix_time_ms, 1_417_468_096_000);

    // observation bandwidth:    1.28 MHz,
    assert_eq!(context.bandwidth_hz, 1_280_000);

    // num coarse channels,      1,
    assert_eq!(context.num_coarse_chans, 1);

    // coarse channels:          [gpu=1 corr=0 rec=109 @ 139.520 MHz],
    assert_eq!(context.coarse_chans[0].gpubox_number, 1);
    assert_eq!(context.coarse_chans[0].rec_chan_number, 109);
    assert_eq!(context.coarse_chans[0].chan_centre_hz, 139_520_000);

    // gpubox HDU size:          32.25 MiB,
    // Memory usage per scan:    32.25 MiB,

    // metafits filename:        ../test_files/1101503312_1_timestep/1101503312.metafits,
    // gpubox batches:           [
    // batch_number=0 gpubox_files=[filename=../test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits channelidentifier=1]
}

#[test]
fn test_read_by_frequency_invalid_inputs() {
    let mwax_metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
    let mwax_filename = "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![mwax_filename];
    let mut context = CorrelatorContext::new(&mwax_metafits_filename, &gpuboxfiles)
        .expect("Failed to create CorrelatorContext");

    // 99999 is invalid as a timestep for this observation
    let result_invalid_timestep = context.read_by_frequency(99999, 0);

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
}

#[test]
fn test_read_by_baseline_invalid_inputs() {
    let mwax_metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
    let mwax_filename = "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![mwax_filename];
    let mut context = CorrelatorContext::new(&mwax_metafits_filename, &gpuboxfiles)
        .expect("Failed to create CorrelatorContext");

    // 99999 is invalid as a timestep for this observation
    let result_invalid_timestep = context.read_by_baseline(99999, 0);

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
    let mut context = CorrelatorContext::new(&mwax_metafits_filename, &gpuboxfiles)
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
    approx_eq!(f64, sum_bl, sum_freq, F64Margin::default());
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
    let mut context = CorrelatorContext::new(&mwax_metafits_filename, &gpuboxfiles)
        .expect("Failed to create CorrelatorContext");

    // Read and convert first HDU by baseline
    let mwalib_hdu_data_by_bl: Vec<f32> = context.read_by_baseline(0, 0).expect("Error!");

    // Read and convert first HDU by frequency
    let mwalib_hdu_data_by_freq: Vec<f32> = context.read_by_frequency(0, 0).expect("Error!");

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
    let context = CorrelatorContext::new(&metafits_filename, &gpuboxfiles)
        .expect("Failed to create CorrelatorContext");

    let coarse_chan = context.coarse_chans[0].gpubox_number;
    let (batch_index, _) =
        context.gpubox_time_map[&context.timesteps[0].unix_time_ms][&coarse_chan];

    let mut fptr =
        fits_open!(&context.gpubox_batches[batch_index].gpubox_files[0].filename).unwrap();

    let result_valid = CorrelatorContext::validate_first_hdu(
        context.corr_version,
        context.metafits_context.num_corr_fine_chans_per_coarse,
        context.metafits_context.num_baselines,
        context.metafits_context.num_visibility_pols,
        &mut fptr,
    );

    let result_invalid1 = CorrelatorContext::validate_first_hdu(
        context.corr_version,
        context.metafits_context.num_corr_fine_chans_per_coarse + 1,
        context.metafits_context.num_baselines,
        context.metafits_context.num_visibility_pols,
        &mut fptr,
    );

    let result_invalid2 = CorrelatorContext::validate_first_hdu(
        context.corr_version,
        context.metafits_context.num_corr_fine_chans_per_coarse,
        context.metafits_context.num_baselines + 1,
        context.metafits_context.num_visibility_pols,
        &mut fptr,
    );

    let result_invalid3 = CorrelatorContext::validate_first_hdu(
        context.corr_version,
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
        CorrelatorVersion::OldLegacy,
        metafits_fine_chans_per_coarse,
        metafits_baselines,
        visibility_pols,
        8256 * 4 * 2,
        128,
    );

    assert!(result_good1.is_ok());

    let result_good2 = CorrelatorContext::validate_hdu_axes(
        CorrelatorVersion::Legacy,
        metafits_fine_chans_per_coarse,
        metafits_baselines,
        visibility_pols,
        8256 * 4 * 2,
        128,
    );

    assert!(result_good2.is_ok());

    let result_good3 = CorrelatorContext::validate_hdu_axes(
        CorrelatorVersion::V2,
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
        CorrelatorVersion::OldLegacy,
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
        CorrelatorVersion::OldLegacy,
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
        CorrelatorVersion::Legacy,
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
        CorrelatorVersion::Legacy,
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
        CorrelatorVersion::V2,
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
        CorrelatorVersion::V2,
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
