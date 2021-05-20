// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Unit tests for coarse channel metadata
*/
#[cfg(test)]
use super::*;
use std::collections::BTreeMap;

// Create a BTree Structure for testing
#[cfg(test)]
fn get_gpubox_time_map(sub_map_keys: Vec<usize>) -> GpuboxTimeMap {
    let mut sub_map = BTreeMap::new();
    for i in sub_map_keys {
        sub_map.insert(i, (0, 1));
    }
    let mut gpubox_time_map = BTreeMap::new();
    gpubox_time_map.insert(1_381_844_923_000, sub_map);
    gpubox_time_map
}

// Create a BTree Structure for testing
#[cfg(test)]
fn get_voltage_time_map(sub_map_keys: Vec<usize>) -> VoltageFileTimeMap {
    let mut sub_map = BTreeMap::new();
    for i in sub_map_keys {
        sub_map.insert(i, String::from("filename"));
    }
    let mut voltage_file_time_map = BTreeMap::new();
    voltage_file_time_map.insert(1_234_567_890, sub_map);
    voltage_file_time_map
}

#[test]
fn test_get_metafits_coarse_chan_array() {
    assert_eq!(
        8,
        CoarseChannel::get_metafits_coarse_chan_array("0,1,2,3,127,128,129,255").len()
    );
}

#[test]
/// Tests coarse channel processing for a Legacy observation where we don't have all the coarse channels
/// What we expect from metafits is 4 coarse channels but we only get the middle 2
/// Metafits has: 109,110,111,112
/// User supplied gpuboxes 2 and 3 (which when 0 indexed is 1 and 2)
/// So:
/// 109 ==  missing
/// 110 == gpubox02 == correlator index 1
/// 111 == gpubox03 == correlator index 2
/// 112 == missing
fn test_process_coarse_chans_legacy_middle_two_gpuboxes() {
    // Create the BTree Structure for an simple test which has 2 coarse channels
    let gpubox_time_map = get_gpubox_time_map((2..=3).collect());

    // Metafits coarse channel array
    let metafits_chan_array = vec![109, 110, 111, 112];

    // Process coarse channels
    let result = CoarseChannel::populate_coarse_channels(
        MWAVersion::CorrLegacy,
        &metafits_chan_array,
        1_280_000,
        Some(&gpubox_time_map),
        None,
    );

    assert!(result.is_ok());

    let coarse_chan_array = result.unwrap();

    assert_eq!(coarse_chan_array.len(), 2);
    assert_eq!(coarse_chan_array[0].corr_chan_number, 1);
    assert_eq!(coarse_chan_array[0].rec_chan_number, 110);
    assert_eq!(coarse_chan_array[0].gpubox_number, 2);
    assert_eq!(coarse_chan_array[1].corr_chan_number, 2);
    assert_eq!(coarse_chan_array[1].rec_chan_number, 111);
    assert_eq!(coarse_chan_array[1].gpubox_number, 3);
}

#[test]
/// Tests coarse channel processing when we have a legacy observation
/// and the coarse channels span the 128 mark, thereby reversing
/// Input from Legacy metafits:
/// receiver channels: 126,127,128,129,130
/// this would map to correlator indexes: 0,1,2,4,3
fn test_process_coarse_chans_legacy_chan_reversal() {
    // Create the BTree Structure for an simple test which has 5 coarse channels
    let gpubox_time_map = get_gpubox_time_map((1..=5).collect());

    // Metafits coarse channel array
    let metafits_chan_array = vec![126, 127, 128, 129, 130];

    // Process coarse channels
    let result = CoarseChannel::populate_coarse_channels(
        MWAVersion::CorrLegacy,
        &metafits_chan_array,
        1_280_000,
        Some(&gpubox_time_map),
        None,
    );

    assert!(result.is_ok());

    let coarse_chan_array = result.unwrap();

    assert_eq!(coarse_chan_array.len(), 5);
    assert_eq!(coarse_chan_array[0].corr_chan_number, 0);
    assert_eq!(coarse_chan_array[0].rec_chan_number, 126);
    assert_eq!(coarse_chan_array[0].gpubox_number, 1);
    assert_eq!(coarse_chan_array[1].corr_chan_number, 1);
    assert_eq!(coarse_chan_array[1].rec_chan_number, 127);
    assert_eq!(coarse_chan_array[1].gpubox_number, 2);
    assert_eq!(coarse_chan_array[2].corr_chan_number, 2);
    assert_eq!(coarse_chan_array[2].rec_chan_number, 128);
    assert_eq!(coarse_chan_array[2].gpubox_number, 3);
    assert_eq!(coarse_chan_array[3].corr_chan_number, 4);
    assert_eq!(coarse_chan_array[3].rec_chan_number, 129);
    assert_eq!(coarse_chan_array[3].gpubox_number, 5);
    assert_eq!(coarse_chan_array[4].corr_chan_number, 3);
    assert_eq!(coarse_chan_array[4].rec_chan_number, 130);
    assert_eq!(coarse_chan_array[4].gpubox_number, 4);
}

#[test]
/// Tests coarse channel processing for a Legacy observation where we don't have all the coarse channels
/// What we expect from metafits is 4 coarse channels but we only get the first and last
/// Metafits has: 109,110,111,112
/// User supplied gpuboxes 1 and 4 (which when 0 indexed is 0 and 3)
/// So:
/// 109 == gpubox01 == correlator index 0
/// 110 == missing
/// 111 == missing
/// 112 == gpubox04 == correlator index 3
fn test_process_coarse_chans_legacy_first_and_last() {
    // Create the BTree Structure for an simple test which has 2 coarse channels
    let gpubox_time_map = get_gpubox_time_map(vec![1, 4]);

    // Metafits coarse channel array
    let metafits_chan_array = vec![109, 110, 111, 112];

    // Process coarse channels
    let result = CoarseChannel::populate_coarse_channels(
        MWAVersion::CorrLegacy,
        &metafits_chan_array,
        1_280_000,
        Some(&gpubox_time_map),
        None,
    );

    assert!(result.is_ok());

    let coarse_chan_array = result.unwrap();

    assert_eq!(coarse_chan_array.len(), 2);
    assert_eq!(coarse_chan_array[0].corr_chan_number, 0);
    assert_eq!(coarse_chan_array[0].rec_chan_number, 109);
    assert_eq!(coarse_chan_array[0].gpubox_number, 1);
    assert_eq!(coarse_chan_array[1].corr_chan_number, 3);
    assert_eq!(coarse_chan_array[1].rec_chan_number, 112);
    assert_eq!(coarse_chan_array[1].gpubox_number, 4);
}

#[test]
/// Tests coarse channel processing when we have a MWAX observation
/// and the coarse channels span the 128 mark. In this case we DO NOT reverse coarse channels post 128
/// Input from MWAX metafits:
/// receiver channels: 126,127,128,129,130
/// this would map to correlator indexes: 0,1,2,3,4
fn test_process_coarse_chans_mwax_no_reverse() {
    // Create the BTree Structure for an simple test which has 5 coarse channels
    let gpubox_time_map = get_gpubox_time_map(vec![126, 127, 128, 129, 130]);

    // Metafits coarse channel array
    let metafits_chan_array = vec![126, 127, 128, 129, 130];

    // Process coarse channels
    let result = CoarseChannel::populate_coarse_channels(
        MWAVersion::CorrMWAXv2,
        &metafits_chan_array,
        1_280_000,
        Some(&gpubox_time_map),
        None,
    );

    assert!(result.is_ok());

    let coarse_chan_array = result.unwrap();

    assert_eq!(coarse_chan_array.len(), 5);
    assert_eq!(coarse_chan_array[0].corr_chan_number, 0);
    assert_eq!(coarse_chan_array[0].rec_chan_number, 126);
    assert_eq!(coarse_chan_array[0].gpubox_number, 126);
    assert_eq!(coarse_chan_array[1].corr_chan_number, 1);
    assert_eq!(coarse_chan_array[1].rec_chan_number, 127);
    assert_eq!(coarse_chan_array[1].gpubox_number, 127);
    assert_eq!(coarse_chan_array[2].corr_chan_number, 2);
    assert_eq!(coarse_chan_array[2].rec_chan_number, 128);
    assert_eq!(coarse_chan_array[2].gpubox_number, 128);
    assert_eq!(coarse_chan_array[3].corr_chan_number, 3);
    assert_eq!(coarse_chan_array[3].rec_chan_number, 129);
    assert_eq!(coarse_chan_array[3].gpubox_number, 129);
    assert_eq!(coarse_chan_array[4].corr_chan_number, 4);
    assert_eq!(coarse_chan_array[4].rec_chan_number, 130);
    assert_eq!(coarse_chan_array[4].gpubox_number, 130);
}

#[test]
/// This test exposed a bug which is triggered when a legacy observation has
/// all coarse channel numbers > 128 (typical for EoR).
fn test_process_coarse_chans_legacy_eor() {
    let gpubox_time_map = get_gpubox_time_map((1..=3).collect());
    let metafits_chan_array: Vec<_> = (133..=135).collect();
    let channel_width = 1_280_000;

    // Process coarse channels
    let result = CoarseChannel::populate_coarse_channels(
        MWAVersion::CorrLegacy,
        &metafits_chan_array,
        channel_width,
        Some(&gpubox_time_map),
        None,
    );

    assert!(result.is_ok());

    let coarse_chan_array = result.unwrap();

    assert_eq!(coarse_chan_array.len(), 3);
    assert_eq!(coarse_chan_array[0].corr_chan_number, 2);
    assert_eq!(coarse_chan_array[0].rec_chan_number, 133);
    assert_eq!(coarse_chan_array[0].gpubox_number, 3);
    assert_eq!(coarse_chan_array[1].corr_chan_number, 1);
    assert_eq!(coarse_chan_array[1].rec_chan_number, 134);
    assert_eq!(coarse_chan_array[1].gpubox_number, 2);
    assert_eq!(coarse_chan_array[2].corr_chan_number, 0);
    assert_eq!(coarse_chan_array[2].rec_chan_number, 135);
    assert_eq!(coarse_chan_array[2].gpubox_number, 1);
}

#[test]
fn test_process_coarse_chans_no_time_maps_legacy() {
    let metafits_chan_array: Vec<_> = (133..=135).collect();
    let channel_width = 1_280_000;

    // Process coarse channels
    let result = CoarseChannel::populate_coarse_channels(
        MWAVersion::CorrLegacy,
        &metafits_chan_array,
        channel_width,
        None,
        None,
    );

    assert!(result.is_ok());

    let coarse_chan_array = result.unwrap();

    assert_eq!(coarse_chan_array.len(), 3);
    assert_eq!(coarse_chan_array[0].rec_chan_number, 133);
    assert_eq!(coarse_chan_array[0].corr_chan_number, 2);
    assert_eq!(coarse_chan_array[0].gpubox_number, 3);
    assert_eq!(coarse_chan_array[1].rec_chan_number, 134);
    assert_eq!(coarse_chan_array[1].corr_chan_number, 1);
    assert_eq!(coarse_chan_array[1].gpubox_number, 2);
    assert_eq!(coarse_chan_array[2].rec_chan_number, 135);
    assert_eq!(coarse_chan_array[2].corr_chan_number, 0);
    assert_eq!(coarse_chan_array[2].gpubox_number, 1);
}

#[test]
fn test_process_coarse_chans_no_time_maps_mwax_v2() {
    let metafits_chan_array: Vec<_> = (133..=135).collect();
    let channel_width = 1_280_000;

    // Process coarse channels
    let result = CoarseChannel::populate_coarse_channels(
        MWAVersion::CorrMWAXv2,
        &metafits_chan_array,
        channel_width,
        None,
        None,
    );

    assert!(result.is_ok());

    let coarse_chan_array = result.unwrap();

    assert_eq!(coarse_chan_array.len(), 3);
    assert_eq!(coarse_chan_array[0].rec_chan_number, 133);
    assert_eq!(coarse_chan_array[0].corr_chan_number, 0);
    assert_eq!(coarse_chan_array[0].gpubox_number, 133);
    assert_eq!(coarse_chan_array[1].rec_chan_number, 134);
    assert_eq!(coarse_chan_array[1].corr_chan_number, 1);
    assert_eq!(coarse_chan_array[1].gpubox_number, 134);
    assert_eq!(coarse_chan_array[2].rec_chan_number, 135);
    assert_eq!(coarse_chan_array[2].corr_chan_number, 2);
    assert_eq!(coarse_chan_array[2].gpubox_number, 135);
}

#[test]
fn test_process_coarse_chans_both_time_maps() {
    let gpubox_time_map = get_gpubox_time_map((1..=3).collect());
    let voltage_time_map = get_voltage_time_map((1..=3).collect());
    let metafits_chan_array: Vec<_> = (133..=135).collect();
    let channel_width = 1_280_000;

    // Process coarse channels for legacy
    let result1 = CoarseChannel::populate_coarse_channels(
        MWAVersion::CorrLegacy,
        &metafits_chan_array,
        channel_width,
        Some(&gpubox_time_map),
        Some(&voltage_time_map),
    );

    assert!(matches!(
        result1.unwrap_err(),
        MwalibError::CoarseChannel(CoarseChannelError::BothGpuboxAndVoltageTimeMapSupplied)
    ));

    // v2
    let result2 = CoarseChannel::populate_coarse_channels(
        MWAVersion::CorrMWAXv2,
        &metafits_chan_array,
        channel_width,
        Some(&gpubox_time_map),
        Some(&voltage_time_map),
    );

    assert!(matches!(
        result2.unwrap_err(),
        MwalibError::CoarseChannel(CoarseChannelError::BothGpuboxAndVoltageTimeMapSupplied)
    ));
}

#[test]
fn test_coarse_chan_debug() {
    let cc = CoarseChannel::new(1, 109, 2, 1_280_000);

    assert_eq!(format!("{:?}", cc), "gpu=2 corr=1 rec=109 @ 139.520 MHz");
}
