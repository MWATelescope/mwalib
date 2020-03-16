// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for antenna metadata
*/
use crate::rfinput::*;
use std::fmt;

// Structure for storing MWA antennas (tiles without polarisation) information from the metafits file
#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct mwalibAntenna {
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
    /// Reference to the X pol rf_input
    pub x_pol: mwalibRFInput,
    /// Reference to the Y pol rf_input
    pub y_pol: mwalibRFInput,
}

impl mwalibAntenna {
    /// Creates a new, populated mwalibAntenna struct
    ///
    /// # Arguments
    ///
    /// * `x_pol` - A reference to an already populated mwalibRFInput struct which is the x polarisation of this antenna
    ///
    /// * `y_pol` - A reference to an already populated mwalibRFInput struct which is the y polarisation of this antenna
    ///
    ///
    /// # Returns
    ///
    /// * An Result containing a populated mwalibAntenna struct or an Error
    ///
    pub fn new(x_pol: &mwalibRFInput, y_pol: &mwalibRFInput) -> Self {
        Self {
            antenna: x_pol.antenna,
            tile_id: x_pol.tile_id,
            tile_name: x_pol.tile_name.to_string(),
            x_pol: x_pol.clone(),
            y_pol: y_pol.clone(),
        }
    }

    /// Creates a populated vector of mwalibAntenna structs.
    ///
    /// # Arguments
    ///
    /// `rf_inputs` - a vector or slice of mwalibRFInputs.
    ///
    /// # Returns
    ///
    /// * A vector of populated mwalibAntenna structs.
    ///
    pub fn populate_antennas(rf_inputs: &[mwalibRFInput]) -> Vec<mwalibAntenna> {
        let mut antennas: Vec<mwalibAntenna> = Vec::with_capacity(rf_inputs.len() / 2);
        for index in (0..rf_inputs.len()).step_by(2) {
            antennas.push(mwalibAntenna::new(&rf_inputs[index], &rf_inputs[index + 1]));
        }
        antennas
    }
}

/// Implements fmt::Debug for mwalibAntenna struct
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
#[cfg_attr(tarpaulin, skip)]
impl fmt::Debug for mwalibAntenna {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tile_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_populate_antennas() {
        // Create some rf_inputs
        let mut rf_inputs: Vec<mwalibRFInput> = Vec::new();

        rf_inputs.push(mwalibRFInput {
            input: 0,
            antenna: 101,
            tile_id: 101,
            tile_name: String::from("Tile101"),
            pol: String::from("X"),
            electrical_length_m: 101.,
            north_m: 11.,
            east_m: 21.,
            height_m: 31.,
            vcs_order: 0,
            subfile_order: 0,
            flagged: false,
        });

        rf_inputs.push(mwalibRFInput {
            input: 1,
            antenna: 101,
            tile_id: 101,
            tile_name: String::from("Tile101"),
            pol: String::from("Y"),
            electrical_length_m: 102.,
            north_m: 12.,
            east_m: 22.,
            height_m: 32.,
            vcs_order: 4,
            subfile_order: 1,
            flagged: false,
        });

        rf_inputs.push(mwalibRFInput {
            input: 2,
            antenna: 102,
            tile_id: 102,
            tile_name: String::from("Tile102"),
            pol: String::from("X"),
            electrical_length_m: 103.,
            north_m: 13.,
            east_m: 23.,
            height_m: 33.,
            vcs_order: 0,
            subfile_order: 0,
            flagged: false,
        });

        rf_inputs.push(mwalibRFInput {
            input: 3,
            antenna: 102,
            tile_id: 102,
            tile_name: String::from("Tile102"),
            pol: String::from("Y"),
            electrical_length_m: 104.,
            north_m: 14.,
            east_m: 24.,
            height_m: 34.,
            vcs_order: 4,
            subfile_order: 1,
            flagged: false,
        });

        rf_inputs.push(mwalibRFInput {
            input: 4,
            antenna: 103,
            tile_id: 103,
            tile_name: String::from("Tile103"),
            pol: String::from("X"),
            electrical_length_m: 105.,
            north_m: 15.,
            east_m: 25.,
            height_m: 35.,
            vcs_order: 0,
            subfile_order: 0,
            flagged: false,
        });

        rf_inputs.push(mwalibRFInput {
            input: 5,
            antenna: 103,
            tile_id: 103,
            tile_name: String::from("Tile103"),
            pol: String::from("Y"),
            electrical_length_m: 106.,
            north_m: 16.,
            east_m: 26.,
            height_m: 36.,
            vcs_order: 4,
            subfile_order: 1,
            flagged: false,
        });

        rf_inputs.push(mwalibRFInput {
            input: 6,
            antenna: 104,
            tile_id: 104,
            tile_name: String::from("Tile104"),
            pol: String::from("X"),
            electrical_length_m: 107.,
            north_m: 17.,
            east_m: 27.,
            height_m: 37.,
            vcs_order: 0,
            subfile_order: 0,
            flagged: false,
        });

        rf_inputs.push(mwalibRFInput {
            input: 7,
            antenna: 104,
            tile_id: 104,
            tile_name: String::from("Tile104"),
            pol: String::from("Y"),
            electrical_length_m: 108.,
            north_m: 18.,
            east_m: 28.,
            height_m: 38.,
            vcs_order: 4,
            subfile_order: 1,
            flagged: false,
        });

        // Call populate
        let antennas = mwalibAntenna::populate_antennas(&rf_inputs);

        // Check
        assert_eq!(antennas.len(), 4);
        assert_eq!(antennas[0].tile_id, 101);
        assert_eq!(antennas[0].antenna, 101);
        assert_eq!(antennas[1].y_pol.pol, "Y");
        assert_eq!(antennas[1].tile_name, "Tile102");
        assert_eq!(antennas[2].tile_name, "Tile103");
        assert_eq!(antennas[2].x_pol.input, 4);
        assert_eq!(antennas[3].tile_id, 104);
    }
}
