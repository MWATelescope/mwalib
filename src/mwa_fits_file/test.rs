// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Unit tests for the mwa_fits_file module
#[cfg(test)]
use fitsio::FitsFile;

#[test]
fn test_mwa_fits_file_new() {
    use super::MWAFitsFile;

    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

    let fptr = FitsFile::open(metafits_filename).expect("Could not open fits file!");

    let mut mwa_fits_file = MWAFitsFile::new(metafits_filename.into(), fptr);

    // Check it works!
    assert_eq!(
        mwa_fits_file.filename.display().to_string(),
        metafits_filename
    );

    // Open a hdu
    let hdu = mwa_fits_file
        .fits_file
        .hdu(0)
        .expect("Could not open PRIMARY HDU");

    // Read the obs_id from the fits file
    let obs_id = hdu
        .read_key::<i64>(&mut mwa_fits_file.fits_file, "GPSTIME")
        .expect("Cannot read key");

    // Ensure it is what is expected
    assert_eq!(obs_id, 1101503312);
}
