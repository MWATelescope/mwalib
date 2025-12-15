// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ffi::c_char;

use libc::time_t;

use crate::{
    beam::{self, ffi},
    ffi::{ffi_create_c_array, ffi_create_c_string, ffi_free_rust_c_string, ffi_free_rust_struct},
    types::DataFileType,
    MetafitsContext,
};

///
/// C Representation of a `Beam` struct
///
///
/// Beam
///
#[repr(C)]
pub struct Beam {
    /// Arbitrary integer identifying this beam
    pub number: usize,
    /// True if the beam is coherent (has a defined position on the sky), False if incoherent (indicating that all tiles should be summed without phase correction)
    pub coherent: bool,
    /// azimuth, elevation - for coherent beams, these describe a fixed position relative to the telescope centre (eg, a geosynchronous satellite or ground-based RFI source)
    /// for incoherent beams these are set to 0.0
    pub az_deg: f64,
    pub alt_deg: f64,
    /// ra, dec - for coherent beams, a fixed source on the sky to track as the Earth rotates.
    /// for incoherent beams these are set to 0.0
    pub ra_deg: f64,
    pub dec_deg: f64,
    /// tle - for coherent beams, a ‘Two Line Elements’ ephemeris description string for an Earth orbiting satellite.
    pub tle: *mut c_char,
    /// nsample_avg - number of time samples to average in the output date.
    pub num_time_samples_to_average: usize,
    /// fres_hz - Output frequency resolution, in Hz.
    pub frequency_resolution_hz: u32,
    /// channel_set - array of coarse channel indicies of up to 24 coarse channels to include in the output data.
    pub coarse_channels: *mut usize,
    /// Number of coarse channels included in this beam
    pub num_coarse_chans: usize,
    /// Array of antenna indices. Must be the same as, or a subset of, the main observation tileset.
    pub antennas: *mut usize,
    /// Number of antennas included in this beam
    pub num_ants: usize,
    /// polarisation - string describing the polarisation format in the output data.
    pub polarisation: *mut c_char,
    /// data_file_type - integer index into the ‘data_file_types’ database table describing the output format for this beam.
    pub data_file_type: DataFileType,
    /// creator - arbitrary string describing the person or script that scheduled this voltage beam.
    pub creator: *mut c_char,
    /// modtime - ISO format timestamp for this voltage beam record.
    pub modtime: time_t,
    /// beam_index - Starts at zero for the first coherent beam in this observation, and increments by one for each coherent beam. Used to index into the BeamAltAz. -1 if not coherent.
    pub beam_index: i32,
}

impl Beam {
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
    pub fn populate_array(metafits_context: &MetafitsContext) -> (*mut ffi::Beam, usize) {
        let mut item_vec: Vec<ffi::Beam> = Vec::new();

        if let Some(v) = &metafits_context.metafits_beams {
            for item in v.iter() {
                // Get a list of antenna indicies for this beam
                let tileset_antenna_indices: Vec<usize> = item
                    .antennas
                    .iter()
                    .filter_map(|sel| metafits_context.antennas.iter().position(|a| a == sel))
                    .collect();

                // Get a list of coarse channel indices for this beam
                let coarse_channel_indices = item
                    .coarse_channels
                    .iter()
                    .filter_map(|sel| {
                        metafits_context
                            .metafits_coarse_chans
                            .iter()
                            .position(|c| c == sel)
                    })
                    .collect();

                let out_item = {
                    let beam::Beam {
                        number,
                        coherent,
                        az_deg,
                        alt_deg,
                        ra_deg,
                        dec_deg,
                        tle,
                        num_time_samples_to_average,
                        frequency_resolution_hz,
                        coarse_channels: _,
                        num_coarse_chans: _,
                        antennas: _,
                        num_ants: _,
                        polarisation,
                        data_file_type,
                        creator,
                        modtime,
                        beam_index,
                    } = item;

                    let (coarse_channel_indicies_ptr, coarse_channel_indicies_len) =
                        ffi_create_c_array(coarse_channel_indices);
                    let (tileset_antenna_indicies_ptr, tileset_antenna_indices_len) =
                        ffi_create_c_array(tileset_antenna_indices);

                    beam::ffi::Beam {
                        number: *number,
                        coherent: *coherent,
                        az_deg: az_deg.unwrap_or_default(),
                        alt_deg: alt_deg.unwrap_or_default(),
                        ra_deg: ra_deg.unwrap_or_default(),
                        dec_deg: dec_deg.unwrap_or_default(),
                        tle: ffi_create_c_string(tle.as_deref().unwrap_or_default()),
                        num_time_samples_to_average: *num_time_samples_to_average,
                        frequency_resolution_hz: *frequency_resolution_hz,
                        coarse_channels: coarse_channel_indicies_ptr,
                        num_coarse_chans: coarse_channel_indicies_len,
                        num_ants: tileset_antenna_indices_len,
                        antennas: tileset_antenna_indicies_ptr,
                        polarisation: ffi_create_c_string(
                            polarisation.as_deref().unwrap_or_default(),
                        ),
                        data_file_type: *data_file_type as DataFileType,
                        creator: ffi_create_c_string(creator),
                        modtime: chrono::DateTime::<chrono::Utc>::from(*modtime).timestamp(),
                        beam_index: match beam_index {
                            Some(idx) => *idx as i32,
                            None => -1,
                        },
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
    fn destroy_item(item: *mut ffi::Beam) {
        if item.is_null() {
            return;
        }

        let a = unsafe { &mut *item };

        // Free string if present
        if !a.tle.is_null() {
            ffi_free_rust_c_string(&mut a.tle);
        }

        if !a.polarisation.is_null() {
            ffi_free_rust_c_string(&mut a.polarisation);
        }

        if !a.creator.is_null() {
            ffi_free_rust_c_string(&mut a.creator);
        }

        // Free arrays
        if !a.coarse_channels.is_null() {
            ffi_free_rust_struct(a.coarse_channels, a.num_coarse_chans);
        }

        if !a.antennas.is_null() {
            ffi_free_rust_struct(a.antennas, a.num_ants);
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
    pub fn destroy_array(items_ptr: *mut ffi::Beam, items_len: usize) {
        let items_slice = unsafe { std::slice::from_raw_parts_mut(items_ptr, items_len) };
        for item in items_slice {
            Self::destroy_item(item);
        }
        // Now free the array itself by reconstructing the Vec and letting it drop
        ffi_free_rust_struct(items_ptr, items_len);
    }
}
