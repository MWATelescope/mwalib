use std::ffi::*;
use std::ptr;
use std::str;

use fitsio::{hdu::*, FitsFile};
use fitsio_sys::{ffgkls, ffopen, fitsfile};
use libc::c_char;

use crate::error::ErrorKind;

/// Given a FITS file pointer, a HDU that belongs to it, and a keyword, pull out
/// the value of the keyword, parsing it into the desired type.
///
/// Benchmarks show that, for pulling out an i64, this function is *slightly*
/// slower than just using the hdu with read_key::<i64> (by ~100ns on my Ryzen 9
/// 3900X). But, even for small FITS values (e.g. BITPIX = -32), using an i32
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

/// Via FFI, get a file pointer to a FITS file.
// pub unsafe fn open_fits_ffi(filename: &str) -> (i32, ptr::NonNull<fitsfile>) {
pub unsafe fn open_fits_ffi(filename: &str) -> (i32, *mut fitsfile) {
    let mut fptr = ptr::null_mut();
    let filename_ffi = CString::new(filename).expect("open_fits_ffi: CString::new() failed");
    // 0 for read only.
    let iomode = 0;
    let mut status = 0;
    ffopen(
        &mut fptr as *mut *mut _,
        filename_ffi.as_ptr(),
        iomode,
        &mut status,
    );
    (status, fptr)
}

/// Via FFI, get a long string from a FITS file.
///
/// This function exists because the rust library `fitsio` does not support
/// reading in long strings (i.e. those that have CONTINUE statements).
pub unsafe fn get_fits_long_string(fptr: *mut fitsfile, keyword: &str) -> (i32, String) {
    let keyword_ffi =
        CString::new(keyword).expect("get_fits_long_string: CString::new() failed for keyword");
    // For reasons I cannot fathom, ffgkls expects `value` to be a malloc'd
    // char** in C, but will only use a char* inside it. Anyway, Vec<*mut c_char>
    // works for me in rust.
    let mut value: Vec<*mut c_char> = vec![ptr::null_mut()];
    let mut status = 0;
    #[allow(clippy::cast_ptr_alignment)]
    ffgkls(
        fptr,
        keyword_ffi.as_ptr(),
        value.as_mut_ptr() as *mut *mut c_char,
        ptr::null_mut(),
        &mut status,
    );
    let long_string = CString::from_raw(value[0])
        .to_str()
        .expect("get_fits_long_string: converting string_ptr failed")
        .to_owned();
    (status, long_string)
}
