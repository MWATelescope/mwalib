// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! General helper/utility methods

use crate::antenna;
use crate::MWAVersion;
use std::fs::File;
use std::io::{Error, Write};
use std::{fmt::Debug, fmt::Display, mem, slice};

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

/// Returns a bool based on whether this cable flavour has a whitening filter. (Used by rfinput::new())
///
/// If Whitening_Filter col is present in TILEDATA hdu, then it will be used in the first instance.
/// A whitening_filter value of -1 indicates its absense and it will use the cable flavour rules instead.
/// Rules for determining via cable_flavour can be found in github issue #64.
///
/// # Arguments
///
/// * `flavour` - refernce to a string which has the cable flavour (value of the "flavor" column from
///   TILEDATA HDU of the metafits file).
///
/// # Returns
///
/// * True if this flavour has a whitening filter, False if not.
///
pub fn has_whitening_filter(flavour: &str, whitening_filter: i32) -> bool {
    if whitening_filter == -1 {
        // no whitening filter col in this metafits. Use the flavour logic
        if flavour.len() >= 3 {
            match flavour[0..3].to_uppercase().as_str() {
                "RG6" => !matches!(flavour, "RG6_90"),
                "LMR" => true,
                _ => false,
            }
        } else {
            false
        }
    } else {
        // whitening filter is present
        // any non-zero means it HAS a whitening filter
        whitening_filter != 0
    }
}

/// Returns True if the f32's are equal even if one or both are NaNs.
/// Code is from https://stackoverflow.com/questions/40767815/how-do-i-check-whether-a-vector-is-equal-to-another-vector-that-contains-nan-and
///
/// # Arguments
///
/// * `a` - first f32 to compare
///
/// * `b` - second f32 to compare
///
/// # Returns
///
/// * Equality of `a and `b`
///
pub fn eq_with_nan_eq_f32(a: f32, b: f32) -> bool {
    (a.is_nan() && b.is_nan()) || (a == b)
}

/// Returns True if the Vec<f32>'s are equal even if one or both contain NaNs.
/// Code is from https://stackoverflow.com/questions/40767815/how-do-i-check-whether-a-vector-is-equal-to-another-vector-that-contains-nan-and
///
/// # Arguments
///
/// * `va` - first Vec<f32> to compare
///
/// * `vb` - second Vec<f32> to compare
///
/// # Returns
///
/// * Equality of `a and `b`
///
pub fn vec_compare_f32(va: &[f32], vb: &[f32]) -> bool {
    (va.len() == vb.len()) &&  // zip stops at the shortest
     va.iter()
       .zip(vb)
       .all(|(a,b)| eq_with_nan_eq_f32(*a,*b))
}

/// Returns True if the f64's are equal even if one or both are NaNs.
/// Code is from https://stackoverflow.com/questions/40767815/how-do-i-check-whether-a-vector-is-equal-to-another-vector-that-contains-nan-and
///
/// # Arguments
///
/// * `a` - first f64 to compare
///
/// * `b` - second f64 to compare
///
/// # Returns
///
/// * Equality of `a and `b`
///
pub fn eq_with_nan_eq_f64(a: f64, b: f64) -> bool {
    (a.is_nan() && b.is_nan()) || (a == b)
}

/// Returns True if the Vec<f64>'s are equal even if one or both contain NaNs.
/// Code is from https://stackoverflow.com/questions/40767815/how-do-i-check-whether-a-vector-is-equal-to-another-vector-that-contains-nan-and
///
/// # Arguments
///
/// * `va` - first Vec<f64> to compare
///
/// * `vb` - second Vec<f64> to compare
///
/// # Returns
///
/// * Equality of `a and `b`
///
pub fn vec_compare_f64(va: &[f64], vb: &[f64]) -> bool {
    (va.len() == vb.len()) &&  // zip stops at the shortest
     va.iter()
       .zip(vb)
       .all(|(a,b)| eq_with_nan_eq_f64(*a,*b))
}

/// Returns a formatted string to 'pretty print' a vector
/// Example 1:
/// show_first_elements = 2
///
/// vec = 0, 1, 2, 3, 4, 5, 6, 7, 8, 9
///
/// result = "[0, 1...]"
///
/// Example 2:
/// show_first_elements = 12
/// vec = 0, 1, 2, 3, 4, 5, 6, 7, 8, 9
///
/// result = "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"
///
/// # Arguments
///
/// * `vec` - slice of type T to get a formatted string for. Must support the Display trait
///
/// * `show_first_elements` - how many elements from the start of the slice should we display?
///
/// # Returns
///
/// * a string which puts an elipses "..." after the specified number of elements are included in the string
///   - basically a "pretty" format of a slice
///
pub fn pretty_print_vec_to_string<T>(vec: &[T], show_first_elements: usize) -> String
where
    T: Display + Debug,
{
    let vec_len = vec.len();

    // Check for silliness
    if vec_len == 0 || show_first_elements == 0 {
        return String::from("[]");
    }

    let pos: usize = if vec_len < show_first_elements {
        vec_len
    } else {
        show_first_elements
    };

    let suffix: String = if vec_len <= show_first_elements {
        String::from("")
    } else {
        String::from("...")
    };

    let mut ret_string = vec[0..pos]
        .iter()
        .fold(String::new(), |acc, num| {
            acc + format!("{}", &num).as_str() + ", "
        })
        .to_string();

    // Remove the last ", " and format properly
    ret_string.remove(ret_string.len() - 1);
    ret_string.remove(ret_string.len() - 1);

    format!("[{}{}]", ret_string, suffix)
}

/// Helper fuctions to generate (small-sh) test voltage files
/// for mwax test files they contain an incrememting byte for the real in each samples and decrementing byte value for the imag value.
/// for legacy test files they contain a single incrememnting byte for the real/imag value.
pub fn generate_test_voltage_file(
    filename: &str,
    mwa_version: MWAVersion,
    num_voltage_blocks: usize,
    samples_per_block: usize,
    rf_inputs: usize,
    fine_chans: usize,
    bytes_per_sample: usize,
    initial_value: u8,
) -> Result<String, Error> {
    // initialization test data
    let mut output_file: File = File::create(filename)?;

    // Write out header if one is needed
    if mwa_version == MWAVersion::VCSMWAXv2 {
        let header_buffer: Vec<u8> = vec![0x01; 4096];
        output_file
            .write_all(&header_buffer)
            .expect("Cannot write header!");
    }

    // Each voltage_block has samples_per_rf_fine for each combination of rfinputs x fine_chans
    let num_bytes_per_voltage_block = samples_per_block * rf_inputs * fine_chans * bytes_per_sample;

    // Write out delay block if one is needed
    if mwa_version == MWAVersion::VCSMWAXv2 {
        let delay_buffer: Vec<u8> = vec![0x02; num_bytes_per_voltage_block];
        output_file
            .write_all(&delay_buffer)
            .expect("Cannot write delay block!");
    }

    // Write out num_voltage_blocks
    //

    // Loop for each voltage block
    // legacy: 1 blocks per file
    // mwax  : 160 blocks per file
    for b in 0..num_voltage_blocks {
        let mut value1: u8;
        let mut value2: u8;

        // Allocate a buffer
        let mut voltage_block_buffer: Vec<u8> = vec![0; num_bytes_per_voltage_block];

        // Populate the buffer with test data
        let mut bptr: usize = 0; // Keeps track of where we are in the byte array

        match mwa_version {
            MWAVersion::VCSMWAXv2 => {
                // Data should be written in the following order (slowest to fastest axis)
                // voltage_block (time1), rf_input, sample (time2), value (complex)
                for r in 0..rf_inputs {
                    for s in 0..samples_per_block {
                        // Encode the data location (plus inital value)
                        value1 =
                            ((initial_value as u64 + (b * 5 + r * 4 + s * 2) as u64) % 256) as u8;

                        // Value 2 is the reverse
                        value2 = 255 - value1;

                        // Byte 1
                        voltage_block_buffer[bptr] = value1;
                        bptr += 1;

                        // Byte 2
                        voltage_block_buffer[bptr] = value2;
                        bptr += 1;
                    }
                }
            }
            MWAVersion::VCSLegacyRecombined => {
                // Data should be written in the following order (slowest to fastest axis)
                // sample (time1), fine_chan, rf_input, value (complex)
                for s in 0..samples_per_block {
                    for f in 0..fine_chans {
                        for r in 0..rf_inputs {
                            // Encode the data location (plus inital value)
                            value1 = ((initial_value as u64 + (s * 4 + f * 3 + r * 2) as u64) % 256)
                                as u8;

                            // In this case 1 byte is split into 4bits real and 4bits imag
                            voltage_block_buffer[bptr] = value1;
                            bptr += 1;
                        }
                    }
                }
            }
            _ => {}
        }
        output_file
            .write_all(&voltage_block_buffer)
            .expect("Cannot write voltage data block");
    }

    output_file.flush()?;

    Ok(String::from(filename))
}
