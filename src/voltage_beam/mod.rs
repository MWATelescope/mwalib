use core::f64;
use std::fmt;

use crate::{read_optional_cell_string_value, CoarseChannel};
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::{
    read_cell_array_u32, read_cell_value, read_optional_cell_value, types::DataFileType, Antenna,
    MAX_ANTENNAS,
};
use chrono::{DateTime, FixedOffset};
use fitsio::hdu::{FitsHdu, HduInfo};
use fitsio::FitsFile;
use num_traits::FromPrimitive;
#[cfg(any(feature = "python", feature = "python-stubgen"))]
use pyo3::prelude::*;
#[cfg(feature = "python-stubgen")]
use pyo3_stub_gen_derive::gen_stub_pyclass;

pub mod error;
pub mod ffi;

#[cfg(test)]
mod test;

#[cfg_attr(feature = "python-stubgen", gen_stub_pyclass)]
#[cfg_attr(
    any(feature = "python", feature = "python-stubgen"),
    pyclass(get_all, set_all)
)]
#[derive(Clone, Debug, PartialEq)]
pub struct VoltageBeam {
    /// Arbitrary integer identifying this beam
    pub number: usize,
    /// True if the beam is coherent (has a defined position on the sky), False if incoherent (indicating that all tiles should be summed without phase correction)
    pub coherent: bool,
    /// azimuth, elevation - for coherent beams, these describe a fixed position relative to the telescope centre (eg, a geosynchronous satellite or ground-based RFI source)
    pub az_deg: Option<f64>,
    pub alt_deg: Option<f64>,
    /// ra, dec - for coherent beams, a fixed source on the sky to track as the Earth rotates.
    pub ra_deg: Option<f64>,
    pub dec_deg: Option<f64>,
    /// tle - for coherent beams, a ‘Two Line Elements’ ephemeris description string for an Earth orbiting satellite.
    pub tle: Option<String>,
    /// nsample_avg - number of time samples to average in the output date.
    pub num_time_samples_to_average: usize,
    /// fres_hz - Output frequency resolution, in Hz.
    pub frequency_resolution_hz: u32,
    /// channel_set - list of up to 24 coarse channels to include in the output data. If not present, include all 24 coarse channels.
    pub coarse_channels: Vec<CoarseChannel>,
    /// Number of coarse channels included in this beam
    pub num_coarse_chans: usize,
    /// tileset - Array of antennas which are the tiles used for this beam. Must be the same as, or a subset of, the main observation tileset.
    pub antennas: Vec<Antenna>,
    /// num_antennas - number of antennas / tiles used for this beam
    pub num_ants: usize,
    /// polarisation - string describing the polarisation format in the output data.
    pub polarisation: Option<String>,
    /// data_file_type - integer index into the ‘data_file_types’ database table describing the output format for this beam.
    pub data_file_type: DataFileType,
    /// creator - arbitrary string describing the person or script that scheduled this voltage beam.
    pub creator: String,
    /// modtime - ISO format timestamp for this voltage beam record.
    pub modtime: DateTime<FixedOffset>,
    /// beam_index - Starts at zero for the first coherent beam in this observation, and increments by one for each coherent beam. Used to index into the BeamAltAz
    pub beam_index: Option<usize>,
}

/// Read the voltagebeam FitsHdu and return a populated vector of `Beam`s
///
/// # Arguments
///
/// * `metafits_fptr` - reference to the FitsFile representing the metafits file.
///
/// * `voltagebeams_hdu` - The FitsHdu containing valid voltagebeams data.
///
/// * `num_coarse_channels` - The number of coarse channels in the observation (usually 24).
///
/// # Returns
///
/// * Result containing a vector of voltage beams read from the voltagebeams HDU.
///
pub(crate) fn populate_voltage_beams(
    metafits_fptr: &mut FitsFile,
    voltagebeams_hdu: &FitsHdu,
    coarse_channels: &[CoarseChannel],
    antennas: &[Antenna],
) -> Result<Vec<VoltageBeam>, error::BeamError> {
    // Find out how many beams there are in the table
    let rows = match &voltagebeams_hdu.info {
        HduInfo::TableInfo {
            column_descriptions: _,
            num_rows,
        } => *num_rows,
        _ => 0,
    };

    let mut beam_vec: Vec<VoltageBeam> = Vec::new();

    for row in 0..rows {
        let number: u64 = read_cell_value(metafits_fptr, voltagebeams_hdu, "number", row)?;
        let coherent: i32 = read_cell_value(metafits_fptr, voltagebeams_hdu, "coherent", row)?;
        let az_deg: Option<f64> =
            read_optional_cell_value(metafits_fptr, voltagebeams_hdu, "azimuth", row)?;
        let alt_deg: Option<f64> =
            read_optional_cell_value(metafits_fptr, voltagebeams_hdu, "elevation", row)?;
        let ra_deg: Option<f64> =
            read_optional_cell_value(metafits_fptr, voltagebeams_hdu, "ra", row)?;
        let dec_deg: Option<f64> =
            read_optional_cell_value(metafits_fptr, voltagebeams_hdu, "dec", row)?;
        let tle: Option<String> =
            read_optional_cell_value(metafits_fptr, voltagebeams_hdu, "tle", row)?;
        let num_time_samples_to_average: u64 =
            read_cell_value(metafits_fptr, voltagebeams_hdu, "nsample_avg", row)?;
        let frequency_resolution_hz: u32 =
            read_cell_value(metafits_fptr, voltagebeams_hdu, "fres_hz", row)?;
        let coarse_channels_string: Option<String> =
            read_optional_cell_string_value(metafits_fptr, voltagebeams_hdu, "channel_set", row)?;
        let tileset: Vec<u32> = read_cell_array_u32(
            metafits_fptr,
            voltagebeams_hdu,
            "tileset",
            row,
            MAX_ANTENNAS,
        )?;
        let polarisation: Option<String> =
            read_optional_cell_value(metafits_fptr, voltagebeams_hdu, "polarisation", row)?;
        let data_file_type_index: u32 =
            read_cell_value(metafits_fptr, voltagebeams_hdu, "data_file_type", row)?;
        let creator: String = read_cell_value(metafits_fptr, voltagebeams_hdu, "creator", row)?;
        let modtime_string: String =
            read_cell_value(metafits_fptr, voltagebeams_hdu, "modtime", row)?;
        let modtime = DateTime::parse_from_rfc3339(&modtime_string).unwrap();
        let beam_index: Option<u64> =
            read_optional_cell_value(metafits_fptr, voltagebeams_hdu, "beam_index", row)?;
        let data_file_type = match DataFileType::from_u32(data_file_type_index) {
            Some(dft) => dft,
            None => DataFileType::UnknownType,
        };

        // Determine which antennas are in this beam's tileset
        let mut beam_antennas: Vec<Antenna> = Vec::new();
        for tile_id in tileset.iter() {
            if let Some(ant) = antennas.iter().find(|a| a.tile_id == *tile_id) {
                beam_antennas.push(ant.clone());
            }
        }
        let num_beam_antennas = beam_antennas.len();

        // Determine which coarse channels to include in this beam
        let beam_coarse_channels: Vec<CoarseChannel> = match coarse_channels_string {
            Some(chan_set) => CoarseChannel::get_metafits_coarse_chan_array(&chan_set)
                .iter()
                .filter_map(|chan_num| {
                    coarse_channels
                        .iter()
                        .find(|cc| cc.rec_chan_number == *chan_num)
                })
                .cloned()
                .collect(),
            // No channel set specified - include a copy of all coarse channels
            None => coarse_channels.to_vec(),
        };
        let num_beam_coarse_channels = beam_coarse_channels.len();

        beam_vec.push(VoltageBeam {
            number: number as usize,
            coherent: coherent == 1,
            az_deg,
            alt_deg,
            ra_deg,
            dec_deg,
            tle,
            num_time_samples_to_average: num_time_samples_to_average as usize,
            frequency_resolution_hz,
            coarse_channels: beam_coarse_channels,
            num_coarse_chans: num_beam_coarse_channels,
            antennas: beam_antennas,
            num_ants: num_beam_antennas,
            polarisation,
            data_file_type,
            creator,
            modtime,
            beam_index: beam_index.map(|bi| bi as usize),
        });
    }

    Ok(beam_vec)
}

/// Implements fmt::Display for Beam struct
///
/// # Arguments
///
/// * `f` - A fmt::Formatter
///
///
/// # Returns
///
/// * `fmt::Result` - Result of this method
///
///
impl fmt::Display for VoltageBeam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Voltage beam: {} {{Type: {}, Tiles: {}, Coarse chans: {}, RA: {:.3} deg, Dec: {:.3} deg}}",
            self.number,
            if self.coherent {
                String::from("Coherent")
            } else {
                String::from("Incoherent")
            },
            self.num_ants,
            self.num_coarse_chans,
            self.ra_deg.unwrap_or(f64::NAN),
            self.dec_deg.unwrap_or(f64::NAN),
        )
    }
}
