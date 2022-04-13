// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Unit tests for fits reading functions
*/
#[cfg(test)]
use super::*;
use crate::misc::test::*;
use crate::*;
use fitsio::images::{ImageDescription, ImageType};
use fitsio::tables::{ColumnDataType, ColumnDescription};
use fitsio_sys::ffpkls;
use float_cmp::*;

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
fn test_get_fits_image_valid_f32() {
    // with_temp_file creates a temp dir and temp file, then removes them once out of scope
    with_new_temp_fits_file("test_get_fits_image.fits", |mut fptr| {
        // Ensure we have 1 hdu
        fits_open_hdu!(fptr, 0).expect("Couldn't open HDU 0");

        let image_description = ImageDescription {
            data_type: ImageType::Float,
            dimensions: &[1, 3],
        };

        // Create a new image HDU
        fptr.create_image("EXTNAME".to_string(), &image_description)
            .unwrap();
        let hdu = fits_open_hdu!(fptr, 1).expect("Couldn't open HDU 1");

        // Write some data
        assert!(hdu.write_image(&mut fptr, &[1.0, 2.0, 3.0]).is_ok());

        // Run our test, check dimensions
        let size_vec = get_hdu_image_size!(fptr, &hdu).unwrap();

        assert_eq!(size_vec.len(), 2);
        assert_eq!(size_vec[0], 1);
        assert_eq!(size_vec[1], 3);

        // Read and check data
        let result1: Result<Vec<f32>, FitsError> = get_fits_image!(fptr, &hdu);
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), vec![1.0, 2.0, 3.0]);
    });
}

#[test]
fn test_get_fits_image_valid_i32() {
    // with_temp_file creates a temp dir and temp file, then removes them once out of scope
    with_new_temp_fits_file("test_get_fits_image.fits", |mut fptr| {
        // Ensure we have 1 hdu
        fits_open_hdu!(fptr, 0).expect("Couldn't open HDU 0");

        let image_description = ImageDescription {
            data_type: ImageType::Long,
            dimensions: &[1, 3],
        };

        // Create a new image HDU
        fptr.create_image("EXTNAME".to_string(), &image_description)
            .unwrap();
        let hdu = fits_open_hdu!(fptr, 1).expect("Couldn't open HDU 1");

        // Write some data
        assert!(hdu.write_image(&mut fptr, &[-1, 0, 1]).is_ok());

        // Run our test, check dimensions
        let size_vec = get_hdu_image_size!(fptr, &hdu).unwrap();

        assert_eq!(size_vec.len(), 2);
        assert_eq!(size_vec[0], 1);
        assert_eq!(size_vec[1], 3);

        // Read and check data
        let result1: Result<Vec<i32>, FitsError> = get_fits_image!(fptr, &hdu);
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), vec![-1, 0, 1]);
    });
}

#[test]
fn test_get_fits_image_invalid() {
    // with_temp_file creates a temp dir and temp file, then removes them once out of scope
    with_new_temp_fits_file("test_get_fits_image.fits", |mut fptr| {
        // Ensure we have 1 hdu
        fits_open_hdu!(fptr, 0).expect("Couldn't open HDU 0");

        let image_description = ImageDescription {
            data_type: ImageType::Long,
            dimensions: &[1, 3],
        };

        // Create a new image HDU
        fptr.create_image("EXTNAME".to_string(), &image_description)
            .unwrap();
        let hdu = fits_open_hdu!(fptr, 1).expect("Couldn't open HDU 1");

        // Write some data
        assert!(hdu
            .write_image(&mut fptr, &[-12345678, 0, 12345678])
            .is_ok());

        // Run our test, check dimensions
        let size_vec = get_hdu_image_size!(fptr, &hdu).unwrap();

        assert_eq!(size_vec.len(), 2);
        assert_eq!(size_vec[0], 1);
        assert_eq!(size_vec[1], 3);

        // Read and check data- this should be an error due to a type mismatch
        let result1: Result<Vec<u8>, FitsError> = get_fits_image!(fptr, &hdu);
        assert!(result1.is_err());

        assert!(matches!(
            result1.unwrap_err(),
            FitsError::Fitsio {
                fits_error: _,
                fits_filename: _,
                hdu_num: _,
                source_file: _,
                source_line: _
            }
        ));
    });
}

#[test]
fn test_get_fits_image_not_image() {
    // with_temp_file creates a temp dir and temp file, then removes them once out of scope
    with_new_temp_fits_file("test_get_fits_image.fits", |fptr| {
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

        // Read image. This should fail with a specific fits error
        let result1: Result<Vec<f32>, FitsError> = get_fits_image!(fptr, &hdu);
        assert!(result1.is_err());
        assert!(matches!(
            result1.unwrap_err(),
            FitsError::NotImage {
                fits_filename: _,
                hdu_num: _,
                source_file: _,
                source_line: _
            }
        ));
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
        let fits_value_i32 = hdu.read_key::<i32>(fptr, "FOO");
        let fits_value_i64 = hdu.read_key::<i64>(fptr, "FOO");
        assert!(fits_value_i32.is_ok());
        assert!(fits_value_i64.is_ok());
        assert_eq!(fits_value_i32.unwrap(), 1);
        assert_eq!(fits_value_i64.unwrap(), 10);

        // Despite writing to "fits_value", the key is written as "FOO".
        let fits_value_i64 = hdu.read_key::<i64>(fptr, "FOO");
        assert!(fits_value_i64.is_ok());
        assert_eq!(fits_value_i64.unwrap(), 10);

        let fits_value_u8: Result<u8, _> = get_required_fits_key!(fptr, &hdu, "foo");
        let fits_value_i8: Result<i8, _> = get_required_fits_key!(fptr, &hdu, "foo");
        assert!(fits_value_u8.is_ok());
        assert!(fits_value_i8.is_ok());
        assert_eq!(fits_value_u8.unwrap(), 10);
        assert_eq!(fits_value_i8.unwrap(), 10);

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
        let fits_value: Result<Option<u8>, _> = get_optional_fits_key!(fptr, &hdu, "foo");
        assert!(fits_value.is_ok());
        assert!(fits_value.unwrap().is_none());

        // Key types must be i64 to get any sort of sanity.
        hdu.write_key(fptr, "foo", 10i64)
            .expect("Couldn't write key 'foo'");
        hdu.write_key(fptr, "bar", -5i64)
            .expect("Couldn't write key 'bar'");

        // Verify that using the normal `fitsio` gives the wrong result, unless
        // the type is an i64.
        let fits_value_i32 = hdu.read_key::<i32>(fptr, "FOO");
        let fits_value_i64 = hdu.read_key::<i64>(fptr, "FOO");
        assert!(fits_value_i32.is_ok());
        assert!(fits_value_i64.is_ok());
        assert_eq!(fits_value_i32.unwrap(), 1);
        assert_eq!(fits_value_i64.unwrap(), 10);

        // Despite writing to "foo", the key is written as "FOO".
        let fits_value_i64 = hdu.read_key::<i64>(fptr, "FOO");
        assert!(fits_value_i64.is_ok());
        assert_eq!(fits_value_i64.unwrap(), 10);

        let fits_value_u8: Result<Option<u8>, _> = get_optional_fits_key!(fptr, &hdu, "foo");
        let fits_value_i8: Result<Option<i8>, _> = get_optional_fits_key!(fptr, &hdu, "foo");
        assert!(fits_value_u8.is_ok());
        assert!(fits_value_i8.is_ok());
        assert_eq!(fits_value_u8.unwrap(), Some(10));
        assert_eq!(fits_value_i8.unwrap(), Some(10));

        // Can't parse the negative number into a unsigned int.
        let bar_u8: Result<Option<u8>, _> = get_optional_fits_key!(fptr, &hdu, "bar");
        let bar_i8: Result<Option<i8>, _> = get_optional_fits_key!(fptr, &hdu, "bar");
        assert!(bar_u8.is_err());
        assert!(bar_i8.is_ok());
        assert_eq!(bar_i8.unwrap().unwrap(), -5);
    });
}

#[test]
fn test_get_required_fits_key_string() {
    // with_temp_file creates a temp dir and temp file, then removes them once out of scope
    with_new_temp_fits_file("test_fits_read_key_string.fits", |fptr| {
        let hdu = fits_open_hdu!(fptr, 0).expect("Couldn't open HDU 0");

        // Failure to get a key that doesn't exist.
        let does_not_exist: Result<String, FitsError> =
            get_required_fits_key!(fptr, &hdu, "fits_value");
        assert!(does_not_exist.is_err());

        // Add a test string
        hdu.write_key(fptr, "fits_value", "hello")
            .expect("Couldn't write key 'fits_value'");

        // Read fits_value back in
        let fits_value_string: String = get_required_fits_key!(fptr, &hdu, "fits_value").unwrap();

        // Despite writing to "fits_value", the key is written as "FOO".
        assert_eq!(fits_value_string, "hello");
    });
}

#[test]
fn test_get_optional_fits_key_string() {
    // with_temp_file creates a temp dir and temp file, then removes them once out of scope
    with_new_temp_fits_file("test_fits_read_key_string.fits", |fptr| {
        let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

        // No Failure to get a key that doesn't exist.
        let does_not_exist: Result<Option<String>, FitsError> =
            get_optional_fits_key!(fptr, &hdu, "fits_value");

        assert!(does_not_exist.is_ok());
        assert!(does_not_exist.unwrap().is_none());

        // Add a test string
        hdu.write_key(fptr, "fits_value", "hello")
            .expect("Couldn't write key 'fits_value'");

        // Read fits_value back in
        let fits_value_string: Result<Option<String>, FitsError> =
            get_optional_fits_key!(fptr, &hdu, "fits_value");

        // Despite writing to "fits_value", the key is written as "FOO".
        assert!(fits_value_string.is_ok());
        assert_eq!(fits_value_string.unwrap().unwrap(), "hello");
    });
}

#[test]
fn test_get_fits_long_string() {
    // with_temp_file creates a temp dir and temp file, then removes them once out of scope
    with_new_temp_fits_file("test_get_fits_long_string.fits", |fptr| {
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

        let hdu = fptr.hdu(0).unwrap();
        let fits_value_str = get_required_fits_key_long_string!(fptr, &hdu, "FOO");
        assert!(fits_value_str.is_ok());
        assert_eq!(fits_value_str.unwrap(), complete_string);

        // Try out the `fitsio` read key.
        let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");
        let fitsio_str = hdu.read_key::<String>(fptr, "FOO");
        assert!(fitsio_str.is_ok());
        assert_eq!(fitsio_str.unwrap(), first_string);

        // A repeated read just returns the first string again.
        let fitsio_str = hdu.read_key::<String>(fptr, "FOO");
        assert!(fitsio_str.is_ok());
        assert_eq!(fitsio_str.unwrap(), first_string);
    });
}

#[test]
fn test_get_required_fits_long_string() {
    // with_temp_file creates a temp dir and temp file, then removes them once out of scope
    with_new_temp_fits_file("test_get_fits_long_string.fits", |fptr| {
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

        let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

        // Check for a valid long string
        let result1 = get_required_fits_key_long_string!(fptr, &hdu, "FOO");
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), complete_string);

        // Try out the `fitsio` read key.
        let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");
        let fitsio_str = hdu.read_key::<String>(fptr, "FOO");
        assert!(fitsio_str.is_ok());
        assert_eq!(fitsio_str.unwrap(), first_string);

        // A repeated read just returns the first string again.
        let fitsio_str = hdu.read_key::<String>(fptr, "FOO");
        assert!(fitsio_str.is_ok());
        assert_eq!(fitsio_str.unwrap(), first_string);

        // Check for a invalid key long string
        let result2 = get_required_fits_key_long_string!(fptr, &hdu, "BAR");
        assert!(matches!(
            result2.unwrap_err(),
            FitsError::MissingKey {
                key: _,
                fits_filename: _,
                hdu_num: _,
                source_file: _,
                source_line: _
            }
        ));
    });
}

#[test]
fn test_get_optional_fits_long_string() {
    // with_temp_file creates a temp dir and temp file, then removes them once out of scope
    with_new_temp_fits_file("test_get_fits_long_string.fits", |fptr| {
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

        let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

        // Read a key that IS there
        let result1 = get_optional_fits_key_long_string!(fptr, &hdu, "FOO");
        // Check no error
        assert!(result1.is_ok());
        let fits_value1_str = result1.unwrap();
        // Check they match
        assert_eq!(fits_value1_str.unwrap(), complete_string);

        // Try out the `fitsio` read key.
        let fitsio_str = hdu.read_key::<String>(fptr, "FOO");
        assert!(fitsio_str.is_ok());
        assert_eq!(fitsio_str.unwrap(), first_string);

        // A repeated read just returns the first string again.
        let fitsio_str = hdu.read_key::<String>(fptr, "FOO");
        assert!(fitsio_str.is_ok());
        assert_eq!(fitsio_str.unwrap(), first_string);

        // Now read a key that does NOT exist
        let result2 = get_optional_fits_key_long_string!(fptr, &hdu, "BAR");
        // Check no error
        assert!(result2.is_ok());
        let fits_value2_str = result2.unwrap();
        // Check it returns None
        assert!(fits_value2_str.is_none());
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
            let keyword_ffi = CString::new("fits_value")
                .expect("get_fits_long_string: CString::new() failed for 'fits_value'");
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

        let hdu = fptr.hdu(0).unwrap();
        let fits_value_str = get_required_fits_key_long_string!(fptr, &hdu, "NOTfits_value");
        assert!(fits_value_str.is_err());
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
    assert!(approx_eq!(f64, freq_centre, 154.24, F64Margin::default()));
    let fine_chan_width: u8 = get_required_fits_key!(&mut fptr, &hdu, "FINECHAN")?;
    assert_eq!(fine_chan_width, 10);

    let doesnt_exist: Result<f64, FitsError> = get_required_fits_key!(&mut fptr, &hdu, "FINE");
    assert!(doesnt_exist.is_err());

    let cant_parse: Result<u64, FitsError> = get_required_fits_key!(&mut fptr, &hdu, "FREQCENT");
    assert!(cant_parse.is_err());

    let hdu_too_big = fits_open_hdu!(&mut fptr, 1000);
    assert!(hdu_too_big.is_err());

    let hdu = fits_open_hdu!(&mut fptr, 1)?;
    let east: Vec<f32> = get_fits_col!(&mut fptr, &hdu, "East")?;
    assert!(approx_eq!(f32, east[0], -585.675, F32Margin::default()));
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
    assert!(approx_eq!(f64, freq_centre, 147.84, F64Margin::default()));
    let fine_chan_width: u8 = get_required_fits_key!(&mut fptr, &hdu, "FINECHAN")?;
    assert_eq!(fine_chan_width, 10);

    let doesnt_exist: Result<f64, FitsError> = get_required_fits_key!(&mut fptr, &hdu, "FINE");
    assert!(doesnt_exist.is_err());

    let cant_parse: Result<u64, FitsError> = get_required_fits_key!(&mut fptr, &hdu, "FREQCENT");
    assert!(cant_parse.is_err());

    let hdu_too_big = fits_open_hdu!(&mut fptr, 1000);
    assert!(hdu_too_big.is_err());

    let hdu = fits_open_hdu!(&mut fptr, 1)?;
    let east: Vec<f32> = get_fits_col!(&mut fptr, &hdu, "East")?;
    assert!(approx_eq!(f32, east[0], -585.675, F32Margin::default()));
    let doesnt_exist: Result<Vec<f32>, FitsError> = get_fits_col!(&mut fptr, &hdu, "South");
    assert!(doesnt_exist.is_err());
    Ok(())
}
