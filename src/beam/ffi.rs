// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ffi::c_char;

use libc::time_t;

use crate::types::DataFileType;

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
