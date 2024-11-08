// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Unit tests for fits reading functions

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
    with_new_temp_fits_file("test_get_fits_image.fits", |fptr| {
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
        assert!(hdu.write_image(fptr, &[1.0, 2.0, 3.0]).is_ok());

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
    with_new_temp_fits_file("test_get_fits_image.fits", |fptr| {
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
        assert!(hdu.write_image(fptr, &[-1, 0, 1]).is_ok());

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
    with_new_temp_fits_file("test_get_fits_image.fits", |fptr| {
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
        assert!(hdu.write_image(fptr, &[-12345678, 0, 12345678]).is_ok());

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
    let fine_chan_width: f32 = get_required_fits_key!(&mut fptr, &hdu, "FINECHAN")?;
    assert_eq!(fine_chan_width, 10.0);

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
fn test_read_cell_value_valid() {
    let metafits_filename = "test_files/metafits_cal_sol/1111842752_metafits.fits";
    let mut fptr = fits_open!(&metafits_filename).unwrap();
    let hdu = fits_open_hdu!(&mut fptr, 2).unwrap();

    let result_value: Result<f32, FitsError> = read_cell_value(&mut fptr, &hdu, "Calib_Delay", 0);
    assert!(result_value.is_ok());

    let value = result_value.unwrap();

    assert!(float_cmp::approx_eq!(
        f32,
        value,
        -135.49985,
        float_cmp::F32Margin::default()
    ));
}

#[test]
fn test_read_cell_value_nan() {
    let metafits_filename = "test_files/metafits_cal_sol/1111842752_metafits.fits";
    let mut fptr = fits_open!(&metafits_filename).unwrap();
    let hdu = fits_open_hdu!(&mut fptr, 2).unwrap();

    let result_value: Result<f32, FitsError> = read_cell_value(&mut fptr, &hdu, "Calib_Delay", 2);
    assert!(result_value.is_ok());

    let value = result_value.unwrap();

    assert!(float_cmp::approx_eq!(
        f32,
        value,
        f32::NAN,
        float_cmp::F32Margin::default()
    ));

    // Reassign to another float
    let new_float = value;

    assert!(float_cmp::approx_eq!(
        f32,
        new_float,
        f32::NAN,
        float_cmp::F32Margin::default()
    ));
}

#[test]
fn test_read_cell_array_u32() {
    let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
    let mut fptr = fits_open!(&metafits_filename).unwrap();
    let hdu = fits_open_hdu!(&mut fptr, 1).unwrap();

    let delays = read_cell_array_u32(&mut fptr, &hdu, "Delays", 0, 16);
    assert!(delays.is_ok());
    assert_eq!(&delays.unwrap(), &[0; 16]);

    let digital_gains = read_cell_array_u32(&mut fptr, &hdu, "Gains", 0, 24);
    assert!(digital_gains.is_ok());
    assert_eq!(
        digital_gains.unwrap(),
        &[
            74, 73, 73, 72, 71, 70, 68, 67, 66, 65, 65, 65, 66, 66, 65, 65, 64, 64, 64, 65, 65, 66,
            67, 68,
        ]
    );

    let asdf = read_cell_array_u32(&mut fptr, &hdu, "NotReal", 0, 24);
    assert!(asdf.is_err());

    let asdf = read_cell_array_u32(&mut fptr, &hdu, "Delays", 999, 16);
    assert!(asdf.is_err());
}

#[test]
fn test_read_cell_array_f32() {
    let metafits_filename = "test_files/metafits_cal_sol/1111842752_metafits.fits";
    let mut fptr = fits_open!(&metafits_filename).unwrap();
    let hdu = fits_open_hdu!(&mut fptr, 2).unwrap();

    let gains = read_cell_array_f32(&mut fptr, &hdu, "Calib_Gains", 0, 24);
    assert!(gains.is_ok());
    assert_eq!(
        gains.unwrap(),
        &[
            0.9759591, 0.9845909, 0.9918698, 1.0030527, 1.0122633, 1.011114, 1.0253462, 1.0438296,
            1.0606546, 1.0932951, 1.1034626, 1.1153094, 1.1335917, 1.1480684, 1.1665345, 1.1722057,
            1.2186697, 1.2541704, 1.2993327, 1.35544, 1.410614, 1.488467, 1.544273, 1.6084857,
        ]
    );

    let gains_nans = read_cell_array_f32(&mut fptr, &hdu, "Calib_Gains", 2, 24);
    assert!(gains_nans.is_ok());
    let gains_nans = gains_nans.unwrap();
    assert!(gains_nans[0].is_nan());
    assert!(gains_nans[23].is_nan());

    test_func(gains_nans);

    fn test_func(nan_float_vec: Vec<f32>) {
        assert!(nan_float_vec[0].is_nan());
        assert!(nan_float_vec[23].is_nan());
    }

    let asdf = read_cell_array_f32(&mut fptr, &hdu, "NotReal", 0, 24);
    assert!(asdf.is_err());
}

#[test]
fn test_read_cell_array_f64() {
    let metafits_filename = "test_files/metafits_signal_chain_corr/1096952256_metafits.fits";
    let mut fptr = fits_open!(&metafits_filename).unwrap();
    let hdu = fits_open_hdu_by_name!(&mut fptr, "SIGCHAINDATA").unwrap();

    let row = 0; // should be RRI no whitening filter

    let corrections =
        read_cell_array_f64(&mut fptr, &hdu, "corrections", row, MAX_RECEIVER_CHANNELS);
    assert!(corrections.is_ok());
    assert_eq!(
        corrections.unwrap(),
        &[
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.16073910960211837,
            0.03376808122489539,
            -0.16875572467295086,
            -0.2726834291618158,
            -0.29100507181168256,
            -0.31404361416672355,
            -0.35770916097994576,
            -0.39735109579857036,
            -0.42380750248793997,
            -0.45720499055600966,
            -0.49881996754325747,
            -0.5466102156845976,
            -0.5902517956432913,
            -0.640851829526285,
            -0.6946601848374445,
            -0.7477974857408997,
            -0.7903966699358909,
            -0.7851318357223317,
            -0.6059008657392213,
            0.1972529119976446,
            0.9159293479649063,
            1.0584393700018015,
            1.0432414488129331,
            0.9791018507686252,
            0.895747685408171,
            0.802170131856825,
            0.7040280213476127,
            0.5951632964652229,
            0.4799332972547082,
            0.3542760037810869,
            0.21954773035567407,
            0.07495932871505957,
            -0.07829068611215982,
            -0.23483891873218882,
            -0.38910449476007053,
            -0.5375875007381348,
            -0.6776067078892374,
            -0.8045688528700525,
            -0.9193495917550321,
            -1.0227228257758973,
            -1.1159565164228131,
            -1.2004469852863593,
            -1.2767973302021556,
            -1.347572827221595,
            -1.4128208507897722,
            -1.474227906772989,
            -1.5306058898383734,
            -1.5828714119603253,
            -1.6309091143834211,
            -1.6759463989382288,
            -1.7165482046038276,
            -1.7530123984049486,
            -1.7878645768581038,
            -1.8184826416330773,
            -1.8458297380337774,
            -1.8711014877683314,
            -1.8926201927696045,
            -1.9119997516320868,
            -1.9292801390316783,
            -1.9424550760776582,
            -1.955141724921763,
            -1.9638048138489257,
            -1.9720689990430844,
            -1.9774895705234985,
            -1.9814649960314157,
            -1.9830268965126037,
            -1.9842979245238517,
            -1.9828567038032836,
            -1.9808576111711738,
            -1.977437923653932,
            -1.9710550867168755,
            -1.963894552619373,
            -1.955666027977895,
            -1.9464051151788755,
            -1.9353472245344836,
            -1.9234949546866809,
            -1.9103379670098404,
            -1.8958678373962816,
            -1.8808482370877706,
            -1.864949321938699,
            -1.8484201247489462,
            -1.83133169666019,
            -1.8136884907836552,
            -1.7962061048291669,
            -1.7779311354211755,
            -1.7587406610685499,
            -1.7398304863940541,
            -1.7205780638507455,
            -1.7010040653414518,
            -1.6820026670229444,
            -1.6628391562450502,
            -1.6421504494470107,
            -1.62212350646555,
            -1.6024907082026354,
            -1.5827745090536072,
            -1.5624682153734661,
            -1.5420859337360902,
            -1.5220610139973965,
            -1.5009745316552985,
            -1.4794841349719223,
            -1.4587995946811692,
            -1.437249506286105,
            -1.4146758531250516,
            -1.3926346610019302,
            -1.3683328015677823,
            -1.3464757427229508,
            -1.3216232100172864,
            -1.2971030800643764,
            -1.2721780280576418,
            -1.247287322727292,
            -1.2212307853218196,
            -1.195112195827877,
            -1.168434546618297,
            -1.1417797169966892,
            -1.1136181500804536,
            -1.0863781705089444,
            -1.0597212689150415,
            -1.0312447938162688,
            -1.0025129767063197,
            -0.9726203320867985,
            -0.9442245670950103,
            -0.9145857843927921,
            -0.8856557483795751,
            -0.8568281943589875,
            -0.8268547931202496,
            -0.7970532079015483,
            -0.7662514374677699,
            -0.7364776868598679,
            -0.7061298896453309,
            -0.6748886971907214,
            -0.644315234723551,
            -0.6134998519732355,
            -0.5817654012939172,
            -0.5513095778126074,
            -0.5192384388626234,
            -0.4868956937500769,
            -0.4555785697486637,
            -0.42367066465421427,
            -0.39120411326027404,
            -0.3579635153506763,
            -0.3254358122815729,
            -0.2931940654378237,
            -0.2605547934214806,
            -0.22921659404127293,
            -0.19681024534932043,
            -0.1643615523752524,
            -0.13317172490409226,
            -0.10020471984578648,
            -0.06956393582855606,
            -0.037655455503209354,
            -0.007911837757060525,
            0.023148783382734434,
            0.052960107939851946,
            0.08251575042350444,
            0.11095983964672508,
            0.14167385964626167,
            0.1684027715506908,
            0.19637040844482165,
            0.22424077302227124,
            0.2506956767433734,
            0.27717416014684293,
            0.3024548873044261,
            0.32804494180477595,
            0.35225319809927397,
            0.37658394340744356,
            0.3995768935109655,
            0.4224919129178774,
            0.44639301954491584,
            0.4683592754042337,
            0.490823181385476,
            0.5126533149923969,
            0.5341770305355963,
            0.5565625040142232,
            0.5775892255139037,
            0.5989103803405952,
            0.6197132944770236,
            0.6404252147000588,
            0.6623027621577716,
            0.6827335066112413,
            0.7041397012442325,
            0.7257810975553967,
            0.7464738592452299,
            0.7677033910152425,
            0.789285618716982,
            0.8095139425028725,
            0.8313673132185,
            0.8523074247470024,
            0.8734007930828113,
            0.8946159891426515,
            0.9156315821677309,
            0.9376684723891006,
            0.96043319831508,
            0.9826305560356353,
            1.0048199770304564,
            1.0274982948002858,
            1.0502131746500893,
            1.0735225454157236,
            1.0952211235987876,
            1.1181945764427663,
            1.138446395388729,
            1.1600746323024633,
            1.18187743109126,
            1.2003692639347328,
            1.2190760308289308,
            1.2362829944831693,
            1.2528726558831247,
            1.2671982788912146,
            1.2789045923632167,
            1.2891721124565936,
            1.2946643778544529,
            1.2981044392688323,
            1.2979659749021706,
            1.2930622894469268,
            1.2821780440500878,
            1.2671605932553054,
            1.2448016886670161,
            1.2171892910372473,
            1.1840266707632825,
            1.143317254695277,
            1.0989365813942888,
            1.0495242553555002,
            1.001749738440827,
            0.9514803198528214,
            0.9061282270374892,
            0.8632106451584965,
            0.8257200434001825,
            0.7956566595911808,
            0.7727745112028128,
            0.7547167251776683,
            0.748082665512771,
            0.7462463845742535,
            0.7475747352187672,
            0.7598147243238643
        ]
    );

    let asdf = read_cell_array_f64(&mut fptr, &hdu, "NotReal", 0, 24);
    assert!(asdf.is_err());
}

#[test]
fn test_is_fitsio_reentrant() {
    assert!(is_fitsio_reentrant())
}
