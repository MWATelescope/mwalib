// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Unit tests for timestep metadata
*/
#[cfg(test)]
use super::*;
use crate::gpubox_files::GpuboxTimeMap;
use crate::MetafitsContext;

///
/// This is helper method for many of the tests for generating the correlator timesteps
/// Returns:
/// * Vector of metafits timesteps
/// * GpuboxTimeMap of correlator data files
/// * Option, containing a populated Vector of correlator timesteps (or None). This is what we are checking for validity.
///
#[cfg(test)]
fn create_corr_timestep_testdata(
    scheduled_start_unix_ms: u64,
    scheduled_start_gpstime_ms: u64,
    scheduled_duration_ms: u64,
    corr_int_time_ms: u64,
    data_timesteps_unix_ms: Vec<u64>,
) -> (Vec<TimeStep>, GpuboxTimeMap, Option<Vec<TimeStep>>) {
    // Create metafits timesteps vec
    let mut metafits_timesteps: Vec<TimeStep> = Vec::new();

    // populate metafits timesteps
    for (t_index, t_time) in (scheduled_start_unix_ms
        ..(scheduled_start_unix_ms + scheduled_duration_ms))
        .step_by(corr_int_time_ms as usize)
        .enumerate()
    {
        metafits_timesteps.push(TimeStep::new(
            t_time,
            1_065_880_139_000 + (t_index as u64 * corr_int_time_ms),
        ));
    }

    // Now create a GpuboxtimeMap based on passed in Data unix time vector
    // Create a dummy BTree GPUbox map
    let mut gpubox_time_map = BTreeMap::new();

    // Populate gpubox time map
    for (i, time) in data_timesteps_unix_ms.iter().enumerate() {
        let mut new_time_tree = BTreeMap::new();
        // gpubox 0.
        new_time_tree.insert(0, (0, i + 1));
        // gpubox 1.
        new_time_tree.insert(1, (0, i + 1));
        gpubox_time_map.insert(*time, new_time_tree);
    }

    // perform the actual test
    let correlator_timesteps = TimeStep::populate_correlator_timesteps(
        &gpubox_time_map,
        &metafits_timesteps,
        scheduled_start_gpstime_ms,
        scheduled_start_unix_ms,
        corr_int_time_ms,
    );

    (metafits_timesteps, gpubox_time_map, correlator_timesteps)
}

#[test]
fn test_populate_correlator_timesteps_simple() {
    // In this scenario the data matches the metafits timesteps 1:1
    // metafits: [1_381_844_923_000, 1_381_844_923_500, 1_381_844_924_000, 1_381_844_924_500, 1_381_844_925_000, 1_381_844_925_500]
    // data    : [1_381_844_923_000, 1_381_844_923_500, 1_381_844_924_000, 1_381_844_924_500, 1_381_844_925_000, 1_381_844_925_500]
    //
    let (metafits_timesteps, _, correlator_timesteps) = create_corr_timestep_testdata(
        1_381_844_923_000,
        1_065_880_139_000,
        3000,
        500,
        vec![
            1_381_844_923_000,
            1_381_844_923_500,
            1_381_844_924_000,
            1_381_844_924_500,
            1_381_844_925_000,
            1_381_844_925_500,
        ],
    );

    // First check metafits is what we expect- this is testing the helper code!
    assert_eq!(metafits_timesteps.len(), 6);

    // Check returned correlator timesteps
    assert!(correlator_timesteps.is_some());
    let timesteps = correlator_timesteps.unwrap();

    assert_eq!(6, timesteps.len());
    assert_eq!(timesteps[0].unix_time_ms, 1_381_844_923_000);
    assert_eq!(timesteps[0].gps_time_ms, 1_065_880_139_000);
    assert_eq!(timesteps[5].unix_time_ms, 1_381_844_925_500);
    assert_eq!(timesteps[5].gps_time_ms, 1_065_880_141_500);
}

#[test]
fn test_populate_correlator_timesteps_data_earlier_and_later_than_metafits() {
    // In this scenario, the metafits first timestep starts AFTER the first data timestep
    // also the metafits last timestep is BEFORE the last data timestep
    // e.g.
    //
    // metafits:                                       [1_381_844_923_000, 1_381_844_923_500, 1_381_844_924_000, 1_381_844_924_500, 1_381_844_925_000, 1_381_844_925_500]
    // data:     [1_381_844_922_000, 1_381_844_922_500, 1_381_844_923_000, 1_381_844_923_500, 1_381_844_924_000, 1_381_844_924_500, 1_381_844_925_000, 1_381_844_925_500, 1_381_844_926_000]
    //
    let (metafits_timesteps, _, correlator_timesteps) = create_corr_timestep_testdata(
        1_381_844_923_000,
        1_065_880_139_000,
        3000,
        500,
        vec![
            1_381_844_922_000,
            1_381_844_922_500,
            1_381_844_923_000,
            1_381_844_923_500,
            1_381_844_924_000,
            1_381_844_924_500,
            1_381_844_925_000,
            1_381_844_925_500,
            1_381_844_926_000,
        ],
    );

    // First check metafits is what we expect- this is testing the helper code!
    assert_eq!(metafits_timesteps.len(), 6);

    // Check returned correlator timesteps
    assert!(correlator_timesteps.is_some());
    let timesteps = correlator_timesteps.unwrap();

    assert_eq!(9, timesteps.len());
    assert_eq!(timesteps[0].unix_time_ms, 1_381_844_922_000);
    assert_eq!(timesteps[0].gps_time_ms, 1_065_880_138_000);
    assert_eq!(timesteps[2].unix_time_ms, 1_381_844_923_000);
    assert_eq!(timesteps[2].gps_time_ms, 1_065_880_139_000);
    assert_eq!(timesteps[7].unix_time_ms, 1_381_844_925_500);
    assert_eq!(timesteps[7].gps_time_ms, 1_065_880_141_500);
    assert_eq!(timesteps[8].unix_time_ms, 1_381_844_926_000);
    assert_eq!(timesteps[8].gps_time_ms, 1_065_880_142_000);
}

#[test]
fn test_populate_correlator_timesteps_data_between_metafits_start_and_end() {
    // In this scenario, the metafits first timestep starts BEFORE the first data timestep
    // also the metafits last timestep is AFTER the last data timestep
    // e.g.
    //
    // metafits: [1_381_844_923_000, 1_381_844_923_500, 1_381_844_924_000, 1_381_844_924_500, 1_381_844_925_000, 1_381_844_925_500]
    // data:                        [1_381_844_923_500, 1_381_844_924_000, 1_381_844_924_500, 1_381_844_925_000]
    //
    let (metafits_timesteps, _, correlator_timesteps) = create_corr_timestep_testdata(
        1_381_844_923_000,
        1_065_880_139_000,
        3000,
        500,
        vec![
            1_381_844_923_500,
            1_381_844_924_000,
            1_381_844_924_500,
            1_381_844_925_000,
        ],
    );

    // First check metafits is what we expect- this is testing the helper code!
    assert_eq!(metafits_timesteps.len(), 6);

    // Check returned correlator timesteps
    assert!(correlator_timesteps.is_some());
    let timesteps = correlator_timesteps.unwrap();

    // Check
    assert_eq!(6, timesteps.len());
    assert_eq!(timesteps[0].unix_time_ms, 1_381_844_923_000);
    assert_eq!(timesteps[0].gps_time_ms, 1_065_880_139_000);
    assert_eq!(timesteps[5].unix_time_ms, 1_381_844_925_500);
    assert_eq!(timesteps[5].gps_time_ms, 1_065_880_141_500);
}

#[test]
fn test_populate_correlator_timesteps_data_between_metafits_start_and_end_gaps() {
    // In this scenario, the metafits first timestep starts BEFORE the first data timestep
    // also the metafits last timestep is AFTER the last data timestep. We also have data gaps
    // e.g.
    //
    // metafits: [1_381_844_923_000, 1_381_844_923_500, 1_381_844_924_000, 1_381_844_924_500, 1_381_844_925_000, 1_381_844_925_500]
    // data:                        [1_381_844_923_500,                                       1_381_844_925_000]
    //
    let (metafits_timesteps, _, correlator_timesteps) = create_corr_timestep_testdata(
        1_381_844_923_000,
        1_065_880_139_000,
        3000,
        500,
        vec![1_381_844_923_500, 1_381_844_925_000],
    );

    // First check metafits is what we expect- this is testing the helper code!
    assert_eq!(metafits_timesteps.len(), 6);

    // Check returned correlator timesteps
    assert!(correlator_timesteps.is_some());
    let timesteps = correlator_timesteps.unwrap();

    // Check
    assert_eq!(timesteps.len(), 6);
    assert_eq!(timesteps[0].unix_time_ms, 1_381_844_923_000);
    assert_eq!(timesteps[0].gps_time_ms, 1_065_880_139_000);
    assert_eq!(timesteps[5].unix_time_ms, 1_381_844_925_500);
    assert_eq!(timesteps[5].gps_time_ms, 1_065_880_141_500);
}

#[test]
fn test_populate_correlator_timesteps_data_earlier_and_later_than_metafits_with_offset() {
    // In this scenario, the metafits first timestep starts AFTER the first data timestep
    // also the metafits last timestep is BEFORE the last data timestep
    // AND the metafits timesteps are offset from the data timesteps by 1 second
    // e.g.
    //
    // metafits:                            [1_381_844_923_000, 1_381_844_925_000, 1_381_844_927_000, 1_381_844_929_000, 1_381_844_931_000, 1_381_844_933_000]
    // data:     [1_381_844_920_000, 1_381_844_922_000, 1_381_844_924_000, 1_381_844_926_000, 1_381_844_928_000, 1_381_844_930_000, 1_381_844_932_000, 1_381_844_934_000, 1_381_844_936_000]
    //
    let (metafits_timesteps, _, correlator_timesteps) = create_corr_timestep_testdata(
        1_381_844_923_000,
        1_065_880_139_000,
        12000,
        2000,
        vec![
            1_381_844_920_000,
            1_381_844_922_000,
            1_381_844_924_000,
            1_381_844_926_000,
            1_381_844_928_000,
            1_381_844_930_000,
            1_381_844_932_000,
            1_381_844_934_000,
            1_381_844_936_000,
        ],
    );

    // First check metafits is what we expect- this is testing the helper code!
    assert_eq!(metafits_timesteps.len(), 6);

    // Check returned correlator timesteps
    assert!(correlator_timesteps.is_some());
    let timesteps = correlator_timesteps.unwrap();

    // Check
    assert_eq!(9, timesteps.len());
    assert_eq!(timesteps[0].unix_time_ms, 1_381_844_920_000);
    assert_eq!(timesteps[0].gps_time_ms, 1_065_880_136_000);
    assert_eq!(timesteps[2].unix_time_ms, 1_381_844_924_000);
    assert_eq!(timesteps[2].gps_time_ms, 1_065_880_140_000);
    assert_eq!(timesteps[7].unix_time_ms, 1_381_844_934_000);
    assert_eq!(timesteps[7].gps_time_ms, 1_065_880_150_000);
    assert_eq!(timesteps[8].unix_time_ms, 1_381_844_936_000);
    assert_eq!(timesteps[8].gps_time_ms, 1_065_880_152_000);
}

#[test]
fn test_populate_correlator_timesteps_data_between_metafits_start_and_end_with_offset() {
    // In this scenario, the metafits first timestep starts BEFORE the first data timestep
    // also the metafits last timestep is AFTER the last data timestep
    // AND the metafits timesteps are offset from the data timesteps by 1 second
    // e.g.
    //
    // metafits: [1_381_844_923_000, 1_381_844_925_000, 1_381_844_927_000, 1_381_844_929_000, 1_381_844_931_000, 1_381_844_933_000]
    // data:                                 [1_381_844_926_000, 1_381_844_928_000, 1_381_844_930_000, 1_381_844_932_000]
    //
    let (metafits_timesteps, _, correlator_timesteps) = create_corr_timestep_testdata(
        1_381_844_923_000,
        1_065_880_139_000,
        12000,
        2000,
        vec![
            1_381_844_926_000,
            1_381_844_928_000,
            1_381_844_930_000,
            1_381_844_932_000,
        ],
    );

    // First check metafits is what we expect- this is testing the helper code!
    assert_eq!(metafits_timesteps.len(), 6);

    // Check returned correlator timesteps
    assert!(correlator_timesteps.is_some());
    let timesteps = correlator_timesteps.unwrap();

    // Check
    assert_eq!(5, timesteps.len());
    assert_eq!(timesteps[0].unix_time_ms, 1_381_844_924_000);
    assert_eq!(timesteps[0].gps_time_ms, 1_065_880_140_000);
    assert_eq!(timesteps[4].unix_time_ms, 1_381_844_932_000);
    assert_eq!(timesteps[4].gps_time_ms, 1_065_880_148_000);
}

#[test]
fn test_populate_correlator_timesteps_no_overlap_with_metafits() {
    // In this scenario, the metafits timesteps have no overlap with data timesteps
    // e.g.
    //
    // metafits: [1_381_844_923_000, 1_381_844_923_500, 1_381_844_924_000, 1_381_844_924_500, 1_381_844_925_000, 1_381_844_925_500]
    // data:                                                                                                                                           [1_381_844_926_500]
    //
    let (metafits_timesteps, _, correlator_timesteps) = create_corr_timestep_testdata(
        1_381_844_923_000,
        1_065_880_139_000,
        3000,
        500,
        vec![1_381_844_926_500],
    );

    // First check metafits is what we expect- this is testing the helper code!
    assert_eq!(metafits_timesteps.len(), 6);

    // Check returned correlator timesteps
    assert!(correlator_timesteps.is_some());
    let timesteps = correlator_timesteps.unwrap();

    assert_eq!(8, timesteps.len());
    assert_eq!(timesteps[0].unix_time_ms, 1_381_844_923_000);
    assert_eq!(timesteps[0].gps_time_ms, 1_065_880_139_000);
    assert_eq!(timesteps[5].unix_time_ms, 1_381_844_925_500);
    assert_eq!(timesteps[5].gps_time_ms, 1_065_880_141_500);
    assert_eq!(timesteps[6].unix_time_ms, 1_381_844_926_000);
    assert_eq!(timesteps[6].gps_time_ms, 1_065_880_142_000);
    assert_eq!(timesteps[7].unix_time_ms, 1_381_844_926_500);
    assert_eq!(timesteps[7].gps_time_ms, 1_065_880_142_500);
}

#[test]
fn test_populate_correlator_timesteps_none() {
    // Create a dummy BTree GPUbox map
    let gpubox_time_map = BTreeMap::new();

    let metafits_timesteps: Vec<TimeStep> = Vec::new();

    // Get a vector timesteps
    let scheduled_start_gpstime_ms = 0;
    let scheduled_start_unix_ms = 0;
    let corr_int_time_ms = 500;

    let timesteps = TimeStep::populate_correlator_timesteps(
        &gpubox_time_map,
        &metafits_timesteps,
        scheduled_start_gpstime_ms,
        scheduled_start_unix_ms,
        corr_int_time_ms,
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

#[test]
fn test_get_timstep_indicies() {
    let all_timesteps: Vec<TimeStep> = vec![
        TimeStep::new(1000, 1000),
        TimeStep::new(2000, 2000),
        TimeStep::new(3000, 3000),
        TimeStep::new(4000, 4000),
    ];

    let indices_1 = TimeStep::get_timstep_indicies(&all_timesteps, 1000, 5000);
    assert_eq!(indices_1.len(), 4);
    assert_eq!(indices_1[0], 0);
    assert_eq!(indices_1[1], 1);
    assert_eq!(indices_1[2], 2);
    assert_eq!(indices_1[3], 3);

    let indices_2 = TimeStep::get_timstep_indicies(&all_timesteps, 0000, 6000);
    assert_eq!(indices_2.len(), 4);
    assert_eq!(indices_2[0], 0);
    assert_eq!(indices_2[1], 1);
    assert_eq!(indices_2[2], 2);
    assert_eq!(indices_2[3], 3);

    let indices_3 = TimeStep::get_timstep_indicies(&all_timesteps, 2000, 6000);
    assert_eq!(indices_3.len(), 3);
    assert_eq!(indices_3[0], 1);
    assert_eq!(indices_3[1], 2);
    assert_eq!(indices_3[2], 3);

    let indices_4 = TimeStep::get_timstep_indicies(&all_timesteps, 2000, 4000);
    assert_eq!(indices_4.len(), 2);
    assert_eq!(indices_4[0], 1);
    assert_eq!(indices_4[1], 2);

    let indices_5 = TimeStep::get_timstep_indicies(&all_timesteps, 2000, 3000);
    assert_eq!(indices_5.len(), 1);
    assert_eq!(indices_5[0], 1);
}
