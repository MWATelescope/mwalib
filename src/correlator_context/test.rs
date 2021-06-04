// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Unit tests for correlator context
*/
#[cfg(test)]
use super::*;
use float_cmp::*;

#[cfg(test)]
/// Helper method will create a gpubox timemap which will be used by some tests in this module
///
fn create_determine_common_obs_times_and_chans_test_data(
    coarse_chan101_unix_times: Vec<u64>,
    coarse_chan102_unix_times: Vec<u64>,
    coarse_chan103_unix_times: Vec<u64>,
    coarse_chan104_unix_times: Vec<u64>,
) -> GpuboxTimeMap {
    // Create a dummy BTree GPUbox map
    let mut gpubox_time_map = GpuboxTimeMap::new();

    for (chan_index, unix_time_ms) in coarse_chan101_unix_times.iter().enumerate() {
        gpubox_time_map
            .entry(*unix_time_ms)
            .or_insert_with(BTreeMap::new)
            .entry(101)
            .or_insert((0, chan_index + 1));
    }

    for (chan_index, unix_time_ms) in coarse_chan102_unix_times.iter().enumerate() {
        gpubox_time_map
            .entry(*unix_time_ms)
            .or_insert_with(BTreeMap::new)
            .entry(102)
            .or_insert((0, chan_index + 1));
    }

    for (chan_index, unix_time_ms) in coarse_chan103_unix_times.iter().enumerate() {
        gpubox_time_map
            .entry(*unix_time_ms)
            .or_insert_with(BTreeMap::new)
            .entry(103)
            .or_insert((0, chan_index + 1));
    }

    for (chan_index, unix_time_ms) in coarse_chan104_unix_times.iter().enumerate() {
        gpubox_time_map
            .entry(*unix_time_ms)
            .or_insert_with(BTreeMap::new)
            .entry(104)
            .or_insert((0, chan_index + 1));
    }

    gpubox_time_map
}

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
    // Open the test legacy file
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
    assert_eq!(context.mwa_version, MWAVersion::CorrLegacy);

    // Actual UNIX start time:   1417468096,
    assert_eq!(context.common_start_unix_time_ms, 1_417_468_096_000);

    // Actual UNIX end time:     1417468098,
    assert_eq!(context.common_end_unix_time_ms, 1_417_468_098_000);

    // Actual duration:          2 s,
    assert_eq!(context.common_duration_ms, 2000);

    // num timesteps:            1,
    assert_eq!(context.num_timesteps, 56);

    // timesteps:                [unix=1417468096.000],
    assert_eq!(context.timesteps[0].unix_time_ms, 1_417_468_096_000);

    // observation bandwidth:    1.28 MHz,
    assert_eq!(context.common_bandwidth_hz, 1_280_000);

    // num coarse channels,      1,
    assert_eq!(context.num_common_coarse_chans, 1);

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
    let context = CorrelatorContext::new(&metafits_filename, &gpuboxfiles)
        .expect("Failed to create mwalibContext");

    // Test the properties of the context object match what we expect
    // Correlator version:       v2,
    assert_eq!(context.mwa_version, MWAVersion::CorrMWAXv2);

    // Actual UNIX start time:   1560938470,
    assert_eq!(context.common_start_unix_time_ms, 1_560_938_470_000);

    // Actual UNIX end time:     1560938471,
    assert_eq!(context.common_end_unix_time_ms, 1_560_938_471_000);

    // Actual duration:          1 s,
    assert_eq!(context.common_duration_ms, 1000);

    // num timesteps:            1,
    assert_eq!(context.num_timesteps, 120);

    // timesteps:                [unix=1560938470.000],
    assert_eq!(context.timesteps[0].unix_time_ms, 1_560_938_470_000);

    // observation bandwidth:    1.28 MHz,
    assert_eq!(context.common_bandwidth_hz, 1_280_000);

    // num coarse channels,      1,
    assert_eq!(context.num_common_coarse_chans, 1);

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
}

#[test]
fn test_read_by_frequency_invalid_inputs() {
    let mwax_metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
    let mwax_filename = "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

    // Open a context and load in a test metafits and gpubox file
    let gpuboxfiles = vec![mwax_filename];
    let context = CorrelatorContext::new(&mwax_metafits_filename, &gpuboxfiles)
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
    let context = CorrelatorContext::new(&mwax_metafits_filename, &gpuboxfiles)
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
    let context = CorrelatorContext::new(&mwax_metafits_filename, &gpuboxfiles)
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
    assert_eq!(
        approx_eq!(f64, sum_bl, sum_freq, F64Margin::default()),
        true
    );
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
    let context = CorrelatorContext::new(&mwax_metafits_filename, &gpuboxfiles)
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
    let context = CorrelatorContext::new(&metafits_filename, &gpuboxfiles)
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
    let context = CorrelatorContext::new(&mwax_metafits_filename, &gpuboxfiles)
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
    assert_eq!(approx_eq!(f64, sum_freq, 0., F64Margin::default()), false);
    assert_eq!(approx_eq!(f64, sum_bl, 0., F64Margin::default()), false);
    assert_eq!(approx_eq!(f64, sum_bl2, 0., F64Margin::default()), false);

    // Check they all match each other
    assert_eq!(
        approx_eq!(f64, sum_bl, sum_freq, F64Margin::default()),
        true
    );

    assert_eq!(approx_eq!(f64, sum_bl, sum_bl2, F64Margin::default()), true);
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
    let context = CorrelatorContext::new(&mwax_metafits_filename, &gpuboxfiles)
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
    assert_eq!(approx_eq!(f64, sum_bl, 0., F64Margin::default()), false);
    assert_eq!(approx_eq!(f64, sum_freq, 0., F64Margin::default()), false);
    assert_eq!(approx_eq!(f64, sum_freq2, 0., F64Margin::default()), false);

    // Check they all match each other
    assert_eq!(
        approx_eq!(f64, sum_bl, sum_freq, F64Margin::default()),
        true
    );

    assert_eq!(
        approx_eq!(f64, sum_freq, sum_freq2, F64Margin::default()),
        true
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
    let context = CorrelatorContext::new(&mwax_metafits_filename, &gpuboxfiles)
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
    assert_eq!(approx_eq!(f64, sum_freq, 0., F64Margin::default()), false);
    assert_eq!(approx_eq!(f64, sum_bl, 0., F64Margin::default()), false);
    assert_eq!(approx_eq!(f64, sum_bl2, 0., F64Margin::default()), false);

    // Check they all match each other
    assert_eq!(
        approx_eq!(f64, sum_bl, sum_freq, F64Margin::default()),
        true
    );

    assert_eq!(approx_eq!(f64, sum_bl, sum_bl2, F64Margin::default()), true);
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
    let context = CorrelatorContext::new(&mwax_metafits_filename, &gpuboxfiles)
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
    assert_eq!(approx_eq!(f64, sum_bl, 0., F64Margin::default()), false);
    assert_eq!(approx_eq!(f64, sum_freq, 0., F64Margin::default()), false);
    assert_eq!(approx_eq!(f64, sum_freq2, 0., F64Margin::default()), false);

    // Check they all match each other
    assert_eq!(
        approx_eq!(f64, sum_bl, sum_freq, F64Margin::default()),
        true
    );

    assert_eq!(
        approx_eq!(f64, sum_freq, sum_freq2, F64Margin::default()),
        true
    );
}

#[test]
fn test_determine_common_obs_times_and_chans_all_common() {
    // Scenario- all 4 coarse chans have a common timestep
    //        1000
    // chan101 X
    // chan102 X
    // chan103 X
    // chan104 X
    //

    // Set corr integration time
    let corr_int_time_ms = 1000;

    // Setup variables to generate gpuboxtimemap
    let coarse_chan101_timesteps: Vec<u64> = vec![1000];
    let coarse_chan102_timesteps: Vec<u64> = vec![1000];
    let coarse_chan103_timesteps: Vec<u64> = vec![1000];
    let coarse_chan104_timesteps: Vec<u64> = vec![1000];

    // Get a populated GpuboxTimeMap
    let gpubox_time_map = create_determine_common_obs_times_and_chans_test_data(
        coarse_chan101_timesteps,
        coarse_chan102_timesteps,
        coarse_chan103_timesteps,
        coarse_chan104_timesteps,
    );

    // Actually run our test!
    let result = determine_common_obs_times_and_chans(&gpubox_time_map, corr_int_time_ms, None);

    // Check we did not encounter an error
    assert!(result.is_ok());

    // Unwrap to get the obstimesandchans struct
    let o = result.unwrap();

    // Check contents is what is expected
    assert_eq!(o.end_time_unix_ms - o.start_time_unix_ms, o.duration_ms);
    assert_eq!(o.start_time_unix_ms, 1000);
    assert_eq!(o.end_time_unix_ms, 2000);
    assert_eq!(o.duration_ms, 1000);
    assert_eq!(o.coarse_chan_identifiers, vec![101, 102, 103, 104]);

    //
    // Now run the same test, but with a good time == 1000 (same as the data)
    //
    // Actually run our test!
    let result_good =
        determine_common_obs_times_and_chans(&gpubox_time_map, corr_int_time_ms, Some(1000));

    // Check we did not encounter an error
    assert!(result_good.is_ok());

    // Unwrap to get the obstimesandchans struct
    let o_good = result_good.unwrap();

    // Check contents is what is expected
    assert_eq!(
        o_good.end_time_unix_ms - o_good.start_time_unix_ms,
        o_good.duration_ms
    );
    assert_eq!(o_good.start_time_unix_ms, 1000);
    assert_eq!(o_good.end_time_unix_ms, 2000);
    assert_eq!(o_good.duration_ms, 1000);
    assert_eq!(o_good.coarse_chan_identifiers, vec![101, 102, 103, 104]);

    //
    // Now run the same test, but with a good time == 2000 (1 second AFTER the data)
    //
    // Actually run our test!
    let result_good2 =
        determine_common_obs_times_and_chans(&gpubox_time_map, corr_int_time_ms, Some(2000));

    // Check we did not encounter an error
    assert!(result_good2.is_ok());

    // Unwrap to get the obstimesandchans struct
    let o_good2 = result_good2.unwrap();

    // Check contents is what is expected
    assert_eq!(o_good2.start_time_unix_ms, 0);
    assert_eq!(o_good2.end_time_unix_ms, 0);
    assert_eq!(o_good2.duration_ms, 0);
    assert_eq!(o_good2.coarse_chan_identifiers, vec![]);

    //
    // Now run the same test, but with a good time == 0 (1 second before the data)
    //
    // Actually run our test!
    let result_good3 =
        determine_common_obs_times_and_chans(&gpubox_time_map, corr_int_time_ms, Some(0));

    // Check we did not encounter an error
    assert!(result_good3.is_ok());

    // Unwrap to get the obstimesandchans struct
    let o_good3 = result_good3.unwrap();

    // Check contents is what is expected
    assert_eq!(o_good3.start_time_unix_ms, 1000);
    assert_eq!(o_good3.end_time_unix_ms, 2000);
    assert_eq!(o_good3.duration_ms, 1000);
    assert_eq!(o_good3.coarse_chan_identifiers, vec![101, 102, 103, 104]);
}

#[test]
fn test_determine_common_obs_times_and_chans_no_common() {
    // Scenario- all 4 coarse chans have no common timesteps (so it will use the first)
    //        1000 2000 3000 4000
    // chan101 X
    // chan102       X
    // chan103            X
    // chan104                X
    //

    // Set corr integration time
    let corr_int_time_ms = 1000;

    // Setup variables to generate gpuboxtimemap
    let coarse_chan101_timesteps: Vec<u64> = vec![1000];
    let coarse_chan102_timesteps: Vec<u64> = vec![2000];
    let coarse_chan103_timesteps: Vec<u64> = vec![3000];
    let coarse_chan104_timesteps: Vec<u64> = vec![4000];

    // Get a populated GpuboxTimeMap
    let gpubox_time_map = create_determine_common_obs_times_and_chans_test_data(
        coarse_chan101_timesteps,
        coarse_chan102_timesteps,
        coarse_chan103_timesteps,
        coarse_chan104_timesteps,
    );

    // Actually run our test!
    let result = determine_common_obs_times_and_chans(&gpubox_time_map, corr_int_time_ms, None);

    // Check we did not encounter an error
    assert!(result.is_ok());

    // Unwrap to get the obstimesandchans struct
    let o = result.unwrap();

    // Check contents of common timesteps and coarse channels is what is expected
    assert_eq!(o.end_time_unix_ms - o.start_time_unix_ms, o.duration_ms);
    assert_eq!(o.start_time_unix_ms, 1000);
    assert_eq!(o.end_time_unix_ms, 2000);
    assert_eq!(o.duration_ms, 1000);
    assert_eq!(o.coarse_chan_identifiers, vec![101]);

    //
    // Now run the same test, but with a good time 2000
    //
    // Actually run our test!
    let result_good =
        determine_common_obs_times_and_chans(&gpubox_time_map, corr_int_time_ms, Some(2000));

    // Check we did not encounter an error
    assert!(result_good.is_ok());

    // Unwrap to get the obstimesandchans struct
    let o_good = result_good.unwrap();

    // Check contents is what is expected
    assert_eq!(
        o_good.end_time_unix_ms - o_good.start_time_unix_ms,
        o_good.duration_ms
    );
    assert_eq!(o_good.start_time_unix_ms, 2000);
    assert_eq!(o_good.end_time_unix_ms, 3000);
    assert_eq!(o_good.duration_ms, 1000);
    assert_eq!(o_good.coarse_chan_identifiers, vec![102]);
}

#[test]
fn test_determine_common_obs_times_and_chans_two_common() {
    // Scenario- 2000-3000  and 4000-5000 have 2 common, but we take the first (2000-3000)
    //        1000 2000 3000 4000 5000
    // chan101  X
    // chan102       X    X
    // chan103       X    X    X    X
    // chan104                 X    X
    //

    // Set corr integration time
    let corr_int_time_ms = 2000;

    // Setup variables to generate gpuboxtimemap
    let coarse_chan101_timesteps: Vec<u64> = vec![1000];
    let coarse_chan102_timesteps: Vec<u64> = vec![2000, 3000];
    let coarse_chan103_timesteps: Vec<u64> = vec![2000, 3000, 4000, 5000];
    let coarse_chan104_timesteps: Vec<u64> = vec![4000, 5000];

    // Get a populated GpuboxTimeMap
    let gpubox_time_map = create_determine_common_obs_times_and_chans_test_data(
        coarse_chan101_timesteps,
        coarse_chan102_timesteps,
        coarse_chan103_timesteps,
        coarse_chan104_timesteps,
    );

    // Actually run our test!
    let result = determine_common_obs_times_and_chans(&gpubox_time_map, corr_int_time_ms, None);

    // Check we did not encounter an error
    assert!(result.is_ok());

    // Unwrap to get the obstimesandchans struct
    let o = result.unwrap();

    // Check contents of common timesteps and coarse channels is what is expected
    assert_eq!(o.end_time_unix_ms - o.start_time_unix_ms, o.duration_ms);
    assert_eq!(o.start_time_unix_ms, 2000);
    assert_eq!(o.end_time_unix_ms, 4000);
    assert_eq!(o.duration_ms, 2000);
    assert_eq!(o.coarse_chan_identifiers, vec![102, 103]);

    //
    // Now run the same test, but with a good time of 3000
    //
    // Actually run our test!
    let result_good =
        determine_common_obs_times_and_chans(&gpubox_time_map, corr_int_time_ms, Some(3000));

    // Check we did not encounter an error
    assert!(result_good.is_ok());

    // Unwrap to get the obstimesandchans struct
    let o_good = result_good.unwrap();

    // Check contents is what is expected
    assert_eq!(
        o_good.end_time_unix_ms - o_good.start_time_unix_ms,
        o_good.duration_ms
    );
    assert_eq!(o_good.start_time_unix_ms, 3000);
    assert_eq!(o_good.end_time_unix_ms, 4000, "{:?}", o_good);
    assert_eq!(o_good.duration_ms, 1000);
    assert_eq!(o_good.coarse_chan_identifiers, vec![102, 103]);
}

#[test]
fn test_determine_common_obs_times_and_chans_two_then_three() {
    // Scenario- there is a run of 2 chans (1000-2000) then a different set of 2 chans for 1 ts then 3 chans in a 3 timestep run (4000,5000,6000).
    //        1000 2000 3000 4000 5000 6000
    // chan101 X     X         X    X    X
    // chan102 X     X         X    X    X
    // chan103            X    X    X    X
    // chan104            X
    //

    // Set corr integration time
    let corr_int_time_ms = 1000;

    // Setup variables to generate gpuboxtimemap
    let coarse_chan101_timesteps: Vec<u64> = vec![1000, 2000, 4000, 5000, 6000];
    let coarse_chan102_timesteps: Vec<u64> = vec![1000, 2000, 4000, 5000, 6000];
    let coarse_chan103_timesteps: Vec<u64> = vec![3000, 4000, 5000, 6000];
    let coarse_chan104_timesteps: Vec<u64> = vec![3000];

    // Get a populated GpuboxTimeMap
    let gpubox_time_map = create_determine_common_obs_times_and_chans_test_data(
        coarse_chan101_timesteps,
        coarse_chan102_timesteps,
        coarse_chan103_timesteps,
        coarse_chan104_timesteps,
    );

    // Actually run our test!
    let result = determine_common_obs_times_and_chans(&gpubox_time_map, corr_int_time_ms, None);

    // Check we did not encounter an error
    assert!(result.is_ok());

    // Unwrap to get the obstimesandchans struct
    let o = result.unwrap();

    // Check contents of common timesteps and coarse channels is what is expected
    assert_eq!(o.end_time_unix_ms - o.start_time_unix_ms, o.duration_ms);
    assert_eq!(o.start_time_unix_ms, 4000);
    assert_eq!(o.end_time_unix_ms, 7000);
    assert_eq!(o.duration_ms, 3000);
    assert_eq!(o.coarse_chan_identifiers, vec![101, 102, 103]);

    //
    // Now run the same test, but with a good time
    //
    // Actually run our test!
    let result_good =
        determine_common_obs_times_and_chans(&gpubox_time_map, corr_int_time_ms, None);

    // Check we did not encounter an error
    assert!(result_good.is_ok());

    // Unwrap to get the obstimesandchans struct
    let o_good = result_good.unwrap();

    // Check contents is what is expected
    assert_eq!(
        o_good.end_time_unix_ms - o_good.start_time_unix_ms,
        o_good.duration_ms
    );
    assert_eq!(o_good.start_time_unix_ms, 4000);
    assert_eq!(o_good.end_time_unix_ms, 7000);
    assert_eq!(o_good.duration_ms, 3000);
    assert_eq!(o_good.coarse_chan_identifiers, vec![101, 102, 103]);
}

#[test]
fn test_determine_common_obs_times_and_chans_non_contiguous() {
    // Scenario- there is a run of 2 chans (1000-2000) then a different set of 2 chans for 1 ts then 3 chans in a 1 timestep run (4000), a gap (5000), then 3 chans for (6000).
    //        1000 2000 3000 4000 5000 6000
    // chan101 X     X                   X
    // chan102 X     X         X         X
    // chan103            X    X         X
    // chan104            X    X
    //

    // Set corr integration time
    let corr_int_time_ms = 1000;

    // Setup variables to generate gpuboxtimemap
    let coarse_chan101_timesteps: Vec<u64> = vec![1000, 2000, 5000, 6000];
    let coarse_chan102_timesteps: Vec<u64> = vec![1000, 2000, 4000, 5000, 6000];
    let coarse_chan103_timesteps: Vec<u64> = vec![3000, 4000, 5000, 6000];
    let coarse_chan104_timesteps: Vec<u64> = vec![3000, 4000];

    // Get a populated GpuboxTimeMap
    let gpubox_time_map = create_determine_common_obs_times_and_chans_test_data(
        coarse_chan101_timesteps,
        coarse_chan102_timesteps,
        coarse_chan103_timesteps,
        coarse_chan104_timesteps,
    );

    // Actually run our test!
    let result = determine_common_obs_times_and_chans(&gpubox_time_map, corr_int_time_ms, None);

    // Check we did not encounter an error
    assert!(result.is_ok());

    // Unwrap to get the obstimesandchans struct
    let o = result.unwrap();

    // Check contents of common timesteps and coarse channels is what is expected
    assert_eq!(o.end_time_unix_ms - o.start_time_unix_ms, o.duration_ms);
    assert_eq!(o.start_time_unix_ms, 4000);
    assert_eq!(o.end_time_unix_ms, 5000);
    assert_eq!(o.duration_ms, 1000);
    assert_eq!(o.coarse_chan_identifiers, vec![102, 103, 104]);

    //
    // Now run the same test, but with a good time
    //
    // Actually run our test!
    let result_good =
        determine_common_obs_times_and_chans(&gpubox_time_map, corr_int_time_ms, None);

    // Check we did not encounter an error
    assert!(result_good.is_ok());

    // Unwrap to get the obstimesandchans struct
    let o_good = result_good.unwrap();

    // Check contents is what is expected
    assert_eq!(
        o_good.end_time_unix_ms - o_good.start_time_unix_ms,
        o_good.duration_ms
    );
    assert_eq!(o_good.start_time_unix_ms, 4000);
    assert_eq!(o_good.end_time_unix_ms, 5000);
    assert_eq!(o_good.duration_ms, 1000);
    assert_eq!(o_good.coarse_chan_identifiers, vec![102, 103, 104]);
}
