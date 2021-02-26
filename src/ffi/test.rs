use super::*;
use std::fs::File;
use std::io::{Error, Write};
use std::mem;

//
// Helper methods for many tests
//

/// Create and return a metafits context based on a test metafits file. Used in many tests in the module.
///
///
/// # Arguments
///
/// * None
///
///
/// # Returns
///
/// * a raw pointer to an instantiated MetafitsContext for the test metafits and gpubox file
///
fn get_test_metafits_context() -> *mut MetafitsContext {
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    unsafe {
        // Create a MetafitsContext
        let mut metafits_context_ptr: *mut MetafitsContext = std::ptr::null_mut();
        let retval = mwalib_metafits_context_new(
            metafits_file_ptr,
            &mut metafits_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_metafits_context_new
        assert_eq!(retval, 0, "mwalib_metafits_context_new failure");

        // Check we got valid MetafitsContext pointer
        let context_ptr = metafits_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        context_ptr.unwrap()
    }
}

/// Create and return a correlator context ptr based on a test metafits and gpubox file. Used in many tests in the module.
///
///
/// # Arguments
///
/// * None
///
///
/// # Returns
///
/// * a raw pointer to an instantiated CorrelatorContext for the test metafits and gpubox file
///
fn get_test_correlator_context() -> *mut CorrelatorContext {
    // This tests for a valid correlator context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file =
        CString::new("test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits")
            .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

    unsafe {
        // Create a CorrelatorContext
        let mut correlator_context_ptr: *mut CorrelatorContext = std::ptr::null_mut();
        let retval = mwalib_correlator_context_new(
            metafits_file_ptr,
            gpubox_files_ptr,
            1,
            &mut correlator_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_correlator_context_new
        assert_eq!(retval, 0, "mwalib_correlator_context_new failure");

        // Check we got valid MetafitsContext pointer
        let context_ptr = correlator_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        context_ptr.unwrap()
    }
}

/// Create and return a voltage context ptr based on a test metafits and voltage file. Used in many tests in the module.
///
///
/// # Arguments
///
/// * None
///
///
/// # Returns
///
/// * a raw pointer to an instantiated VoltageContext for the test metafits and voltage file
///
fn get_test_voltage_context() -> *mut VoltageContext {
    // This tests for a valid voltage context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    // Create a temp dir for the temp files
    // Once out of scope the temp dir and it's contents will be deleted
    let temp_dir = tempdir::TempDir::new("voltage_test").unwrap();

    // Populate vector of filenames
    let new_voltage_filename1 = CString::new(
        generate_test_voltage_file(&temp_dir, "1101503312_1101503312_123.sub", 2, 256).unwrap(),
    )
    .unwrap();
    let new_voltage_filename2 = CString::new(
        generate_test_voltage_file(&temp_dir, "1101503312_1101503312_124.sub", 2, 256).unwrap(),
    )
    .unwrap();
    let new_voltage_filename3 = CString::new(
        generate_test_voltage_file(&temp_dir, "1101503312_1101503320_123.sub", 2, 256).unwrap(),
    )
    .unwrap();
    let new_voltage_filename4 = CString::new(
        generate_test_voltage_file(&temp_dir, "1101503312_1101503320_124.sub", 2, 256).unwrap(),
    )
    .unwrap();
    let mut input_voltage_files: Vec<*const c_char> = Vec::new();
    input_voltage_files.push(new_voltage_filename1.as_ptr());
    input_voltage_files.push(new_voltage_filename2.as_ptr());
    input_voltage_files.push(new_voltage_filename3.as_ptr());
    input_voltage_files.push(new_voltage_filename4.as_ptr());

    let voltage_files_ptr = input_voltage_files.as_ptr() as *mut *const c_char;

    unsafe {
        // Create a VoltageContext
        let mut voltage_context_ptr: *mut VoltageContext = std::ptr::null_mut();
        let retval = mwalib_voltage_context_new(
            metafits_file_ptr,
            voltage_files_ptr,
            input_voltage_files.len(),
            &mut voltage_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_voltage_context_new
        let mut ret_error_message: String = String::new();

        if retval != 0 {
            let c_str: &CStr = CStr::from_ptr(error_message_ptr);
            let str_slice: &str = c_str.to_str().unwrap();
            ret_error_message = str_slice.to_owned();
        }
        assert_eq!(
            retval, 0,
            "mwalib_voltage_context_new failure {}, files = {:?}",
            ret_error_message, input_voltage_files
        );

        // Check we got valid VoltageContext pointer
        let context_ptr = voltage_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        context_ptr.unwrap()
    }
}

/// Reconstructs a Vec<T> from FFI using a pointer to a rust-allocated array of *mut T.
///
///
/// # Arguments
///
/// * `ptr` - raw pointer pointing to an array of T
///
/// * 'len' - number of elements in the array
///
///
/// # Returns
///
/// * Array of T expressed as Vec<T>
///
fn ffi_boxed_slice_to_array<T>(ptr: *mut T, len: usize) -> Vec<T> {
    unsafe {
        let vec: Vec<T> = Vec::from_raw_parts(ptr, len, len);
        vec
    }
}

fn ffi_array_to_boxed_slice<T>(rust_vector_or_slice: Vec<T>) -> (*mut T, usize) {
    let mut boxed_slice: Box<[T]> = rust_vector_or_slice.into_boxed_slice();
    let array_ptr: *mut T = boxed_slice.as_mut_ptr();
    let array_ptr_len: usize = boxed_slice.len();
    assert_eq!(array_ptr_len, array_ptr_len);

    // Prevent the slice from being destroyed (Leak the memory).
    // This is because we are using our ffi code to free the memory
    mem::forget(boxed_slice);

    (array_ptr, array_ptr_len)
}

// Helper fuction to generate (small) test voltage files
fn generate_test_voltage_file(
    temp_dir: &tempdir::TempDir,
    filename: &str,
    time_samples: usize,
    rf_inputs: usize,
) -> Result<String, Error> {
    let tdir_path = temp_dir.path();
    let full_filename = tdir_path.join(filename);

    let mut output_file = File::create(&full_filename)?;
    // Write out x time samples
    // Each sample has x rfinputs
    // and 1 float for real 1 float for imaginary
    let floats = time_samples * rf_inputs * 2;
    let mut buffer: Vec<f32> = vec![0.0; floats];

    let mut bptr: usize = 0;

    // This will write out the sequence:
    // 0.25, 0.75, 1.25, 1.75..511.25,511.75  (1024 floats in all)
    for t in 0..time_samples {
        for r in 0..rf_inputs {
            // real
            buffer[bptr] = ((t * rf_inputs) + r) as f32 + 0.25;
            bptr += 1;
            // imag
            buffer[bptr] = ((t * rf_inputs) + r) as f32 + 0.75;
            bptr += 1;
        }
    }
    output_file.write_all(misc::as_u8_slice(buffer.as_slice()))?;
    output_file.flush()?;

    Ok(String::from(full_filename.to_str().unwrap()))
}

//
// Simple test of the error message helper
//
#[test]
fn test_set_error_message() {
    let buffer = CString::new("HELLO WORLD").unwrap();
    let buffer_ptr = buffer.as_ptr() as *mut u8;

    set_error_message("hello world", buffer_ptr, 12);

    assert_eq!(buffer, CString::new("hello world").unwrap());
}

#[test]
fn test_set_error_message_null_ptr() {
    let buffer_ptr: *mut i8 = std::ptr::null_mut();
    unsafe {
        assert_eq!(mwalib_free_rust_cstring(buffer_ptr), 0);
    }
}

#[test]
fn test_mwalib_free_rust_cstring() {
    let buffer = CString::new("HELLO WORLD").unwrap();
    let buffer_ptr = buffer.into_raw() as *mut i8;

    // into_raw will take garbage collection of the buffer away from rust, so
    // some ffi/C code can free it (like below)
    unsafe {
        assert_eq!(mwalib_free_rust_cstring(buffer_ptr), 0);
    }
}

//
// Metafits context Tests
//
#[test]
fn test_mwalib_metafits_context_new_valid() {
    // This tests for a valid metafitscontext
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    unsafe {
        // Create a MetafitsContext
        let mut metafits_context_ptr: *mut MetafitsContext = std::ptr::null_mut();
        let retval = mwalib_metafits_context_new(
            metafits_file_ptr,
            &mut metafits_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_metafits_context_new
        assert_eq!(retval, 0, "mwalib_metafits_context_new failure");

        // Check we got valid MetafitsContext pointer
        let context_ptr = metafits_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Now ensure we can free the rust memory
        assert_eq!(mwalib_metafits_context_free(context_ptr.unwrap()), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_metafits_context_free(std::ptr::null_mut()), 0);
    }
}

//
// CorrelatorContext Tests
//
#[test]
fn test_mwalib_correlator_context_new_valid() {
    // This tests for a valid correlator context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    let gpubox_file =
        CString::new("test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits")
            .unwrap();
    let mut gpubox_files: Vec<*const c_char> = Vec::new();
    gpubox_files.push(gpubox_file.as_ptr());
    let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

    unsafe {
        // Create a CorrelatorContext
        let mut correlator_context_ptr: *mut CorrelatorContext = std::ptr::null_mut();
        let retval = mwalib_correlator_context_new(
            metafits_file_ptr,
            gpubox_files_ptr,
            1,
            &mut correlator_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_correlator_context_new
        assert_eq!(retval, 0, "mwalib_correlator_context_new failure");

        // Check we got valid CorrelatorContext pointer
        let context_ptr = correlator_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Now ensure we can free the rust memory
        assert_eq!(mwalib_correlator_context_free(context_ptr.unwrap()), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_correlator_context_free(std::ptr::null_mut()), 0);
    }
}

//
// VoltageContext Tests
//
#[test]
fn test_mwalib_voltage_context_new_valid() {
    // This tests for a valid voltage context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    let metafits_file =
        CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
    let metafits_file_ptr = metafits_file.as_ptr();

    // Create a temp dir for the temp files
    // Once out of scope the temp dir and it's contents will be deleted
    let temp_dir = tempdir::TempDir::new("voltage_test").unwrap();

    // Create a test file
    let voltage_file = CString::new(
        generate_test_voltage_file(&temp_dir, "1101503312_1101503312_123.sub", 2, 256).unwrap(),
    )
    .unwrap();
    let mut voltage_files: Vec<*const c_char> = Vec::new();
    voltage_files.push(voltage_file.as_ptr());
    let voltage_files_ptr = voltage_files.as_ptr() as *mut *const c_char;

    unsafe {
        // Create a VoltageContext
        let mut voltage_context_ptr: *mut VoltageContext = std::ptr::null_mut();
        let retval = mwalib_voltage_context_new(
            metafits_file_ptr,
            voltage_files_ptr,
            1,
            &mut voltage_context_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value of mwalib_voltage_context_new
        let mut ret_error_message: String = String::new();

        if retval != 0 {
            let c_str: &CStr = CStr::from_ptr(error_message_ptr);
            let str_slice: &str = c_str.to_str().unwrap();
            ret_error_message = str_slice.to_owned();
        }
        assert_eq!(
            retval, 0,
            "mwalib_voltage_context_new failure {}",
            ret_error_message
        );

        // Check we got valid VoltageContext pointer
        let context_ptr = voltage_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Now ensure we can free the rust memory
        assert_eq!(mwalib_voltage_context_free(context_ptr.unwrap()), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_voltage_context_free(std::ptr::null_mut()), 0);
    }
}

//
// Metafits Metadata Tests
//
#[test]
fn test_mwalib_metafits_metadata_get_from_metafits_context_valid() {
    // This tests for a valid metafits context and metadata returned
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;
    // Create a MetafitsContext
    let metafits_context_ptr: *mut MetafitsContext = get_test_metafits_context();
    unsafe {
        // Check we got valid MetafitsContext pointer
        let context_ptr = metafits_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: &mut *mut MetafitsMetadata = &mut std::ptr::null_mut();
        let retval = mwalib_metafits_metadata_get(
            metafits_context_ptr,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut metafits_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        let mut ret_error_message: String = String::new();

        if retval != 0 {
            let c_str: &CStr = CStr::from_ptr(error_message_ptr);
            let str_slice: &str = c_str.to_str().unwrap();
            ret_error_message = str_slice.to_owned();
        }
        assert_eq!(
            retval, 0,
            "mwalib_metafits_metadata_get failure {}",
            ret_error_message
        );

        // Get the mwalibMetadata struct from the pointer
        let metafits_metadata = Box::from_raw(*metafits_metadata_ptr);

        // We should get a valid obsid and no error message
        assert_eq!(metafits_metadata.obs_id, 1_101_503_312);

        // Now ensure we can free the rust memory
        assert_eq!(
            mwalib_metafits_metadata_free(Box::into_raw(metafits_metadata)),
            0
        );

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_metafits_metadata_free(std::ptr::null_mut()), 0);
    }
}

#[test]
fn test_mwalib_metafits_metadata_get_null_contexts() {
    // This tests for a null context passed to mwalib_metafits_metadata_get
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut metafits_metadata_ptr: &mut *mut MetafitsMetadata = &mut std::ptr::null_mut();
        let ret_val = mwalib_metafits_metadata_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut metafits_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // We should get a non-zero return code
        assert_ne!(ret_val, 0);
    }
}

#[test]
fn test_mwalib_metafits_metadata_get_from_correlator_context_valid() {
    // This tests for a valid metafits metadata returned given a correlator context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        // Create a CorrelatorContext
        let correlator_context_ptr: *mut CorrelatorContext = get_test_correlator_context();

        // Check we got valid MetafitsContext pointer
        let context_ptr = correlator_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: &mut *mut MetafitsMetadata = &mut std::ptr::null_mut();
        let retval = mwalib_metafits_metadata_get(
            std::ptr::null_mut(),
            correlator_context_ptr,
            std::ptr::null_mut(),
            &mut metafits_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        assert_eq!(
            retval, 0,
            "mwalib_metafits_metadata_get did not return success"
        );

        // Get the mwalibMetadata struct from the pointer
        let metafits_metadata = Box::from_raw(*metafits_metadata_ptr);

        // We should get a valid obsid and no error message
        assert_eq!(metafits_metadata.obs_id, 1_101_503_312);

        // Now ensure we can free the rust memory
        assert_eq!(
            mwalib_metafits_metadata_free(Box::into_raw(metafits_metadata)),
            0
        );

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_metafits_metadata_free(std::ptr::null_mut()), 0);
    }
}

#[test]
fn test_mwalib_metafits_metadata_get_from_voltage_context_valid() {
    // This tests for a valid metafits metadata returned given a voltage context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        // Create a VoltageContext
        let voltage_context_ptr: *mut VoltageContext = get_test_voltage_context();

        // Check we got valid MetafitsContext pointer
        let context_ptr = voltage_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a mwalibMetafitsMetadata struct
        let mut metafits_metadata_ptr: &mut *mut MetafitsMetadata = &mut std::ptr::null_mut();
        let retval = mwalib_metafits_metadata_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            voltage_context_ptr,
            &mut metafits_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        assert_eq!(
            retval, 0,
            "mwalib_metafits_metadata_get did not return success"
        );

        // Get the metafits metadata struct from the pointer
        let metafits_metadata = Box::from_raw(*metafits_metadata_ptr);

        // We should get a valid obsid and no error message
        assert_eq!(metafits_metadata.obs_id, 1_101_503_312);

        // Now ensure we can free the rust memory
        assert_eq!(
            mwalib_metafits_metadata_free(Box::into_raw(metafits_metadata)),
            0
        );

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_metafits_metadata_free(std::ptr::null_mut()), 0);
    }
}

#[test]
fn test_mwalib_correlator_metadata_get_valid() {
    // This tests for a valid correlator metadata struct being instantiated
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        // Create a CorrelatorContext
        let correlator_context_ptr: *mut CorrelatorContext = get_test_correlator_context();

        // Check we got valid MetafitsContext pointer
        let context_ptr = correlator_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a CorrelatorMetadata struct
        let mut correlator_metadata_ptr: &mut *mut CorrelatorMetadata = &mut std::ptr::null_mut();
        let retval = mwalib_correlator_metadata_get(
            correlator_context_ptr,
            &mut correlator_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        assert_eq!(
            retval, 0,
            "mwalib_correlator_metadata_get did not return success"
        );

        // Get the correlator metadata struct from the pointer
        let correlator_metadata = Box::from_raw(*correlator_metadata_ptr);

        // We should get a valid number of coarse channels and no error message
        assert_eq!(correlator_metadata.num_coarse_chans, 1);

        // Now ensure we can free the rust memory
        assert_eq!(
            mwalib_correlator_metadata_free(Box::into_raw(correlator_metadata)),
            0
        );

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_correlator_metadata_free(std::ptr::null_mut()), 0);
    }
}

#[test]
fn test_mwalib_correlator_metadata_get_null_context() {
    // This tests for passing a null context to the mwalib_correlator_metadata_get() method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut correlator_metadata_ptr: &mut *mut CorrelatorMetadata = &mut std::ptr::null_mut();

        let context_ptr = std::ptr::null_mut();
        let ret_val = mwalib_correlator_metadata_get(
            context_ptr,
            &mut correlator_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // We should get a non-zero return code
        assert_ne!(ret_val, 0);
    }
}

#[test]
fn test_mwalib_voltage_metadata_get_valid() {
    // This tests for a valid voltage metadata struct being instantiated
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        // Create a VoltageContext
        let voltage_context_ptr: *mut VoltageContext = get_test_voltage_context();

        // Check we got valid MetafitsContext pointer
        let context_ptr = voltage_context_ptr.as_mut();
        assert!(context_ptr.is_some());

        // Populate a VoltageMetadata struct
        let mut voltage_metadata_ptr: &mut *mut VoltageMetadata = &mut std::ptr::null_mut();
        let retval = mwalib_voltage_metadata_get(
            voltage_context_ptr,
            &mut voltage_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // Check return value
        assert_eq!(
            retval, 0,
            "mwalib_voltage_metadata_get did not return success"
        );

        // Get the voltage metadata struct from the pointer
        let voltage_metadata = Box::from_raw(*voltage_metadata_ptr);

        // We should get a valid number of coarse channels and no error message
        assert_eq!(voltage_metadata.num_coarse_chans, 2);

        // Now ensure we can free the rust memory
        assert_eq!(
            mwalib_voltage_metadata_free(Box::into_raw(voltage_metadata)),
            0
        );

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_voltage_metadata_free(std::ptr::null_mut()), 0);
    }
}

#[test]
fn test_mwalib_voltage_metadata_get_null_context() {
    // This tests for passing a null context to the mwalib_voltage_metadata_get() method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut voltage_metadata_ptr: &mut *mut VoltageMetadata = &mut std::ptr::null_mut();

        let context_ptr = std::ptr::null_mut();
        let ret_val = mwalib_voltage_metadata_get(
            context_ptr,
            &mut voltage_metadata_ptr,
            error_message_ptr,
            error_len,
        );

        // We should get a non-zero return code
        assert_ne!(ret_val, 0);
    }
}

#[test]
fn test_mwalib_antennas_get_from_metafits_context_valid() {
    // This test populates antennas given a metafits context
    let index = 2; // valid  should be Tile013

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_metafits_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut Antenna = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_antennas_get(
            context,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_antennas_get did not return success");

        // reconstitute into a vector
        let item: Vec<Antenna> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 128, "Array length is not correct");
        assert_eq!(item[index].tile_id, 13);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let (array_to_free, array_to_free_len) = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_antennas_free(array_to_free, array_to_free_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_antennas_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_antennas_get_from_correlator_context_valid() {
    // This test populates antennas given a correlator context
    let index = 2; // valid  should be Tile013
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_correlator_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut Antenna = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_antennas_get(
            std::ptr::null_mut(),
            context,
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_antennas_get did not return success");

        // reconstitute into a vector
        let item: Vec<Antenna> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 128, "Array length is not correct");
        assert_eq!(item[index].tile_id, 13);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let (array_to_free, array_to_free_len) = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_antennas_free(array_to_free, array_to_free_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_antennas_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_antennas_get_from_voltage_context_valid() {
    // This test populates antennas given a voltage context
    let index = 2; // valid  should be Tile013
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_voltage_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut Antenna = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_antennas_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            context,
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_antennas_get did not return success");

        // reconstitute into a vector
        let item: Vec<Antenna> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 128, "Array length is not correct");
        assert_eq!(item[index].tile_id, 13);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let (array_to_free, array_to_free_len) = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_antennas_free(array_to_free, array_to_free_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_antennas_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_antennas_get_null_contexts() {
    // This tests for a null context passed into the _get method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut array_ptr: &mut *mut Antenna = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;
        let retval = mwalib_antennas_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer, non-zero retval and an error message
        assert_ne!(retval, 0);
        assert!(array_ptr.is_null());
        let expected_error: &str = &"mwalib_antennas_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

// Baselines
#[test]
fn test_mwalib_baselines_get_valid_using_metafits_context() {
    // This test populates baselines given a metafits context
    let index = 2; // valid  should be baseline (0,2)

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_metafits_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut Baseline = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_baselines_get(
            context,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_baselines_get did not return success");

        // reconstitute into a vector
        let item: Vec<Baseline> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 8256, "Array length is not correct");
        assert_eq!(item[index].ant1_index, 0);
        assert_eq!(item[index].ant2_index, 2);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let (array_to_free, array_to_free_len) = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_baselines_free(array_to_free, array_to_free_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_baselines_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_baselines_get_valid_using_correlator_context() {
    // This test populates baselines given a correlator context
    let index = 2; // valid  should be baseline (0,2)

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_correlator_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut Baseline = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_baselines_get(
            std::ptr::null_mut(),
            context,
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_baselines_get did not return success");

        // reconstitute into a vector
        let item: Vec<Baseline> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 8256, "Array length is not correct");
        assert_eq!(item[index].ant1_index, 0);
        assert_eq!(item[index].ant2_index, 2);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let (array_to_free, array_to_free_len) = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_baselines_free(array_to_free, array_to_free_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_baselines_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_baselines_get_valid_using_voltage_context() {
    // This test populates baselines given a voltage context
    let index = 2; // valid  should be baseline (0,2)

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_voltage_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut Baseline = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_baselines_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            context,
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_baselines_get did not return success");

        // reconstitute into a vector
        let item: Vec<Baseline> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 8256, "Array length is not correct");
        assert_eq!(item[index].ant1_index, 0);
        assert_eq!(item[index].ant2_index, 2);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let (array_to_free, array_to_free_len) = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_baselines_free(array_to_free, array_to_free_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_baselines_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_baselines_get_null_context() {
    // This tests for a null context passed into the _get method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut array_ptr: &mut *mut Baseline = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;
        let retval = mwalib_baselines_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer, non-zero retval and an error message
        assert_ne!(retval, 0);
        assert!(array_ptr.is_null());
        let expected_error: &str = &"mwalib_baselines_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

// Coarse Channels
#[test]
fn test_mwalib_correlator_coarse_channels_get_valid() {
    // This test populates coarse_chans given a correlator context
    let index = 0; // valid  should be receiver channel 109

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_correlator_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut CoarseChannel = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_correlator_coarse_channels_get(
            context,
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(
            retval, 0,
            "mwalib_correlator_coarse_channels_get did not return success"
        );

        // reconstitute into a vector
        let item: Vec<CoarseChannel> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 1, "Array length is not correct");
        assert_eq!(item[index].rec_chan_number, 109);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let (array_to_free, array_to_free_len) = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(
            mwalib_coarse_channels_free(array_to_free, array_to_free_len),
            0
        );

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_coarse_channels_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_correlator_coarse_channels_get_null_context() {
    // This tests for a null context passed into the _get method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut array_ptr: &mut *mut CoarseChannel = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;
        let retval = mwalib_correlator_coarse_channels_get(
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer, non-zero retval and an error message
        assert_ne!(retval, 0);
        assert!(array_ptr.is_null());
        let expected_error: &str = &"mwalib_correlator_coarse_channels_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

#[test]
fn test_mwalib_voltage_coarse_channels_get_valid() {
    // This test populates coarse_chans given a voltage context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_voltage_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut CoarseChannel = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_voltage_coarse_channels_get(
            context,
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(
            retval, 0,
            "mwalib_voltage_coarse_channels_get did not return success"
        );

        // reconstitute into a vector
        let item: Vec<CoarseChannel> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(
            array_len, 2,
            "Coarse channel array length is not correct- should be 2"
        );
        assert_eq!(item[0].rec_chan_number, 123);
        assert_eq!(item[1].rec_chan_number, 124);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let (array_to_free, array_to_free_len) = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(
            mwalib_coarse_channels_free(array_to_free, array_to_free_len),
            0
        );

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_coarse_channels_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_voltage_coarse_channels_get_null_context() {
    // This tests for a null context passed into the _get method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut array_ptr: &mut *mut CoarseChannel = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;
        let retval = mwalib_voltage_coarse_channels_get(
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer, non-zero retval and an error message
        assert_ne!(retval, 0);
        assert!(array_ptr.is_null());
        let expected_error: &str = &"mwalib_voltage_coarse_channels_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

// RF Input
#[test]
fn test_mwalib_rfinputs_get_from_metafits_context_valid() {
    // This test populates rfinputs given a metafits context
    let index = 2; // valid  should be Tile012(X)

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_metafits_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut RFInput = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_rfinputs_get(
            context,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_rfinputs_get did not return success");

        // reconstitute into a vector
        let item: Vec<RFInput> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 256, "Array length is not correct");

        assert_eq!(item[index].ant, 1);

        assert_eq!(
            CString::from_raw(item[index].tile_name),
            CString::new("Tile012").unwrap()
        );

        assert_eq!(
            CString::from_raw(item[index].pol),
            CString::new("X").unwrap()
        );
    }
}
#[test]
fn test_mwalib_rfinputs_free() {
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_metafits_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut RFInput = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_rfinputs_get(
            context,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_rfinputs_get did not return success");

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_rfinputs_free(*array_ptr, array_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_rfinputs_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_rfinputs_get_from_correlator_context_valid() {
    // This test populates rfinputs given a correlator context
    let index = 2; // valid  should be Tile012(X)

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_correlator_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut RFInput = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_rfinputs_get(
            std::ptr::null_mut(),
            context,
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_rfinputs_get did not return success");

        // reconstitute into a vector
        let item: Vec<RFInput> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 256, "Array length is not correct");

        assert_eq!(item[index].ant, 1);

        assert_eq!(
            CString::from_raw(item[index].tile_name),
            CString::new("Tile012").unwrap()
        );

        assert_eq!(
            CString::from_raw(item[index].pol),
            CString::new("X").unwrap()
        );
    }
}

#[test]
fn test_mwalib_rfinputs_get_from_voltage_context_valid() {
    // This test populates rfinputs given a voltage context
    let index = 2; // valid  should be Tile012(X)

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_voltage_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut RFInput = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_rfinputs_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            context,
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_rfinputs_get did not return success");

        // reconstitute into a vector
        let item: Vec<RFInput> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 256, "Array length is not correct");

        assert_eq!(item[index].ant, 1);

        assert_eq!(
            CString::from_raw(item[index].tile_name),
            CString::new("Tile012").unwrap()
        );

        assert_eq!(
            CString::from_raw(item[index].pol),
            CString::new("X").unwrap()
        );
    }
}

#[test]
fn test_mwalib_rfinputs_get_null_contexts() {
    // This tests for a null context passed into the _get method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut array_ptr: &mut *mut RFInput = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;
        let retval = mwalib_rfinputs_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer, non-zero retval and an error message
        assert_ne!(retval, 0);
        assert!(array_ptr.is_null());
        let expected_error: &str = &"mwalib_rfinputs_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

// Timesteps
#[test]
fn test_mwalib_correlator_timesteps_get_valid() {
    // This test populates timesteps given a correlator context
    let index = 0; // valid  should be timestep at unix_time 1417468096.0;

    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_correlator_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut TimeStep = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_correlator_timesteps_get(
            context,
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(retval, 0, "mwalib_timesteps_get did not return success");

        // reconstitute into a vector
        let item: Vec<TimeStep> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 1, "Array length is not correct");
        assert_eq!(item[index].unix_time_ms, 1_417_468_096_000);

        // Test freeing the memory
        // First get a raw pointer and have rust not own that memory
        let (array_to_free, array_to_free_len) = ffi_array_to_boxed_slice(item);

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_timesteps_free(array_to_free, array_to_free_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_timesteps_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_correlator_timesteps_get_null_context() {
    // This tests for a null context passed into the _get method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut array_ptr: &mut *mut TimeStep = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;
        let retval = mwalib_correlator_timesteps_get(
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer, non-zero retval and an error message
        assert_ne!(retval, 0);
        assert!(array_ptr.is_null());
        let expected_error: &str = &"mwalib_correlator_timesteps_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}

// Visibility Pols
#[test]
fn test_mwalib_visibility_pols_get_valid_using_metafits_context() {
    // This test populates visibility_pols given a metafits context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_metafits_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut VisibilityPol = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_visibility_pols_get(
            context,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(
            retval, 0,
            "mwalib_correlator_visibility_pols_get did not return success"
        );

        // reconstitute into a vector
        let item: Vec<VisibilityPol> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 4, "Array length is not correct");
        assert_eq!(
            CString::from_raw(item[0].polarisation),
            CString::new("XX").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[1].polarisation),
            CString::new("XY").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[2].polarisation),
            CString::new("YX").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[3].polarisation),
            CString::new("YY").unwrap()
        );
    }
}
#[test]
fn test_mwalib_visibility_pols_get_valid_using_correlator_context() {
    // This test populates visibility_pols given a correlator context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_correlator_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut VisibilityPol = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_visibility_pols_get(
            std::ptr::null_mut(),
            context,
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(
            retval, 0,
            "mwalib_correlator_visibility_pols_get did not return success"
        );

        // reconstitute into a vector
        let item: Vec<VisibilityPol> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 4, "Array length is not correct");
        assert_eq!(
            CString::from_raw(item[0].polarisation),
            CString::new("XX").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[1].polarisation),
            CString::new("XY").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[2].polarisation),
            CString::new("YX").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[3].polarisation),
            CString::new("YY").unwrap()
        );
    }
}

#[test]
fn test_mwalib_visibility_pols_get_valid_using_voltage_context() {
    // This test populates visibility_pols given a voltage context
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_voltage_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut VisibilityPol = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_visibility_pols_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            context,
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(
            retval, 0,
            "mwalib_correlator_visibility_pols_get did not return success"
        );

        // reconstitute into a vector
        let item: Vec<VisibilityPol> = ffi_boxed_slice_to_array(*array_ptr, array_len);

        // We should get a valid, populated array
        assert_eq!(array_len, 4, "Array length is not correct");
        assert_eq!(
            CString::from_raw(item[0].polarisation),
            CString::new("XX").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[1].polarisation),
            CString::new("XY").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[2].polarisation),
            CString::new("YX").unwrap()
        );
        assert_eq!(
            CString::from_raw(item[3].polarisation),
            CString::new("YY").unwrap()
        );
    }
}

#[test]
fn test_mwalib_visibility_pols_free() {
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let context = get_test_metafits_context();

        // Check we got a context object
        let context_ptr = context.as_mut();
        assert!(context_ptr.is_some());

        let mut array_ptr: &mut *mut VisibilityPol = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;

        let retval = mwalib_visibility_pols_get(
            context,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // check ret val is ok
        assert_eq!(
            retval, 0,
            "mwalib_correlator_visibility_pols_get did not return success"
        );

        // Now we are good to actually free the memory via our ffi function
        assert_eq!(mwalib_visibility_pols_free(*array_ptr, array_len), 0);

        // Now ensure we don't panic if we try to free a null pointer
        assert_eq!(mwalib_visibility_pols_free(std::ptr::null_mut(), 0), 0);
    }
}

#[test]
fn test_mwalib_visibilitypols_get_null_context() {
    // This tests for a null context passed into the _get method
    let error_len: size_t = 128;
    let error_message = CString::new(" ".repeat(error_len)).unwrap();
    let error_message_ptr = error_message.as_ptr() as *const c_char;

    unsafe {
        let mut array_ptr: &mut *mut VisibilityPol = &mut std::ptr::null_mut();
        let mut array_len: usize = 0;
        let retval = mwalib_visibility_pols_get(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut array_ptr,
            &mut array_len,
            error_message_ptr,
            error_len,
        );

        // We should get a null pointer, non-zero retval and an error message
        assert_ne!(retval, 0);
        assert!(array_ptr.is_null());
        let expected_error: &str = &"mwalib_visibility_pols_get() ERROR:";
        assert_eq!(
            error_message.into_string().unwrap()[0..expected_error.len()],
            *expected_error
        );
    }
}
