// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Unit tests for voltage file metadata
*/
#[cfg(test)]
use super::*;
use std::fs::File;
use std::io::{Error, Write};

// Helper fuction to generate (small) test voltage files
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
fn test_determine_voltage_file_unrecognised_files() {
    let files = vec![
        "1065880128_106588012_ch123.dat",
        "106588012_1065880129_ch121.dat",
        "1065880128_1065880128__ch121.dat",
        "1065880128_1065880130_ch124.txt",
    ];
    let result = determine_voltage_file_gpstime_batches(&files, 1065880128);

    assert!(matches!(
        result.unwrap_err(),
        VoltageFileError::Unrecognised(_)
    ));
}

#[test]
fn test_determine_voltage_file_gpstime_batches_proper_legacy_format() {
    let files = vec![
        "1065880128_1065880129_ch122.dat",
        "1065880128_1065880129_ch21.dat",
        "1065880128_1065880129_ch1.dat",
        "1065880128_1065880128_ch21.dat",
        "1065880128_1065880128_ch001.dat",
        "1065880128_1065880128_ch122.dat",
        "1065880128_1065880130_ch122.dat",
        "1065880128_1065880130_ch021.dat",
        "1065880128_1065880130_ch01.dat",
    ];
    let result = determine_voltage_file_gpstime_batches(&files, 1065880128);
    assert!(result.is_ok(), "{:?}", result.unwrap_err());
    let (temp_voltage_files, mwa_version, num_gputimes, voltage_file_interval_ms) = result.unwrap();
    assert_eq!(mwa_version, MWAVersion::VCSLegacyRecombined);
    assert_eq!(num_gputimes, 3);
    assert_eq!(voltage_file_interval_ms, 1000);

    let expected_voltage_files = vec![
        TempVoltageFile {
            filename: "1065880128_1065880128_ch001.dat",
            obs_id: 1065880128,
            gps_time: 1065880128,
            channel_identifier: 1,
        },
        TempVoltageFile {
            filename: "1065880128_1065880128_ch21.dat",
            obs_id: 1065880128,
            gps_time: 1065880128,
            channel_identifier: 21,
        },
        TempVoltageFile {
            filename: "1065880128_1065880128_ch122.dat",
            obs_id: 1065880128,
            gps_time: 1065880128,
            channel_identifier: 122,
        },
        TempVoltageFile {
            filename: "1065880128_1065880129_ch1.dat",
            obs_id: 1065880128,
            gps_time: 1065880129,
            channel_identifier: 1,
        },
        TempVoltageFile {
            filename: "1065880128_1065880129_ch21.dat",
            obs_id: 1065880128,
            gps_time: 1065880129,
            channel_identifier: 21,
        },
        TempVoltageFile {
            filename: "1065880128_1065880129_ch122.dat",
            obs_id: 1065880128,
            gps_time: 1065880129,
            channel_identifier: 122,
        },
        TempVoltageFile {
            filename: "1065880128_1065880130_ch01.dat",
            obs_id: 1065880128,
            gps_time: 1065880130,
            channel_identifier: 1,
        },
        TempVoltageFile {
            filename: "1065880128_1065880130_ch021.dat",
            obs_id: 1065880128,
            gps_time: 1065880130,
            channel_identifier: 21,
        },
        TempVoltageFile {
            filename: "1065880128_1065880130_ch122.dat",
            obs_id: 1065880128,
            gps_time: 1065880130,
            channel_identifier: 122,
        },
    ];

    assert_eq!(temp_voltage_files, expected_voltage_files);
}

#[test]
fn test_determine_voltage_file_gpstime_batches_proper_mwax_format() {
    let files = vec![
        "1065880128_1065880136_122.sub",
        "1065880128_1065880136_21.sub",
        "1065880128_1065880136_1.sub",
        "1065880128_1065880128_21.sub",
        "1065880128_1065880128_001.sub",
        "1065880128_1065880128_122.sub",
        "1065880128_1065880144_122.sub",
        "1065880128_1065880144_021.sub",
        "1065880128_1065880144_01.sub",
    ];
    let result = determine_voltage_file_gpstime_batches(&files, 1065880128);
    assert!(result.is_ok(), "{:?}", result.unwrap_err());
    let (temp_voltage_files, mwa_version, num_gputimes, voltage_file_interval_ms) = result.unwrap();
    assert_eq!(mwa_version, MWAVersion::VCSMWAXv2);
    assert_eq!(num_gputimes, 3);
    assert_eq!(voltage_file_interval_ms, 8000);

    let expected_voltage_files = vec![
        TempVoltageFile {
            filename: "1065880128_1065880128_001.sub",
            obs_id: 1065880128,
            gps_time: 1065880128,
            channel_identifier: 1,
        },
        TempVoltageFile {
            filename: "1065880128_1065880128_21.sub",
            obs_id: 1065880128,
            gps_time: 1065880128,
            channel_identifier: 21,
        },
        TempVoltageFile {
            filename: "1065880128_1065880128_122.sub",
            obs_id: 1065880128,
            gps_time: 1065880128,
            channel_identifier: 122,
        },
        TempVoltageFile {
            filename: "1065880128_1065880136_1.sub",
            obs_id: 1065880128,
            gps_time: 1065880136,
            channel_identifier: 1,
        },
        TempVoltageFile {
            filename: "1065880128_1065880136_21.sub",
            obs_id: 1065880128,
            gps_time: 1065880136,
            channel_identifier: 21,
        },
        TempVoltageFile {
            filename: "1065880128_1065880136_122.sub",
            obs_id: 1065880128,
            gps_time: 1065880136,
            channel_identifier: 122,
        },
        TempVoltageFile {
            filename: "1065880128_1065880144_01.sub",
            obs_id: 1065880128,
            gps_time: 1065880144,
            channel_identifier: 1,
        },
        TempVoltageFile {
            filename: "1065880128_1065880144_021.sub",
            obs_id: 1065880128,
            gps_time: 1065880144,
            channel_identifier: 21,
        },
        TempVoltageFile {
            filename: "1065880128_1065880144_122.sub",
            obs_id: 1065880128,
            gps_time: 1065880144,
            channel_identifier: 122,
        },
    ];

    assert_eq!(temp_voltage_files, expected_voltage_files);
}

#[test]
fn test_determine_voltage_file_gpstime_batches_chan_mismatch() {
    let files = vec![
        "1065880128_1065880129_ch123.dat",
        "1065880128_1065880129_ch121.dat",
        "1065880128_1065880128_ch121.dat",
        "1065880128_1065880130_ch124.dat",
    ];
    let result = determine_voltage_file_gpstime_batches(&files, 1065880128);
    assert!(matches!(
        result.unwrap_err(),
        VoltageFileError::UnevenChannelsForGpsTime {
            expected: _,
            got: _
        }
    ));
}

#[test]
fn test_determine_voltage_file_gpstime_batches_no_files() {
    let files: Vec<String> = Vec::new();
    let result = determine_voltage_file_gpstime_batches(&files, 1065880128);
    assert!(matches!(
        result.unwrap_err(),
        VoltageFileError::NoVoltageFiles
    ));
}

#[test]
fn test_determine_voltage_file_mwa_version_mismatch() {
    let files = vec![
        "1065880128_1065880129_ch123.dat",
        "1065880128_1065880129_121.sub",
        "1065880128_1065880128_ch121.dat",
        "1065880128_1065880130_ch124.dat",
    ];
    let result = determine_voltage_file_gpstime_batches(&files, 1065880128);
    assert!(matches!(result.unwrap_err(), VoltageFileError::Mixture));
}

#[test]
fn test_determine_voltage_file_metafits_obs_id_mismatch() {
    let files = vec![
        "1065880128_1065880128_ch121.dat",
        "1065880128_1065880129_ch121.dat",
        "1065880128_1065880130_ch121.dat",
    ];
    let result = determine_voltage_file_gpstime_batches(&files, 1234567890);
    assert!(matches!(
        result.unwrap_err(),
        VoltageFileError::MetafitsObsidMismatch
    ));
}

#[test]
fn test_determine_voltage_file_gpstime_missing() {
    let files = vec![
        "1065880128_1065880128_ch121.dat",
        "1065880128_1065880130_ch121.dat",
    ];
    let result = determine_voltage_file_gpstime_batches(&files, 1065880128);
    assert!(matches!(
        result.unwrap_err(),
        VoltageFileError::GpsTimeMissing {
            expected: _,
            got: _
        }
    ));
}

#[test]
fn test_determine_obs_times_test_many_timesteps_legacy() {
    let common_times: Vec<u64> = vec![1065880129, 1065880130, 1065880131, 1065880132, 1065880133];
    let mut input = VoltageFileTimeMap::new();
    // insert a "dangling time" at the beginning (1065880128) which is not a common timestep
    let mut new_time_tree = BTreeMap::new();
    new_time_tree.insert(120, String::from("1065880128_1065880128_ch120.dat"));
    input.insert(1065880128, new_time_tree);

    // Add the common times to the data structure
    for time in common_times.iter() {
        input
            .entry(*time)
            .or_insert_with(BTreeMap::new)
            .entry(121)
            .or_insert(format!("1065880128_{}_ch121.dat", time));

        input
            .entry(*time)
            .or_insert_with(BTreeMap::new)
            .entry(122)
            .or_insert(format!("1065880128_{}_ch122.dat", time));
    }

    // insert a "dangling time" at the end (1065880134) which is not a common timestep
    new_time_tree = BTreeMap::new();
    new_time_tree.insert(120, String::from("1065880128_1065880134_ch120.dat"));
    input.insert(1065880134, new_time_tree);

    let expected_interval: u64 = 1000; // 1000 since we are Legacy
    let expected_start: u64 = *common_times.first().unwrap() * 1000;
    let expected_end: u64 = (*common_times.last().unwrap() * 1000) + expected_interval;
    // Duration = common end - common start + integration time
    // == 1065880133 - 1065880129 + 1
    let expected_duration = 5000;
    let voltage_file_interval_ms: u64 = 1000;

    let result = determine_obs_times(&input, voltage_file_interval_ms);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(
        result.start_gps_time_ms, expected_start,
        "start_gps_time_ms incorrect {:?}",
        result
    );
    assert_eq!(
        result.end_gps_time_ms, expected_end,
        "end_gps_time_ms incorrect {:?}",
        result
    );
    assert_eq!(
        result.duration_ms, expected_duration,
        "duration_ms incorrect {:?}",
        result
    );
    assert_eq!(
        result.voltage_file_interval_ms, expected_interval,
        "voltage_file_interval_ms incorrect {:?}",
        result
    );
}

#[test]
fn test_determine_obs_times_test_many_timesteps_mwax() {
    let common_times: Vec<u64> = vec![1065880136, 1065880144, 1065880152, 1065880160, 1065880168];
    let mut input = VoltageFileTimeMap::new();
    // insert a "dangling time" at the beginning (1065880128) which is not a common timestep
    let mut new_time_tree = BTreeMap::new();
    new_time_tree.insert(120, String::from("1065880128_1065880128_120.sub"));
    input.insert(1065880128, new_time_tree);

    // Add the common times to the data structure
    for time in common_times.iter() {
        input
            .entry(*time)
            .or_insert_with(BTreeMap::new)
            .entry(121)
            .or_insert(format!("1065880128_{}_121.sub", time));

        input
            .entry(*time)
            .or_insert_with(BTreeMap::new)
            .entry(122)
            .or_insert(format!("1065880128_{}_122.sub", time));
    }

    // insert a "dangling time" at the end (1065880176) which is not a common timestep
    new_time_tree = BTreeMap::new();
    new_time_tree.insert(120, String::from("1065880128_1065880176_120.sub"));
    input.insert(1065880176, new_time_tree);

    let expected_interval: u64 = 8000; // 8000 since we are MWAX
    let expected_start: u64 = *common_times.first().unwrap() * 1000;
    let expected_end: u64 = (*common_times.last().unwrap() * 1000) + expected_interval;
    // Duration = common end - common start + integration time
    // == 1065880168 - 1065880136 + 8
    let expected_duration = 40000;
    let voltage_file_interval_ms: u64 = 8000;

    let result = determine_obs_times(&input, voltage_file_interval_ms);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(
        result.start_gps_time_ms, expected_start,
        "start_gps_time_ms incorrect {:?}",
        result
    );
    assert_eq!(
        result.end_gps_time_ms, expected_end,
        "end_gps_time_ms incorrect {:?}",
        result
    );
    assert_eq!(
        result.duration_ms, expected_duration,
        "duration_ms incorrect {:?}",
        result
    );
    assert_eq!(
        result.voltage_file_interval_ms, expected_interval,
        "voltage_file_interval_ms incorrect {:?}",
        result
    );
}

#[test]
fn test_voltage_file_batch_new() {
    let new_batch = VoltageFileBatch::new(1234567890);

    // Check the new batch is created ok
    assert_eq!(new_batch.gps_time, 1234567890);
    assert_eq!(new_batch.voltage_files.len(), 0);
}

#[test]
fn test_voltage_file_batch_partialeq() {
    let mut batch1 = VoltageFileBatch::new(1234567890);
    let voltage_file1 = VoltageFile {
        filename: String::from("test.dat"),
        channel_identifier: 123,
    };
    batch1.voltage_files.push(voltage_file1);

    // Should be == to batch1
    let mut batch2 = VoltageFileBatch::new(1234567890);
    let voltage_file2 = VoltageFile {
        filename: String::from("test.dat"),
        channel_identifier: 123,
    };
    batch2.voltage_files.push(voltage_file2);

    // Should be != batch1 (filename)
    let mut batch3 = VoltageFileBatch::new(1234567890);
    let voltage_file3 = VoltageFile {
        filename: String::from("test1.dat"),
        channel_identifier: 123,
    };
    batch3.voltage_files.push(voltage_file3);

    // Should be != batch1 (gpstime)
    let mut batch4 = VoltageFileBatch::new(9876543210);
    let voltage_file4 = VoltageFile {
        filename: String::from("test.dat"),
        channel_identifier: 123,
    };
    batch4.voltage_files.push(voltage_file4);

    // Check the eq works
    assert_eq!(batch1, batch2);

    assert_ne!(batch1, batch3);

    assert_ne!(batch1, batch4);
}

#[test]
fn test_convert_temp_voltage_files() {
    let temp_voltage_files: Vec<TempVoltageFile> = vec![
        TempVoltageFile {
            filename: "1234567000_1234567000_123.sub",
            obs_id: 1234567000,
            channel_identifier: 123,
            gps_time: 1234567000,
        },
        TempVoltageFile {
            filename: "1234567890_1234567008_124.sub",
            obs_id: 1234567000,
            channel_identifier: 124,
            gps_time: 1234567008,
        },
        TempVoltageFile {
            filename: "1234567890_1234567008_123.sub",
            obs_id: 1234567000,
            channel_identifier: 123,
            gps_time: 1234567008,
        },
        TempVoltageFile {
            filename: "1234567890_1234567008_125.sub",
            obs_id: 1234567000,
            channel_identifier: 125,
            gps_time: 1234567008,
        },
        TempVoltageFile {
            filename: "1234567000_1234567000_124.sub",
            obs_id: 1234567000,
            channel_identifier: 124,
            gps_time: 1234567000,
        },
    ];

    // The resulting VoltageFileBatches should:
    // * have 2 batches
    // * batches sorted by gpstime
    // * each batch sorted by coarse channel indentifier
    let batches: HashMap<u64, VoltageFileBatch> = convert_temp_voltage_files(temp_voltage_files);

    assert_eq!(
        batches.len(),
        2,
        "Error - number of batches is incorrect: {} should be 2.",
        batches.len()
    );
    assert_eq!(batches.get(&1234567000).unwrap().voltage_files.len(), 2);
    assert_eq!(batches.get(&1234567008).unwrap().voltage_files.len(), 3);
}

#[test]
fn test_examine_voltage_files_valid() {
    // Get a metafits context
    // Open the metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits
    let context =
        MetafitsContext::new(&metafits_filename).expect("Failed to create MetafitsContext");

    // Create a temp dir for the temp files
    // Once out of scope the temp dir and it's contents will be deleted
    let temp_dir = tempdir::TempDir::new("voltage_test").unwrap();

    // Populate vector of filenames
    let voltage_filenames: Vec<String> = vec![
        String::from("1101503312_1101503312_123.sub"),
        String::from("1101503312_1101503312_124.sub"),
        String::from("1101503312_1101503320_123.sub"),
        String::from("1101503312_1101503320_124.sub"),
    ];

    let mut temp_filenames: Vec<String> = Vec::new();

    for f in voltage_filenames.iter() {
        temp_filenames.push(generate_test_voltage_file(&temp_dir, f, 2, 256).unwrap());
    }
    let result = examine_voltage_files(&context, &temp_filenames);

    assert!(
        result.is_ok(),
        "examine_voltage_files failed {:?} - temp filenames: {:?}",
        result,
        temp_filenames
    );
}

#[test]
fn test_examine_voltage_files_error_mismatched_sizes() {
    // Get a metafits context
    // Open the metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits
    let context =
        MetafitsContext::new(&metafits_filename).expect("Failed to create MetafitsContext");

    // Create a temp dir for the temp files
    // Once out of scope the temp dir and it's contents will be deleted
    let temp_dir = tempdir::TempDir::new("voltage_test").unwrap();

    // Populate vector of filenames
    let voltage_filenames: Vec<String> = vec![
        String::from("1101503312_1101503312_123.sub"),
        String::from("1101503312_1101503312_124.sub"),
        String::from("1101503312_1101503320_123.sub"),
        String::from("1101503312_1101503320_124.sub"),
    ];

    let mut temp_filenames: Vec<String> = Vec::new();

    for f in voltage_filenames.iter() {
        temp_filenames.push(generate_test_voltage_file(&temp_dir, f, 2, 256).unwrap());
    }
    // Now add a gps time batch which is a different size
    temp_filenames.push(
        generate_test_voltage_file(&temp_dir, "1101503312_1101503328_123.sub", 1, 256).unwrap(),
    );
    temp_filenames.push(
        generate_test_voltage_file(&temp_dir, "1101503312_1101503328_124.sub", 1, 256).unwrap(),
    );

    let result = examine_voltage_files(&context, &temp_filenames);

    assert!(result.is_err());

    assert!(matches!(
        result.unwrap_err(),
        VoltageFileError::UnequalFileSizes
    ));
}

#[test]
fn test_examine_voltage_files_error_gpstime_gaps() {
    // Get a metafits context
    // Open the metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits
    let context =
        MetafitsContext::new(&metafits_filename).expect("Failed to create MetafitsContext");

    // Create a temp dir for the temp files
    // Once out of scope the temp dir and it's contents will be deleted
    let temp_dir = tempdir::TempDir::new("voltage_test").unwrap();

    // Populate vector of filenames
    // NOTE: Gap of 8 seconds between elements 0,1 and 2,3
    let voltage_filenames: Vec<String> = vec![
        String::from("1101503312_1101503312_123.sub"),
        String::from("1101503312_1101503312_124.sub"),
        String::from("1101503312_1101503328_123.sub"),
        String::from("1101503312_1101503328_124.sub"),
    ];

    let mut temp_filenames: Vec<String> = Vec::new();

    for f in voltage_filenames.iter() {
        temp_filenames.push(generate_test_voltage_file(&temp_dir, f, 2, 256).unwrap());
    }

    let result = examine_voltage_files(&context, &temp_filenames);

    assert!(result.is_err());

    assert!(matches!(
        result.unwrap_err(),
        VoltageFileError::GpsTimeMissing {
            expected: _,
            got: _
        }
    ));
}

#[test]
fn test_examine_voltage_files_error_file_not_found() {
    // Get a metafits context
    // Open the metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits
    let context =
        MetafitsContext::new(&metafits_filename).expect("Failed to create MetafitsContext");

    // Populate vector of filenames
    let voltage_filenames: Vec<String> = vec![
        String::from("test_files_invalid/1101503312_1101503312_123.sub"),
        String::from("test_files_invalid/1101503312_1101503312_124.sub"),
        String::from("test_files_invalid/1101503312_1101503320_123.sub"),
        String::from("test_files_invalid/1101503312_1101503320_124.sub"),
    ];

    let result = examine_voltage_files(&context, &voltage_filenames);

    assert!(result.is_err());

    assert!(matches!(
        result.unwrap_err(),
        VoltageFileError::VoltageFileError(_, _)
    ));
}
