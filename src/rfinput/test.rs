// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Unit tests for rfinput metadata
use super::*;
use crate::{
    fits_open, fits_open_hdu, test::with_new_temp_fits_file, MWAVersion, MetafitsContext,
    MAX_RECEIVER_CHANNELS,
};
use fitsio::tables::{ColumnDataType, ColumnDescription};
use float_cmp::*;

#[test]
fn test_get_vcs_order() {
    assert_eq!(0, get_vcs_order(0));
    assert_eq!(4, get_vcs_order(1));
    assert_eq!(32, get_vcs_order(8));
    assert_eq!(127, get_vcs_order(127));
    assert_eq!(128, get_vcs_order(128));
    assert_eq!(194, get_vcs_order(224));
    assert_eq!(251, get_vcs_order(254));
    assert_eq!(255, get_vcs_order(255));
    assert_eq!(256, get_vcs_order(256));
    assert_eq!(271, get_vcs_order(271));
}

#[test]
fn test_get_mwax_order() {
    assert_eq!(0, get_mwax_order(0, Pol::X));
    assert_eq!(1, get_mwax_order(0, Pol::Y));
    assert_eq!(32, get_mwax_order(16, Pol::X));
    assert_eq!(33, get_mwax_order(16, Pol::Y));
    assert_eq!(120, get_mwax_order(60, Pol::X));
    assert_eq!(121, get_mwax_order(60, Pol::Y));
    assert_eq!(254, get_mwax_order(127, Pol::X));
    assert_eq!(255, get_mwax_order(127, Pol::Y));
    assert_eq!(256, get_mwax_order(128, Pol::X));
    assert_eq!(271, get_mwax_order(135, Pol::Y));
}

#[test]
fn test_get_electrical_length() {
    assert!(float_cmp::approx_eq!(
        f64,
        123.45,
        get_electrical_length(String::from("EL_123.45"), 1.204),
        float_cmp::F64Margin::default()
    ));
    assert!(float_cmp::approx_eq!(
        f64,
        1.204 * 16.,
        get_electrical_length(String::from("16"), 1.204),
        float_cmp::F64Margin::default()
    ));
}

#[test]
fn test_read_metafits_tiledata_values_from_row_0() {
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let mut metafits_fptr = fits_open!(&metafits_filename).unwrap();

    let metafits_tile_table_hdu = fits_open_hdu!(&mut metafits_fptr, 1).unwrap();

    // Get values from row 1
    let row: RfInputMetafitsTableRow =
        Rfinput::read_metafits_tiledata_values(&mut metafits_fptr, &metafits_tile_table_hdu, 0, 24)
            .unwrap();
    assert_eq!(row.input, 0);
    assert_eq!(row.antenna, 75);
    assert_eq!(row.tile_id, 104);
    assert_eq!(row.tile_name, "Tile104");
    assert_eq!(row.pol, Pol::Y);
    assert_eq!(row.length_string, "EL_-756.49");
    assert!(float_cmp::approx_eq!(
        f64,
        row.north_m,
        -101.529_998_779_296_88,
        float_cmp::F64Margin::default()
    ));
    assert!(float_cmp::approx_eq!(
        f64,
        row.east_m,
        -585.674_987_792_968_8,
        float_cmp::F64Margin::default()
    ));
    assert!(float_cmp::approx_eq!(
        f64,
        row.height_m,
        375.212_005_615_234_4,
        float_cmp::F64Margin::default()
    ));
    assert_eq!(row.flag, 1);
    assert_eq!(row.rx, 10);
    assert_eq!(row.slot, 4);
    assert_eq!(row.rx_type, "");
}

#[test]
fn test_read_metafits_tiledata_values_from_invalid_metafits() {
    let metafits_filename = "read_metafits_values_from_invalid_metafits.metafits";

    with_new_temp_fits_file(metafits_filename, |metafits_fptr| {
        // Create a tiledata hdu
        let first_description = ColumnDescription::new("A")
            .with_type(ColumnDataType::Int)
            .create()
            .unwrap();
        let second_description = ColumnDescription::new("B")
            .with_type(ColumnDataType::Long)
            .create()
            .unwrap();
        let descriptions = [first_description, second_description];

        metafits_fptr
            .create_table("TILEDATA".to_string(), &descriptions)
            .unwrap();

        let metafits_tile_table_hdu = fits_open_hdu!(metafits_fptr, 1).unwrap();

        // Get values from row 1
        let metafits_result =
            Rfinput::read_metafits_tiledata_values(metafits_fptr, &metafits_tile_table_hdu, 0, 24);

        assert!(metafits_result.is_err());
    });
}

#[test]
fn test_read_metafits_calibdata_values_from_row_0() {
    let metafits_filename = "test_files/metafits_cal_sol/1111842752_metafits.fits";
    let mut metafits_fptr = fits_open!(&metafits_filename).unwrap();

    let metafits_calib_table_hdu = fits_open_hdu!(&mut metafits_fptr, 2).unwrap();

    // Get values from row 1
    let row: Option<RfInputMetafitsCalibDataTableRow> = Rfinput::read_metafits_calibdata_values(
        &mut metafits_fptr,
        &Some(metafits_calib_table_hdu),
        0,
        24,
    )
    .unwrap();
    assert!(row.is_some());

    let row = row.unwrap();

    assert_eq!(row.antenna, 75);
    assert_eq!(row.tile, 104);
    assert_eq!(row.tilename, "Tile104");
    assert_eq!(row.pol, Pol::Y);
    assert!(float_cmp::approx_eq!(
        f32,
        row.calib_delay,
        -135.49985,
        float_cmp::F32Margin::default()
    ));
    assert!(row.calib_gains.len() == 24);
    assert!(row.calib_gains[0] == 0.9759591);
    assert!(row.calib_gains[23] == 1.6084857);
}

#[test]
fn test_read_metafits_calibdata_values_from_invalid_metafits() {
    let metafits_filename = "read_metafits_values_from_invalid_metafits.metafits";

    with_new_temp_fits_file(metafits_filename, |metafits_fptr| {
        // Create a tiledata hdu
        let first_description = ColumnDescription::new("A")
            .with_type(ColumnDataType::Int)
            .create()
            .unwrap();
        let second_description = ColumnDescription::new("B")
            .with_type(ColumnDataType::Long)
            .create()
            .unwrap();
        let descriptions = [first_description, second_description];

        metafits_fptr
            .create_table("TILEDATA".to_string(), &descriptions)
            .unwrap();

        metafits_fptr
            .create_table("CALIBDATA".to_string(), &descriptions)
            .unwrap();

        let metafits_calib_table_hdu = fits_open_hdu!(metafits_fptr, 2).unwrap();

        // Get values from row 1
        let metafits_result = Rfinput::read_metafits_calibdata_values(
            metafits_fptr,
            &Some(metafits_calib_table_hdu),
            0,
            24,
        );

        assert!(metafits_result.is_err());
    });
}

#[test]
fn test_populate_rf_inputs() {
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let mut metafits_fptr = fits_open!(&metafits_filename).unwrap();
    let metafits_tile_table_hdu = fits_open_hdu!(&mut metafits_fptr, 1).unwrap();
    let result = Rfinput::populate_rf_inputs(
        256,
        &mut metafits_fptr,
        &metafits_tile_table_hdu,
        1.204,
        24,
        &None,
    );

    assert!(result.is_ok());

    let rfinput = result.unwrap();

    assert_eq!(rfinput[0].input, 0);
    assert_eq!(rfinput[0].ant, 75);
    assert_eq!(rfinput[0].tile_id, 104);
    assert_eq!(rfinput[0].tile_name, "Tile104");
    assert_eq!(rfinput[0].pol, Pol::Y);
    assert!(approx_eq!(
        f64,
        rfinput[0].electrical_length_m,
        -756.49,
        F64Margin::default()
    ));

    assert!(float_cmp::approx_eq!(
        f64,
        rfinput[0].north_m,
        -101.529_998_779_296_88,
        float_cmp::F64Margin::default()
    ));
    assert!(float_cmp::approx_eq!(
        f64,
        rfinput[0].east_m,
        -585.674_987_792_968_8,
        float_cmp::F64Margin::default()
    ));
    assert!(float_cmp::approx_eq!(
        f64,
        rfinput[0].height_m,
        375.212_005_615_234_4,
        float_cmp::F64Margin::default()
    ));
    assert!(rfinput[0].flagged);

    assert!(float_cmp::approx_eq!(
        f64,
        rfinput[0].digital_gains[0],
        74. / 64.,
        float_cmp::F64Margin::default()
    ));

    assert!(float_cmp::approx_eq!(
        f64,
        rfinput[0].digital_gains[12],
        66. / 64.,
        float_cmp::F64Margin::default()
    ));

    assert!(float_cmp::approx_eq!(
        f64,
        rfinput[0].digital_gains[23],
        68. / 64.,
        float_cmp::F64Margin::default()
    ));

    assert_eq!(
        rfinput[0].dipole_delays,
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(rfinput[0].rec_number, 10);
    assert_eq!(rfinput[0].rec_slot_number, 4);
    assert_eq!(rfinput[0].rec_type, ReceiverType::Unknown);
}

#[test]
fn test_populate_rf_inputs_newer_metafits() {
    let metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
    let mut metafits_fptr = fits_open!(&metafits_filename).unwrap();
    let metafits_tile_table_hdu = fits_open_hdu!(&mut metafits_fptr, 1).unwrap();
    let result = Rfinput::populate_rf_inputs(
        256,
        &mut metafits_fptr,
        &metafits_tile_table_hdu,
        1.204,
        24,
        &None,
    );

    assert!(result.is_ok());

    let rfinput = result.unwrap();
    assert_eq!(rfinput.len(), 256);

    assert_eq!(rfinput[0].rec_type, ReceiverType::RRI);
}

#[test]
fn test_string_to_receiver_type() {
    assert!(String::from("RRI").parse::<ReceiverType>().unwrap() == ReceiverType::RRI);
    assert!(String::from("NI").parse::<ReceiverType>().unwrap() == ReceiverType::NI);
    assert!(String::from("PSEUDO").parse::<ReceiverType>().unwrap() == ReceiverType::Pseudo);
    assert!(String::from("SHAO").parse::<ReceiverType>().unwrap() == ReceiverType::SHAO);
    assert!(String::from("EDA2").parse::<ReceiverType>().unwrap() == ReceiverType::EDA2);

    // what happens for lowercase?
    assert!(String::from("rri").parse::<ReceiverType>().unwrap() == ReceiverType::RRI);

    assert!(
        String::from("something else")
            .parse::<ReceiverType>()
            .unwrap()
            == ReceiverType::Unknown
    );
    assert!(String::from("").parse::<ReceiverType>().unwrap() == ReceiverType::Unknown);
}

#[test]
fn test_flavor_and_whitening_filter() {
    let metafits_filename = "test_files/1384808344/1384808344_metafits.fits";
    let mut metafits_fptr = fits_open!(&metafits_filename).unwrap();
    let metafits_tile_table_hdu = fits_open_hdu!(&mut metafits_fptr, 1).unwrap();
    let result = Rfinput::populate_rf_inputs(
        290,
        &mut metafits_fptr,
        &metafits_tile_table_hdu,
        1.204,
        24,
        &None,
    );

    assert!(result.is_ok());
    let mut rfinput = result.unwrap();

    // need to remember to apply correct sort
    rfinput.sort_by_key(|k| k.subfile_order);

    assert_eq!(rfinput.len(), 290);

    assert_eq!(rfinput[0].rec_type, ReceiverType::RRI);
    assert_eq!(rfinput[0].tile_name, "Tile011");
    assert_eq!(rfinput[0].flavour, "RG6_90");
    assert!(!rfinput[0].has_whitening_filter);

    assert_eq!(rfinput[4].rec_type, ReceiverType::RRI);
    assert_eq!(rfinput[4].tile_name, "Tile013");
    assert_eq!(rfinput[4].flavour, "RG6_150");
    assert!(rfinput[4].has_whitening_filter);

    assert_eq!(rfinput[289].rec_type, ReceiverType::NI);
    assert_eq!(rfinput[289].tile_name, "LBG8");
    assert_eq!(rfinput[289].flavour, "RFOF-NI");
    assert!(!rfinput[289].has_whitening_filter);
}

#[test]
fn test_populate_rf_inputs_calib_metafits() {
    let metafits_filename = "test_files/metafits_cal_sol/1111842752_metafits.fits";
    let mut metafits_fptr = fits_open!(&metafits_filename).unwrap();
    let metafits_tile_table_hdu = fits_open_hdu!(&mut metafits_fptr, 1).unwrap();
    let result = Rfinput::populate_rf_inputs(
        256,
        &mut metafits_fptr,
        &metafits_tile_table_hdu,
        1.204,
        24,
        &None,
    );

    assert!(result.is_ok());

    let rfinput = result.unwrap();
    assert_eq!(rfinput.len(), 256);

    assert_eq!(rfinput[0].calib_delay, Some(-135.49985));
}

#[test]
fn test_populate_rf_inputs_calib_metafits_context() {
    let metafits_filename = "test_files/metafits_cal_sol/1111842752_metafits.fits";

    let context = MetafitsContext::new(metafits_filename, Some(MWAVersion::CorrLegacy));

    let context = context.unwrap();

    assert_eq!(context.rf_inputs.len(), 256);
    assert_eq!(context.rf_inputs[151].tile_name, "Tile104");
    assert_eq!(context.rf_inputs[151].calib_delay, Some(-135.49985));
    assert_eq!(context.antennas.len(), 128);
}

#[test]
fn test_populate_rf_inputs_sig_chain_in_metafits() {
    let metafits_filename = "test_files/metafits_signal_chain_corr/1096952256_metafits.fits";
    let mut metafits_fptr = fits_open!(&metafits_filename).unwrap();
    let metafits_tile_table_hdu = fits_open_hdu!(&mut metafits_fptr, 1).unwrap();

    let all_ones: Vec<f64> = vec![1.0; MAX_RECEIVER_CHANNELS];
    let all_twos: Vec<f64> = vec![2.0; MAX_RECEIVER_CHANNELS];

    let sig_chain_corrs: Vec<SignalChainCorrection> = vec![
        SignalChainCorrection {
            receiver_type: ReceiverType::RRI,
            whitening_filter: false,
            num_corrections: all_ones.len(),
            corrections: all_ones,
        },
        SignalChainCorrection {
            receiver_type: ReceiverType::RRI,
            whitening_filter: true,
            num_corrections: all_twos.len(),
            corrections: all_twos,
        },
    ];

    let result = Rfinput::populate_rf_inputs(
        256,
        &mut metafits_fptr,
        &metafits_tile_table_hdu,
        1.204,
        24,
        &Some(sig_chain_corrs),
    );

    assert!(result.is_ok());

    let mut rfinput = result.unwrap();
    assert_eq!(rfinput.len(), 256);
    rfinput.sort_by_key(|k| k.subfile_order);

    // Check the signal chain correct index is correct!
    assert_eq!(rfinput[0].tile_id, 11);
    assert_eq!(rfinput[0].signal_chain_corrections_index, Some(0)); // RRI, no whitening filter

    assert_eq!(rfinput[5].tile_id, 13);
    assert_eq!(rfinput[5].signal_chain_corrections_index, Some(1)); // RRI, whitening filter
}

#[test]
fn test_populate_rf_inputs_sig_chain_not_in_metafits() {
    let metafits_filename = "test_files/metafits_cal_sol/1111842752_metafits.fits";
    let mut metafits_fptr = fits_open!(&metafits_filename).unwrap();
    let metafits_tile_table_hdu = fits_open_hdu!(&mut metafits_fptr, 1).unwrap();

    let result = Rfinput::populate_rf_inputs(
        256,
        &mut metafits_fptr,
        &metafits_tile_table_hdu,
        1.204,
        24,
        &None,
    );

    assert!(result.is_ok());

    let mut rfinput = result.unwrap();
    assert_eq!(rfinput.len(), 256);
    rfinput.sort_by_key(|k| k.subfile_order);

    // Check the signal chain correct index is correct - no info in this metafits so it will be None!
    assert_eq!(rfinput[0].signal_chain_corrections_index, None); // RRI, no whitening filter

    assert_eq!(rfinput[5].signal_chain_corrections_index, None); // RRI, whitening filter
}
