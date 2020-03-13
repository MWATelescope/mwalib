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

fn get_electrical_length(metafits_length_string: String, coax_v_factor: f64) -> f64 {
    if metafits_length_string.starts_with("EL_") {
        metafits_length_string
            .replace("EL_", "")
            .parse::<f64>()
            .unwrap()
    } else {
        metafits_length_string.parse::<f64>().unwrap() * coax_v_factor
    }
}

#[allow(non_camel_case_types)]
/// Structure to hold one row of the metafits tiledata table
struct mwalibRFInputMetafitsTableRow {
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
    pub length_string: String,
    /// Antenna position North from the array centre (metres)
    pub north_m: f64,
    /// Antenna position East from the array centre (metres)
    pub east_m: f64,
    /// Antenna height from the array centre (metres)
    pub height_m: f64,
    /// Is this rf_input flagged out (due to tile error, etc from metafits)
    pub flag: i32,
}

impl mwalibRFInputMetafitsTableRow {
    pub fn new(
        input: u32,
        antenna: u32,
        tile_id: u32,
        tile_name: String,
        pol: String,
        length_string: String,
        north_m: f64,
        east_m: f64,
        height_m: f64,
        flag: i32,
    ) -> mwalibRFInputMetafitsTableRow {
        mwalibRFInputMetafitsTableRow {
            input,
            antenna,
            tile_id,
            tile_name,
            pol,
            length_string,
            north_m,
            east_m,
            height_m,
            flag,
        }
    }
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
    /// Is this rf_input flagged out (due to tile error, etc from metafits)
    pub flagged: bool,
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
        flagged: bool,
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
            flagged,
        }
    }
    /// This method just reads a row from the metafits tiledata table
    fn read_metafits_values(
        metafits_fptr: &mut fitsio::FitsFile,
        metafits_tile_table_hdu: &fitsio::hdu::FitsHdu,
        row: usize,
    ) -> Result<mwalibRFInputMetafitsTableRow, ErrorKind> {
        let input = metafits_tile_table_hdu
            .read_cell_value(metafits_fptr, "Input", row)
            .with_context(|| {
                format!("Failed to read table row {} for Input from metafits.", row)
            })?;

        let antenna = metafits_tile_table_hdu
            .read_cell_value(metafits_fptr, "Antenna", row)
            .with_context(|| {
                format!(
                    "Failed to read table row {} for Antenna from metafits.",
                    row
                )
            })?;

        let tile_id = metafits_tile_table_hdu
            .read_cell_value(metafits_fptr, "Tile", row)
            .with_context(|| format!("Failed to read table row {} for Tile from metafits.", row))?;

        let tile_name = metafits_tile_table_hdu
            .read_cell_value(metafits_fptr, "TileName", row)
            .with_context(|| {
                format!(
                    "Failed to read table row {} for TileName from metafits.",
                    row
                )
            })?;

        let pol: String = metafits_tile_table_hdu
            .read_cell_value(metafits_fptr, "Pol", row)
            .with_context(|| format!("Failed to read table row {} for Pol from metafits.", row))?;
        // Length is stored as a string (no one knows why) starting with "EL_" the rest is a float so remove the prefix and get the float
        let length_desc: String = metafits_tile_table_hdu
            .read_cell_value(metafits_fptr, "Length", row)
            .with_context(|| {
                format!("Failed to read table row {} for Length from metafits.", row)
            })?;
        let north = metafits_tile_table_hdu
            .read_cell_value(metafits_fptr, "North", row)
            .with_context(|| {
                format!("Failed to read table row {} for North from metafits.", row)
            })?;

        let east = metafits_tile_table_hdu
            .read_cell_value(metafits_fptr, "East", row)
            .with_context(|| format!("Failed to read table row {} for East from metafits.", row))?;

        let height = metafits_tile_table_hdu
            .read_cell_value(metafits_fptr, "Height", row)
            .with_context(|| {
                format!("Failed to read table row {} for Height from metafits", row)
            })?;

        let flag = metafits_tile_table_hdu
            .read_cell_value(metafits_fptr, "Flag", row)
            .with_context(|| format!("Failed to read table row {} for Flag from metafits", row))?;

        Ok(mwalibRFInputMetafitsTableRow::new(
            input,
            antenna,
            tile_id,
            tile_name,
            pol,
            length_desc,
            north,
            east,
            height,
            flag,
        ))
    }

    /// Given the number of (rf)inputs, a metafits fits pointer, ptr to hdu for the tiledata table and coax_v_factor,
    /// populate a vector of rf_inputs
    pub fn populate_rf_inputs(
        num_inputs: usize,
        metafits_fptr: &mut fitsio::FitsFile,
        metafits_tile_table_hdu: fitsio::hdu::FitsHdu,
        coax_v_factor: f64,
    ) -> Result<Vec<mwalibRFInput>, ErrorKind> {
        let mut rf_inputs: Vec<mwalibRFInput> = Vec::with_capacity(num_inputs);
        for input in 0..num_inputs {
            // Note fits row numbers start at 1
            let metafits_row = mwalibRFInput::read_metafits_values(
                metafits_fptr,
                &metafits_tile_table_hdu,
                input,
            )?;

            // The metafits TILEDATA table contains 2 rows for each antenna.
            // Some metafits will have
            // EL_nnn => this is the electrical length. Just use it (chop off the EL)
            // or
            // nnn    => this needs to be multiplied by the v_factor to get the eqiuvalent EL
            let electrical_length_m =
                get_electrical_length(metafits_row.length_string, coax_v_factor);

            let vcs_order = get_vcs_order(metafits_row.input);
            let subfile_order = get_mwax_order(metafits_row.antenna, metafits_row.pol.to_string());

            rf_inputs.push(mwalibRFInput::new(
                metafits_row.input,
                metafits_row.antenna,
                metafits_row.tile_id,
                metafits_row.tile_name,
                metafits_row.pol,
                electrical_length_m,
                metafits_row.north_m,
                metafits_row.east_m,
                metafits_row.height_m,
                vcs_order,
                subfile_order,
                metafits_row.flag == 1,
            ))
        }
        Ok(rf_inputs)
    }
}

/// Implements fmt::Debug for mwalibRFInput struct
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
impl fmt::Debug for mwalibRFInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.tile_name, self.pol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate float_cmp;

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

    #[test]
    fn test_get_electrical_length() {
        assert!(float_cmp::approx_eq!(
            f64,
            123.45,
            get_electrical_length(String::from("EL_123.45"), 1.204),
            float_cmp::F64Margin::default()
        ));
        assert!(float_cmp::approx_eq!(
            f64,
            1.204 * 16.,
            get_electrical_length(String::from("16"), 1.204),
            float_cmp::F64Margin::default()
        ));
    }
}
