// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ffi::c_char;

use libc::size_t;

use crate::{
    coarse_channel,
    ffi::{
        ffi_array_to_boxed_slice, set_c_string, MWALIB_FAILURE,
        MWALIB_NO_DATA_FOR_TIMESTEP_COARSECHAN, MWALIB_SUCCESS,
    },
    timestep, voltage_context, MWAVersion, VoltageFileError,
};

///
/// C Representation of the `VoltageContext` metadata
///
#[repr(C)]
pub struct VoltageMetadata {
    /// Version of the correlator format
    pub mwa_version: MWAVersion,
    /// This is an array of all known timesteps (union of metafits and provided timesteps from data files). The only exception is when the metafits timesteps are
    /// offset from the provided timesteps, in which case see description in `timestep::populate_metafits_provided_superset_of_timesteps`.
    pub timesteps: *mut timestep::ffi::TimeStep,
    /// Number of timesteps in the timestep array
    pub num_timesteps: usize,
    /// The number of millseconds interval between timestep indices
    pub timestep_duration_ms: u64,
    /// Vector of coarse channels which is the effectively the same as the metafits provided coarse channels
    pub coarse_chans: *mut coarse_channel::ffi::CoarseChannel,
    /// Number of coarse channels after we've validated the input voltage files
    pub num_coarse_chans: usize,
    /// Number of common timesteps
    pub num_common_timesteps: usize,
    /// Vector of (in)common timestep indices
    pub common_timestep_indices: *mut usize,
    /// Number of common coarse chans
    pub num_common_coarse_chans: usize,
    /// Vector of (in)common coarse channel indices
    pub common_coarse_chan_indices: *mut usize,
    /// The start of the observation (the time that is common to all
    /// provided data files).
    pub common_start_unix_time_ms: u64,
    /// `end_unix_time_ms` is the common end time of the observation
    /// i.e. start time of last common timestep plus integration time.
    pub common_end_unix_time_ms: u64,
    /// `start_unix_time_ms` but in GPS milliseconds
    pub common_start_gps_time_ms: u64,
    /// `end_unix_time_ms` but in GPS milliseconds
    pub common_end_gps_time_ms: u64,
    /// Total duration of common timesteps
    pub common_duration_ms: u64,
    /// Total bandwidth of the common coarse channels
    pub common_bandwidth_hz: u32,
    /// Number of common timesteps only including timesteps after the quack time
    pub num_common_good_timesteps: usize,
    /// Vector of (in)common timestep indices only including timesteps after the quack time
    pub common_good_timestep_indices: *mut usize,
    /// Number of common coarse channels only including timesteps after the quack time
    pub num_common_good_coarse_chans: usize,
    /// Vector of (in)common coarse channel indices only including timesteps after the quack time
    pub common_good_coarse_chan_indices: *mut usize,
    /// The start of the observation (the time that is common to all
    /// provided data files) only including timesteps after the quack time
    pub common_good_start_unix_time_ms: u64,
    /// `end_unix_time_ms` is the common end time of the observation only including timesteps after the quack time
    /// i.e. start time of last common timestep plus integration time.
    pub common_good_end_unix_time_ms: u64,
    /// `common_good_start_unix_time_ms` but in GPS milliseconds
    pub common_good_start_gps_time_ms: u64,
    /// `common_good_end_unix_time_ms` but in GPS milliseconds
    pub common_good_end_gps_time_ms: u64,
    /// Total duration of common_good timesteps
    pub common_good_duration_ms: u64,
    /// Total bandwidth of the common coarse channels only including timesteps after the quack time
    pub common_good_bandwidth_hz: u32,
    /// Number of provided timestep indices we have at least *some* data for
    pub num_provided_timesteps: usize,
    /// The indices of any timesteps which we have *some* data for
    pub provided_timestep_indices: *mut usize,
    /// Number of provided coarse channel indices we have at least *some* data for
    pub num_provided_coarse_chans: usize,
    /// The indices of any coarse channels which we have *some* data for
    pub provided_coarse_chan_indices: *mut usize,
    /// Bandwidth of each coarse channel
    pub coarse_chan_width_hz: u32,
    /// Volatge fine_chan_resolution (if applicable- MWA legacy is 10 kHz, MWAX is unchannelised i.e. the full coarse channel width)
    pub fine_chan_width_hz: u32,
    /// Number of fine channels in each coarse channel
    pub num_fine_chans_per_coarse: usize,
    /// Number of bytes in each sample (a sample is a complex, thus includes r and i)
    pub sample_size_bytes: u64,
    /// Number of voltage blocks per timestep
    pub num_voltage_blocks_per_timestep: usize,
    /// Number of voltage blocks of samples in each second of data    
    pub num_voltage_blocks_per_second: usize,
    /// Number of samples in each voltage_blocks for each second of data per rf_input * fine_chans * real|imag
    pub num_samples_per_voltage_block: usize,
    /// The size of each voltage block    
    pub voltage_block_size_bytes: u64,
    /// Number of bytes used to store delays - for MWAX this is the same as a voltage block size, for legacy it is 0
    pub delay_block_size_bytes: u64,
    /// The amount of bytes to skip before getting into real data within the voltage files
    pub data_file_header_size_bytes: u64,
    /// Expected voltage file size
    pub expected_voltage_data_file_size_bytes: u64,
}

/// This returns a struct containing the `VoltageContext` metadata
///
/// # Arguments
///
/// * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object.
///
/// * `out_voltage_metadata_ptr` - A Rust-owned populated `VoltageMetadata` struct. Free with `mwalib_voltage_metadata_free`.
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
/// * `voltage_context_ptr` must point to a populated `VoltageContext` object from the `mwalib_voltage_context_new` function.
/// * Caller must call `mwalib_voltage_metadata_free` once finished, to free the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_voltage_metadata_get(
    voltage_context_ptr: *mut voltage_context::VoltageContext,
    out_voltage_metadata_ptr: &mut *mut VoltageMetadata,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    if voltage_context_ptr.is_null() {
        set_c_string(
            "mwalib_voltage_metadata_get() ERROR: Warning: null pointer for voltage_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }
    // Get the voltage context object from the raw pointer passed in
    let context = &*voltage_context_ptr;

    // Populate voltage coarse channels
    let mut coarse_chan_vec: Vec<coarse_channel::ffi::CoarseChannel> = Vec::new();

    for item in context.coarse_chans.iter() {
        let out_item = {
            let coarse_channel::CoarseChannel {
                corr_chan_number,
                rec_chan_number,
                gpubox_number,
                chan_width_hz,
                chan_start_hz,
                chan_centre_hz,
                chan_end_hz,
            } = item;
            coarse_channel::ffi::CoarseChannel {
                corr_chan_number: *corr_chan_number,
                rec_chan_number: *rec_chan_number,
                gpubox_number: *gpubox_number,
                chan_width_hz: *chan_width_hz,
                chan_start_hz: *chan_start_hz,
                chan_centre_hz: *chan_centre_hz,
                chan_end_hz: *chan_end_hz,
            }
        };

        coarse_chan_vec.push(out_item);
    }

    // Populate voltage timesteps
    let mut timestep_vec: Vec<timestep::ffi::TimeStep> = Vec::new();

    for item in context.timesteps.iter() {
        let out_item = {
            let timestep::TimeStep {
                unix_time_ms,
                gps_time_ms,
            } = item;
            timestep::ffi::TimeStep {
                unix_time_ms: *unix_time_ms,
                gps_time_ms: *gps_time_ms,
            }
        };
        timestep_vec.push(out_item);
    }

    // Populate the rust owned data structure with data from the voltage context
    // We explicitly break out the attributes so at compile time it will let us know
    // if there have been new fields added to the rust struct, then we can choose to
    // ignore them (with _) or add that field to the FFI struct.
    let out_context = {
        let voltage_context::VoltageContext {
            metafits_context: _, // This is provided by the seperate metafits_metadata struct in FFI
            mwa_version,
            num_timesteps,
            timesteps: _, // This is populated seperately
            timestep_duration_ms,
            num_coarse_chans,
            coarse_chans: _, // This is populated seperately
            common_timestep_indices,
            num_common_timesteps,
            common_coarse_chan_indices,
            num_common_coarse_chans,
            common_start_unix_time_ms,
            common_end_unix_time_ms,
            common_start_gps_time_ms,
            common_end_gps_time_ms,
            common_duration_ms,
            common_bandwidth_hz,
            common_good_timestep_indices,
            num_common_good_timesteps,
            common_good_coarse_chan_indices,
            num_common_good_coarse_chans,
            common_good_start_unix_time_ms,
            common_good_end_unix_time_ms,
            common_good_start_gps_time_ms,
            common_good_end_gps_time_ms,
            common_good_duration_ms,
            common_good_bandwidth_hz,
            provided_timestep_indices,
            num_provided_timesteps: num_provided_timestep_indices,
            provided_coarse_chan_indices,
            num_provided_coarse_chans: num_provided_coarse_chan_indices,
            coarse_chan_width_hz,
            fine_chan_width_hz,
            num_fine_chans_per_coarse,
            sample_size_bytes,
            num_voltage_blocks_per_timestep,
            num_voltage_blocks_per_second,
            num_samples_per_voltage_block,
            voltage_block_size_bytes,
            delay_block_size_bytes,
            data_file_header_size_bytes,
            expected_voltage_data_file_size_bytes,
            voltage_batches: _, // This is currently not provided to FFI as it is private
            voltage_time_map: _, // This is currently not provided to FFI as it is private
        } = context;
        VoltageMetadata {
            mwa_version: *mwa_version,
            timesteps: ffi_array_to_boxed_slice(timestep_vec),
            num_timesteps: *num_timesteps,
            timestep_duration_ms: *timestep_duration_ms,
            coarse_chans: ffi_array_to_boxed_slice(coarse_chan_vec),
            num_coarse_chans: *num_coarse_chans,
            num_common_timesteps: *num_common_timesteps,
            common_timestep_indices: ffi_array_to_boxed_slice(common_timestep_indices.clone()),
            num_common_coarse_chans: *num_common_coarse_chans,
            common_coarse_chan_indices: ffi_array_to_boxed_slice(
                common_coarse_chan_indices.clone(),
            ),
            common_start_unix_time_ms: *common_start_unix_time_ms,
            common_end_unix_time_ms: *common_end_unix_time_ms,
            common_start_gps_time_ms: *common_start_gps_time_ms,
            common_end_gps_time_ms: *common_end_gps_time_ms,
            common_duration_ms: *common_duration_ms,
            common_bandwidth_hz: *common_bandwidth_hz,
            num_common_good_timesteps: *num_common_good_timesteps,
            common_good_timestep_indices: ffi_array_to_boxed_slice(
                common_good_timestep_indices.clone(),
            ),
            num_common_good_coarse_chans: *num_common_good_coarse_chans,
            common_good_coarse_chan_indices: ffi_array_to_boxed_slice(
                common_good_coarse_chan_indices.clone(),
            ),
            common_good_start_unix_time_ms: *common_good_start_unix_time_ms,
            common_good_end_unix_time_ms: *common_good_end_unix_time_ms,
            common_good_start_gps_time_ms: *common_good_start_gps_time_ms,
            common_good_end_gps_time_ms: *common_good_end_gps_time_ms,
            common_good_duration_ms: *common_good_duration_ms,
            common_good_bandwidth_hz: *common_good_bandwidth_hz,
            num_provided_timesteps: *num_provided_timestep_indices,
            provided_timestep_indices: ffi_array_to_boxed_slice(provided_timestep_indices.clone()),
            num_provided_coarse_chans: *num_provided_coarse_chan_indices,
            provided_coarse_chan_indices: ffi_array_to_boxed_slice(
                provided_coarse_chan_indices.clone(),
            ),
            coarse_chan_width_hz: *coarse_chan_width_hz,
            fine_chan_width_hz: *fine_chan_width_hz,
            num_fine_chans_per_coarse: *num_fine_chans_per_coarse,
            sample_size_bytes: *sample_size_bytes,
            num_voltage_blocks_per_timestep: *num_voltage_blocks_per_timestep,
            num_voltage_blocks_per_second: *num_voltage_blocks_per_second,
            num_samples_per_voltage_block: *num_samples_per_voltage_block,
            voltage_block_size_bytes: *voltage_block_size_bytes,
            delay_block_size_bytes: *delay_block_size_bytes,
            data_file_header_size_bytes: *data_file_header_size_bytes,
            expected_voltage_data_file_size_bytes: *expected_voltage_data_file_size_bytes,
        }
    };

    // Pass out the pointer to the rust owned data structure
    *out_voltage_metadata_ptr = Box::into_raw(Box::new(out_context));

    // Return success
    MWALIB_SUCCESS
}

/// Free a previously-allocated `VoltageMetadata` struct.
///
/// # Arguments
///
/// * `voltage_metadata_ptr` - pointer to an already populated `VoltageMetadata` object
///
///
/// # Returns
///
/// * MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `VoltageMetadata` object
/// * `voltage_metadata_ptr` must point to a populated `VoltageMetadata` object from the `mwalib_voltage_metadata_get` function.
/// * `voltage_metadata_ptr` must not have already been freed.
#[no_mangle]
pub unsafe extern "C" fn mwalib_voltage_metadata_free(
    voltage_metadata_ptr: *mut VoltageMetadata,
) -> i32 {
    if voltage_metadata_ptr.is_null() {
        return MWALIB_SUCCESS;
    }

    //
    // free any other members first
    //

    // coarse_channels
    if !(*voltage_metadata_ptr).coarse_chans.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [coarse_channel::ffi::CoarseChannel] = slice::from_raw_parts_mut(
            (*voltage_metadata_ptr).coarse_chans,
            (*voltage_metadata_ptr).num_coarse_chans,
        );
        drop(Box::from_raw(slice));
    }

    // timesteps
    if !(*voltage_metadata_ptr).timesteps.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [timestep::ffi::TimeStep] = slice::from_raw_parts_mut(
            (*voltage_metadata_ptr).timesteps,
            (*voltage_metadata_ptr).num_timesteps,
        );
        drop(Box::from_raw(slice));
    }

    // common timestep indices
    if !(*voltage_metadata_ptr).common_timestep_indices.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [usize] = slice::from_raw_parts_mut(
            (*voltage_metadata_ptr).common_timestep_indices,
            (*voltage_metadata_ptr).num_common_timesteps,
        );
        drop(Box::from_raw(slice));
    }

    // common coarse chan indices
    if !(*voltage_metadata_ptr).common_coarse_chan_indices.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [usize] = slice::from_raw_parts_mut(
            (*voltage_metadata_ptr).common_coarse_chan_indices,
            (*voltage_metadata_ptr).num_common_coarse_chans,
        );
        drop(Box::from_raw(slice));
    }

    // common good timestep indices
    if !(*voltage_metadata_ptr)
        .common_good_timestep_indices
        .is_null()
    {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [usize] = slice::from_raw_parts_mut(
            (*voltage_metadata_ptr).common_good_timestep_indices,
            (*voltage_metadata_ptr).num_common_good_timesteps,
        );
        drop(Box::from_raw(slice));
    }

    // common good coarse chan indices
    if !(*voltage_metadata_ptr)
        .common_good_coarse_chan_indices
        .is_null()
    {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [usize] = slice::from_raw_parts_mut(
            (*voltage_metadata_ptr).common_good_coarse_chan_indices,
            (*voltage_metadata_ptr).num_common_good_coarse_chans,
        );
        drop(Box::from_raw(slice));
    }

    // provided timestep indices
    if !(*voltage_metadata_ptr).provided_timestep_indices.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [usize] = slice::from_raw_parts_mut(
            (*voltage_metadata_ptr).provided_timestep_indices,
            (*voltage_metadata_ptr).num_provided_timesteps,
        );
        drop(Box::from_raw(slice));
    }

    // provided coarse channel indices
    if !(*voltage_metadata_ptr)
        .provided_coarse_chan_indices
        .is_null()
    {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [usize] = slice::from_raw_parts_mut(
            (*voltage_metadata_ptr).provided_coarse_chan_indices,
            (*voltage_metadata_ptr).num_provided_coarse_chans,
        );
        drop(Box::from_raw(slice));
    }

    // Free main metadata struct
    drop(Box::from_raw(voltage_metadata_ptr));

    // Return success
    MWALIB_SUCCESS
}

/// For a given slice of voltage coarse channel indices, return a vector of the center
/// frequencies for all the fine channels in the given coarse channels
///
/// # Arguments
///
/// * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object.
///
/// * `corr_coarse_chan_indices_array_ptr` - a pointer to an array containing correlator coarse channel indices
///   for which you want fine channels for. Does not need to be contiguous.
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
pub unsafe extern "C" fn mwalib_voltage_context_get_fine_chan_freqs_hz_array(
    voltage_context_ptr: *mut VoltageContext,
    volt_coarse_chan_indices_array_ptr: *mut size_t,
    volt_coarse_chan_indices_array_len: size_t,
    out_fine_chan_freq_array_ptr: *mut c_double,
    out_fine_chan_freq_array_len: size_t,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    // Load the previously-initialised context and buffer structs. Exit if
    // either of these are null.
    let volt_context = if voltage_context_ptr.is_null() {
        set_c_string(
            "mwalib_voltage_context_get_fine_chan_freqs_hz_array() ERROR: null pointer for voltage_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    } else {
        &mut *voltage_context_ptr
    };

    // Don't do anything if the input pointer is null.
    if volt_coarse_chan_indices_array_ptr.is_null() {
        set_c_string(
            "mwalib_voltage_context_get_fine_chan_freqs_hz_array() ERROR: null pointer for volt_coarse_chan_indices_array_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }

    // Get input buffer ready to be passed into rust method
    let input_coarse_chan_indices = slice::from_raw_parts_mut(
        volt_coarse_chan_indices_array_ptr,
        volt_coarse_chan_indices_array_len,
    );

    // Don't do anything if the buffer pointer is null.
    if out_fine_chan_freq_array_ptr.is_null() {
        set_c_string(
            "mwalib_voltage_context_get_fine_chan_freqs_hz_array() ERROR: null pointer for out_fine_chan_freq_array_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }

    // Get output buffer ready
    let output_slice =
        slice::from_raw_parts_mut(out_fine_chan_freq_array_ptr, out_fine_chan_freq_array_len);

    // Sanity check the length
    let expected_output_len = volt_coarse_chan_indices_array_len
        * volt_context.metafits_context.num_corr_fine_chans_per_coarse;
    if output_slice.len() != expected_output_len {
        set_c_string(
            &format!("mwalib_voltage_context_get_fine_chan_freqs_hz_array() ERROR: number of elements in out_fine_chan_freq_array_ptr does not match expected value {}", expected_output_len),
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }

    // Read data into provided buffer
    let fine_chans = volt_context.get_fine_chan_freqs_hz_array(input_coarse_chan_indices);

    // Write the fine chans back into the provided array
    output_slice.clone_from_slice(&fine_chans);

    MWALIB_SUCCESS
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
/// * MWALIB_SUCCESS on success, non-zero on failure
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

    let voltage_slice = slice::from_raw_parts(voltage_filenames, voltage_file_count);
    let mut voltage_files = Vec::with_capacity(voltage_file_count);
    for v in voltage_slice {
        let s = match CStr::from_ptr(*v).to_str() {
            Ok(s) => s,
            Err(_) => {
                set_c_string(
                    "invalid UTF-8 in voltage_filename",
                    error_message as *mut c_char,
                    error_message_length,
                );
                return MWALIB_FAILURE;
            }
        };
        voltage_files.push(s.to_string())
    }

    let context = match VoltageContext::new(m, &voltage_files) {
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
    *out_voltage_context_ptr = Box::into_raw(Box::new(context));
    // Return success
    MWALIB_SUCCESS
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
/// * MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `voltage_context_ptr` must contain an `VoltageContext` object already populated via `mwalib_voltage_context_new`
#[no_mangle]
pub unsafe extern "C" fn mwalib_voltage_context_display(
    voltage_context_ptr: *const VoltageContext,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    if voltage_context_ptr.is_null() {
        set_c_string(
            "mwalib_voltage_context() ERROR: null pointer for voltage_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }

    let context = &*voltage_context_ptr;

    println!("{}", context);

    // Return success
    MWALIB_SUCCESS
}

/// Read a single timestep / coarse channel of MWA voltage data.
///
/// This method takes as input a timestep_index and a coarse_chan_index to return one
/// file-worth of voltage data.
///
/// The output data are in the format:
///
/// MWA Recombined VCS:
///
/// NOTE: antennas are in tile_id order for recombined VCS...
///
/// sample[0]|finechan[0]|antenna[0]|X|sample
/// sample[0]|finechan[0]|antenna[0]|Y|sample    
/// ...
/// sample[0]|finechan[0]|antenna[127]|X|sample
/// sample[0]|finechan[0]|antenna[127]|Y|sample
/// ...
/// sample[0]|finechan[1]|antenna[0]|X|sample
/// sample[0]|finechan[1]|antenna[0]|Y|sample
/// ...
/// sample[0]|finechan[127]|antenna[127]|X|sample
/// sample[0]|finechan[127]|antenna[127]|Y|sample
/// ...
/// sample[1]|finechan[0]|antenna[0]|X|sample
/// sample[1]|finechan[0]|antenna[0]|Y|sample        
///
/// MWAX:
/// block[0]antenna[0]|pol[0]|sample[0]...sample[63999]
/// block[0]antenna[0]|pol[1]|sample[0]...sample[63999]
/// block[0]antenna[1]|pol[0]|sample[0]...sample[63999]
/// block[0]antenna[1]|pol[1]|sample[0]...sample[63999]
/// ...
/// block[0]antenna[ntiles-1]|pol[1]|sample[0]...sample[63999]    
/// block[1]antenna[0]|pol[0]|sample[0]...sample[63999]
/// ...
/// block[19]antenna[ntiles-1]|pol[1]|sample[0]...sample[63999]
///
/// File format information:
/// type    tiles   pols    fine ch bytes/samp  samples/block   block size  blocks  header  delay size  data size   file size   seconds/file    size/sec
/// =====================================================================================================================================================
/// Lgeacy  128     2       128     1           10000           327680000   1       0       0           327680000   327680000   1               327680000
/// MWAX    128     2       1       2           64000           32768000    160     4096    32768000    5242880000  5275652096  8               659456512
/// NOTE: 'sample' refers to a complex value per tile/pol/chan/time. So legacy stores r/i as a byte (4bits r + 4bits i), mwax as 1 byte real, 1 byte imag.
///
///
/// # Arguments
///
/// * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object.
///
/// * `voltage_timestep_index` - index within the voltage timestep array for the desired timestep.
///
/// * `voltage_coarse_chan_index` - index within the voltage coarse_chan array for the desired coarse channel.
///
/// * `buffer_ptr` - pointer to caller-owned and allocated buffer of signed bytes to write data into. Buffer must be large enough
///   for all of the data. Calculate the buffer size in bytes using:
///   vcontext.voltage_block_size_bytes * vcontext.num_voltage_blocks_per_timestep
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
/// * `voltage_context_ptr` must point to a populated object from the `mwalib_voltage_context_new` function.
#[no_mangle]
pub unsafe extern "C" fn mwalib_voltage_context_read_file2(
    voltage_context_ptr: *mut VoltageContext,
    voltage_timestep_index: size_t,
    voltage_coarse_chan_index: size_t,
    buffer_ptr: *mut c_schar,
    buffer_len: size_t,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    // Load the previously-initialised context and buffer structs. Exit if
    // either of these are null.
    let voltage_context = if voltage_context_ptr.is_null() {
        set_c_string(
            "mwalib_voltage_context_read_by_file() ERROR: null pointer for voltage_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    } else {
        &mut *voltage_context_ptr
    };

    // Don't do anything if the buffer pointer is null.
    if buffer_ptr.is_null() {
        return MWALIB_FAILURE;
    }

    let output_slice: &mut [i8] = slice::from_raw_parts_mut(buffer_ptr, buffer_len);

    // Read data in.
    match voltage_context.read_file2(
        voltage_timestep_index,
        voltage_coarse_chan_index,
        output_slice,
    ) {
        Ok(_) => MWALIB_SUCCESS,
        Err(e) => match e {
            VoltageFileError::NoDataForTimeStepCoarseChannel {
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

/// Read a single timestep / coarse channel of MWA voltage data.
///
/// This method takes as input a timestep_index and a coarse_chan_index to return one
/// file-worth of voltage data.
///
/// The output data are in the format:
///
/// MWA Recombined VCS:
///
/// NOTE: antennas are in tile_id order for recombined VCS...
///
/// sample[0]|finechan[0]|antenna[0]|X|sample
/// sample[0]|finechan[0]|antenna[0]|Y|sample    
/// ...
/// sample[0]|finechan[0]|antenna[127]|X|sample
/// sample[0]|finechan[0]|antenna[127]|Y|sample
/// ...
/// sample[0]|finechan[1]|antenna[0]|X|sample
/// sample[0]|finechan[1]|antenna[0]|Y|sample
/// ...
/// sample[0]|finechan[127]|antenna[127]|X|sample
/// sample[0]|finechan[127]|antenna[127]|Y|sample
/// ...
/// sample[1]|finechan[0]|antenna[0]|X|sample
/// sample[1]|finechan[0]|antenna[0]|Y|sample        
///
/// MWAX:
/// block[0]antenna[0]|pol[0]|sample[0]...sample[63999]
/// block[0]antenna[0]|pol[1]|sample[0]...sample[63999]
/// block[0]antenna[1]|pol[0]|sample[0]...sample[63999]
/// block[0]antenna[1]|pol[1]|sample[0]...sample[63999]
/// ...
/// block[0]antenna[ntiles-1]|pol[1]|sample[0]...sample[63999]    
/// block[1]antenna[0]|pol[0]|sample[0]...sample[63999]
/// ...
/// block[19]antenna[ntiles-1]|pol[1]|sample[0]...sample[63999]
///
/// File format information:
/// type    tiles   pols    fine ch bytes/samp  samples/block   block size  blocks  header  delay size  data size   file size   seconds/file    size/sec
/// =====================================================================================================================================================
/// Lgeacy  128     2       128     1           10000           327680000   1       0       0           327680000   327680000   1               327680000
/// MWAX    128     2       1       2           64000           32768000    160     4096    32768000    5242880000  5275652096  8               659456512
/// NOTE: 'sample' refers to a complex value per tile/pol/chan/time. So legacy stores r/i as a byte (4bits r + 4bits i), mwax as 1 byte real, 1 byte imag.
///
///
/// # Arguments
///
/// * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object.
///
/// * `voltage_timestep_index` - index within the voltage timestep array for the desired timestep.
///
/// * `voltage_coarse_chan_index` - index within the voltage coarse_chan array for the desired coarse channel.
///
/// * `buffer_ptr` - pointer to caller-owned and allocated buffer of signed bytes to write data into. Buffer must be large enough
///   for all of the data. Calculate the buffer size in bytes using:
///   vcontext.voltage_block_size_bytes * vcontext.num_voltage_blocks_per_timestep
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
/// * `voltage_context_ptr` must point to a populated object from the `mwalib_voltage_context_new` function.
#[no_mangle]
pub unsafe extern "C" fn mwalib_voltage_context_read_file(
    voltage_context_ptr: *mut VoltageContext,
    voltage_timestep_index: size_t,
    voltage_coarse_chan_index: size_t,
    buffer_ptr: *mut c_schar,
    buffer_len: size_t,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    // Load the previously-initialised context and buffer structs. Exit if
    // either of these are null.
    let voltage_context = if voltage_context_ptr.is_null() {
        set_c_string(
            "mwalib_voltage_context_read_by_file() ERROR: null pointer for voltage_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    } else {
        &mut *voltage_context_ptr
    };

    // Don't do anything if the buffer pointer is null.
    if buffer_ptr.is_null() {
        return MWALIB_FAILURE;
    }

    let output_slice: &mut [i8] = slice::from_raw_parts_mut(buffer_ptr, buffer_len);

    // Read data in.
    match voltage_context.read_file(
        voltage_timestep_index,
        voltage_coarse_chan_index,
        output_slice,
    ) {
        Ok(_) => MWALIB_SUCCESS,
        Err(e) => match e {
            VoltageFileError::NoDataForTimeStepCoarseChannel {
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

/// Read a single second / coarse channel of MWA voltage data.
///
/// This method takes as input a gps_time (in seconds) and a coarse_chan_index to return one
/// second-worth of voltage data.
///
/// The output data are in the format:
///
/// MWA Recombined VCS:
///
/// NOTE: antennas are in tile_id order for recombined VCS...
///
/// sample[0]|finechan[0]|antenna[0]|X|sample
/// sample[0]|finechan[0]|antenna[0]|Y|sample    
/// ...
/// sample[0]|finechan[0]|antenna[127]|X|sample
/// sample[0]|finechan[0]|antenna[127]|Y|sample
/// ...
/// sample[0]|finechan[1]|antenna[0]|X|sample
/// sample[0]|finechan[1]|antenna[0]|Y|sample
/// ...
/// sample[0]|finechan[127]|antenna[127]|X|sample
/// sample[0]|finechan[127]|antenna[127]|Y|sample
/// ...
/// sample[1]|finechan[0]|antenna[0]|X|sample
/// sample[1]|finechan[0]|antenna[0]|Y|sample        
///
/// MWAX:
/// block[0]antenna[0]|pol[0]|sample[0]...sample[63999]
/// block[0]antenna[0]|pol[1]|sample[0]...sample[63999]
/// block[0]antenna[1]|pol[0]|sample[0]...sample[63999]
/// block[0]antenna[1]|pol[1]|sample[0]...sample[63999]
/// ...
/// block[0]antenna[ntiles-1]|pol[1]|sample[0]...sample[63999]    
/// block[1]antenna[0]|pol[0]|sample[0]...sample[63999]
/// ...
/// block[19]antenna[ntiles-1]|pol[1]|sample[0]...sample[63999]
///
/// File format information:
/// type    tiles   pols    fine ch bytes/samp  samples/block   block size  blocks  header  delay size  data size   file size   seconds/file    size/sec
/// =====================================================================================================================================================
/// Lgeacy  128     2       128     1           10000           327680000   1       0       0           327680000   327680000   1               327680000
/// MWAX    128     2       1       2           64000           32768000    160     4096    32768000    5242880000  5275652096  8               659456512
/// NOTE: 'sample' refers to a complex value per tile/pol/chan/time. So legacy stores r/i as a byte (4bits r + 4bits i), mwax as 1 byte real, 1 byte imag.
///
/// # Arguments
///
/// * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object.
///
/// * `gps_second_start` - GPS second which to start getting data at.
///
/// * `gps_second_count` - How many GPS seconds of data to get (inclusive).
///
/// * `voltage_coarse_chan_index` - index within the coarse_chan array for the desired coarse channel.
///
/// * `buffer_ptr` - pointer to caller-owned and allocated buffer of signed bytes to write data into. Buffer must be large enough
///   for all of the data. Calculate the buffer size in bytes using:
///   (vcontext.voltage_block_size_bytes * vcontext.num_voltage_blocks_per_second) * gps_second_count
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
/// * `voltage_context_ptr` must point to a populated object from the `mwalib_voltage_context_new` function.
#[no_mangle]
pub unsafe extern "C" fn mwalib_voltage_context_read_second(
    voltage_context_ptr: *mut VoltageContext,
    gps_second_start: u64,
    gps_second_count: size_t,
    voltage_coarse_chan_index: size_t,
    buffer_ptr: *mut c_schar,
    buffer_len: size_t,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    // Load the previously-initialised context and buffer structs. Exit if
    // either of these are null.
    let voltage_context = if voltage_context_ptr.is_null() {
        set_c_string(
            "mwalib_voltage_context_read_by_file() ERROR: null pointer for voltage_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    } else {
        &mut *voltage_context_ptr
    };

    // Don't do anything if the buffer pointer is null.
    if buffer_ptr.is_null() {
        return MWALIB_FAILURE;
    }

    let output_slice: &mut [i8] = slice::from_raw_parts_mut(buffer_ptr, buffer_len);

    // Read data in.
    match voltage_context.read_second(
        gps_second_start,
        gps_second_count,
        voltage_coarse_chan_index,
        output_slice,
    ) {
        Ok(_) => MWALIB_SUCCESS,
        Err(e) => match e {
            VoltageFileError::NoDataForTimeStepCoarseChannel {
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

/// Read a single second / coarse channel of MWA voltage data.
///
/// This method takes as input a gps_time (in seconds) and a coarse_chan_index to return one
/// second-worth of voltage data.
///
/// The output data are in the format:
///
/// MWA Recombined VCS:
///
/// NOTE: antennas are in tile_id order for recombined VCS...
///
/// sample[0]|finechan[0]|antenna[0]|X|sample
/// sample[0]|finechan[0]|antenna[0]|Y|sample    
/// ...
/// sample[0]|finechan[0]|antenna[127]|X|sample
/// sample[0]|finechan[0]|antenna[127]|Y|sample
/// ...
/// sample[0]|finechan[1]|antenna[0]|X|sample
/// sample[0]|finechan[1]|antenna[0]|Y|sample
/// ...
/// sample[0]|finechan[127]|antenna[127]|X|sample
/// sample[0]|finechan[127]|antenna[127]|Y|sample
/// ...
/// sample[1]|finechan[0]|antenna[0]|X|sample
/// sample[1]|finechan[0]|antenna[0]|Y|sample        
///
/// MWAX:
/// block[0]antenna[0]|pol[0]|sample[0]...sample[63999]
/// block[0]antenna[0]|pol[1]|sample[0]...sample[63999]
/// block[0]antenna[1]|pol[0]|sample[0]...sample[63999]
/// block[0]antenna[1]|pol[1]|sample[0]...sample[63999]
/// ...
/// block[0]antenna[ntiles-1]|pol[1]|sample[0]...sample[63999]    
/// block[1]antenna[0]|pol[0]|sample[0]...sample[63999]
/// ...
/// block[19]antenna[ntiles-1]|pol[1]|sample[0]...sample[63999]
///
/// File format information:
/// type    tiles   pols    fine ch bytes/samp  samples/block   block size  blocks  header  delay size  data size   file size   seconds/file    size/sec
/// =====================================================================================================================================================
/// Lgeacy  128     2       128     1           10000           327680000   1       0       0           327680000   327680000   1               327680000
/// MWAX    128     2       1       2           64000           32768000    160     4096    32768000    5242880000  5275652096  8               659456512
/// NOTE: 'sample' refers to a complex value per tile/pol/chan/time. So legacy stores r/i as a byte (4bits r + 4bits i), mwax as 1 byte real, 1 byte imag.
///
/// # Arguments
///
/// * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object.
///
/// * `gps_second_start` - GPS second which to start getting data at.
///
/// * `gps_second_count` - How many GPS seconds of data to get (inclusive).
///
/// * `voltage_coarse_chan_index` - index within the coarse_chan array for the desired coarse channel.
///
/// * `buffer_ptr` - pointer to caller-owned and allocated buffer of signed bytes to write data into. Buffer must be large enough
///   for all of the data. Calculate the buffer size in bytes using:
///   (vcontext.voltage_block_size_bytes * vcontext.num_voltage_blocks_per_second) * gps_second_count
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
/// * `voltage_context_ptr` must point to a populated object from the `mwalib_voltage_context_new` function.
#[no_mangle]
pub unsafe extern "C" fn mwalib_voltage_context_read_second2(
    voltage_context_ptr: *mut VoltageContext,
    gps_second_start: u64,
    gps_second_count: size_t,
    voltage_coarse_chan_index: size_t,
    buffer_ptr: *mut c_schar,
    buffer_len: size_t,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    // Load the previously-initialised context and buffer structs. Exit if
    // either of these are null.
    let voltage_context = if voltage_context_ptr.is_null() {
        set_c_string(
            "mwalib_voltage_context_read_by_file() ERROR: null pointer for voltage_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    } else {
        &mut *voltage_context_ptr
    };

    // Don't do anything if the buffer pointer is null.
    if buffer_ptr.is_null() {
        return MWALIB_FAILURE;
    }

    let output_slice: &mut [i8] = slice::from_raw_parts_mut(buffer_ptr, buffer_len);

    // Read data in.
    match voltage_context.read_second2(
        gps_second_start,
        gps_second_count,
        voltage_coarse_chan_index,
        output_slice,
    ) {
        Ok(_) => MWALIB_SUCCESS,
        Err(e) => match e {
            VoltageFileError::NoDataForTimeStepCoarseChannel {
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

/// Free a previously-allocated `VoltageContext` struct (and it's members).
///
/// # Arguments
///
/// * `voltage_context_ptr` - pointer to an already populated `VoltageContext` object
///
///
/// # Returns
///
/// * MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `VoltageContext` object
/// * `voltage_context_ptr` must point to a populated `VoltageContext` object from the `mwalib_voltage_context_new` function.
/// * `voltage_context_ptr` must not have already been freed.
#[no_mangle]
#[allow(unused_must_use)]
pub unsafe extern "C" fn mwalib_voltage_context_free(
    voltage_context_ptr: *mut crate::VoltageContext,
) -> i32 {
    if voltage_context_ptr.is_null() {
        return MWALIB_SUCCESS;
    }
    // Release voltage context if applicable
    drop(Box::from_raw(voltage_context_ptr));

    // Return success
    MWALIB_SUCCESS
}
