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
    pub fn new(x_pol: &mwalibRFInput, y_pol: &mwalibRFInput) -> mwalibAntenna {
        mwalibAntenna {
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
            let new_antenna: mwalibAntenna =
                mwalibAntenna::new(&rf_inputs[index], &rf_inputs[index + 1]);
            antennas.push(new_antenna);
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
impl fmt::Debug for mwalibAntenna {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tile_name)
    }
}
