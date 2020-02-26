// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for rf_input metadata
*/
use crate::*;
use std::fmt;

// VCS_ORDER is the order that comes out of PFB and into the correlator (for legacy observations)
// It can be calculated, so we do that, rather than make the user get a newer metafits (only metafits after mid 2018
// have this column pre populated).
fn get_vcs_order(input: u32) -> u32 {
    (input & 0xC0) | ((input & 0x30) >> 4) | ((input & 0x0F) << 2)
}

// mwax_order (aka subfile_order) is the order we want the antennas in, after conversion.
// For Correlator v2, the data is already in this order.
fn get_mwax_order(antenna: u32, pol: String) -> u32 {
    assert!(antenna < 128);
    (antenna * 2) + (if pol == "Y" { 1 } else { 0 })
}

// Structure for storing MWA rf_chains (tile with polarisation) information from the metafits file
#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct mwalibRFInput {
    /// This is the metafits order (0-n inputs)
    pub input: u32,
    /// This is the antenna number.
    /// Nominally this is the field we sort by to get the desired output order of antenna.
    /// X and Y have the same antenna number. This is the sorted ordinal order of the antenna.None
    /// e.g. 0...N-1
    pub antenna: u32,
    /// Numeric part of tile_name for the antenna. Each pol has the same value
    /// e.g. tile_name "tile011" hsa tile_id of 11
    pub tile_id: u32,
    /// Human readable name of the antenna
    /// X and Y have the same name
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
    ) -> mwalibRFInput {
        mwalibRFInput {
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
        }
    }

    pub fn populate_rf_input(
        num_inputs: usize,
        metafits_fptr: &mut fitsio::FitsFile,
        metafits_tile_table_hdu: fitsio::hdu::FitsHdu,
    ) -> Result<Vec<mwalibRFInput>, ErrorKind> {
        let mut rf_inputs: Vec<mwalibRFInput> = Vec::with_capacity(num_inputs);
        for input in 0..num_inputs {
            // Note fits row numbers start at 1

            // The metafits TILEDATA table contains 2 rows for each antenna.
            let table_input = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "Input", input)
                .with_context(|| {
                    format!(
                        "Failed to read table row {} for Input from metafits.",
                        input
                    )
                })?;

            let table_antenna = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "Antenna", input)
                .with_context(|| {
                    format!(
                        "Failed to read table row {} for Antenna from metafits.",
                        input
                    )
                })?;

            let table_tile_id = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "Tile", input)
                .with_context(|| {
                    format!("Failed to read table row {} for Tile from metafits.", input)
                })?;

            let table_tile_name = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "TileName", input)
                .with_context(|| {
                    format!(
                        "Failed to read table row {} for TileName from metafits.",
                        input
                    )
                })?;

            let table_pol: String = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "Pol", input)
                .with_context(|| {
                    format!("Failed to read table row {} for Pol from metafits.", input)
                })?;
            // Length is stored as a string (no one knows why) starting with "EL_" the rest is a float so remove the prefix and get the float
            let table_electrical_length_desc: String = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "Length", input)
                .with_context(|| {
                    format!(
                        "Failed to read table row {} for Length from metafits.",
                        input
                    )
                })?;
            let table_electrical_length = table_electrical_length_desc
                .replace("EL_", "")
                .parse()
                .unwrap();

            let table_north = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "North", input)
                .with_context(|| {
                    format!(
                        "Failed to read table row {} for North from metafits.",
                        input
                    )
                })?;

            let table_east = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "East", input)
                .with_context(|| {
                    format!("Failed to read table row {} for East from metafits.", input)
                })?;

            let table_height = metafits_tile_table_hdu
                .read_cell_value(metafits_fptr, "Height", input)
                .with_context(|| {
                    format!(
                        "Failed to read table row {} for Height from metafits",
                        input
                    )
                })?;

            let vcs_order = get_vcs_order(table_input);
            let subfile_order = get_mwax_order(table_antenna, table_pol.to_string());

            rf_inputs.push(mwalibRFInput::new(
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
            ))
        }
        Ok(rf_inputs)
    }
}

impl fmt::Debug for mwalibRFInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.tile_name, self.pol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_vcs_order() {
        assert_eq!(0, get_vcs_order(0));
        assert_eq!(4, get_vcs_order(1));
        assert_eq!(32, get_vcs_order(8));
        assert_eq!(127, get_vcs_order(127));
        assert_eq!(128, get_vcs_order(128));
        assert_eq!(194, get_vcs_order(224));
        assert_eq!(251, get_vcs_order(254));
        assert_eq!(255, get_vcs_order(255));
    }

    #[test]
    fn test_get_mwax_order() {
        assert_eq!(0, get_mwax_order(0, String::from("X")));
        assert_eq!(1, get_mwax_order(0, String::from("Y")));
        assert_eq!(32, get_mwax_order(16, String::from("X")));
        assert_eq!(33, get_mwax_order(16, String::from("Y")));
        assert_eq!(120, get_mwax_order(60, String::from("X")));
        assert_eq!(121, get_mwax_order(60, String::from("Y")));
        assert_eq!(254, get_mwax_order(127, String::from("X")));
        assert_eq!(255, get_mwax_order(127, String::from("Y")));
    }
}
