// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Unit tests for rfinput metadata
*/
#[cfg(test)]
use super::*;
use crate::misc::test::*;
use crate::*;
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
fn test_read_cell_array() {
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let mut fptr = fits_open!(&metafits_filename).unwrap();
    let hdu = fits_open_hdu!(&mut fptr, 1).unwrap();

    let delays = read_cell_array(&mut fptr, &hdu, "Delays", 0, 16);
    assert!(delays.is_ok());
    assert_eq!(&delays.unwrap(), &[0; 16]);

    let digital_gains = read_cell_array(&mut fptr, &hdu, "Gains", 0, 24);
    assert!(digital_gains.is_ok());
    assert_eq!(
        digital_gains.unwrap(),
        &[
            74, 73, 73, 72, 71, 70, 68, 67, 66, 65, 65, 65, 66, 66, 65, 65, 64, 64, 64, 65, 65, 66,
            67, 68,
        ]
    );

    let asdf = read_cell_array(&mut fptr, &hdu, "NotReal", 0, 24);
    assert!(asdf.is_err());
}

#[test]
fn test_read_metafits_values_from_row_0() {
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let mut metafits_fptr = fits_open!(&metafits_filename).unwrap();

    let metafits_tile_table_hdu = fits_open_hdu!(&mut metafits_fptr, 1).unwrap();

    // Get values from row 1
    let row: RfInputMetafitsTableRow =
        Rfinput::read_metafits_values(&mut metafits_fptr, &metafits_tile_table_hdu, 0).unwrap();
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
}

#[test]
fn test_read_metafits_values_from_invalid_metafits() {
    let metafits_filename = "read_metafits_values_from_invalid_metafits.metafits";

    with_new_temp_fits_file(&metafits_filename, |metafits_fptr| {
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
            Rfinput::read_metafits_values(metafits_fptr, &metafits_tile_table_hdu, 0);

        assert!(metafits_result.is_err());
    });
}

#[test]
fn test_populate_rf_inputs() {
    /* populate_rf_inputs(
        num_inputs: usize,
        metafits_fptr: &mut fitsio::FitsFile,
        metafits_tile_table_hdu: fitsio::hdu::FitsHdu,
        coax_v_factor: f64,
    ) -> Result<Vec<Self>, RfinputError>*/
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let mut metafits_fptr = fits_open!(&metafits_filename).unwrap();
    let metafits_tile_table_hdu = fits_open_hdu!(&mut metafits_fptr, 1).unwrap();
    let result =
        Rfinput::populate_rf_inputs(256, &mut metafits_fptr, metafits_tile_table_hdu, 1.204);

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
    assert_eq!(
        rfinput[0].digital_gains,
        vec![
            74, 73, 73, 72, 71, 70, 68, 67, 66, 65, 65, 65, 66, 66, 65, 65, 64, 64, 64, 65, 65, 66,
            67, 68
        ]
    );
    assert_eq!(
        rfinput[0].dipole_delays,
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(rfinput[0].rec_number, 10);
    assert_eq!(rfinput[0].rec_slot_number, 4);
}
