// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    ffi::{ffi_create_c_array, ffi_free_c_array},
    timestep::{self, ffi},
};

///
/// C Representation of a `TimeStep` struct
///
#[repr(C)]
pub struct TimeStep {
    /// UNIX time (in milliseconds to avoid floating point inaccuracy)
    pub unix_time_ms: u64,
    /// gps time (in milliseconds)
    pub gps_time_ms: u64,
}

impl TimeStep {
    /// This function populates a C array (owned by Rust) of this class
    ///
    /// # Arguments
    ///
    /// * `timesteps` - Reference to the populated Rust timesteps slice
    ///    
    /// # Returns
    ///
    /// * Noting
    ///
    ///
    /// # Safety
    /// * The corresponding `ffi_destroy` function for this class must be called to free the memory
    ///
    pub fn populate_array(timesteps: &[crate::timestep::TimeStep]) -> (*mut ffi::TimeStep, usize) {
        let mut item_vec: Vec<ffi::TimeStep> = Vec::new();

        for item in timesteps {
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

            item_vec.push(out_item);
        }

        ffi_create_c_array(item_vec)
    }

    /// This function frees all array items (owned by Rust) of this class
    ///
    /// # Arguments
    ///
    /// * `items_ptr` - the pointer to the array
    ///
    /// * `items_len` - elements in the array
    ///    
    /// # Returns
    ///
    /// * Nothing
    ///
    pub fn destroy_array(items_ptr: *mut ffi::TimeStep, items_len: usize) {
        // Now free the array itself by reconstructing the Vec and letting it drop
        ffi_free_c_array(items_ptr, items_len);
    }
}
