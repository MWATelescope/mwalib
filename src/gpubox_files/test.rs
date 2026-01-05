// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Unit tests for gpubox file metadata
use super::*;
use crate::test::with_new_temp_fits_file;
use fitsio::images::{ImageDescription, ImageType};
use std::time::SystemTime;

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
            .or_default()
            .entry(101)
            .or_insert((0, chan_index + 1));
    }

    for (chan_index, unix_time_ms) in coarse_chan102_unix_times.iter().enumerate() {
        gpubox_time_map
            .entry(*unix_time_ms)
            .or_default()
            .entry(102)
            .or_insert((0, chan_index + 1));
    }

    for (chan_index, unix_time_ms) in coarse_chan103_unix_times.iter().enumerate() {
        gpubox_time_map
            .entry(*unix_time_ms)
            .or_default()
            .entry(103)
            .or_insert((0, chan_index + 1));
    }

    for (chan_index, unix_time_ms) in coarse_chan104_unix_times.iter().enumerate() {
        gpubox_time_map
            .entry(*unix_time_ms)
            .or_default()
            .entry(104)
            .or_insert((0, chan_index + 1));
    }

    gpubox_time_map
}

#[test]
fn test_determine_gpubox_batches_proper_format() {
    let files = vec![
        "1065880128_20131015134930_gpubox20_01.fits",
        "1065880128_20131015134930_gpubox01_00.fits",
        "1065880128_20131015134930_gpubox15_02.fits",
    ];
    let result = determine_gpubox_batches(&files);
    assert!(result.is_ok());
    let (temp_gpuboxes, corr_format) = result.unwrap();
    assert_eq!(corr_format, MWAVersion::CorrLegacy);

    let expected_gpuboxes = vec![
        TempGpuBoxFile {
            filename: "1065880128_20131015134930_gpubox01_00.fits",
            channel_identifier: 1,
            batch_number: 0,
        },
        TempGpuBoxFile {
            filename: "1065880128_20131015134930_gpubox20_01.fits",
            channel_identifier: 20,
            batch_number: 1,
        },
        TempGpuBoxFile {
            filename: "1065880128_20131015134930_gpubox15_02.fits",
            channel_identifier: 15,
            batch_number: 2,
        },
    ];

    assert_eq!(temp_gpuboxes, expected_gpuboxes);
}

#[test]
fn test_determine_gpubox_batches_proper_format2() {
    let files = vec![
        "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
        "/home/gs/1065880128_20131015134930_gpubox20_01.fits",
        "/var/cache/1065880128_20131015134930_gpubox15_02.fits",
    ];
    let result = determine_gpubox_batches(&files);
    assert!(result.is_ok());
    let (gpubox_batches, corr_format) = result.unwrap();
    assert_eq!(corr_format, MWAVersion::CorrLegacy);
    let expected_batches = vec![
        TempGpuBoxFile {
            filename: "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
            channel_identifier: 1,
            batch_number: 0,
        },
        TempGpuBoxFile {
            filename: "/home/gs/1065880128_20131015134930_gpubox20_01.fits",
            channel_identifier: 20,
            batch_number: 1,
        },
        TempGpuBoxFile {
            filename: "/var/cache/1065880128_20131015134930_gpubox15_02.fits",
            channel_identifier: 15,
            batch_number: 2,
        },
    ];

    assert_eq!(gpubox_batches, expected_batches);
}

#[test]
fn test_determine_gpubox_batches_proper_format3() {
    let files = vec![
        "/home/chj/1065880128_20131015134930_gpubox02_00.fits",
        "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
        "/home/chj/1065880128_20131015134930_gpubox20_01.fits",
        "/home/chj/1065880128_20131015134930_gpubox19_01.fits",
        "/home/chj/1065880128_20131015134930_gpubox14_02.fits",
        "/home/chj/1065880128_20131015134930_gpubox15_02.fits",
    ];
    let result = determine_gpubox_batches(&files);
    assert!(result.is_ok());
    let (gpubox_batches, corr_format) = result.unwrap();
    assert_eq!(corr_format, MWAVersion::CorrLegacy);

    let expected_batches = vec![
        TempGpuBoxFile {
            filename: "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
            channel_identifier: 1,
            batch_number: 0,
        },
        TempGpuBoxFile {
            filename: "/home/chj/1065880128_20131015134930_gpubox02_00.fits",
            channel_identifier: 2,
            batch_number: 0,
        },
        TempGpuBoxFile {
            filename: "/home/chj/1065880128_20131015134930_gpubox19_01.fits",
            channel_identifier: 19,
            batch_number: 1,
        },
        TempGpuBoxFile {
            filename: "/home/chj/1065880128_20131015134930_gpubox20_01.fits",
            channel_identifier: 20,
            batch_number: 1,
        },
        TempGpuBoxFile {
            filename: "/home/chj/1065880128_20131015134930_gpubox14_02.fits",
            channel_identifier: 14,
            batch_number: 2,
        },
        TempGpuBoxFile {
            filename: "/home/chj/1065880128_20131015134930_gpubox15_02.fits",
            channel_identifier: 15,
            batch_number: 2,
        },
    ];

    assert_eq!(gpubox_batches, expected_batches);
}

#[test]
fn test_determine_gpubox_batches_proper_format4() {
    let files = vec![
        "/home/chj/1065880128_20131015134929_gpubox02_00.fits",
        "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
        "/home/chj/1065880128_20131015134929_gpubox20_01.fits",
        "/home/chj/1065880128_20131015134930_gpubox19_01.fits",
        "/home/chj/1065880128_20131015134931_gpubox14_02.fits",
        "/home/chj/1065880128_20131015134930_gpubox15_02.fits",
    ];
    let result = determine_gpubox_batches(&files);
    assert!(result.is_ok());
    let (gpubox_batches, corr_format) = result.unwrap();
    assert_eq!(corr_format, MWAVersion::CorrLegacy);

    let expected_batches = vec![
        TempGpuBoxFile {
            filename: "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
            channel_identifier: 1,
            batch_number: 0,
        },
        TempGpuBoxFile {
            filename: "/home/chj/1065880128_20131015134929_gpubox02_00.fits",
            channel_identifier: 2,
            batch_number: 0,
        },
        TempGpuBoxFile {
            filename: "/home/chj/1065880128_20131015134930_gpubox19_01.fits",
            channel_identifier: 19,
            batch_number: 1,
        },
        TempGpuBoxFile {
            filename: "/home/chj/1065880128_20131015134929_gpubox20_01.fits",
            channel_identifier: 20,
            batch_number: 1,
        },
        TempGpuBoxFile {
            filename: "/home/chj/1065880128_20131015134931_gpubox14_02.fits",
            channel_identifier: 14,
            batch_number: 2,
        },
        TempGpuBoxFile {
            filename: "/home/chj/1065880128_20131015134930_gpubox15_02.fits",
            channel_identifier: 15,
            batch_number: 2,
        },
    ];

    assert_eq!(gpubox_batches, expected_batches);
}

#[test]
fn test_determine_gpubox_batches_invalid_filename() {
    let files = vec!["1065880128_20131015134930_gpubox0100.fits"];
    let result = determine_gpubox_batches(&files);
    assert!(result.is_err());
}

#[test]
fn test_determine_gpubox_batches_invalid_filename2() {
    let files = vec!["1065880128x_20131015134930_gpubox01_00.fits"];
    let result = determine_gpubox_batches(&files);
    assert!(result.is_err());
}

#[test]
fn test_determine_gpubox_batches_invalid_filename3() {
    let files = vec!["1065880128_920131015134930_gpubox01_00.fits"];
    let result = determine_gpubox_batches(&files);
    assert!(result.is_err());
}

#[test]
fn test_determine_gpubox_batches_valid() {
    // There are no gpubox files for batch "01".
    let files = vec![
        "1065880128_20131015134930_gpubox01_00.fits",
        "1065880128_20131015134930_gpubox15_02.fits",
    ];
    let result = determine_gpubox_batches(&files);
    assert!(result.is_ok());
}

#[test]
fn test_determine_gpubox_batches_valid2() {
    // There are not different numbers of gpubox files for batches "00" and "01" vs "02".
    let files = vec![
        "1065880128_20131015134930_gpubox01_00.fits",
        "1065880128_20131015134930_gpubox02_00.fits",
        "1065880128_20131015134930_gpubox01_01.fits",
        "1065880128_20131015134930_gpubox02_01.fits",
        "1065880128_20131015134930_gpubox15_02.fits",
    ];
    let result = determine_gpubox_batches(&files);
    assert!(result.is_ok());
}

#[test]
fn test_determine_gpubox_batches_old_format() {
    let files = vec![
        "1065880128_20131015134930_gpubox01.fits",
        "1065880128_20131015134930_gpubox20.fits",
        "1065880128_20131015134930_gpubox15.fits",
    ];
    let result = determine_gpubox_batches(&files);
    assert!(result.is_ok());
    let (gpubox_batches, corr_format) = result.unwrap();
    assert_eq!(corr_format, MWAVersion::CorrOldLegacy);

    let expected_batches = vec![
        TempGpuBoxFile {
            filename: "1065880128_20131015134930_gpubox01.fits",
            channel_identifier: 1,
            batch_number: 0,
        },
        TempGpuBoxFile {
            filename: "1065880128_20131015134930_gpubox15.fits",
            channel_identifier: 15,
            batch_number: 0,
        },
        TempGpuBoxFile {
            filename: "1065880128_20131015134930_gpubox20.fits",
            channel_identifier: 20,
            batch_number: 0,
        },
    ];

    assert_eq!(gpubox_batches, expected_batches);
}

#[test]
fn test_determine_gpubox_batches_new_format() {
    let files = vec![
        "1065880128_20131015134930_ch101_000.fits",
        "1065880128_20131015134930_ch102_000.fits",
        "1065880128_20131015135030_ch101_001.fits",
        "1065880128_20131015135030_ch102_001.fits",
    ];
    let result = determine_gpubox_batches(&files);
    assert!(result.is_ok());
    let (gpubox_batches, corr_format) = result.unwrap();
    assert_eq!(corr_format, MWAVersion::CorrMWAXv2);

    let expected_batches = vec![
        TempGpuBoxFile {
            filename: "1065880128_20131015134930_ch101_000.fits",
            channel_identifier: 101,
            batch_number: 0,
        },
        TempGpuBoxFile {
            filename: "1065880128_20131015134930_ch102_000.fits",
            channel_identifier: 102,
            batch_number: 0,
        },
        TempGpuBoxFile {
            filename: "1065880128_20131015135030_ch101_001.fits",
            channel_identifier: 101,
            batch_number: 1,
        },
        TempGpuBoxFile {
            filename: "1065880128_20131015135030_ch102_001.fits",
            channel_identifier: 102,
            batch_number: 1,
        },
    ];

    assert_eq!(gpubox_batches, expected_batches);
}

#[test]
fn test_determine_gpubox_batches_mix() {
    let files = vec![
        "1065880128_20131015134930_gpubox01.fits",
        "1065880128_20131015134930_gpubox15_01.fits",
    ];
    let result = determine_gpubox_batches(&files);
    assert!(result.is_err());
}

#[test]
fn test_no_hdus() {
    // When a fits file has no hdus, lets abort processing and return an error
    let metafits_filename = "test_files/no_hdus/1196175296.metafits";
    let filename = "test_files/no_hdus/1196175296_20171201145440_gpubox01_00.fits";
    let gpuboxfiles = vec![filename];

    let result = CorrelatorContext::new(metafits_filename, &gpuboxfiles);

    assert!(matches!(
        result.unwrap_err(),
        MwalibError::Gpubox(GpuboxError::NoDataHDUsInGpuboxFile { gpubox_filename: _ })
    ));
}

#[test]
fn test_determine_hdu_time_test1() {
    // with_temp_file creates a temp dir and temp file, then removes them once out of scope
    with_new_temp_fits_file("determine_hdu_time_test1.fits", |fptr| {
        let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

        // Write the TIME and MILLITIM keys. Key types must be i64 to get any
        // sort of sanity.
        hdu.write_key(fptr, "TIME", 1_434_494_061)
            .expect("Couldn't write key 'TIME'");
        hdu.write_key(fptr, "MILLITIM", 0)
            .expect("Couldn't write key 'MILLITIM'");

        let result = determine_hdu_time(fptr, &hdu);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1_434_494_061_000);
    });
}

#[test]
fn test_determine_hdu_time_test2() {
    // with_temp_file creates a temp dir and temp file, then removes them once out of scope
    with_new_temp_fits_file("determine_hdu_time_test2.fits", |fptr| {
        let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

        hdu.write_key(fptr, "TIME", 1_381_844_923)
            .expect("Couldn't write key 'TIME'");
        hdu.write_key(fptr, "MILLITIM", 500)
            .expect("Couldn't write key 'MILLITIM'");

        let result = determine_hdu_time(fptr, &hdu);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1_381_844_923_500);
    });
}

#[test]
fn test_determine_hdu_time_test3() {
    // Use the current UNIX time.
    let current = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Err(e) => panic!("Something is wrong with time on your system: {}", e),
        Ok(n) => n.as_secs(),
    };

    // with_temp_file creates a temp dir and temp file, then removes them once out of scope
    with_new_temp_fits_file("determine_hdu_time_test3.fits", |fptr| {
        let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

        hdu.write_key(fptr, "TIME", current)
            .expect("Couldn't write key 'TIME'");
        hdu.write_key(fptr, "MILLITIM", 500)
            .expect("Couldn't write key 'MILLITIM'");

        let result = determine_hdu_time(fptr, &hdu);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), current * 1000 + 500);
    });
}

#[test]
fn test_map_unix_times_to_hdus_test() {
    // with_temp_file creates a temp dir and temp file, then removes them once out of scope
    with_new_temp_fits_file("map_unix_times_to_hdus_test.fits", |fptr| {
        let times: Vec<(u64, u64)> =
            vec![(1_381_844_923, 500), (1_381_844_924, 0), (1_381_844_950, 0)];
        let mut expected = BTreeMap::new();
        let image_description = ImageDescription {
            data_type: ImageType::Float,
            dimensions: &[100, 100],
        };
        for (i, (time, millitime)) in times.iter().enumerate() {
            let hdu = fptr
                .create_image("EXTNAME".to_string(), &image_description)
                .expect("Couldn't create image");
            hdu.write_key(fptr, "TIME", *time)
                .expect("Couldn't write key 'TIME'");
            hdu.write_key(fptr, "MILLITIM", *millitime)
                .expect("Couldn't write key 'MILLITIM'");

            expected.insert(time * 1000 + millitime, i + 1);
        }

        let result = map_unix_times_to_hdus(fptr, MWAVersion::CorrLegacy);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    });
}

#[test]
fn test_determine_common_times_test_many_timesteps() {
    // Create two files, with mostly overlapping times, but also a little
    // dangling at the start and end.
    let common_times: Vec<u64> = vec![
        1_381_844_923_500,
        1_381_844_924_000,
        1_381_844_924_500,
        1_381_844_925_000,
        1_381_844_925_500,
    ];
    let integration_time_ms = 500;

    let mut input = BTreeMap::new();
    let mut new_time_tree = BTreeMap::new();
    new_time_tree.insert(0, (0, 1));
    input.insert(1_381_844_923_000, new_time_tree);

    for (i, time) in common_times.iter().enumerate() {
        let mut new_time_tree = BTreeMap::new();
        // gpubox 0.
        new_time_tree.insert(0, (0, i + 2));
        // gpubox 1.
        new_time_tree.insert(1, (0, i + 1));
        input.insert(*time, new_time_tree);
    }

    let mut new_time_tree = BTreeMap::new();
    new_time_tree.insert(1, (0, common_times.len() + 1));
    input.insert(1_381_844_926_000, new_time_tree);

    let expected_start = *common_times.first().unwrap();
    let expected_end = *common_times.last().unwrap() + integration_time_ms;
    // Duration = common end - common start + integration time
    // == 1_381_844_925_500 - 1_381_844_923_500 + 500
    let expected_duration = 2500;

    let result = determine_common_obs_times_and_chans(&input, integration_time_ms, None);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!(result.start_time_unix_ms, expected_start);
    assert_eq!(result.end_time_unix_ms, expected_end);
    assert_eq!(result.duration_ms, expected_duration);
    assert_eq!(result.coarse_chan_identifiers.len(), 2);
    assert_eq!(result.coarse_chan_identifiers[0], 0);
    assert_eq!(result.coarse_chan_identifiers[1], 1);
}

#[test]
fn test_determine_common_times_test_one_timestep() {
    // Create two files, with 1 overlapping times, but also a little
    // dangling at the start and end.
    let common_times: Vec<u64> = vec![1_381_844_923_500];
    let integration_time_ms = 500;

    let mut input = BTreeMap::new();
    let mut new_time_tree = BTreeMap::new();
    new_time_tree.insert(0, (0, 1));
    // Add a dangling time before the common time
    input.insert(1_381_844_923_000, new_time_tree);

    for (i, time) in common_times.iter().enumerate() {
        let mut new_time_tree = BTreeMap::new();
        // gpubox 0.
        new_time_tree.insert(0, (0, i + 2));
        // gpubox 1.
        new_time_tree.insert(1, (0, i + 1));
        input.insert(*time, new_time_tree);
    }

    let mut new_time_tree = BTreeMap::new();
    new_time_tree.insert(1, (0, common_times.len() + 1));
    // Add a dangling time after the common time
    input.insert(1_381_844_924_000, new_time_tree);

    let expected_start = *common_times.first().unwrap();
    let expected_end = *common_times.last().unwrap() + integration_time_ms;
    // Duration = common end - common start + integration time
    // == 1_381_844_923_500 - 1_381_844_923_500 + 500
    let expected_duration = 500;

    let result = determine_common_obs_times_and_chans(&input, integration_time_ms, None);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!(result.start_time_unix_ms, expected_start);
    assert_eq!(result.end_time_unix_ms, expected_end);
    assert_eq!(result.duration_ms, expected_duration);
}

#[test]
fn test_validate_gpubox_metadata_mwa_version() {
    // with_temp_file creates a temp dir and temp file, then removes them once out of scope
    with_new_temp_fits_file("test_validate_gpubox_metadata_mwa_version.fits", |fptr| {
        let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

        // This should succeed- LegacyOld should NOT have CORR_VER key
        assert!(validate_gpubox_metadata_mwa_version(
            fptr,
            &hdu,
            &String::from("test_file.fits"),
            MWAVersion::CorrOldLegacy,
        )
        .is_ok());

        // This should succeed- Legacy should NOT have CORR_VER key
        assert!(validate_gpubox_metadata_mwa_version(
            fptr,
            &hdu,
            &String::from("test_file.fits"),
            MWAVersion::CorrLegacy,
        )
        .is_ok());

        // This should fail- V2 needs CORR_VER key
        assert!(validate_gpubox_metadata_mwa_version(
            fptr,
            &hdu,
            &String::from("test_file.fits"),
            MWAVersion::CorrMWAXv2,
        )
        .is_err());

        // Now put in a corr version
        hdu.write_key(fptr, "CORR_VER", 2)
            .expect("Couldn't write key 'CORR_VER'");

        // This should succeed- V2 should have CORR_VER key
        assert!(validate_gpubox_metadata_mwa_version(
            fptr,
            &hdu,
            &String::from("test_file.fits"),
            MWAVersion::CorrMWAXv2,
        )
        .is_ok());

        // This should fail- OldLegacy should NOT have CORR_VER key
        assert!(validate_gpubox_metadata_mwa_version(
            fptr,
            &hdu,
            &String::from("test_file.fits"),
            MWAVersion::CorrOldLegacy,
        )
        .is_err());

        // This should fail- Legacy should NOT have CORR_VER key
        assert!(validate_gpubox_metadata_mwa_version(
            fptr,
            &hdu,
            &String::from("test_file.fits"),
            MWAVersion::CorrLegacy,
        )
        .is_err());
    });

    // with_temp_file creates a temp dir and temp file, then removes them once out of scope
    // This section tests CORR_VER where it is != 2
    with_new_temp_fits_file("test_validate_gpubox_metadata_mwa_version.fits", |fptr| {
        let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

        // This should not succeed- CORR_VER key if it exists should be 2
        // CORR_VER did not exist in OldLegacy or Legacy correlator
        // Now put in a corr version
        hdu.write_key(fptr, "CORR_VER", 1)
            .expect("Couldn't write key 'CORR_VER'");

        assert!(validate_gpubox_metadata_mwa_version(
            fptr,
            &hdu,
            &String::from("test_file.fits"),
            MWAVersion::CorrMWAXv2,
        )
        .is_err());
    });
}

#[test]
fn test_validate_gpubox_metadata_obsid() {
    // with_temp_file creates a temp dir and temp file, then removes them once out of scope
    with_new_temp_fits_file("test_validate_gpubox_metadata_mwa_version.fits", |fptr| {
        let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

        // OBSID is not there, this should be an error
        assert!(validate_gpubox_metadata_obs_id(
            fptr,
            &hdu,
            &String::from("test_file.fits"),
            1_234_567_890,
        )
        .is_err());

        // Now add the key
        hdu.write_key(fptr, "OBSID", 1_234_567_890)
            .expect("Couldn't write key 'OBSID'");

        // OBSID is there, but does not match metafits- this should be an error
        assert!(validate_gpubox_metadata_obs_id(
            fptr,
            &hdu,
            &String::from("test_file.fits"),
            2_345_678_901,
        )
        .is_err());

        // OBSID is there, and it matches
        assert!(validate_gpubox_metadata_obs_id(
            fptr,
            &hdu,
            &String::from("test_file.fits"),
            1_234_567_890,
        )
        .is_ok());
    });
}

#[test]
fn test_populate_provided_timesteps_all() {
    // In this test we have data for 2 timesteps

    // Setup variables to generate gpuboxtimemap
    let coarse_chan101_timesteps: Vec<u64> = vec![1000, 2000];
    let coarse_chan102_timesteps: Vec<u64> = vec![1000, 2000];
    let coarse_chan103_timesteps: Vec<u64> = vec![1000, 2000];
    let coarse_chan104_timesteps: Vec<u64> = vec![1000, 2000];

    // Get a populated GpuboxTimeMap
    let gpubox_time_map = create_determine_common_obs_times_and_chans_test_data(
        coarse_chan101_timesteps,
        coarse_chan102_timesteps,
        coarse_chan103_timesteps,
        coarse_chan104_timesteps,
    );

    let correlator_timesteps = vec![
        TimeStep::new(1000, 1000),
        TimeStep::new(2000, 2000),
        TimeStep::new(3000, 3000),
        TimeStep::new(4000, 4000),
    ];

    let provided_timesteps: Vec<usize> =
        populate_provided_timesteps(&gpubox_time_map, &correlator_timesteps);

    assert_eq!(provided_timesteps.len(), 2);
    assert_eq!(provided_timesteps[0], 0);
    assert_eq!(provided_timesteps[1], 1);
}

#[test]
fn test_populate_provided_timesteps_all_but_spread_out() {
    // In this test we have data 4 timesteps

    // Setup variables to generate gpuboxtimemap
    let coarse_chan101_timesteps: Vec<u64> = vec![1000, 2000];
    let coarse_chan102_timesteps: Vec<u64> = vec![3000, 4000];
    let coarse_chan103_timesteps: Vec<u64> = vec![1000, 2000, 3000];
    let coarse_chan104_timesteps: Vec<u64> = vec![4000];

    // Get a populated GpuboxTimeMap
    let gpubox_time_map = create_determine_common_obs_times_and_chans_test_data(
        coarse_chan101_timesteps,
        coarse_chan102_timesteps,
        coarse_chan103_timesteps,
        coarse_chan104_timesteps,
    );

    let correlator_timesteps = vec![
        TimeStep::new(1000, 1000),
        TimeStep::new(2000, 2000),
        TimeStep::new(3000, 3000),
        TimeStep::new(4000, 4000),
    ];

    let provided_timesteps: Vec<usize> =
        populate_provided_timesteps(&gpubox_time_map, &correlator_timesteps);

    assert_eq!(provided_timesteps.len(), 4);
    assert_eq!(provided_timesteps[0], 0);
    assert_eq!(provided_timesteps[1], 1);
    assert_eq!(provided_timesteps[2], 2);
    assert_eq!(provided_timesteps[3], 3);
}

#[test]
fn test_populate_provided_timesteps_some() {
    // In this test, we have data for 3 timesteps

    // Setup variables to generate gpuboxtimemap
    let coarse_chan101_timesteps: Vec<u64> = vec![1000, 2000];
    let coarse_chan102_timesteps: Vec<u64> = vec![];
    let coarse_chan103_timesteps: Vec<u64> = vec![4000];
    let coarse_chan104_timesteps: Vec<u64> = vec![2000, 4000];

    // Get a populated GpuboxTimeMap
    let gpubox_time_map = create_determine_common_obs_times_and_chans_test_data(
        coarse_chan101_timesteps,
        coarse_chan102_timesteps,
        coarse_chan103_timesteps,
        coarse_chan104_timesteps,
    );

    let correlator_timesteps = vec![
        TimeStep::new(1000, 1000),
        TimeStep::new(2000, 2000),
        TimeStep::new(3000, 3000),
        TimeStep::new(4000, 4000),
    ];

    let provided_timesteps: Vec<usize> =
        populate_provided_timesteps(&gpubox_time_map, &correlator_timesteps);

    assert_eq!(provided_timesteps.len(), 3);
    assert_eq!(provided_timesteps[0], 0);
    assert_eq!(provided_timesteps[1], 1);
    assert_eq!(provided_timesteps[2], 3);
}

#[test]
fn test_populate_provided_coarse_channels_all() {
    // In this test we have data for all coarse chans.

    // Setup variables to generate gpuboxtimemap
    let coarse_chan101_timesteps: Vec<u64> = vec![1000, 2000];
    let coarse_chan102_timesteps: Vec<u64> = vec![1000, 2000];
    let coarse_chan103_timesteps: Vec<u64> = vec![1000, 2000];
    let coarse_chan104_timesteps: Vec<u64> = vec![1000, 2000];

    // Get a populated GpuboxTimeMap
    let gpubox_time_map = create_determine_common_obs_times_and_chans_test_data(
        coarse_chan101_timesteps,
        coarse_chan102_timesteps,
        coarse_chan103_timesteps,
        coarse_chan104_timesteps,
    );

    let correlator_coarse_chans = vec![
        CoarseChannel::new(1, 101, 101, 1_280_000),
        CoarseChannel::new(2, 102, 102, 1_280_000),
        CoarseChannel::new(3, 103, 103, 1_280_000),
        CoarseChannel::new(4, 104, 104, 1_280_000),
    ];

    let provided_coarse_chans: Vec<usize> =
        populate_provided_coarse_channels(&gpubox_time_map, &correlator_coarse_chans);

    assert_eq!(provided_coarse_chans.len(), 4);
    assert_eq!(provided_coarse_chans[0], 0);
    assert_eq!(provided_coarse_chans[1], 1);
    assert_eq!(provided_coarse_chans[2], 2);
    assert_eq!(provided_coarse_chans[3], 3);
}

#[test]
fn test_populate_provided_coarse_channels_all_but_spread_out() {
    // In this test we have data for all coarse chans, but the chans are spread out across time (ie not all chans for all times).

    // Setup variables to generate gpuboxtimemap
    let coarse_chan101_timesteps: Vec<u64> = vec![1000, 2000];
    let coarse_chan102_timesteps: Vec<u64> = vec![3000, 4000];
    let coarse_chan103_timesteps: Vec<u64> = vec![1000, 2000, 3000];
    let coarse_chan104_timesteps: Vec<u64> = vec![4000];

    // Get a populated GpuboxTimeMap
    let gpubox_time_map = create_determine_common_obs_times_and_chans_test_data(
        coarse_chan101_timesteps,
        coarse_chan102_timesteps,
        coarse_chan103_timesteps,
        coarse_chan104_timesteps,
    );

    let correlator_coarse_chans = vec![
        CoarseChannel::new(1, 101, 101, 1_280_000),
        CoarseChannel::new(2, 102, 102, 1_280_000),
        CoarseChannel::new(3, 103, 103, 1_280_000),
        CoarseChannel::new(4, 104, 104, 1_280_000),
    ];

    let provided_coarse_chans: Vec<usize> =
        populate_provided_coarse_channels(&gpubox_time_map, &correlator_coarse_chans);

    assert_eq!(provided_coarse_chans.len(), 4);
    assert_eq!(provided_coarse_chans[0], 0);
    assert_eq!(provided_coarse_chans[1], 1);
    assert_eq!(provided_coarse_chans[2], 2);
    assert_eq!(provided_coarse_chans[3], 3);
}

#[test]
fn test_populate_provided_coarse_channels_some() {
    // In this test, the metafits has 4 coarse chans, but we only have data for 101,103 and 104

    // Setup variables to generate gpuboxtimemap
    let coarse_chan101_timesteps: Vec<u64> = vec![1000];
    let coarse_chan102_timesteps: Vec<u64> = vec![];
    let coarse_chan103_timesteps: Vec<u64> = vec![1000];
    let coarse_chan104_timesteps: Vec<u64> = vec![1000];

    // Get a populated GpuboxTimeMap
    let gpubox_time_map = create_determine_common_obs_times_and_chans_test_data(
        coarse_chan101_timesteps,
        coarse_chan102_timesteps,
        coarse_chan103_timesteps,
        coarse_chan104_timesteps,
    );

    let correlator_coarse_chans = vec![
        CoarseChannel::new(1, 101, 101, 1_280_000),
        CoarseChannel::new(2, 102, 102, 1_280_000),
        CoarseChannel::new(3, 103, 103, 1_280_000),
        CoarseChannel::new(4, 104, 104, 1_280_000),
    ];

    let provided_coarse_chans: Vec<usize> =
        populate_provided_coarse_channels(&gpubox_time_map, &correlator_coarse_chans);

    assert_eq!(provided_coarse_chans.len(), 3);
    assert_eq!(provided_coarse_chans[0], 0);
    assert_eq!(provided_coarse_chans[1], 2);
    assert_eq!(provided_coarse_chans[2], 3);
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
    assert!(o.is_some());
    let o = o.unwrap();

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
    assert!(o_good.is_some());
    let o_good = o_good.unwrap();

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
    assert!(o_good2.is_none());

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
    assert!(o_good3.is_some());
    let o_good3 = o_good3.unwrap();

    // Check contents is what is expected
    assert_eq!(o_good3.start_time_unix_ms, 1000);
    assert_eq!(o_good3.end_time_unix_ms, 2000);
    assert_eq!(o_good3.duration_ms, 1000);
    assert_eq!(o_good3.coarse_chan_identifiers, vec![101, 102, 103, 104]);
}

#[test]
fn test_determine_common_obs_times_and_chans_no_common() {
    // Scenario- all 4 coarse chans have no common timesteps
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
    assert!(o.is_none());

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
    assert!(o_good.is_none());
}

#[test]
fn test_determine_common_obs_times_and_chans_two_common() {
    // Scenario- 2000-3000  and 4000-5000 have 2 common, but we take the first (2000-3000)
    //        1000 2000 3000 4000 5000 6000
    // chan101  X    X    X         X   X
    // chan102       X    X         X   X
    // chan103       X    X    X    X   X
    // chan104       X    X    X    X   X
    //

    // Set corr integration time
    let corr_int_time_ms = 1000;

    // Setup variables to generate gpuboxtimemap
    let coarse_chan101_timesteps: Vec<u64> = vec![1000, 2000, 3000, 5000, 6000];
    let coarse_chan102_timesteps: Vec<u64> = vec![2000, 3000, 5000, 6000];
    let coarse_chan103_timesteps: Vec<u64> = vec![2000, 3000, 4000, 5000, 6000];
    let coarse_chan104_timesteps: Vec<u64> = vec![2000, 3000, 4000, 5000, 6000];

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
    assert!(o.is_some());
    let o = o.unwrap();

    // Check contents of common timesteps and coarse channels is what is expected
    assert_eq!(o.end_time_unix_ms - o.start_time_unix_ms, o.duration_ms);
    assert_eq!(o.start_time_unix_ms, 2000);
    assert_eq!(o.end_time_unix_ms, 4000);
    assert_eq!(o.duration_ms, 2000);
    assert_eq!(o.coarse_chan_identifiers, vec![101, 102, 103, 104]);

    //
    // Now run the same test, but with a good time of 4000
    //
    // Actually run our test!
    let result_good =
        determine_common_obs_times_and_chans(&gpubox_time_map, corr_int_time_ms, Some(4000));

    // Check we did not encounter an error
    assert!(result_good.is_ok());

    // Unwrap to get the obstimesandchans struct
    let o_good = result_good.unwrap();
    assert!(o_good.is_some());
    let o_good = o_good.unwrap();

    // Check contents is what is expected
    assert_eq!(
        o_good.end_time_unix_ms - o_good.start_time_unix_ms,
        o_good.duration_ms
    );
    assert_eq!(o_good.start_time_unix_ms, 5000);
    assert_eq!(o_good.end_time_unix_ms, 7000);
    assert_eq!(o_good.duration_ms, 2000);
    assert_eq!(o_good.coarse_chan_identifiers, vec![101, 102, 103, 104]);
}

#[test]
fn test_determine_common_obs_times_and_chans_two_then_three() {
    // Scenario- there is a run of 2 chans (1000-2000) then a different set of 2 chans for 1 ts then 4 chans, 1 chan, 4 chans (4000,5000,6000).
    //        1000 2000 3000 4000 5000 6000
    // chan101 X     X         X         X
    // chan102 X     X         X         X
    // chan103            X    X    X    X
    // chan104            X    X         X
    //

    // Set corr integration time
    let corr_int_time_ms = 1000;

    // Setup variables to generate gpuboxtimemap
    let coarse_chan101_timesteps: Vec<u64> = vec![1000, 2000, 4000, 6000];
    let coarse_chan102_timesteps: Vec<u64> = vec![1000, 2000, 4000, 6000];
    let coarse_chan103_timesteps: Vec<u64> = vec![3000, 4000, 5000, 6000];
    let coarse_chan104_timesteps: Vec<u64> = vec![3000, 4000, 6000];

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
    assert!(o.is_some());
    let o = o.unwrap();

    // Check contents of common timesteps and coarse channels is what is expected
    assert_eq!(o.end_time_unix_ms - o.start_time_unix_ms, o.duration_ms);
    assert_eq!(o.start_time_unix_ms, 4000);
    assert_eq!(o.end_time_unix_ms, 5000);
    assert_eq!(o.duration_ms, 1000);
    assert_eq!(o.coarse_chan_identifiers, vec![101, 102, 103, 104]);

    //
    // Now run the same test, but with a good time
    //
    // Actually run our test!
    let result_good =
        determine_common_obs_times_and_chans(&gpubox_time_map, corr_int_time_ms, Some(5000));

    // Check we did not encounter an error
    assert!(result_good.is_ok());

    // Unwrap to get the obstimesandchans struct
    let o_good = result_good.unwrap();
    assert!(o_good.is_some());
    let o_good = o_good.unwrap();

    // Check contents is what is expected
    assert_eq!(
        o_good.end_time_unix_ms - o_good.start_time_unix_ms,
        o_good.duration_ms
    );
    assert_eq!(o_good.start_time_unix_ms, 6000);
    assert_eq!(o_good.end_time_unix_ms, 7000);
    assert_eq!(o_good.duration_ms, 1000);
    assert_eq!(o_good.coarse_chan_identifiers, vec![101, 102, 103, 104]);
}

#[test]
fn test_determine_common_obs_times_and_chans_non_contiguous() {
    // Scenario- there is a run of 2 chans (1000-2000) then a different set of 2 chans for 1 ts then 3 chans in a 1 timestep run (4000), a gap (5000), then 4 chans for (6000).
    //        1000 2000 3000 4000 5000 6000
    // chan101 X     X                   X
    // chan102 X     X         X         X
    // chan103            X    X         X
    // chan104            X    X         X
    //

    // Set corr integration time
    let corr_int_time_ms = 1000;

    // Setup variables to generate gpuboxtimemap
    let coarse_chan101_timesteps: Vec<u64> = vec![1000, 2000, 5000, 6000];
    let coarse_chan102_timesteps: Vec<u64> = vec![1000, 2000, 4000, 5000, 6000];
    let coarse_chan103_timesteps: Vec<u64> = vec![3000, 4000, 5000, 6000];
    let coarse_chan104_timesteps: Vec<u64> = vec![3000, 4000, 6000];

    // Get a populated GpuboxTimeMap
    let gpubox_time_map = create_determine_common_obs_times_and_chans_test_data(
        coarse_chan101_timesteps,
        coarse_chan102_timesteps,
        coarse_chan103_timesteps,
        coarse_chan104_timesteps,
    );

    // Actually run our test!
    let result =
        determine_common_obs_times_and_chans(&gpubox_time_map, corr_int_time_ms, Some(3000));

    // Check we did not encounter an error
    assert!(result.is_ok());

    // Unwrap to get the obstimesandchans struct
    let o = result.unwrap();
    assert!(o.is_some());
    let o = o.unwrap();

    // Check contents of common timesteps and coarse channels is what is expected
    assert_eq!(o.end_time_unix_ms - o.start_time_unix_ms, o.duration_ms);
    assert_eq!(o.start_time_unix_ms, 6000);
    assert_eq!(o.end_time_unix_ms, 7000);
    assert_eq!(o.duration_ms, 1000);
    assert_eq!(o.coarse_chan_identifiers, vec![101, 102, 103, 104]);

    //
    // Now run the same test, but with a good time of 6000
    //
    // Actually run our test!
    let result_good =
        determine_common_obs_times_and_chans(&gpubox_time_map, corr_int_time_ms, Some(6000));

    // Check we did not encounter an error
    assert!(result_good.is_ok());

    // Unwrap to get the obstimesandchans struct
    let o_good = result_good.unwrap();
    assert!(o_good.is_some());
    let o_good = o_good.unwrap();

    // Check contents is what is expected
    assert_eq!(
        o_good.end_time_unix_ms - o_good.start_time_unix_ms,
        o_good.duration_ms
    );
    assert_eq!(o_good.start_time_unix_ms, 6000);
    assert_eq!(o_good.end_time_unix_ms, 7000);
    assert_eq!(o_good.duration_ms, 1000);
    assert_eq!(o_good.coarse_chan_identifiers, vec![101, 102, 103, 104]);
}
