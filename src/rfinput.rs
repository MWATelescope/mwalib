// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for rf_input metadata
*/
use crate::*;
use std::fmt;

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
pub struct mwalibRFInput {
    /// This is the metafits order (0-n inputs)
    pub input: u32,
    /// This is the antenna number.
    /// Nominally this is the field we sort by to get the desired output order of antenna.
    /// X and Y have the same antenna number.
    pub antenna: u32,
    /// Yet another descriptor for the antenna but includes polarisation
    pub tile_id: u32,
    /// Human readable name of the antenna
    pub tile_name: String,
    /// Polarisation - X & Y
    pub pol: String,
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
}

impl mwalibRFInput {
    pub fn new(
        input: u32,
        antenna: u32,
        tile_id: u32,
        tile_name: String,
        pol: String,
        electrical_length_m: f64,
        north_m: f64,
        east_m: f64,
        height_m: f64,
        vcs_order: u32,
        subfile_order: u32,
    ) -> Result<mwalibRFInput, ErrorKind> {
        Ok(mwalibRFInput {
            input,
            antenna,
            tile_id,
            tile_name,
            pol,
            electrical_length_m,
            north_m,
            east_m,
            height_m,
            vcs_order,
            subfile_order,
        })
    }

    pub fn populate_rf_input(
        num_inputs: usize,
        metafits_fptr: &mut fitsio::FitsFile,
        metafits_tile_table_hdu: fitsio::hdu::FitsHdu,
    ) -> Result<Vec<mwalibRFInput>, ErrorKind> {
        let mut rf_inputs: Vec<mwalibRFInput> = Vec::with_capacity(num_inputs);
        for input in 1..num_inputs {
            // Note fits row numbers start at 1

            // The metafits TILEDATA table contains 2 rows for each antenna.
            let table_input = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "Input", input)
                .with_context(|| format!("Failed to read table for Input from metafits."))?;

            let table_antenna = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "Antenna", input)
                .with_context(|| format!("Failed to read table for Antenna from metafits."))?;

            let table_tile_id = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "Tile", input)
                .with_context(|| format!("Failed to read table for Tile from metafits."))?;

            let table_tile_name = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "TileName", input)
                .with_context(|| format!("Failed to read table for TileName from metafits."))?;

            let table_pol = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "Pol", input)
                .with_context(|| format!("Failed to read table for Pol from metafits."))?;
            // Length is stored as a string (no one knows why) starting with "EL_" the rest is a float so remove the prefix and get the float
            let table_electrical_length_desc: String = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "Length", input)
                .with_context(|| format!("Failed to read table for Length from metafits."))?;
            let table_electrical_length = table_electrical_length_desc
                .replace("EL_", "")
                .parse()
                .unwrap();

            let table_north = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "North", input)
                .with_context(|| format!("Failed to read table for North from metafits."))?;

            let table_east = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "East", input)
                .with_context(|| format!("Failed to read table for East from metafits."))?;

            let table_height = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "Height", input)
                .with_context(|| format!("Failed to read table for Height from metafits"))?;
            // VCS_ORDER is the order that comes out of PFB and into the correlator (for legacy observations)
            // It can be calculated, so we do that, rather than make the user get a newer metafits (only metafits after mid 2018
            // have this column pre populated).
            let vcs_order =
                (table_input & 0xC0) | ((table_input & 0x30) >> 4) | ((table_input & 0x0F) << 2);

            // Subfile_order is the order we want the antennas in, after conversion. For Correlator v2, the data is already in this order.
            let subfile_order =
                (table_antenna << 1) + (if table_pol == "Y" { 1 } else { 0 }) as u32;

            rf_inputs.push(
                mwalibRFInput::new(
                    table_input,
                    table_antenna,
                    table_tile_id,
                    table_tile_name,
                    table_pol,
                    table_electrical_length,
                    table_north,
                    table_east,
                    table_height,
                    vcs_order,
                    subfile_order,
                )
                .unwrap(),
            )
        }

        // Sort the Antenna vector by the "subfile" column to get the actual order of tiles we want in the output
        rf_inputs.sort_by(|a, b| a.subfile_order.cmp(&b.subfile_order));

        return Ok(rf_inputs);
    }
}

impl fmt::Debug for mwalibRFInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{} ({})", self.tile_name, self.pol, self.vcs_order)
    }
}