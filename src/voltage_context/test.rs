// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Unit tests for voltage context
use super::*;
use float_cmp::*;
use misc::generate_test_voltage_file;
use std::io::Error;
use std::path::PathBuf;
use std::sync::Once;

// Define three static "once" variables to control creation of VCS test data (so it only happens once, the first time it's needed)
pub(crate) static VCS_LEGACY_TEST_DATA_CREATED: Once = Once::new();
pub(crate) static VCS_MWAXV2_TEST_DATA_CREATED: Once = Once::new();
pub(crate) static VCS_MWAXV2_OS_TEST_DATA_CREATED: Once = Once::new();

pub(crate) fn generate_test_voltage_file_legacy_recombined(
    filename: &str,
    initial_value: u8,
) -> Result<String, Error> {
    // Note we are only producing data for 2 rfinputs (ant0 X and ant0 Y)
    // The initial value is used to differentiate different timesteps and coarse channels
    generate_test_voltage_file(
        filename,
        MWAVersion::VCSLegacyRecombined,
        1,
        10000,
        2,
        128,
        1,
        initial_value,
    )
}

pub(crate) fn generate_test_voltage_file_mwax(
    filename: &str,
    initial_value: u8,
) -> Result<String, Error> {
    // Note we are only producing data for 2 rfinputs (ant0 X and ant0 Y)
    // The initial value is used to differentiate different timesteps and coarse channels
    generate_test_voltage_file(
        filename,
        MWAVersion::VCSMWAXv2,
        160,
        64_000,
        2,
        1,
        2,
        initial_value,
    )
}

pub(crate) fn generate_test_voltage_file_mwax_os(
    filename: &str,
    initial_value: u8,
) -> Result<String, Error> {
    // Note we are only producing data for 2 rfinputs (ant0 X and ant0 Y)
    // The initial value is used to differentiate different timesteps and coarse channels
    generate_test_voltage_file(
        filename,
        MWAVersion::VCSMWAXv2,
        160,
        81_920,
        2,
        1,
        2,
        initial_value,
    )
}

pub(crate) fn get_test_voltage_files(mwa_version: MWAVersion, oversampled: bool) -> Vec<String> {
    // Create some test files
    // Populate vector of filenames
    let test_filenames: Vec<String>;

    match mwa_version {
        MWAVersion::VCSMWAXv2 => {
            match oversampled {
                true => {
                    // Now for the oversampled case
                    test_filenames = vec![
                        String::from(
                            "test_files/1370755832_mwax_vcs_os/1370755832_1370755832_123.sub",
                        ),
                        String::from(
                            "test_files/1370755832_mwax_vcs_os/1370755832_1370755832_124.sub",
                        ),
                        String::from(
                            "test_files/1370755832_mwax_vcs_os/1370755832_1370755840_123.sub",
                        ),
                        String::from(
                            "test_files/1370755832_mwax_vcs_os/1370755832_1370755840_124.sub",
                        ),
                    ];

                    // This ensure the test data is created once only
                    VCS_MWAXV2_OS_TEST_DATA_CREATED.call_once(|| {
                        // Create this test data, but only once!
                        for (i, f) in test_filenames.iter().enumerate() {
                            generate_test_voltage_file_mwax_os(f, i as u8).unwrap();
                        }
                    });
                }
                false => {
                    test_filenames = vec![
                        String::from(
                            "test_files/1101503312_mwax_vcs/1101503312_1101503312_123.sub",
                        ),
                        String::from(
                            "test_files/1101503312_mwax_vcs/1101503312_1101503312_124.sub",
                        ),
                        String::from(
                            "test_files/1101503312_mwax_vcs/1101503312_1101503320_123.sub",
                        ),
                        String::from(
                            "test_files/1101503312_mwax_vcs/1101503312_1101503320_124.sub",
                        ),
                    ];

                    // This ensure the test data is created once only
                    VCS_MWAXV2_TEST_DATA_CREATED.call_once(|| {
                        // Create this test data, but only once!
                        for (i, f) in test_filenames.iter().enumerate() {
                            generate_test_voltage_file_mwax(f, i as u8).unwrap();
                        }
                    });
                }
            }
        }
        MWAVersion::VCSLegacyRecombined => {
            test_filenames = vec![
                String::from("test_files/1101503312_vcs/1101503312_1101503312_ch123.dat"),
                String::from("test_files/1101503312_vcs/1101503312_1101503312_ch124.dat"),
                String::from("test_files/1101503312_vcs/1101503312_1101503313_ch123.dat"),
                String::from("test_files/1101503312_vcs/1101503312_1101503313_ch124.dat"),
            ];

            // This ensure the test data is created once only
            VCS_LEGACY_TEST_DATA_CREATED.call_once(|| {
                for (i, f) in test_filenames.iter().enumerate() {
                    generate_test_voltage_file_legacy_recombined(f, i as u8).unwrap();
                }
            });
        }
        _ => {
            panic!("Other mwa_version values are not supported for VCS");
        }
    }

    test_filenames
}

pub(crate) fn get_index_for_location_in_test_voltage_file_legacy(
    sample_index: usize,
    fine_chan_index: usize,
    rfinput_index: usize,
) -> usize {
    let num_rfinputs = 2;

    // Note for legacy always only have 1 block (i.e. no concept of a block)
    let bytes_per_rfinput = 1;

    let bytes_per_fine_chan = bytes_per_rfinput * num_rfinputs;

    let bytes_per_sample = bytes_per_fine_chan * 128;

    // Sample is the slowest moving axis
    let s = sample_index * bytes_per_sample;

    // Now within the sample, move to the correct fine chan
    let fc = fine_chan_index * bytes_per_fine_chan;

    // Now within the fine channel get the rf_input
    let rf = rfinput_index * bytes_per_rfinput;

    // Return the correct index
    s + rf + fc
}

pub(crate) fn get_index_for_location_in_test_voltage_file_mwaxv2(
    voltage_block_index: usize,
    rfinput_index: usize,
    sample_index: usize,
    value_index: usize,
) -> usize {
    let num_finechan = 1;
    let num_rfinputs = 2;

    let bytes_per_fine_chan = 64000 * 2;

    let bytes_per_rfinput = num_finechan * bytes_per_fine_chan;

    let bytes_per_voltage_block = num_rfinputs * bytes_per_rfinput;

    // This will position us at the correct block
    let vb = voltage_block_index * bytes_per_voltage_block;

    // Now within the block, move to the correct rf_input
    let rf = rfinput_index * bytes_per_rfinput;

    // Return the correct index
    vb + rf + (sample_index * 2) + value_index
}

pub(crate) fn get_index_for_location_in_test_voltage_file_mwaxv2_os(
    voltage_block_index: usize,
    rfinput_index: usize,
    sample_index: usize,
    value_index: usize,
) -> usize {
    let num_finechan = 1;
    let num_rfinputs = 2;

    let bytes_per_fine_chan = 81920 * 2;

    let bytes_per_rfinput = num_finechan * bytes_per_fine_chan;

    let bytes_per_voltage_block = num_rfinputs * bytes_per_rfinput;

    // This will position us at the correct block
    let vb = voltage_block_index * bytes_per_voltage_block;

    // Now within the block, move to the correct rf_input
    let rf = rfinput_index * bytes_per_rfinput;

    // Return the correct index
    vb + rf + (sample_index * 2) + value_index
}

pub(crate) fn get_test_voltage_context(
    mwa_version: MWAVersion,
    oversampled: bool,
) -> VoltageContext {
    // Open the test metafits file
    let metafits_filename = match mwa_version {
        MWAVersion::VCSMWAXv2 => match oversampled {
            true => "test_files/1370755832_mwax_vcs_os/1370755832_metafits.fits",
            false => "test_files/1101503312_mwax_vcs/1101503312.metafits",
        },
        MWAVersion::VCSLegacyRecombined => "test_files/1101503312_vcs/1101503312.metafits",
        _ => "",
    };

    // Create some test files
    // Populate vector of filenames
    let generated_filenames = get_test_voltage_files(mwa_version, oversampled);

    let temp_strings = generated_filenames.iter().map(String::as_str);
    let test_filenames: Vec<&str> = temp_strings.collect();

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let context = VoltageContext::new(metafits_filename, &test_filenames)
        .expect("Failed to create VoltageContext");

    // Also check our test file is the right size!
    // Note our test file only has 2 rfinputs, not 256!
    // Check the files are the right size
    // Obtain metadata
    let metadata = std::fs::metadata(test_filenames[0]).expect("unable to read metadata");

    // Also check our test file is the right size!
    assert_eq!(
        metadata.len(),
        context.data_file_header_size_bytes
            + context.delay_block_size_bytes
            + (context.voltage_block_size_bytes * context.num_voltage_blocks_per_timestep as u64),
        "mwa_v={} header={} + delay={}  + (voltage_block={} * vbs per ts={} * finech={})",
        context.mwa_version,
        context.data_file_header_size_bytes,
        context.delay_block_size_bytes,
        context.voltage_block_size_bytes,
        context.num_voltage_blocks_per_timestep,
        context.num_fine_chans_per_coarse
    );

    context
}

#[test]
fn test_2_digit_channel_in_subfiles() {
    let metafits_filename = "test_files/1380365160/1380365160_metafits.fits";
    generate_test_voltage_file_mwax("test_files/1380365160/1380365160_1380365160_65.sub", 0)
        .unwrap();
    let filename = "test_files/1380365160/1380365160_1380365160_65.sub";
    let voltage_files = vec![filename];

    let context = VoltageContext::new(metafits_filename, &voltage_files).unwrap();
    assert!(context.num_provided_coarse_chans == 1);
    assert!(context.provided_coarse_chan_indices[0] == 3);
    assert!(context.coarse_chans[3].rec_chan_number == 65);
}

#[test]
fn test_2_digit_channel_with_leading_zero_in_subfiles() {
    let metafits_filename = "test_files/1380365160/1380365160_metafits.fits";
    generate_test_voltage_file_mwax("test_files/1380365160/1380365160_1380365160_065.sub", 0)
        .unwrap();
    let filename = "test_files/1380365160/1380365160_1380365160_065.sub";
    let voltage_files = vec![filename];
    let context = VoltageContext::new(metafits_filename, &voltage_files).unwrap();
    assert!(context.num_provided_coarse_chans == 1);
    assert!(context.provided_coarse_chan_indices[0] == 3);
    assert!(context.coarse_chans[3].rec_chan_number == 65);
}

#[test]
fn test_context_new_missing_voltage_files() {
    let metafits_filename = "test_files/1101503312_vcs/1101503312.metafits";
    let voltagefiles: Vec<PathBuf> = Vec::new();

    // No gpubox files provided
    let context = VoltageContext::new(metafits_filename, &voltagefiles);
    assert!(matches!(
        context.unwrap_err(),
        MwalibError::Voltage(VoltageFileError::NoVoltageFiles)
    ));
}

#[test]
fn test_context_new_invalid_metafits() {
    let metafits_filename = "invalid.metafits";
    let filename = "test_files/1101503312_vcs/1101503312_1101503312_ch123.dat";
    let voltage_files = vec![filename];

    // No gpubox files provided
    let context = VoltageContext::new(metafits_filename, &voltage_files);

    assert!(context.is_err());
}

#[test]
fn test_context_legacy_v1() {
    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits and gpubox file
    let context = get_test_voltage_context(MWAVersion::VCSLegacyRecombined, false);

    // Test the properties of the context object match what we expect
    // MWA version:       v1 Legacy,
    assert_eq!(context.mwa_version, MWAVersion::VCSLegacyRecombined);

    // Actual gps start time:   1_101_503_312,
    assert_eq!(context.common_start_gps_time_ms, 1_101_503_312_000);

    // Actual gps end time:     1_101_503_314,
    assert_eq!(context.common_end_gps_time_ms, 1_101_503_314_000);

    // Actual duration:          2 s,
    assert_eq!(context.common_duration_ms, 2_000);

    // num timesteps:            112,
    assert_eq!(context.num_timesteps, 112);

    // timesteps:
    assert_eq!(context.timesteps[0].gps_time_ms, 1_101_503_312_000);
    assert_eq!(context.timesteps[1].gps_time_ms, 1_101_503_313_000);
    assert_eq!(context.timesteps[111].gps_time_ms, 1_101_503_423_000);

    // num coarse channels,      24,
    assert_eq!(context.num_coarse_chans, 24);

    // observation bandwidth:    2.56 MHz,
    assert_eq!(context.common_bandwidth_hz, 1_280_000 * 2);

    // coarse channels:
    assert_eq!(context.coarse_chans[14].rec_chan_number, 123);
    assert_eq!(context.coarse_chans[14].chan_centre_hz, 157_440_000);
    assert_eq!(context.coarse_chans[15].rec_chan_number, 124);
    assert_eq!(context.coarse_chans[15].chan_centre_hz, 158_720_000);
    // fine channel resolution:  10 kHz,
    assert_eq!(context.fine_chan_width_hz, 10_000);
    // num fine channels/coarse: 128,
    assert_eq!(context.num_fine_chans_per_coarse, 128);

    // Number of bytes in each sample
    assert_eq!(context.sample_size_bytes, 1);
    // Number of voltage blocks per timestep
    assert_eq!(context.num_voltage_blocks_per_timestep, 1);
    // Number of voltage blocks of samples in each second of data
    assert_eq!(context.num_voltage_blocks_per_second, 1);
    // Number of samples in each voltage_blocks for each second of data per rf_input * fine_chans * real|imag
    assert_eq!(context.num_samples_per_voltage_block, 10_000);
    // The size of each voltage block
    assert_eq!(context.voltage_block_size_bytes, 2_560_000);
    // Number of bytes used to store delays - for MWAX this is the same as a voltage block size, for legacy it is 0
    assert_eq!(context.delay_block_size_bytes, 0);
    // The amount of bytes to skip before getting into real data within the voltage files
    assert_eq!(context.data_file_header_size_bytes, 0);
    // Expected voltage file size
    assert_eq!(context.expected_voltage_data_file_size_bytes, 2_560_000);
    // Check batches
    assert_eq!(context.voltage_batches.len(), 2);

    // Check for num of fine chan freqs
    assert_eq!(
        context.metafits_context.num_metafits_fine_chan_freqs,
        24 * 128
    );
    assert_eq!(
        context.metafits_context.num_metafits_fine_chan_freqs,
        context.metafits_context.metafits_fine_chan_freqs_hz.len()
    );

    // Check rfinput order (for Legacy it is vcs_order, mwax is subfile_order)
    let mut rf_input_copy = context.metafits_context.rf_inputs.clone();
    rf_input_copy.sort_by_key(|k| k.vcs_order);
    // Now compare this copy with the 'real' rf_inputs
    assert_eq!(&rf_input_copy, &context.metafits_context.rf_inputs);
    // Ensure the antenna->rf_input mapping is still in tact
    assert_eq!(context.metafits_context.antennas[0].rfinput_x.vcs_order, 93);
    assert_eq!(context.metafits_context.antennas[0].rfinput_y.vcs_order, 89);
}

#[test]
fn test_context_legacy_v1_128_tiles() {
    // Create some test files
    // Populate vector of filenames
    let generated_filenames = get_test_voltage_files(MWAVersion::VCSLegacyRecombined, false);

    let temp_strings = generated_filenames.iter().map(String::as_str);
    let test_filenames: Vec<&str> = temp_strings.collect();

    // Open a context and load in a test metafits
    let context = VoltageContext::new(
        "test_files/1101503312_vcs/1101503312.metafits128",
        &test_filenames,
    )
    .expect("Failed to create VoltageContext");

    // Ensure the antenna->rf_input mapping is still in tact (voltage context reorders them from the metafits)
    for i in 0..128 {
        if context.metafits_context.antennas[i].tile_id == 154 {
            assert_eq!(context.metafits_context.antennas[i].rfinput_y.vcs_order, 1);
        }

        if context.metafits_context.antennas[i].tile_id == 104 {
            assert_eq!(context.metafits_context.antennas[i].rfinput_y.vcs_order, 0);
        }
    }
}

#[test]
fn test_context_legacy_v1_read_file_no_data_for_timestep() {
    // Open a context and load in a test metafits and gpubox file
    let context = get_test_voltage_context(MWAVersion::VCSLegacyRecombined, false);

    //
    // Now do a read of the data from time 0, channel 0
    //
    // Create output buffer
    let mut buffer: Vec<i8> = vec![
        0;
        context.voltage_block_size_bytes as usize
            * context.num_voltage_blocks_per_timestep
    ];
    // No data for timestep index 10
    let read_result: Result<(), VoltageFileError> = context.read_file(10, 14, &mut buffer);

    // Ensure read is err
    assert!(read_result.is_err());

    let error = read_result.unwrap_err();
    assert!(
        matches!(
            error,
            VoltageFileError::NoDataForTimeStepCoarseChannel {
                timestep_index: 10,
                coarse_chan_index: 14
            }
        ),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_context_legacy_v1_read_file() {
    // Open a context and load in a test metafits and gpubox file
    let context = get_test_voltage_context(MWAVersion::VCSLegacyRecombined, false);

    //
    // Now do a read of the data from time 0, channel 0
    //
    // Create output buffer
    let mut buffer: Vec<i8> = vec![
        0;
        context.voltage_block_size_bytes as usize
            * context.num_voltage_blocks_per_timestep
    ];
    let read_result: Result<(), VoltageFileError> = context.read_file(0, 14, &mut buffer);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Check for various values
    // sample: 0, fine_chan: 0, rfinput: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 0, 0)],
        0
    );

    // sample: 0, fine_chan: 0, rfinput: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 0, 1)],
        2
    );

    // sample: 0, fine_chan: 1, rfinput: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 1, 1)],
        5
    );

    // sample: 0, fine_chan: 127, rfinput: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 127, 0)],
        125
    );

    // sample: 10, fine_chan: 32, rfinput: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(10, 32, 1)],
        -118
    );

    // sample: 9999, fine_chan: 127, rfinput: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(9999, 127, 1)],
        -69
    );

    //
    // Now do a read of the data from time 0, channel 1. Values are offset by +1 from time 0, chan 0.
    //
    let read_result: Result<(), VoltageFileError> = context.read_file(0, 15, &mut buffer);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Check for various values
    // sample: 0, fine_chan: 0, rfinput: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 0, 0)],
        1
    );

    // sample: 0, fine_chan: 0, rfinput: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 0, 1)],
        3
    );

    // sample: 0, fine_chan: 1, rfinput: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 1, 1)],
        6
    );

    // sample: 0, fine_chan: 127, rfinput: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 127, 0)],
        126
    );

    // sample: 10, fine_chan: 32, rfinput: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(10, 32, 1)],
        -117
    );

    // sample: 9999, fine_chan: 127, rfinput: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(9999, 127, 1)],
        -68
    );

    //
    // Now do a read of the data from time 1, channel 0. Values are offset by +2 from time 0, chan 0.
    //
    let read_result: Result<(), VoltageFileError> = context.read_file(1, 14, &mut buffer);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Check for various values
    // sample: 0, fine_chan: 0, rfinput: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 0, 0)],
        2
    );

    // sample: 0, fine_chan: 0, rfinput: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 0, 1)],
        4
    );

    // sample: 0, fine_chan: 1, rfinput: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 1, 1)],
        7
    );

    // sample: 0, fine_chan: 127, rfinput: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 127, 0)],
        127
    );

    // sample: 10, fine_chan: 32, rfinput: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(10, 32, 1)],
        -116
    );

    // sample: 9999, fine_chan: 127, rfinput: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(9999, 127, 1)],
        -67
    );

    //
    // Now do a read of the data from time 1, channel 1. Values are offset by +3 from time 0, chan 0.
    //
    let read_result: Result<(), VoltageFileError> = context.read_file(1, 15, &mut buffer);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Check for various values
    // sample: 0, fine_chan: 0, rfinput: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 0, 0)],
        3
    );

    // sample: 0, fine_chan: 0, rfinput: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 0, 1)],
        5
    );

    // sample: 0, fine_chan: 1, rfinput: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 1, 1)],
        8
    );

    // sample: 0, fine_chan: 127, rfinput: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 127, 0)],
        -128
    );

    // sample: 10, fine_chan: 32, rfinput: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(10, 32, 1)],
        -115
    );

    // sample: 9999, fine_chan: 127, rfinput: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(9999, 127, 1)],
        -66
    );
}

#[test]
fn test_context_mwax_v2() {
    // Create voltage context
    let context = get_test_voltage_context(MWAVersion::VCSMWAXv2, false);

    // Test the properties of the context object match what we expect
    // MWA version:       v2 mwax,
    assert_eq!(context.mwa_version, MWAVersion::VCSMWAXv2);

    // Actual gps start time:   1_101_503_312,
    assert_eq!(context.common_start_gps_time_ms, 1_101_503_312_000);

    // Actual gps end time:     1_101_503_328,
    assert_eq!(context.common_end_gps_time_ms, 1_101_503_328_000);

    // Actual duration:          16 s,
    assert_eq!(context.common_duration_ms, 16_000);

    // num timesteps:            14,  (what the metafits says)
    assert_eq!(context.num_timesteps, 14);

    // timesteps:
    assert_eq!(context.timesteps[0].gps_time_ms, 1_101_503_312_000);
    assert_eq!(context.timesteps[1].gps_time_ms, 1_101_503_320_000);
    assert_eq!(context.timesteps[13].gps_time_ms, 1_101_503_416_000);

    // num coarse channels,      2,
    assert_eq!(context.num_coarse_chans, 24);

    // observation bandwidth:    2.56 MHz,
    assert_eq!(context.common_bandwidth_hz, 1_280_000 * 2);

    // coarse channels:
    assert_eq!(context.coarse_chans[14].rec_chan_number, 123);
    assert_eq!(context.coarse_chans[14].chan_centre_hz, 157_440_000);
    assert_eq!(context.coarse_chans[15].rec_chan_number, 124);
    assert_eq!(context.coarse_chans[15].chan_centre_hz, 158_720_000);
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
    assert_eq!(context.voltage_block_size_bytes, 256_000);
    // Number of bytes used to store delays - for MWAX this is the same as a voltage block size, for legacy it is 0
    assert_eq!(
        context.delay_block_size_bytes,
        context.voltage_block_size_bytes
    );
    // The amount of bytes to skip before getting into real data within the voltage files
    assert_eq!(context.data_file_header_size_bytes, 4096);
    // Expected voltage file size
    assert_eq!(context.expected_voltage_data_file_size_bytes, 41_220_096);
    // Check number of batches
    assert_eq!(context.voltage_batches.len(), 2);

    // Check rfinput order (for Legacy it is vcs_order, mwax is subfile_order)
    let mut rf_input_copy = context.metafits_context.rf_inputs.clone();
    rf_input_copy.sort_by_key(|k| k.subfile_order);
    // Now compare this copy with the 'real' rf_inputs
    assert_eq!(&rf_input_copy, &context.metafits_context.rf_inputs);
}

#[test]
fn test_context_mwaxv2_read_file_no_data_for_timestep() {
    // Open a context and load in a test metafits and gpubox file
    let context = get_test_voltage_context(MWAVersion::VCSMWAXv2, false);

    //
    // Now do a read of the data from time 0, channel 0
    //
    // Create output buffer
    let mut buffer: Vec<i8> = vec![
        0;
        context.voltage_block_size_bytes as usize
            * context.num_voltage_blocks_per_timestep
    ];
    // No data for timestep index 10
    let read_result: Result<(), VoltageFileError> = context.read_file(10, 14, &mut buffer);

    // Ensure read is err
    assert!(read_result.is_err());

    let error = read_result.unwrap_err();
    assert!(
        matches!(
            error,
            VoltageFileError::NoDataForTimeStepCoarseChannel {
                timestep_index: 10,
                coarse_chan_index: 14
            }
        ),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_context_mwax_v2_read_file() {
    // Test reading from an critically sampled mwaxv2 file

    // Create voltage context
    let context = get_test_voltage_context(MWAVersion::VCSMWAXv2, false);

    // Create output buffer
    let mut buffer: Vec<i8> = vec![
        0;
        context.voltage_block_size_bytes as usize
            * context.num_voltage_blocks_per_timestep
    ];

    //
    // Now do a read of the data from time 0, channel 0
    //
    let read_result: Result<(), VoltageFileError> = context.read_file(0, 14, &mut buffer);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Check for various values
    // block: 0, rfinput: 0, sample: 0, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 0, 0)],
        0
    );

    // block: 0, rfinput: 0, sample: 1, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 1, 1)],
        -3
    );

    // block: 0, rfinput: 0, sample: 255, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 255, 0)],
        -2
    );

    // block: 0, rfinput: 0, sample: 256, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 256, 1)],
        -1
    );

    // block: 1, rfinput: 0, sample: 2, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(1, 0, 2, 0)],
        9
    );

    // block: 159, rfinput: 1, sample: 63999, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(159, 1, 63999, 1)],
        -30
    );

    // block: 120, rfinput: 0, sample: 0, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(120, 0, 0, 0)],
        88
    );

    //
    // Now do a read of the data from time 0, channel 1. Values are offset by +1 from time 0, chan 0.
    //
    let read_result: Result<(), VoltageFileError> = context.read_file(0, 15, &mut buffer);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Check for various values
    // block: 0, rfinput: 0, sample: 0, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 0, 0)],
        1
    );

    // block: 0, rfinput: 0, sample: 1, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 1, 1)],
        -4
    );

    // block: 0, rfinput: 0, sample: 255, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 255, 0)],
        -1
    );

    // block: 0, rfinput: 0, sample: 256, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 256, 1)],
        -2
    );

    // block: 1, rfinput: 0, sample: 2, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(1, 0, 2, 0)],
        10
    );

    // block: 159, rfinput: 1, sample: 63999, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(159, 1, 63999, 1)],
        -31
    );

    //
    // Now do a read of the data from time 1, channel 0. Values are offset by +2 from time 0, chan 0.
    //
    let read_result: Result<(), VoltageFileError> = context.read_file(1, 14, &mut buffer);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Check for various values
    // block: 0, rfinput: 0, sample: 0, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 0, 0)],
        2
    );

    // block: 0, rfinput: 0, sample: 1, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 1, 1)],
        -5
    );

    // block: 0, rfinput: 0, sample: 255, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 255, 0)],
        0
    );

    // block: 0, rfinput: 0, sample: 256, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 256, 1)],
        -3
    );

    // block: 1, rfinput: 0, sample: 2, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(1, 0, 2, 0)],
        11
    );

    // block: 159, rfinput: 1, sample: 63999, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(159, 1, 63999, 1)],
        -32
    );

    //
    // Now do a read of the data from time 1, channel 1. Values are offset by +3 from time 0, chan 0.
    //
    let read_result: Result<(), VoltageFileError> = context.read_file(1, 15, &mut buffer);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Check for various values
    // block: 0, rfinput: 0, sample: 0, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 0, 0)],
        3
    );

    // block: 0, rfinput: 0, sample: 1, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 1, 1)],
        -6
    );

    // block: 0, rfinput: 0, sample: 255, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 255, 0)],
        1
    );

    // block: 0, rfinput: 0, sample: 256, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 256, 1)],
        -4
    );

    // block: 1, rfinput: 0, sample: 2, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(1, 0, 2, 0)],
        12
    );

    // block: 159, rfinput: 1, sample: 63999, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(159, 1, 63999, 1)],
        -33
    );
}

#[test]
fn test_context_mwax_v2_oversampled_read_file() {
    // Test reading from an oversampled mwaxv2 file

    // Create voltage context
    let context = get_test_voltage_context(MWAVersion::VCSMWAXv2, true);

    // Create output buffer
    let mut buffer: Vec<i8> = vec![
        0;
        context.voltage_block_size_bytes as usize
            * context.num_voltage_blocks_per_timestep
    ];

    //
    // Now do a read of the data from time 0, channel 0
    //
    let read_result: Result<(), VoltageFileError> = context.read_file(0, 14, &mut buffer);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Check for various values
    // block: 0, rfinput: 0, sample: 0, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 0, 0)],
        0
    );

    // block: 0, rfinput: 0, sample: 1, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 1, 1)],
        -3
    );

    // block: 0, rfinput: 0, sample: 255, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 255, 0)],
        -2
    );

    // block: 0, rfinput: 0, sample: 256, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 256, 1)],
        -1
    );

    // block: 1, rfinput: 0, sample: 2, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(1, 0, 2, 0)],
        9
    );

    // block: 159, rfinput: 1, sample: 63999, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(159, 1, 63999, 1)],
        -30
    );

    // block: 159, rfinput: 1, sample: 81919, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(159, 1, 81919, 1)],
        -30
    );

    // block: 120, rfinput: 0, sample: 0, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(120, 0, 0, 0)],
        88
    );

    //
    // Now do a read of the data from time 0, channel 1. Values are offset by +1 from time 0, chan 0.
    //
    let read_result: Result<(), VoltageFileError> = context.read_file(0, 15, &mut buffer);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Check for various values
    // block: 0, rfinput: 0, sample: 0, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 0, 0)],
        1
    );

    // block: 0, rfinput: 0, sample: 1, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 1, 1)],
        -4
    );

    // block: 0, rfinput: 0, sample: 255, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 255, 0)],
        -1
    );

    // block: 0, rfinput: 0, sample: 256, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 256, 1)],
        -2
    );

    // block: 1, rfinput: 0, sample: 2, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(1, 0, 2, 0)],
        10
    );

    // block: 159, rfinput: 1, sample: 63999, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(159, 1, 63999, 1)],
        -31
    );

    // block: 159, rfinput: 1, sample: 81919, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(159, 1, 81919, 1)],
        -31
    );

    //
    // Now do a read of the data from time 1, channel 0. Values are offset by +2 from time 0, chan 0.
    //
    let read_result: Result<(), VoltageFileError> = context.read_file(1, 14, &mut buffer);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Check for various values
    // block: 0, rfinput: 0, sample: 0, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 0, 0)],
        2
    );

    // block: 0, rfinput: 0, sample: 1, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 1, 1)],
        -5
    );

    // block: 0, rfinput: 0, sample: 255, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 255, 0)],
        0
    );

    // block: 0, rfinput: 0, sample: 256, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 256, 1)],
        -3
    );

    // block: 1, rfinput: 0, sample: 2, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(1, 0, 2, 0)],
        11
    );

    // block: 159, rfinput: 1, sample: 63999, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(159, 1, 63999, 1)],
        -32
    );

    // block: 159, rfinput: 1, sample: 81919, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(159, 1, 81919, 1)],
        -32
    );

    //
    // Now do a read of the data from time 1, channel 1. Values are offset by +3 from time 0, chan 0.
    //
    let read_result: Result<(), VoltageFileError> = context.read_file(1, 15, &mut buffer);

    // Ensure read is ok
    assert!(read_result.is_ok());

    // Check for various values
    // block: 0, rfinput: 0, sample: 0, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 0, 0)],
        3
    );

    // block: 0, rfinput: 0, sample: 1, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 1, 1)],
        -6
    );

    // block: 0, rfinput: 0, sample: 255, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 255, 0)],
        1
    );

    // block: 0, rfinput: 0, sample: 256, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(0, 0, 256, 1)],
        -4
    );

    // block: 1, rfinput: 0, sample: 2, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(1, 0, 2, 0)],
        12
    );

    // block: 159, rfinput: 1, sample: 63999, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(159, 1, 63999, 1)],
        -33
    );

    // block: 159, rfinput: 1, sample: 81919, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2_os(159, 1, 81919, 1)],
        -33
    );
}

#[test]
fn test_validate_gps_time_parameters_legacy() {
    // Create test files and a test Voltage Context
    let context = get_test_voltage_context(MWAVersion::VCSLegacyRecombined, false);

    let result = VoltageContext::validate_gps_time_parameters(&context, 1_101_503_312, 1);

    assert!(result.is_ok(), "Result was {:?}", result);

    assert_eq!(result.unwrap(), 1_101_503_312);
}

#[test]
fn test_validate_gps_time_parameters_mwax_v2() {
    // Create test files and a test Voltage Context
    let context = get_test_voltage_context(MWAVersion::VCSMWAXv2, false);

    let result = VoltageContext::validate_gps_time_parameters(&context, 1_101_503_312, 10);

    assert!(result.is_ok(), "Result was {:?}", result);

    assert_eq!(result.unwrap(), 1_101_503_321);
}

#[test]
fn test_validate_gps_time_parameters_invalid_gps_second_start_legacy() {
    // Create test files and a test Voltage Context
    let context = get_test_voltage_context(MWAVersion::VCSLegacyRecombined, false);

    let result = VoltageContext::validate_gps_time_parameters(&context, 1_101_503_311, 1);

    assert!(result.is_err(), "{:?}", result);

    let error = result.unwrap_err();
    assert!(
        matches!(
            error,
            VoltageFileError::InvalidGpsSecondStart(1_101_503_312, 1_101_503_424)
        ),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_validate_gps_time_parameters_invalid_gps_second_count_legacy() {
    // This test obs starts at 1_101_503_312 and has 112 seconds.
    // Create test files and a test Voltage Context
    let context = get_test_voltage_context(MWAVersion::VCSLegacyRecombined, false);

    let result = VoltageContext::validate_gps_time_parameters(&context, 1_101_503_424, 3);

    assert!(result.is_err(), "{:?}", result);

    let error = result.unwrap_err();
    assert!(
        matches!(
            error,
            VoltageFileError::InvalidGpsSecondCount(1_101_503_312, 3, 1_101_503_424)
        ),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_validate_gps_time_parameters_invalid_gps_second_start_mwax_v2() {
    // Create test files and a test Voltage Context
    let context = get_test_voltage_context(MWAVersion::VCSMWAXv2, false);

    let result = VoltageContext::validate_gps_time_parameters(&context, 1_101_503_311, 1);

    assert!(result.is_err(), "{:?}", result);

    let error = result.unwrap_err();
    assert!(
        matches!(
            error,
            VoltageFileError::InvalidGpsSecondStart(1_101_503_312, 1_101_503_424)
        ),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_validate_gps_time_parameters_invalid_gps_second_count_mwax_v2() {
    // Create test files and a test Voltage Context
    let context = get_test_voltage_context(MWAVersion::VCSMWAXv2, false);

    let result = VoltageContext::validate_gps_time_parameters(&context, 1_101_503_312, 118);

    assert!(result.is_err(), "{:?}", result);

    let error = result.unwrap_err();
    assert!(
        matches!(
            error,
            VoltageFileError::InvalidGpsSecondCount(1_101_503_312, 118, 1_101_503_424)
        ),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_context_read_second_invalid_coarse_chan_index() {
    // Open a context and load in a test metafits and gpubox file
    let context = get_test_voltage_context(MWAVersion::VCSLegacyRecombined, false);

    // Create output buffer
    let mut buffer: Vec<i8> = vec![
        0;
        context.voltage_block_size_bytes as usize
            * context.num_voltage_blocks_per_timestep
    ];

    // Do the read
    let gps_second_start = 1_101_503_312;
    let gps_second_count = 1;
    let coarse_chan_index = 100;

    let read_result: Result<(), VoltageFileError> = context.read_second(
        gps_second_start,
        gps_second_count,
        coarse_chan_index,
        &mut buffer,
    );

    // Ensure read returns correct error
    assert!(read_result.is_err(), "{:?}", read_result);

    let error = read_result.unwrap_err();
    assert!(
        matches!(error, VoltageFileError::InvalidCoarseChanIndex(23)),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_context_read_second_invalid_buffer_size() {
    // Open a context and load in a test metafits and gpubox file
    let context = get_test_voltage_context(MWAVersion::VCSLegacyRecombined, false);

    let gps_second_start = 1_101_503_312;
    let gps_second_count: usize = 1;
    let coarse_chan_index = 0;

    // Create output buffer
    // NOTE we are sabotaging this to generate our error, by dividing by 2
    let mut buffer: Vec<i8> = vec![
        0;
        (context.voltage_block_size_bytes
            * context.num_voltage_blocks_per_second as u64
            * gps_second_count as u64
            / 2) as usize
    ];

    // Do the read
    let read_result: Result<(), VoltageFileError> = context.read_second(
        gps_second_start,
        gps_second_count,
        coarse_chan_index,
        &mut buffer,
    );

    // Ensure read returns correct error
    assert!(read_result.is_err(), "{:?}", read_result);

    let error = read_result.unwrap_err();
    assert!(
        matches!(
            error,
            VoltageFileError::InvalidBufferSize(1_280_000, 2_560_000)
        ),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_context_read_second_legacy_invalid_data_file_size() {
    // Open a context and load in a test metafits and gpubox file
    let mut context = get_test_voltage_context(MWAVersion::VCSLegacyRecombined, false);

    // Alter the context values so we generate an invalid file size error
    context.expected_voltage_data_file_size_bytes += 1;

    let gps_second_start = 1_101_503_312;
    let gps_second_count: usize = 1;
    let coarse_chan_index = 14;

    //
    // Now do a read of the data
    //
    // Create output buffer
    let mut buffer: Vec<i8> = vec![
        0;
        (context.voltage_block_size_bytes
            * context.num_voltage_blocks_per_second as u64
            * gps_second_count as u64) as usize
    ];

    // Do the read
    let read_result: Result<(), VoltageFileError> = context.read_second(
        gps_second_start,
        gps_second_count,
        coarse_chan_index,
        &mut buffer,
    );

    // Ensure read returns correct error
    assert!(read_result.is_err(), "{:?}", read_result);

    let error = read_result.unwrap_err();
    assert!(
        matches!(error, VoltageFileError::InvalidVoltageFileSize(_, _, _)),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_context_read_second_mwaxv2_invalid_data_file_size() {
    // Open a context and load in a test metafits and gpubox file
    let mut context = get_test_voltage_context(MWAVersion::VCSMWAXv2, false);

    // Alter the context values so we generate an invalid file size error
    context.expected_voltage_data_file_size_bytes += 1;

    let gps_second_start = 1_101_503_312;
    let gps_second_count: usize = 1;
    let coarse_chan_index = 14;

    //
    // Now do a read of the data
    //
    // Create output buffer
    let mut buffer: Vec<i8> = vec![
        0;
        (context.voltage_block_size_bytes
            * context.num_voltage_blocks_per_second as u64
            * gps_second_count as u64) as usize
    ];

    // Do the read
    let read_result: Result<(), VoltageFileError> = context.read_second(
        gps_second_start,
        gps_second_count,
        coarse_chan_index,
        &mut buffer,
    );

    // Ensure read returns correct error
    assert!(read_result.is_err(), "{:?}", read_result,);

    let error = read_result.unwrap_err();
    assert!(
        matches!(error, VoltageFileError::InvalidVoltageFileSize(_, _, _)),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_context_read_second_legacy_no_data_for_gpstime() {
    // Open a context and load in a test metafits and gpubox file
    let context = get_test_voltage_context(MWAVersion::VCSLegacyRecombined, false);

    let gps_second_start = 1_101_503_350; // No data at this timestep
    let gps_second_count: usize = 1;
    let coarse_chan_index = 14;

    //
    // Now do a read of the data
    //
    // Create output buffer
    let mut buffer: Vec<i8> = vec![
        0;
        (context.voltage_block_size_bytes
            * context.num_voltage_blocks_per_second as u64
            * gps_second_count as u64) as usize
    ];

    // Do the read
    let read_result: Result<(), VoltageFileError> = context.read_second(
        gps_second_start,
        gps_second_count,
        coarse_chan_index,
        &mut buffer,
    );

    // Ensure read returns correct error
    assert!(read_result.is_err(), "{:?}", read_result);

    let error = read_result.unwrap_err();
    assert!(
        matches!(
            error,
            VoltageFileError::NoDataForTimeStepCoarseChannel {
                timestep_index: _,
                coarse_chan_index: _
            }
        ),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_context_read_second_mwaxv2_no_data_for_gpstime() {
    // Open a context and load in a test metafits and gpubox file
    let context = get_test_voltage_context(MWAVersion::VCSMWAXv2, false);

    let gps_second_start = 1_101_503_350; // No data at this timestep
    let gps_second_count: usize = 1;
    let coarse_chan_index = 14;

    //
    // Now do a read of the data
    //
    // Create output buffer
    let mut buffer: Vec<i8> = vec![
        0;
        (context.voltage_block_size_bytes
            * context.num_voltage_blocks_per_second as u64
            * gps_second_count as u64) as usize
    ];

    // Do the read
    let read_result: Result<(), VoltageFileError> = context.read_second(
        gps_second_start,
        gps_second_count,
        coarse_chan_index,
        &mut buffer,
    );

    // Ensure read returns correct error
    assert!(read_result.is_err(), "{:?}", read_result);

    let error = read_result.unwrap_err();
    assert!(
        matches!(
            error,
            VoltageFileError::NoDataForTimeStepCoarseChannel {
                timestep_index: _,
                coarse_chan_index: _
            }
        ),
        "Error was {:?}",
        error
    );
}

#[test]
fn test_context_read_second_legacyv1_valid() {
    //
    // We will test reading across 2 data files in time
    // file 0 has gps seconds 1_101_503_312 - 1_101_503_313
    // file 1 has gps seconds 1_101_503_313 - 1_101_503_314
    //
    // We will read the 1 sec from file 0 and the 1 sec from file 1
    // which is 1_101_503_312, 1_101_503_313

    // Open a context and load in a test metafits and gpubox file
    let context = get_test_voltage_context(MWAVersion::VCSLegacyRecombined, false);

    let gps_second_start = 1_101_503_312;
    let gps_second_count: usize = 2;
    let coarse_chan_index = 14;

    //
    // Now do a read of the data
    //
    // Create output buffer
    let mut buffer: Vec<i8> = vec![
        0;
        (context.voltage_block_size_bytes
            * context.num_voltage_blocks_per_second as u64
            * gps_second_count as u64) as usize
    ];

    // Do the read
    let read_result: Result<(), VoltageFileError> = context.read_second(
        gps_second_start,
        gps_second_count,
        coarse_chan_index,
        &mut buffer,
    );

    // Ensure read returns correct error
    assert!(read_result.is_ok(), "{:?}", read_result);

    // Check values
    // Sample: 0, fine_chan: 0, rfinput: 0
    // Second 1_101_503_312
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(0, 0, 0)],
        0
    );

    // Second 1_101_503_313 data is offset by +2
    assert_eq!(
        buffer[context.voltage_block_size_bytes as usize * context.num_voltage_blocks_per_second
            + get_index_for_location_in_test_voltage_file_legacy(0, 0, 0)],
        2
    );

    // Sample: 1000, fine_chan: 13, rfinput: 1
    // Second 1_101_503_312
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_legacy(1000, 13, 1)],
        -55
    );

    // Second 1_101_503_313 data is offset by +2
    assert_eq!(
        buffer[context.voltage_block_size_bytes as usize * context.num_voltage_blocks_per_second
            + get_index_for_location_in_test_voltage_file_legacy(1000, 13, 1)],
        -53
    );
}

#[test]
fn test_context_read_second_mwaxv2_valid() {
    //
    // We will test reading across 2 data files in time
    // file 0 has gps seconds 1_101_503_312 - 1_101_503_319
    // file 1 has gps seconds 1_101_503_320 - 1_101_503_327
    //
    // We will read the last 2 secs from file 0 and the first 2 secs from file 1
    // which is 1_101_503_318, 1_101_503_319, 1_101_503_320, 1_101_503_321

    // Open a context and load in a test metafits and gpubox file
    let context = get_test_voltage_context(MWAVersion::VCSMWAXv2, false);

    let gps_second_start = 1_101_503_318;
    let gps_second_count: usize = 4;
    let coarse_chan_index = 14;

    //
    // Now do a read of the data
    //
    // Create output buffer
    let mut buffer: Vec<i8> = vec![
        0;
        (context.voltage_block_size_bytes
            * context.num_voltage_blocks_per_second as u64
            * gps_second_count as u64) as usize
    ];

    // Do the read
    let read_result: Result<(), VoltageFileError> = context.read_second(
        gps_second_start,
        gps_second_count,
        coarse_chan_index,
        &mut buffer,
    );

    // Ensure read returns correct error
    assert!(read_result.is_ok(), "{:?}", read_result);

    // Check values
    //
    // Second 1_101_503_318
    //
    // location in buffer: block: 0, rfinput: 0, sample: 0, value: 0
    // location in file0:  block: 120, rfinput: 0, sample: 0, value: 0
    //
    // (this is the 120th block / 7th second of the 8 second block in the FILE, but the first block of the buffer)
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 0, 0)],
        88
    );

    // Second 1_101_503_319
    // location in buffer: block: 20, rfinput: 0, sample: 0, value: 0
    // location in file0:  block: 140, rfinput: 0, sample: 0, value: 0
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(20, 0, 0, 0)],
        -68
    );

    // Second 1_101_503_320 (now we are in a new data file so the values are incrememented by 2 from the first file)
    // location in buffer: block: 40+0, rfinput: 0, sample: 0, value: 0
    // location in file1:  block: 0, rfinput: 0, sample: 0, value: 0
    assert_eq!(
        buffer[2
            * context.voltage_block_size_bytes as usize
            * context.num_voltage_blocks_per_second
            + get_index_for_location_in_test_voltage_file_mwaxv2(0, 0, 0, 0)],
        2
    );

    // Second 1_101_503_321 (now we are in a new data file so the values are incrememented by 2 from the first file)
    // location in buffer: block: 40+20, rfinput: 0, sample: 0, value: 0
    // location in file1:  block: 20, rfinput: 0, sample: 0, value: 0
    assert_eq!(
        buffer[2
            * context.voltage_block_size_bytes as usize
            * context.num_voltage_blocks_per_second
            + get_index_for_location_in_test_voltage_file_mwaxv2(20, 0, 0, 0)],
        102
    );

    // Second 1_101_503_318 (this is the 7th block / 7th second of the 8 second block in the FILE, but the first block of the buffer)
    // location in buffer: block: 0, rfinput: 1, sample: 16750, value: 1
    // location in file0:  block: 120, rfinput: 1, sample: 16750, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(0, 1, 16750, 1)],
        -57,
    );

    // Second 1_101_503_319
    // location in buffer: block: 20, rfinput: 1, sample: 16750, value: 1
    // location in file0:  block: 140, rfinput: 1, sample: 16750, value: 1
    assert_eq!(
        buffer[get_index_for_location_in_test_voltage_file_mwaxv2(20, 1, 16750, 1)],
        99
    );

    // Second 1_101_503_320 (now we are in a new data file so the values are incrememented by 2 from the first file)
    // location in buffer: block: 40+0, rfinput: 1, sample: 16750, value: 1
    // location in file0:  block: 0, rfinput: 1, sample: 16750, value: 1
    assert_eq!(
        buffer[2
            * context.voltage_block_size_bytes as usize
            * context.num_voltage_blocks_per_second
            + get_index_for_location_in_test_voltage_file_mwaxv2(0, 1, 16750, 1)],
        29
    );

    // Second 1_101_503_321 (now we are in a new data file so the values are incrememented by 2 from the first file)
    // location in buffer: block: 40+0, rfinput: 1, sample: 16750, value: 1
    // location in file0:  block: 0, rfinput: 1, sample: 16750, value: 1
    assert_eq!(
        buffer[2
            * context.voltage_block_size_bytes as usize
            * context.num_voltage_blocks_per_second
            + get_index_for_location_in_test_voltage_file_mwaxv2(20, 1, 16750, 1)],
        -71
    );
}

#[test]
fn test_context_legacy_v1_get_fine_chan_feqs_one_coarse_chan() {
    let context = get_test_voltage_context(MWAVersion::VCSLegacyRecombined, false);

    // Get fine channel freqs
    let coarse_channels: Vec<usize> = vec![0];
    let fine_chan_freqs: Vec<f64> = context.get_fine_chan_freqs_hz_array(&coarse_channels);

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
    let context = get_test_voltage_context(MWAVersion::VCSLegacyRecombined, false);

    // Get fine channel freqs
    let coarse_channels: Vec<usize> = vec![10, 20];
    let fine_chan_freqs: Vec<f64> = context.get_fine_chan_freqs_hz_array(&coarse_channels);

    assert_eq!(fine_chan_freqs.len(), 256);
    assert!(approx_eq!(
        f64,
        fine_chan_freqs[0],
        151_680_000.0,
        F64Margin::default()
    ));

    assert!(approx_eq!(
        f64,
        fine_chan_freqs[128],
        164_480_000.0,
        F64Margin::default()
    ));
}

//
// Oversampled tests
//
#[test]
fn test_context_mwax_v2_oversampled() {
    // Create voltage context
    let context = get_test_voltage_context(MWAVersion::VCSMWAXv2, true);

    // Test the properties of the context object match what we expect
    // MWA version:       v2 mwax,
    assert_eq!(context.mwa_version, MWAVersion::VCSMWAXv2);

    // Actual gps start time:   1_370_755_832,
    assert_eq!(context.common_start_gps_time_ms, 1_370_755_832_000);

    // Actual gps end time:     1_370_755_848,
    assert_eq!(context.common_end_gps_time_ms, 1_370_755_848_000);

    // Actual duration:          16 s,
    assert_eq!(context.common_duration_ms, 16_000);

    // num timesteps:            4, (from metafits)
    assert_eq!(context.num_timesteps, 4);

    // timesteps:
    assert_eq!(context.timesteps[0].gps_time_ms, 1_370_755_832_000);
    assert_eq!(context.timesteps[1].gps_time_ms, 1_370_755_840_000);

    // num coarse channels,      2,
    assert_eq!(context.num_coarse_chans, 24);

    // observation bandwidth:    3.256_000 MHz,
    assert_eq!(context.common_bandwidth_hz, 1_280_000 * 2);

    // coarse channels:
    assert_eq!(context.coarse_chans[14].rec_chan_number, 123);
    assert_eq!(context.coarse_chans[14].chan_centre_hz, 157_440_000);
    assert_eq!(context.coarse_chans[15].rec_chan_number, 124);
    assert_eq!(context.coarse_chans[15].chan_centre_hz, 158_720_000);
    // fine channel resolution:  1.6384 MHz,
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
    assert_eq!(context.num_samples_per_voltage_block, 81_920);
    // The size of each voltage block
    assert_eq!(context.voltage_block_size_bytes, 327_680);
    // Number of bytes used to store delays - for MWAX this is the same as a voltage block size, for legacy it is 0
    assert_eq!(
        context.delay_block_size_bytes,
        context.voltage_block_size_bytes
    );
    // The amount of bytes to skip before getting into real data within the voltage files
    assert_eq!(context.data_file_header_size_bytes, 4096);
    // Expected voltage file size
    assert_eq!(context.expected_voltage_data_file_size_bytes, 52_760_576);
    // Check number of batches
    assert_eq!(context.voltage_batches.len(), 2);

    // Check rfinput order (for Legacy it is vcs_order, mwax is subfile_order)
    let mut rf_input_copy = context.metafits_context.rf_inputs.clone();
    rf_input_copy.sort_by_key(|k| k.subfile_order);
    // Now compare this copy with the 'real' rf_inputs
    assert_eq!(&rf_input_copy, &context.metafits_context.rf_inputs);
}
