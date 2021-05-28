// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Unit tests for antenna metadata
*/
#[cfg(test)]
use super::*;
use float_cmp::*;

#[test]
fn test_populate_antennas() {
    // Create some rf_inputs
    let rf_inputs: Vec<Rfinput> = vec![
        Rfinput {
            input: 0,
            ant: 101,
            tile_id: 101,
            tile_name: String::from("Tile101"),
            pol: Pol::X,
            electrical_length_m: 101.,
            north_m: 11.,
            east_m: 21.,
            height_m: 31.,
            vcs_order: 0,
            subfile_order: 0,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 1,
            rec_slot_number: 0,
        },
        Rfinput {
            input: 1,
            ant: 101,
            tile_id: 101,
            tile_name: String::from("Tile101"),
            pol: Pol::Y,
            electrical_length_m: 102.,
            north_m: 12.,
            east_m: 22.,
            height_m: 32.,
            vcs_order: 4,
            subfile_order: 1,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 1,
            rec_slot_number: 1,
        },
        Rfinput {
            input: 2,
            ant: 102,
            tile_id: 102,
            tile_name: String::from("Tile102"),
            pol: Pol::X,
            electrical_length_m: 103.,
            north_m: 13.,
            east_m: 23.,
            height_m: 33.,
            vcs_order: 0,
            subfile_order: 0,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 2,
            rec_slot_number: 0,
        },
        Rfinput {
            input: 3,
            ant: 102,
            tile_id: 102,
            tile_name: String::from("Tile102"),
            pol: Pol::Y,
            electrical_length_m: 104.,
            north_m: 14.,
            east_m: 24.,
            height_m: 34.,
            vcs_order: 4,
            subfile_order: 1,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 2,
            rec_slot_number: 1,
        },
        Rfinput {
            input: 4,
            ant: 103,
            tile_id: 103,
            tile_name: String::from("Tile103"),
            pol: Pol::X,
            electrical_length_m: 105.,
            north_m: 15.,
            east_m: 25.,
            height_m: 35.,
            vcs_order: 0,
            subfile_order: 0,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 3,
            rec_slot_number: 0,
        },
        Rfinput {
            input: 5,
            ant: 103,
            tile_id: 103,
            tile_name: String::from("Tile103"),
            pol: Pol::Y,
            electrical_length_m: 106.,
            north_m: 16.,
            east_m: 26.,
            height_m: 36.,
            vcs_order: 4,
            subfile_order: 1,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 3,
            rec_slot_number: 1,
        },
        Rfinput {
            input: 6,
            ant: 104,
            tile_id: 104,
            tile_name: String::from("Tile104"),
            pol: Pol::X,
            electrical_length_m: 107.,
            north_m: 17.,
            east_m: 27.,
            height_m: 37.,
            vcs_order: 0,
            subfile_order: 0,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 4,
            rec_slot_number: 0,
        },
        Rfinput {
            input: 7,
            ant: 104,
            tile_id: 104,
            tile_name: String::from("Tile104"),
            pol: Pol::Y,
            electrical_length_m: 108.,
            north_m: 18.,
            east_m: 28.,
            height_m: 38.,
            vcs_order: 4,
            subfile_order: 1,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 4,
            rec_slot_number: 1,
        },
    ];

    // Call populate
    let antennas = Antenna::populate_antennas(&rf_inputs);

    // Check
    assert_eq!(antennas.len(), 4);
    assert_eq!(antennas[0].tile_id, 101);
    assert_eq!(antennas[0].ant, 101);
    assert!(approx_eq!(
        f64,
        antennas[0].electrical_length_m,
        antennas[0].rfinput_x.electrical_length_m,
        F64Margin::default()
    ));
    assert_eq!(antennas[1].rfinput_y.pol, Pol::Y);
    assert_eq!(antennas[1].tile_name, "Tile102");
    assert!(approx_eq!(
        f64,
        antennas[1].north_m,
        antennas[1].rfinput_x.north_m,
        F64Margin::default()
    ));
    assert!(approx_eq!(
        f64,
        antennas[1].east_m,
        antennas[1].rfinput_x.east_m,
        F64Margin::default()
    ));
    assert!(approx_eq!(
        f64,
        antennas[1].height_m,
        antennas[1].rfinput_x.height_m,
        F64Margin::default()
    ));
    assert_eq!(antennas[2].tile_name, "Tile103");
    assert_eq!(antennas[2].rfinput_x.input, 4);
    assert_eq!(antennas[3].tile_id, 104);
}

#[test]
fn test_antenna_debug() {
    // Create some rf_inputs
    let rf_inputs: Vec<Rfinput> = vec![
        Rfinput {
            input: 0,
            ant: 101,
            tile_id: 101,
            tile_name: String::from("Tile101"),
            pol: Pol::X,
            electrical_length_m: 101.,
            north_m: 11.,
            east_m: 21.,
            height_m: 31.,
            vcs_order: 0,
            subfile_order: 0,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 1,
            rec_slot_number: 0,
        },
        Rfinput {
            input: 1,
            ant: 101,
            tile_id: 101,
            tile_name: String::from("Tile101"),
            pol: Pol::Y,
            electrical_length_m: 102.,
            north_m: 12.,
            east_m: 22.,
            height_m: 32.,
            vcs_order: 4,
            subfile_order: 1,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 1,
            rec_slot_number: 1,
        },
    ];

    // Call populate
    let antennas = Antenna::populate_antennas(&rf_inputs);

    assert_eq!(format!("{:?}", antennas[0]), "Tile101");
}
