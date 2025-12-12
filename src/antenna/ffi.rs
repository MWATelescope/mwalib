// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ffi::c_char;

use crate::{
    antenna::{self, ffi},
    ffi::{ffi_create_c_array, ffi_create_c_string, ffi_free_rust_c_string, ffi_free_rust_struct},
    MetafitsContext,
};

/// Representation in C of an `Antenna` struct
#[repr(C)]
pub struct Antenna {
    /// This is the antenna number.
    /// Nominally this is the field we sort by to get the desired output order of antenna.
    /// X and Y have the same antenna number. This is the sorted ordinal order of the antenna.None
    /// e.g. 0...N-1
    pub ant: u32,
    /// Numeric part of tile_name for the antenna. Each pol has the same value
    /// e.g. tile_name "tile011" hsa tile_id of 11
    pub tile_id: u32,
    /// Human readable name of the antenna
    /// X and Y have the same name
    pub tile_name: *mut c_char,
    /// Index within the array of rfinput structs of the x pol
    pub rfinput_x: usize,
    /// Index within the array of rfinput structs of the y pol
    pub rfinput_y: usize,
    ///
    /// Note: the next 4 values are from the rfinput of which we have X and Y, however these values are the same for each pol so can be safely placed in the antenna struct
    /// for efficiency
    ///
    /// Electrical length in metres for this antenna and polarisation to the receiver
    pub electrical_length_m: f64,
    /// Antenna position North from the array centre (metres)
    pub north_m: f64,
    /// Antenna position East from the array centre (metres)
    pub east_m: f64,
    /// Antenna height from the array centre (metres)
    pub height_m: f64,
}

impl Antenna {
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
    pub fn ffi_populate(metafits_context: &MetafitsContext) -> (*mut ffi::Antenna, usize) {
        let mut item_vec: Vec<ffi::Antenna> = Vec::new();

        for item in metafits_context.antennas.iter() {
            let out_item = {
                let antenna::Antenna {
                    ant,
                    tile_id,
                    tile_name: _,
                    rfinput_x,
                    rfinput_y,
                    electrical_length_m,
                    north_m,
                    east_m,
                    height_m,
                } = item;

                let tile_name_c_str = ffi_create_c_string(&item.tile_name);

                antenna::ffi::Antenna {
                    ant: *ant,
                    tile_id: *tile_id,
                    tile_name: tile_name_c_str,
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

            item_vec.push(out_item);
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
    fn ffi_destroy(item: *mut ffi::Antenna) {
        if item.is_null() {
            return;
        }

        let a = unsafe { &mut *item };

        // Free string if present
        if !a.tile_name.is_null() {
            ffi_free_rust_c_string(&mut a.tile_name);
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
    pub fn ffi_destroy_array(items_ptr: *mut Antenna, items_len: usize) {
        let items_slice = unsafe { std::slice::from_raw_parts_mut(items_ptr, items_len) };
        for item in items_slice {
            Self::ffi_destroy(item);
        }
        // Now free the array itself by reconstructing the Vec and letting it drop
        ffi_free_rust_struct(items_ptr, items_len);
    }
}
