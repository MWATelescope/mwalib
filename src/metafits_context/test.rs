use super::*;
use float_cmp::*;

#[test]
fn test_obs_context_new_invalid_metafits() {
    let metafits_filename = "invalid.metafits";

    // No gpubox files provided
    let context = MetafitsContext::new(&metafits_filename);

    assert!(context.is_err());
}

#[test]
#[allow(clippy::cognitive_complexity)]
fn test_obs_context_legacy_v1() {
    // Open the test mwa v 1 metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits
    let context =
        MetafitsContext::new(&metafits_filename).expect("Failed to create MetafitsContext");

    // Test the properties of the context object match what we expect

    // MWA latitude:             -26.703319405555554 degrees,
    assert!(approx_eq!(
        f64,
        context.mwa_latitude_radians.to_degrees(),
        -26.703_319_405_555_554,
        F64Margin::default()
    ));
    // MWA longitude:            116.67081523611111 degrees
    assert!(approx_eq!(
        f64,
        context.mwa_longitude_radians.to_degrees(),
        116.670_815_236_111_11,
        F64Margin::default()
    ));
    // MWA altitude:             377.827 m,
    assert!(approx_eq!(
        f64,
        context.mwa_altitude_metres,
        377.827,
        F64Margin::default()
    ));

    // obsid:                    1101503312,
    assert_eq!(context.obsid, 1_101_503_312);

    // Creator:                  Randall,
    assert_eq!(context.creator, "Randall");

    // Project ID:               G0009,
    assert_eq!(context.project_id, "G0009");

    // Observation Name:         FDS_DEC-26.7_121,
    assert_eq!(context.observation_name, "FDS_DEC-26.7_121");

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
        context.scheduled_start_utc,
        DateTime::parse_from_rfc3339("2014-12-01T21:08:16+00:00").unwrap()
    );

    // Scheduled start (MJD)     56992.88074074074,
    assert!(approx_eq!(
        f64,
        context.scheduled_start_mjd,
        56_992.880_740_740_74,
        F64Margin::default()
    ));

    // Scheduled duration        112 s,
    assert_eq!(context.scheduled_duration_milliseconds, 112_000);

    // Quack time:               2 s,
    assert_eq!(context.quack_time_duration_milliseconds, 2000);

    // Good UNIX start time:     1417468098,
    assert_eq!(context.good_time_unix_milliseconds, 1_417_468_098_000);

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
    assert!(approx_eq!(
        f64,
        context.azimuth_degrees,
        0.,
        F64Margin::default()
    ));

    // Altitude:                 90 degrees,
    assert!(approx_eq!(
        f64,
        context.altitude_degrees,
        90.,
        F64Margin::default()
    ));

    // Sun altitude:             -1.53222775573148 degrees,
    assert!(approx_eq!(
        f64,
        context.sun_altitude_degrees,
        -1.532_227_755_731_48,
        F64Margin::default()
    ));

    // Sun distance:             91.5322277557315 degrees,
    assert!(approx_eq!(
        f64,
        context.sun_distance_degrees,
        91.532_227_755_731_5,
        F64Margin::default()
    ));

    // Moon distance:            131.880015235607 degrees,
    assert!(approx_eq!(
        f64,
        context.moon_distance_degrees,
        131.880_015_235_607,
        F64Margin::default()
    ));

    // Jupiter distance:         41.401684338269 degrees,
    assert!(approx_eq!(
        f64,
        context.jupiter_distance_degrees,
        41.401_684_338_269,
        F64Margin::default()
    ));

    // LST:                      144.381251875516 degrees,
    assert!(approx_eq!(
        f64,
        context.lst_degrees,
        144.381_251_875_516,
        F64Margin::default()
    ));

    // Hour angle:               -00:00:00.00 degrees,
    // Grid name:                sweet,
    assert_eq!(context.grid_name, "sweet");

    // Grid number:              0,
    assert_eq!(context.grid_number, 0);

    // num antennas:             128,
    assert_eq!(context.num_antennas, 128);

    // antennas:                 [Tile011, Tile012, ... Tile167, Tile168],
    assert_eq!(context.antennas[0].tile_name, "Tile011");
    assert_eq!(context.antennas[127].tile_name, "Tile168");

    // rf_inputs:                [Tile011X, Tile011Y, ... Tile168X, Tile168Y],
    assert_eq!(context.num_rf_inputs, 256);
    assert_eq!(context.rf_inputs[0].pol, Pol::X);
    assert_eq!(context.rf_inputs[0].tile_name, "Tile011");
    assert_eq!(context.rf_inputs[255].pol, Pol::Y);
    assert_eq!(context.rf_inputs[255].tile_name, "Tile168");

    // num antenna pols:         2,
    assert_eq!(context.num_antenna_pols, 2);

    // Mode:                     HW_LFILES,
    assert_eq!(context.mode, "HW_LFILES");

    // metafits_filename
    assert_eq!(context.metafits_filename, metafits_filename);
}
