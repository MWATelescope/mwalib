// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    coarse_channel,
    ffi::{
        ffi_create_c_array, ffi_free_c_array, set_c_string, MWALIB_FAILURE,
        MWALIB_NO_DATA_FOR_TIMESTEP_COARSECHAN, MWALIB_SUCCESS,
    },
    timestep, CorrelatorContext, GpuboxError, MWAVersion,
};
use libc::size_t;
use std::{
    ffi::{c_char, c_double, c_float, CStr},
    slice,
};

///
/// C Representation of the `CorrelatorContext` metadata
///

#[repr(C)]
pub struct CorrelatorMetadata {
    // ---- 8-byte aligned fields first (u64) ----
    /// The proper start of the observation (the time that is common to all provided gpubox files).
    pub common_start_unix_time_ms: u64,
    /// `end_time_ms` is the actual end time of the observation
    pub common_end_unix_time_ms: u64,
    /// `start_unix_time_ms` but in GPS milliseconds
    pub common_start_gps_time_ms: u64,
    /// `end_unix_time_ms` but in GPS milliseconds
    pub common_end_gps_time_ms: u64,
    /// Total duration of observation (based on gpubox files)
    pub common_duration_ms: u64,
    /// The start of the observation only including timesteps after the quack time
    pub common_good_start_unix_time_ms: u64,
    /// Common end time only including timesteps after the quack time
    pub common_good_end_unix_time_ms: u64,
    /// `common_good_start_unix_time_ms` but in GPS milliseconds
    pub common_good_start_gps_time_ms: u64,
    /// `common_good_end_unix_time_ms` but in GPS milliseconds
    pub common_good_end_gps_time_ms: u64,
    /// Total duration of common_good timesteps
    pub common_good_duration_ms: u64,

    // ---- Pointers (also 8 bytes on 64-bit) ----
    /// This is an array of all known timesteps
    pub timesteps: *mut timestep::ffi::TimeStep,
    /// Vector of coarse channels
    pub coarse_chans: *mut coarse_channel::ffi::CoarseChannel,
    /// Vector of (in)common timestep indices
    pub common_timestep_indices: *mut usize,
    /// Indices of common coarse channels
    pub common_coarse_chan_indices: *mut usize,
    /// Vector of (in)common timestep indices only including timesteps after the quack time
    pub common_good_timestep_indices: *mut usize,
    /// Vector of (in)common coarse channel indices only including timesteps after the quack time
    pub common_good_coarse_chan_indices: *mut usize,
    /// The indices of any timesteps which we have *some* data for
    pub provided_timestep_indices: *mut usize,
    /// The indices of any coarse channels which we have *some* data for
    pub provided_coarse_chan_indices: *mut usize,

    // ---- usize counters ----
    /// Number of timesteps in the timestep array
    pub num_timesteps: usize,
    /// Count of coarse channels
    pub num_coarse_chans: usize,
    /// Count of common timesteps
    pub num_common_timesteps: usize,
    /// Count of common coarse channels
    pub num_common_coarse_chans: usize,
    /// Number of common timesteps only including timesteps after the quack time
    pub num_common_good_timesteps: usize,
    /// Number of common coarse channels only including timesteps after the quack time
    pub num_common_good_coarse_chans: usize,
    /// Number of provided timestep indices we have at least *some* data for
    pub num_provided_timesteps: usize,
    /// Number of provided coarse channel indices we have at least *some* data for
    pub num_provided_coarse_chans: usize,

    // ---- Remaining usize counters ----
    /// The number of bytes taken up by a scan/timestep in each gpubox file.
    pub num_timestep_coarse_chan_bytes: usize,
    /// The number of floats in each gpubox visibility HDU.
    pub num_timestep_coarse_chan_floats: usize,
    /// The number of floats in each gpubox weights HDU.
    pub num_timestep_coarse_chan_weight_floats: usize,
    /// This is the number of gpubox files *per batch*.
    pub num_gpubox_files: usize,

    // ---- 32-bit integers ----
    /// Total bandwidth of the common coarse channels which have been provided
    pub common_bandwidth_hz: u32,
    /// Total bandwidth of the common coarse channels only including timesteps after the quack time
    pub common_good_bandwidth_hz: u32,

    // ---- Floats smaller than f64 ----
    /// BSCALE - FITS BSCALE or SCALEFAC value set on the visibility HDUs
    pub bscale: f32,

    // ---- Enums ----
    /// Version of the correlator format
    pub mwa_version: MWAVersion,
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
/// * MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated `CorrelatorContext` object from the `mwalib_correlator_context_new` function.
/// * Caller must call `mwalib_correlator_metadata_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_metadata_get(
    correlator_context_ptr: *mut CorrelatorContext,
    out_correlator_metadata_ptr: *mut *mut CorrelatorMetadata,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    if correlator_context_ptr.is_null() {
        set_c_string(
            "mwalib_correlator_metadata_get() ERROR: Warning: null pointer for correlator_context_ptr passed in",
            error_message,
            error_message_length,
        );

        if !out_correlator_metadata_ptr.is_null() {
            *out_correlator_metadata_ptr = std::ptr::null_mut();
        }

        return MWALIB_FAILURE;
    }
    // Get the correlator context object from the raw pointer passed in
    let context = &*correlator_context_ptr;

    // Populate correlator coarse channels
    let (coarse_channels_ptr, coarse_channels_len) =
        coarse_channel::ffi::CoarseChannel::populate_array(&context.coarse_chans);

    // Populate correlator timesteps
    let (timesteps_ptr, timesteps_len) =
        timestep::ffi::TimeStep::populate_array(&context.timesteps);

    // Populate the rust owned data structure with data from the correlator context
    // We explicitly break out the attributes so at compile time it will let us know
    // if there have been new fields added to the rust struct, then we can choose to
    // ignore them (with _) or add that field to the FFI struct.
    let out_metadata = {
        let CorrelatorContext {
            metafits_context: _, // This is provided by the seperate metafits_metadata struct in FFI
            mwa_version,
            num_timesteps: _,
            timesteps: _, // This is populated seperately
            num_coarse_chans: _,
            coarse_chans: _, // This is populated seperately
            common_timestep_indices: _,
            num_common_timesteps: _,
            common_coarse_chan_indices: _,
            num_common_coarse_chans: _,
            common_start_unix_time_ms,
            common_end_unix_time_ms,
            common_start_gps_time_ms,
            common_end_gps_time_ms,
            common_duration_ms,
            common_bandwidth_hz,
            common_good_timestep_indices: _,
            num_common_good_timesteps: _,
            common_good_coarse_chan_indices: _,
            num_common_good_coarse_chans: _,
            common_good_start_unix_time_ms,
            common_good_end_unix_time_ms,
            common_good_start_gps_time_ms,
            common_good_end_gps_time_ms,
            common_good_duration_ms,
            common_good_bandwidth_hz,
            provided_timestep_indices: _,
            num_provided_timesteps: _,
            provided_coarse_chan_indices: _,
            num_provided_coarse_chans: _,
            num_timestep_coarse_chan_bytes,
            num_timestep_coarse_chan_floats,
            num_timestep_coarse_chan_weight_floats,
            num_gpubox_files,
            gpubox_batches: _, // This is currently not provided to FFI as it is private
            gpubox_time_map: _, // This is currently not provided to FFI
            legacy_conversion_table: _, // This is currently not provided to FFI as it is private
            bscale,
        } = context;

        let (common_timestep_indices_ptr, common_timestep_indices_len) =
            ffi_create_c_array(context.common_timestep_indices.clone());

        let (common_good_timestep_indices_ptr, common_good_timestep_indices_len) =
            ffi_create_c_array(context.common_good_timestep_indices.clone());

        let (provided_timestep_indices_ptr, provided_timestep_indices_len) =
            ffi_create_c_array(context.provided_timestep_indices.clone());

        let (common_coarse_chan_indices_ptr, common_coarse_chan_indices_len) =
            ffi_create_c_array(context.common_coarse_chan_indices.clone());

        let (common_good_coarse_chan_indices_ptr, common_good_coarse_chan_indices_len) =
            ffi_create_c_array(context.common_good_coarse_chan_indices.clone());

        let (provided_coarse_chan_indices_ptr, provided_coarse_chan_indices_len) =
            ffi_create_c_array(context.provided_coarse_chan_indices.clone());

        CorrelatorMetadata {
            mwa_version: *mwa_version,
            num_timesteps: timesteps_len,
            timesteps: timesteps_ptr,
            num_coarse_chans: coarse_channels_len,
            coarse_chans: coarse_channels_ptr,
            num_common_timesteps: common_timestep_indices_len,
            common_timestep_indices: common_timestep_indices_ptr,
            num_common_coarse_chans: common_coarse_chan_indices_len,
            common_coarse_chan_indices: common_coarse_chan_indices_ptr,
            common_start_unix_time_ms: *common_start_unix_time_ms,
            common_end_unix_time_ms: *common_end_unix_time_ms,
            common_start_gps_time_ms: *common_start_gps_time_ms,
            common_end_gps_time_ms: *common_end_gps_time_ms,
            common_duration_ms: *common_duration_ms,
            common_bandwidth_hz: *common_bandwidth_hz,

            num_common_good_timesteps: common_good_timestep_indices_len,
            common_good_timestep_indices: common_good_timestep_indices_ptr,
            num_common_good_coarse_chans: common_good_coarse_chan_indices_len,
            common_good_coarse_chan_indices: common_good_coarse_chan_indices_ptr,
            common_good_start_unix_time_ms: *common_good_start_unix_time_ms,
            common_good_end_unix_time_ms: *common_good_end_unix_time_ms,
            common_good_start_gps_time_ms: *common_good_start_gps_time_ms,
            common_good_end_gps_time_ms: *common_good_end_gps_time_ms,
            common_good_duration_ms: *common_good_duration_ms,
            common_good_bandwidth_hz: *common_good_bandwidth_hz,

            num_provided_timesteps: provided_timestep_indices_len,
            provided_timestep_indices: provided_timestep_indices_ptr,
            num_provided_coarse_chans: provided_coarse_chan_indices_len,
            provided_coarse_chan_indices: provided_coarse_chan_indices_ptr,
            num_timestep_coarse_chan_bytes: *num_timestep_coarse_chan_bytes,
            num_timestep_coarse_chan_floats: *num_timestep_coarse_chan_floats,
            num_timestep_coarse_chan_weight_floats: *num_timestep_coarse_chan_weight_floats,
            num_gpubox_files: *num_gpubox_files,
            bscale: *bscale,
        }
    };

    // Return ownership to C via raw pointer
    if !out_correlator_metadata_ptr.is_null() {
        *out_correlator_metadata_ptr = Box::into_raw(Box::new(out_metadata));
        return MWALIB_SUCCESS;
    } else {
        // Cannot write the out pointer; report failure
        set_c_string(
            "mwalib_correlator_metadata_get() ERROR: out_correlator_metadata_ptr was NULL.",
            error_message,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }
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
/// * MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `CorrelatorMetadata` object
/// * `correlator_metadata_ptr` must point to a populated `CorrelatorMetadata` object from the `mwalib_correlator_metadata_get` function.
/// * `correlator_metadata_ptr` must not have already been freed.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_metadata_free(c_ptr: *mut CorrelatorMetadata) -> i32 {
    if c_ptr.is_null() {
        return MWALIB_SUCCESS;
    }

    // Box the object to free its contents
    let boxed: Box<CorrelatorMetadata> = Box::from_raw(c_ptr);

    //
    // free any other members first
    //

    // coarse_channels
    coarse_channel::ffi::CoarseChannel::destroy_array(boxed.coarse_chans, boxed.num_coarse_chans);

    // timesteps
    timestep::ffi::TimeStep::destroy_array(boxed.timesteps, boxed.num_timesteps);

    //
    // Primitive arrays
    //

    // common timestep indices
    ffi_free_c_array(boxed.common_timestep_indices, boxed.num_common_timesteps);

    // common coarse chan indices
    ffi_free_c_array(
        boxed.common_coarse_chan_indices,
        boxed.num_common_coarse_chans,
    );

    // common good timestep indices
    ffi_free_c_array(
        boxed.common_good_timestep_indices,
        boxed.num_common_good_timesteps,
    );

    // common good coarse chan indices
    ffi_free_c_array(
        boxed.common_good_coarse_chan_indices,
        boxed.num_common_good_coarse_chans,
    );

    // provided timestep indices
    ffi_free_c_array(
        boxed.provided_timestep_indices,
        boxed.num_provided_timesteps,
    );

    // provided coarse channel indices
    ffi_free_c_array(
        boxed.provided_coarse_chan_indices,
        boxed.num_provided_coarse_chans,
    );

    // Free strings
    // (None)

    // Free main metadata struct
    drop(boxed);

    // Return success
    MWALIB_SUCCESS
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
/// * MWALIB_SUCCESS on success, non-zero on failure
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
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    let m = match CStr::from_ptr(metafits_filename).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_c_string(
                "invalid UTF-8 in metafits_filename",
                error_message as *mut c_char,
                error_message_length,
            );
            return MWALIB_FAILURE;
        }
    };

    let gpubox_slice = slice::from_raw_parts(gpubox_filenames, gpubox_count);
    let mut gpubox_files = Vec::with_capacity(gpubox_count);
    for g in gpubox_slice {
        let s = match CStr::from_ptr(*g).to_str() {
            Ok(s) => s,
            Err(_) => {
                set_c_string(
                    "invalid UTF-8 in gpubox_filename",
                    error_message as *mut c_char,
                    error_message_length,
                );
                return MWALIB_FAILURE;
            }
        };
        gpubox_files.push(s.to_string())
    }
    let context = match CorrelatorContext::new(m, &gpubox_files) {
        Ok(c) => c,
        Err(e) => {
            set_c_string(
                &format!("{}", e),
                error_message as *mut c_char,
                error_message_length,
            );
            // Return failure
            return MWALIB_FAILURE;
        }
    };
    *out_correlator_context_ptr = Box::into_raw(Box::new(context));
    // Return success
    MWALIB_SUCCESS
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
/// * MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must contain an `CorrelatorContext` object already populated via `mwalib_correlator_context_new`
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_context_display(
    correlator_context_ptr: *const CorrelatorContext,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    if correlator_context_ptr.is_null() {
        set_c_string(
            "mwalib_correlator_context() ERROR: null pointer for correlator_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }

    let context = &*correlator_context_ptr;

    println!("{}", context);

    // Return success
    MWALIB_SUCCESS
}

/// Read a single timestep / coarse channel of MWA data.
///
/// This method takes as input a timestep_index and a coarse_chan_index to return one
/// HDU of data in baseline,freq,pol,r,i format
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
///
/// * `corr_timestep_index` - index within the CorrelatorContext timestep array for the desired timestep.
///
/// * `corr_coarse_chan_index` - index within the CorrelatorContext coarse_chan array for the desired coarse channel.
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
/// * MWALIB_SUCCESS on success, MWALIB_NO_DATA_FOR_TIMESTEP_COARSE_CHAN if the combination of timestep and coarse channel has no associated data file (no data), any other non-zero code on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated object from the `mwalib_correlator_context_new` function.
/// * Caller *must* call `mwalib_correlator_context_free_read_buffer` function to release the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_context_read_by_baseline(
    correlator_context_ptr: *mut CorrelatorContext,
    corr_timestep_index: size_t,
    corr_coarse_chan_index: size_t,
    buffer_ptr: *mut c_float,
    buffer_len: size_t,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    // Load the previously-initialised context and buffer structs. Exit if
    // either of these are null.
    let corr_context = if correlator_context_ptr.is_null() {
        set_c_string(
            "mwalib_correlator_context_read_by_baseline() ERROR: null pointer for correlator_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    } else {
        &mut *correlator_context_ptr
    };

    // Don't do anything if the buffer pointer is null.
    if buffer_ptr.is_null() {
        return MWALIB_FAILURE;
    }

    let output_slice = slice::from_raw_parts_mut(buffer_ptr, buffer_len);

    // Read data into provided buffer
    match corr_context.read_by_baseline_into_buffer(
        corr_timestep_index,
        corr_coarse_chan_index,
        output_slice,
    ) {
        Ok(_) => MWALIB_SUCCESS,
        Err(e) => match e {
            GpuboxError::NoDataForTimeStepCoarseChannel {
                timestep_index: _,
                coarse_chan_index: _,
            } => {
                set_c_string(
                    &format!("{}", e),
                    error_message as *mut c_char,
                    error_message_length,
                );
                MWALIB_NO_DATA_FOR_TIMESTEP_COARSECHAN
            }
            _ => {
                set_c_string(
                    &format!("{}", e),
                    error_message as *mut c_char,
                    error_message_length,
                );
                MWALIB_FAILURE
            }
        },
    }
}

/// Read a single timestep / coarse channel of MWA data.
///
/// This method takes as input a timestep_index and a coarse_chan_index to return one
/// HDU of data in freq,baseline,pol,r,i format
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
///
/// * `corr_timestep_index` - index within the CorrelatorContext timestep array for the desired timestep. This corresponds
///   to TimeStep.get(context, N) where N is timestep_index.
///
/// * `corr_coarse_chan_index` - index within the CorrelatorContext coarse_chan array for the desired coarse channel. This corresponds
///   to CoarseChannel.get(context, N) where N is coarse_chan_index.
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
/// * MWALIB_SUCCESS on success, MWALIB_NO_DATA_FOR_TIMESTEP_COARSE_CHAN if the combination of timestep and coarse channel has no associated data file (no data), any other non-zero code on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated object from the `mwalib_correlator_context_new` function.
/// * Caller *must* call `mwalib_correlator_context_free_read_buffer` function to release the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_context_read_by_frequency(
    correlator_context_ptr: *mut CorrelatorContext,
    corr_timestep_index: size_t,
    corr_coarse_chan_index: size_t,
    buffer_ptr: *mut c_float,
    buffer_len: size_t,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    // Load the previously-initialised context and buffer structs. Exit if
    // either of these are null.
    let corr_context = if correlator_context_ptr.is_null() {
        set_c_string(
            "mwalib_correlator_context_read_by_frequency() ERROR: null pointer for correlator_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    } else {
        &mut *correlator_context_ptr
    };
    // Don't do anything if the buffer pointer is null.
    if buffer_ptr.is_null() {
        return MWALIB_FAILURE;
    }

    let output_slice = slice::from_raw_parts_mut(buffer_ptr, buffer_len);

    // Read data into provided buffer
    match corr_context.read_by_frequency_into_buffer(
        corr_timestep_index,
        corr_coarse_chan_index,
        output_slice,
    ) {
        Ok(_) => MWALIB_SUCCESS,
        Err(e) => match e {
            GpuboxError::NoDataForTimeStepCoarseChannel {
                timestep_index: _,
                coarse_chan_index: _,
            } => {
                set_c_string(
                    &format!("{}", e),
                    error_message as *mut c_char,
                    error_message_length,
                );
                MWALIB_NO_DATA_FOR_TIMESTEP_COARSECHAN
            }
            _ => {
                set_c_string(
                    &format!("{}", e),
                    error_message as *mut c_char,
                    error_message_length,
                );
                MWALIB_FAILURE
            }
        },
    }
}

/// Read a single timestep / coarse channel of MWA weights data.
///
/// This method takes as input a timestep_index and a coarse_chan_index to return one
/// HDU of weights data in baseline,pol format
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
///
/// * `corr_timestep_index` - index within the CorrelatorContext timestep array for the desired timestep.
///
/// * `corr_coarse_chan_index` - index within the CorrelatorContext coarse_chan array for the desired coarse channel.
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
/// * MWALIB_SUCCESS on success, MWALIB_NO_DATA_FOR_TIMESTEP_COARSE_CHAN if the combination of timestep and coarse channel has no associated data file (no data), any other non-zero code on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated object from the `mwalib_correlator_context_new` function.
/// * Caller *must* call `mwalib_correlator_context_free_read_buffer` function to release the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_context_read_weights_by_baseline(
    correlator_context_ptr: *mut CorrelatorContext,
    corr_timestep_index: size_t,
    corr_coarse_chan_index: size_t,
    buffer_ptr: *mut c_float,
    buffer_len: size_t,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    // Load the previously-initialised context and buffer structs. Exit if
    // either of these are null.
    let corr_context = if correlator_context_ptr.is_null() {
        set_c_string(
            "mwalib_correlator_context_read_weights_by_baseline() ERROR: null pointer for correlator_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    } else {
        &mut *correlator_context_ptr
    };

    // Don't do anything if the buffer pointer is null.
    if buffer_ptr.is_null() {
        return MWALIB_FAILURE;
    }

    let output_slice = slice::from_raw_parts_mut(buffer_ptr, buffer_len);

    // Read data into provided buffer
    match corr_context.read_weights_by_baseline_into_buffer(
        corr_timestep_index,
        corr_coarse_chan_index,
        output_slice,
    ) {
        Ok(_) => MWALIB_SUCCESS,
        Err(e) => match e {
            GpuboxError::NoDataForTimeStepCoarseChannel {
                timestep_index: _,
                coarse_chan_index: _,
            } => {
                set_c_string(
                    &format!("{}", e),
                    error_message as *mut c_char,
                    error_message_length,
                );
                MWALIB_NO_DATA_FOR_TIMESTEP_COARSECHAN
            }
            _ => {
                set_c_string(
                    &format!("{}", e),
                    error_message as *mut c_char,
                    error_message_length,
                );
                MWALIB_FAILURE
            }
        },
    }
}

/// For a given slice of correlator coarse channel indices, return a vector of the center
/// frequencies for all the fine channels in the given coarse channels
///
/// # Arguments
///
/// * `correlator_context_ptr` - pointer to an already populated `CorrelatorContext` object.
///
/// * `corr_coarse_chan_indices_array_ptr` - a pointer to an array containing correlator coarse channel indices
///   for which you want fine channels for. Does not need to be
///   contiguous.
///
/// * `corr_coarse_chan_indices_array_len` - length of `corr_coarse_chan_indices_array_ptr`.
///
/// * `out_fine_chan_freq_array_ptr` - pointer to caller-owned and allocated array of doubles to write frequencies into.
///
/// * `out_fine_chan_freq_array_len` - length of `out_fine_chan_freq_array_ptr`.
///
/// * `error_message` - pointer to already allocated buffer for any error messages to be returned to the caller.
///
/// * `error_message_length` - length of error_message char* buffer.
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `correlator_context_ptr` must point to a populated object from the `mwalib_correlator_context_new` function.
/// * Caller *must* call `mwalib_correlator_context_free_read_buffer` function to release the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_correlator_context_get_fine_chan_freqs_hz_array(
    correlator_context_ptr: *mut CorrelatorContext,
    corr_coarse_chan_indices_array_ptr: *mut size_t,
    corr_coarse_chan_indices_array_len: size_t,
    out_fine_chan_freq_array_ptr: *mut c_double,
    out_fine_chan_freq_array_len: size_t,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    // Load the previously-initialised context and buffer structs. Exit if
    // either of these are null.
    let corr_context = if correlator_context_ptr.is_null() {
        set_c_string(
            "mwalib_correlator_context_get_fine_chan_freqs_hz_array() ERROR: null pointer for correlator_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    } else {
        &mut *correlator_context_ptr
    };

    // Don't do anything if the input pointer is null.
    if corr_coarse_chan_indices_array_ptr.is_null() {
        set_c_string(
            "mwalib_correlator_context_get_fine_chan_freqs_hz_array() ERROR: null pointer for corr_coarse_chan_indices_array_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }

    // Get input buffer ready to be passed into rust method
    let input_coarse_chan_indices = slice::from_raw_parts_mut(
        corr_coarse_chan_indices_array_ptr,
        corr_coarse_chan_indices_array_len,
    );

    // Don't do anything if the buffer pointer is null.
    if out_fine_chan_freq_array_ptr.is_null() {
        set_c_string(
            "mwalib_correlator_context_get_fine_chan_freqs_hz_array() ERROR: null pointer for out_fine_chan_freq_array_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }

    // Get output buffer ready
    let output_slice =
        slice::from_raw_parts_mut(out_fine_chan_freq_array_ptr, out_fine_chan_freq_array_len);

    // Sanity check the length
    let expected_output_len = corr_coarse_chan_indices_array_len
        * corr_context.metafits_context.num_corr_fine_chans_per_coarse;
    if output_slice.len() != expected_output_len {
        set_c_string(
            &format!("mwalib_correlator_context_get_fine_chan_freqs_hz_array() ERROR: number of elements in out_fine_chan_freq_array_ptr does not match expected value {}", expected_output_len),
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }

    // Read data into provided buffer
    let fine_chans = corr_context.get_fine_chan_freqs_hz_array(input_coarse_chan_indices);

    // Write the fine chans back into the provided array
    output_slice.clone_from_slice(&fine_chans);

    MWALIB_SUCCESS
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
/// * MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `CorrelatorContext` object
/// * `correlator_context_ptr` must point to a populated `CorrelatorContext` object from the `mwalib_correlator_context_new` function.
/// * `correlator_context_ptr` must not have already been freed.
#[no_mangle]
#[allow(unused_must_use)]
pub unsafe extern "C" fn mwalib_correlator_context_free(
    correlator_context_ptr: *mut CorrelatorContext,
) -> i32 {
    if correlator_context_ptr.is_null() {
        return MWALIB_SUCCESS;
    }
    // Release correlator context if applicable
    drop(Box::from_raw(correlator_context_ptr));

    // Return success
    MWALIB_SUCCESS
}
