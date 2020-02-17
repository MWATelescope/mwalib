// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for metadata
*/
use crate::*;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum CorrelatorVersion {
    /// New correlator data (a.k.a. MWAX).
    V2,
    /// MWA raw data files with "gpubox" and batch numbers in their names.
    Legacy,
    /// gpubox files without any batch numbers.
    OldLegacy,
}

impl fmt::Display for CorrelatorVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CorrelatorVersion::V2 => "V2 (MWAX)",
                CorrelatorVersion::Legacy => "Legacy",
                CorrelatorVersion::OldLegacy => "Legacy (no file indices)",
            }
        )
    }
}

// Structure for storing MWA rf_chains (tile with polarisation) information from the metafits file
#[allow(non_camel_case_types)]
pub struct mwalibRFChain {
    pub input: u32,
    pub antenna: u32,
    pub tile_id: u32,
    pub tile_name: String,
    pub pol: String,
    pub electrical_length: f64,
    pub north: f64,
    pub east: f64,
    pub height: f64,
    pub vcs_order: u32,
}

impl mwalibRFChain {
    pub fn new(
        input: u32,
        antenna: u32,
        tile_id: u32,
        tile_name: String,
        pol: String,
        electrical_length: f64,
        north: f64,
        east: f64,
        height: f64,
        vcs_order: u32,
    ) -> Result<mwalibRFChain, ErrorKind> {
        Ok(mwalibRFChain {
            input,
            antenna,
            tile_id,
            tile_name,
            pol,
            electrical_length,
            north,
            east,
            height,
            vcs_order,
        })
    }
}

impl fmt::Debug for mwalibRFChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tile_name)
    }
}

/// This is a struct for our coarse channels
#[allow(non_camel_case_types)]
pub struct mwalibCoarseChannel {
    // Correlator channel is 0 indexed
    pub correlator_channel_number: u16,

    // Receiver channel is 0-255 in the RRI recivers
    pub receiver_channel_number: u16,

    pub channel_width_hz: u32,
    pub channel_start_hz: u32,
    pub channel_centre_hz: u32,
    pub channel_end_hz: u32,
}

impl mwalibCoarseChannel {
    pub fn new(
        correlator_channel_number: u16,
        receiver_channel_number: u16,
        channel_width_hz: u32,
    ) -> Result<mwalibCoarseChannel, ErrorKind> {
        let centre_chan_hz: u32 = (receiver_channel_number as u32) * channel_width_hz;

        Ok(mwalibCoarseChannel {
            correlator_channel_number,
            receiver_channel_number,
            channel_width_hz,
            channel_centre_hz: centre_chan_hz,
            channel_start_hz: centre_chan_hz - (channel_width_hz / 2),
            channel_end_hz: centre_chan_hz + (channel_width_hz / 2),
        })
    }
}

impl fmt::Debug for mwalibCoarseChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "corr={} rec={} @ {:.3} MHz",
            self.correlator_channel_number,
            self.receiver_channel_number,
            self.channel_centre_hz as f32 / 1000000.
        )
    }
}
