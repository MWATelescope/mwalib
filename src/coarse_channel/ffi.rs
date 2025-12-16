// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    coarse_channel::{self, ffi},
    ffi::{ffi_create_c_array, ffi_free_c_array},
};

/// Representation in C of an `CoarseChannel` struct
#[repr(C)]
pub struct CoarseChannel {
    /// Correlator channel is 0 indexed (0..N-1)
    pub corr_chan_number: usize,
    /// Receiver channel is 0-255 in the RRI recivers
    pub rec_chan_number: usize,
    /// gpubox channel number
    /// This is better described as the identifier which would be in the filename of visibility files
    /// Legacy e.g. obsid_datetime_gpuboxXX_00
    /// v2     e.g. obsid_datetime_gpuboxXXX_00
    pub gpubox_number: usize,
    /// Width of a coarse channel in Hz
    pub chan_width_hz: u32,
    /// Starting frequency of coarse channel in Hz
    pub chan_start_hz: u32,
    /// Centre frequency of coarse channel in Hz
    pub chan_centre_hz: u32,
    /// Ending frequency of coarse channel in Hz
    pub chan_end_hz: u32,
}

impl CoarseChannel {
    /// This function populates a C array (owned by Rust) of this class
    ///
    /// # Arguments
    ///
    /// * `coarse_chans` - Reference to the populated Rust slice of coarse channels
    ///    
    /// # Returns
    ///
    /// * Noting
    ///
    ///
    /// # Safety
    /// * The corresponding `ffi_destroy` function for this class must be called to free the memory
    ///
    pub fn populate_array(
        coarse_chans: &[crate::coarse_channel::CoarseChannel],
    ) -> (*mut ffi::CoarseChannel, usize) {
        let mut item_vec: Vec<ffi::CoarseChannel> = Vec::new();

        for item in coarse_chans.iter() {
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
    pub fn destroy_array(items_ptr: *mut ffi::CoarseChannel, items_len: usize) {
        // Now free the array itself by reconstructing the Vec and letting it drop
        ffi_free_c_array(items_ptr, items_len);
    }
}
