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
use libc::{c_char, c_float, c_longlong, size_t};
use std::ffi::*;
use std::ptr;
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
    let error_message_bytes = error_message.as_bytes_with_nul();

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
/// * Nothing
///
/// # Safety
/// * rust_cstring must not have already been freed and must point to a Rust string.
#[no_mangle]
#[cfg(not(tarpaulin_include))]
pub unsafe extern "C" fn mwalib_free_rust_cstring(rust_cstring: *mut c_char) {
    // Don't do anything if the pointer is null.
    if rust_cstring.is_null() {
        return;
    }
    CString::from_raw(rust_cstring);
}

/// Create and return a pointer to an `MetafitsContext` struct given only a metafits file
///
/// # Arguments
///
/// * `metafits_filename` - pointer to char* buffer containing the full path and filename of a metafits file.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * A Rust-owned populated `MetafitsContext` pointer or NULL if there was an error (check error_message)
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated `char*` buffer for any error messages.
/// * Caller *must* call the `mwalib_metafits_context_free` function to release the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_context_new(
    metafits_filename: *const c_char,
    error_message: *mut u8,
    error_message_length: size_t,
) -> *mut MetafitsContext {
    let m = CStr::from_ptr(metafits_filename)
        .to_str()
        .unwrap()
        .to_string();
    let context = match MetafitsContext::new(&m) {
        Ok(c) => c,
        Err(e) => {
            set_error_message(&format!("{}", e), error_message, error_message_length);
            return ptr::null_mut();
        }
    };

    Box::into_raw(Box::new(context))
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
/// * 0 on success, 1 on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `metafits_context_ptr` must contain an MetafitsContext object already populated via `mwalib_metafits_context_new`
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_context_display(
    metafits_context_ptr: *const MetafitsContext,
    error_message: *mut u8,
    error_message_length: size_t,
) -> i32 {
    if metafits_context_ptr.is_null() {
        set_error_message(
            "mwalib_metafits_context_display() ERROR: null pointer for metafits_context_ptr passed in",
            error_message,
            error_message_length,
        );
        return 1;
    }

    let context = &*metafits_context_ptr;

    println!("{}", context);
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
/// * Nothing
///
///
/// # Safety
/// * This must be called once caller is finished with the `MetafitsContext` object
/// * `metafits_context_ptr` must point to a populated mwalibContext object from the `mwalib_metafits_context_new` functions.
/// * `metafits_context_ptr` must not have already been freed.
#[no_mangle]
#[cfg(not(tarpaulin_include))]
pub unsafe extern "C" fn mwalib_metafits_context_free(metafits_context_ptr: *mut MetafitsContext) {
    if metafits_context_ptr.is_null() {
        return;
    }
    // Release correlator context if applicable
    Box::from_raw(metafits_context_ptr);
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
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * A Rust-owned populated `CorrelatorContext` pointer or NULL if there was an error (check error_message)
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
    error_message: *mut u8,
    error_message_length: size_t,
) -> *mut CorrelatorContext {
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
            set_error_message(&format!("{}", e), error_message, error_message_length);
            return ptr::null_mut();
        }
    };
    Box::into_raw(Box::new(context))
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
/// * 0 on success, 1 on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must contain an `CorrelatorContext` object already populated via `mwalib_correlator_context_new`
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_context_display(
    correlator_context_ptr: *const CorrelatorContext,
    error_message: *mut u8,
    error_message_length: size_t,
) -> i32 {
    if correlator_context_ptr.is_null() {
        set_error_message(
            "mwalib_correlator_context() ERROR: null pointer for correlator_context_ptr passed in",
            error_message,
            error_message_length,
        );
        return 1;
    }

    let context = &*correlator_context_ptr;

    println!("{}", context);
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
///                      to mwalibTimeStep.get(context, N) where N is timestep_index.
///
/// * `coarse_channel_index` - index within the coarse_channel array for the desired coarse channel. This corresponds
///                            to mwalibCoarseChannel.get(context, N) where N is coarse_channel_index.
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
/// * 0 on success, 1 on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated object from the `mwalib_correlator_context_new` function.
/// * Caller *must* call `mwalib_correlator_context_free_read_buffer` function to release the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_context_read_by_baseline(
    correlator_context_ptr: *mut CorrelatorContext,
    timestep_index: usize,
    coarse_channel_index: usize,
    buffer_ptr: *mut c_float,
    buffer_len: size_t,
    error_message: *mut u8,
    error_message_length: size_t,
) -> i32 {
    // Load the previously-initialised context and buffer structs. Exit if
    // either of these are null.
    let corr_context = if correlator_context_ptr.is_null() {
        set_error_message(
            "mwalib_correlator_context_read_by_baseline() ERROR: null pointer for correlator_context_ptr passed in",
            error_message,
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
            set_error_message(&format!("{}", e), error_message, error_message_length);
            return 1;
        }
    };

    // If the data buffer is empty, then just return a null pointer.
    if data.is_empty() {
        set_error_message(
            "mwalib_correlator_context_read_by_baseline() ERROR: no data was returned.",
            error_message,
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
///                      to mwalibTimeStep.get(context, N) where N is timestep_index.
///
/// * `coarse_channel_index` - index within the coarse_channel array for the desired coarse channel. This corresponds
///                            to mwalibCoarseChannel.get(context, N) where N is coarse_channel_index.
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
/// * 0 on success, 1 on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated object from the `mwalib_correlator_context_new` function.
/// * Caller *must* call `mwalib_correlator_context_free_read_buffer` function to release the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_context_read_by_frequency(
    correlator_context_ptr: *mut CorrelatorContext,
    timestep_index: usize,
    coarse_channel_index: usize,
    buffer_ptr: *mut c_float,
    buffer_len: size_t,
    error_message: *mut u8,
    error_message_length: size_t,
) -> i32 {
    // Load the previously-initialised context and buffer structs. Exit if
    // either of these are null.
    let corr_context = if correlator_context_ptr.is_null() {
        set_error_message(
            "mwalib_correlator_context_read_by_frequency() ERROR: null pointer for correlator_context_ptr passed in",
            error_message,
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
            set_error_message(&format!("{}", e), error_message, error_message_length);
            return 1;
        }
    };

    // If the data buffer is empty, then just return a null pointer.
    if data.is_empty() {
        set_error_message(
            "mwalib_correlator_context_read_by_frequency() ERROR: no data was returned.",
            error_message,
            error_message_length,
        );
        return 1;
    }

    // Populate the buffer which was provided to us by caller
    output_slice[..data.len()].copy_from_slice(data.as_slice());
    // Return Success
    0
}

/// Free a previously-allocated float* created by `mwalib_correlator_context_read_by_baseline` or `mwalib_correlator_context_read_by_frequency`.
///
/// Python can't free memory itself, so this is useful for Python (and perhaps
/// other languages).
///
/// # Arguments
///
/// * `read_buffer_ptr` - pointer to an already populated read buffer object.
///
/// * `read_buffer_len` - length of read buffer.
///
///
/// # Returns
///
/// * Nothing
///
///
/// # Safety
/// * This must be called once caller is finished with the float buffer object
/// * `read_buffer_ptr` must point to a populated float buffer from the
///   `mwalib_correlator_context_read_by_baseline` or `mwalib_correlator_context_read_by_frequency function`.
/// * `read_buffer_ptr` must not have already been freed.
#[no_mangle]
#[cfg(not(tarpaulin_include))]
pub unsafe extern "C" fn mwalib_correlator_context_free_read_buffer(
    read_buffer_ptr: *mut c_float,
    read_buffer_len: *const c_longlong,
) {
    if read_buffer_ptr.is_null() {
        return;
    }

    drop(Vec::from_raw_parts(
        read_buffer_ptr,
        *read_buffer_len as usize,
        *read_buffer_len as usize,
    ));
}

/// Free a previously-allocated `CorrelatorContext` struct (and it's members).
///
/// # Arguments
///
/// * `context_ptr` - pointer to an already populated `CorrelatorContext` object
///
///
/// # Returns
///
/// * Nothing
///
///
/// # Safety
/// * This must be called once caller is finished with the `CorrelatorContext` object
/// * `correlator_context_ptr` must point to a populated `CorrelatorContext` object from the `mwalib_correlator_context_new` function.
/// * `correlator_context_ptr` must not have already been freed.
#[no_mangle]
#[cfg(not(tarpaulin_include))]
pub unsafe extern "C" fn mwalib_correlator_context_free(
    correlator_context_ptr: *mut CorrelatorContext,
) {
    if correlator_context_ptr.is_null() {
        return;
    }
    // Release correlator context if applicable
    Box::from_raw(correlator_context_ptr);
}

///
/// This a C struct to allow the caller to consume the metafits metadata
///
#[repr(C)]
pub struct mwalibMetafitsMetadata {
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

/// This returns a struct containing the `MetafitsContext` metadata
///
/// # Arguments
///
/// * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * A Rust-owned populated `mwalibMetafitsMetadata` struct or NULL if there was an error (check error_message)
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `metafits_context_ptr` must point to a populated MetafitsContext object from the `mwalib_metafits_context_new` function.
/// * Caller must call `mwalib_metafits_metadata_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_metadata_get(
    metafits_context_ptr: *mut MetafitsContext,
    error_message: *mut u8,
    error_message_length: size_t,
) -> *mut mwalibMetafitsMetadata {
    if metafits_context_ptr.is_null() {
        set_error_message(
            "mwalib_metafits_metadata_get() ERROR: null pointer for metafits_context_ptr passed in",
            error_message,
            error_message_length,
        );
        return ptr::null_mut();
    }

    let metafits_context = &*metafits_context_ptr;

    let out_context = mwalibMetafitsMetadata {
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

    Box::into_raw(Box::new(out_context))
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
/// * Nothing
///
///
/// # Safety
/// * This must be called once caller is finished with the `mwalibMetafitsMetadata` object
/// * `metafits_metadata_ptr` must point to a populated `mwalibMetafitsMetadata` object from the `mwalib_metafits_metadata_get` function.
/// * `metafits_metadata_ptr` must not have already been freed.
#[no_mangle]
#[cfg(not(tarpaulin_include))]
pub unsafe extern "C" fn mwalib_metafits_metadata_free(
    metafits_metadata_ptr: *mut mwalibMetafitsMetadata,
) {
    if metafits_metadata_ptr.is_null() {
        return;
    }
    drop(Box::from_raw(metafits_metadata_ptr));
}

///
/// C Representation of the `CorrelatorContext` metadata
///
#[repr(C)]
pub struct mwalibCorrelatorMetadata {
    /// Pointer to a MetafitsContext object
    pub metafits_context_ptr: *mut MetafitsContext,
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
    pub observation_bandwidth_hz: u32,
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
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * A Rust-owned populated `mwalibCorrelatorMetadata` struct or NULL if there was an error (check error_message)
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated `CorrelatorContext` object from the `mwalib_correlator_context_new` function.
/// * Caller must call `mwalib_correlator_metadata_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_metadata_get(
    correlator_context_ptr: *mut CorrelatorContext,
    error_message: *mut u8,
    error_message_length: size_t,
) -> *mut mwalibCorrelatorMetadata {
    if correlator_context_ptr.is_null() {
        set_error_message(
            "mwalib_correlator_metadata_get() ERROR: Warning: null pointer for correlator_context_ptr passed in",
            error_message,
            error_message_length,
        );
        return ptr::null_mut();
    }
    let context = &*correlator_context_ptr;
    let out_context = mwalibCorrelatorMetadata {
        metafits_context_ptr: context.metafits_context.as_ptr(),
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
        observation_bandwidth_hz: context.observation_bandwidth_hz,
        coarse_channel_width_hz: context.coarse_channel_width_hz,
        num_fine_channels_per_coarse: context.num_fine_channels_per_coarse,
        num_timestep_coarse_channel_bytes: context.num_timestep_coarse_channel_bytes,
        num_timestep_coarse_channel_floats: context.num_timestep_coarse_channel_floats,
        num_gpubox_files: context.num_gpubox_files,
    };

    Box::into_raw(Box::new(out_context))
}

/// Free a previously-allocated `mwalibCorrelatorMetadata` struct.
///
/// # Arguments
///
/// * `correlator_metadata_ptr` - pointer to an already populated `mwalibCorrelatorMetadata` object
///
///
/// # Returns
///
/// * Nothing
///
///
/// # Safety
/// * This must be called once caller is finished with the `mwalibCorrelatorMetadata` object
/// * `correlator_metadata_ptr` must point to a populated `mwalibCorrelatorMetadata` object from the `mwalib_correlator_metadata_get` function.
/// * `correlator_metadata_ptr` must not have already been freed.
#[no_mangle]
#[cfg(not(tarpaulin_include))]
pub unsafe extern "C" fn mwalib_correlator_metadata_free(
    correlator_metadata_ptr: *mut mwalibCorrelatorMetadata,
) {
    if correlator_metadata_ptr.is_null() {
        return;
    }
    drop(Box::from_raw(correlator_metadata_ptr));
}

///
/// C Representation of a mwalibBaseline struct
///
#[repr(C)]
pub struct mwalibBaseline {
    /// Index in the mwalibCorrelatorContext.antenna array for antenna1 for this baseline
    pub antenna1_index: usize,
    /// Index in the mwalibCorrelatorContext.antenna array for antenna2 for this baseline
    pub antenna2_index: usize,
}

/// This returns a struct containing the requested baseline
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
///
/// * `baseline_index` - item in the baseline array to return. This must be be between 0 and context->num_baselines - 1.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * A Rust-owned populated mwalibBaseline struct or NULL if there was an error (check error_message)
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated mwalibCorrelatorContext object from the mwalibCorrelatorContext_new function.
/// * Caller must call `mwalib_baseline_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_baseline_get(
    correlator_context_ptr: *mut CorrelatorContext,
    baseline_index: size_t,
    error_message: *mut u8,
    error_message_length: size_t,
) -> *mut mwalibBaseline {
    if correlator_context_ptr.is_null() {
        set_error_message(
            "mwalib_baseline_get() ERROR: null pointer for correlator_context_ptr passed in",
            error_message,
            error_message_length,
        );
        return ptr::null_mut();
    }
    let context = &*correlator_context_ptr;

    if baseline_index < context.num_baselines {
        let out_baseline = mwalibBaseline {
            antenna1_index: context.baselines[baseline_index].antenna1_index,
            antenna2_index: context.baselines[baseline_index].antenna2_index,
        };

        Box::into_raw(Box::new(out_baseline))
    } else {
        set_error_message(
            &format!(
                "mwalib_baseline_get() ERROR: baseline_index index must be between 0 ({} v {}) and {} ({} v {}).",
                context.baselines[0].antenna1_index,
                context.baselines[0].antenna2_index,
                context.num_baselines - 1,
                context.baselines[context.num_baselines - 1].antenna1_index,
                context.baselines[context.num_baselines - 1].antenna2_index,
            ),
            error_message,
            error_message_length,
        );
        ptr::null_mut()
    }
}

/// Free a previously-allocated `mwalibBaseline` struct.
///
/// # Arguments
///
/// * `baseline_ptr` - pointer to an already populated mwalibBaseline object
///
///
/// # Returns
///
/// * Nothing
///
///
/// # Safety
/// * This must be called once caller is finished with the mwalibBaseline object
/// * `baseline_ptr` must point to a populated mwalibBaseline object from the mwalib_baseline_get function.
/// * `baseline_ptr` must not have already been freed.
#[no_mangle]
#[cfg(not(tarpaulin_include))]
pub unsafe extern "C" fn mwalib_baseline_free(baseline_ptr: *mut mwalibBaseline) {
    if baseline_ptr.is_null() {
        return;
    }
    drop(Box::from_raw(baseline_ptr));
}

/// Representation in C of an mwalibRFInput struct
#[repr(C)]
pub struct mwalibRFInput {
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

/// This returns a struct containing the requested antenna
/// Or NULL if there was an error
///
/// # Arguments
///
/// * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object.
///
/// * `rf_input_index` - item in the rf_input array to return. This must be be between 0 and context->num_rf_inputs - 1.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * A Rust-owned populated mwalibRFInput struct or NULL if there was an error (check error_message)
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `metafits_context_ptr` must point to a populated mwalibCorrelatorContext object from the mwalibCorrelatorContext_new function.
/// * Caller must call `mwalib_rfinput_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_rfinput_get(
    metafits_context_ptr: *mut MetafitsContext,
    rf_input_index: size_t,
    error_message: *mut u8,
    error_message_length: size_t,
) -> *mut mwalibRFInput {
    if metafits_context_ptr.is_null() {
        set_error_message(
            "mwalib_rfinput_get() ERROR: null pointer for metafits_context_ptr passed in",
            error_message,
            error_message_length,
        );
        return ptr::null_mut();
    }
    let context = &*metafits_context_ptr;

    if rf_input_index < context.num_rf_inputs {
        let out_antenna = mwalibRFInput {
            input: context.rf_inputs[rf_input_index].input,
            antenna: context.rf_inputs[rf_input_index].antenna,
            tile_id: context.rf_inputs[rf_input_index].tile_id,
            tile_name: CString::new(String::from(&context.rf_inputs[rf_input_index].tile_name))
                .unwrap()
                .into_raw(),
            pol: CString::new(context.rf_inputs[rf_input_index].pol.to_string())
                .unwrap()
                .into_raw(),
            electrical_length_m: context.rf_inputs[rf_input_index].electrical_length_m,
            north_m: context.rf_inputs[rf_input_index].north_m,
            east_m: context.rf_inputs[rf_input_index].east_m,
            height_m: context.rf_inputs[rf_input_index].height_m,
            vcs_order: context.rf_inputs[rf_input_index].vcs_order,
            subfile_order: context.rf_inputs[rf_input_index].subfile_order,
            flagged: context.rf_inputs[rf_input_index].flagged,
            receiver_number: context.rf_inputs[rf_input_index].receiver_number,
            receiver_slot_number: context.rf_inputs[rf_input_index].receiver_slot_number,
        };

        Box::into_raw(Box::new(out_antenna))
    } else {
        set_error_message(
            &format!(
                "mwalib_rfinput_get() ERROR: rf_input index must be between 0 ({}{}) and {} ({}{}).",
                context.rf_inputs[0].tile_name,
                context.rf_inputs[0].pol,
                context.num_rf_inputs - 1,
                context.rf_inputs[context.num_rf_inputs - 1]
                    .tile_name,
                context.rf_inputs[context.num_rf_inputs - 1].pol
            ),
            error_message,
            error_message_length,
        );
        ptr::null_mut()
    }
}

/// Free a previously-allocated `mwalibRFInput` struct.
///
/// # Arguments
///
/// * `rf_input_ptr` - pointer to an already populated mwalibRFInput object
///
///
/// # Returns
///
/// * Nothing
///
///
/// # Safety
/// * This must be called once caller is finished with the mwalibRFInput object
/// * `rf_input_ptr` must point to a populated mwalibRFInput object from the mwalib_rfinput_get function.
/// * `rf_input_ptr` must not have already been freed.
#[no_mangle]
#[cfg(not(tarpaulin_include))]
pub unsafe extern "C" fn mwalib_rfinput_free(rf_input_ptr: *mut mwalibRFInput) {
    if rf_input_ptr.is_null() {
        return;
    }
    // Materialise object, so rust will drop it when it hits out of scope
    let rf_input = Box::from_raw(rf_input_ptr);

    // Also materialise the tile_name string
    CString::from_raw(rf_input.tile_name);
    CString::from_raw(rf_input.pol);
}

/// Representation in C of an mwalibCoarseChannel struct
#[repr(C)]
pub struct mwalibCoarseChannel {
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

/// This returns a struct containing the requested correlator coarse channel
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
///
/// * `coarse_channel_index` - item in the coarse_channel array to return. This must be be between 0 and context->num_coarse_channels - 1.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * A Rust-owned populated mwalibCoarseChannel struct or NULL if there was an error (check error_message)
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated mwalibCorrelatorContext object from the mwalibCorrelatorContext_new function.
/// * Caller must call `mwalib_coarse_channel_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_coarse_channel_get(
    correlator_context_ptr: *mut CorrelatorContext,
    coarse_channel_index: size_t,
    error_message: *mut u8,
    error_message_length: size_t,
) -> *mut mwalibCoarseChannel {
    if correlator_context_ptr.is_null() {
        set_error_message(
            "mwalib_correlator_coarse_channel_get() ERROR: null pointer for correlator_context_ptr passed in",
            error_message,
            error_message_length,
        );
        return ptr::null_mut();
    }
    let context = &*correlator_context_ptr;

    if coarse_channel_index < context.num_coarse_channels {
        let out_coarse_channel = mwalibCoarseChannel {
            correlator_channel_number: context.coarse_channels[coarse_channel_index]
                .correlator_channel_number,
            receiver_channel_number: context.coarse_channels[coarse_channel_index]
                .receiver_channel_number,
            gpubox_number: context.coarse_channels[coarse_channel_index].gpubox_number,
            channel_width_hz: context.coarse_channels[coarse_channel_index].channel_width_hz,
            channel_start_hz: context.coarse_channels[coarse_channel_index].channel_start_hz,
            channel_centre_hz: context.coarse_channels[coarse_channel_index].channel_centre_hz,
            channel_end_hz: context.coarse_channels[coarse_channel_index].channel_end_hz,
        };

        Box::into_raw(Box::new(out_coarse_channel))
    } else {
        set_error_message(
            &format!(
                "mwalib_correlator_coarse_channel_get() ERROR: coarse channel index must be between 0 and {}.",
                context.num_coarse_channels - 1
            ),
            error_message,
            error_message_length,
        );
        ptr::null_mut()
    }
}

/// Free a previously-allocated `mwalibCoarseChannel` struct.
///
/// # Arguments
///
/// * `coarse_channel_ptr` - pointer to an already populated mwalibCoarseChannel object
///
///
/// # Returns
///
/// * Nothing
///
///
/// # Safety
/// * This must be called once caller is finished with the mwalibCoarseChannel object
/// * `coarse_channel_ptr` must point to a populated mwalibCoarseChannel object from the mwalib_correlator_coarse_channel_get function.
/// * `coarse_channel_ptr` must not have already been freed.
#[no_mangle]
#[cfg(not(tarpaulin_include))]
pub unsafe extern "C" fn mwalib_coarse_channel_free(coarse_channel_ptr: *mut mwalibCoarseChannel) {
    if coarse_channel_ptr.is_null() {
        return;
    }
    drop(Box::from_raw(coarse_channel_ptr));
}

/// Representation in C of an mwalibAntenna struct
#[repr(C)]
pub struct mwalibAntenna {
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
    pub tile_name: *mut libc::c_char,
}

/// This returns a struct containing the requested antenna
///
/// # Arguments
///
/// * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object.
///
/// * `antenna_index` - item in the antenna array to return. This must be be between 0 and context->num_antennas - 1.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * A Rust-owned populated mwalibAntenna struct or NULL if there was an error (check error_message)
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `metafits_context_ptr` must point to a populated mwalibCorrelatorContext object from the mwalibCorrelatorContext_new function.
/// * Caller must call `mwalib_antenna_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_antenna_get(
    metafits_context_ptr: *mut MetafitsContext,
    antenna_index: size_t,
    error_message: *mut u8,
    error_message_length: size_t,
) -> *mut mwalibAntenna {
    if metafits_context_ptr.is_null() {
        set_error_message(
            "mwalib_antenna_get() ERROR: Warning: null pointer for metafits_context_ptr passed in",
            error_message,
            error_message_length,
        );
        return ptr::null_mut();
    }
    let context = &*metafits_context_ptr;

    if antenna_index < context.num_antennas {
        let out_antenna = mwalibAntenna {
            antenna: context.antennas[antenna_index].antenna,
            tile_id: context.antennas[antenna_index].tile_id,
            tile_name: CString::new(String::from(&context.antennas[antenna_index].tile_name))
                .unwrap()
                .into_raw(),
        };

        Box::into_raw(Box::new(out_antenna))
    } else {
        set_error_message(
            &format!(
                "mwalib_antenna_get() ERROR: antenna index must be between 0 ({}) and {} ({}).",
                context.antennas[0].tile_name,
                context.num_antennas - 1,
                context.antennas[context.num_antennas - 1].tile_name
            ),
            error_message,
            error_message_length,
        );
        ptr::null_mut()
    }
}

/// Free a previously-allocated `mwalibAntenna` struct.
///
/// # Arguments
///
/// * `antenna_ptr` - pointer to an already populated mwalibAntenna object
///
///
/// # Returns
///
/// * Nothing
///
///
/// # Safety
/// * This must be called once caller is finished with the mwalibAntenna object
/// * `antenna_ptr` must point to a populated mwalibAntenna object from the mwalib_antenna_get function.
/// * `antenna_ptr` must not have already been freed.
#[no_mangle]
#[cfg(not(tarpaulin_include))]
pub unsafe extern "C" fn mwalib_antenna_free(antenna_ptr: *mut mwalibAntenna) {
    if antenna_ptr.is_null() {
        return;
    }

    // Materialise object, so rust will drop it when it hits out of scope
    let antenna = Box::from_raw(antenna_ptr);

    // Also materialise the tile_name string
    CString::from_raw(antenna.tile_name);
}

///
/// C Representation of a mwalibTimeStep struct
///
#[repr(C)]
pub struct mwalibTimeStep {
    /// UNIX time (in milliseconds to avoid floating point inaccuracy)
    pub unix_time_ms: u64,
}

/// This returns a struct containing the requested timestep
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
///
/// * `timestep_index` - item in the timestep array to return. This must be be between 0 and context->num_timesteps - 1.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * A Rust-owned populated mwalibTimeStep struct or NULL if there was an error (check error_message)
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated mwalibCorrelatorContext object from the mwalibCorrelatorContext_new function.
/// * Caller must call `mwalib_timestep_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_timestep_get(
    correlator_context_ptr: *mut CorrelatorContext,
    timestep_index: size_t,
    error_message: *mut u8,
    error_message_length: size_t,
) -> *mut mwalibTimeStep {
    if correlator_context_ptr.is_null() {
        set_error_message(
            "mwalib_correlator_timestep_get() ERROR: null pointer for correlator_context_ptr passed in",
            error_message,
            error_message_length,
        );
        return ptr::null_mut();
    }
    let context = &*correlator_context_ptr;

    if timestep_index < context.num_timesteps {
        let out_timestep = mwalibTimeStep {
            unix_time_ms: context.timesteps[timestep_index].unix_time_ms,
        };

        Box::into_raw(Box::new(out_timestep))
    } else {
        set_error_message(
            &format!(
                "mwalib_correlator_timestep_get() ERROR: timestep index must be between 0 ({}) and {} ({}).",
                context.timesteps[0].unix_time_ms as f32 / 1000.,
                context.num_timesteps - 1,
                context.timesteps[context.num_timesteps - 1].unix_time_ms as f32 / 1000.,
            ),
            error_message,
            error_message_length,
        );
        ptr::null_mut()
    }
}

/// Free a previously-allocated `mwalibTimeStep` struct.
///
/// # Arguments
///
/// * `timestep_ptr` - pointer to an already populated mwalibTimeStep object
///
///
/// # Returns
///
/// * Nothing
///
///
/// # Safety
/// * This must be called once caller is finished with the mwalibTimeStep object
/// * `timestep_ptr` must point to a populated mwalibTimeStep object from the mwalib_correlator_timestep_get function.
/// * `timestep_ptr` must not have already been freed.
#[no_mangle]
#[cfg(not(tarpaulin_include))]
pub unsafe extern "C" fn mwalib_timestep_free(timestep_ptr: *mut mwalibTimeStep) {
    if timestep_ptr.is_null() {
        return;
    }
    drop(Box::from_raw(timestep_ptr));
}

///
/// C Representation of a mwalibVisibilityPol struct
///
#[repr(C)]
pub struct mwalibVisibilityPol {
    /// Polarisation (e.g. "XX" or "XY" or "YX" or "YY")
    pub polarisation: *mut libc::c_char,
}

/// This returns a struct containing the requested visibility polarisation
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
///
/// * `visibility_pol_index` - item in the visibility pol array to return. This must be be between 0 and context->num_visibility_pols - 1.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Returns
///
/// * A Rust-owned populated mwalibVisibilityPol struct or NULL if there was an error (check error_message)
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated mwalibCorrelatorContext object from the mwalibCorrelatorContext_new function.
/// * Caller must call `mwalib_visibility_pol_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_visibility_pol_get(
    correlator_context_ptr: *mut CorrelatorContext,
    visibility_pol_index: size_t,
    error_message: *mut u8,
    error_message_length: size_t,
) -> *mut mwalibVisibilityPol {
    if correlator_context_ptr.is_null() {
        set_error_message(
            "mwalib_correlator_visibility_pol_get() ERROR: null pointer for correlator_context_ptr passed in",
            error_message,
            error_message_length,
        );
        return ptr::null_mut();
    }
    let context = &*correlator_context_ptr;

    if visibility_pol_index < context.num_visibility_pols {
        let out_visibility_pol = mwalibVisibilityPol {
            polarisation: CString::new(String::from(
                &context.visibility_pols[visibility_pol_index].polarisation,
            ))
            .unwrap()
            .into_raw(),
        };

        Box::into_raw(Box::new(out_visibility_pol))
    } else {
        set_error_message(
            &format!(
                "mwalib_correlator_visibility_pol_get() ERROR: visibility_pol_index index must be between 0 ({}) and {} ({}).",
                context.visibility_pols[0].polarisation,
                context.num_visibility_pols - 1,
                context.visibility_pols[context.num_visibility_pols - 1].polarisation,
            ),
            error_message,
            error_message_length,
        );
        ptr::null_mut()
    }
}

/// Free a previously-allocated `mwalibVisibilityPol` struct.
///
/// # Arguments
///
/// * `visibility_pol_ptr` - pointer to an already populated mwalibVisibilityPol object
///
///
/// # Returns
///
/// * Nothing
///
///
/// # Safety
/// * This must be called once caller is finished with the mwalibVisibilityPol object
/// * `visibility_pol_ptr` must point to a populated mwalibVisibilityPol object from the mwalib_correlator_visibility_pol_get function.
/// * `visibility_pol_ptr` must not have already been freed.
#[no_mangle]
#[cfg(not(tarpaulin_include))]
pub unsafe extern "C" fn mwalib_visibility_pol_free(visibility_pol_ptr: *mut mwalibVisibilityPol) {
    if visibility_pol_ptr.is_null() {
        return;
    }
    drop(Box::from_raw(visibility_pol_ptr));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_error_message() {
        let buffer = CString::new("HELLO WORLD").unwrap();
        let buffer_ptr = buffer.as_ptr() as *mut u8;

        set_error_message("hello world", buffer_ptr, 12);

        assert_eq!(buffer, CString::new("hello world").unwrap());
    }

    // Metadata
    #[test]
    fn test_mwalib_metadata_get_valid() {
        // This tests for a valid context and metadata returned

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        let metafits_file =
            CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
        let metafits_file_ptr = metafits_file.as_ptr();

        /*let gpubox_file = CString::new(
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
        )
        .unwrap();
        let mut gpubox_files: Vec<*const c_char> = Vec::new();
        gpubox_files.push(gpubox_file.as_ptr());
        let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;*/

        unsafe {
            let context = mwalib_metafits_context_new(
                metafits_file_ptr,
                //gpubox_files_ptr,
                //1,
                error_message_ptr,
                60,
            );

            // Check we got a context object
            let context_ptr = context.as_mut();
            assert!(context_ptr.is_some());

            let md = Box::from_raw(mwalib_metafits_metadata_get(
                context,
                error_message_ptr,
                error_len,
            ));

            // We should get a valid timestep and no error message
            assert_eq!(md.obsid, 1_101_503_312);

            let expected_error: &str = &"mwalib_metafits_metadata_get() ERROR:";
            assert_ne!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    #[test]
    fn test_mwalibmetadata_get_null_context() {
        // This tests for a null context
        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        unsafe {
            let context_ptr = std::ptr::null_mut();
            let md_ptr = mwalib_metafits_metadata_get(context_ptr, error_message_ptr, error_len);

            // We should get a null pointer and an error message
            assert!(md_ptr.is_null());
            let expected_error: &str = &"mwalib_metafits_metadata_get() ERROR:";
            assert_eq!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    // RF Input
    #[test]
    fn test_mwalib_rfinput_get_valid() {
        // This tests for a valid context with a valid timestmwalibRFInputep
        let rf_index = 2; // valid  should be Tile012(X)

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        let metafits_file =
            CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
        let metafits_file_ptr = metafits_file.as_ptr();

        /*let gpubox_file = CString::new(
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
        )
        .unwrap();
        let mut gpubox_files: Vec<*const c_char> = Vec::new();
        gpubox_files.push(gpubox_file.as_ptr());
        let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;*/

        unsafe {
            let context = mwalib_metafits_context_new(
                metafits_file_ptr,
                //gpubox_files_ptr,
                //1,
                error_message_ptr,
                60,
            );

            // Check we got a context object
            let context_ptr = context.as_mut();
            assert!(context_ptr.is_some());

            let rf = Box::from_raw(mwalib_rfinput_get(
                context,
                rf_index,
                error_message_ptr,
                error_len,
            ));

            // We should get a valid timestep and no error message
            assert_eq!(rf.antenna, 1);

            assert_eq!(
                CString::from_raw(rf.tile_name),
                CString::new("Tile012").unwrap()
            );

            assert_eq!(CString::from_raw(rf.pol), CString::new("X").unwrap());

            let expected_error: &str = &"mwalib_rfinput_get() ERROR:";
            assert_ne!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    #[test]
    fn test_mwalib_rfinput_get_invalid() {
        // This tests for a valid context with an invalid mwalibRFInput (out of bounds)
        let rf_index = 300; // invalid

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        let metafits_file =
            CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
        let metafits_file_ptr = metafits_file.as_ptr();

        /*let gpubox_file = CString::new(
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
        )
        .unwrap();
        let mut gpubox_files: Vec<*const c_char> = Vec::new();
        gpubox_files.push(gpubox_file.as_ptr());
        let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;*/

        unsafe {
            let context = mwalib_metafits_context_new(
                metafits_file_ptr,
                //gpubox_files_ptr,
                //1,
                error_message_ptr,
                60,
            );

            // Check we got a context object
            let context_ptr = context.as_mut();
            assert!(context_ptr.is_some());

            let rf_ptr = mwalib_rfinput_get(context, rf_index, error_message_ptr, error_len);

            // We should get a null pointer and an error message
            assert!(rf_ptr.is_null());
            let expected_error: &str = &"mwalib_rfinput_get() ERROR:";
            assert_eq!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    #[test]
    fn test_mwalib_rfinput_get_null_context() {
        // This tests for a null context with an valid mwalibRFInput
        let rf_index = 100; // valid

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        unsafe {
            let context_ptr = std::ptr::null_mut();
            let rf_ptr = mwalib_rfinput_get(context_ptr, rf_index, error_message_ptr, error_len);

            // We should get a null pointer and an error message
            assert!(rf_ptr.is_null());
            let expected_error: &str = &"mwalib_rfinput_get() ERROR:";
            assert_eq!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    // Coarse Channel
    #[test]
    fn test_mwalib_coarsechannel_get_valid() {
        // This tests for a valid context with a valid mwalibCoarseChannel
        let channel_index = 0; // valid

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        let metafits_file =
            CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
        let metafits_file_ptr = metafits_file.as_ptr();

        let gpubox_file = CString::new(
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
        )
        .unwrap();
        let mut gpubox_files: Vec<*const c_char> = Vec::new();
        gpubox_files.push(gpubox_file.as_ptr());
        let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

        unsafe {
            let context = mwalib_correlator_context_new(
                metafits_file_ptr,
                gpubox_files_ptr,
                1,
                error_message_ptr,
                60,
            );

            // Check we got a context object
            let context_ptr = context.as_mut();
            assert!(context_ptr.is_some());

            let ch = Box::from_raw(mwalib_correlator_coarse_channel_get(
                context,
                channel_index,
                error_message_ptr,
                error_len,
            ));

            // We should get a valid timestep and no error message
            assert_eq!(ch.receiver_channel_number, 109);

            let expected_error: &str = &"mwalib_correlator_coarse_channel_get() ERROR:";
            assert_ne!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    #[test]
    fn test_mwalib_coarsechannel_get_invalid() {
        // This tests for a valid context with an invalid mwalibCoarseChannel (out of bounds)
        let chan_index = 100; // invalid

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        let metafits_file =
            CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
        let metafits_file_ptr = metafits_file.as_ptr();

        let gpubox_file = CString::new(
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
        )
        .unwrap();
        let mut gpubox_files: Vec<*const c_char> = Vec::new();
        gpubox_files.push(gpubox_file.as_ptr());
        let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

        unsafe {
            let context = mwalib_correlator_context_new(
                metafits_file_ptr,
                gpubox_files_ptr,
                1,
                error_message_ptr,
                60,
            );

            // Check we got a context object
            let context_ptr = context.as_mut();
            assert!(context_ptr.is_some());

            let ch_ptr = mwalib_correlator_coarse_channel_get(
                context,
                chan_index,
                error_message_ptr,
                error_len,
            );

            // We should get a null pointer and an error message
            assert!(ch_ptr.is_null());
            let expected_error: &str = &"mwalib_correlator_coarse_channel_get() ERROR:";
            assert_eq!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    #[test]
    fn test_mwalibcoarsechannel_get_null_context() {
        // This tests for a null context with a valid mwalibCoarseChannel
        let timestep_index = 0; // valid

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        unsafe {
            let context_ptr = std::ptr::null_mut();
            let ch_ptr = mwalib_correlator_coarse_channel_get(
                context_ptr,
                timestep_index,
                error_message_ptr,
                error_len,
            );

            // We should get a null pointer and an error message
            assert!(ch_ptr.is_null());
            let expected_error: &str = &"mwalib_correlator_coarse_channel_get() ERROR:";
            assert_eq!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    // Antenna
    #[test]
    fn test_mwalibantenna_get_valid() {
        // This tests for a valid context with a valid mwalibAntenna
        let ant_index = 2; // valid- should be Tile013

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        let metafits_file =
            CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
        let metafits_file_ptr = metafits_file.as_ptr();

        /*let gpubox_file = CString::new(
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
        )
        .unwrap();
        let mut gpubox_files: Vec<*const c_char> = Vec::new();
        gpubox_files.push(gpubox_file.as_ptr());
        let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;*/

        unsafe {
            let context = mwalib_metafits_context_new(
                metafits_file_ptr,
                //gpubox_files_ptr,
                //1,
                error_message_ptr,
                60,
            );

            // Check we got a context object
            let context_ptr = context.as_mut();
            assert!(context_ptr.is_some());

            let ant = Box::from_raw(mwalib_antenna_get(
                context,
                ant_index,
                error_message_ptr,
                error_len,
            ));

            // We should get a valid timestep and no error message
            assert_eq!(ant.tile_id, 13);

            let expected_error: &str = &"mwalib_antenna_get() ERROR:";
            assert_ne!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    #[test]
    fn test_mwalibantenna_get_invalid() {
        // This tests for a valid context with an invalid mwalibAntenna (out of bounds)
        let ant_index = 300; // invalid

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        let metafits_file =
            CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
        let metafits_file_ptr = metafits_file.as_ptr();

        /*let gpubox_file = CString::new(
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
        )
        .unwrap();
        let mut gpubox_files: Vec<*const c_char> = Vec::new();
        gpubox_files.push(gpubox_file.as_ptr());
        let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;*/

        unsafe {
            let context = mwalib_metafits_context_new(
                metafits_file_ptr,
                //gpubox_files_ptr,
                //1,
                error_message_ptr,
                60,
            );

            // Check we got a context object
            let context_ptr = context.as_mut();
            assert!(context_ptr.is_some());

            let ant_ptr = mwalib_antenna_get(context, ant_index, error_message_ptr, error_len);

            // We should get a null pointer and an error message
            assert!(ant_ptr.is_null());
            let expected_error: &str = &"mwalib_antenna_get() ERROR:";
            assert_eq!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    #[test]
    fn test_mwalibantenna_get_null_context() {
        // This tests for a null context with an valid mwalibAntenna
        let ant_index = 2; // valid

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        unsafe {
            let context_ptr = std::ptr::null_mut();
            let ant_ptr = mwalib_antenna_get(context_ptr, ant_index, error_message_ptr, error_len);

            // We should get a null pointer and an error message
            assert!(ant_ptr.is_null());
            let expected_error: &str = &"mwalib_antenna_get() ERROR:";
            assert_eq!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    // baseline
    #[test]
    fn test_mwalib_baseline_get_valid() {
        // This tests for a valid context with a valid baseline
        let baseline_index = 2; // valid

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        let metafits_file =
            CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
        let metafits_file_ptr = metafits_file.as_ptr();

        let gpubox_file = CString::new(
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
        )
        .unwrap();
        let mut gpubox_files: Vec<*const c_char> = Vec::new();
        gpubox_files.push(gpubox_file.as_ptr());
        let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

        unsafe {
            let context = mwalib_correlator_context_new(
                metafits_file_ptr,
                gpubox_files_ptr,
                1,
                error_message_ptr,
                60,
            );

            // Check we got a context object
            let context_ptr = context.as_mut();
            assert!(context_ptr.is_some());

            let bl = Box::from_raw(mwalib_baseline_get(
                context,
                baseline_index,
                error_message_ptr,
                error_len,
            ));

            // We should get a valid baseline and no error message
            assert_eq!(bl.antenna1_index, 0);
            assert_eq!(bl.antenna2_index, 2);

            let expected_error: &str = &"mwalib_baseline_get() ERROR:";
            assert_ne!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    #[test]
    fn test_mwalib_baseline_get_invalid() {
        // This tests for a valid context with an invalid baseline (out of bounds)
        let baseline_index = 100_000; // invalid

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        let metafits_file =
            CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
        let metafits_file_ptr = metafits_file.as_ptr();

        let gpubox_file = CString::new(
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
        )
        .unwrap();
        let mut gpubox_files: Vec<*const c_char> = Vec::new();
        gpubox_files.push(gpubox_file.as_ptr());
        let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

        unsafe {
            let context = mwalib_correlator_context_new(
                metafits_file_ptr,
                gpubox_files_ptr,
                1,
                error_message_ptr,
                60,
            );

            // Check we got a context object
            let context_ptr = context.as_mut();
            assert!(context_ptr.is_some());

            let bl_ptr = mwalib_baseline_get(context, baseline_index, error_message_ptr, error_len);

            // We should get a null pointer and an error message
            assert!(bl_ptr.is_null());
            let expected_error: &str = &"mwalib_baseline_get() ERROR:";
            assert_eq!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    #[test]
    fn test_mwalibbaseline_get_null_context() {
        // This tests for a null context with an valid baseline
        let baseline_index = 1; // valid

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        unsafe {
            let context_ptr = std::ptr::null_mut();
            let bl_ptr =
                mwalib_baseline_get(context_ptr, baseline_index, error_message_ptr, error_len);

            // We should get a null pointer and an error message
            assert!(bl_ptr.is_null());
            let expected_error: &str = &"mwalib_baseline_get() ERROR:";
            assert_eq!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    // timestep
    #[test]
    fn test_mwalibtimestep_get_valid() {
        // This tests for a valid context with a valid timestep
        let timestep_index = 0; // valid

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        let metafits_file =
            CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
        let metafits_file_ptr = metafits_file.as_ptr();

        let gpubox_file = CString::new(
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
        )
        .unwrap();
        let mut gpubox_files: Vec<*const c_char> = Vec::new();
        gpubox_files.push(gpubox_file.as_ptr());
        let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

        unsafe {
            let context = mwalib_correlator_context_new(
                metafits_file_ptr,
                gpubox_files_ptr,
                1,
                error_message_ptr,
                60,
            );

            // Check we got a context object
            let context_ptr = context.as_mut();
            assert!(context_ptr.is_some());

            let ts = Box::from_raw(mwalib_correlator_timestep_get(
                context,
                timestep_index,
                error_message_ptr,
                error_len,
            ));

            // We should get a valid timestep and no error message
            assert_eq!(ts.unix_time_ms, 1_417_468_096_000);

            let expected_error: &str = &"mwalib_correlator_timestep_get() ERROR:";
            assert_ne!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    #[test]
    fn test_mwalibtimestep_get_invalid() {
        // This tests for a valid context with an invalid timestep (out of bounds)
        let timestep_index = 100; // invalid

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        let metafits_file =
            CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
        let metafits_file_ptr = metafits_file.as_ptr();

        let gpubox_file = CString::new(
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
        )
        .unwrap();
        let mut gpubox_files: Vec<*const c_char> = Vec::new();
        gpubox_files.push(gpubox_file.as_ptr());
        let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

        unsafe {
            let context = mwalib_correlator_context_new(
                metafits_file_ptr,
                gpubox_files_ptr,
                1,
                error_message_ptr,
                60,
            );

            // Check we got a context object
            let context_ptr = context.as_mut();
            assert!(context_ptr.is_some());

            let ts_ptr = mwalib_correlator_timestep_get(
                context,
                timestep_index,
                error_message_ptr,
                error_len,
            );

            // We should get a null pointer and an error message
            assert!(ts_ptr.is_null());
            let expected_error: &str = &"mwalib_correlator_timestep_get() ERROR:";
            assert_eq!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    #[test]
    fn test_mwalibtimestep_get_null_context() {
        // This tests for a null context with an valid timestep
        let timestep_index = 0; // valid

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        unsafe {
            let context_ptr = std::ptr::null_mut();
            let ts_ptr = mwalib_correlator_timestep_get(
                context_ptr,
                timestep_index,
                error_message_ptr,
                error_len,
            );

            // We should get a null pointer and an error message
            assert!(ts_ptr.is_null());
            let expected_error: &str = &"mwalib_correlator_timestep_get() ERROR:";
            assert_eq!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    // visibilitypol
    #[test]
    fn test_mwalib_correlator_visibility_pol_get_valid() {
        // This tests for a valid context with a valid visibilitypol
        let vispol_index = 0; // valid

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        let metafits_file =
            CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
        let metafits_file_ptr = metafits_file.as_ptr();

        let gpubox_file = CString::new(
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
        )
        .unwrap();
        let mut gpubox_files: Vec<*const c_char> = Vec::new();
        gpubox_files.push(gpubox_file.as_ptr());
        let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

        unsafe {
            let context = mwalib_correlator_context_new(
                metafits_file_ptr,
                gpubox_files_ptr,
                1,
                error_message_ptr,
                60,
            );

            // Check we got a context object
            let context_ptr = context.as_mut();
            assert!(context_ptr.is_some());

            let vp = Box::from_raw(mwalib_correlator_visibility_pol_get(
                context,
                vispol_index,
                error_message_ptr,
                error_len,
            ));

            // We should get a valid timestep and no error message
            assert_eq!(
                CString::from_raw(vp.polarisation).into_string().unwrap(),
                String::from("XX")
            );

            let expected_error: &str = &"mwalib_correlator_visibility_pol_get() ERROR:";
            assert_ne!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    #[test]
    fn test_mwalib_correlator_visibility_pol_get_invalid() {
        // This tests for a valid context with an invalid visibility pol (out of bounds)
        let vispol_index = 100; // invalid

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        let metafits_file =
            CString::new("test_files/1101503312_1_timestep/1101503312.metafits").unwrap();
        let metafits_file_ptr = metafits_file.as_ptr();

        let gpubox_file = CString::new(
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
        )
        .unwrap();
        let mut gpubox_files: Vec<*const c_char> = Vec::new();
        gpubox_files.push(gpubox_file.as_ptr());
        let gpubox_files_ptr = gpubox_files.as_ptr() as *mut *const c_char;

        unsafe {
            let context = mwalib_correlator_context_new(
                metafits_file_ptr,
                gpubox_files_ptr,
                1,
                error_message_ptr,
                60,
            );

            // Check we got a context object
            let context_ptr = context.as_mut();
            assert!(context_ptr.is_some());

            let ts_ptr = mwalib_correlator_visibility_pol_get(
                context,
                vispol_index,
                error_message_ptr,
                error_len,
            );

            // We should get a null pointer and an error message
            assert!(ts_ptr.is_null());
            let expected_error: &str = &"mwalib_correlator_visibility_pol_get() ERROR:";
            assert_eq!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }

    #[test]
    fn test_mwalibvisibilitypol_get_null_context() {
        // This tests for a null context with an valid visibility pol
        let vispol_index = 0; // valid

        let error_message =
            CString::new("                                                            ").unwrap();
        let error_message_ptr = error_message.as_ptr() as *mut u8;
        let error_len: size_t = 60;

        unsafe {
            let context_ptr = std::ptr::null_mut();
            let ts_ptr = mwalib_correlator_visibility_pol_get(
                context_ptr,
                vispol_index,
                error_message_ptr,
                error_len,
            );

            // We should get a null pointer and an error message
            assert!(ts_ptr.is_null());
            let expected_error: &str = &"mwalib_correlator_visibility_pol_get() ERROR:";
            assert_eq!(
                error_message.into_string().unwrap()[0..expected_error.len()],
                *expected_error
            );
        }
    }
}
