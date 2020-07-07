// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Helper functions for reading FITS files.
 */

pub mod error;
pub use error::FitsError;

use std::ffi::*;
use std::ptr;

use fitsio::{hdu::*, FitsFile};
use fitsio_sys::{ffgkls, fitsfile};
use libc::c_char;

/// Open a fits file.
///
/// # Examples
///
/// ```
/// # use mwalib::*;
/// # fn main() -> Result<(), FitsError> {
/// let metafits = "test_files/1101503312_1_timestep/1101503312.metafits";
/// let mut fptr = fits_open!(&metafits)?;
/// #     Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! fits_open {
    ($fptr:expr) => {
        _open_fits($fptr, file!(), line!())
    };
}

/// Open a fits file's HDU.
///
/// # Examples
///
/// ```
/// # use mwalib::*;
/// # fn main() -> Result<(), FitsError> {
/// // Open a fits file
/// let metafits = "test_files/1101503312_1_timestep/1101503312.metafits";
/// let mut fptr = fits_open!(&metafits)?;
/// // Open HDU 1 (index 0).
/// let hdu = fits_open_hdu!(&mut fptr, 0)?;
/// #     Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! fits_open_hdu {
    ($fptr:expr, $hdu_num:expr) => {
        _open_hdu($fptr, $hdu_num, file!(), line!())
    };
}

/// Given a FITS file pointer, a HDU that belongs to it, and a keyword that may
/// or may not exist, pull out the value of the keyword, parsing it into the
/// desired type.
///
/// # Arguments
///
/// * `fits_fptr` - A reference to the `FITSFile` object.
///
/// * `hdu` - A reference to the HDU you want to find `keyword` in the header of.
///
/// * `keyword` - String containing the keyword to read.
///
///
/// # Returns
///
/// *  A Result containing an Option containing the value read or None if the key did not exist, or an error.
///
/// # Examples
///
/// ```
/// # use mwalib::*;
/// # fn main() -> Result<(), FitsError> {
/// let metafits = "test_files/1101503312_1_timestep/1101503312.metafits";
/// let mut fptr = fits_open!(&metafits)?;
/// let hdu = fits_open_hdu!(&mut fptr, 0)?;
/// let freq_centre: Option<f64> = get_optional_fits_key!(&mut fptr, &hdu, "FREQCENT")?;
/// assert_eq!(freq_centre, Some(154.24));
/// let not_real: Option<f64> = get_optional_fits_key!(&mut fptr, &hdu, "NOTREAL")?;
/// assert_eq!(not_real, None);
/// #     Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! get_optional_fits_key {
    ($fptr:expr, $hdu:expr, $keyword:expr) => {
        _get_optional_fits_key($fptr, $hdu, $keyword, file!(), line!())
    };
}

/// Given a FITS file pointer, a HDU that belongs to it, and a keyword, pull out
/// the value of the keyword, parsing it into the desired type.
///
/// # Arguments
///
/// * `fits_fptr` - A reference to the `FITSFile` object.
///
/// * `hdu` - A reference to the HDU you want to find `keyword` in the header of.
///
/// * `keyword` - String containing the keyword to read.
///
///
/// # Returns
///
/// *  A Result containing the value read or an error.
///
/// # Examples
///
/// ```
/// # use mwalib::*;
/// # fn main() -> Result<(), FitsError> {
/// let metafits = "test_files/1101503312_1_timestep/1101503312.metafits";
/// let mut fptr = fits_open!(&metafits)?;
/// let hdu = fits_open_hdu!(&mut fptr, 0)?;
/// let freq_centre: f64 = get_required_fits_key!(&mut fptr, &hdu, "FREQCENT")?;
/// assert_eq!(freq_centre, 154.24);
/// #     Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! get_required_fits_key {
    ($fptr:expr, $hdu:expr, $keyword:expr) => {
        _get_required_fits_key($fptr, $hdu, $keyword, file!(), line!())
    };
}

/// Get a column from a fits file's HDU.
///
/// # Examples
///
/// ```
/// # use mwalib::*;
/// # fn main() -> Result<(), FitsError> {
/// let metafits = "test_files/1101503312_1_timestep/1101503312.metafits";
/// let mut fptr = fits_open!(&metafits)?;
/// let hdu = fits_open_hdu!(&mut fptr, 1)?;
/// let east: Vec<f32> = get_fits_col!(&mut fptr, &hdu, "East")?;
/// assert_eq!(east[0], -585.675);
/// #     Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! get_fits_col {
    ($fptr:expr, $hdu:expr, $keyword:expr) => {
        _get_fits_col($fptr, $hdu, $keyword, file!(), line!())
    };
}

/// Get the size of the image on the supplied FITS file pointer and HDU.
///
/// # Arguments
///
/// * `fits_fptr` - A reference to the `FITSFile` object.
///
/// * `hdu` - A reference to the HDU you want to find the image dimensions of.
///
///
/// # Returns
///
/// *  A Result containing a vector of the size of each dimension, if Ok.
///
#[macro_export]
macro_rules! get_hdu_image_size {
    ($fptr:expr, $hdu:expr) => {
        _get_hdu_image_size($fptr, $hdu, file!(), line!())
    };
}

/// Given a FITS file pointer, and a keyword to a long string keyword that may
/// or may not exist, pull out the long string of the keyword. This deals with
/// FITSs CONTINUE mechanism by calling a low level fits function.
///
/// # Arguments
///
/// * `fits_fptr` - A reference to the `FITSFile` object.
///
/// * `hdu` - A reference to the HDU you want to find `keyword` in the header
/// of.
///
/// * `keyword` - String containing the keyword to read.
///
///
/// # Returns
///
/// * A Result containing an Option containing the value read or None if the key
/// did not exist, or an error.
///
#[macro_export]
macro_rules! get_optional_fits_key_long_string {
    ($fptr:expr, $hdu:expr, $keyword:expr) => {
        _get_optional_fits_key_long_string($fptr, $hdu, $keyword, file!(), line!())
    };
}

/// Given a FITS file pointer, and a keyword to a long string keyword, pull out
/// the long string of the keyword. This deals with FITSs CONTINUE mechanism by
/// calling a low level fits function.
///
/// # Arguments
///
/// * `fits_fptr` - A reference to the `FITSFile` object.
///
/// * `hdu` - A reference to the HDU you want to find `keyword` in the header
/// of.
///
/// * `keyword` - String containing the keyword to read.
///
///
/// # Returns
///
/// * A Result containing the value read or an error.
///
#[macro_export]
macro_rules! get_required_fits_key_long_string {
    ($fptr:expr, $hdu:expr, $keyword:expr) => {
        _get_required_fits_key_long_string($fptr, $hdu, $keyword, file!(), line!())
    };
}

/// Given a FITS file pointer and a HDU, read the associated image.
///
/// # Arguments
///
/// * `fits_fptr` - A reference to the `FITSFile` object.
///
/// * `hdu` - A reference to the HDU you want to find `keyword` in the header
/// of.
///
///
/// # Returns
///
/// * A Result containing the vector of data or an error.
///
#[macro_export]
macro_rules! get_fits_image {
    ($fptr:expr, $hdu:expr) => {
        _get_fits_image($fptr, $hdu, file!(), line!())
    };
}

/// Open a fits file.
///
/// To only be used internally; use the `fits_open!` macro instead.
#[doc(hidden)]
pub fn _open_fits<T: AsRef<std::path::Path>>(
    file: &T,
    source_file: &str,
    source_line: u32,
) -> Result<FitsFile, FitsError> {
    match FitsFile::open(file) {
        Ok(f) => Ok(f),
        Err(e) => Err(FitsError::Open {
            fits_error: e,
            fits_filename: file.as_ref().to_str().unwrap().to_string(),
            source_file: source_file.to_string(),
            source_line,
        }),
    }
}

/// Open a fits file's HDU.
///
/// To only be used internally; use the `fits_open_hdu!` macro instead.
#[doc(hidden)]
pub fn _open_hdu(
    fits_fptr: &mut FitsFile,
    hdu_num: usize,
    source_file: &str,
    source_line: u32,
) -> Result<FitsHdu, FitsError> {
    match fits_fptr.hdu(hdu_num) {
        Ok(f) => Ok(f),
        Err(e) => Err(FitsError::Fitsio {
            fits_error: e,
            fits_filename: fits_fptr.filename.clone(),
            hdu_num: hdu_num + 1,
            source_file: source_file.to_string(),
            source_line,
        }),
    }
}

/// Get an optional key from a fits file's HDU.
///
/// To only be used internally; use the `get_optional_fits_key!` macro instead.
// Benchmarks show that, for pulling out an i64, this function is *slightly*
// slower than just using the hdu with `read_key::<i64>` (by ~100ns on my Ryzen
// 9 3900X). But, even for small FITS values (e.g. BITPIX = -32), using an i32
// gives the wrong result (consistent with cfitsio in C), forcing the use of
// i64 for even these kinds of values. Thus, this function is nice because is
// lets rust parse the string it extracts.
//
#[doc(hidden)]
pub fn _get_optional_fits_key<T: std::str::FromStr>(
    fits_fptr: &mut FitsFile,
    hdu: &FitsHdu,
    keyword: &str,
    source_file: &str,
    source_line: u32,
) -> Result<Option<T>, FitsError> {
    let unparsed_value: String = match hdu.read_key(fits_fptr, keyword) {
        Ok(key_value) => key_value,
        Err(e) => match &e {
            fitsio::errors::Error::Fits(fe) => match fe.status {
                202 => return Ok(None),
                _ => {
                    return Err(FitsError::Fitsio {
                        fits_error: e,
                        fits_filename: fits_fptr.filename.clone(),
                        hdu_num: hdu.number + 1,
                        source_file: source_file.to_string(),
                        source_line,
                    })
                }
            },
            _ => {
                return Err(FitsError::Fitsio {
                    fits_error: e,
                    fits_filename: fits_fptr.filename.clone(),
                    hdu_num: hdu.number + 1,
                    source_file: source_file.to_string(),
                    source_line,
                })
            }
        },
    };

    match unparsed_value.parse() {
        Ok(parsed_value) => Ok(Some(parsed_value)),
        Err(_) => Err(FitsError::Parse {
            key: keyword.to_string(),
            fits_filename: fits_fptr.filename.to_string(),
            hdu_num: hdu.number + 1,
            source_file: source_file.to_string(),
            source_line,
        }),
    }
}

/// Get a required key from a fits file's HDU.
///
/// To only be used internally; use the `get_required_fits_key!` macro instead.
#[doc(hidden)]
pub fn _get_required_fits_key<T: std::str::FromStr>(
    fits_fptr: &mut FitsFile,
    hdu: &FitsHdu,
    keyword: &str,
    source_file: &str,
    source_line: u32,
) -> Result<T, FitsError> {
    match _get_optional_fits_key(fits_fptr, hdu, keyword, source_file, source_line) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(FitsError::MissingKey {
            key: keyword.to_string(),
            fits_filename: fits_fptr.filename.to_string(),
            hdu_num: hdu.number + 1,
            source_file: source_file.to_string(),
            source_line,
        }),
        Err(error) => Err(error),
    }
}

/// Get a column from a fits file's HDU.
///
/// To only be used internally; use the `fits_get_col!` macro instead.
#[doc(hidden)]
pub fn _get_fits_col<T: fitsio::tables::ReadsCol>(
    fits_fptr: &mut FitsFile,
    hdu: &FitsHdu,
    keyword: &str,
    source_file: &str,
    source_line: u32,
) -> Result<Vec<T>, FitsError> {
    match hdu.read_col(fits_fptr, keyword) {
        Ok(c) => Ok(c),
        Err(fits_error) => Err(FitsError::Fitsio {
            fits_error,
            fits_filename: fits_fptr.filename.clone(),
            hdu_num: hdu.number + 1,
            source_file: source_file.to_string(),
            source_line,
        }),
    }
}

/// Get an optional long string out of a FITS file.
///
/// The HDU is actually not needed in this function. We supply it to this
/// function *only* because it forces the caller to open the intended HDU first,
/// so that when cfitsio is called directly, it tries to get the string from the
/// right place.
///
/// To only be used internally; use the `get_optional_fits_key_long_string!`
/// macro instead.
#[doc(hidden)]
pub fn _get_optional_fits_key_long_string(
    fits_fptr: &mut fitsio::FitsFile,
    hdu: &FitsHdu,
    keyword: &str,
    source_file: &str,
    source_line: u32,
) -> Result<Option<String>, FitsError> {
    // Read the long string.
    let (status, long_string) = unsafe { get_fits_long_string(fits_fptr.as_raw(), keyword) };
    match status {
        0 => Ok(Some(long_string)),
        202 => Ok(None),
        _ => Err(FitsError::LongString {
            key: keyword.to_string(),
            fits_filename: fits_fptr.filename.clone(),
            hdu_num: hdu.number + 1,
            source_file: source_file.to_string(),
            source_line,
        }),
    }
}

/// Get a required long string out of a FITS file.
///
/// The HDU is actually not needed in this function. We supply it to this
/// function *only* because it forces the caller to open the intended HDU first,
/// so that when cfitsio is called directly, it tries to get the string from the
/// right place.
///
/// To only be used internally; use the `get_required_fits_key_long_string!`
/// macro instead.
#[doc(hidden)]
pub fn _get_required_fits_key_long_string(
    fits_fptr: &mut FitsFile,
    hdu: &FitsHdu,
    keyword: &str,
    source_file: &str,
    source_line: u32,
) -> Result<String, FitsError> {
    match _get_optional_fits_key_long_string(fits_fptr, hdu, keyword, source_file, source_line) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(FitsError::MissingKey {
            key: keyword.to_string(),
            fits_filename: fits_fptr.filename.clone(),
            hdu_num: hdu.number + 1,
            source_file: source_file.to_string(),
            source_line,
        }),
        Err(error) => Err(error),
    }
}

/// Get the size of an image from the supplied HDU.
///
/// To only be used internally; use the `get_hdu_image_size!` macro instead.
#[doc(hidden)]
pub fn _get_hdu_image_size(
    fits_fptr: &mut FitsFile,
    hdu: &FitsHdu,
    source_file: &str,
    source_line: u32,
) -> Result<Vec<usize>, FitsError> {
    match &hdu.info {
        HduInfo::ImageInfo { shape, .. } => Ok(shape.clone()),
        _ => Err(FitsError::NotImage {
            fits_filename: fits_fptr.filename.clone(),
            hdu_num: hdu.number + 1,
            source_file: source_file.to_string(),
            source_line,
        }),
    }
}

/// Get the data out of a HDU's image.
///
/// To only be used internally; use the `get_fits_image!` macro instead.
#[doc(hidden)]
pub fn _get_fits_image<T: fitsio::images::ReadImage>(
    fits_fptr: &mut FitsFile,
    hdu: &FitsHdu,
    source_file: &str,
    source_line: u32,
) -> Result<T, FitsError> {
    match &hdu.info {
        HduInfo::ImageInfo { .. } => match hdu.read_image(fits_fptr) {
            Ok(img) => Ok(img),
            Err(e) => Err(FitsError::Fitsio {
                fits_error: e,
                fits_filename: fits_fptr.filename.clone(),
                hdu_num: hdu.number + 1,
                source_file: source_file.to_string(),
                source_line,
            }),
        },
        _ => Err(FitsError::NotImage {
            fits_filename: fits_fptr.filename.clone(),
            hdu_num: hdu.number + 1,
            source_file: source_file.to_string(),
            source_line,
        }),
    }
}

/// Get a long string from a FITS file. The supplied FITS file pointer *must* be
/// using the appropriate HDU already, or this function will fail.
///
/// This function exists because the rust library `fitsio` does not support
/// reading in long strings (i.e. those that have CONTINUE statements).
///
///
/// # Arguments
///
/// * `fits_fptr` - A reference to the `FITSFile` object.
///
/// * `keyword` - String containing the keyword to read.
///
///
/// # Returns
///
/// *  A FITS status code and the long string
///
///
/// # Safety
///
/// This function is no less safe than calling cfitsio itself.
///
unsafe fn get_fits_long_string(fptr: *mut fitsfile, keyword: &str) -> (i32, String) {
    let keyword_ffi =
        CString::new(keyword).expect("get_fits_long_string: CString::new() failed for keyword");
    // For reasons I cannot fathom, ffgkls expects `value` to be a malloc'd
    // char** in C, but will only use a single char* inside it, and that doesn't
    // need to be allocated. Anyway, Vec<*mut c_char> works for me in rust.
    let mut value: [*mut c_char; 1] = [ptr::null_mut()];
    let mut status = 0;
    ffgkls(
        fptr,
        keyword_ffi.as_ptr(),
        value.as_mut_ptr(),
        ptr::null_mut(),
        &mut status,
    );

    // Check the call worked!
    if status == 0 {
        let long_string = CString::from_raw(value[0])
            .into_string()
            .expect("get_fits_long_string: converting string_ptr failed");
        (status, long_string)
    } else {
        let long_string = String::from("");
        (status, long_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::misc::*;
    use fitsio::images::{ImageDescription, ImageType};
    use fitsio::tables::{ColumnDataType, ColumnDescription};
    use fitsio_sys::ffpkls;

    #[test]
    fn test_get_hdu_image_size_image() {
        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        with_new_temp_fits_file("test_fits_read_key.fits", |fptr| {
            // Ensure we have 1 hdu
            fits_open_hdu!(fptr, 0).expect("Couldn't open HDU 0");

            let image_description = ImageDescription {
                data_type: ImageType::Float,
                dimensions: &[101, 102],
            };

            // Create a new image HDU
            fptr.create_image("EXTNAME".to_string(), &image_description)
                .unwrap();
            let hdu = fits_open_hdu!(fptr, 1).expect("Couldn't open HDU 1");

            // Run our test
            let size_vec = get_hdu_image_size!(fptr, &hdu).unwrap();

            assert_eq!(size_vec.len(), 2);
            assert_eq!(size_vec[0], 101);
            assert_eq!(size_vec[1], 102);
            assert_ne!(size_vec[0], 200);
            assert_ne!(size_vec[1], 200);
        });
    }

    #[test]
    fn test_get_hdu_image_size_non_image() {
        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        with_new_temp_fits_file("test_fits_read_key.fits", |fptr| {
            // Ensure we have 1 hdu
            fits_open_hdu!(fptr, 0).expect("Couldn't open HDU 0");

            let first_description = ColumnDescription::new("A")
                .with_type(ColumnDataType::Int)
                .create()
                .unwrap();
            let second_description = ColumnDescription::new("B")
                .with_type(ColumnDataType::Long)
                .create()
                .unwrap();
            let descriptions = [first_description, second_description];

            fptr.create_table("EXTNAME".to_string(), &descriptions)
                .unwrap();
            let hdu = fits_open_hdu!(fptr, 1).expect("Couldn't open HDU 1");

            // Run our test
            get_hdu_image_size!(fptr, &hdu).unwrap_err();
        });
    }

    #[test]
    fn test_get_required_fits_key() {
        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        with_new_temp_fits_file("test_fits_read_key.fits", |fptr| {
            let hdu = fits_open_hdu!(fptr, 0).expect("Couldn't open HDU 0");

            // Failure to get a key that doesn't exist.
            let doesnt_exist: Result<u8, _> = get_required_fits_key!(fptr, &hdu, "foo");
            assert!(doesnt_exist.is_err());

            // Key types must be i64 to get any sort of sanity.
            hdu.write_key(fptr, "foo", 10i64)
                .expect("Couldn't write key 'foo'");
            hdu.write_key(fptr, "bar", -5i64)
                .expect("Couldn't write key 'bar'");

            // Verify that using the normal `fitsio` gives the wrong result, unless
            // the type is an i64.
            let foo_i32 = hdu.read_key::<i32>(fptr, "foo");
            let foo_i64 = hdu.read_key::<i64>(fptr, "foo");
            assert!(foo_i32.is_ok());
            assert!(foo_i64.is_ok());
            assert_eq!(foo_i32.unwrap(), 1);
            assert_eq!(foo_i64.unwrap(), 10);

            // Despite writing to "foo", the key is written as "FOO".
            let foo_i64 = hdu.read_key::<i64>(fptr, "FOO");
            assert!(foo_i64.is_ok());
            assert_eq!(foo_i64.unwrap(), 10);

            let foo_u8: Result<u8, _> = get_required_fits_key!(fptr, &hdu, "foo");
            let foo_i8: Result<i8, _> = get_required_fits_key!(fptr, &hdu, "foo");
            assert!(foo_u8.is_ok());
            assert!(foo_i8.is_ok());
            assert_eq!(foo_u8.unwrap(), 10);
            assert_eq!(foo_i8.unwrap(), 10);

            // Can't parse the negative number into a unsigned int.
            let bar_u8: Result<u8, _> = get_required_fits_key!(fptr, &hdu, "bar");
            let bar_i8: Result<i8, _> = get_required_fits_key!(fptr, &hdu, "bar");
            assert!(bar_u8.is_err());
            assert!(bar_i8.is_ok());
            assert_eq!(bar_i8.unwrap(), -5);
        });
    }

    #[test]
    fn test_get_optional_fits_key() {
        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        with_new_temp_fits_file("test_fits_read_key.fits", |fptr| {
            let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

            // Failure to get a key that doesn't exist is OK if we're using the optional variant.
            let foo: Result<Option<u8>, _> = get_optional_fits_key!(fptr, &hdu, "foo");
            assert!(foo.is_ok());
            assert!(foo.unwrap().is_none());

            // Key types must be i64 to get any sort of sanity.
            hdu.write_key(fptr, "foo", 10i64)
                .expect("Couldn't write key 'foo'");
            hdu.write_key(fptr, "bar", -5i64)
                .expect("Couldn't write key 'bar'");

            // Verify that using the normal `fitsio` gives the wrong result, unless
            // the type is an i64.
            let foo_i32 = hdu.read_key::<i32>(fptr, "foo");
            let foo_i64 = hdu.read_key::<i64>(fptr, "foo");
            assert!(foo_i32.is_ok());
            assert!(foo_i64.is_ok());
            assert_eq!(foo_i32.unwrap(), 1);
            assert_eq!(foo_i64.unwrap(), 10);

            // Despite writing to "foo", the key is written as "FOO".
            let foo_i64 = hdu.read_key::<i64>(fptr, "FOO");
            assert!(foo_i64.is_ok());
            assert_eq!(foo_i64.unwrap(), 10);

            let foo_u8: Result<Option<u8>, _> = get_optional_fits_key!(fptr, &hdu, "foo");
            let foo_i8: Result<Option<i8>, _> = get_optional_fits_key!(fptr, &hdu, "foo");
            assert!(foo_u8.is_ok());
            assert!(foo_i8.is_ok());
            assert_eq!(foo_u8.unwrap(), Some(10));
            assert_eq!(foo_i8.unwrap(), Some(10));

            // Can't parse the negative number into a unsigned int.
            let bar_u8: Result<u8, _> = get_required_fits_key!(fptr, &hdu, "bar");
            let bar_i8: Result<i8, _> = get_required_fits_key!(fptr, &hdu, "bar");
            assert!(bar_u8.is_err());
            assert!(bar_i8.is_ok());
            assert_eq!(bar_i8.unwrap(), -5);
        });
    }

    #[test]
    fn test_get_required_fits_key_string() {
        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        with_new_temp_fits_file("test_fits_read_key_string.fits", |fptr| {
            let hdu = fits_open_hdu!(fptr, 0).expect("Couldn't open HDU 0");

            // Failure to get a key that doesn't exist.
            let does_not_exist: Result<String, FitsError> =
                get_required_fits_key!(fptr, &hdu, "foo");
            assert!(does_not_exist.is_err());

            // Add a test string
            hdu.write_key(fptr, "foo", "hello")
                .expect("Couldn't write key 'foo'");

            // Read foo back in
            let foo_string: String = get_required_fits_key!(fptr, &hdu, "foo").unwrap();

            // Despite writing to "foo", the key is written as "FOO".
            assert_eq!(foo_string, "hello");
        });
    }

    #[test]
    fn test_get_optional_fits_key_string() {
        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        with_new_temp_fits_file("test_fits_read_key_string.fits", |fptr| {
            let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

            // No Failure to get a key that doesn't exist.
            let does_not_exist: Result<Option<String>, FitsError> =
                get_optional_fits_key!(fptr, &hdu, "foo");

            assert!(does_not_exist.is_ok());
            assert!(does_not_exist.unwrap().is_none());

            // Add a test string
            hdu.write_key(fptr, "foo", "hello")
                .expect("Couldn't write key 'foo'");

            // Read foo back in
            let foo_string: Result<Option<String>, FitsError> =
                get_optional_fits_key!(fptr, &hdu, "foo");

            // Despite writing to "foo", the key is written as "FOO".
            assert!(foo_string.is_ok());
            assert_eq!(foo_string.unwrap().unwrap(), "hello");
        });
    }

    #[test]
    fn test_get_fits_long_string() {
        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        with_new_temp_fits_file("test_get_fits_long_string.fits", |fptr| {
            let complete_string = "131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147,148,149,150,151,152,153,154";
            let first_string =
                "131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147&";

            // Sadly, rust's `fitsio` library doesn't support writing long strings
            // with CONTINUE statements. We have to do it for ourselves.
            unsafe {
                let fptr_ffi = fptr.as_raw();
                let keyword_ffi = CString::new("foo")
                    .expect("get_fits_long_string: CString::new() failed for 'foo'");
                let value_ffi = CString::new(complete_string)
                    .expect("get_fits_long_string: CString::new() failed for 'complete_string'");
                let mut status = 0;

                ffpkls(
                    fptr_ffi,
                    keyword_ffi.as_ptr(),
                    value_ffi.as_ptr(),
                    ptr::null_mut(),
                    &mut status,
                );
            }

            let (status, foo_str) = unsafe { get_fits_long_string(fptr.as_raw(), "foo") };
            assert_eq!(status, 0);
            assert_eq!(foo_str, complete_string);

            // Try out the `fitsio` read key.
            let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");
            let fitsio_str = hdu.read_key::<String>(fptr, "foo");
            assert!(fitsio_str.is_ok());
            assert_eq!(fitsio_str.unwrap(), first_string);

            // A repeated read just returns the first string again.
            let fitsio_str = hdu.read_key::<String>(fptr, "foo");
            assert!(fitsio_str.is_ok());
            assert_eq!(fitsio_str.unwrap(), first_string);
        });
    }

    #[test]
    fn test_get_fits_long_string_failure() {
        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        with_new_temp_fits_file("test_get_fits_long_string_failure.fits", |fptr| {
            let complete_string = "131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147,148,149,150,151,152,153,154";

            // Sadly, rust's `fitsio` library doesn't support writing long strings
            // with CONTINUE statements. We have to do it for ourselves.
            unsafe {
                let fptr_ffi = fptr.as_raw();
                let keyword_ffi = CString::new("foo")
                    .expect("get_fits_long_string: CString::new() failed for 'foo'");
                let value_ffi = CString::new(complete_string)
                    .expect("get_fits_long_string: CString::new() failed for 'complete_string'");
                let mut status = 0;

                ffpkls(
                    fptr_ffi,
                    keyword_ffi.as_ptr(),
                    value_ffi.as_ptr(),
                    ptr::null_mut(),
                    &mut status,
                );
            }

            let (status, _) = unsafe { get_fits_long_string(fptr.as_raw(), "NOTfoo") };
            assert_ne!(status, 0);
        });
    }

    #[test]
    fn test_file_doesnt_exist() {
        let metafits = "im_not_real.metafits";
        let fptr = fits_open!(&metafits);
        assert!(fptr.is_err());
    }

    #[test]
    fn test_1101503312_metafits() -> Result<(), FitsError> {
        let metafits = "test_files/1101503312_1_timestep/1101503312.metafits";
        let mut fptr = fits_open!(&metafits)?;
        let hdu = fits_open_hdu!(&mut fptr, 0)?;
        let freq_centre: f64 = get_required_fits_key!(&mut fptr, &hdu, "FREQCENT")?;
        assert_eq!(freq_centre, 154.24);
        let fine_chan_width: u8 = get_required_fits_key!(&mut fptr, &hdu, "FINECHAN")?;
        assert_eq!(fine_chan_width, 10);

        let doesnt_exist: Result<f64, FitsError> = get_required_fits_key!(&mut fptr, &hdu, "FINE");
        assert!(doesnt_exist.is_err());

        let cant_parse: Result<u64, FitsError> =
            get_required_fits_key!(&mut fptr, &hdu, "FREQCENT");
        assert!(cant_parse.is_err());

        let hdu_too_big = fits_open_hdu!(&mut fptr, 1000);
        assert!(hdu_too_big.is_err());

        let hdu = fits_open_hdu!(&mut fptr, 1)?;
        let east: Vec<f32> = get_fits_col!(&mut fptr, &hdu, "East")?;
        assert_eq!(east[0], -585.675);
        let doesnt_exist: Result<Vec<f32>, FitsError> = get_fits_col!(&mut fptr, &hdu, "South");
        assert!(doesnt_exist.is_err());
        Ok(())
    }

    #[test]
    fn test_1244973688_metafits() -> Result<(), FitsError> {
        let metafits = "test_files/1244973688_1_timestep/1244973688.metafits";
        let mut fptr = fits_open!(&metafits)?;
        let hdu = fits_open_hdu!(&mut fptr, 0)?;
        let freq_centre: f64 = get_required_fits_key!(&mut fptr, &hdu, "FREQCENT")?;
        assert_eq!(freq_centre, 147.84);
        let fine_chan_width: u8 = get_required_fits_key!(&mut fptr, &hdu, "FINECHAN")?;
        assert_eq!(fine_chan_width, 10);

        let doesnt_exist: Result<f64, FitsError> = get_required_fits_key!(&mut fptr, &hdu, "FINE");
        assert!(doesnt_exist.is_err());

        let cant_parse: Result<u64, FitsError> =
            get_required_fits_key!(&mut fptr, &hdu, "FREQCENT");
        assert!(cant_parse.is_err());

        let hdu_too_big = fits_open_hdu!(&mut fptr, 1000);
        assert!(hdu_too_big.is_err());

        let hdu = fits_open_hdu!(&mut fptr, 1)?;
        let east: Vec<f32> = get_fits_col!(&mut fptr, &hdu, "East")?;
        assert_eq!(east[0], -585.675);
        let doesnt_exist: Result<Vec<f32>, FitsError> = get_fits_col!(&mut fptr, &hdu, "South");
        assert!(doesnt_exist.is_err());
        Ok(())
    }
}
