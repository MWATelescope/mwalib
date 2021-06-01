// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Unit tests for timestep metadata
*/
#[cfg(test)]
use super::*;
use crate::MetafitsContext;

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

    let metafits_timesteps: Vec<TimeStep> = Vec::new();

    for (i, time) in times.iter().enumerate() {
        let mut new_time_tree = BTreeMap::new();
        // gpubox 0.
        new_time_tree.insert(0, (0, i));
        // gpubox 1.
        new_time_tree.insert(1, (0, i + 1));
        gpubox_time_map.insert(*time, new_time_tree);
    }

    // Get a vector timesteps
    let scheduled_start_gpstime_ms = 1_065_880_139_000;
    let scheduled_start_unix_ms = 1_381_844_923_000;
    let timesteps = TimeStep::populate_correlator_timesteps(
        &gpubox_time_map,
        &metafits_timesteps,
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

    let metafits_timesteps: Vec<TimeStep> = Vec::new();

    // Get a vector timesteps
    let scheduled_start_gpstime_ms = 0;
    let scheduled_start_unix_ms = 0;
    let timesteps = TimeStep::populate_correlator_timesteps(
        &gpubox_time_map,
        &metafits_timesteps,
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
fn test_populate_timesteps_metafits_corr() {
    let metafits_file = String::from("test_files/1101503312_1_timestep/1101503312.metafits");

    let versions: Vec<MWAVersion> = vec![
        MWAVersion::CorrOldLegacy,
        MWAVersion::CorrLegacy,
        MWAVersion::CorrMWAXv2,
    ];

    for mwa_version in versions {
        let metafits_context =
            MetafitsContext::new_internal(&metafits_file).expect("Error creating metafits context");

        let timesteps = TimeStep::populate_timesteps(
            &metafits_context,
            mwa_version,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_duration_ms,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
        );

        assert_eq!(timesteps.len(), 56);
        assert_eq!(timesteps[0].gps_time_ms, 1_101_503_312_000);
        assert_eq!(timesteps[0].unix_time_ms, 1_417_468_096_000);
        assert_eq!(timesteps[1].gps_time_ms, 1_101_503_314_000);
        assert_eq!(timesteps[1].unix_time_ms, 1_417_468_098_000);
        assert_eq!(timesteps[2].gps_time_ms, 1_101_503_316_000);
        assert_eq!(timesteps[2].unix_time_ms, 1_417_468_100_000);
        assert_eq!(timesteps[55].gps_time_ms, 1_101_503_422_000);
        assert_eq!(timesteps[55].unix_time_ms, 1_417_468_206_000);
    }
}

#[test]
fn test_populate_timesteps_metafits_vcs_legacy_recombined() {
    let metafits_file = String::from("test_files/1101503312_1_timestep/1101503312.metafits");

    let metafits_context =
        MetafitsContext::new_internal(&metafits_file).expect("Error creating metafits context");

    let timesteps = TimeStep::populate_timesteps(
        &metafits_context,
        MWAVersion::VCSLegacyRecombined,
        metafits_context.sched_start_gps_time_ms,
        metafits_context.sched_duration_ms,
        metafits_context.sched_start_gps_time_ms,
        metafits_context.sched_start_unix_time_ms,
    );

    assert_eq!(timesteps.len(), 112);
    assert_eq!(timesteps[0].gps_time_ms, 1_101_503_312_000);
    assert_eq!(timesteps[0].unix_time_ms, 1_417_468_096_000);
    assert_eq!(timesteps[1].gps_time_ms, 1_101_503_313_000);
    assert_eq!(timesteps[1].unix_time_ms, 1_417_468_097_000);
    assert_eq!(timesteps[2].gps_time_ms, 1_101_503_314_000);
    assert_eq!(timesteps[2].unix_time_ms, 1_417_468_098_000);
    assert_eq!(timesteps[111].gps_time_ms, 1_101_503_423_000);
    assert_eq!(timesteps[111].unix_time_ms, 1_417_468_207_000);
}

#[test]
fn test_populate_timesteps_metafits_vcs_mwaxv2() {
    let metafits_file = String::from("test_files/1101503312_1_timestep/1101503312.metafits");

    let metafits_context =
        MetafitsContext::new_internal(&metafits_file).expect("Error creating metafits context");

    let timesteps = TimeStep::populate_timesteps(
        &metafits_context,
        MWAVersion::VCSMWAXv2,
        metafits_context.sched_start_gps_time_ms,
        metafits_context.sched_duration_ms,
        metafits_context.sched_start_gps_time_ms,
        metafits_context.sched_start_unix_time_ms,
    );

    assert_eq!(timesteps.len(), 14);
    assert_eq!(timesteps[0].gps_time_ms, 1_101_503_312_000);
    assert_eq!(timesteps[0].unix_time_ms, 1_417_468_096_000);
    assert_eq!(timesteps[1].gps_time_ms, 1_101_503_320_000);
    assert_eq!(timesteps[1].unix_time_ms, 1_417_468_104_000);
    assert_eq!(timesteps[2].gps_time_ms, 1_101_503_328_000);
    assert_eq!(timesteps[2].unix_time_ms, 1_417_468_112_000);
    assert_eq!(timesteps[13].gps_time_ms, 1_101_503_416_000);
    assert_eq!(timesteps[13].unix_time_ms, 1_417_468_200_000);
}
