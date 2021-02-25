// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
General helper/utility methods
*/
use crate::antenna;
use std::{mem, slice};

/// Function to take d m s and return the decimal degrees.
///
/// # Arguments
///
/// * `degrees` - integer number of degrees
///
/// * `minutes` - integer number of minutes
///
/// * `seconds` - number of seconds (may be a float)
///
///
/// # Returns
///
/// * a float64 containing the number of decimal degrees
///
pub fn dms_to_degrees(degrees: i32, minutes: u32, seconds: f64) -> f64 {
    let deg = degrees.abs() as f64 + (minutes as f64 / 60_f64) + (seconds.abs() / 3600_f64);

    if degrees < 0 {
        -deg
    } else {
        deg
    }
}

// Helper to write out f32 slice as u8 slice
pub fn as_u8_slice(v: &[f32]) -> &[u8] {
    let element_size = mem::size_of::<i32>();
    unsafe { slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * element_size) }
}

/// Function to allow access to a temporary file. Temp directory and File is dropped once out of scope.
/// This is derived from fitsio crate.
///
/// # Arguments
///
/// * `filename` - string filename to use when creating a temp file
///
///
/// # Returns
///
/// * A temporary file which will be deleted (along with the temp directory created) once out of scope
///
#[cfg(test)]
pub fn with_temp_file<F>(filename: &str, callback: F)
where
    F: for<'a> Fn(&'a str),
{
    let tdir = tempdir::TempDir::new("fitsio-").unwrap();
    let tdir_path = tdir.path();
    let filename = tdir_path.join(filename);

    let filename_str = filename.to_str().expect("cannot create string filename");
    callback(filename_str);
}

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
    F: for<'a> Fn(&'a mut fitsio::FitsFile),
{
    let tdir = tempdir::TempDir::new("fitsio-").unwrap();
    let tdir_path = tdir.path();
    let filename = tdir_path.join(filename);

    let filename_str = filename.to_str().expect("cannot create string filename");

    let mut fptr = fitsio::FitsFile::create(filename_str)
        .open()
        .expect("Couldn't open tempfile");

    callback(&mut fptr);
}

/// Given the number of antennas, calculate the number of baselines (cross+autos)
///
/// # Arguments
///
/// * `antennas` - number of antennas in the array
///
///
/// # Returns
///
/// * total number of baselines (including autos)
///
pub fn get_baseline_count(antennas: usize) -> usize {
    antennas * (antennas + 1) / 2
}

/// Given a baseline index, return a tuple of (ant1,ant2) for a std right upper triangle e.g. (where N is num antennas)
/// Returns None if baseline was not found (your baseline is out of range)
/// 0,0
/// 0,1
/// ...
/// 0,N-1
/// 1,1
/// 1,2
/// 1,N-1
/// ...
/// 2,2
/// ...
/// N-1,N-1
///
/// # Arguments
///
/// * `baseline` - index of baseline.
///
/// * `num_antennas` - total number of antennas in the array.
///
///
/// # Returns
///
/// * An Option containing antenna1 index and antenna2 index if baseline exists, or None if doesn't exist.
///
pub fn get_antennas_from_baseline(baseline: usize, num_antennas: usize) -> Option<(usize, usize)> {
    let ant1 = (-0.5
        * ((4 * num_antennas * num_antennas + 4 * num_antennas - 8 * baseline + 1) as f32).sqrt()
        + num_antennas as f32
        + 1. / 2.) as usize;

    let ant2 = baseline - (ant1 * num_antennas - (ant1 * ant1 + ant1) / 2);

    if ant1 > num_antennas - 1 || ant2 > num_antennas - 1 {
        None
    } else {
        Some((ant1, ant2))
    }
}

/// Given two antenna indicies, return the baseline index.
///
/// # Arguments
///
/// * `antenna1` - index of antenna1
///
/// * `antenna2` - index of antenna2
///
/// * `num_antennas` - total number of antennas in the array.
///
///
/// # Returns
///
/// * An Option containing a baseline index if baseline exists, or None if doesn't exist.
///
pub fn get_baseline_from_antennas(
    antenna1: usize,
    antenna2: usize,
    num_antennas: usize,
) -> Option<usize> {
    let mut baseline_index = 0;
    for ant1 in 0..num_antennas {
        for ant2 in ant1..num_antennas {
            if ant1 == antenna1 && ant2 == antenna2 {
                return Some(baseline_index);
            }
            baseline_index += 1;
        }
    }

    // Baseline was not found at all
    None
}

/// Given two antenna names and the vector of Antenna structs from metafits, return the baseline index.
///
/// # Arguments
///
/// * `antenna1` - Tile name of antenna1
///
/// * `antenna2` - Tile name of antenna2
///
/// * `antennas` - A vector of Antenna structs.
///
///
/// # Returns
///
/// * An Option containing a baseline index if baseline exists, or None if doesn't exist.
///
pub fn get_baseline_from_antenna_names(
    antenna1_tile_name: String,
    antenna2_tile_name: String,
    antennas: &[antenna::Antenna],
) -> usize {
    let mut baseline_index = 0;

    let antenna1_index = antennas
        .iter()
        .position(|a| a.tile_name == antenna1_tile_name)
        .unwrap();
    let antenna2_index = antennas
        .iter()
        .position(|a| a.tile_name == antenna2_tile_name)
        .unwrap();

    for ant1 in 0..antennas.len() {
        for ant2 in ant1..antennas.len() {
            if ant1 == antenna1_index && ant2 == antenna2_index {
                return baseline_index;
            }
            baseline_index += 1;
        }
    }

    // Baseline was not found at all
    unreachable!("Baseline was not found")
}

/// Returns a UNIX time given a GPStime
///
/// NOTE: this method relies on the fact that metafits files have the following information, which we use to
/// determine the UNIX vs GPS offset in seconds, which has already been corrected for leap seconds:assert_eq!
///
/// GOODTIME = the first UNIX time of "good" data (after receivers, beamformers, etc have settled down)
/// QUACKTIM = the number of seconds added to the scheduled UNIX start time to skip "bad" data.
/// GPSTIME  = the GPS scheduled start time of an observation
///
/// Thus we can subtract QUACKTIM from GOODTIME to get the UNIX scheduled start time.assert_eq!
/// Know things and that we have the GPSTIME for the same instant, we can compute and offset and
/// use THAT to adjust any times in THIS OBSERVATION. NOTE: this only works because the telescope garauntees
/// that we will never observe OVER a leap second change.
///
/// # Arguments
///
/// * `gpstime_ms` - GPS time (in ms) you want to convert to UNIX timestamp
///
/// * `mwa_start_gps_time_ms` - Scheduled GPS start time (in ms) of observation according to metafits.
///
/// * `mwa_start_unix_time_ms` - Scheduled UNIX start time (in ms) according to the metafits (GOODTIM-QUACKTIM).
///    
///
/// # Returns
///
/// * The UNIX time (in ms) converted from the `gpstime_ms`.
///
pub fn convert_gpstime_to_unixtime(
    gpstime_ms: u64,
    mwa_start_gpstime_ms: u64,
    mwa_start_unixtime_ms: u64,
) -> u64 {
    // We have a UNIX time reference and a gpstime reference
    // Compute an offset
    let offset_ms = mwa_start_unixtime_ms - mwa_start_gpstime_ms;

    // The new converted Unix time is gpstime + offset
    gpstime_ms + offset_ms
}

/// Returns a UNIX time given a GPStime
///
/// NOTE: see `convert_gpstime_to_unixtime` for more details.
///
/// # Arguments
///
/// * `unixtime_ms` - GPS time (in ms) you want to convert to UNIX timestamp
///
/// * `mwa_start_gps_time_ms` - Scheduled GPS start time (in ms) of observation according to metafits.
///
/// * `mwa_start_unix_time_ms` - Scheduled UNIX start time (in ms) according to the metafits (GOODTIM-QUACKTIM).
///    
///
/// # Returns
///
/// * The GPS time (in ms) converted from the `unixtime_ms`.
///
pub fn convert_unixtime_to_gpstime(
    unixtime_ms: u64,
    mwa_start_gpstime_ms: u64,
    mwa_start_unixtime_ms: u64,
) -> u64 {
    // We have a UNIX time reference and a gpstime reference
    // Compute an offset
    let offset_ms = mwa_start_unixtime_ms - mwa_start_gpstime_ms;

    // The new converted gps time is unix time - offset
    unixtime_ms - offset_ms
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::antenna::*;
    use crate::rfinput::*;
    use float_cmp::*;

    #[test]
    fn test_convert_gpstime_to_unixtime() {
        // Tested using https://www.andrews.edu/~tzs/timeconv/timedisplay.php
        let gpstime_ms = 1298013490_000;
        let mwa_start_gpstime_ms = 1242552568_000;
        let mwa_start_unixtime_ms = 1558517350_000;

        let new_unixtime_ms =
            convert_gpstime_to_unixtime(gpstime_ms, mwa_start_gpstime_ms, mwa_start_unixtime_ms);
        assert_eq!(new_unixtime_ms, 1613978272_000);
    }

    #[test]
    fn test_convert_unixtime_to_gpstime() {
        // Tested using https://www.andrews.edu/~tzs/timeconv/timedisplay.php
        let unixtime_ms = 1613978272_000;
        let mwa_start_gpstime_ms = 1242552568_000;
        let mwa_start_unixtime_ms = 1558517350_000;

        let new_unixtime_ms =
            convert_unixtime_to_gpstime(unixtime_ms, mwa_start_gpstime_ms, mwa_start_unixtime_ms);
        assert_eq!(new_unixtime_ms, 1298013490_000);
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
        let dummy_rf_input_x = RFInput {
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
        };

        let dummy_rf_input_y = RFInput {
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
        };

        ants.push(Antenna {
            ant: 101,
            tile_id: 101,
            tile_name: String::from("tile101"),
            x_pol: dummy_rf_input_x.clone(),
            y_pol: dummy_rf_input_y.clone(),
        });

        ants.push(Antenna {
            ant: 102,
            tile_id: 102,
            tile_name: String::from("tile102"),
            x_pol: dummy_rf_input_x.clone(),
            y_pol: dummy_rf_input_y.clone(),
        });

        ants.push(Antenna {
            ant: 103,
            tile_id: 103,
            tile_name: String::from("tile103"),
            x_pol: dummy_rf_input_x.clone(),
            y_pol: dummy_rf_input_y.clone(),
        });

        ants.push(Antenna {
            ant: 104,
            tile_id: 104,
            tile_name: String::from("tile104"),
            x_pol: dummy_rf_input_x.clone(),
            y_pol: dummy_rf_input_y.clone(),
        });

        ants.push(Antenna {
            ant: 105,
            tile_id: 105,
            tile_name: String::from("tile105"),
            x_pol: dummy_rf_input_x.clone(),
            y_pol: dummy_rf_input_y.clone(),
        });

        ants.push(Antenna {
            ant: 106,
            tile_id: 106,
            tile_name: String::from("tile106"),
            x_pol: dummy_rf_input_x.clone(),
            y_pol: dummy_rf_input_y.clone(),
        });

        ants.push(Antenna {
            ant: 107,
            tile_id: 107,
            tile_name: String::from("tile107"),
            x_pol: dummy_rf_input_x.clone(),
            y_pol: dummy_rf_input_y.clone(),
        });

        ants.push(Antenna {
            ant: 108,
            tile_id: 108,
            tile_name: String::from("tile108"),
            x_pol: dummy_rf_input_x,
            y_pol: dummy_rf_input_y,
        });

        // Now do some tests!
        assert_eq!(
            0,
            get_baseline_from_antenna_names(
                String::from("tile101"),
                String::from("tile101"),
                &ants
            ),
            "Baseline from antenna names test 1 is wrong"
        );
        assert_eq!(
            1,
            get_baseline_from_antenna_names(
                String::from("tile101"),
                String::from("tile102"),
                &ants
            ),
            "Baseline from antenna names test 2 is wrong"
        );
        assert_eq!(
            7,
            get_baseline_from_antenna_names(
                String::from("tile101"),
                String::from("tile108"),
                &ants
            ),
            "Baseline from antenna names test 3 is wrong"
        );
        assert_eq!(
            8,
            get_baseline_from_antenna_names(
                String::from("tile102"),
                String::from("tile102"),
                &ants
            ),
            "Baseline from antenna names test 4 is wrong"
        );
        assert_eq!(
            14,
            get_baseline_from_antenna_names(
                String::from("tile102"),
                String::from("tile108"),
                &ants
            ),
            "Baseline from antenna names test 5 is wrong"
        );
    }

    #[test]
    #[should_panic]
    fn test_get_baseline_from_antenna_names_ant1_not_valid() {
        // Create a small antenna vector
        let mut ants: Vec<Antenna> = Vec::new();

        // We need a dummy rf inputs
        let dummy_rf_input_x = RFInput {
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
        };

        let dummy_rf_input_y = RFInput {
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
        };

        ants.push(Antenna {
            ant: 101,
            tile_id: 101,
            tile_name: String::from("tile101"),
            x_pol: dummy_rf_input_x.clone(),
            y_pol: dummy_rf_input_y.clone(),
        });

        ants.push(Antenna {
            ant: 102,
            tile_id: 102,
            tile_name: String::from("tile102"),
            x_pol: dummy_rf_input_x,
            y_pol: dummy_rf_input_y,
        });

        // Now do some tests!
        let _panic_result = get_baseline_from_antenna_names(
            String::from("tile110"),
            String::from("tile102"),
            &ants,
        );
    }

    #[test]
    #[should_panic]
    fn test_get_baseline_from_antenna_names_ant2_not_valid() {
        // Create a small antenna vector
        let mut ants: Vec<Antenna> = Vec::new();

        // We need a dummy rf inputs
        let dummy_rf_input_x = RFInput {
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
        };

        let dummy_rf_input_y = RFInput {
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
        };

        ants.push(Antenna {
            ant: 101,
            tile_id: 101,
            tile_name: String::from("tile101"),
            x_pol: dummy_rf_input_x.clone(),
            y_pol: dummy_rf_input_y.clone(),
        });

        ants.push(Antenna {
            ant: 102,
            tile_id: 102,
            tile_name: String::from("tile102"),
            x_pol: dummy_rf_input_x,
            y_pol: dummy_rf_input_y,
        });

        // Now do some tests!
        let _panic_result = get_baseline_from_antenna_names(
            String::from("tile101"),
            String::from("tile112"),
            &ants,
        );
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
}
