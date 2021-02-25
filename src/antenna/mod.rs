// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for antenna metadata
*/
use crate::rfinput::*;
use std::fmt;

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
    pub x_pol: RFInput,
    /// Reference to the Y pol rf_input
    pub y_pol: RFInput,
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
            x_pol: x_pol.clone(),
            y_pol: y_pol.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_populate_antennas() {
        // Create some rf_inputs
        let mut rf_inputs: Vec<RFInput> = Vec::new();

        rf_inputs.push(RFInput {
            input: 0,
            ant: 101,
            tile_id: 101,
            tile_name: String::from("Tile101"),
            pol: Pol::X,
            electrical_length_m: 101.,
            north_m: 11.,
            east_m: 21.,
            height_m: 31.,
            vcs_order: 0,
            subfile_order: 0,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 1,
            rec_slot_number: 0,
        });

        rf_inputs.push(RFInput {
            input: 1,
            ant: 101,
            tile_id: 101,
            tile_name: String::from("Tile101"),
            pol: Pol::Y,
            electrical_length_m: 102.,
            north_m: 12.,
            east_m: 22.,
            height_m: 32.,
            vcs_order: 4,
            subfile_order: 1,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 1,
            rec_slot_number: 1,
        });

        rf_inputs.push(RFInput {
            input: 2,
            ant: 102,
            tile_id: 102,
            tile_name: String::from("Tile102"),
            pol: Pol::X,
            electrical_length_m: 103.,
            north_m: 13.,
            east_m: 23.,
            height_m: 33.,
            vcs_order: 0,
            subfile_order: 0,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 2,
            rec_slot_number: 0,
        });

        rf_inputs.push(RFInput {
            input: 3,
            ant: 102,
            tile_id: 102,
            tile_name: String::from("Tile102"),
            pol: Pol::Y,
            electrical_length_m: 104.,
            north_m: 14.,
            east_m: 24.,
            height_m: 34.,
            vcs_order: 4,
            subfile_order: 1,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 2,
            rec_slot_number: 1,
        });

        rf_inputs.push(RFInput {
            input: 4,
            ant: 103,
            tile_id: 103,
            tile_name: String::from("Tile103"),
            pol: Pol::X,
            electrical_length_m: 105.,
            north_m: 15.,
            east_m: 25.,
            height_m: 35.,
            vcs_order: 0,
            subfile_order: 0,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 3,
            rec_slot_number: 0,
        });

        rf_inputs.push(RFInput {
            input: 5,
            ant: 103,
            tile_id: 103,
            tile_name: String::from("Tile103"),
            pol: Pol::Y,
            electrical_length_m: 106.,
            north_m: 16.,
            east_m: 26.,
            height_m: 36.,
            vcs_order: 4,
            subfile_order: 1,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 3,
            rec_slot_number: 1,
        });

        rf_inputs.push(RFInput {
            input: 6,
            ant: 104,
            tile_id: 104,
            tile_name: String::from("Tile104"),
            pol: Pol::X,
            electrical_length_m: 107.,
            north_m: 17.,
            east_m: 27.,
            height_m: 37.,
            vcs_order: 0,
            subfile_order: 0,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 4,
            rec_slot_number: 0,
        });

        rf_inputs.push(RFInput {
            input: 7,
            ant: 104,
            tile_id: 104,
            tile_name: String::from("Tile104"),
            pol: Pol::Y,
            electrical_length_m: 108.,
            north_m: 18.,
            east_m: 28.,
            height_m: 38.,
            vcs_order: 4,
            subfile_order: 1,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 4,
            rec_slot_number: 1,
        });

        // Call populate
        let antennas = Antenna::populate_antennas(&rf_inputs);

        // Check
        assert_eq!(antennas.len(), 4);
        assert_eq!(antennas[0].tile_id, 101);
        assert_eq!(antennas[0].ant, 101);
        assert_eq!(antennas[1].y_pol.pol, Pol::Y);
        assert_eq!(antennas[1].tile_name, "Tile102");
        assert_eq!(antennas[2].tile_name, "Tile103");
        assert_eq!(antennas[2].x_pol.input, 4);
        assert_eq!(antennas[3].tile_id, 104);
    }

    #[test]
    fn test_antenna_debug() {
        // Create some rf_inputs
        let mut rf_inputs: Vec<RFInput> = Vec::new();

        rf_inputs.push(RFInput {
            input: 0,
            ant: 101,
            tile_id: 101,
            tile_name: String::from("Tile101"),
            pol: Pol::X,
            electrical_length_m: 101.,
            north_m: 11.,
            east_m: 21.,
            height_m: 31.,
            vcs_order: 0,
            subfile_order: 0,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 1,
            rec_slot_number: 0,
        });

        rf_inputs.push(RFInput {
            input: 1,
            ant: 101,
            tile_id: 101,
            tile_name: String::from("Tile101"),
            pol: Pol::Y,
            electrical_length_m: 102.,
            north_m: 12.,
            east_m: 22.,
            height_m: 32.,
            vcs_order: 4,
            subfile_order: 1,
            flagged: false,
            digital_gains: vec![],
            dipole_gains: vec![],
            dipole_delays: vec![],
            rec_number: 1,
            rec_slot_number: 1,
        });

        // Call populate
        let antennas = Antenna::populate_antennas(&rf_inputs);

        assert_eq!(format!("{:?}", antennas[0]), "Tile101");
    }
}
