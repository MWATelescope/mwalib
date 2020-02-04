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
pub fn get_fits_key<T>(
    fits_fptr: &mut FitsFile,
    hdu: &FitsHdu,
    keyword: &str,
) -> Result<T, ErrorKind>
where
    T: std::str::FromStr,
    ErrorKind: From<<T as str::FromStr>::Err>,
{
    Ok(hdu.read_key::<String>(fits_fptr, keyword)?.parse()?)
}

/// Given a FITS file pointer, get the size of the image on HDU 2.
// TODO: Write tests.
pub fn get_hdu_image_size(fptr: &mut FitsFile) -> Result<Vec<usize>, ErrorKind> {
    match fptr.hdu(1)?.info {
        HduInfo::ImageInfo { shape, .. } => Ok(shape),
        _ => Err(ErrorKind::Custom(
            "mwalibBuffer::read: HDU 2 of the first gpubox_fptr was not an image".to_string(),
        )),
    }
}

/// Via FFI, get a long string from a FITS file.
///
/// This function exists because the rust library `fitsio` does not support
/// reading in long strings (i.e. those that have CONTINUE statements).
pub unsafe fn get_fits_long_string(fptr: *mut fitsfile, keyword: &str) -> (i32, String) {
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
    let long_string = CString::from_raw(value[0])
        .into_string()
        .expect("get_fits_long_string: converting string_ptr failed");
    (status, long_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fitsio_sys::ffpkls;
    use std::env;
    use std::fs::remove_file;

    #[test]
    fn test_get_fits_key() {
        // Create a (temporary) FITS file with some keys to test our functions.

        // FitsFile::create() expects the filename passed in to not
        // exist. Delete it if it exists.
        let mut file = env::temp_dir();
        file.push("test_fits_read_key.fits");
        if file.exists() {
            remove_file(&file).unwrap();
        }
        let mut fptr = FitsFile::create(&file)
            .open()
            .expect("Couldn't open tempfile");
        let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

        // Failure to get a key that doesn't exist.
        assert!(get_fits_key::<u8>(&mut fptr, &hdu, "foo").is_err());

        // Key types must be i64 to get any sort of sanity.
        hdu.write_key(&mut fptr, "foo", 10i64)
            .expect("Couldn't write key 'foo'");
        hdu.write_key(&mut fptr, "bar", -5i64)
            .expect("Couldn't write key 'bar'");

        // Verify that using the normal `fitsio` gives the wrong result, unless
        // the type is an i64.
        let foo_i32 = hdu.read_key::<i32>(&mut fptr, "foo");
        let foo_i64 = hdu.read_key::<i64>(&mut fptr, "foo");
        assert!(foo_i32.is_ok());
        assert!(foo_i64.is_ok());
        assert_eq!(foo_i32.unwrap(), 1);
        assert_eq!(foo_i64.unwrap(), 10);

        // Despite writing to "foo", the key is written as "FOO".
        let foo_i64 = hdu.read_key::<i64>(&mut fptr, "FOO");
        assert!(foo_i64.is_ok());
        assert_eq!(foo_i64.unwrap(), 10);

        let foo_u8 = get_fits_key::<u8>(&mut fptr, &hdu, "foo");
        let foo_i8 = get_fits_key::<i8>(&mut fptr, &hdu, "foo");
        assert!(foo_u8.is_ok());
        assert!(foo_i8.is_ok());
        assert_eq!(foo_u8.unwrap(), 10);
        assert_eq!(foo_i8.unwrap(), 10);

        // Can't parse the negative number into a unsigned int.
        let bar_u8 = get_fits_key::<u8>(&mut fptr, &hdu, "bar");
        let bar_i8 = get_fits_key::<i8>(&mut fptr, &hdu, "bar");
        assert!(bar_u8.is_err());
        assert!(bar_i8.is_ok());
        assert_eq!(bar_i8.unwrap(), -5);

        remove_file(&file).unwrap();
    }

    #[test]
    fn test_get_fits_long_string() {
        let mut file = env::temp_dir();
        file.push("test_get_fits_long_string.fits");
        if file.exists() {
            remove_file(&file).unwrap();
        }
        let mut fptr = FitsFile::create(&file)
            .open()
            .expect("Couldn't open tempfile");

        let complete_string = "131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147,148,149,150,151,152,153,154";
        let first_string = "131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147&";

        // Sadly, rust's `fitsio` library doesn't support writing long strings
        // with CONTINUE statements. We have to do it for ourselves.
        unsafe {
            let fptr_ffi = fptr.as_raw();
            let keyword_ffi =
                CString::new("foo").expect("get_fits_long_string: CString::new() failed for 'foo'");
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
        let fitsio_str = hdu.read_key::<String>(&mut fptr, "foo");
        assert!(fitsio_str.is_ok());
        assert_eq!(fitsio_str.unwrap(), first_string);

        // A repeated read just returns the first string again.
        let fitsio_str = hdu.read_key::<String>(&mut fptr, "foo");
        assert!(fitsio_str.is_ok());
        assert_eq!(fitsio_str.unwrap(), first_string);

        remove_file(&file).unwrap();
    }
}
