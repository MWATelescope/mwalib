// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Unit tests for voltage context
*/
#[cfg(test)]
use super::*;
use std::fs::File;
use std::io::{Error, Write};

/// Helper fuction to generate (small) test voltage files
#[cfg(test)]
fn generate_test_voltage_file(
    temp_dir: &tempdir::TempDir,
    filename: &str,
    time_samples: usize,
    rf_inputs: usize,
) -> Result<String, Error> {
    let tdir_path = temp_dir.path();
    let full_filename = tdir_path.join(filename);

    let mut output_file = File::create(&full_filename)?;
    // Write out x time samples
    // Each sample has x rfinputs
    // and 1 float for real 1 float for imaginary
    let floats = time_samples * rf_inputs * 2;
    let mut buffer: Vec<f32> = vec![0.0; floats];

    let mut bptr: usize = 0;

    // This will write out the sequence:
    // 0.25, 0.75, 1.25, 1.75..511.25,511.75  (1024 floats in all)
    for t in 0..time_samples {
        for r in 0..rf_inputs {
            // real
            buffer[bptr] = ((t * rf_inputs) + r) as f32 + 0.25;
            bptr += 1;
            // imag
            buffer[bptr] = ((t * rf_inputs) + r) as f32 + 0.75;
            bptr += 1;
        }
    }
    output_file.write_all(misc::as_u8_slice(buffer.as_slice()))?;
    output_file.flush()?;

    Ok(String::from(full_filename.to_str().unwrap()))
}

#[test]
fn test_context_new_missing_voltage_files() {
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let voltagefiles = Vec::new();

    // No gpubox files provided
    let context = VoltageContext::new(&metafits_filename, &voltagefiles);
    assert!(matches!(
        context.unwrap_err(),
        MwalibError::Voltage(VoltageFileError::NoVoltageFiles)
    ));
}

#[test]
fn test_context_new_invalid_metafits() {
    let metafits_filename = "invalid.metafits";
    let filename = "test_files/1101503312_1_timestep/1101503312_1101503312_ch123.dat";
    let voltage_files = vec![filename];

    // No gpubox files provided
    let context = VoltageContext::new(&metafits_filename, &voltage_files);

    assert!(context.is_err());
}

#[test]
fn test_context_legacy_v1() {
    // Open the test mwax file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    // Create some test files
    // Create a temp dir for the temp files
    // Once out of scope the temp dir and it's contents will be deleted
    let temp_dir = tempdir::TempDir::new("voltage_test").unwrap();

    // Populate vector of filenames
    let mut temp_filenames: Vec<&str> = Vec::new();
    let tvf1 =
        generate_test_voltage_file(&temp_dir, "1101503312_1101503312_ch123.dat", 2, 256).unwrap();
    temp_filenames.push(&tvf1);
    let tvf2 =
        generate_test_voltage_file(&temp_dir, "1101503312_1101503312_ch124.dat", 2, 256).unwrap();
    temp_filenames.push(&tvf2);
    let tvf3 =
        generate_test_voltage_file(&temp_dir, "1101503312_1101503313_ch123.dat", 2, 256).unwrap();
    temp_filenames.push(&tvf3);
    let tvf4 =
        generate_test_voltage_file(&temp_dir, "1101503312_1101503313_ch124.dat", 2, 256).unwrap();
    temp_filenames.push(&tvf4);

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let context = VoltageContext::new(&metafits_filename, &temp_filenames)
        .expect("Failed to create VoltageContext");

    // Test the properties of the context object match what we expect
    // Correlator version:       v1 Legacy,
    assert_eq!(context.corr_version, CorrelatorVersion::Legacy);

    // Actual gps start time:   1_101_503_312,
    assert_eq!(context.start_gps_time_ms, 1_101_503_312_000);

    // Actual gps end time:     1_101_503_314,
    assert_eq!(context.end_gps_time_ms, 1_101_503_314_000);

    // Actual duration:          2 s,
    assert_eq!(context.duration_ms, 2_000);

    // num timesteps:            2,
    assert_eq!(context.num_timesteps, 2);

    // timesteps:
    assert_eq!(context.timesteps[0].gps_time_ms, 1_101_503_312_000);
    assert_eq!(context.timesteps[1].gps_time_ms, 1_101_503_313_000);

    // num coarse channels,      2,
    assert_eq!(context.num_coarse_chans, 2);

    // observation bandwidth:    2.56 MHz,
    assert_eq!(context.bandwidth_hz, 1_280_000 * 2);

    // coarse channels:
    assert_eq!(context.coarse_chans[0].rec_chan_number, 123);
    assert_eq!(context.coarse_chans[0].chan_centre_hz, 157_440_000);
    assert_eq!(context.coarse_chans[1].rec_chan_number, 124);
    assert_eq!(context.coarse_chans[1].chan_centre_hz, 158_720_000);
    // fine channel resolution:  10 kHz,
    assert_eq!(context.fine_chan_width_hz, 10_000);
    // num fine channels/coarse: 128,
    assert_eq!(context.num_fine_chans_per_coarse, 128);
    assert_eq!(context.voltage_batches.len(), 2);
}

#[test]
fn test_context_mwax_v2() {
    // Open the test mwax file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    // Create some test files
    // Create a temp dir for the temp files
    // Once out of scope the temp dir and it's contents will be deleted
    let temp_dir = tempdir::TempDir::new("voltage_test").unwrap();

    // Populate vector of filenames
    let mut temp_filenames: Vec<&str> = Vec::new();
    let tvf1 =
        generate_test_voltage_file(&temp_dir, "1101503312_1101503312_123.sub", 2, 256).unwrap();
    temp_filenames.push(&tvf1);
    let tvf2 =
        generate_test_voltage_file(&temp_dir, "1101503312_1101503312_124.sub", 2, 256).unwrap();
    temp_filenames.push(&tvf2);
    let tvf3 =
        generate_test_voltage_file(&temp_dir, "1101503312_1101503320_123.sub", 2, 256).unwrap();
    temp_filenames.push(&tvf3);
    let tvf4 =
        generate_test_voltage_file(&temp_dir, "1101503312_1101503320_124.sub", 2, 256).unwrap();
    temp_filenames.push(&tvf4);

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let context = VoltageContext::new(&metafits_filename, &temp_filenames)
        .expect("Failed to create VoltageContext");

    // Test the properties of the context object match what we expect
    // Correlator version:       v2 mwax,
    assert_eq!(context.corr_version, CorrelatorVersion::V2);

    // Actual gps start time:   1_101_503_312,
    assert_eq!(context.start_gps_time_ms, 1_101_503_312_000);

    // Actual gps end time:     1_101_503_328,
    assert_eq!(context.end_gps_time_ms, 1_101_503_328_000);

    // Actual duration:          16 s,
    assert_eq!(context.duration_ms, 16_000);

    // num timesteps:            2,
    assert_eq!(context.num_timesteps, 2);

    // timesteps:
    assert_eq!(context.timesteps[0].gps_time_ms, 1_101_503_312_000);
    assert_eq!(context.timesteps[1].gps_time_ms, 1_101_503_320_000);

    // num coarse channels,      2,
    assert_eq!(context.num_coarse_chans, 2);

    // observation bandwidth:    2.56 MHz,
    assert_eq!(context.bandwidth_hz, 1_280_000 * 2);

    // coarse channels:
    assert_eq!(context.coarse_chans[0].rec_chan_number, 123);
    assert_eq!(context.coarse_chans[0].chan_centre_hz, 157_440_000);
    assert_eq!(context.coarse_chans[1].rec_chan_number, 124);
    assert_eq!(context.coarse_chans[1].chan_centre_hz, 158_720_000);
    // fine channel resolution:  1.28 MHz,
    assert_eq!(context.fine_chan_width_hz, 1_280_000);
    // num fine channels/coarse: 1,
    assert_eq!(context.num_fine_chans_per_coarse, 1);
    assert_eq!(context.voltage_batches.len(), 2);
}
