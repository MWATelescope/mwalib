// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! General helper/utility methods

use crate::antenna;
use std::{mem, slice};

#[cfg(test)]
pub(crate) mod test;

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
    match unixtime_ms {
        0 => 0,
        _ => {
            // We have a UNIX time reference and a gpstime reference
            // Compute an offset
            let offset_ms = mwa_start_unixtime_ms - mwa_start_gpstime_ms;

            // The new converted gps time is unix time - offset
            unixtime_ms - offset_ms
        }
    }
}
