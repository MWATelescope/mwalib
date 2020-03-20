// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Helper functions for reading FITS files.
 */

use std::ffi::*;
use std::ptr;
use std::str;

use fitsio::{hdu::*, FitsFile};
use fitsio_sys::{ffgkls, fitsfile};
use libc::c_char;

use crate::error::ErrorKind;

/// Given a FITS file pointer, a HDU that belongs to it, and a keyword, pull out
/// the value of the keyword, parsing it into the desired type.
///
/// Benchmarks show that, for pulling out an i64, this function is *slightly*
/// slower than just using the hdu with `read_key::<i64>` (by ~100ns on my Ryzen
/// 9 3900X). But, even for small FITS values (e.g. BITPIX = -32), using an i32
/// gives the wrong result (consistent with cfitsio in C), forcing the use of
/// i64 for even these kinds of values. Thus, this function is nice because is
/// lets rust parse the string it extracts.
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
pub fn get_optional_fits_key<T>(
    fits_fptr: &mut FitsFile,
    hdu: &FitsHdu,
    keyword: &str,
) -> Result<Option<T>, ErrorKind>
where
    T: std::str::FromStr,
{
    let unparsed_value: String = match hdu.read_key(fits_fptr, keyword) {
        Ok(key_value) => key_value,
        Err(e) => match e {
            fitsio::errors::Error::Fits(fe) => match fe.status {
                202 => return Ok(None),
                _ => {
                    return Err(ErrorKind::Custom(format!(
                        "FITS error reading key {} code: {} message: {}",
                        keyword, fe.status, fe.message
                    )))
                }
            },
            _ => return Err(ErrorKind::from(e)),
        },
    };

    match unparsed_value.parse() {
        Ok(parsed_value) => Ok(Some(parsed_value)),
        Err(_) => Err(ErrorKind::Custom(format!(
            "unable to parse fits key {} - invalid data type",
            keyword
        ))),
    }
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
/// *  A Result containing an Option containing the value read or None if the key did not exist, or an error.
///
pub fn get_required_fits_key<T>(
    fits_fptr: &mut FitsFile,
    hdu: &FitsHdu,
    keyword: &str,
) -> Result<T, ErrorKind>
where
    T: std::str::FromStr,
{
    match get_optional_fits_key(fits_fptr, hdu, keyword) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(ErrorKind::Custom(format!(
            "FITS keyword {} was not found",
            keyword
        ))),
        Err(error) => Err(error),
    }
}

/// Given a FITS file pointer, and a keyword to a long string keyword, pull out
/// the string of the keyword. This deals with FITSs CONTINUE mechanism by calling a low level fits function.
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
/// *  A Result containing an Option containing the value read or None if the key did not exist, or an error.
///
pub fn get_optional_fits_key_long_string(
    fits_fptr: &mut fitsio::FitsFile,
    keyword: &str,
) -> Result<Option<String>, ErrorKind> {
    // Read the long string.
    let (status, long_string) = unsafe { get_fits_long_string(fits_fptr.as_raw(), keyword) };
    match status {
        0 => Ok(Some(long_string)),
        202 => Ok(None),
        _ => Err(ErrorKind::Custom(format!(
            "get_optional_fits_key_long_string failed reading {}",
            keyword
        ))),
    }
}

/// Given a FITS file pointer, and a keyword to a long string keyword, pull out
/// the string of the keyword. This deals with FITSs CONTINUE mechanism by calling a low level fits function.
///
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
pub fn get_required_fits_key_long_string(
    fits_fptr: &mut FitsFile,
    keyword: &str,
) -> Result<String, ErrorKind> {
    match get_optional_fits_key_long_string(fits_fptr, keyword) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(ErrorKind::Custom(format!(
            "FITS keyword {} was not found",
            keyword
        ))),
        Err(error) => Err(error),
    }
}

/// # Safety
/// Via FFI, get a long string from a FITS file.
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

/// Given a FITS file pointer, get the size of the image on HDU 2.
///
/// # Arguments
///
/// * `fits_fptr` - A reference to the `FITSFile` object.
///
///
/// # Returns
///
/// *  A Result containing a vector of the size of each dimension, if Ok.
///
pub fn get_hdu_image_size(fits_fptr: &mut FitsFile) -> Result<Vec<usize>, ErrorKind> {
    match fits_fptr.hdu(1)?.info {
        HduInfo::ImageInfo { shape, .. } => Ok(shape),
        _ => Err(ErrorKind::Custom(
            "fits_read::get_hdu_image_size: HDU 2 of the first gpubox_fptr was not an image"
                .to_string(),
        )),
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
            fptr.hdu(0).expect("Couldn't open HDU 0");

            let image_description = ImageDescription {
                data_type: ImageType::Float,
                dimensions: &[101, 102],
            };

            // Create a new image HDU
            fptr.create_image("EXTNAME".to_string(), &image_description)
                .unwrap();

            // Run our test
            let size_vec = get_hdu_image_size(fptr).unwrap();

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
            fptr.hdu(0).expect("Couldn't open HDU 0");

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

            // Run our test
            get_hdu_image_size(fptr).unwrap_err();
        });
    }

    #[test]
    fn test_get_fits_key() {
        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        with_new_temp_fits_file("test_fits_read_key.fits", |fptr| {
            let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

            // Failure to get a key that doesn't exist.
            assert!(get_required_fits_key::<u8>(fptr, &hdu, "foo").is_err());

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

            let foo_u8 = get_required_fits_key::<u8>(fptr, &hdu, "foo");
            let foo_i8 = get_required_fits_key::<i8>(fptr, &hdu, "foo");
            assert!(foo_u8.is_ok());
            assert!(foo_i8.is_ok());
            assert_eq!(foo_u8.unwrap(), 10);
            assert_eq!(foo_i8.unwrap(), 10);

            // Can't parse the negative number into a unsigned int.
            let bar_u8 = get_required_fits_key::<u8>(fptr, &hdu, "bar");
            let bar_i8 = get_required_fits_key::<i8>(fptr, &hdu, "bar");
            assert!(bar_u8.is_err());
            assert!(bar_i8.is_ok());
            assert_eq!(bar_i8.unwrap(), -5);
        });
    }

    #[test]
    fn test_get_fits_key_string() {
        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        with_new_temp_fits_file("test_fits_read_key_string.fits", |fptr| {
            let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

            // Failure to get a key that doesn't exist.
            let does_not_exist: Result<String, ErrorKind> =
                get_required_fits_key(fptr, &hdu, "foo");
            assert!(does_not_exist.is_err());

            // Add a test string
            hdu.write_key(fptr, "foo", "hello")
                .expect("Couldn't write key 'foo'");

            // Read foo back in
            let foo_string: String = get_required_fits_key(fptr, &hdu, "foo").unwrap();

            // Despite writing to "foo", the key is written as "FOO".
            assert_eq!(foo_string, "hello");
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
}
