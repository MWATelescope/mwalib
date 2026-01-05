// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Unit tests for metafits context
use super::*;
use float_cmp::*;

#[test]
fn test_metafits_context_new_invalid() {
    let metafits_filename = "invalid.metafits";

    // No gpubox files provided
    let context = MetafitsContext::new(metafits_filename, Some(MWAVersion::CorrMWAXv2));

    assert!(context.is_err());
}

#[test]
fn test_metafits_context_new_vcs_legacy_valid() {
    // Open the test mwa v 1 metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits
    let context = MetafitsContext::new(metafits_filename, Some(MWAVersion::VCSLegacyRecombined))
        .expect("Failed to create MetafitsContext");

    // rf_inputs:                [Tile104Y, ..., Tile055X],
    assert_eq!(context.num_rf_inputs, 256);
    assert_eq!(context.rf_inputs[0].pol, Pol::Y);
    assert_eq!(context.rf_inputs[0].tile_name, "Tile104");
    assert_eq!(context.rf_inputs[255].pol, Pol::X);
    assert_eq!(context.rf_inputs[255].tile_name, "Tile055");

    // Test the properties of the context object match what we expect
    // antennas:                 [Tile011, Tile012, ... Tile167, Tile168],
    // NOTE: since in Legacy VCS the VCS order may look like Tile104Y, Tile103Y, Tile102Y, Tile104X, ...
    // so the order of antennas makes no sense, since 104 needs to be first AND further down the list!, so we leave it in the MWAX order.
    assert_eq!(context.antennas[0].tile_name, "Tile011");
    assert_eq!(context.antennas[127].tile_name, "Tile168");

    assert_eq!(context.metafits_fine_chan_freqs_hz.len(), 3072);
    assert_eq!(
        context.metafits_fine_chan_freqs_hz.len(),
        context.num_metafits_fine_chan_freqs
    );
}

#[test]
fn test_metafits_context_new_corrlegacy_valid() {
    // Open the test mwa v 1 metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits
    let context = MetafitsContext::new(metafits_filename, Some(MWAVersion::CorrLegacy))
        .expect("Failed to create MetafitsContext");

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

    // Calibrator
    assert!(!context.calibrator);
    assert_eq!(context.calibrator_source, "");

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
        context.sun_alt_deg.unwrap(),
        -1.532_227_755_731_48,
        F64Margin::default()
    ));

    // Sun distance:             91.5322277557315 degrees,
    assert!(approx_eq!(
        f64,
        context.sun_distance_deg.unwrap(),
        91.532_227_755_731_5,
        F64Margin::default()
    ));

    // Moon distance:            131.880015235607 degrees,
    assert!(approx_eq!(
        f64,
        context.moon_distance_deg.unwrap(),
        131.880_015_235_607,
        F64Margin::default()
    ));

    // Jupiter distance:         41.401684338269 degrees,
    assert!(approx_eq!(
        f64,
        context.jupiter_distance_deg.unwrap(),
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
    assert_eq!(context.grid_name, String::from("sweet"));

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
    assert_eq!(context.mode, MWAMode::Hw_Lfiles);

    // Geometric delays - this old metafits has none of these keys so it will be No
    assert_eq!(context.geometric_delays_applied, GeometricDelaysApplied::No);
    // Cable delays applied - this old metafits has none of these keys so it will be No
    assert_eq!(
        context.cable_delays_applied,
        CableDelaysApplied::NoCableDelaysApplied
    );

    // Calibration delays & gains applied  - this old metafits has none of these keys so it will be false
    assert!(!context.calibration_delays_and_gains_applied);

    // metafits_filename
    assert_eq!(context.metafits_filename, metafits_filename);

    // Check vispols
    assert_eq!(VisPol::XX.to_string(), "XX");
    assert_eq!(VisPol::XY.to_string(), "XY");
    assert_eq!(VisPol::YX.to_string(), "YX");
    assert_eq!(VisPol::YY.to_string(), "YY");

    // Check correlator mode
    assert_eq!(context.corr_fine_chan_width_hz, 10_000);
    assert_eq!(context.corr_int_time_ms, 2_000);

    // Check metafits fine chan freqs
    assert_eq!(context.metafits_fine_chan_freqs_hz.len(), 3072);
    assert_eq!(
        context.metafits_fine_chan_freqs_hz.len(),
        context.num_metafits_fine_chan_freqs
    );

    // Check that the correct num of digital gains elements (which are based on coarse channels from the metafits) appear in the rf_inputs
    assert_eq!(
        context.rf_inputs[0].digital_gains.len(),
        context.num_metafits_coarse_chans
    );

    // Test oversample flag
    assert!(!context.oversampled);

    // test deripple
    assert!(!context.deripple_applied);
}

#[test]
fn test_metafits_context_new_corrmwaxv2_valid() {
    // Open the test mwa v 1 metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits
    let context = MetafitsContext::new(metafits_filename, Some(MWAVersion::CorrMWAXv2))
        .expect("Failed to create MetafitsContext");

    // Test the properties of the context object match what we expect

    // obsid:                    1101503312,
    assert_eq!(context.obs_id, 1_101_503_312);

    assert_eq!(context.metafits_fine_chan_freqs_hz.len(), 3072);
    assert_eq!(
        context.metafits_fine_chan_freqs_hz.len(),
        context.num_metafits_fine_chan_freqs
    );
}

#[test]
fn test_metafits_context_new_vcsmwax2_valid() {
    // Open the test mwa v 1 metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits
    let context = MetafitsContext::new(metafits_filename, Some(MWAVersion::VCSMWAXv2))
        .expect("Failed to create MetafitsContext");

    // Test the properties of the context object match what we expect

    // obsid:                    1101503312,
    assert_eq!(context.obs_id, 1_101_503_312);

    assert_eq!(context.volt_fine_chan_width_hz, 1_280_000);
    assert_eq!(context.num_volt_fine_chans_per_coarse, 1);

    assert_eq!(context.metafits_fine_chan_freqs_hz.len(), 24);
    assert_eq!(
        context.metafits_fine_chan_freqs_hz.len(),
        context.num_metafits_fine_chan_freqs
    );
}

#[test]
fn test_populate_expected_timesteps() {
    // Note the timesteps returned are fully tested in the timesteps tests, so this is checking the metafits_context calling of that code
    // Open the test metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    let mwa_versions: Vec<MWAVersion> = vec![
        MWAVersion::CorrOldLegacy,
        MWAVersion::CorrLegacy,
        MWAVersion::CorrMWAXv2,
        MWAVersion::VCSLegacyRecombined,
        MWAVersion::VCSMWAXv2,
    ];

    for mwa_version in mwa_versions {
        // Open a context and load in a test metafits
        let result = MetafitsContext::new_internal(metafits_filename);

        assert!(result.is_ok());

        let mut context = result.unwrap();

        let ets_result = context.populate_expected_timesteps(mwa_version);

        assert!(ets_result.is_ok());

        // Confirm basic info
        assert_eq!(
            context.metafits_timesteps.len(),
            match mwa_version {
                MWAVersion::CorrOldLegacy
                | MWAVersion::CorrLegacy
                | MWAVersion::CorrMWAXv2
                | MWAVersion::CorrBeamformerMWAXv2 => {
                    56
                }
                MWAVersion::VCSLegacyRecombined => {
                    112
                }
                MWAVersion::VCSMWAXv2 => {
                    14
                }
                MWAVersion::BeamformerMWAXv2 => {
                    0 // Beamformer metafits timesteps not yet implemented
                }
            }
        );
    }
}

#[test]
fn test_populate_expected_coarse_channels_legacy() {
    // Open the test metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    let mwa_versions: Vec<MWAVersion> = vec![
        MWAVersion::CorrOldLegacy,
        MWAVersion::CorrLegacy,
        MWAVersion::VCSLegacyRecombined,
    ];

    for mwa_version in mwa_versions {
        // Open a context and load in a test metafits
        let result = MetafitsContext::new_internal(metafits_filename);

        assert!(result.is_ok());

        let mut context = result.unwrap();

        let ecc_result = context.populate_expected_coarse_channels(mwa_version);

        assert!(ecc_result.is_ok());

        let chans = context.metafits_coarse_chans;

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
}

#[test]
fn test_populate_expected_coarse_channels_corr_mwaxv2() {
    // Open the test metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    let mwa_versions: Vec<MWAVersion> = vec![MWAVersion::CorrMWAXv2, MWAVersion::VCSMWAXv2];

    for mwa_version in mwa_versions {
        // Open a context and load in a test metafits
        let result = MetafitsContext::new_internal(metafits_filename);

        assert!(result.is_ok());

        let mut context = result.unwrap();

        let ecc_result = context.populate_expected_coarse_channels(mwa_version);

        assert!(ecc_result.is_ok());

        let chans = context.metafits_coarse_chans;

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
}

#[test]
fn test_metafits_context_new_guess_version() {
    // Open the test metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    // Open a context and load in a test metafits
    let result = MetafitsContext::new(metafits_filename, None);
    assert!(result.is_ok());

    let context = result.unwrap();
    assert_eq!(context.mwa_version.unwrap(), MWAVersion::CorrLegacy);
}

#[test]
fn test_generate_expected_volt_filename_legacy_vcs() {
    // Open the test metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    // Open a context and load in a test metafits
    let result = MetafitsContext::new(metafits_filename, Some(MWAVersion::VCSLegacyRecombined));
    assert!(result.is_ok());

    let context = result.unwrap();
    let result = context.generate_expected_volt_filename(3, 1);
    assert!(result.is_ok());
    let new_filename = result.unwrap();
    assert_eq!(new_filename, "1101503312_1101503315_ch110.dat")
}

#[test]
fn test_generate_expected_volt_filename_mwax_vcs() {
    // Open the test metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    // Open a context and load in a test metafits
    let result = MetafitsContext::new(metafits_filename, Some(MWAVersion::VCSMWAXv2));
    assert!(result.is_ok());

    let context = result.unwrap();
    let result = context.generate_expected_volt_filename(2, 1);
    assert!(result.is_ok());
    let new_filename = result.unwrap();
    assert_eq!(new_filename, "1101503312_1101503328_110.sub")
}

#[test]
fn test_generate_expected_volt_filename_invalid_timestep() {
    // Open the test metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    // Open a context and load in a test metafits
    let result = MetafitsContext::new(metafits_filename, Some(MWAVersion::VCSLegacyRecombined));
    assert!(result.is_ok());

    let context = result.unwrap();
    let result = context.generate_expected_volt_filename(99999, 0);
    assert!(result.is_err());
}

#[test]
fn test_generate_expected_volt_filename_invalid_coarse_chan() {
    // Open the test metafits file
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    // Open a context and load in a test metafits
    let result = MetafitsContext::new(metafits_filename, Some(MWAVersion::VCSLegacyRecombined));
    assert!(result.is_ok());

    let context = result.unwrap();
    let result = context.generate_expected_volt_filename(0, 99);
    assert!(result.is_err());
}

#[test]
fn test_deripple_on_in_metafits() {
    // Open the test metafits file
    let metafits_filename = "test_files/metafits_tests/1370752512_metafits_deripple_os.fits";

    // Open a context and load in a test metafits
    let result = MetafitsContext::new(metafits_filename, None);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    let context = result.unwrap();

    assert!(context.deripple_applied);
    assert_eq!(context.deripple_param, "deripplev1");
}

#[test]
fn test_oversampling_on_in_metafits() {
    // Open the test metafits file
    let metafits_filename = "test_files/metafits_tests/1370752512_metafits_deripple_os.fits";

    // Open a context and load in a test metafits
    let result = MetafitsContext::new(metafits_filename, None);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    let context = result.unwrap();

    assert!(context.oversampled);
}

#[test]
fn test_calibration_hdu_in_metafits() {
    // Open the test metafits file
    let metafits_filename = "test_files/metafits_cal_sol/1111842752_metafits.fits";

    // Open a context and load in a test metafits
    let result = MetafitsContext::new(metafits_filename, None);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    let context = result.unwrap();

    assert_eq!(context.best_cal_fit_id, Some(1720774022));
    assert_eq!(context.best_cal_obs_id, Some(1111842752));
    assert_eq!(context.best_cal_code_ver, Some(String::from("0.17.22")));
    assert_eq!(
        context.best_cal_fit_timestamp,
        Some(String::from("2024-07-12T08:47:02.308203+00:00"))
    );
    assert_eq!(context.best_cal_creator, Some(String::from("calvin")));
    assert_eq!(context.best_cal_fit_iters, Some(3));
    assert_eq!(context.best_cal_fit_iter_limit, Some(20));

    assert_eq!(context.rf_inputs[2].calib_delay, Some(0.4399995));
    assert_eq!(
        context.rf_inputs[2].calib_gains.clone().unwrap()[0],
        0.70867455
    );
    assert_eq!(
        context.rf_inputs[2].calib_gains.clone().unwrap()[23],
        1.1947584
    );

    assert_eq!(context.num_rf_inputs, context.rf_inputs.len());
    assert_eq!(context.num_ants, context.antennas.len());
    assert_eq!(context.num_ants * 2, context.num_rf_inputs);
}

#[test]
fn test_calibration_hdu_not_in_metafits() {
    // Open the test metafits file
    let metafits_filename = "test_files/metafits_tests/1370752512_metafits_deripple_os.fits";

    // Open a context and load in a test metafits
    let result = MetafitsContext::new(metafits_filename, None);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    let context = result.unwrap();

    assert_eq!(context.best_cal_fit_id, None);
    assert_eq!(context.best_cal_obs_id, None);
    assert_eq!(context.best_cal_code_ver, None);
    assert_eq!(context.best_cal_fit_timestamp, None);
    assert_eq!(context.best_cal_creator, None);
    assert_eq!(context.best_cal_fit_iters, None);
    assert_eq!(context.best_cal_fit_iter_limit, None);
}

#[test]
fn test_signal_chain_corrections_hdu_not_in_metafits() {
    // Open the test metafits file
    let metafits_filename = "test_files/metafits_tests/1370752512_metafits_deripple_os.fits";

    // Open a context and load in a test metafits
    let result = MetafitsContext::new(metafits_filename, None);
    assert!(result.is_ok(), "{}", result.unwrap_err());

    let context = result.unwrap();

    assert_eq!(context.signal_chain_corrections, None);
}

#[test]
fn test_signal_chain_corrections_hdu_in_metafits() {
    // Open the test metafits file
    let metafits_filename = "test_files/metafits_signal_chain_corr/1096952256_metafits.fits";

    // Open a context and load in a test metafits
    let result = MetafitsContext::new(metafits_filename, None);

    assert!(result.is_ok(), "{}", result.unwrap_err());

    let context = result.unwrap();

    let sig_chain_corr = context.signal_chain_corrections.unwrap();

    assert_eq!(sig_chain_corr.len(), 8);
    assert_eq!(sig_chain_corr.len(), context.num_signal_chain_corrections);

    // First row is:
    // RRI                0  0.16073910960211837 .. 0.7598147243238643
    assert_eq!(sig_chain_corr[0].receiver_type, ReceiverType::RRI);
    assert!(!sig_chain_corr[0].whitening_filter);
    assert_eq!(sig_chain_corr[0].corrections[0], 0.16073910960211837);
    assert_eq!(sig_chain_corr[0].corrections[255], 0.7598147243238643);

    // 4th row is:
    // NI                 1   0.0 .. 0.0
    assert_eq!(sig_chain_corr[3].receiver_type, ReceiverType::NI);
    assert!(sig_chain_corr[3].whitening_filter);
    assert_eq!(sig_chain_corr[3].corrections[0], 0.0);
    assert_eq!(sig_chain_corr[3].corrections[255], 0.0);
}

#[test]
fn test_metafits_context_new_corrmwaxv2_224_tiles_valid() {
    // Open the test mwa v 1 metafits file
    let metafits_filename = "test_files/1428041840_224T/1428041840_metafits.fits";

    //
    // Read the observation using mwalib
    //
    // Open a context and load in a test metafits
    let context = MetafitsContext::new(metafits_filename, Some(MWAVersion::CorrMWAXv2))
        .expect("Failed to create MetafitsContext");

    // Test the properties of the context object match what we expect

    // obsid:                    1428041840,
    assert_eq!(context.obs_id, 1_428_041_840);

    assert_eq!(context.num_receivers, 28);

    assert_eq!(context.num_ants, 224);
}
