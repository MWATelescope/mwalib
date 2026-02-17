// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::ReceiverType;

use crate::{
    ffi::{ffi_create_c_array, ffi_free_c_array},
    signal_chain_correction::{self, ffi},
    MetafitsContext,
};

///
/// C Representation of a `SignalChainCorrection` struct
///
///
/// Signal chain correction table
///
#[repr(C)]
pub struct SignalChainCorrection {
    /// Receiver Type
    pub receiver_type: ReceiverType,

    /// Whitening Filter
    pub whitening_filter: bool,

    /// Corrections
    pub corrections: *mut f64,

    // Number of Corrections
    pub num_corrections: usize,
}

impl SignalChainCorrection {
    /// This function populates a C array (owned by Rust) of this class
    ///
    /// # Arguments
    ///
    /// * `metafits_context` - Reference to the populated Rust MetafitsContext
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
        metafits_context: &MetafitsContext,
    ) -> (*mut ffi::SignalChainCorrection, usize) {
        let mut item_vec: Vec<ffi::SignalChainCorrection> = Vec::new();

        if let Some(v) = &metafits_context.signal_chain_corrections {
            for item in v.iter() {
                let (corrections_ptr, corrections_len) =
                    ffi_create_c_array(item.corrections.clone());

                let out_item = {
                    let signal_chain_correction::SignalChainCorrection {
                        receiver_type,
                        whitening_filter,
                        corrections: _,
                        num_corrections: _,
                    } = item;
                    signal_chain_correction::ffi::SignalChainCorrection {
                        receiver_type: *receiver_type,
                        whitening_filter: *whitening_filter,
                        corrections: corrections_ptr,
                        num_corrections: corrections_len,
                    }
                };
                item_vec.push(out_item);
            }
        }
        ffi_create_c_array(item_vec)
    }

    /// This function frees an individual instance (owned by Rust) of this class
    ///
    /// # Arguments
    ///
    /// * `item` - the pointer to the instance of this class
    ///    
    /// # Returns
    ///
    /// * Nothing
    ///
    fn destroy_item(item: *mut ffi::SignalChainCorrection) {
        if item.is_null() {
            return;
        }

        let a = unsafe { &mut *item };

        // Free array if present
        if !a.corrections.is_null() {
            ffi_free_c_array(a.corrections, a.num_corrections);
        }
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
    pub fn destroy_array(items_ptr: *mut ffi::SignalChainCorrection, items_len: usize) {
        let items_slice = unsafe { std::slice::from_raw_parts_mut(items_ptr, items_len) };
        for item in items_slice {
            Self::destroy_item(item);
        }
        // Now free the array itself by reconstructing the Vec and letting it drop
        ffi_free_c_array(items_ptr, items_len);
    }
}
