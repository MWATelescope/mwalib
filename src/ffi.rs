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

use std::ffi::*;
use std::process::exit;
use std::ptr;
use std::slice;

use libc::{c_char, c_double, c_float, c_int, c_longlong, c_uint, c_ulong, size_t};

use crate::*;

#[repr(C)]
pub struct mwalibMetadata {
    ///
    /// This is just a C struct to allow the caller to consume all of the metadata
    ///
    /// See definition of context::mwalibContext for full description of each attribute
    ///
    ///
    pub obsid: c_uint,
    pub corr_version: CorrelatorVersion,
    pub coax_v_factor: c_double,
    pub start_unix_time_milliseconds: c_ulong,
    pub end_unix_time_milliseconds: c_ulong,
    pub duration_milliseconds: c_ulong,
    pub num_timesteps: size_t,
    pub num_antennas: size_t,
    //pub rf_inputs: Vec<mwalibRFInput>,
    //pub antennas: Vec<mwalibAntenna>,
    pub num_baselines: size_t,
    pub integration_time_milliseconds: c_ulong,
    pub num_antenna_pols: size_t,
    pub num_visibility_pols: size_t,
    pub num_fine_channels_per_coarse: size_t,
    pub num_coarse_channels: size_t,
    //pub coarse_channels: Vec<mwalibCoarseChannel>,
    pub fine_channel_width_hz: c_uint,
    pub coarse_channel_width_hz: c_uint,
    pub observation_bandwidth_hz: c_uint,
    pub timestep_coarse_channel_bytes: size_t,
    pub num_gpubox_files: size_t,
    pub timestep_coarse_channel_floats: size_t,
}

/// This returns a struct containing the mwalibContext metadata
/// # Safety
/// TODO
#[no_mangle]
pub unsafe extern "C" fn mwalibMetadata_get(ptr: *mut mwalibContext) -> *mut mwalibMetadata {
    if ptr.is_null() {
        eprintln!("mwalibMetadata_get: Warning: null pointer passed in");
        exit(1);
    }
    let context = &*ptr;

    let out_context = mwalibMetadata {
        obsid: context.obsid,
        corr_version: context.corr_version,
        coax_v_factor: context.coax_v_factor,
        start_unix_time_milliseconds: context.start_unix_time_milliseconds,
        end_unix_time_milliseconds: context.end_unix_time_milliseconds,
        duration_milliseconds: context.duration_milliseconds,
        num_timesteps: context.num_timesteps,
        num_antennas: context.num_antennas,
        num_baselines: context.num_baselines,
        integration_time_milliseconds: context.integration_time_milliseconds,
        num_antenna_pols: context.num_antenna_pols,
        num_visibility_pols: context.num_visibility_pols,
        num_fine_channels_per_coarse: context.num_fine_channels_per_coarse,
        num_coarse_channels: context.num_coarse_channels,
        fine_channel_width_hz: context.fine_channel_width_hz,
        coarse_channel_width_hz: context.coarse_channel_width_hz,
        observation_bandwidth_hz: context.observation_bandwidth_hz,
        timestep_coarse_channel_bytes: context.timestep_coarse_channel_bytes,
        num_gpubox_files: context.num_gpubox_files,
        timestep_coarse_channel_floats: context.timestep_coarse_channel_floats,
    };

    Box::into_raw(Box::new(out_context))
}

/// # Safety
/// TODO: What does the caller need to know?
/// Free a previously-allocated `mwalibContext` struct.
#[no_mangle]
pub unsafe extern "C" fn mwalibMetadata_free(ptr: *mut mwalibMetadata) {
    if ptr.is_null() {
        eprintln!("mwalibMetadata_free: Warning: null pointer passed in");
        return;
    }
    Box::from_raw(ptr);
}

#[repr(C)]
pub struct mwalibTimeStep {
    // UNIX time (in milliseconds to avoid floating point inaccuracy)
    pub unix_time_ms: c_ulong,
}

/// This returns a struct containing the requested timestep
/// Or NULL if there was an error
/// # Safety
/// TODO
#[no_mangle]
pub unsafe extern "C" fn mwalibTimeStep_get(
    ptr: *mut mwalibContext,
    timestep_index: size_t,
) -> *mut mwalibTimeStep {
    if ptr.is_null() {
        eprintln!("mwalibTimeStep_get: Warning: null pointer passed in");
        exit(1);
    }
    let context = &*ptr;

    if timestep_index < context.num_timesteps {
        let out_timestep = mwalibTimeStep {
            unix_time_ms: context.timesteps[timestep_index].unix_time_ms,
        };

        Box::into_raw(Box::new(out_timestep))
    } else {
        eprintln!(
            "mwalibTimeStep_get: timestep index must be between 0 {} and {} ({}).",
            context.timesteps[0].unix_time_ms,
            context.num_timesteps - 1,
            context.timesteps[context.num_timesteps - 1].unix_time_ms
        );
        ptr::null_mut()
    }
}

/// # Safety
/// TODO: What does the caller need to know?
/// Free a previously-allocated `mwalibTimeStep` struct.
#[no_mangle]
pub unsafe extern "C" fn mwalibTimeStep_free(ptr: *mut mwalibTimeStep) {
    if ptr.is_null() {
        eprintln!("mwalibTimeStep_free: Warning: null pointer passed in");
        return;
    }
    Box::from_raw(ptr);
}

/// # Safety
/// Free a rust-allocated CString.
///
/// mwalib uses error strings to detail the caller with anything that went
/// wrong. Non-rust languages cannot deallocate these strings; so, call this
/// function with the pointer to do that.
#[no_mangle]
pub unsafe extern "C" fn mwalib_free_rust_cstring(rust_cstring: *mut c_char) {
    // Don't do anything if the pointer is null.
    if rust_cstring.is_null() {
        return;
    }
    CString::from_raw(rust_cstring);
}

/// # Safety
/// TODO: What does the caller need to know?
/// Create and return a pointer to an `mwalibContext` struct or NULL if error occurs
#[no_mangle]
pub unsafe extern "C" fn mwalibContext_new(
    metafits: *const c_char,
    gpuboxes: *mut *const c_char,
    gpubox_count: size_t,
) -> *mut mwalibContext {
    let m = CStr::from_ptr(metafits).to_str().unwrap().to_string();
    let gpubox_slice = slice::from_raw_parts(gpuboxes, gpubox_count);
    let mut gpubox_files = Vec::with_capacity(gpubox_count);
    for g in gpubox_slice {
        let s = CStr::from_ptr(*g).to_str().unwrap();
        gpubox_files.push(s.to_string())
    }
    let context = match mwalibContext::new(&m, &gpubox_files) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return ptr::null_mut();
        }
    };
    Box::into_raw(Box::new(context))
}

/// # Safety
/// TODO: What does the caller need to know?
/// Free a previously-allocated `mwalibContext` struct.
#[no_mangle]
pub unsafe extern "C" fn mwalibContext_free(ptr: *mut mwalibContext) {
    if ptr.is_null() {
        eprintln!("mwalibContext_free: Warning: null pointer passed in");
        return;
    }
    Box::from_raw(ptr);
}

/// # Safety
/// TODO: What does the caller need to know?
/// Display an `mwalibContext` struct.
#[no_mangle]
pub unsafe extern "C" fn mwalibContext_display(ptr: *const mwalibContext) {
    if ptr.is_null() {
        eprintln!("mwalibContext_display: Warning: null pointer passed in");
        return;
    }
    let context = &*ptr;
    println!("{}", context);
}

/// # Safety
/// Read MWA data.
///
/// This method takes as input a timestep_index and a coarse_channel_index to return one
/// HDU of data in [baseline][freq][pol][r][i] format
#[no_mangle]
pub unsafe extern "C" fn mwalibContext_read_one_timestep_coarse_channel_bfp(
    context_ptr: *mut mwalibContext,
    timestep_index: *mut c_int,
    coarse_channel_index: *mut c_int,
) -> *mut c_float {
    // Load the previously-initialised context and buffer structs. Exit if
    // either of these are null.
    let context = if context_ptr.is_null() {
        eprintln!("mwalibContext_read_one_timestep_coarse_channel_bfp: Error: null pointer for \"context_ptr\" passed in");
        return ptr::null_mut();
    } else {
        &mut *context_ptr
    };

    // Read data in.
    let mut data = match context.read_one_timestep_coarse_channel_bfp(
        *timestep_index as usize,
        *coarse_channel_index as usize,
    ) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("{}", e);
            return ptr::null_mut();
        }
    };
    // If the data buffer is empty, then just return a null pointer.
    if data.is_empty() {
        return ptr::null_mut();
    }

    // Convert the vector of `data` to C-compatible array.
    let data_buffer_ptr = data.as_mut_ptr();
    std::mem::forget(data);

    data_buffer_ptr
}

/// # Safety
/// Free a previously-allocated float* (designed for use after
/// `mwalibContext_read_one_timestep_coarse_channel_bfp`).
///
/// Python can't free memory itself, so this is useful for Python (and perhaps
/// other languages).
#[no_mangle]
pub unsafe extern "C" fn free_float_buffer(
    float_buffer_ptr: *mut c_float,
    gpubox_hdu_size: *const c_longlong,
) {
    if float_buffer_ptr.is_null() {
        eprintln!("free_float_buffer: Warning: null pointer passed in!");
        return;
    }

    let _ = Vec::from_raw_parts(
        float_buffer_ptr,
        *gpubox_hdu_size as usize,
        *gpubox_hdu_size as usize,
    );
}
