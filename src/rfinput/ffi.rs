use crate::{
    ffi::{ffi_create_c_array, ffi_create_c_string, ffi_free_c_array, ffi_free_rust_c_string},
    rfinput::{self, ffi},
    MetafitsContext, MAX_RECEIVER_CHANNELS,
};

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::ReceiverType;
use std::ffi::c_char;

/// Representation in C of an `RFInput` struct
#[repr(C)]
pub struct Rfinput {
    /// This is the metafits order (0-n inputs)
    pub input: u32,
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
    /// Digital gains
    /// The values from the metafits are scaled by 64, so mwalib divides by 64.
    /// Digital gains are in mwalib metafits coarse channel order (ascending sky frequency order)
    pub digital_gains: *mut f64,
    /// Number of elements in the digital_gains array
    pub num_digital_gains: usize,
    /// Dipole delays
    pub dipole_delays: *mut u32,
    /// Number of elements in the dipole_delays array
    pub num_dipole_delays: usize,
    /// Dipole gains.
    ///
    /// These are either 1 or 0 (on or off), depending on the dipole delay; a
    /// dipole delay of 32 corresponds to "dead dipole", so the dipole gain of 0
    /// reflects that. All other dipoles are assumed to be "live". The values
    /// are made floats for easy use in beam code.
    pub dipole_gains: *mut f64,
    /// Number of elements in the dipole_gains array
    pub num_dipole_gains: usize,
    /// Receiver number
    pub rec_number: u32,
    /// Receiver slot number
    pub rec_slot_number: u32,
    /// Receiver type
    pub rec_type: ReceiverType,
    /// Flavour
    pub flavour: *mut c_char,
    /// Has whitening filter depends on flavour
    pub has_whitening_filter: bool,
    /// Calibration delay in meters (if provided)
    pub calib_delay: f32,
    /// Calibration gains (vector- 1 per coarse channel) if provided. Channels are in `MetafitsContext.course_chans` order.
    pub calib_gains: *mut f32,
    /// Number of elements in the calibration gains vector
    pub num_calib_gains: usize,
    /// Signal chain correction index
    /// This is the index into the MetafitsContext.signal_chain_corrections vector, or MAX_RECEIVER_CHANNELS if not applicable/not found for the
    /// receiver type and whitening filter combination
    pub signal_chain_corrections_index: usize,
}

impl Rfinput {
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
    /// * The corresponding `destroy_array` function for this class must be called to free the memory
    ///
    pub fn populate_array(metafits_context: &MetafitsContext) -> (*mut ffi::Rfinput, usize) {
        let mut item_vec: Vec<ffi::Rfinput> = Vec::new();

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

                // Handle vectors
                let (calib_gains_ptr, calib_gains_len) = match calib_gains {
                    Some(gains) => ffi_create_c_array(gains.clone()),
                    None => ffi_create_c_array(vec![
                        f32::NAN;
                        metafits_context.num_metafits_coarse_chans
                    ]),
                };

                let (digital_gains_ptr, digital_gains_len) =
                    ffi_create_c_array(digital_gains.clone());

                let (dipole_gains_ptr, dipole_gains_len) = ffi_create_c_array(dipole_gains.clone());

                let (dipole_delays_ptr, dipole_delays_len) =
                    ffi_create_c_array(dipole_delays.clone());

                rfinput::ffi::Rfinput {
                    input: *input,
                    ant: *ant,
                    tile_id: *tile_id,
                    tile_name: ffi_create_c_string(tile_name),
                    pol: ffi_create_c_string(&pol.to_string()),
                    electrical_length_m: *electrical_length_m,
                    north_m: *north_m,
                    east_m: *east_m,
                    height_m: *height_m,
                    vcs_order: *vcs_order,
                    subfile_order: *subfile_order,
                    flagged: *flagged,
                    digital_gains: digital_gains_ptr,
                    num_digital_gains: digital_gains_len,
                    dipole_gains: dipole_gains_ptr,
                    num_dipole_gains: dipole_gains_len,
                    dipole_delays: dipole_delays_ptr,
                    num_dipole_delays: dipole_delays_len,
                    rec_number: *rec_number,
                    rec_slot_number: *rec_slot_number,
                    rec_type: *rec_type,
                    flavour: ffi_create_c_string(flavour),
                    has_whitening_filter: *has_whitening_filter,
                    calib_delay: calib_delay.unwrap_or(f32::NAN),
                    calib_gains: calib_gains_ptr,
                    num_calib_gains: calib_gains_len,
                    signal_chain_corrections_index: signal_chain_corrections_index
                        .unwrap_or(MAX_RECEIVER_CHANNELS),
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
    fn destroy_item(item: *mut ffi::Rfinput) {
        if item.is_null() {
            return;
        }

        let a = unsafe { &mut *item };

        // Free strings if present
        if !a.tile_name.is_null() {
            ffi_free_rust_c_string(a.tile_name);
        }

        if !a.pol.is_null() {
            ffi_free_rust_c_string(a.pol);
        }

        if !a.flavour.is_null() {
            ffi_free_rust_c_string(a.flavour);
        }

        // Free arrays
        if !a.calib_gains.is_null() {
            ffi_free_c_array(a.calib_gains, a.num_calib_gains);
        }

        if !a.digital_gains.is_null() {
            ffi_free_c_array(a.digital_gains, a.num_digital_gains);
        }

        if !a.dipole_gains.is_null() {
            ffi_free_c_array(a.dipole_gains, a.num_dipole_gains);
        }

        if !a.dipole_delays.is_null() {
            ffi_free_c_array(a.dipole_delays, a.num_dipole_delays);
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
    pub fn destroy_array(items_ptr: *mut ffi::Rfinput, items_len: usize) {
        let items_slice = unsafe { std::slice::from_raw_parts_mut(items_ptr, items_len) };
        for item in items_slice {
            Self::destroy_item(item);
        }
        // Now free the array itself by reconstructing the Vec and letting it drop
        ffi_free_c_array(items_ptr, items_len);
    }
}
