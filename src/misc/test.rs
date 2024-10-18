// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Unit tests for misc utility functions

use core::f32;

#[cfg(test)]
use super::*;
use crate::antenna::*;
use crate::fits_read::*;
use crate::rfinput::*;
use float_cmp::*;

/// Function to allow access to a temporary FITS file. Temp directory and File is dropped once out of scope.
/// This is derived from fitsio crate.
///
/// # Arguments
///
/// * `filename` - string filename to use when creating a temp FITS file
///
///
/// # Returns
///
/// * A temporary FITS file which will be deleted (along with the temp directory created) once out of scope
///
#[cfg(test)]
pub fn with_new_temp_fits_file<F>(filename: &str, callback: F)
where
    F: for<'a> Fn(&'a mut MWAFitsFile),
{
    let tdir = tempdir::TempDir::new("fitsio-").unwrap();
    let tdir_path = tdir.path();
    let filename = tdir_path.join(filename);

    let filename_str = filename.to_str().expect("cannot create string filename");

    let fptr = fitsio::FitsFile::create(filename_str)
        .open()
        .expect("Couldn't open tempfile");

    let mut mwa_fits_file = MWAFitsFile::new(filename_str.into(), fptr);

    callback(&mut mwa_fits_file);
}

#[test]
fn test_convert_gpstime_to_unixtime() {
    // Tested using https://www.andrews.edu/~tzs/timeconv/timedisplay.php
    let gpstime_ms = 1_298_013_490_000;
    let mwa_start_gpstime_ms = 1_242_552_568_000;
    let mwa_start_unixtime_ms = 1_558_517_350_000;

    let new_unixtime_ms =
        convert_gpstime_to_unixtime(gpstime_ms, mwa_start_gpstime_ms, mwa_start_unixtime_ms);
    assert_eq!(new_unixtime_ms, 1_613_978_272_000);
}

#[test]
fn test_convert_unixtime_to_gpstime() {
    // Tested using https://www.andrews.edu/~tzs/timeconv/timedisplay.php
    let unixtime_ms = 1_613_978_272_000;
    let mwa_start_gpstime_ms = 1_242_552_568_000;
    let mwa_start_unixtime_ms = 1_558_517_350_000;

    let new_unixtime_ms =
        convert_unixtime_to_gpstime(unixtime_ms, mwa_start_gpstime_ms, mwa_start_unixtime_ms);
    assert_eq!(new_unixtime_ms, 1_298_013_490_000);
}

#[test]
fn test_get_baseline_count() {
    assert_eq!(3, get_baseline_count(2));
    assert_eq!(8256, get_baseline_count(128));
}

#[test]
fn test_get_antennas_from_baseline() {
    assert_eq!(Some((0, 0)), get_antennas_from_baseline(0, 128));
    assert_eq!(Some((1, 1)), get_antennas_from_baseline(128, 128));
    assert_eq!(Some((127, 127)), get_antennas_from_baseline(8255, 128));
    assert_eq!(None, get_antennas_from_baseline(8256, 128));
}

#[test]
fn test_get_baseline_from_antennas() {
    assert_eq!(Some(0), get_baseline_from_antennas(0, 0, 128));
    assert_eq!(Some(128), get_baseline_from_antennas(1, 1, 128));
    assert_eq!(Some(8255), get_baseline_from_antennas(127, 127, 128));
    assert_eq!(None, get_baseline_from_antennas(128, 128, 128));
}

#[test]
fn test_get_baseline_from_antenna_names1() {
    // Create a small antenna vector
    let mut ants: Vec<Antenna> = Vec::new();

    // We need a dummy rf inputs
    let dummy_rf_input_x = Rfinput {
        input: 0,
        ant: 0,
        tile_id: 0,
        tile_name: String::from("dummy1"),
        pol: Pol::X,
        electrical_length_m: 0.,
        north_m: 0.,
        east_m: 0.,
        height_m: 0.,
        vcs_order: 0,
        subfile_order: 0,
        flagged: false,
        digital_gains: vec![],
        dipole_gains: vec![],
        dipole_delays: vec![],
        rec_number: 1,
        rec_slot_number: 0,
        rec_type: ReceiverType::Unknown,
        flavour: String::from("dummy1"),
        has_whitening_filter: false,
        calib_delay: None,
        calib_gains: None,
        signal_chain_corrections_index: None,
    };

    let dummy_rf_input_y = Rfinput {
        input: 1,
        ant: 0,
        tile_id: 1,
        tile_name: String::from("dummy1"),
        pol: Pol::Y,
        electrical_length_m: 0.,
        north_m: 0.,
        east_m: 0.,
        height_m: 0.,
        vcs_order: 0,
        subfile_order: 1,
        flagged: false,
        digital_gains: vec![],
        dipole_gains: vec![],
        dipole_delays: vec![],
        rec_number: 1,
        rec_slot_number: 1,
        rec_type: ReceiverType::Unknown,
        flavour: String::from("dummy1"),
        has_whitening_filter: false,
        calib_delay: None,
        calib_gains: None,
        signal_chain_corrections_index: None,
    };

    ants.push(Antenna {
        ant: 101,
        tile_id: 101,
        tile_name: String::from("tile101"),
        rfinput_x: dummy_rf_input_x.clone(),
        rfinput_y: dummy_rf_input_y.clone(),
        electrical_length_m: dummy_rf_input_x.electrical_length_m,
        north_m: dummy_rf_input_x.north_m,
        east_m: dummy_rf_input_x.east_m,
        height_m: dummy_rf_input_x.height_m,
    });

    ants.push(Antenna {
        ant: 102,
        tile_id: 102,
        tile_name: String::from("tile102"),
        rfinput_x: dummy_rf_input_x.clone(),
        rfinput_y: dummy_rf_input_y.clone(),
        electrical_length_m: dummy_rf_input_x.electrical_length_m,
        north_m: dummy_rf_input_x.north_m,
        east_m: dummy_rf_input_x.east_m,
        height_m: dummy_rf_input_x.height_m,
    });

    ants.push(Antenna {
        ant: 103,
        tile_id: 103,
        tile_name: String::from("tile103"),
        rfinput_x: dummy_rf_input_x.clone(),
        rfinput_y: dummy_rf_input_y.clone(),
        electrical_length_m: dummy_rf_input_x.electrical_length_m,
        north_m: dummy_rf_input_x.north_m,
        east_m: dummy_rf_input_x.east_m,
        height_m: dummy_rf_input_x.height_m,
    });

    ants.push(Antenna {
        ant: 104,
        tile_id: 104,
        tile_name: String::from("tile104"),
        rfinput_x: dummy_rf_input_x.clone(),
        rfinput_y: dummy_rf_input_y.clone(),
        electrical_length_m: dummy_rf_input_x.electrical_length_m,
        north_m: dummy_rf_input_x.north_m,
        east_m: dummy_rf_input_x.east_m,
        height_m: dummy_rf_input_x.height_m,
    });

    ants.push(Antenna {
        ant: 105,
        tile_id: 105,
        tile_name: String::from("tile105"),
        rfinput_x: dummy_rf_input_x.clone(),
        rfinput_y: dummy_rf_input_y.clone(),
        electrical_length_m: dummy_rf_input_x.electrical_length_m,
        north_m: dummy_rf_input_x.north_m,
        east_m: dummy_rf_input_x.east_m,
        height_m: dummy_rf_input_x.height_m,
    });

    ants.push(Antenna {
        ant: 106,
        tile_id: 106,
        tile_name: String::from("tile106"),
        rfinput_x: dummy_rf_input_x.clone(),
        rfinput_y: dummy_rf_input_y.clone(),
        electrical_length_m: dummy_rf_input_x.electrical_length_m,
        north_m: dummy_rf_input_x.north_m,
        east_m: dummy_rf_input_x.east_m,
        height_m: dummy_rf_input_x.height_m,
    });

    ants.push(Antenna {
        ant: 107,
        tile_id: 107,
        tile_name: String::from("tile107"),
        rfinput_x: dummy_rf_input_x.clone(),
        rfinput_y: dummy_rf_input_y.clone(),
        electrical_length_m: dummy_rf_input_x.electrical_length_m,
        north_m: dummy_rf_input_x.north_m,
        east_m: dummy_rf_input_x.east_m,
        height_m: dummy_rf_input_x.height_m,
    });

    ants.push(Antenna {
        ant: 108,
        tile_id: 108,
        tile_name: String::from("tile108"),
        rfinput_x: dummy_rf_input_x.clone(),
        rfinput_y: dummy_rf_input_y,
        electrical_length_m: dummy_rf_input_x.electrical_length_m,
        north_m: dummy_rf_input_x.north_m,
        east_m: dummy_rf_input_x.east_m,
        height_m: dummy_rf_input_x.height_m,
    });

    // Now do some tests!
    assert_eq!(
        0,
        get_baseline_from_antenna_names(String::from("tile101"), String::from("tile101"), &ants),
        "Baseline from antenna names test 1 is wrong"
    );
    assert_eq!(
        1,
        get_baseline_from_antenna_names(String::from("tile101"), String::from("tile102"), &ants),
        "Baseline from antenna names test 2 is wrong"
    );
    assert_eq!(
        7,
        get_baseline_from_antenna_names(String::from("tile101"), String::from("tile108"), &ants),
        "Baseline from antenna names test 3 is wrong"
    );
    assert_eq!(
        8,
        get_baseline_from_antenna_names(String::from("tile102"), String::from("tile102"), &ants),
        "Baseline from antenna names test 4 is wrong"
    );
    assert_eq!(
        14,
        get_baseline_from_antenna_names(String::from("tile102"), String::from("tile108"), &ants),
        "Baseline from antenna names test 5 is wrong"
    );
}

#[test]
#[should_panic]
fn test_get_baseline_from_antenna_names_ant1_not_valid() {
    // Create a small antenna vector
    let mut ants: Vec<Antenna> = Vec::new();

    // We need a dummy rf inputs
    let dummy_rf_input_x = Rfinput {
        input: 0,
        ant: 0,
        tile_id: 0,
        tile_name: String::from("dummy1"),
        pol: Pol::X,
        electrical_length_m: 0.,
        north_m: 0.,
        east_m: 0.,
        height_m: 0.,
        vcs_order: 0,
        subfile_order: 0,
        flagged: false,
        digital_gains: vec![],
        dipole_gains: vec![],
        dipole_delays: vec![],
        rec_number: 1,
        rec_slot_number: 0,
        rec_type: ReceiverType::Unknown,
        flavour: String::from("dummy1"),
        has_whitening_filter: false,
        calib_delay: None,
        calib_gains: None,
        signal_chain_corrections_index: None,
    };

    let dummy_rf_input_y = Rfinput {
        input: 1,
        ant: 0,
        tile_id: 1,
        tile_name: String::from("dummy1"),
        pol: Pol::Y,
        electrical_length_m: 0.,
        north_m: 0.,
        east_m: 0.,
        height_m: 0.,
        vcs_order: 0,
        subfile_order: 1,
        flagged: false,
        digital_gains: vec![],
        dipole_gains: vec![],
        dipole_delays: vec![],
        rec_number: 1,
        rec_slot_number: 1,
        rec_type: ReceiverType::Unknown,
        flavour: String::from("dummy1"),
        has_whitening_filter: false,
        calib_delay: None,
        calib_gains: None,
        signal_chain_corrections_index: None,
    };

    ants.push(Antenna {
        ant: 101,
        tile_id: 101,
        tile_name: String::from("tile101"),
        rfinput_x: dummy_rf_input_x.clone(),
        rfinput_y: dummy_rf_input_y.clone(),
        electrical_length_m: dummy_rf_input_x.electrical_length_m,
        north_m: dummy_rf_input_x.north_m,
        east_m: dummy_rf_input_x.east_m,
        height_m: dummy_rf_input_x.height_m,
    });

    ants.push(Antenna {
        ant: 102,
        tile_id: 102,
        tile_name: String::from("tile102"),
        rfinput_x: dummy_rf_input_x.clone(),
        rfinput_y: dummy_rf_input_y,
        electrical_length_m: dummy_rf_input_x.electrical_length_m,
        north_m: dummy_rf_input_x.north_m,
        east_m: dummy_rf_input_x.east_m,
        height_m: dummy_rf_input_x.height_m,
    });

    // Now do some tests!
    let _panic_result =
        get_baseline_from_antenna_names(String::from("tile110"), String::from("tile102"), &ants);
}

#[test]
#[should_panic]
fn test_get_baseline_from_antenna_names_ant2_not_valid() {
    // Create a small antenna vector
    let mut ants: Vec<Antenna> = Vec::new();

    // We need a dummy rf inputs
    let dummy_rf_input_x = Rfinput {
        input: 0,
        ant: 0,
        tile_id: 0,
        tile_name: String::from("dummy2"),
        pol: Pol::X,
        electrical_length_m: 0.,
        north_m: 0.,
        east_m: 0.,
        height_m: 0.,
        vcs_order: 0,
        subfile_order: 0,
        flagged: false,
        digital_gains: vec![],
        dipole_gains: vec![],
        dipole_delays: vec![],
        rec_number: 1,
        rec_slot_number: 0,
        rec_type: ReceiverType::Unknown,
        flavour: String::from("dummy1"),
        has_whitening_filter: false,
        calib_delay: None,
        calib_gains: None,
        signal_chain_corrections_index: None,
    };

    let dummy_rf_input_y = Rfinput {
        input: 1,
        ant: 0,
        tile_id: 1,
        tile_name: String::from("dummy2"),
        pol: Pol::Y,
        electrical_length_m: 0.,
        north_m: 0.,
        east_m: 0.,
        height_m: 0.,
        vcs_order: 0,
        subfile_order: 1,
        flagged: false,
        digital_gains: vec![],
        dipole_gains: vec![],
        dipole_delays: vec![],
        rec_number: 1,
        rec_slot_number: 1,
        rec_type: ReceiverType::Unknown,
        flavour: String::from("dummy1"),
        has_whitening_filter: false,
        calib_delay: None,
        calib_gains: None,
        signal_chain_corrections_index: None,
    };

    ants.push(Antenna {
        ant: 101,
        tile_id: 101,
        tile_name: String::from("tile101"),
        rfinput_x: dummy_rf_input_x.clone(),
        rfinput_y: dummy_rf_input_y.clone(),
        electrical_length_m: dummy_rf_input_x.electrical_length_m,
        north_m: dummy_rf_input_x.north_m,
        east_m: dummy_rf_input_x.east_m,
        height_m: dummy_rf_input_x.height_m,
    });

    ants.push(Antenna {
        ant: 102,
        tile_id: 102,
        tile_name: String::from("tile102"),
        rfinput_x: dummy_rf_input_x.clone(),
        rfinput_y: dummy_rf_input_y,
        electrical_length_m: dummy_rf_input_x.electrical_length_m,
        north_m: dummy_rf_input_x.north_m,
        east_m: dummy_rf_input_x.east_m,
        height_m: dummy_rf_input_x.height_m,
    });

    // Now do some tests!
    let _panic_result =
        get_baseline_from_antenna_names(String::from("tile101"), String::from("tile112"), &ants);
}

#[test]
fn test_dms_to_degrees_zero() {
    assert!(approx_eq!(
        f64,
        dms_to_degrees(0, 0, 0.),
        0.,
        F64Margin::default()
    ));
}

#[test]
fn test_dms_to_degrees_negative() {
    assert!(
        approx_eq!(
            f64,
            dms_to_degrees(-10, 30, 0.),
            -10.5,
            F64Margin::default()
        ),
        "dms_to_degrees(-10, 30, 0.) == {}",
        dms_to_degrees(-10, 30, 0.)
    );
}
#[test]
fn test_dms_to_degrees_large() {
    let test: f64 = dms_to_degrees(180, 59, 59.9999);
    assert!(
        approx_eq!(f64, test, 180.999_999_972_222_2, F64Margin::default()),
        "{}",
        test
    );
}

#[test]
fn test_has_whitening_filter() {
    // From giuthub issue #64:
    // Every cable flavour starting with 'RG6', except for 'RG6_90' and
    // 'everything starting with LMR' [has a whitening filter].

    // Test empty/not present for both
    assert!(!has_whitening_filter("", -1));

    // Test some string, and no whitening_filter column
    assert!(!has_whitening_filter("SomeOtherString", -1));

    // Test RG6_123 and no whitening_filter column
    assert!(has_whitening_filter("RG6_123", -1));
    // Test RG6 by itself and no whitening_filter column
    assert!(has_whitening_filter("RG6", -1));
    // Test RG6_123 (lowecase) and no whitening_filter column
    assert!(has_whitening_filter("rg6_123", -1));

    // Test RG6_90 returns FALSE! and no whitening_filter column
    assert!(!has_whitening_filter("RG6_90", -1));
    // Test LMR_123 and no whitening_filter column
    assert!(has_whitening_filter("LMR_123", -1));
    // Test LMR by itself and no whitening_filter column
    assert!(has_whitening_filter("LMR", -1));

    // Test RG6_90 returns FALSE! and whitening_filter column of 1
    // True because whitening_filter of 1
    assert!(has_whitening_filter("RG6_90", 1));

    // Test LMR_123 and whitening_filter column of 0
    // False because whitening_filter of 0 even though LMR_123 does has WF
    assert!(!has_whitening_filter("LMR_123", 0));

    // Test LMR by itself and whitening_filter column of 0
    // false because whitening_filter of 0
    assert!(!has_whitening_filter("LMR", 0));
}

#[test]
fn test_f32_nan_eq_no_nans_eq() {
    let x: f32 = 9.2;
    let y: f32 = 9.2;

    assert!(eq_with_nan_eq_f32(x, y));
}

#[test]
fn test_f32_nan_eq_no_nans_neq() {
    let x: f32 = 9.2;
    let y: f32 = 9.3;

    assert!(!eq_with_nan_eq_f32(x, y));
}

#[test]
fn test_f32_nan_eq_one_nan() {
    let x: f32 = 9.2;
    let y: f32 = f32::NAN;

    assert!(!eq_with_nan_eq_f32(x, y));
}

#[test]
fn test_f32_nan_eq_two_nans() {
    let x: f32 = f32::NAN;
    let y: f32 = f32::NAN;

    assert!(eq_with_nan_eq_f32(x, y));
}

#[test]
fn test_vecf32_nan_eq_no_nans_eq() {
    let x: Vec<f32> = vec![9.2; 10];
    let y: Vec<f32> = vec![9.2; 10];

    assert!(vec_compare_f32(&x, &y));
}

#[test]
fn test_vecf32_nan_eq_no_nans_neq1() {
    let x: Vec<f32> = vec![9.2; 9];
    let y: Vec<f32> = vec![9.2; 10];

    assert!(!vec_compare_f32(&x, &y));
}

#[test]
fn test_vecf32_nan_eq_no_nans_neq2() {
    let x: Vec<f32> = vec![9.3; 10];
    let y: Vec<f32> = vec![9.2; 10];

    assert!(!vec_compare_f32(&x, &y));
}

#[test]
fn test_vecf32_nan_eq_no_nans_neq3() {
    let mut x: Vec<f32> = vec![9.2; 10];
    x[3] = 5.0;
    let y: Vec<f32> = vec![9.2; 10];

    assert!(!vec_compare_f32(&x, &y));
}

#[test]
fn test_f64_nan_eq_no_nans_eq() {
    let x: f64 = 9.2;
    let y: f64 = 9.2;

    assert!(eq_with_nan_eq_f64(x, y));
}

#[test]
fn test_f64_nan_eq_no_nans_neq() {
    let x: f64 = 9.2;
    let y: f64 = 9.3;

    assert!(!eq_with_nan_eq_f64(x, y));
}

#[test]
fn test_f64_nan_eq_one_nan() {
    let x: f64 = 9.2;
    let y: f64 = f64::NAN;

    assert!(!eq_with_nan_eq_f64(x, y));
}

#[test]
fn test_f64_nan_eq_two_nans() {
    let x: f64 = f64::NAN;
    let y: f64 = f64::NAN;

    assert!(eq_with_nan_eq_f64(x, y));
}

#[test]
fn test_vecf64_nan_eq_no_nans_eq() {
    let x: Vec<f64> = vec![9.2; 10];
    let y: Vec<f64> = vec![9.2; 10];

    assert!(vec_compare_f64(&x, &y));
}

#[test]
fn test_vecf64_nan_eq_no_nans_neq1() {
    let x: Vec<f64> = vec![9.2; 9];
    let y: Vec<f64> = vec![9.2; 10];

    assert!(!vec_compare_f64(&x, &y));
}

#[test]
fn test_vecf64_nan_eq_no_nans_neq2() {
    let x: Vec<f64> = vec![9.3; 10];
    let y: Vec<f64> = vec![9.2; 10];

    assert!(!vec_compare_f64(&x, &y));
}

#[test]
fn test_vecf64_nan_eq_no_nans_neq3() {
    let mut x: Vec<f64> = vec![9.2; 10];
    x[3] = 5.0;
    let y: Vec<f64> = vec![9.2; 10];

    assert!(!vec_compare_f64(&x, &y));
}

#[test]
fn test_pretty_print_vec_to_string() {
    let test_vec1 = vec![1.1; 10];

    let s = pretty_print_vec_to_string(&test_vec1, 12);
    assert_eq!(
        s,
        String::from("[1.1, 1.1, 1.1, 1.1, 1.1, 1.1, 1.1, 1.1, 1.1, 1.1]")
    );

    let s = pretty_print_vec_to_string(&test_vec1, 10);
    assert_eq!(
        s,
        String::from("[1.1, 1.1, 1.1, 1.1, 1.1, 1.1, 1.1, 1.1, 1.1, 1.1]")
    );

    let s = pretty_print_vec_to_string(&test_vec1, 9);
    assert_eq!(
        s,
        String::from("[1.1, 1.1, 1.1, 1.1, 1.1, 1.1, 1.1, 1.1, 1.1...]")
    );

    let s = pretty_print_vec_to_string(&[1.1; 2], 1);
    assert_eq!(s, String::from("[1.1...]"));

    let s = pretty_print_vec_to_string(&[1.1; 1], 1);
    assert_eq!(s, String::from("[1.1]"));

    let empty_vec: Vec<f32> = Vec::new();
    let s = pretty_print_vec_to_string(&empty_vec, 1);
    assert_eq!(s, String::from("[]"));
}
