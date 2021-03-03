// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Unit tests for timestep metadata
*/
#[cfg(test)]
use super::*;

#[test]
fn test_populate_correlator_timesteps() {
    // Create a dummy BTree GPUbox map
    let mut gpubox_time_map = BTreeMap::new();

    let times: Vec<u64> = vec![
        1_381_844_923_000,
        1_381_844_923_500,
        1_381_844_924_000,
        1_381_844_924_500,
        1_381_844_925_000,
        1_381_844_925_500,
    ];

    for (i, time) in times.iter().enumerate() {
        let mut new_time_tree = BTreeMap::new();
        // gpubox 0.
        new_time_tree.insert(0, (0, i));
        // gpubox 1.
        new_time_tree.insert(1, (0, i + 1));
        gpubox_time_map.insert(*time, new_time_tree);
    }

    // Get a vector timesteps
    let scheduled_start_gpstime_ms = 1065880139_000;
    let scheduled_start_unix_ms = 1381844923_000;
    let timesteps = TimeStep::populate_correlator_timesteps(
        &gpubox_time_map,
        scheduled_start_gpstime_ms,
        scheduled_start_unix_ms,
    )
    .unwrap();

    // Check
    assert_eq!(6, timesteps.len());
    assert_eq!(timesteps[0].unix_time_ms, 1_381_844_923_000);
    assert_eq!(timesteps[0].gps_time_ms, 1_065_880_139_000);
    assert_eq!(timesteps[5].unix_time_ms, 1_381_844_925_500);
    assert_eq!(timesteps[5].gps_time_ms, 1_065_880_141_500);
}

#[test]
fn test_populate_correlator_timesteps_none() {
    // Create a dummy BTree GPUbox map
    let gpubox_time_map = BTreeMap::new();
    // Get a vector timesteps
    let scheduled_start_gpstime_ms = 0;
    let scheduled_start_unix_ms = 0;
    let timesteps = TimeStep::populate_correlator_timesteps(
        &gpubox_time_map,
        scheduled_start_gpstime_ms,
        scheduled_start_unix_ms,
    );

    // Check
    assert!(timesteps.is_none());
}

#[test]
fn test_timestep_new() {
    // This test is a bit of a waste right now but it will be useful once
    // julian date and possibly UTC conversions are done in the new() method
    let timestep = TimeStep {
        unix_time_ms: 1_381_844_923_000,
        gps_time_ms: 1_065_880_139_000,
    };
    let new_timestep = TimeStep::new(1_381_844_923_000, 1_065_880_139_000);

    assert_eq!(timestep.unix_time_ms, new_timestep.unix_time_ms,);
    assert_eq!(timestep.gps_time_ms, new_timestep.gps_time_ms);
}

#[test]
fn test_populate_voltage_timesteps_oldlegacy() {
    let scheduled_start_gpstime_ms = 1_065_880_139_000;
    let scheduled_start_unix_ms = 1_381_844_923_000;
    let timesteps = TimeStep::populate_voltage_timesteps(
        1_065_880_139_000,
        1_065_880_143_000,
        1000,
        scheduled_start_gpstime_ms,
        scheduled_start_unix_ms,
    );
    assert_eq!(timesteps.len(), 4);
    assert_eq!(timesteps[0].gps_time_ms, 1_065_880_139_000);
    assert_eq!(timesteps[0].unix_time_ms, 1_381_844_923_000);
    assert_eq!(timesteps[1].gps_time_ms, 1_065_880_140_000);
    assert_eq!(timesteps[1].unix_time_ms, 1_381_844_924_000);
    assert_eq!(timesteps[2].gps_time_ms, 1_065_880_141_000);
    assert_eq!(timesteps[2].unix_time_ms, 1_381_844_925_000);
    assert_eq!(timesteps[3].gps_time_ms, 1_065_880_142_000);
    assert_eq!(timesteps[3].unix_time_ms, 1_381_844_926_000);
}

#[test]
fn test_populate_voltage_timesteps_legacy() {
    let scheduled_start_gpstime_ms = 1_065_880_139_000;
    let scheduled_start_unix_ms = 1_381_844_923_000;

    let timesteps = TimeStep::populate_voltage_timesteps(
        1_065_880_139_000,
        1_065_880_143_000,
        1000,
        scheduled_start_gpstime_ms,
        scheduled_start_unix_ms,
    );
    assert_eq!(timesteps.len(), 4);
    assert_eq!(timesteps[0].gps_time_ms, 1_065_880_139_000);
    assert_eq!(timesteps[0].unix_time_ms, 1_381_844_923_000);
    assert_eq!(timesteps[1].gps_time_ms, 1_065_880_140_000);
    assert_eq!(timesteps[1].unix_time_ms, 1_381_844_924_000);
    assert_eq!(timesteps[2].gps_time_ms, 1_065_880_141_000);
    assert_eq!(timesteps[2].unix_time_ms, 1_381_844_925_000);
    assert_eq!(timesteps[3].gps_time_ms, 1_065_880_142_000);
    assert_eq!(timesteps[3].unix_time_ms, 1_381_844_926_000);
}

#[test]
fn test_populate_voltage_timesteps_mwax() {
    let scheduled_start_gpstime_ms = 1_065_880_139_000;
    let scheduled_start_unix_ms = 1_381_844_923_000;
    let timesteps = TimeStep::populate_voltage_timesteps(
        1_065_880_139_000,
        1_065_880_171_000,
        8000,
        scheduled_start_gpstime_ms,
        scheduled_start_unix_ms,
    );

    assert_eq!(timesteps.len(), 4);
    assert_eq!(timesteps[0].gps_time_ms, 1_065_880_139_000);
    assert_eq!(timesteps[0].unix_time_ms, 1_381_844_923_000);
    assert_eq!(timesteps[1].gps_time_ms, 1_065_880_147_000);
    assert_eq!(timesteps[1].unix_time_ms, 1_381_844_931_000);
    assert_eq!(timesteps[2].gps_time_ms, 1_065_880_155_000);
    assert_eq!(timesteps[2].unix_time_ms, 1_381_844_939_000);
    assert_eq!(timesteps[3].gps_time_ms, 1_065_880_163_000);
    assert_eq!(timesteps[3].unix_time_ms, 1_381_844_947_000);
}
