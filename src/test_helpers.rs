/// This is for helper fuctions for our tests
use fitsio::hdu::*;
use fitsio::*;
use std::fs::*;

pub fn helper_make_fits_file(filename: &str) -> (String, FitsFile, FitsHdu) {
    // FitsFile::create() expects the filename passed in to not
    // exist. Delete it if it exists.
    if std::path::Path::new(filename).exists() {
        remove_file(filename).unwrap();
    }
    let mut fptr = FitsFile::create(filename)
        .open()
        .expect("Couldn't open tempfile");
    let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

    (filename.to_string(), fptr, hdu)
}
