// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Helper functions for reading FITS files.
 */
pub mod error;
pub use error::FitsError;

use fitsio::{hdu::*, FitsFile};
use fitsio_sys::{ffgkls, fitsfile};
use libc::c_char;
use log::trace;
use std::ffi::*;
use std::ptr;

#[cfg(test)]
mod test;

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

/// Given a FITS file pointer and a HDU, read the associated float image.
///
/// # Arguments
///
/// * `fits_fptr` - A reference to the `FITSFile` object.
///
/// * `hdu` - A reference to the HDU you want to find `keyword` in the header
/// of.
///
/// * `buffer` - Buffer of floats (as a slice) to fill with data from the HDU.
///
///
/// # Returns
///
/// * A Result of Ok on success, Err on error.
///
#[macro_export]
macro_rules! get_fits_float_image_into_buffer {
    ($fptr:expr, $hdu:expr, $buffer:expr) => {
        _get_fits_float_img_into_buf($fptr, $hdu, $buffer, file!(), line!())
    };
}

/// Open a fits file.
///
/// To only be used internally; use the `fits_open!` macro instead.
#[doc(hidden)]
pub fn _open_fits<T: AsRef<std::path::Path>>(
    file: &T,
    source_file: &'static str,
    source_line: u32,
) -> Result<FitsFile, FitsError> {
    match FitsFile::open(file) {
        Ok(f) => {
            trace!(
                "_open_fits() filename: '{}'",
                file.as_ref().to_str().unwrap().to_string()
            );
            Ok(f)
        }
        Err(e) => Err(FitsError::Open {
            fits_error: e,
            fits_filename: file.as_ref().to_str().unwrap().to_string(),
            source_file,
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
    source_file: &'static str,
    source_line: u32,
) -> Result<FitsHdu, FitsError> {
    match fits_fptr.hdu(hdu_num) {
        Ok(f) => {
            trace!(
                "_open_hdu() filename: '{}' hdu: {}",
                fits_fptr.filename,
                hdu_num
            );
            Ok(f)
        }
        Err(e) => Err(FitsError::Fitsio {
            fits_error: e,
            fits_filename: fits_fptr.filename.clone(),
            hdu_num: hdu_num + 1,
            source_file,
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
    source_file: &'static str,
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
                        source_file,
                        source_line,
                    })
                }
            },
            _ => {
                return Err(FitsError::Fitsio {
                    fits_error: e,
                    fits_filename: fits_fptr.filename.clone(),
                    hdu_num: hdu.number + 1,
                    source_file,
                    source_line,
                })
            }
        },
    };

    trace!(
        "_get_optional_fits_key() filename: '{}' hdu: {} keyword: '{}' value: '{}'",
        &fits_fptr.filename,
        hdu.number,
        String::from(keyword),
        unparsed_value
    );

    match unparsed_value.parse() {
        Ok(parsed_value) => Ok(Some(parsed_value)),
        Err(_) => Err(FitsError::Parse {
            key: keyword.to_string(),
            fits_filename: fits_fptr.filename.to_string(),
            hdu_num: hdu.number + 1,
            source_file,
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
    source_file: &'static str,
    source_line: u32,
) -> Result<T, FitsError> {
    match _get_optional_fits_key(fits_fptr, hdu, keyword, source_file, source_line) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(FitsError::MissingKey {
            key: keyword.to_string(),
            fits_filename: fits_fptr.filename.to_string(),
            hdu_num: hdu.number + 1,
            source_file,
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
    source_file: &'static str,
    source_line: u32,
) -> Result<Vec<T>, FitsError> {
    match hdu.read_col(fits_fptr, keyword) {
        Ok(c) => {
            trace!(
                "_get_fits_col() filename: '{}' hdu: {} keyword: '{}' values: {}",
                fits_fptr.filename,
                hdu.number,
                keyword,
                c.len()
            );
            Ok(c)
        }
        Err(fits_error) => Err(FitsError::Fitsio {
            fits_error,
            fits_filename: fits_fptr.filename.clone(),
            hdu_num: hdu.number + 1,
            source_file,
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
    source_file: &'static str,
    source_line: u32,
) -> Result<Option<String>, FitsError> {
    // Read the long string.
    let (status, long_string) = unsafe { get_fits_long_string(fits_fptr.as_raw(), keyword) };

    match status {
        0 => {
            trace!(
                "_get_optional_fits_key_long_string() filename: {} keyword: '{}' value: '{}'",
                &fits_fptr.filename,
                keyword,
                long_string
            );
            Ok(Some(long_string))
        }
        202 => Ok(None),
        _ => Err(FitsError::LongString {
            key: keyword.to_string(),
            fits_filename: fits_fptr.filename.clone(),
            hdu_num: hdu.number + 1,
            source_file,
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
    source_file: &'static str,
    source_line: u32,
) -> Result<String, FitsError> {
    match _get_optional_fits_key_long_string(fits_fptr, hdu, keyword, source_file, source_line) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(FitsError::MissingKey {
            key: keyword.to_string(),
            fits_filename: fits_fptr.filename.clone(),
            hdu_num: hdu.number + 1,
            source_file,
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
    source_file: &'static str,
    source_line: u32,
) -> Result<Vec<usize>, FitsError> {
    match &hdu.info {
        HduInfo::ImageInfo { shape, .. } => {
            trace!(
                "_get_hdu_image_size() filename: '{}' hdu: {} shape: {:?}",
                fits_fptr.filename,
                hdu.number,
                shape
            );
            Ok(shape.clone())
        }
        _ => Err(FitsError::NotImage {
            fits_filename: fits_fptr.filename.clone(),
            hdu_num: hdu.number + 1,
            source_file,
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
    source_file: &'static str,
    source_line: u32,
) -> Result<T, FitsError> {
    match &hdu.info {
        HduInfo::ImageInfo { .. } => match hdu.read_image(fits_fptr) {
            Ok(img) => {
                trace!(
                    "_get_fits_image() filename: '{}' hdu: {}",
                    fits_fptr.filename,
                    hdu.number
                );
                Ok(img)
            }
            Err(e) => Err(FitsError::Fitsio {
                fits_error: e,
                fits_filename: fits_fptr.filename.clone(),
                hdu_num: hdu.number + 1,
                source_file,
                source_line,
            }),
        },
        _ => Err(FitsError::NotImage {
            fits_filename: fits_fptr.filename.clone(),
            hdu_num: hdu.number + 1,
            source_file,
            source_line,
        }),
    }
}

/// Direct read of FITS HDU
#[doc(hidden)]
pub fn _get_fits_float_img_into_buf(
    fits_fptr: &mut FitsFile,
    hdu: &FitsHdu,
    buffer: &mut [f32],
    source_file: &'static str,
    source_line: u32,
) -> Result<(), FitsError> {
    unsafe {
        // Get raw ptr and length to user supplied buffer
        let buffer_len = buffer.len() as i64;
        let buffer_ptr = buffer.as_mut_ptr();

        // Call the underlying cfitsio read function for floats
        let mut status = 0;
        fitsio_sys::ffgpv(
            fits_fptr.as_raw(),
            fitsio_sys::TFLOAT as _,
            1,
            buffer_len,
            ptr::null_mut(),
            buffer_ptr as *mut _,
            ptr::null_mut(),
            &mut status,
        );

        // Check fits call status
        match fitsio::errors::check_status(status) {
            Ok(_) => {}
            Err(e) => {
                return Err(FitsError::Fitsio {
                    fits_error: e,
                    fits_filename: fits_fptr.filename.clone(),
                    hdu_num: hdu.number + 1,
                    source_file,
                    source_line,
                });
            }
        }
    }

    trace!(
        "_get_fits_float_img_into_buf() filename: '{}' hdu: {}",
        fits_fptr.filename,
        hdu.number
    );

    Ok(())
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
    let long_string = match status {
        0 => CString::from_raw(value[0])
            .into_string()
            .expect("get_fits_long_string: converting string_ptr failed"),
        _ => String::from(""),
    };

    (status, long_string)
}
