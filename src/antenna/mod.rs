// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for antenna metadata
*/
use crate::rfinput::*;
use std::fmt;

#[cfg(test)]
mod test;

// Structure for storing MWA antennas (tiles without polarisation) information from the metafits file
#[derive(Clone)]
pub struct Antenna {
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
    pub tile_name: String,
    /// Reference to the X pol rf_input
    pub rfinput_x: RFInput,
    /// Reference to the Y pol rf_input
    pub rfinput_y: RFInput,
}

impl Antenna {
    /// Creates a new, populated Antenna struct
    ///
    /// # Arguments
    ///
    /// * `x_pol` - A reference to an already populated RFInput struct which is the x polarisation of this antenna
    ///
    /// * `y_pol` - A reference to an already populated bRFInput struct which is the y polarisation of this antenna
    ///
    ///
    /// # Returns
    ///
    /// * An Result containing a populated Antenna struct or an Error
    ///
    pub(crate) fn new(x_pol: &RFInput, y_pol: &RFInput) -> Self {
        Self {
            ant: x_pol.ant,
            tile_id: x_pol.tile_id,
            tile_name: x_pol.tile_name.to_string(),
            rfinput_x: x_pol.clone(),
            rfinput_y: y_pol.clone(),
        }
    }

    /// Creates a populated vector of Antenna structs.
    ///
    /// # Arguments
    ///
    /// `rf_inputs` - a vector or slice of RFInputs.
    ///
    /// # Returns
    ///
    /// * A vector of populated Antenna structs.
    ///
    pub(crate) fn populate_antennas(rf_inputs: &[RFInput]) -> Vec<Antenna> {
        let mut antennas: Vec<Antenna> = Vec::with_capacity(rf_inputs.len() / 2);
        for index in (0..rf_inputs.len()).step_by(2) {
            antennas.push(Antenna::new(&rf_inputs[index], &rf_inputs[index + 1]));
        }
        antennas
    }
}

/// Implements fmt::Debug for Antenna struct
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
impl fmt::Debug for Antenna {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tile_name)
    }
}
