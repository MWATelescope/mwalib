// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Unit tests for metafits context
*/
#[cfg(test)]
use super::*;
use float_cmp::*;

#[test]
fn test_metafits_context_new_invalid() {
    let metafits_filename = "invalid.metafits";

    // No gpubox files provided
    let context = MetafitsContext::new(&metafits_filename);

    assert!(context.is_err());
}

#[test]
fn test_metafits_context_new_valid() {
    // Open the test mwa v 1 metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits
    let context =
        MetafitsContext::new(&metafits_filename).expect("Failed to create MetafitsContext");

    // Test the properties of the context object match what we expect

    // obsid:                    1101503312,
    assert_eq!(context.obs_id, 1_101_503_312);

    // Creator:                  Randall,
    assert_eq!(context.creator, "Randall");

    // Project ID:               G0009,
    assert_eq!(context.project_id, "G0009");

    // Observation Name:         FDS_DEC-26.7_121,
    assert_eq!(context.obs_name, "FDS_DEC-26.7_121");

    // Receivers:                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
    assert_eq!(context.receivers.len(), 16);
    assert_eq!(context.receivers[0], 1);
    assert_eq!(context.receivers[15], 16);

    // Delays:                   [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    assert_eq!(context.delays.len(), 16);
    assert_eq!(context.delays[0], 0);
    assert_eq!(context.delays[15], 0);

    // Global attenuation:       1 dB,
    assert_eq!(context.global_analogue_attenuation_db as i16, 1);

    // Scheduled start (utc)     2014-12-01 21:08:16 +00:00,
    assert_eq!(
        context.sched_start_utc,
        DateTime::parse_from_rfc3339("2014-12-01T21:08:16+00:00").unwrap()
    );

    // Scheduled start (MJD)     56992.88074074074,
    assert!(approx_eq!(
        f64,
        context.sched_start_mjd,
        56_992.880_740_740_74,
        F64Margin::default()
    ));

    // Scheduled duration        112 s,
    assert_eq!(context.sched_duration_ms, 112_000);

    // Quack time:               2 s,
    assert_eq!(context.quack_time_duration_ms, 2000);

    // Good UNIX start time:     1417468098,
    assert_eq!(context.good_time_unix_ms, 1_417_468_098_000);

    // R.A. (tile_pointing):     144.2107504850443 degrees,
    assert!(approx_eq!(
        f64,
        context.ra_tile_pointing_degrees,
        144.210_750_485_044_3,
        F64Margin::default()
    ));

    // Dec. (tile_pointing):     -26.63403125476213 degrees,
    assert!(approx_eq!(
        f64,
        context.dec_tile_pointing_degrees,
        -26.634_031_254_762_13,
        F64Margin::default()
    ));

    // R.A. (phase center):      None degrees,
    assert!(context.ra_phase_center_degrees.is_none());

    // Dec. (phase center):      None degrees,
    assert!(context.dec_phase_center_degrees.is_none());

    // Azimuth:                  0 degrees,
    assert!(approx_eq!(f64, context.az_deg, 0., F64Margin::default()));

    // Altitude:                 90 degrees,
    assert!(approx_eq!(f64, context.alt_deg, 90., F64Margin::default()));

    // Zenith angle (derived from altitude): 0 degrees,
    assert!(approx_eq!(f64, context.za_deg, 0., F64Margin::default()));

    // Sun altitude:             -1.53222775573148 degrees,
    assert!(approx_eq!(
        f64,
        context.sun_alt_deg,
        -1.532_227_755_731_48,
        F64Margin::default()
    ));

    // Sun distance:             91.5322277557315 degrees,
    assert!(approx_eq!(
        f64,
        context.sun_distance_deg,
        91.532_227_755_731_5,
        F64Margin::default()
    ));

    // Moon distance:            131.880015235607 degrees,
    assert!(approx_eq!(
        f64,
        context.moon_distance_deg,
        131.880_015_235_607,
        F64Margin::default()
    ));

    // Jupiter distance:         41.401684338269 degrees,
    assert!(approx_eq!(
        f64,
        context.jupiter_distance_deg,
        41.401_684_338_269,
        F64Margin::default()
    ));

    // LST:                      144.381251875516 degrees,
    assert!(approx_eq!(
        f64,
        context.lst_deg,
        144.381_251_875_516,
        F64Margin::default()
    ));

    // Hour angle:               -00:00:00.00 degrees,
    // Grid name:                sweet,
    assert_eq!(context.grid_name, "sweet");

    // Grid number:              0,
    assert_eq!(context.grid_number, 0);

    // num antennas:             128,
    assert_eq!(context.num_ants, 128);

    // antennas:                 [Tile011, Tile012, ... Tile167, Tile168],
    assert_eq!(context.antennas[0].tile_name, "Tile011");
    assert_eq!(context.antennas[127].tile_name, "Tile168");

    // rf_inputs:                [Tile011X, Tile011Y, ... Tile168X, Tile168Y],
    assert_eq!(context.num_rf_inputs, 256);
    assert_eq!(context.rf_inputs[0].pol, Pol::X);
    assert_eq!(context.rf_inputs[0].tile_name, "Tile011");
    assert_eq!(context.rf_inputs[255].pol, Pol::Y);
    assert_eq!(context.rf_inputs[255].tile_name, "Tile168");

    // num baselines:            8256,
    assert_eq!(context.num_baselines, 8256);

    // num antenna pols:         2,
    assert_eq!(context.num_ant_pols, 2);

    // Mode:                     HW_LFILES,
    assert_eq!(context.mode, "HW_LFILES");

    // metafits_filename
    assert_eq!(context.metafits_filename, metafits_filename);
}

#[test]
fn test_get_expected_coarse_channels_old_legacy() {
    // Open the test metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    // Open a context and load in a test metafits
    let result = MetafitsContext::new(&metafits_filename);

    assert!(result.is_ok());

    let context = result.unwrap();

    let ecc_result = context.get_expected_coarse_channels(CorrelatorVersion::OldLegacy);

    assert!(ecc_result.is_ok());

    let chans = ecc_result.unwrap();

    assert_eq!(chans.len(), 24);

    assert_eq!(chans[0].corr_chan_number, 0);
    assert_eq!(chans[0].rec_chan_number, 109);

    assert_eq!(chans[19].corr_chan_number, 19);
    assert_eq!(chans[19].rec_chan_number, 128);

    assert_eq!(chans[20].corr_chan_number, 23);
    assert_eq!(chans[20].rec_chan_number, 129);

    assert_eq!(chans[21].corr_chan_number, 22);
    assert_eq!(chans[21].rec_chan_number, 130);

    assert_eq!(chans[22].corr_chan_number, 21);
    assert_eq!(chans[22].rec_chan_number, 131);

    assert_eq!(chans[23].corr_chan_number, 20);
    assert_eq!(chans[23].rec_chan_number, 132);
}

#[test]
fn test_get_expected_coarse_channels_legacy() {
    // Open the test metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    // Open a context and load in a test metafits
    let result = MetafitsContext::new(&metafits_filename);

    assert!(result.is_ok());

    let context = result.unwrap();

    let ecc_result = context.get_expected_coarse_channels(CorrelatorVersion::Legacy);

    assert!(ecc_result.is_ok());

    let chans = ecc_result.unwrap();

    assert_eq!(chans.len(), 24);

    assert_eq!(chans[0].corr_chan_number, 0);
    assert_eq!(chans[0].rec_chan_number, 109);

    assert_eq!(chans[19].corr_chan_number, 19);
    assert_eq!(chans[19].rec_chan_number, 128);

    assert_eq!(chans[20].corr_chan_number, 23);
    assert_eq!(chans[20].rec_chan_number, 129);

    assert_eq!(chans[21].corr_chan_number, 22);
    assert_eq!(chans[21].rec_chan_number, 130);

    assert_eq!(chans[22].corr_chan_number, 21);
    assert_eq!(chans[22].rec_chan_number, 131);

    assert_eq!(chans[23].corr_chan_number, 20);
    assert_eq!(chans[23].rec_chan_number, 132);
}

#[test]
fn test_get_expected_coarse_channels_v2() {
    // Open the test metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    // Open a context and load in a test metafits
    let result = MetafitsContext::new(&metafits_filename);

    assert!(result.is_ok());

    let context = result.unwrap();

    let ecc_result = context.get_expected_coarse_channels(CorrelatorVersion::V2);

    assert!(ecc_result.is_ok());

    let chans = ecc_result.unwrap();

    assert_eq!(chans.len(), 24);

    assert_eq!(chans[0].corr_chan_number, 0);
    assert_eq!(chans[0].rec_chan_number, 109);

    assert_eq!(chans[19].corr_chan_number, 19);
    assert_eq!(chans[19].rec_chan_number, 128);

    assert_eq!(chans[20].corr_chan_number, 20);
    assert_eq!(chans[20].rec_chan_number, 129);

    assert_eq!(chans[21].corr_chan_number, 21);
    assert_eq!(chans[21].rec_chan_number, 130);

    assert_eq!(chans[22].corr_chan_number, 22);
    assert_eq!(chans[22].rec_chan_number, 131);

    assert_eq!(chans[23].corr_chan_number, 23);
    assert_eq!(chans[23].rec_chan_number, 132);
}

#[test]
fn test_correlator_version_display() {
    let cv = CorrelatorVersion::V2;

    assert_eq!(format!("{}", cv), "v2 MWAX");
}
