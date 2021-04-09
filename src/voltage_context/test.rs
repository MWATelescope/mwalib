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
use std::path::Path;

/// Helper fuctions to generate (small-sh) test voltage files
/// for mwax test files they contain an incrememting byte for the real in each samples and decrementing byte value for the imag value.
/// for legacy test files they contain a single incrememnting byte for the real/imag value.
#[cfg(test)]
#[allow(clippy::clippy::too_many_arguments)]
fn generate_test_voltage_file(
    temp_dir: &tempdir::TempDir,
    filename: &str,
    header_bytes: usize,
    delay_block_bytes: usize,
    num_voltage_blocks: usize,
    samples_per_block: usize,
    rf_inputs: usize,
    fine_chans: usize,
    bytes_per_sample: usize,
    initial_value: u8,
) -> Result<String, Error> {
    let tdir_path = temp_dir.path();
    let full_filename = tdir_path.join(filename);

    let mut output_file = File::create(&full_filename)?;

    // Write out header if one is needed
    if header_bytes > 0 {
        let header_buffer: Vec<u8> = vec![0x01; header_bytes];
        output_file
            .write_all(&header_buffer)
            .expect("Cannot write header!");
    }

    // Write out delay block if one is needed
    if delay_block_bytes > 0 {
        let delay_buffer: Vec<u8> = vec![0x02; delay_block_bytes];
        output_file
            .write_all(&delay_buffer)
            .expect("Cannot write delay block!");
    }

    // Write out num_voltage_blocks
    //
    // Each voltage_block has samples_per_rf_fine for each combination of rfinputs x fine_chans
    // and 1 float for real 1 float for imaginary per second
    let num_bytes_per_voltage_block = samples_per_block * rf_inputs * fine_chans * bytes_per_sample;

    // Loop for each voltage block
    // legacy: 50 blocks per file
    // mwaxx : 160 blocks per file
    for _ in 0..num_voltage_blocks {
        let mut value1: u8 = initial_value;
        let mut value2: u8 = u8::MAX - initial_value;

        // Allocate a buffer
        let mut voltage_block_buffer: Vec<u8> = vec![0; num_bytes_per_voltage_block];

        // Populate the buffer with test data
        let mut bptr: usize = 0; // Keeps track of where we are in the byte array

        // each rfinput/finechan within a voltage block has n contiguous samples
        // rfinputs=2 (ant 0 X and ant 0 Y)
        // for legacy: fine_chans=128, samples_per_block=200
        // for mwax  : fine_chans=1  , samples_per_block=64000
        for _ in 0..rf_inputs {
            for _ in 0..fine_chans {
                for _ in 0..samples_per_block {
                    match bytes_per_sample {
                        2 => {
                            // Byte 1
                            voltage_block_buffer[bptr] = value1;
                            bptr += 1;
                            // Byte 2
                            voltage_block_buffer[bptr] = value2;
                            bptr += 1;
                        }
                        1 => {
                            // In this case 1 byte is split into 4bits real and 4bits imag
                            voltage_block_buffer[bptr] = value1;
                            bptr += 1;
                        }
                        _ => panic!("Wrong bytes per sample!"),
                    }

                    // Increment/decrement values
                    value1 = match value1 == u8::MAX {
                        true => u8::MIN,
                        false => value1 + 1,
                    };
                    value2 = match value2 == u8::MIN {
                        true => u8::MAX,
                        false => value2 - 1,
                    };
                }
            }
        }
        output_file
            .write_all(&voltage_block_buffer)
            .expect("Cannot write voltage data block");
    }

    output_file.flush()?;

    Ok(String::from(full_filename.to_str().unwrap()))
}

#[cfg(test)]
fn generate_test_voltage_file_legacy_recombined(
    temp_dir: &tempdir::TempDir,
    filename: &str,
    initial_value: u8,
) -> Result<String, Error> {
    // Note we are only producing data for 2 rfinputs (ant0 X and ant0 Y)
    // The initial value is used to differentiate different timesteps and coarse channels
    generate_test_voltage_file(temp_dir, filename, 0, 0, 50, 200, 2, 128, 1, initial_value)
}

#[cfg(test)]
fn get_index_for_location_in_test_voltage_file(
    context: &VoltageContext,
    voltage_block_index: usize,
    rfinput_index: usize,
    fine_chan_index: usize,
    sample_index: usize,
    value_index: usize,
) -> usize {
    let num_rfinputs = 2;

    let bytes_per_fine_chan =
        context.num_samples_per_voltage_block as usize * context.sample_size_bytes as usize;

    let bytes_per_rfinput = context.num_fine_chans_per_coarse * bytes_per_fine_chan;

    let bytes_per_voltage_block = num_rfinputs * bytes_per_rfinput;

    // This will position us at the correct block
    let vb = voltage_block_index * bytes_per_voltage_block;

    // Now within the block, move to the correct rf_input
    let rf = rfinput_index * bytes_per_rfinput;

    // Now within the rfinput, move to the correct fine channel
    let fc = fine_chan_index * bytes_per_fine_chan;

    // Return the correct index
    vb + rf + fc + (sample_index * context.sample_size_bytes as usize) + value_index
}

#[cfg(test)]
fn generate_test_voltage_file_mwax(
    temp_dir: &tempdir::TempDir,
    filename: &str,
    initial_value: u8,
) -> Result<String, Error> {
    // Note we are only producing data for 2 rfinputs (ant0 X and ant0 Y)
    // The initial value is used to differentiate different timesteps and coarse channels
    generate_test_voltage_file(
        temp_dir,
        filename,
        4096,
        32_768_000,
        160,
        64_000,
        2,
        1,
        2,
        initial_value,
    )
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
    let tvf1 = generate_test_voltage_file_legacy_recombined(
        &temp_dir,
        "1101503312_1101503312_ch123.dat",
        0,
    )
    .unwrap();
    temp_filenames.push(&tvf1);
    let tvf2 = generate_test_voltage_file_legacy_recombined(
        &temp_dir,
        "1101503312_1101503312_ch124.dat",
        1,
    )
    .unwrap();
    temp_filenames.push(&tvf2);
    let tvf3 = generate_test_voltage_file_legacy_recombined(
        &temp_dir,
        "1101503312_1101503313_ch123.dat",
        2,
    )
    .unwrap();
    temp_filenames.push(&tvf3);
    let tvf4 = generate_test_voltage_file_legacy_recombined(
        &temp_dir,
        "1101503312_1101503313_ch124.dat",
        3,
    )
    .unwrap();
    temp_filenames.push(&tvf4);

    // Copy the files to /tmp (TempDir will delete them once they are out of scope)
    for f in &temp_filenames {
        let filename = Path::new(&f).file_name().unwrap().to_str().unwrap();
        std::fs::copy(f, format!("/tmp/{}", filename)).expect("Error copying files to /tmp");
    }

    // Check the files are the right size
    // Obtain metadata
    let metadata = std::fs::metadata(&temp_filenames[0]).expect("unable to read metadata");

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let mut context = VoltageContext::new(&metafits_filename, &temp_filenames)
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

    // Number of bytes in each sample
    assert_eq!(context.sample_size_bytes, 1);
    // Number of voltage blocks per timestep
    assert_eq!(context.num_voltage_blocks_per_timestep, 50);
    // Number of voltage blocks of samples in each second of data
    assert_eq!(context.num_voltage_blocks_per_second, 50);
    // Number of samples in each voltage_blocks for each second of data per rf_input * fine_chans * real|imag
    assert_eq!(context.num_samples_per_voltage_block, 200);
    // The size of each voltage block
    assert_eq!(context.voltage_block_size_bytes, 6_553_600);
    // Number of bytes used to store delays - for MWAX this is the same as a voltage block size, for legacy it is 0
    assert_eq!(context.delay_block_size_bytes, 0);
    // The amount of bytes to skip before getting into real data within the voltage files
    assert_eq!(context.data_file_header_size_bytes, 0);
    // Expected voltage file size
    assert_eq!(context.expected_voltage_data_file_size_bytes, 327_680_000);

    assert_eq!(context.voltage_batches.len(), 2);

    //
    // In order for our smaller voltage files to work with this test we need to reset the voltage_block_size_bytes
    //
    context.voltage_block_size_bytes /= 128;

    // Also check our test file is the right size!
    // Note our test file only has 2 rfinputs, not 256!
    assert_eq!(
        metadata.len(),
        context.voltage_block_size_bytes * context.num_voltage_blocks_per_timestep
    );

    //
    // Now do a read of the data from time 0, channel 0
    //
    let read_result: Result<Vec<u8>, VoltageFileError> = context.read(0, 0);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Unwrap the data
    let read_data = read_result.expect("Error reading data");

    assert_eq!(
        read_data.len() as u64,
        context.voltage_block_size_bytes * context.num_voltage_blocks_per_timestep
    ); // (block size * blocks)

    // Check for various values
    // block: 0, rfinput: 0, fine_chan: 0, sample: 0, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 0, 0)],
        0
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 1, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 1, 0)],
        1
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 255, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 255, 0)],
        255
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 256, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 256, 0)],
        0
    );

    // block: 1, rfinput: 0, fine_chan: 0, sample: 2, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 1, 0, 0, 2, 0)],
        2
    );

    // block: 49, rfinput: 1, fine_chan: 127, sample: 199, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 49, 1, 127, 199, 0)],
        255
    );

    //
    // Now do a read of the data from time 0, channel 1. Values are offset by +1 from time 0, chan 0.
    //
    let read_result: Result<Vec<u8>, VoltageFileError> = context.read(0, 1);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Unwrap the data
    let read_data = read_result.expect("Error reading data");

    // Check for various values
    // block: 0, rfinput: 0, fine_chan: 0, sample: 0, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 0, 0)],
        1
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 1, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 1, 0)],
        2
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 255, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 255, 0)],
        0
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 256, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 256, 0)],
        1
    );

    // block: 49, rfinput: 1, fine_chan: 127, sample: 199, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 49, 1, 127, 199, 0)],
        0
    );

    //
    // Now do a read of the data from time 1, channel 0. Values are offset by +2 from time 0, chan 0.
    //
    let read_result: Result<Vec<u8>, VoltageFileError> = context.read(1, 0);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Unwrap the data
    let read_data = read_result.expect("Error reading data");

    // Check for various values
    // block: 0, rfinput: 0, fine_chan: 0, sample: 0, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 0, 0)],
        2
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 1, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 1, 0)],
        3
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 255, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 255, 0)],
        1
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 256, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 256, 0)],
        2
    );

    // block: 49, rfinput: 1, fine_chan: 127, sample: 199, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 49, 1, 127, 199, 0)],
        1
    );

    //
    // Now do a read of the data from time 1, channel 1. Values are offset by +3 from time 0, chan 0.
    //
    let read_result: Result<Vec<u8>, VoltageFileError> = context.read(1, 1);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Unwrap the data
    let read_data = read_result.expect("Error reading data");

    // Check for various values
    // block: 0, rfinput: 0, fine_chan: 0, sample: 0, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 0, 0)],
        3
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 1, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 1, 0)],
        4
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 255, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 255, 0)],
        2
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 256, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 256, 0)],
        3
    );

    // block: 49, rfinput: 1, fine_chan: 127, sample: 199, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 49, 1, 127, 199, 0)],
        2
    );
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
        generate_test_voltage_file_mwax(&temp_dir, "1101503312_1101503312_123.sub", 0).unwrap();
    temp_filenames.push(&tvf1);
    let tvf2 =
        generate_test_voltage_file_mwax(&temp_dir, "1101503312_1101503312_124.sub", 1).unwrap();
    temp_filenames.push(&tvf2);
    let tvf3 =
        generate_test_voltage_file_mwax(&temp_dir, "1101503312_1101503320_123.sub", 2).unwrap();
    temp_filenames.push(&tvf3);
    let tvf4 =
        generate_test_voltage_file_mwax(&temp_dir, "1101503312_1101503320_124.sub", 3).unwrap();
    temp_filenames.push(&tvf4);

    // Copy the files to /tmp (TempDir will delete them once they are out of scope)
    for f in &temp_filenames {
        let filename = Path::new(&f).file_name().unwrap().to_str().unwrap();
        std::fs::copy(f, format!("/tmp/{}", filename)).expect("Error copying files to /tmp");
    }

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let mut context = VoltageContext::new(&metafits_filename, &temp_filenames)
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
    // Number of bytes in each sample
    assert_eq!(context.sample_size_bytes, 2);
    // Number of voltage blocks per timestep
    assert_eq!(context.num_voltage_blocks_per_timestep, 160);
    // Number of voltage blocks of samples in each second of data
    assert_eq!(context.num_voltage_blocks_per_second, 20);
    // Number of samples in each voltage_blocks for each second of data per rf_input * fine_chans * real|imag
    assert_eq!(context.num_samples_per_voltage_block, 64_000);
    // The size of each voltage block
    assert_eq!(context.voltage_block_size_bytes, 32_768_000);
    // Number of bytes used to store delays - for MWAX this is the same as a voltage block size, for legacy it is 0
    assert_eq!(
        context.delay_block_size_bytes,
        context.voltage_block_size_bytes
    );
    // The amount of bytes to skip before getting into real data within the voltage files
    assert_eq!(context.data_file_header_size_bytes, 4096);
    // Expected voltage file size
    assert_eq!(context.expected_voltage_data_file_size_bytes, 5_275_652_096);
    // Check number of batches
    assert_eq!(context.voltage_batches.len(), 2);

    // Check the files are the right size
    // Obtain metadata
    let metadata = std::fs::metadata(&temp_filenames[0]).expect("unable to read metadata");

    //
    // In order for our smaller voltage files to work with this test we need to reset the voltage_block_size_bytes
    //
    context.voltage_block_size_bytes /= 128;

    // Also check our test file is the right size!
    // Note our test file only has 2 rfinputs, not 256!
    assert_eq!(
        metadata.len(),
        context.data_file_header_size_bytes
            + context.delay_block_size_bytes
            + (context.voltage_block_size_bytes * context.num_voltage_blocks_per_timestep)
    );

    //
    // Now do a read of the data from time 0, channel 0
    //
    let read_result: Result<Vec<u8>, VoltageFileError> = context.read(0, 0);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Unwrap the data
    let read_data = read_result.expect("Error reading data");

    assert_eq!(
        read_data.len() as u64,
        context.voltage_block_size_bytes * context.num_voltage_blocks_per_timestep
    ); // block size * blocks

    // Check for various values
    // block: 0, rfinput: 0, fine_chan: 0, sample: 0, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 0, 0)],
        0
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 1, value: 1
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 1, 1)],
        254
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 255, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 255, 0)],
        255
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 256, value: 1
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 256, 1)],
        255
    );

    // block: 1, rfinput: 0, fine_chan: 0, sample: 2, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 1, 0, 0, 2, 0)],
        2
    );

    // block: 159, rfinput: 1, fine_chan: 0, sample: 63999, value: 1
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 159, 1, 0, 63999, 1)],
        0
    );

    //
    // Now do a read of the data from time 0, channel 1. Values are offset by +1 from time 0, chan 0.
    //
    let read_result: Result<Vec<u8>, VoltageFileError> = context.read(0, 1);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Unwrap the data
    let read_data = read_result.expect("Error reading data");

    // Check for various values
    // block: 0, rfinput: 0, fine_chan: 0, sample: 0, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 0, 0)],
        1
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 1, value: 1
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 1, 1)],
        253
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 255, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 255, 0)],
        0
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 256, value: 1
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 256, 1)],
        254
    );

    // block: 1, rfinput: 0, fine_chan: 0, sample: 2, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 1, 0, 0, 2, 0)],
        3
    );

    // block: 159, rfinput: 1, fine_chan: 0, sample: 63999, value: 1
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 159, 1, 0, 63999, 1)],
        255
    );

    //
    // Now do a read of the data from time 1, channel 0. Values are offset by +2 from time 0, chan 0.
    //
    let read_result: Result<Vec<u8>, VoltageFileError> = context.read(1, 0);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Unwrap the data
    let read_data = read_result.expect("Error reading data");

    // Check for various values
    // block: 0, rfinput: 0, fine_chan: 0, sample: 0, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 0, 0)],
        2
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 1, value: 1
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 1, 1)],
        252
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 255, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 255, 0)],
        1
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 256, value: 1
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 256, 1)],
        253
    );

    // block: 1, rfinput: 0, fine_chan: 0, sample: 2, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 1, 0, 0, 2, 0)],
        4
    );

    // block: 159, rfinput: 1, fine_chan: 0, sample: 63999, value: 1
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 159, 1, 0, 63999, 1)],
        254
    );

    //
    // Now do a read of the data from time 1, channel 1. Values are offset by +3 from time 0, chan 0.
    //
    let read_result: Result<Vec<u8>, VoltageFileError> = context.read(1, 1);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Unwrap the data
    let read_data = read_result.expect("Error reading data");

    // Check for various values
    // block: 0, rfinput: 0, fine_chan: 0, sample: 0, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 0, 0)],
        3
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 1, value: 1
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 1, 1)],
        251
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 255, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 255, 0)],
        2
    );

    // block: 0, rfinput: 0, fine_chan: 0, sample: 256, value: 1
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 0, 0, 0, 256, 1)],
        252
    );

    // block: 1, rfinput: 0, fine_chan: 0, sample: 2, value: 0
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 1, 0, 0, 2, 0)],
        5
    );

    // block: 159, rfinput: 1, fine_chan: 0, sample: 63999, value: 1
    assert_eq!(
        read_data[get_index_for_location_in_test_voltage_file(&context, 159, 1, 0, 63999, 1)],
        253
    );
}
