// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    baseline::{self, ffi},
    ffi::{ffi_create_c_array, ffi_free_rust_struct},
    MetafitsContext,
};

///
/// C Representation of a `Baseline` struct
///
#[repr(C)]
pub struct Baseline {
    /// Index in the `MetafitsContext` antenna array for antenna1 for this baseline
    pub ant1_index: usize,
    /// Index in the `MetafitsContext` antenna array for antenna2 for this baseline
    pub ant2_index: usize,
}

impl Baseline {
    /// This function populates a C array (owned by Rust) of this class
    ///
    /// # Arguments
    ///
    /// * `metafits_context` - Reference to the populated Rust MetafitsContext
    ///    
    /// # Returns
    ///
    /// * Nothing
    ///
    ///
    /// # Safety
    /// * The corresponding `destroy_array` function for this class must be called to free the memory
    ///
    pub fn populate_array(metafits_context: &MetafitsContext) -> (*mut ffi::Baseline, usize) {
        let mut item_vec: Vec<ffi::Baseline> = Vec::new();

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
    pub fn destroy_array(items_ptr: *mut ffi::Baseline, items_len: usize) {
        // Now free the array itself by reconstructing the Vec and letting it drop
        ffi_free_rust_struct(items_ptr, items_len);
    }
}
