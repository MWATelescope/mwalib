// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
extern crate tempdir;
use fitsio::FitsFile;

/// Function to allow access to a temporary file. Temp directory and File is dropped once out of scope
/// This is derived from fitsio crate
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

pub fn with_new_temp_fits_file<F>(filename: &str, callback: F)
where
    F: for<'a> Fn(&'a mut FitsFile),
{
    let tdir = tempdir::TempDir::new("fitsio-").unwrap();
    let tdir_path = tdir.path();
    let filename = tdir_path.join(filename);

    let filename_str = filename.to_str().expect("cannot create string filename");

    let mut fptr = FitsFile::create(filename_str)
        .open()
        .expect("Couldn't open tempfile");

    callback(&mut fptr);
}

/// Given the number of antennas, calculate the number of baselines (cross+autos)
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
pub fn get_antennas_from_baseline(baseline: usize, num_antennas: usize) -> Option<(usize, usize)> {
    let mut baseline_index = 0;
    for ant1 in 0..num_antennas {
        for ant2 in ant1..num_antennas {
            if baseline_index == baseline {
                return Some((ant1, ant2));
            }
            baseline_index += 1;
        }
    }

    // Baseline was not found at all
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_baseline_count() {
        assert_eq!(8256, get_baseline_count(128));
    }

    #[test]
    fn test_get_antennas_from_baseline() {
        assert_eq!(Some((0, 0)), get_antennas_from_baseline(0, 128));
        assert_eq!(Some((1, 1)), get_antennas_from_baseline(128, 128));
        assert_eq!(Some((127, 127)), get_antennas_from_baseline(8255, 128));
    }
}
