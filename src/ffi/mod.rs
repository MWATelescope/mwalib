// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
This module exists purely for other languages to interface with mwalib.

It's very difficult to provide errors to external callers, as rust's concept of
ownership means that any strings made by rust must also be deallocated by
rust. For now, the caller must use these interfaces correctly, and the
correctness of mwalib is verified by using rust directly (and some testing via
C).
 */

use crate::*;
use libc::{c_char, c_float, size_t};
use std::ffi::*;
use std::slice;

/// Generic helper function for all FFI modules to take an already allocated C string
/// and update it with an error message. This is used to pass error messages back to C from Rust.
///
/// # Arguments
///
/// * `in_message` - A Rust string holing the error message you want to pass back to C
///
/// * `error_buffer_ptr` - Pointer to a char* buffer which has already been allocated, for storing the error message.
///
/// * `error_buffer_len` - Length of char* buffer allocated by caller in C.
///
///
/// # Returns
///
/// * Nothing
///
///
/// # Safety
/// It is up to the caller to:
/// - Allocate `error_buffer_len` bytes as a `char*` on the heap
/// - Free `error_buffer_ptr` once finished with the buffer
///
fn set_error_message(in_message: &str, error_buffer_ptr: *mut u8, error_buffer_len: size_t) {
    // Don't do anything if the pointer is null.
    if error_buffer_ptr.is_null() {
        return;
    }
    // Check that error buffer, minus 1 for nul terminator is still >=1
    if error_buffer_len as i32 - 1 < 1 {
        return;
    }
    // Trim it to error_buffer_len - 1 (must include room for null terminator)
    let in_buffer_len = in_message.len();
    let message = if in_buffer_len > error_buffer_len {
        &in_message[..error_buffer_len - 1]
    } else {
        in_message
    };

    // Convert to C string- panic if it can't.
    let error_message = CString::new(message).unwrap();

    // Add null terminator
    let error_message_bytes = error_message.as_bytes();

    unsafe {
        // Reconstruct a string to write into
        let error_message_slice = slice::from_raw_parts_mut(error_buffer_ptr, error_buffer_len);

        // Copy in the bytes
        error_message_slice[..error_message_bytes.len()].copy_from_slice(error_message_bytes);
    }
}

/// Free a rust-allocated CString.
///
/// mwalib uses error strings to detail the caller with anything that went
/// wrong. Non-rust languages cannot deallocate these strings; so, call this
/// function with the pointer to do that.
///
/// # Arguments
///
/// * `rust_cstring` - pointer to a `char*` of a Rust string
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
/// # Safety
/// * rust_cstring must not have already been freed and must point to a Rust string.
#[no_mangle]
pub unsafe extern "C" fn mwalib_free_rust_cstring(rust_cstring: *mut c_char) -> i32 {
    // Don't do anything if the pointer is null.
    if rust_cstring.is_null() {
        return 0;
    }
    CString::from_raw(rust_cstring);

    // return success
    0
}

/// Boxes for FFI a rust-allocated array of T.
///
///
/// # Arguments
///
/// * `v` - Rust vector of T's
///
///
/// # Returns
///
/// * a raw pointer to the array of T's
///
fn array_to_ffi_boxed_slice<T>(v: Vec<T>) -> *mut T {
    // Going from Vec<_> to Box<[_]> just drops the (extra) `capacity`
    let boxed_slice: Box<[T]> = v.into_boxed_slice();
    let fat_ptr: *mut [T] = Box::into_raw(boxed_slice);
    let slim_ptr: *mut T = fat_ptr as _;
    slim_ptr
}

/// Create and return a pointer to an `MetafitsContext` struct given only a metafits file
///
/// # Arguments
///
/// * `metafits_filename` - pointer to char* buffer containing the full path and filename of a metafits file.
///
/// * `out_metafits_context_ptr` - A Rust-owned populated `MetafitsContext` pointer. Free with `mwalib_metafits_context_free'.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated `char*` buffer for any error messages.
/// * Caller *must* call the `mwalib_metafits_context_free` function to release the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_context_new(
    metafits_filename: *const c_char,
    out_metafits_context_ptr: &mut *mut MetafitsContext,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    let m = CStr::from_ptr(metafits_filename)
        .to_str()
        .unwrap()
        .to_string();
    let context = match MetafitsContext::new(&m) {
        Ok(c) => c,
        Err(e) => {
            set_error_message(
                &format!("{}", e),
                error_message as *mut u8,
                error_message_length,
            );
            // Return failure
            return 1;
        }
    };

    *out_metafits_context_ptr = Box::into_raw(Box::new(context));

    // Return success
    0
}

/// Display an `MetafitsContext` struct.
///
///
/// # Arguments
///
/// * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `metafits_context_ptr` must contain an MetafitsContext object already populated via `mwalib_metafits_context_new`
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_context_display(
    metafits_context_ptr: *const MetafitsContext,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    if metafits_context_ptr.is_null() {
        set_error_message(
            "mwalib_metafits_context_display() ERROR: null pointer for metafits_context_ptr passed in",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    }

    let context = &*metafits_context_ptr;

    println!("{}", context);

    // Return success
    0
}

/// Free a previously-allocated `MetafitsContext` struct (and it's members).
///
/// # Arguments
///
/// * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `MetafitsContext` object
/// * `metafits_context_ptr` must point to a populated `MetafitsContext` object from the `mwalib_metafits_context_new` functions.
/// * `metafits_context_ptr` must not have already been freed.
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_context_free(
    metafits_context_ptr: *mut MetafitsContext,
) -> i32 {
    if metafits_context_ptr.is_null() {
        return 0;
    }

    // Release correlator context if applicable
    Box::from_raw(metafits_context_ptr);

    // Return success
    0
}

/// Create and return a pointer to an `CorrelatorContext` struct based on metafits and gpubox files
///
/// # Arguments
///
/// * `metafits_filename` - pointer to char* buffer containing the full path and filename of a metafits file.
///
/// * `gpubox_filenames` - pointer to array of char* buffers containing the full path and filename of the gpubox FITS files.
///
/// * `gpubox_count` - length of the gpubox char* array.
///
/// * `out_correlator_context_ptr` - A Rust-owned populated `CorrelatorContext` pointer. Free with `mwalib_correlator_context_free`.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated `char*` buffer for any error messages.
/// * Caller *must* call function `mwalib_correlator_context_free` to release the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_context_new(
    metafits_filename: *const c_char,
    gpubox_filenames: *mut *const c_char,
    gpubox_count: size_t,
    out_correlator_context_ptr: &mut *mut CorrelatorContext,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    let m = CStr::from_ptr(metafits_filename)
        .to_str()
        .unwrap()
        .to_string();
    let gpubox_slice = slice::from_raw_parts(gpubox_filenames, gpubox_count);
    let mut gpubox_files = Vec::with_capacity(gpubox_count);
    for g in gpubox_slice {
        let s = CStr::from_ptr(*g).to_str().unwrap();
        gpubox_files.push(s.to_string())
    }
    let context = match CorrelatorContext::new(&m, &gpubox_files) {
        Ok(c) => c,
        Err(e) => {
            set_error_message(
                &format!("{}", e),
                error_message as *mut u8,
                error_message_length,
            );
            // Return failure
            return 1;
        }
    };
    *out_correlator_context_ptr = Box::into_raw(Box::new(context));
    // Return success
    0
}

/// Display an `CorrelatorContext` struct.
///
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must contain an `CorrelatorContext` object already populated via `mwalib_correlator_context_new`
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_context_display(
    correlator_context_ptr: *const CorrelatorContext,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    if correlator_context_ptr.is_null() {
        set_error_message(
            "mwalib_correlator_context() ERROR: null pointer for correlator_context_ptr passed in",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    }

    let context = &*correlator_context_ptr;

    println!("{}", context);

    // Return success
    0
}

/// Read a single timestep / coarse channel of MWA data.
///
/// This method takes as input a timestep_index and a coarse_channel_index to return one
/// HDU of data in [baseline][freq][pol][r][i] format
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
///
/// * `timestep_index` - index within the timestep array for the desired timestep. This corresponds
///                      to TimeStep.get(context, N) where N is timestep_index.
///
/// * `coarse_channel_index` - index within the coarse_channel array for the desired coarse channel. This corresponds
///                            to CoarseChannel.get(context, N) where N is coarse_channel_index.
///
/// * `buffer_ptr` - pointer to caller-owned and allocated buffer to write data into.
///
/// * `buffer_len` - length of `buffer_ptr`.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated object from the `mwalib_correlator_context_new` function.
/// * Caller *must* call `mwalib_correlator_context_free_read_buffer` function to release the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_context_read_by_baseline(
    correlator_context_ptr: *mut CorrelatorContext,
    timestep_index: size_t,
    coarse_channel_index: size_t,
    buffer_ptr: *mut c_float,
    buffer_len: size_t,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    // Load the previously-initialised context and buffer structs. Exit if
    // either of these are null.
    let corr_context = if correlator_context_ptr.is_null() {
        set_error_message(
            "mwalib_correlator_context_read_by_baseline() ERROR: null pointer for correlator_context_ptr passed in",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    } else {
        &mut *correlator_context_ptr
    };

    // Don't do anything if the buffer pointer is null.
    if buffer_ptr.is_null() {
        return 1;
    }

    let output_slice = slice::from_raw_parts_mut(buffer_ptr, buffer_len);

    // Read data in.
    let data = match corr_context.read_by_baseline(timestep_index, coarse_channel_index) {
        Ok(data) => data,
        Err(e) => {
            set_error_message(
                &format!("{}", e),
                error_message as *mut u8,
                error_message_length,
            );
            return 1;
        }
    };

    // If the data buffer is empty, then just return a null pointer.
    if data.is_empty() {
        set_error_message(
            "mwalib_correlator_context_read_by_baseline() ERROR: no data was returned.",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    }

    // Populate the buffer which was provided to us by caller
    output_slice[..data.len()].copy_from_slice(data.as_slice());
    // Return Success
    0
}

/// Read a single timestep / coarse channel of MWA data.
///
/// This method takes as input a timestep_index and a coarse_channel_index to return one
/// HDU of data in [freq][baseline][pol][r][i] format
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
///
/// * `timestep_index` - index within the timestep array for the desired timestep. This corresponds
///                      to TimeStep.get(context, N) where N is timestep_index.
///
/// * `coarse_channel_index` - index within the coarse_channel array for the desired coarse channel. This corresponds
///                            to CoarseChannel.get(context, N) where N is coarse_channel_index.
///
/// * `buffer_ptr` - pointer to caller-owned and allocated buffer to write data into.
///
/// * `buffer_len` - length of `buffer_ptr`.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated object from the `mwalib_correlator_context_new` function.
/// * Caller *must* call `mwalib_correlator_context_free_read_buffer` function to release the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_context_read_by_frequency(
    correlator_context_ptr: *mut CorrelatorContext,
    timestep_index: size_t,
    coarse_channel_index: size_t,
    buffer_ptr: *mut c_float,
    buffer_len: size_t,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    // Load the previously-initialised context and buffer structs. Exit if
    // either of these are null.
    let corr_context = if correlator_context_ptr.is_null() {
        set_error_message(
            "mwalib_correlator_context_read_by_frequency() ERROR: null pointer for correlator_context_ptr passed in",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    } else {
        &mut *correlator_context_ptr
    };
    // Don't do anything if the buffer pointer is null.
    if buffer_ptr.is_null() {
        return 1;
    }

    let output_slice = slice::from_raw_parts_mut(buffer_ptr, buffer_len);

    // Read data in.
    let data = match corr_context.read_by_frequency(timestep_index, coarse_channel_index) {
        Ok(data) => data,
        Err(e) => {
            set_error_message(
                &format!("{}", e),
                error_message as *mut u8,
                error_message_length,
            );
            return 1;
        }
    };

    // If the data buffer is empty, then just return a null pointer.
    if data.is_empty() {
        set_error_message(
            "mwalib_correlator_context_read_by_frequency() ERROR: no data was returned.",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    }

    // Populate the buffer which was provided to us by caller
    output_slice[..data.len()].copy_from_slice(data.as_slice());
    // Return Success
    0
}

/// Free a previously-allocated `CorrelatorContext` struct (and it's members).
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `CorrelatorContext` object
/// * `correlator_context_ptr` must point to a populated `CorrelatorContext` object from the `mwalib_correlator_context_new` function.
/// * `correlator_context_ptr` must not have already been freed.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_context_free(
    correlator_context_ptr: *mut CorrelatorContext,
) -> i32 {
    if correlator_context_ptr.is_null() {
        return 0;
    }
    // Release correlator context if applicable
    Box::from_raw(correlator_context_ptr);

    // Return success
    0
}

/// Create and return a pointer to an `VoltageContext` struct based on metafits and voltage files
///
/// # Arguments
///
/// * `metafits_filename` - pointer to char* buffer containing the full path and filename of a metafits file.
///
/// * `voltage_filenames` - pointer to array of char* buffers containing the full path and filename of the voltage files.
///
/// * `voltage_file_count` - length of the voltage char* array.
///
/// * `out_voltage_context_ptr` - A Rust-owned populated `VoltageContext` pointer. Free with `mwalib_voltage_context_free`.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated `char*` buffer for any error messages.
/// * Caller *must* call function `mwalib_voltage_context_free` to release the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_voltage_context_new(
    metafits_filename: *const c_char,
    voltage_filenames: *mut *const c_char,
    voltage_file_count: size_t,
    out_voltage_context_ptr: &mut *mut VoltageContext,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    let m = CStr::from_ptr(metafits_filename)
        .to_str()
        .unwrap()
        .to_string();
    let voltage_slice = slice::from_raw_parts(voltage_filenames, voltage_file_count);
    let mut voltage_files = Vec::with_capacity(voltage_file_count);
    for v in voltage_slice {
        let s = CStr::from_ptr(*v).to_str().unwrap();
        voltage_files.push(s.to_string())
    }
    let context = match VoltageContext::new(&m, &voltage_files) {
        Ok(c) => c,
        Err(e) => {
            set_error_message(
                &format!("{}", e),
                error_message as *mut u8,
                error_message_length,
            );
            // Return failure
            return 1;
        }
    };
    *out_voltage_context_ptr = Box::into_raw(Box::new(context));
    // Return success
    0
}

/// Display a `VoltageContext` struct.
///
///
/// # Arguments
///
/// * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `voltage_context_ptr` must contain an `VoltageContext` object already populated via `mwalib_voltage_context_new`
#[no_mangle]
pub unsafe extern "C" fn mwalib_voltage_context_display(
    voltage_context_ptr: *const VoltageContext,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    if voltage_context_ptr.is_null() {
        set_error_message(
            "mwalib_voltage_context() ERROR: null pointer for voltage_context_ptr passed in",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    }

    let context = &*voltage_context_ptr;

    println!("{}", context);

    // Return success
    0
}

/// Free a previously-allocated `VoltageContext` struct (and it's members).
///
/// # Arguments
///
/// * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `VoltageContext` object
/// * `voltage_context_ptr` must point to a populated `VoltageContext` object from the `mwalib_voltage_context_new` function.
/// * `voltage_context_ptr` must not have already been freed.
#[no_mangle]
pub unsafe extern "C" fn mwalib_voltage_context_free(
    voltage_context_ptr: *mut VoltageContext,
) -> i32 {
    if voltage_context_ptr.is_null() {
        return 0;
    }
    // Release voltage context if applicable
    Box::from_raw(voltage_context_ptr);

    // Return success
    0
}

///
/// This a C struct to allow the caller to consume the metafits metadata
///
#[repr(C)]
pub struct MetafitsMetadata {
    /// Observation id
    pub obsid: u32,
    /// Latitude of centre point of MWA in raidans
    pub mwa_latitude_radians: f64,
    /// Longitude of centre point of MWA in raidans
    pub mwa_longitude_radians: f64,
    /// Altitude of centre poing of MWA in metres
    pub mwa_altitude_metres: f64,
    /// the velocity factor of electic fields in RG-6 like coax
    pub coax_v_factor: f64,
    /// ATTEN_DB  // global analogue attenuation, in dB
    pub global_analogue_attenuation_db: f64,
    /// RA tile pointing
    pub ra_tile_pointing_degrees: f64,
    /// DEC tile pointing
    pub dec_tile_pointing_degrees: f64,
    /// RA phase centre
    pub ra_phase_center_degrees: f64,
    /// DEC phase centre
    pub dec_phase_center_degrees: f64,
    /// AZIMUTH
    pub azimuth_degrees: f64,
    /// ALTITUDE
    pub altitude_degrees: f64,
    /// Altitude of Sun
    pub sun_altitude_degrees: f64,
    /// Distance from pointing center to Sun
    pub sun_distance_degrees: f64,
    /// Distance from pointing center to the Moon
    pub moon_distance_degrees: f64,
    /// Distance from pointing center to Jupiter
    pub jupiter_distance_degrees: f64,
    /// Local Sidereal Time
    pub lst_degrees: f64,
    /// Hour Angle of pointing center (as a string)
    pub hour_angle_string: *mut c_char,
    /// GRIDNAME
    pub grid_name: *mut c_char,
    /// GRIDNUM
    pub grid_number: i32,
    /// CREATOR
    pub creator: *mut c_char,
    /// PROJECT
    pub project_id: *mut c_char,
    /// Observation name
    pub observation_name: *mut c_char,
    /// MWA observation mode
    pub mode: *mut c_char,
    /// Scheduled start (gps time) of observation
    pub scheduled_start_utc: i64,
    /// Scheduled end (gps time) of observation
    pub scheduled_end_utc: i64,
    /// Scheduled start (MJD) of observation
    pub scheduled_start_mjd: f64,
    /// Scheduled end (MJD) of observation
    pub scheduled_end_mjd: f64,
    /// Scheduled start (UNIX time) of observation
    pub scheduled_start_unix_time_milliseconds: u64,
    /// Scheduled end (UNIX time) of observation
    pub scheduled_end_unix_time_milliseconds: u64,
    /// Scheduled duration of observation
    pub scheduled_duration_milliseconds: u64,
    /// Seconds of bad data after observation starts
    pub quack_time_duration_milliseconds: u64,
    /// OBSID+QUACKTIM as Unix timestamp (first good timestep)
    pub good_time_unix_milliseconds: u64,
    /// Total number of antennas (tiles) in the array
    pub num_antennas: usize,
    /// The Metafits defines an rf chain for antennas(tiles) * pol(X,Y)
    pub num_rf_inputs: usize,
    /// Number of antenna pols. e.g. X and Y
    pub num_antenna_pols: usize,
    /// Number of coarse channels
    pub num_coarse_channels: usize,
    /// Total bandwidth of observation (of the coarse channels we have)
    pub observation_bandwidth_hz: u32,
    /// Bandwidth of each coarse channel
    pub coarse_channel_width_hz: u32,
}

/// This passed back a struct containing the `MetafitsContext` metadata, given a MetafitsContext, CorrelatorContext or VoltageContext
///
/// # Arguments
///
/// * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object. (Exclusive with correlator_context_ptr and voltage_context_ptr)
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object. (Exclusive with metafits_context_ptr and voltage_context_ptr)
///
/// * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object. (Exclusive with metafits_context_ptr and correlator_context_ptr)
///
/// * `out_metafits_metadata_ptr` - pointer to a Rust-owned `mwalibMetafitsMetadata` struct. Free with `mwalib_metafits_metadata_free`
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `metafits_context_ptr` must point to a populated MetafitsContext object from the `mwalib_metafits_context_new` function OR
/// * `correlator_context_ptr` must point to a populated CorrelatorContext object from the 'mwalib_correlator_context_new' function OR
/// * `voltage_context_ptr` must point to a populated VoltageContext object from the `mwalib_voltage_context_new` function. (Set the unused contexts to NULL).
/// * Caller must call `mwalib_metafits_metadata_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_metadata_get(
    metafits_context_ptr: *mut MetafitsContext,
    correlator_context_ptr: *mut CorrelatorContext,
    voltage_context_ptr: *mut VoltageContext,
    out_metafits_metadata_ptr: &mut *mut MetafitsMetadata,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    // Ensure only either metafits XOR correlator XOR voltage context is passed in
    if !(!metafits_context_ptr.is_null()
        ^ !correlator_context_ptr.is_null()
        ^ !voltage_context_ptr.is_null())
    {
        set_error_message(
            "mwalib_metafits_metadata_get() ERROR: pointers for metafits_context_ptr, correlator_context_ptr and/or voltage_context_ptr were passed in. Only one should be provided.",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    }
    // Create our metafits context pointer depending on what was passed in
    let metafits_context = {
        if !metafits_context_ptr.is_null() {
            // Caller passed in a metafits context, so use that
            &*metafits_context_ptr
        } else {
            if !correlator_context_ptr.is_null() {
                // Caller passed in a correlator context, so use that
                &(&*correlator_context_ptr).metafits_context
            } else {
                // Caller passed in a voltage context, so use that
                &(&*voltage_context_ptr).metafits_context
            }
        }
    };

    // Populate the outgoing structure with data from the metafits context
    let out_context = MetafitsMetadata {
        obsid: metafits_context.obsid,
        mwa_latitude_radians: metafits_context.mwa_latitude_radians,
        mwa_longitude_radians: metafits_context.mwa_longitude_radians,
        mwa_altitude_metres: metafits_context.mwa_altitude_metres,
        coax_v_factor: metafits_context.coax_v_factor,
        global_analogue_attenuation_db: metafits_context.global_analogue_attenuation_db,
        ra_tile_pointing_degrees: metafits_context.ra_tile_pointing_degrees,
        dec_tile_pointing_degrees: metafits_context.dec_tile_pointing_degrees,
        ra_phase_center_degrees: match metafits_context.ra_phase_center_degrees {
            Some(v) => v,
            None => 0.,
        },
        dec_phase_center_degrees: match metafits_context.dec_phase_center_degrees {
            Some(v) => v,
            None => 0.,
        },
        azimuth_degrees: metafits_context.azimuth_degrees,
        altitude_degrees: metafits_context.altitude_degrees,
        sun_altitude_degrees: metafits_context.sun_altitude_degrees,
        sun_distance_degrees: metafits_context.sun_distance_degrees,
        moon_distance_degrees: metafits_context.moon_distance_degrees,
        jupiter_distance_degrees: metafits_context.jupiter_distance_degrees,
        lst_degrees: metafits_context.lst_degrees,
        hour_angle_string: CString::new(String::from(&metafits_context.hour_angle_string))
            .unwrap()
            .into_raw(),
        grid_name: CString::new(String::from(&metafits_context.grid_name))
            .unwrap()
            .into_raw(),
        grid_number: metafits_context.grid_number,
        creator: CString::new(String::from(&metafits_context.creator))
            .unwrap()
            .into_raw(),
        project_id: CString::new(String::from(&metafits_context.project_id))
            .unwrap()
            .into_raw(),
        observation_name: CString::new(String::from(&metafits_context.observation_name))
            .unwrap()
            .into_raw(),
        mode: CString::new(String::from(&metafits_context.mode))
            .unwrap()
            .into_raw(),
        scheduled_start_utc: metafits_context.scheduled_start_utc.timestamp(),
        scheduled_end_utc: metafits_context.scheduled_end_utc.timestamp(),
        scheduled_start_mjd: metafits_context.scheduled_start_mjd,
        scheduled_end_mjd: metafits_context.scheduled_end_mjd,
        scheduled_duration_milliseconds: metafits_context.scheduled_duration_milliseconds,
        scheduled_start_unix_time_milliseconds: metafits_context
            .scheduled_start_unix_time_milliseconds,
        scheduled_end_unix_time_milliseconds: metafits_context.scheduled_end_unix_time_milliseconds,
        quack_time_duration_milliseconds: metafits_context.quack_time_duration_milliseconds,
        good_time_unix_milliseconds: metafits_context.good_time_unix_milliseconds,
        num_antennas: metafits_context.num_antennas,
        num_rf_inputs: metafits_context.num_rf_inputs,
        num_antenna_pols: metafits_context.num_antenna_pols,
        num_coarse_channels: metafits_context.num_coarse_channels,
        observation_bandwidth_hz: metafits_context.observation_bandwidth_hz,
        coarse_channel_width_hz: metafits_context.coarse_channel_width_hz,
    };

    // Pass back a pointer to the rust owned struct
    *out_metafits_metadata_ptr = Box::into_raw(Box::new(out_context));

    // Return Success
    0
}

/// Free a previously-allocated `mwalibMetafitsMetadata` struct.
///
/// # Arguments
///
/// * `metafits_metadata_ptr` - pointer to an already populated `mwalibMetafitsMetadata` object
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `mwalibMetafitsMetadata` object
/// * `metafits_metadata_ptr` must point to a populated `mwalibMetafitsMetadata` object from the `mwalib_metafits_metadata_get` function.
/// * `metafits_metadata_ptr` must not have already been freed.
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_metadata_free(
    metafits_metadata_ptr: *mut MetafitsMetadata,
) -> i32 {
    // If the pointer is null, just return
    if metafits_metadata_ptr.is_null() {
        return 0;
    }
    drop(Box::from_raw(metafits_metadata_ptr));

    // Return success
    0
}

///
/// C Representation of the `CorrelatorContext` metadata
///
#[repr(C)]
pub struct CorrelatorMetadata {
    /// Version of the correlator format
    pub corr_version: CorrelatorVersion,
    /// The proper start of the observation (the time that is common to all
    /// provided gpubox files).
    pub start_unix_time_milliseconds: u64,
    /// `end_time_milliseconds` will is the actual end time of the observation
    /// i.e. start time of last common timestep plus integration time.
    pub end_unix_time_milliseconds: u64,
    /// Total duration of observation (based on gpubox files)
    pub duration_milliseconds: u64,
    /// Number of timesteps in the observation
    pub num_timesteps: usize,
    /// Number of baselines stored. This is autos plus cross correlations
    pub num_baselines: usize,
    /// Number of polarisation combinations in the visibilities e.g. XX,XY,YX,YY == 4
    pub num_visibility_pols: usize,
    /// Correlator mode dump time
    pub integration_time_milliseconds: u64,
    /// Number of coarse channels
    pub num_coarse_channels: usize,
    /// Total bandwidth of observation (of the coarse channels we have)
    pub bandwidth_hz: u32,
    /// Bandwidth of each coarse channel
    pub coarse_channel_width_hz: u32,
    /// Correlator fine_channel_resolution
    pub fine_channel_width_hz: u32,
    /// Number of fine channels in each coarse channel
    pub num_fine_channels_per_coarse: usize,
    /// The number of bytes taken up by a scan/timestep in each gpubox file.
    pub num_timestep_coarse_channel_bytes: usize,
    /// The number of floats in each gpubox HDU.
    pub num_timestep_coarse_channel_floats: usize,
    /// This is the number of gpubox files *per batch*.
    pub num_gpubox_files: usize,
}

/// This returns a struct containing the `CorrelatorContext` metadata
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
///
/// * `out_correaltor_metadata_ptr` - A Rust-owned populated `CorrelatorMetadata` struct. Free with `mwalib_correlator_metadata_free`.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated `CorrelatorContext` object from the `mwalib_correlator_context_new` function.
/// * Caller must call `mwalib_correlator_metadata_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_metadata_get(
    correlator_context_ptr: *mut CorrelatorContext,
    out_correlator_metadata_ptr: &mut *mut CorrelatorMetadata,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    if correlator_context_ptr.is_null() {
        set_error_message(
            "mwalib_correlator_metadata_get() ERROR: Warning: null pointer for correlator_context_ptr passed in",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    }
    // Get the correlator context object from the raw pointer passed in
    let context = &*correlator_context_ptr;

    // Populate the rust owned data structure with data from the correlator context
    let out_context = CorrelatorMetadata {
        corr_version: context.corr_version,
        start_unix_time_milliseconds: context.start_unix_time_milliseconds,
        end_unix_time_milliseconds: context.end_unix_time_milliseconds,
        duration_milliseconds: context.duration_milliseconds,
        num_timesteps: context.num_timesteps,
        num_baselines: context.num_baselines,
        num_visibility_pols: context.num_visibility_pols,
        num_coarse_channels: context.num_coarse_channels,
        integration_time_milliseconds: context.integration_time_milliseconds,
        fine_channel_width_hz: context.fine_channel_width_hz,
        bandwidth_hz: context.bandwidth_hz,
        coarse_channel_width_hz: context.coarse_channel_width_hz,
        num_fine_channels_per_coarse: context.num_fine_channels_per_coarse,
        num_timestep_coarse_channel_bytes: context.num_timestep_coarse_channel_bytes,
        num_timestep_coarse_channel_floats: context.num_timestep_coarse_channel_floats,
        num_gpubox_files: context.num_gpubox_files,
    };

    // Pass out the pointer to the rust owned data structure
    *out_correlator_metadata_ptr = Box::into_raw(Box::new(out_context));

    // Return success
    0
}

/// Free a previously-allocated `CorrelatorMetadata` struct.
///
/// # Arguments
///
/// * `correlator_metadata_ptr` - pointer to an already populated `CorrelatorMetadata` object
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `CorrelatorMetadata` object
/// * `correlator_metadata_ptr` must point to a populated `CorrelatorMetadata` object from the `mwalib_correlator_metadata_get` function.
/// * `correlator_metadata_ptr` must not have already been freed.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_metadata_free(
    correlator_metadata_ptr: *mut CorrelatorMetadata,
) -> i32 {
    if correlator_metadata_ptr.is_null() {
        return 0;
    }
    drop(Box::from_raw(correlator_metadata_ptr));

    // Return success
    0
}

///
/// C Representation of the `VoltageContext` metadata
///
#[repr(C)]
pub struct mwalibVoltageMetadata {
    /// Version of the correlator format
    pub corr_version: CorrelatorVersion,
    /// The proper start of the observation (the time that is common to all
    /// provided voltage files).
    pub start_gps_time_milliseconds: u64,
    /// `end_gps_time_milliseconds` is the actual end time of the observation    
    /// i.e. start time of last common timestep plus length of a voltage file (1 sec for MWA Legacy, 8 secs for MWAX).
    pub end_gps_time_milliseconds: u64,
    /// Total duration of observation (based on voltage files)
    pub duration_milliseconds: u64,
    /// Number of timesteps in the observation
    pub num_timesteps: usize,
    /// Number of coarse channels after we've validated the input voltage files
    pub num_coarse_channels: usize,
    /// Total bandwidth of observation (of the coarse channels we have)
    pub bandwidth_hz: u32,
    /// Bandwidth of each coarse channel
    pub coarse_channel_width_hz: u32,
    /// Volatge fine_channel_resolution (if applicable- MWA legacy is 10 kHz, MWAX is unchannelised i.e. the full coarse channel width)
    pub fine_channel_width_hz: u32,
    /// Number of fine channels in each coarse channel
    pub num_fine_channels_per_coarse: usize,
}

/// This returns a struct containing the `VoltageContext` metadata
///
/// # Arguments
///
/// * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object.
///
/// * `out_voltage_metadata_ptr` - A Rust-owned populated `CorrelatorMetadata` struct. Free with `mwalib_correlator_metadata_free`.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated `VoltageContext` object from the `mwalib_correlator_context_new` function.
/// * Caller must call `mwalib_correlator_metadata_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_voltage_metadata_get(
    voltage_context_ptr: *mut VoltageContext,
    out_voltage_metadata_ptr: &mut *mut mwalibVoltageMetadata,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    if voltage_context_ptr.is_null() {
        set_error_message(
            "mwalib_voltage_metadata_get() ERROR: Warning: null pointer for voltage_context_ptr passed in",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    }
    // Get the voltage context object from the raw pointer passed in
    let context = &*voltage_context_ptr;

    // Populate the rust owned data structure with data from the voltage context
    let out_context = mwalibVoltageMetadata {
        corr_version: context.corr_version,
        start_gps_time_milliseconds: context.start_gps_time_milliseconds,
        end_gps_time_milliseconds: context.end_gps_time_milliseconds,
        duration_milliseconds: context.duration_milliseconds,
        num_timesteps: context.num_timesteps,
        num_coarse_channels: context.num_coarse_channels,
        bandwidth_hz: context.bandwidth_hz,
        coarse_channel_width_hz: context.coarse_channel_width_hz,
        fine_channel_width_hz: context.fine_channel_width_hz,
        num_fine_channels_per_coarse: context.num_fine_channels_per_coarse,
    };

    // Pass out the pointer to the rust owned data structure
    *out_voltage_metadata_ptr = Box::into_raw(Box::new(out_context));

    // Return success
    0
}

/// Free a previously-allocated `mwalibVoltageMetadata` struct.
///
/// # Arguments
///
/// * `voltage_metadata_ptr` - pointer to an already populated `mwalibVoltageMetadata` object
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `mwalibVoltageMetadata` object
/// * `voltage_metadata_ptr` must point to a populated `mwalibVoltageMetadata` object from the `mwalib_voltage_metadata_get` function.
/// * `voltage_metadata_ptr` must not have already been freed.
#[no_mangle]
pub unsafe extern "C" fn mwalib_voltage_metadata_free(
    voltage_metadata_ptr: *mut mwalibVoltageMetadata,
) -> i32 {
    if voltage_metadata_ptr.is_null() {
        return 0;
    }
    drop(Box::from_raw(voltage_metadata_ptr));

    // Return success
    0
}

/// Representation in C of an `Antenna` struct
#[repr(C)]
pub struct Antenna {
    /// This is the antenna number.
    /// Nominally this is the field we sort by to get the desired output order of antenna.
    /// X and Y have the same antenna number. This is the sorted ordinal order of the antenna.None
    /// e.g. 0...N-1
    pub antenna: u32,
    /// Numeric part of tile_name for the antenna. Each pol has the same value
    /// e.g. tile_name "tile011" hsa tile_id of 11
    pub tile_id: u32,
    /// Human readable name of the antenna
    /// X and Y have the same name
    pub tile_name: *mut c_char,
}

/// This passes back an array of structs containing all antennas given a metafits OR correlator context.
///
/// # Arguments
///
/// * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object. (Exclusive with `correlator_context_ptr` and `voltage_context_ptr`)
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object. (Exclusive with `metafits_context_ptr` and `voltage_context_ptr`)
///
/// * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object. (Exclusive with `metafits_context_ptr` and `correlator_context_ptr`)
///
/// * `out_antennas_ptr` - A Rust-owned populated array of `Antenna` struct. Free with `mwalib_antennas_free`.
///
/// * `out_antennas_len` - Antennas array length.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `metafits_context_ptr` must point to a populated MetafitsContext object from the `mwalib_metafits_context_new` function.
/// * Caller must call `mwalib_antenna_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_antennas_get(
    metafits_context_ptr: *mut MetafitsContext,
    correlator_context_ptr: *mut CorrelatorContext,
    voltage_context_ptr: *mut VoltageContext,
    out_antennas_ptr: &mut *mut Antenna,
    out_antennas_len: &mut size_t,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    // Ensure only either metafits XOR correlator XOR voltage context is passed in
    if !(!metafits_context_ptr.is_null()
        ^ !correlator_context_ptr.is_null()
        ^ !voltage_context_ptr.is_null())
    {
        set_error_message(
            "mwalib_antennas_get() ERROR: pointers for metafits_context_ptr, correlator_context_ptr and/or voltage_context_ptr were passed in. Only one should be provided.",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    }
    // Create our metafits context pointer depending on what was passed in
    let metafits_context = {
        if !metafits_context_ptr.is_null() {
            // Caller passed in a metafits context, so use that
            &*metafits_context_ptr
        } else {
            if !correlator_context_ptr.is_null() {
                // Caller passed in a correlator context, so use that
                &(&*correlator_context_ptr).metafits_context
            } else {
                // Caller passed in a voltage context, so use that
                &(&*voltage_context_ptr).metafits_context
            }
        }
    };

    let mut item_vec: Vec<Antenna> = Vec::new();

    for item in metafits_context.antennas.iter() {
        let out_item = Antenna {
            antenna: item.antenna,
            tile_id: item.tile_id,
            tile_name: CString::new(String::from(&item.tile_name))
                .unwrap()
                .into_raw(),
        };

        item_vec.push(out_item);
    }

    // Pass back the array and length of the array
    *out_antennas_ptr = array_to_ffi_boxed_slice(item_vec);
    *out_antennas_len = metafits_context.antennas.len();

    // Return success
    0
}

/// Free a previously-allocated `Antenna` array of structs.
///
/// # Arguments
///
/// * `antennas_ptr` - pointer to an already populated `Antenna` array
///
/// * `antennas_len` - number of elements in the pointed to array
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `Antenna` array
/// * `antenna_ptr` must point to a populated `Antenna` array from the `mwalib_antennas_get` function.
/// * `antenna_ptr` must not have already been freed.
#[no_mangle]
pub unsafe extern "C" fn mwalib_antennas_free(
    antennas_ptr: *mut Antenna,
    antennas_len: size_t,
) -> i32 {
    if antennas_ptr.is_null() {
        return 0;
    }

    // Extract a slice from the pointer
    let slice: &mut [Antenna] = slice::from_raw_parts_mut(antennas_ptr, antennas_len);
    // Now for each item we need to free anything on the heap
    for i in slice.into_iter() {
        drop(Box::from_raw(i.tile_name));
    }

    // Free the memory for the slice
    drop(Box::from_raw(slice));

    // Return success
    0
}

///
/// C Representation of a `Baseline` struct
///
#[repr(C)]
pub struct Baseline {
    /// Index in the `MetafitsContext` antenna array for antenna1 for this baseline
    pub antenna1_index: usize,
    /// Index in the `MetafitsContext` antenna array for antenna2 for this baseline
    pub antenna2_index: usize,
}

/// This passes a pointer to an array of baselines
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
///
/// * `out_baselines_ptr` - populated, array of rust-owned baseline structs. Free with `mwalib_baselines_free`.
///
/// * `out_baselines_len` - baseline array length.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated `CorrelatorContext` object from the `mwalib_correlator_context_new` function.
/// * Caller must call `mwalib_baselines_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_baselines_get(
    correlator_context_ptr: *mut CorrelatorContext,
    out_baselines_ptr: &mut *mut Baseline,
    out_baselines_len: &mut size_t,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    if correlator_context_ptr.is_null() {
        set_error_message(
            "mwalib_correlator_baselines_get() ERROR: null pointer for correlator_context_ptr passed in",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    }

    let context = &*correlator_context_ptr;

    let mut item_vec: Vec<Baseline> = Vec::new();

    for item in context.baselines.iter() {
        let out_item = Baseline {
            antenna1_index: item.antenna1_index,
            antenna2_index: item.antenna2_index,
        };

        item_vec.push(out_item);
    }

    // Pass back the array and length of the array
    *out_baselines_ptr = array_to_ffi_boxed_slice(item_vec);
    *out_baselines_len = context.baselines.len();

    return 0;
}

/// Free a previously-allocated `Baseline` struct.
///
/// # Arguments
///
/// * `baselines_ptr` - pointer to an already populated `Baseline` array
///
/// * `baselines_len` - number of elements in the pointed to array
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `Baseline` array
/// * `baseline_ptr` must point to a populated `Baseline` array from the `mwalib_baselines_get` function.
/// * `baseline_ptr` must not have already been freed.
#[no_mangle]
pub unsafe extern "C" fn mwalib_baselines_free(
    baselines_ptr: *mut Baseline,
    baselines_len: size_t,
) -> i32 {
    if baselines_ptr.is_null() {
        return 0;
    }
    // Extract a slice from the pointer
    let slice: &mut [Baseline] = slice::from_raw_parts_mut(baselines_ptr, baselines_len);

    // Free the memory for the slice
    drop(Box::from_raw(slice));

    // Return success
    0
}

/// Representation in C of an `CoarseChannel` struct
#[repr(C)]
pub struct CoarseChannel {
    /// Correlator channel is 0 indexed (0..N-1)
    pub correlator_channel_number: usize,
    /// Receiver channel is 0-255 in the RRI recivers
    pub receiver_channel_number: usize,
    /// gpubox channel number
    /// Legacy e.g. obsid_datetime_gpuboxXX_00
    /// v2     e.g. obsid_datetime_gpuboxXXX_00
    pub gpubox_number: usize,
    /// Width of a coarse channel in Hz
    pub channel_width_hz: u32,
    /// Starting frequency of coarse channel in Hz
    pub channel_start_hz: u32,
    /// Centre frequency of coarse channel in Hz
    pub channel_centre_hz: u32,
    /// Ending frequency of coarse channel in Hz
    pub channel_end_hz: u32,
}

/// This passes a pointer to an array of correlator coarse channel
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
///
/// * `out_coarse_channels_ptr` - A Rust-owned populated `CoarseChannel` array of structs. Free with `mwalib_coarse_channels_free`.
///
/// * `out_coarse_channels_len` - Coarse channel array length.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated `mwalibCorrelatorContext` object from the `mwalib_correlator_context_new` function.
/// * Caller must call `mwalib_coarse_channels_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_coarse_channels_get(
    correlator_context_ptr: *mut CorrelatorContext,
    out_coarse_channels_ptr: &mut *mut CoarseChannel,
    out_coarse_channels_len: &mut size_t,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    if correlator_context_ptr.is_null() {
        set_error_message(
            "mwalib_correlator_coarse_channels_get() ERROR: null pointer for correlator_context_ptr passed in",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    }
    let context = &*correlator_context_ptr;

    let mut item_vec: Vec<CoarseChannel> = Vec::new();

    for item in context.coarse_channels.iter() {
        let out_item = CoarseChannel {
            correlator_channel_number: item.correlator_channel_number,
            receiver_channel_number: item.receiver_channel_number,
            gpubox_number: item.gpubox_number,
            channel_width_hz: item.channel_width_hz,
            channel_start_hz: item.channel_start_hz,
            channel_centre_hz: item.channel_centre_hz,
            channel_end_hz: item.channel_end_hz,
        };

        item_vec.push(out_item);
    }

    // Pass back the array and length of the array
    *out_coarse_channels_ptr = array_to_ffi_boxed_slice(item_vec);
    *out_coarse_channels_len = context.coarse_channels.len();

    // return success
    0
}

/// This passes a pointer to an array of voltage coarse channel
///
/// # Arguments
///
/// * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object.
///
/// * `out_coarse_channels_ptr` - A Rust-owned populated `CoarseChannel` array of structs. Free with `mwalib_coarse_channels_free`.
///
/// * `out_coarse_channels_len` - Coarse channel array length.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `voltage_context_ptr` must point to a populated `mwalibVoltageContext` object from the `mwalib_voltage_context_new` function.
/// * Caller must call `mwalib_coarse_channels_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_voltage_coarse_channels_get(
    voltage_context_ptr: *mut VoltageContext,
    out_coarse_channels_ptr: &mut *mut CoarseChannel,
    out_coarse_channels_len: &mut usize,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    if voltage_context_ptr.is_null() {
        set_error_message(
            "mwalib_voltage_coarse_channels_get() ERROR: null pointer for voltage_context_ptr passed in",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    }
    let context = &*voltage_context_ptr;

    let mut item_vec: Vec<CoarseChannel> = Vec::new();

    for item in context.coarse_channels.iter() {
        let out_item = CoarseChannel {
            correlator_channel_number: item.correlator_channel_number,
            receiver_channel_number: item.receiver_channel_number,
            gpubox_number: item.gpubox_number,
            channel_width_hz: item.channel_width_hz,
            channel_start_hz: item.channel_start_hz,
            channel_centre_hz: item.channel_centre_hz,
            channel_end_hz: item.channel_end_hz,
        };

        item_vec.push(out_item);
    }

    // Pass back the array and length of the array
    *out_coarse_channels_ptr = array_to_ffi_boxed_slice(item_vec);
    *out_coarse_channels_len = context.coarse_channels.len();

    // return success
    0
}

/// Free a previously-allocated `CoarseChannel` struct.
///
/// # Arguments
///
/// * `coarse_channels_ptr` - pointer to an already populated `CoarseChannel` array
///
/// * `coarse_channels_len` - number of elements in the pointed to array
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `CoarseChannel` array
/// * `coarse_channel_ptr` must point to a populated `CoarseChannel` array from the `mwalib_correlator_coarse_channels_get` function.
/// * `coarse_channel_ptr` must not have already been freed.
#[no_mangle]
pub unsafe extern "C" fn mwalib_coarse_channels_free(
    coarse_channels_ptr: *mut CoarseChannel,
    coarse_channels_len: size_t,
) -> i32 {
    if coarse_channels_ptr.is_null() {
        return 0;
    }
    // Extract a slice from the pointer
    let slice: &mut [CoarseChannel] =
        slice::from_raw_parts_mut(coarse_channels_ptr, coarse_channels_len);
    // Free the memory for the slice
    drop(Box::from_raw(slice));

    // Return success
    0
}

/// Representation in C of an `RFInput` struct
#[repr(C)]
pub struct RFInput {
    /// This is the metafits order (0-n inputs)
    pub input: u32,
    /// This is the antenna number.
    /// Nominally this is the field we sort by to get the desired output order of antenna.
    /// X and Y have the same antenna number. This is the sorted ordinal order of the antenna.None
    /// e.g. 0...N-1
    pub antenna: u32,
    /// Numeric part of tile_name for the antenna. Each pol has the same value
    /// e.g. tile_name "tile011" hsa tile_id of 11
    pub tile_id: u32,
    /// Human readable name of the antenna
    /// X and Y have the same name
    pub tile_name: *mut c_char,
    /// Polarisation - X or Y
    pub pol: *mut c_char,
    /// Electrical length in metres for this antenna and polarisation to the receiver
    pub electrical_length_m: f64,
    /// Antenna position North from the array centre (metres)
    pub north_m: f64,
    /// Antenna position East from the array centre (metres)
    pub east_m: f64,
    /// Antenna height from the array centre (metres)
    pub height_m: f64,
    /// AKA PFB to correlator input order (only relevant for pre V2 correlator)
    pub vcs_order: u32,
    /// Subfile order is the order in which this rf_input is desired in our final output of data
    pub subfile_order: u32,
    /// Is this rf_input flagged out (due to tile error, etc from metafits)
    pub flagged: bool,
    /// Receiver number
    pub receiver_number: u32,
    /// Receiver slot number
    pub receiver_slot_number: u32,
}

/// This passes a pointer to an array of antenna given a metafits context OR correlator context
///
/// # Arguments
///
/// * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object. (Exclusive with `correlator_context_ptr` and `voltage_context_ptr`)
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object. (Exclusive with `metafits_context_ptr` and `voltage_context_ptr`)
///
/// * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object. (Exclusive with `metafits_context_ptr` and `correlator_context_ptr`)
///
/// * `out_rfinputs_ptr` - A Rust-owned populated `RFInput` array of structs. Free with `mwalib_rfinputs_free`.
///
/// * `out_rfinputs_len` - rfinputs array length.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `metafits_context_ptr` must point to a populated `MetafitsContext` object from the `mwalib_metafits_context_new` function.
/// * Caller must call `mwalib_rfinputs_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_rfinputs_get(
    metafits_context_ptr: *mut MetafitsContext,
    correlator_context_ptr: *mut CorrelatorContext,
    voltage_context_ptr: *mut VoltageContext,
    out_rfinputs_ptr: &mut *mut RFInput,
    out_rfinputs_len: &mut size_t,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    // Ensure only either metafits XOR correlator XOR voltage context is passed in
    if !(!metafits_context_ptr.is_null()
        ^ !correlator_context_ptr.is_null()
        ^ !voltage_context_ptr.is_null())
    {
        set_error_message(
            "mwalib_rfinputs_get() ERROR: pointers for metafits_context_ptr, correlator_context_ptr and/or voltage_context_ptr were passed in. Only one should be provided.",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    }
    // Create our metafits context pointer depending on what was passed in
    let metafits_context = {
        if !metafits_context_ptr.is_null() {
            // Caller passed in a metafits context, so use that
            &*metafits_context_ptr
        } else {
            if !correlator_context_ptr.is_null() {
                // Caller passed in a correlator context, so use that
                &(&*correlator_context_ptr).metafits_context
            } else {
                // Caller passed in a voltage context, so use that
                &(&*voltage_context_ptr).metafits_context
            }
        }
    };

    let mut item_vec: Vec<RFInput> = Vec::new();

    for item in metafits_context.rf_inputs.iter() {
        let out_item = RFInput {
            input: item.input,
            antenna: item.antenna,
            tile_id: item.tile_id,
            tile_name: CString::new(String::from(&item.tile_name))
                .unwrap()
                .into_raw(),
            pol: CString::new(item.pol.to_string()).unwrap().into_raw(),
            electrical_length_m: item.electrical_length_m,
            north_m: item.north_m,
            east_m: item.east_m,
            height_m: item.height_m,
            vcs_order: item.vcs_order,
            subfile_order: item.subfile_order,
            flagged: item.flagged,
            receiver_number: item.receiver_number,
            receiver_slot_number: item.receiver_slot_number,
        };

        item_vec.push(out_item);
    }

    // Pass back the array and length of the array
    *out_rfinputs_ptr = array_to_ffi_boxed_slice(item_vec);
    *out_rfinputs_len = metafits_context.rf_inputs.len();

    // Return success
    0
}

/// Free a previously-allocated `RFInput` struct.
///
/// # Arguments
///
/// * `rf_inputs_ptr` - pointer to an already populated `RFInput` object
///
/// * `rf_inputs_len` - number of elements in the pointed to array
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `RFInput` array
/// * `rf_input_ptr` must point to a populated `RFInput` array from the `mwalib_rfinputs_get` function.
/// * `rf_input_ptr` must not have already been freed.
#[no_mangle]
pub unsafe extern "C" fn mwalib_rfinputs_free(
    rf_inputs_ptr: *mut RFInput,
    rf_inputs_len: size_t,
) -> i32 {
    if rf_inputs_ptr.is_null() {
        return 0;
    }
    // Extract a slice from the pointer
    let slice: &mut [RFInput] = slice::from_raw_parts_mut(rf_inputs_ptr, rf_inputs_len);
    // Now for each item we need to free anything on the heap
    for i in slice.into_iter() {
        drop(Box::from_raw(i.tile_name));
        drop(Box::from_raw(i.pol));
    }

    // Free the memory for the slice
    drop(Box::from_raw(slice));

    // Return success
    0
}

///
/// C Representation of a `TimeStep` struct
///
#[repr(C)]
pub struct TimeStep {
    /// UNIX time (in milliseconds to avoid floating point inaccuracy)
    pub unix_time_milliseconds: u64,
    pub gps_time_milliseconds: u64,
}

/// This passes a pointer to an array of timesteps
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
///
/// * `out_timesteps_ptr` - A Rust-owned populated `TimeStep` struct. Free with `mwalib_timestep_free`.
///
/// * `out_timesteps_len` - Timesteps array length.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated `CorrelatorContext` object from the `mwalib_correlator_context_new` function.
/// * Caller must call `mwalib_timestep_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_timesteps_get(
    correlator_context_ptr: *mut CorrelatorContext,
    out_timesteps_ptr: &mut *mut TimeStep,
    out_timesteps_len: &mut size_t,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    if correlator_context_ptr.is_null() {
        set_error_message(
            "mwalib_correlator_timesteps_get() ERROR: null pointer for correlator_context_ptr passed in",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    }
    let context = &*correlator_context_ptr;

    let mut item_vec: Vec<TimeStep> = Vec::new();

    for item in context.timesteps.iter() {
        let out_item = TimeStep {
            unix_time_milliseconds: item.unix_time_milliseconds,
            gps_time_milliseconds: item.gps_time_milliseconds,
        };

        item_vec.push(out_item);
    }

    // Pass back the array and length of the array
    *out_timesteps_ptr = array_to_ffi_boxed_slice(item_vec);
    *out_timesteps_len = context.timesteps.len();

    // Return success
    0
}

/// This passes a pointer to an array of timesteps
///
/// # Arguments
///
/// * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object.
///
/// * `out_timesteps_ptr` - A Rust-owned populated `TimeStep` struct. Free with `mwalib_timestep_free`.
///
/// * `out_timesteps_len` - Timesteps array length.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `voltage_context_ptr` must point to a populated `VoltageContext` object from the `mwalib_voltage_context_new` function.
/// * Caller must call `mwalib_timestep_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_voltage_timesteps_get(
    voltage_context_ptr: *mut VoltageContext,
    out_timesteps_ptr: &mut *mut TimeStep,
    out_timesteps_pols_len: &mut size_t,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    if voltage_context_ptr.is_null() {
        set_error_message(
            "mwalib_voltage_timesteps_get() ERROR: null pointer for voltage_context_ptr passed in",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    }
    let context = &*voltage_context_ptr;

    let mut item_vec: Vec<TimeStep> = Vec::new();

    for item in context.timesteps.iter() {
        let out_item = TimeStep {
            unix_time_milliseconds: item.unix_time_milliseconds,
            gps_time_milliseconds: item.gps_time_milliseconds,
        };

        item_vec.push(out_item);
    }

    // Pass back the array and length of the array
    *out_timesteps_ptr = array_to_ffi_boxed_slice(item_vec);
    *out_timesteps_pols_len = context.timesteps.len();

    // Return success
    0
}

/// Free a previously-allocated `TimeStep` struct.
///
/// # Arguments
///
/// * `timesteps_ptr` - pointer to an already populated `TimeStep` array
///
/// * `timesteps_len` - number of elements in the pointed to array
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `TimeStep` array
/// * `timestep_ptr` must point to a populated `TimeStep` array from the `mwalib_correlator_timesteps_get` function.
/// * `timestep_ptr` must not have already been freed.
#[no_mangle]
pub unsafe extern "C" fn mwalib_timesteps_free(
    timesteps_ptr: *mut TimeStep,
    timesteps_len: size_t,
) -> i32 {
    if timesteps_ptr.is_null() {
        return 0;
    }
    // Extract a slice from the pointer
    let slice: &mut [TimeStep] = slice::from_raw_parts_mut(timesteps_ptr, timesteps_len);
    // Free the memory for the slice
    drop(Box::from_raw(slice));

    // Return success
    0
}

///
/// C Representation of a `VisibilityPol` struct
///
#[repr(C)]
pub struct VisibilityPol {
    /// Polarisation (e.g. "XX" or "XY" or "YX" or "YY")
    pub polarisation: *mut c_char,
}

/// This passes back a pointer to an array of all visibility polarisations
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
///
/// * `out_visibility_pols_ptr` - A Rust-owned populated array of `VisibilityPol` structs. Free with `mwalib_visibility_pols_free`.
///
/// * `out_visibility_pols_len` - Visibility Pols array length.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated `CorrelatorContext` object from the `mwalib_correlator_context_new` function.
/// * Caller must call `mwalib_visibility_pols_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_visibility_pols_get(
    correlator_context_ptr: *mut CorrelatorContext,
    out_visibility_pols_ptr: &mut *mut VisibilityPol,
    out_visibility_pols_len: &mut size_t,
    error_message: *const c_char,
    error_message_length: size_t,
) -> i32 {
    if correlator_context_ptr.is_null() {
        set_error_message(
            "mwalib_correlator_visibility_pols_get() ERROR: null pointer for correlator_context_ptr passed in",
            error_message as *mut u8,
            error_message_length,
        );
        return 1;
    }
    let context = &*correlator_context_ptr;
    let mut item_vec: Vec<VisibilityPol> = Vec::new();

    for item in context.visibility_pols.iter() {
        let out_visibility_pol = VisibilityPol {
            polarisation: CString::new(String::from(&item.polarisation))
                .unwrap()
                .into_raw(),
        };

        item_vec.push(out_visibility_pol);
    }

    // Pass back the array and length of the array
    *out_visibility_pols_ptr = array_to_ffi_boxed_slice(item_vec);
    *out_visibility_pols_len = context.visibility_pols.len();

    // Return success
    0
}

/// Free a previously-allocated `VisibilityPol` array of structs.
///
/// # Arguments
///
/// * `visibility_pols_ptr` - pointer to an already populated `VisibilityPol` array
///
/// * `visibility_pols_len` - number of elements in the pointed to array
///
/// # Returns
///
/// * 0 on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `VisibilityPol` array
/// * `visibility_pols_ptr` must point to a populated `VisibilityPol` array from the `mwalib_correlator_visibility_pols_get` function.
/// * `visibility_pols_ptr` must not have already been freed.
#[no_mangle]
pub unsafe extern "C" fn mwalib_visibility_pols_free(
    visibility_pols_ptr: *mut VisibilityPol,
    visibility_pols_len: size_t,
) -> i32 {
    // Just return 0 if the pointer is already null
    if visibility_pols_ptr.is_null() {
        return 0;
    }
    // Extract a slice from the pointer
    let slice: &mut [VisibilityPol] =
        slice::from_raw_parts_mut(visibility_pols_ptr, visibility_pols_len);
    // Now for each item we need to free anything on the heap
    for i in slice.into_iter() {
        drop(Box::from_raw(i.polarisation));
    }

    // Free the memory for the slice
    drop(Box::from_raw(slice));

    // Return success
    0
}

#[cfg(test)]
mod test;
