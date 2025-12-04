// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This module exists purely for other languages to interface with mwalib.

use crate::*;
use libc::{c_char, c_double, c_float, c_schar, c_uint, size_t};
use std::ffi::*;
use std::mem;
use std::slice;

#[cfg(test)]
mod test;

/// mwalib FFI interface return codes
pub const MWALIB_SUCCESS: i32 = 0;
pub const MWALIB_FAILURE: i32 = 1;
pub const MWALIB_NO_DATA_FOR_TIMESTEP_COARSECHAN: i32 = -1;

/// Generic helper function for all FFI modules to take an already allocated C string buffer
/// and populate it with a string. This is primarily used to pass error messages back to C from Rust.
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
pub(crate) fn set_c_string(
    in_message: &str,
    error_buffer_ptr: *mut c_char,
    error_buffer_len: size_t,
) {
    // Don't do anything if the pointer is null.
    if error_buffer_ptr.is_null() {
        return;
    }
    // Check that error buffer, minus 1 for nul terminator is still >=1
    if error_buffer_len < 2 {
        return;
    } // need at least 1 char + NUL

    // Trim it to error_buffer_len - 1 (must include room for null terminator)
    let max_bytes = error_buffer_len - 1;
    // Strip interior NULs to avoid CString failure
    let sanitized = in_message.replace('\0', "");
    let message = if sanitized.len() > max_bytes {
        &sanitized[..max_bytes]
    } else {
        &sanitized
    };

    // Convert to C string- panic if it can't.
    let error_message = CString::new(message).unwrap_or_else(|_| CString::new("").unwrap());

    // Add null terminator
    let error_message_bytes = error_message.as_bytes_with_nul();

    unsafe {
        let buf = slice::from_raw_parts_mut(error_buffer_ptr as *mut u8, error_buffer_len);
        buf[..error_message_bytes.len()].copy_from_slice(error_message_bytes);
    }
}

/// Return the MAJOR version number of mwalib
///
/// Uses the built crate in build.rs to generate at build time a built.rs module
///
/// # Arguments
///
/// * None
///
/// # Returns
///
/// * u16 representing the major version number of mwalib
///
#[no_mangle]
pub extern "C" fn mwalib_get_version_major() -> c_uint {
    built_info::PKG_VERSION_MAJOR.parse::<c_uint>().unwrap()
}

/// Return the MINOR version number of mwalib
///
/// Uses the built crate in build.rs to generate at build time a built.rs module
///
/// # Arguments
///
/// * None
///
/// # Returns
///
/// * u16 representing the minor version number of mwalib
///
#[no_mangle]
pub extern "C" fn mwalib_get_version_minor() -> c_uint {
    built_info::PKG_VERSION_MINOR.parse::<c_uint>().unwrap()
}

/// Return the PATCH version number of mwalib
///
/// Uses the built crate in build.rs to generate at build time a built.rs module
///
/// # Arguments
///
/// * None
///
/// # Returns
///
/// * u16 representing the patch version number of mwalib
///
#[no_mangle]
pub extern "C" fn mwalib_get_version_patch() -> c_uint {
    built_info::PKG_VERSION_PATCH.parse::<c_uint>().unwrap()
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
/// * MWALIB_SUCCESS on success, non-zero on failure
///
/// # Safety
/// * rust_cstring must not have already been freed and must point to a Rust string.
#[no_mangle]
pub unsafe extern "C" fn mwalib_free_rust_cstring(rust_cstring: *mut c_char) -> i32 {
    // Don't do anything if the pointer is null.
    if rust_cstring.is_null() {
        return MWALIB_SUCCESS;
    }
    drop(CString::from_raw(rust_cstring));

    // return success
    MWALIB_SUCCESS
}

/// Boxes for FFI a rust-allocated vector of T. If the vector is 0 length, returns a null pointer.
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
pub(crate) fn ffi_array_to_boxed_slice<T>(v: Vec<T>) -> *mut T {
    if !v.is_empty() {
        let mut boxed_slice: Box<[T]> = v.into_boxed_slice();
        let array_ptr: *mut T = boxed_slice.as_mut_ptr();
        let array_ptr_len: usize = boxed_slice.len();
        assert_eq!(boxed_slice.len(), array_ptr_len);

        // Prevent the slice from being destroyed (Leak the memory).
        // This is because we are using our ffi code to free the memory
        mem::forget(boxed_slice);

        array_ptr
    } else {
        std::ptr::null_mut()
    }
}

/// Create and return a pointer to an `MetafitsContext` struct given only a metafits file and MWAVersion.
///
/// # Arguments
///
/// * `metafits_filename` - pointer to char* buffer containing the full path and filename of a metafits file.
///
/// * `mwa_version` - enum providing mwalib with the intended mwa version which the metafits should be interpreted.
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
/// * return MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated `char*` buffer for any error messages.
/// * Caller *must* call the `mwalib_metafits_context_free` function to release the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_context_new(
    metafits_filename: *const c_char,
    mwa_version: MWAVersion,
    out_metafits_context_ptr: &mut *mut MetafitsContext,
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

    let context = match MetafitsContext::new(m, Some(mwa_version)) {
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

    *out_metafits_context_ptr = Box::into_raw(Box::new(context));

    // Return success
    MWALIB_SUCCESS
}

/// Create and return a pointer to an `MetafitsContext` struct given only a metafits file. Same as mwalib_metafits_context_new, but mwalib will guess the MWAVersion.
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
/// * return MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated `char*` buffer for any error messages.
/// * Caller *must* call the `mwalib_metafits_context_free` function to release the rust memory.
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_context_new2(
    metafits_filename: *const c_char,
    out_metafits_context_ptr: &mut *mut MetafitsContext,
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

    let context = match MetafitsContext::new(m, None) {
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

    *out_metafits_context_ptr = Box::into_raw(Box::new(context));

    // Return success
    MWALIB_SUCCESS
}

/// Generates an expected filename, given a MetafitsContext, timestep index and coarse channel index.
///
///
/// # Arguments
///
/// * `metafits_context_ptr` - pointer to an already populated `MetafitsContext` object
///
/// * `metafits_timestep_index` - the timestep index you are generating this filename for.
///
/// * `metafits_coarse_chan_index` - the coarse_chan index you are generating this filename for.
///
/// * `out_filename_ptr` - Pointer to a char* buffer which has already been allocated, for storing the filename.
///
/// * `out_filename_len` - Length of char* buffer allocated by caller in C.
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
/// * `out_filename_ptr` *must* point to an already allocated char* buffer for the output filename to be written to.
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `metafits_context_ptr` must contain an MetafitsContext object already populated via `mwalib_metafits_context_new`
///   It is up to the caller to:
///   - Free `out_filename_ptr` once finished with the buffer.
///   - Free `error_message` once finished with the buffer.
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_get_expected_volt_filename(
    metafits_context_ptr: *const MetafitsContext,
    metafits_timestep_index: usize,
    metafits_coarse_chan_index: usize,
    out_filename_ptr: *const c_char,
    out_filename_len: size_t,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    if metafits_context_ptr.is_null() {
        set_c_string(
            "mwalib_metafits_get_expected_voltage_filename() ERROR: null pointer for metafits_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }

    let context = &*metafits_context_ptr;

    match context
        .generate_expected_volt_filename(metafits_timestep_index, metafits_coarse_chan_index)
    {
        Err(e) => {
            set_c_string(
                &e.to_string(),
                error_message as *mut c_char,
                error_message_length,
            );
            MWALIB_FAILURE
        }
        Ok(s) => {
            set_c_string(&s, out_filename_ptr as *mut c_char, out_filename_len);

            // Return success
            MWALIB_SUCCESS
        }
    }
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
/// * MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * `error_message` *must* point to an already allocated char* buffer for any error messages.
/// * `metafits_context_ptr` must contain an MetafitsContext object already populated via `mwalib_metafits_context_new`
#[no_mangle]
pub unsafe extern "C" fn mwalib_metafits_context_display(
    metafits_context_ptr: *const MetafitsContext,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    if metafits_context_ptr.is_null() {
        set_c_string(
            "mwalib_metafits_context_display() ERROR: null pointer for metafits_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }

    let context = &*metafits_context_ptr;

    println!("{}", context);

    // Return success
    MWALIB_SUCCESS
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
/// * MWALIB_SUCCESS on success, non-zero on failure
///
///
/// # Safety
/// * This must be called once caller is finished with the `MetafitsContext` object
/// * `metafits_context_ptr` must point to a populated `MetafitsContext` object from the `mwalib_metafits_context_new` functions.
/// * `metafits_context_ptr` must not have already been freed.
#[no_mangle]
#[allow(unused_must_use)]
pub unsafe extern "C" fn mwalib_metafits_context_free(
    metafits_context_ptr: *mut MetafitsContext,
) -> i32 {
    if metafits_context_ptr.is_null() {
        return MWALIB_SUCCESS;
    }

    // Release correlator context if applicable
    drop(Box::from_raw(metafits_context_ptr));

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

///
/// This a C struct to allow the caller to consume the metafits metadata
///
#[repr(C)]
pub struct MetafitsMetadata {
    /// mwa version
    pub mwa_version: MWAVersion,
    /// Observation id
    pub obs_id: u32,
    /// ATTEN_DB  // global analogue attenuation, in dB
    pub global_analogue_attenuation_db: f64,
    /// RA tile pointing
    pub ra_tile_pointing_deg: f64,
    /// DEC tile pointing
    pub dec_tile_pointing_deg: f64,
    /// RA phase centre
    pub ra_phase_center_deg: f64,
    /// DEC phase centre
    pub dec_phase_center_deg: f64,
    /// AZIMUTH
    pub az_deg: f64,
    /// ALTITUDE
    pub alt_deg: f64,
    /// Zenith angle of the pointing centre in degrees
    pub za_deg: f64,
    /// AZIMUTH of the pointing centre in radians
    pub az_rad: f64,
    /// ALTITUDE (a.k.a. elevation) of the pointing centre in radians
    pub alt_rad: f64,
    /// Zenith angle of the pointing centre in radians
    pub za_rad: f64,
    /// Altitude of Sun
    pub sun_alt_deg: f64,
    /// Distance from pointing center to Sun
    pub sun_distance_deg: f64,
    /// Distance from pointing center to the Moon
    pub moon_distance_deg: f64,
    /// Distance from pointing center to Jupiter
    pub jupiter_distance_deg: f64,
    /// Local Sidereal Time in degrees (at the midpoint of the observation)
    pub lst_deg: f64,
    /// Local Sidereal Time in radians (at the midpoint of the observation)        
    pub lst_rad: f64,
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
    pub obs_name: *mut c_char,
    /// MWA observation mode
    pub mode: MWAMode,
    /// Which Geometric delays have been applied to the data
    pub geometric_delays_applied: GeometricDelaysApplied,
    /// Have cable delays been applied to the data?    
    pub cable_delays_applied: CableDelaysApplied,
    /// Have calibration delays and gains been applied to the data?
    pub calibration_delays_and_gains_applied: bool,
    /// Correlator fine_chan_resolution
    pub corr_fine_chan_width_hz: u32,
    /// Correlator mode dump time
    pub corr_int_time_ms: u64,
    /// Correlator visibility scaling factor used to get the visibilities in Jansky-like units
    pub corr_raw_scale_factor: f32,
    /// Number of fine channels in each coarse channel for a correlator observation
    pub num_corr_fine_chans_per_coarse: usize,
    /// Voltage fine_chan_resolution
    pub volt_fine_chan_width_hz: u32,
    /// Number of fine channels in each coarse channel for a voltage observation
    pub num_volt_fine_chans_per_coarse: usize,
    /// Array of receiver numbers (this tells us how many receivers too)
    pub receivers: *mut usize,
    /// Number of receivers
    pub num_receivers: usize,
    /// Array of beamformer delays
    pub delays: *mut u32,
    /// Number of beamformer delays
    pub num_delays: usize,
    /// Intended for calibration
    pub calibrator: bool,
    /// Calibrator source
    pub calibrator_source: *mut c_char,
    /// Scheduled start (UTC) of observation as a time_t (unix timestamp)
    pub sched_start_utc: libc::time_t,
    /// Scheduled end (UTC) of observation as a time_t (unix timestamp)
    pub sched_end_utc: libc::time_t,
    /// Scheduled start (MJD) of observation
    pub sched_start_mjd: f64,
    /// Scheduled end (MJD) of observation
    pub sched_end_mjd: f64,
    /// Scheduled start (UNIX time) of observation
    pub sched_start_unix_time_ms: u64,
    /// Scheduled end (UNIX time) of observation
    pub sched_end_unix_time_ms: u64,
    /// Scheduled start (GPS) of observation
    pub sched_start_gps_time_ms: u64,
    /// Scheduled end (GPS) of observation
    pub sched_end_gps_time_ms: u64,
    /// Scheduled duration of observation
    pub sched_duration_ms: u64,
    /// DUT1 (i.e. UTC-UT1). The UTC of the obsid is used to determine this
    /// value. Calculated by astropy. Made optional for compatibility.
    pub dut1: f64,
    /// Seconds of bad data after observation starts
    pub quack_time_duration_ms: u64,
    /// OBSID+QUACKTIM as Unix timestamp (first good timestep)
    pub good_time_unix_ms: u64,
    /// Good time expressed as GPS seconds
    pub good_time_gps_ms: u64,
    /// Total number of antennas (tiles) in the array
    pub num_ants: usize,
    /// Array of antennas
    pub antennas: *mut antenna::ffi::Antenna,
    /// The Metafits defines an rf chain for antennas(tiles) * pol(X,Y)
    pub num_rf_inputs: usize,
    /// Array of rf inputs
    pub rf_inputs: *mut rfinput::ffi::Rfinput,
    /// Number of antenna pols. e.g. X and Y
    pub num_ant_pols: usize,
    /// Number of baselines
    pub num_baselines: usize,
    /// Baseline array
    pub baselines: *mut baseline::ffi::Baseline,
    /// Number of visibility_pols
    pub num_visibility_pols: usize,
    /// Number of coarse channels based on the metafits
    pub num_metafits_coarse_chans: usize,
    /// metafits_coarse_chans array
    pub metafits_coarse_chans: *mut coarse_channel::ffi::CoarseChannel,
    /// Number of fine channels for the whole observation
    pub num_metafits_fine_chan_freqs_hz: usize,
    /// Vector of fine channel frequencies for the whole observation
    pub metafits_fine_chan_freqs_hz: *mut f64,
    /// Number of timesteps based on the metafits
    pub num_metafits_timesteps: usize,
    /// metafits_timesteps array
    pub metafits_timesteps: *mut timestep::ffi::TimeStep,
    /// Total bandwidth of observation assuming we have all coarse channels
    pub obs_bandwidth_hz: u32,
    /// Bandwidth of each coarse channel
    pub coarse_chan_width_hz: u32,
    /// Centre frequency of observation
    pub centre_freq_hz: u32,
    /// filename of metafits file used
    pub metafits_filename: *mut c_char,
    /// Was this observation using oversampled coarse channels?
    pub oversampled: bool,
    /// Was deripple applied to this observation?
    pub deripple_applied: bool,
    /// What was the configured deripple_param?
    /// If deripple_applied is False then this deripple param was not applied
    pub deripple_param: *mut c_char,
    /// Best calibration fit ID
    pub best_cal_fit_id: u32,
    /// Best calibration observation ID
    pub best_cal_obs_id: u32,
    /// Best calibration fit code version
    pub best_cal_code_ver: *mut c_char,
    /// Best calibration fit timestamp
    pub best_cal_fit_timestamp: *mut c_char,
    /// Best calibration fit creator
    pub best_cal_creator: *mut c_char,
    /// Best calibration fit iterations
    pub best_cal_fit_iters: u16,
    /// Best calibration fit iteration limit
    pub best_cal_fit_iter_limit: u16,
    /// Signal Chain corrections array
    pub signal_chain_corrections: *mut signal_chain_correction::ffi::SignalChainCorrection,
    /// Number of signal chain corrections in the array
    pub num_signal_chain_corrections: usize,
    /// Calibration fits
    pub calibration_fits: *mut calibration_fit::ffi::CalibrationFit,
    /// Number of calibration fits in the array
    pub num_calibration_fits: usize,
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
/// * MWALIB_SUCCESS on success, non-zero on failure
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
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    // Ensure only either metafits XOR correlator XOR voltage context is passed in
    let provided = usize::from(!metafits_context_ptr.is_null())
        + usize::from(!correlator_context_ptr.is_null())
        + usize::from(!voltage_context_ptr.is_null());
    if provided != 1 {
        set_c_string(
            "mwalib_metafits_metadata_get() ERROR: pointers for metafits_context_ptr, correlator_context_ptr and/or voltage_context_ptr were passed in. Only one should be provided.",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }

    // Create our metafits context pointer depending on what was passed in
    let metafits_context = if !metafits_context_ptr.is_null() {
        // Caller passed in a metafits context, so use that
        &*metafits_context_ptr
    } else if !correlator_context_ptr.is_null() {
        // Caller passed in a correlator context, so use that
        &(*correlator_context_ptr).metafits_context
    } else {
        // Caller passed in a voltage context, so use that
        &(*voltage_context_ptr).metafits_context
    };

    // Populate baselines
    let mut baseline_vec: Vec<baseline::ffi::Baseline> = Vec::new();
    for item in metafits_context.baselines.iter() {
        let out_item = {
            let baseline::Baseline {
                ant1_index,
                ant2_index,
            } = item;
            baseline::ffi::Baseline {
                ant1_index: *ant1_index,
                ant2_index: *ant2_index,
            }
        };

        baseline_vec.push(out_item);
    }

    // Populate antennas
    let mut antenna_vec: Vec<antenna::ffi::Antenna> = Vec::new();
    for item in metafits_context.antennas.iter() {
        let out_item = {
            let antenna::Antenna {
                ant,
                tile_id,
                tile_name,
                rfinput_x,
                rfinput_y,
                electrical_length_m,
                north_m,
                east_m,
                height_m,
            } = item;
            antenna::ffi::Antenna {
                ant: *ant,
                tile_id: *tile_id,
                tile_name: CString::new(tile_name.replace('\0', ""))
                    .unwrap_or_else(|_| CString::new("").unwrap())
                    .into_raw(),
                rfinput_x: metafits_context
                    .rf_inputs
                    .iter()
                    .position(|x| x == rfinput_x)
                    .unwrap(),
                rfinput_y: metafits_context
                    .rf_inputs
                    .iter()
                    .position(|y| y == rfinput_y)
                    .unwrap(),
                electrical_length_m: *electrical_length_m,
                north_m: *north_m,
                east_m: *east_m,
                height_m: *height_m,
            }
        };

        antenna_vec.push(out_item);
    }

    // Populate rf_inputs
    let mut rfinput_vec: Vec<rfinput::ffi::Rfinput> = Vec::new();
    for item in metafits_context.rf_inputs.iter() {
        let out_item = {
            let rfinput::Rfinput {
                input,
                ant,
                tile_id,
                tile_name,
                pol,
                electrical_length_m,
                north_m,
                east_m,
                height_m,
                vcs_order,
                subfile_order,
                flagged,
                digital_gains,
                dipole_gains,
                dipole_delays,
                rec_number,
                rec_slot_number,
                rec_type,
                flavour,
                has_whitening_filter,
                calib_delay,
                calib_gains,
                signal_chain_corrections_index,
            } = item;

            let calib_delay = calib_delay.unwrap_or(f32::NAN);
            let calib_gains_vec: Vec<f32> = calib_gains
                .clone()
                .unwrap_or(vec![f32::NAN; metafits_context.num_metafits_coarse_chans]);
            let num_calib_gains = calib_gains_vec.len();

            rfinput::ffi::Rfinput {
                input: *input,
                ant: *ant,
                tile_id: *tile_id,
                tile_name: CString::new(tile_name.replace('\0', ""))
                    .unwrap_or_else(|_| CString::new("").unwrap())
                    .into_raw(),
                pol: CString::new(pol.to_string()).unwrap().into_raw(),
                electrical_length_m: *electrical_length_m,
                north_m: *north_m,
                east_m: *east_m,
                height_m: *height_m,
                vcs_order: *vcs_order,
                subfile_order: *subfile_order,
                flagged: *flagged,
                digital_gains: ffi_array_to_boxed_slice(digital_gains.clone()),
                num_digital_gains: digital_gains.len(),
                dipole_gains: ffi_array_to_boxed_slice(dipole_gains.clone()),
                num_dipole_gains: dipole_gains.len(),
                dipole_delays: ffi_array_to_boxed_slice(dipole_delays.clone()),
                num_dipole_delays: dipole_delays.len(),
                rec_number: *rec_number,
                rec_slot_number: *rec_slot_number,
                rec_type: *rec_type,
                flavour: CString::new(flavour.replace('\0', ""))
                    .unwrap_or_else(|_| CString::new("").unwrap())
                    .into_raw(),
                has_whitening_filter: *has_whitening_filter,
                calib_delay,
                calib_gains: ffi_array_to_boxed_slice(calib_gains_vec),
                num_calib_gains,
                signal_chain_corrections_index: signal_chain_corrections_index
                    .unwrap_or(MAX_RECEIVER_CHANNELS),
            }
        };
        rfinput_vec.push(out_item);
    }

    // Populate metafits coarse channels
    let mut coarse_chan_vec: Vec<coarse_channel::ffi::CoarseChannel> = Vec::new();

    for item in metafits_context.metafits_coarse_chans.iter() {
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

    // Populate metafits timesteps
    let mut timestep_vec: Vec<timestep::ffi::TimeStep> = Vec::new();

    for item in metafits_context.metafits_timesteps.iter() {
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

    // Populate signal chain corrections
    let mut signal_chain_corrections_vec: Vec<signal_chain_correction::ffi::SignalChainCorrection> =
        Vec::new();

    if let Some(v) = &metafits_context.signal_chain_corrections {
        for item in v.iter() {
            let out_item = {
                let signal_chain_correction::SignalChainCorrection {
                    receiver_type,
                    whitening_filter,
                    corrections,
                } = item;
                signal_chain_correction::ffi::SignalChainCorrection {
                    receiver_type: *receiver_type,
                    whitening_filter: *whitening_filter,
                    corrections: ffi_array_to_boxed_slice(corrections.clone()),
                }
            };
            signal_chain_corrections_vec.push(out_item);
        }
    }

    // Populate calibration fits
    let mut calibration_fits_vec: Vec<calibration_fit::ffi::CalibrationFit> = Vec::new();

    if let Some(v) = &metafits_context.calibration_fits {
        for item in v.iter() {
            let out_item = {
                let calibration_fit::CalibrationFit {
                    rf_input: _,
                    delay_metres,
                    intercept_metres,
                    gains,
                    gain_polynomial_fit0,
                    gain_polynomial_fit1,
                    phase_fit_quality,
                    gain_fit_quality,
                } = item;

                calibration_fit::ffi::CalibrationFit {
                    rf_input: metafits_context
                        .rf_inputs
                        .iter()
                        .position(|x| x.ant == item.rf_input.ant && x.pol == item.rf_input.pol)
                        .unwrap(),
                    delay_metres: *delay_metres,
                    intercept_metres: *intercept_metres,
                    gains: ffi_array_to_boxed_slice(gains.clone()),
                    gain_polynomial_fit0: ffi_array_to_boxed_slice(gain_polynomial_fit0.clone()),
                    gain_polynomial_fit1: ffi_array_to_boxed_slice(gain_polynomial_fit1.clone()),
                    phase_fit_quality: *phase_fit_quality,
                    gain_fit_quality: *gain_fit_quality,
                }
            };
            calibration_fits_vec.push(out_item);
        }
    }

    // Populate the outgoing structure with data from the metafits context
    // We explicitly break out the attributes so at compile time it will let us know
    // if there have been new fields added to the rust struct, then we can choose to
    // ignore them (with _) or add that field to the FFI struct.
    let out_metadata = {
        let MetafitsContext {
            mwa_version,
            obs_id,
            sched_start_gps_time_ms,
            sched_end_gps_time_ms,
            sched_start_unix_time_ms,
            sched_end_unix_time_ms,
            sched_start_utc,
            sched_end_utc,
            sched_start_mjd,
            sched_end_mjd,
            sched_duration_ms,
            dut1,
            ra_tile_pointing_degrees,
            dec_tile_pointing_degrees,
            ra_phase_center_degrees,
            dec_phase_center_degrees,
            az_deg,
            alt_deg,
            za_deg,
            az_rad,
            alt_rad,
            za_rad,
            sun_alt_deg,
            sun_distance_deg,
            moon_distance_deg,
            jupiter_distance_deg,
            lst_deg: lst_degrees,
            lst_rad: lst_radians,
            hour_angle_string,
            grid_name,
            grid_number,
            creator,
            project_id,
            obs_name,
            mode,
            geometric_delays_applied,
            cable_delays_applied,
            calibration_delays_and_gains_applied,
            corr_fine_chan_width_hz,
            corr_int_time_ms,
            corr_raw_scale_factor,
            num_corr_fine_chans_per_coarse,
            volt_fine_chan_width_hz,
            num_volt_fine_chans_per_coarse,
            receivers,
            num_receivers,
            delays,
            num_delays,
            calibrator,
            calibrator_source,
            global_analogue_attenuation_db,
            quack_time_duration_ms,
            good_time_unix_ms,
            good_time_gps_ms,
            num_ants,
            antennas: _, // This is populated seperately
            num_rf_inputs,
            rf_inputs: _, // This is populated seperately
            num_ant_pols,
            num_metafits_timesteps,
            metafits_timesteps: _, // This is populated seperately
            num_metafits_coarse_chans,
            metafits_coarse_chans: _, // This is populated seperately
            num_metafits_fine_chan_freqs,
            metafits_fine_chan_freqs_hz,
            obs_bandwidth_hz,
            coarse_chan_width_hz,
            centre_freq_hz,
            num_baselines,
            baselines: _, // This is populated seperately
            num_visibility_pols,
            metafits_filename,
            oversampled,
            deripple_applied,
            deripple_param,
            best_cal_fit_id,
            best_cal_obs_id,
            best_cal_code_ver,
            best_cal_fit_timestamp,
            best_cal_creator,
            best_cal_fit_iters,
            best_cal_fit_iter_limit,
            signal_chain_corrections: _, // This is populated seperately
            num_signal_chain_corrections,
            calibration_fits: _, // This is populated seperately
            num_calibration_fits,
        } = metafits_context;
        MetafitsMetadata {
            mwa_version: mwa_version.unwrap(),
            obs_id: *obs_id,
            global_analogue_attenuation_db: *global_analogue_attenuation_db,
            ra_tile_pointing_deg: *ra_tile_pointing_degrees,
            dec_tile_pointing_deg: *dec_tile_pointing_degrees,
            ra_phase_center_deg: (*ra_phase_center_degrees).unwrap_or(f64::NAN),
            dec_phase_center_deg: (*dec_phase_center_degrees).unwrap_or(f64::NAN),
            az_deg: *az_deg,
            alt_deg: *alt_deg,
            za_deg: *za_deg,
            az_rad: *az_rad,
            alt_rad: *alt_rad,
            za_rad: *za_rad,
            sun_alt_deg: (*sun_alt_deg).unwrap_or(f64::NAN),
            sun_distance_deg: (*sun_distance_deg).unwrap_or(f64::NAN),
            moon_distance_deg: (*moon_distance_deg).unwrap_or(f64::NAN),
            jupiter_distance_deg: (*jupiter_distance_deg).unwrap_or(f64::NAN),
            lst_deg: *lst_degrees,
            lst_rad: *lst_radians,
            hour_angle_string: CString::new(hour_angle_string.replace('\0', ""))
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            grid_name: CString::new(grid_name.replace('\0', ""))
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            grid_number: *grid_number,
            creator: CString::new(creator.replace('\0', ""))
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            project_id: CString::new(project_id.replace('\0', ""))
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            obs_name: CString::new(obs_name.replace('\0', ""))
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            mode: *mode,
            geometric_delays_applied: *geometric_delays_applied,
            cable_delays_applied: *cable_delays_applied,
            calibration_delays_and_gains_applied: *calibration_delays_and_gains_applied,
            corr_fine_chan_width_hz: *corr_fine_chan_width_hz,
            corr_int_time_ms: *corr_int_time_ms,
            corr_raw_scale_factor: *corr_raw_scale_factor,
            num_corr_fine_chans_per_coarse: *num_corr_fine_chans_per_coarse,
            volt_fine_chan_width_hz: *volt_fine_chan_width_hz,
            num_volt_fine_chans_per_coarse: *num_volt_fine_chans_per_coarse,
            receivers: ffi_array_to_boxed_slice(receivers.clone()),
            num_receivers: *num_receivers,
            delays: ffi_array_to_boxed_slice(delays.clone()),
            num_delays: *num_delays,
            calibrator: *calibrator,
            calibrator_source: CString::new(calibrator_source.replace('\0', ""))
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            sched_start_utc: sched_start_utc.timestamp(),
            sched_end_utc: sched_end_utc.timestamp(),
            sched_start_mjd: *sched_start_mjd,
            sched_end_mjd: *sched_end_mjd,
            sched_start_unix_time_ms: *sched_start_unix_time_ms,
            sched_end_unix_time_ms: *sched_end_unix_time_ms,
            sched_start_gps_time_ms: *sched_start_gps_time_ms,
            sched_end_gps_time_ms: *sched_end_gps_time_ms,
            sched_duration_ms: *sched_duration_ms,
            dut1: dut1.unwrap_or(0.0),
            quack_time_duration_ms: *quack_time_duration_ms,
            good_time_unix_ms: *good_time_unix_ms,
            good_time_gps_ms: *good_time_gps_ms,
            num_ants: *num_ants,
            antennas: ffi_array_to_boxed_slice(antenna_vec),
            num_rf_inputs: *num_rf_inputs,
            rf_inputs: ffi_array_to_boxed_slice(rfinput_vec),
            num_ant_pols: *num_ant_pols,
            num_baselines: *num_baselines,
            baselines: ffi_array_to_boxed_slice(baseline_vec),
            num_visibility_pols: *num_visibility_pols,
            num_metafits_coarse_chans: *num_metafits_coarse_chans,
            metafits_coarse_chans: ffi_array_to_boxed_slice(coarse_chan_vec),
            num_metafits_fine_chan_freqs_hz: *num_metafits_fine_chan_freqs,
            metafits_fine_chan_freqs_hz: ffi_array_to_boxed_slice(
                metafits_fine_chan_freqs_hz.clone(),
            ),
            num_metafits_timesteps: *num_metafits_timesteps,
            metafits_timesteps: ffi_array_to_boxed_slice(timestep_vec),
            obs_bandwidth_hz: *obs_bandwidth_hz,
            coarse_chan_width_hz: *coarse_chan_width_hz,
            centre_freq_hz: *centre_freq_hz,
            metafits_filename: CString::new(metafits_filename.replace('\0', ""))
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            oversampled: *oversampled,
            deripple_applied: *deripple_applied,
            deripple_param: CString::new(deripple_param.replace('\0', ""))
                .unwrap_or_else(|_| CString::new("").unwrap())
                .into_raw(),
            best_cal_fit_id: best_cal_fit_id.unwrap_or_else(|| 0),
            best_cal_obs_id: best_cal_obs_id.unwrap_or_else(|| 0),
            best_cal_code_ver: CString::new(
                best_cal_code_ver
                    .clone()
                    .unwrap_or_default()
                    .replace('\0', ""),
            )
            .unwrap_or_else(|_| CString::new("").unwrap())
            .into_raw(),
            best_cal_fit_timestamp: CString::new(
                best_cal_fit_timestamp
                    .clone()
                    .unwrap_or_default()
                    .replace('\0', ""),
            )
            .unwrap_or_else(|_| CString::new("").unwrap())
            .into_raw(),
            best_cal_creator: CString::new(
                best_cal_creator
                    .clone()
                    .unwrap_or_default()
                    .replace('\0', ""),
            )
            .unwrap_or_else(|_| CString::new("").unwrap())
            .into_raw(),
            best_cal_fit_iters: best_cal_fit_iters.unwrap_or_else(|| 0),
            best_cal_fit_iter_limit: best_cal_fit_iter_limit.unwrap_or_else(|| 0),
            signal_chain_corrections: ffi_array_to_boxed_slice(signal_chain_corrections_vec),
            num_signal_chain_corrections: *num_signal_chain_corrections,
            calibration_fits: ffi_array_to_boxed_slice(calibration_fits_vec),
            num_calibration_fits: *num_calibration_fits,
        }
    };

    // Pass back a pointer to the rust owned struct
    *out_metafits_metadata_ptr = Box::into_raw(Box::new(out_metadata));

    // Return Success
    MWALIB_SUCCESS
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
/// * MWALIB_SUCCESS on success, non-zero on failure
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
        return MWALIB_SUCCESS;
    }

    //
    // Free members first
    //
    // baselines
    if !(*metafits_metadata_ptr).baselines.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [baseline::ffi::Baseline] = slice::from_raw_parts_mut(
            (*metafits_metadata_ptr).baselines,
            (*metafits_metadata_ptr).num_baselines,
        );
        drop(Box::from_raw(slice));
    }

    // antennas
    if !(*metafits_metadata_ptr).antennas.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [antenna::ffi::Antenna] = slice::from_raw_parts_mut(
            (*metafits_metadata_ptr).antennas,
            (*metafits_metadata_ptr).num_ants,
        );
        // Now for each item we need to free anything on the heap
        for i in slice.iter_mut() {
            drop(CString::from_raw(i.tile_name));
        }

        // Free the memory for the slice
        drop(Box::from_raw(slice));
    }

    // rf inputs
    if !(*metafits_metadata_ptr).rf_inputs.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [rfinput::ffi::Rfinput] = slice::from_raw_parts_mut(
            (*metafits_metadata_ptr).rf_inputs,
            (*metafits_metadata_ptr).num_rf_inputs,
        );
        // Now for each item we need to free anything on the heap
        for i in slice.iter_mut() {
            drop(CString::from_raw(i.tile_name));
            drop(CString::from_raw(i.pol));

            if !i.digital_gains.is_null() {
                drop(Box::from_raw(i.digital_gains));
            }
            if !i.dipole_gains.is_null() {
                drop(Box::from_raw(i.dipole_gains));
            }
            if !i.dipole_delays.is_null() {
                drop(Box::from_raw(i.dipole_delays));
            }
            drop(CString::from_raw(i.flavour));
        }

        // Free the memory for the slice
        drop(Box::from_raw(slice));
    }

    // coarse_channels
    if !(*metafits_metadata_ptr).metafits_coarse_chans.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [coarse_channel::ffi::CoarseChannel] = slice::from_raw_parts_mut(
            (*metafits_metadata_ptr).metafits_coarse_chans,
            (*metafits_metadata_ptr).num_metafits_coarse_chans,
        );
        drop(Box::from_raw(slice));
    }

    // timesteps
    if !(*metafits_metadata_ptr).metafits_timesteps.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [timestep::ffi::TimeStep] = slice::from_raw_parts_mut(
            (*metafits_metadata_ptr).metafits_timesteps,
            (*metafits_metadata_ptr).num_metafits_timesteps,
        );
        drop(Box::from_raw(slice));
    }

    // receivers
    if !(*metafits_metadata_ptr).receivers.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [usize] = slice::from_raw_parts_mut(
            (*metafits_metadata_ptr).receivers,
            (*metafits_metadata_ptr).num_receivers,
        );
        drop(Box::from_raw(slice));
    }

    // delays
    if !(*metafits_metadata_ptr).delays.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [u32] = slice::from_raw_parts_mut(
            (*metafits_metadata_ptr).delays,
            (*metafits_metadata_ptr).num_delays,
        );
        drop(Box::from_raw(slice));
    }

    // fine channel freqs
    if !(*metafits_metadata_ptr)
        .metafits_fine_chan_freqs_hz
        .is_null()
    {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [f64] = slice::from_raw_parts_mut(
            (*metafits_metadata_ptr).metafits_fine_chan_freqs_hz,
            (*metafits_metadata_ptr).num_metafits_fine_chan_freqs_hz,
        );
        drop(Box::from_raw(slice));
    }

    // signal chain corrections
    if !(*metafits_metadata_ptr).signal_chain_corrections.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [signal_chain_correction::ffi::SignalChainCorrection] =
            slice::from_raw_parts_mut(
                (*metafits_metadata_ptr).signal_chain_corrections,
                (*metafits_metadata_ptr).num_signal_chain_corrections,
            );

        // Now for each item we need to free anything on the heap
        for i in slice.iter_mut() {
            if !i.corrections.is_null() {
                drop(Box::from_raw(i.corrections));
            }
        }

        drop(Box::from_raw(
            (*metafits_metadata_ptr).signal_chain_corrections,
        ));
    }

    // calibration fits
    if !(*metafits_metadata_ptr).calibration_fits.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [calibration_fit::ffi::CalibrationFit] = slice::from_raw_parts_mut(
            (*metafits_metadata_ptr).calibration_fits,
            (*metafits_metadata_ptr).num_calibration_fits,
        );

        // Now for each item we need to free anything on the heap
        for i in slice.iter_mut() {
            if !i.gains.is_null() {
                drop(Box::from_raw(i.gains));
            }

            if !i.gains.is_null() {
                drop(Box::from_raw(i.gain_polynomial_fit0));
            }

            if !i.gains.is_null() {
                drop(Box::from_raw(i.gain_polynomial_fit1));
            }
        }

        drop(Box::from_raw((*metafits_metadata_ptr).calibration_fits));
    }

    // Free top level string fields

    if !(*metafits_metadata_ptr).hour_angle_string.is_null() {
        drop(CString::from_raw(
            (*metafits_metadata_ptr).hour_angle_string,
        ));
    }

    if !(*metafits_metadata_ptr).grid_name.is_null() {
        drop(CString::from_raw((*metafits_metadata_ptr).grid_name));
    }

    if !(*metafits_metadata_ptr).creator.is_null() {
        drop(CString::from_raw((*metafits_metadata_ptr).creator));
    }

    if !(*metafits_metadata_ptr).project_id.is_null() {
        drop(CString::from_raw((*metafits_metadata_ptr).project_id));
    }

    if !(*metafits_metadata_ptr).calibrator_source.is_null() {
        drop(CString::from_raw(
            (*metafits_metadata_ptr).calibrator_source,
        ));
    }

    if !(*metafits_metadata_ptr).metafits_filename.is_null() {
        drop(CString::from_raw(
            (*metafits_metadata_ptr).metafits_filename,
        ));
    }

    if !(*metafits_metadata_ptr).deripple_param.is_null() {
        drop(CString::from_raw((*metafits_metadata_ptr).deripple_param));
    }

    if !(*metafits_metadata_ptr).best_cal_code_ver.is_null() {
        drop(CString::from_raw(
            (*metafits_metadata_ptr).best_cal_code_ver,
        ));
    }

    if !(*metafits_metadata_ptr).best_cal_fit_timestamp.is_null() {
        drop(CString::from_raw(
            (*metafits_metadata_ptr).best_cal_fit_timestamp,
        ));
    }

    if !(*metafits_metadata_ptr).best_cal_creator.is_null() {
        drop(CString::from_raw((*metafits_metadata_ptr).best_cal_creator));
    }

    // Free main metadata struct
    drop(Box::from_raw(metafits_metadata_ptr));

    // Return success
    MWALIB_SUCCESS
}

///
/// C Representation of the `CorrelatorContext` metadata
///
#[repr(C)]
pub struct CorrelatorMetadata {
    /// Version of the correlator format
    pub mwa_version: MWAVersion,
    /// This is an array of all known timesteps (union of metafits and provided timesteps from data files). The only exception is when the metafits timesteps are
    /// offset from the provided timesteps, in which case see description in `timestep::populate_metafits_provided_superset_of_timesteps`.
    pub timesteps: *mut timestep::ffi::TimeStep,
    /// Number of timesteps in the timestep array
    pub num_timesteps: usize,
    /// Vector of coarse channels which is the effectively the same as the metafits provided coarse channels
    pub coarse_chans: *mut coarse_channel::ffi::CoarseChannel,
    /// Count of coarse channels (same as metafits coarse channel count)
    pub num_coarse_chans: usize,
    /// Count of common timesteps
    pub num_common_timesteps: usize,
    /// Vector of (in)common timestep indices
    pub common_timestep_indices: *mut usize,
    /// Count of common coarse channels
    pub num_common_coarse_chans: usize,
    /// Indices of common coarse channels
    pub common_coarse_chan_indices: *mut usize,
    /// The proper start of the observation (the time that is common to all
    /// provided gpubox files).
    pub common_start_unix_time_ms: u64,
    /// `end_time_ms` will is the actual end time of the observation
    /// i.e. start time of last common timestep plus integration time.
    pub common_end_unix_time_ms: u64,
    /// `start_unix_time_ms` but in GPS milliseconds
    pub common_start_gps_time_ms: u64,
    /// `end_unix_time_ms` but in GPS milliseconds
    pub common_end_gps_time_ms: u64,
    /// Total duration of observation (based on gpubox files)
    pub common_duration_ms: u64,
    /// Total bandwidth of the common coarse channels which have been provided (which may be less than or equal to the bandwith in the MetafitsContext)
    pub common_bandwidth_hz: u32,
    /// Number of common timesteps only including timesteps after the quack time
    pub num_common_good_timesteps: usize,
    /// Vector of (in)common timestep indices only including timesteps after the quack time
    pub common_good_timestep_indices: *mut usize,
    /// Number of common coarse channels only including timesteps after the quack time
    pub num_common_good_coarse_chans: usize,
    /// Vector of (in)common timestep indices only including timesteps after the quack time
    pub common_good_coarse_chan_indices: *mut usize,
    /// The start of the observation (the time that is common to all
    /// provided gpubox files) only including timesteps after the quack time
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
    /// The number of bytes taken up by a scan/timestep in each gpubox file.
    pub num_timestep_coarse_chan_bytes: usize,
    /// The number of floats in each gpubox visibility HDU.
    pub num_timestep_coarse_chan_floats: usize,
    /// The number of floats in each gpubox weights HDU.
    pub num_timestep_coarse_chan_weight_floats: usize,
    /// This is the number of gpubox files *per batch*.
    pub num_gpubox_files: usize,
    /// BSCALE- FITS BSCALE or SCALEFAC value set on the visibility HDUs (used in Legacy Correlator only)
    pub bscale: f32,
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
    out_correlator_metadata_ptr: &mut *mut CorrelatorMetadata,
    error_message: *mut c_char,
    error_message_length: size_t,
) -> i32 {
    if correlator_context_ptr.is_null() {
        set_c_string(
            "mwalib_correlator_metadata_get() ERROR: Warning: null pointer for correlator_context_ptr passed in",
            error_message as *mut c_char,
            error_message_length,
        );
        return MWALIB_FAILURE;
    }
    // Get the correlator context object from the raw pointer passed in
    let context = &*correlator_context_ptr;

    // Populate correlator coarse channels
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

    // Populate correlator timesteps
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

    // Populate the rust owned data structure with data from the correlator context
    // We explicitly break out the attributes so at compile time it will let us know
    // if there have been new fields added to the rust struct, then we can choose to
    // ignore them (with _) or add that field to the FFI struct.
    let out_context = {
        let CorrelatorContext {
            metafits_context: _, // This is provided by the seperate metafits_metadata struct in FFI
            mwa_version,
            num_timesteps,
            timesteps: _, // This is populated seperately
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
            num_timestep_coarse_chan_bytes,
            num_timestep_coarse_chan_floats,
            num_timestep_coarse_chan_weight_floats,
            num_gpubox_files,
            gpubox_batches: _, // This is currently not provided to FFI as it is private
            gpubox_time_map: _, // This is currently not provided to FFI
            legacy_conversion_table: _, // This is currently not provided to FFI as it is private
            bscale,
        } = context;
        CorrelatorMetadata {
            mwa_version: *mwa_version,
            num_timesteps: *num_timesteps,
            timesteps: ffi_array_to_boxed_slice(timestep_vec),
            num_coarse_chans: *num_coarse_chans,
            coarse_chans: ffi_array_to_boxed_slice(coarse_chan_vec),
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
            num_timestep_coarse_chan_bytes: *num_timestep_coarse_chan_bytes,
            num_timestep_coarse_chan_floats: *num_timestep_coarse_chan_floats,
            num_timestep_coarse_chan_weight_floats: *num_timestep_coarse_chan_weight_floats,
            num_gpubox_files: *num_gpubox_files,
            bscale: *bscale,
        }
    };

    // Pass out the pointer to the rust owned data structure
    *out_correlator_metadata_ptr = Box::into_raw(Box::new(out_context));

    // Return success
    MWALIB_SUCCESS
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
pub unsafe extern "C" fn mwalib_correlator_metadata_free(
    correlator_metadata_ptr: *mut CorrelatorMetadata,
) -> i32 {
    if correlator_metadata_ptr.is_null() {
        return MWALIB_SUCCESS;
    }

    //
    // free any other members first
    //

    // coarse_channels
    if !(*correlator_metadata_ptr).coarse_chans.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [coarse_channel::ffi::CoarseChannel] = slice::from_raw_parts_mut(
            (*correlator_metadata_ptr).coarse_chans,
            (*correlator_metadata_ptr).num_coarse_chans,
        );
        drop(Box::from_raw(slice));
    }

    // timesteps
    if !(*correlator_metadata_ptr).timesteps.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [timestep::ffi::TimeStep] = slice::from_raw_parts_mut(
            (*correlator_metadata_ptr).timesteps,
            (*correlator_metadata_ptr).num_timesteps,
        );
        drop(Box::from_raw(slice));
    }

    // common timestep indices
    if !(*correlator_metadata_ptr).common_timestep_indices.is_null() {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [usize] = slice::from_raw_parts_mut(
            (*correlator_metadata_ptr).common_timestep_indices,
            (*correlator_metadata_ptr).num_common_timesteps,
        );
        drop(Box::from_raw(slice));
    }

    // common coarse chan indices
    if !(*correlator_metadata_ptr)
        .common_coarse_chan_indices
        .is_null()
    {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [usize] = slice::from_raw_parts_mut(
            (*correlator_metadata_ptr).common_coarse_chan_indices,
            (*correlator_metadata_ptr).num_common_coarse_chans,
        );
        drop(Box::from_raw(slice));
    }

    // common good timestep indices
    if !(*correlator_metadata_ptr)
        .common_good_timestep_indices
        .is_null()
    {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [usize] = slice::from_raw_parts_mut(
            (*correlator_metadata_ptr).common_good_timestep_indices,
            (*correlator_metadata_ptr).num_common_good_timesteps,
        );
        drop(Box::from_raw(slice));
    }

    // common good coarse chan indices
    if !(*correlator_metadata_ptr)
        .common_good_coarse_chan_indices
        .is_null()
    {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [usize] = slice::from_raw_parts_mut(
            (*correlator_metadata_ptr).common_good_coarse_chan_indices,
            (*correlator_metadata_ptr).num_common_good_coarse_chans,
        );
        drop(Box::from_raw(slice));
    }

    // provided timestep indices
    if !(*correlator_metadata_ptr)
        .provided_timestep_indices
        .is_null()
    {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [usize] = slice::from_raw_parts_mut(
            (*correlator_metadata_ptr).provided_timestep_indices,
            (*correlator_metadata_ptr).num_provided_timesteps,
        );
        drop(Box::from_raw(slice));
    }

    // provided coarse channel indices
    if !(*correlator_metadata_ptr)
        .provided_coarse_chan_indices
        .is_null()
    {
        // Reconstruct a fat slice first then box from that raw slice to allow Rust to deallocate the memory
        let slice: &mut [usize] = slice::from_raw_parts_mut(
            (*correlator_metadata_ptr).provided_coarse_chan_indices,
            (*correlator_metadata_ptr).num_provided_coarse_chans,
        );
        drop(Box::from_raw(slice));
    }

    // Free main metadata struct
    drop(Box::from_raw(correlator_metadata_ptr));

    // Return success
    MWALIB_SUCCESS
}
