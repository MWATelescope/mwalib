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
