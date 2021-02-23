// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for rf_input metadata
*/
pub mod error;
use error::RfinputError;

use std::fmt;

/// VCS_ORDER is the order that comes out of PFB and into the correlator (for legacy observations)
/// It can be calculated, so we do that, rather than make the user get a newer metafits (only metafits after mid 2018
/// have this column pre populated).
///
///
/// # Arguments
///
/// `input` - Value from the "input" column in the metafits TILEDATA table.
///
/// # Returns
///
/// * The PFB order - in other MWA code this is a hardcoded array but we prefer to calculate it.
///
fn get_vcs_order(input: u32) -> u32 {
    (input & 0xC0) | ((input & 0x30) >> 4) | ((input & 0x0F) << 2)
}

/// mwax_order (aka subfile_order) is the order we want the antennas in, after conversion.
/// For Correlator v2, the data is already in this order.
///
/// # Arguments
///
/// `antenna` - value from the "antenna" column of the metafits TILEDATA table.
///
/// `pol` - polarisation (X or Y)
///
/// # Returns
///
/// * a number between 0 and N-1 (where N is the number of tiles * 2). First tile would have 0 for X, 1 for Y.
///   Second tile would have 2 for X, 3 for Y, etc.
///
fn get_mwax_order(antenna: u32, pol: Pol) -> u32 {
    (antenna * 2) + (if pol == Pol::Y { 1 } else { 0 })
}

/// Returns the electrical length for this rf_input.
///
///
/// # Arguments
///
/// `metafits_length_string` - The text from the "Length" field in the metafits TILEDATA table.
///                            May be a string like "nnn.nnn" or "EL_nnn.nn".
///
/// `coax_v_factor` - A constant value for deriving the electrical length based on the properties of the coax used.
///
/// # Returns
///
/// * An f64 containing the electrical length. If Length string contains "EL_" then just get the numeric part. If not, we need
///   to multiply the string (converted into a number) by the coax_v_factor.
///
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

/// Instrument polarisation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pol {
    X,
    Y,
}

/// Implements fmt::Display for Pol
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
impl fmt::Display for Pol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Pol::X => "X",
                Pol::Y => "Y",
            }
        )
    }
}

/// Structure to hold one row of the metafits tiledata table
struct RFInputMetafitsTableRow {
    /// This is the ordinal index of the rf_input in the metafits file
    pub input: u32,
    /// This is the antenna number.
    /// Nominally this is the field we sort by to get the desired output order of antenna.
    /// X and Y have the same antenna number. This is the sorted ordinal order of the antenna.None
    /// e.g. 0...N-1
    pub antenna: u32,
    /// Numeric part of tile_name for the antenna. Each pol has the same value
    /// e.g. tile_name "tile011" has a tile_id of 11
    pub tile_id: u32,
    /// Human readable name of the antenna
    /// X and Y have the same name
    pub tile_name: String,
    /// Polarisation - X or Y
    pub pol: Pol,
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
    /// Digital gains
    pub gains: Vec<i16>,
    /// Dipole delays
    pub delays: Vec<i16>,
    /// Receiver number
    pub rx: u32,
    /// Receiver slot number
    pub slot: u32,
}

// Structure for storing MWA rf_chains (tile with polarisation) information from the metafits file
#[derive(Clone)]
pub struct RFInput {
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
    /// Polarisation - X or Y
    pub pol: Pol,
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
    pub gains: Vec<i16>,
    /// Dipole delays
    pub delays: Vec<i16>,
    /// Receiver number
    pub receiver_number: u32,
    /// Receiver slot number
    pub receiver_slot_number: u32,
}

impl RFInput {
    /// This method just reads a row from the metafits tiledata table to create a new, populated mwalibCoarseChannel struct
    ///
    /// # Arguments
    ///
    /// * `metafits_fptr` - reference to the FitsFile representing the metafits file.
    ///
    /// * `metafits_tile_table_hdu` - reference to the HDU containing the TILEDATA table.
    ///
    /// * `row` - row index to read from the TILEDATA table in the metafits.
    ///
    ///
    /// # Returns
    ///
    /// * An Result containing a populated vector of RFInputMetafitsTableRow structss or an Error
    ///
    fn read_metafits_values(
        metafits_fptr: &mut fitsio::FitsFile,
        metafits_tile_table_hdu: &fitsio::hdu::FitsHdu,
        row: usize,
    ) -> Result<RFInputMetafitsTableRow, RfinputError> {
        let input = read_cell_value(metafits_fptr, metafits_tile_table_hdu, "Input", row)?;
        let antenna = read_cell_value(metafits_fptr, metafits_tile_table_hdu, "Antenna", row)?;
        let tile_id = read_cell_value(metafits_fptr, metafits_tile_table_hdu, "Tile", row)?;
        let tile_name = read_cell_value(metafits_fptr, metafits_tile_table_hdu, "TileName", row)?;
        let pol = {
            let p: String = read_cell_value(metafits_fptr, metafits_tile_table_hdu, "Pol", row)?;
            match p.as_str() {
                "X" => Pol::X,
                "Y" => Pol::Y,
                _ => {
                    return Err(RfinputError::UnrecognisedPol {
                        fits_filename: metafits_fptr.filename.clone(),
                        hdu_num: metafits_tile_table_hdu.number + 1,
                        row_num: row,
                        got: p,
                    })
                }
            }
        };
        // Length is stored as a string (no one knows why) starting with "EL_" the rest is a float so remove the prefix and get the float
        let length_string: String =
            read_cell_value(metafits_fptr, metafits_tile_table_hdu, "Length", row)?;
        let north_m = read_cell_value(metafits_fptr, metafits_tile_table_hdu, "North", row)?;
        let east_m = read_cell_value(metafits_fptr, metafits_tile_table_hdu, "East", row)?;
        let height_m = read_cell_value(metafits_fptr, metafits_tile_table_hdu, "Height", row)?;
        let flag = read_cell_value(metafits_fptr, metafits_tile_table_hdu, "Flag", row)?;
        let gains = read_cell_array(
            metafits_fptr,
            metafits_tile_table_hdu,
            "Gains",
            row as i64,
            24,
        )?;
        let delays = read_cell_array(
            metafits_fptr,
            metafits_tile_table_hdu,
            "Delays",
            row as i64,
            16,
        )?;
        let rx = read_cell_value(metafits_fptr, metafits_tile_table_hdu, "Rx", row)?;
        let slot = read_cell_value(metafits_fptr, metafits_tile_table_hdu, "Slot", row)?;

        Ok(RFInputMetafitsTableRow {
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
            gains,
            delays,
            rx,
            slot,
        })
    }

    /// Given the number of (rf)inputs, a metafits fits pointer, ptr to hdu for the tiledata table and coax_v_factor,
    /// populate a vector of rf_inputs
    ///
    /// # Arguments
    ///
    /// * `num_inputs` - number of rf_inputs to read from the metafits TILEDATA bintable.
    ///
    /// * `metafits_fptr` - reference to the FitsFile representing the metafits file.
    ///
    /// * `metafits_tile_table_hdu` - reference to the HDU containing the TILEDATA table.
    ///
    /// * `coax_v_factor` - a constant- the factor to apply to some older metafits "length" value to get the
    ///                     electrical length, if "length" does not start with "EL".
    ///
    ///
    /// # Returns
    ///
    /// * An Result containing a populated vector of RFInputMetafitsTableRow structss or an Error
    ///
    pub fn populate_rf_inputs(
        num_inputs: usize,
        metafits_fptr: &mut fitsio::FitsFile,
        metafits_tile_table_hdu: fitsio::hdu::FitsHdu,
        coax_v_factor: f64,
    ) -> Result<Vec<Self>, RfinputError> {
        let mut rf_inputs: Vec<Self> = Vec::with_capacity(num_inputs);
        for input in 0..num_inputs {
            // Note fits row numbers start at 1
            let metafits_row =
                Self::read_metafits_values(metafits_fptr, &metafits_tile_table_hdu, input)?;

            // The metafits TILEDATA table contains 2 rows for each antenna.
            // Some metafits will have
            // EL_nnn => this is the electrical length. Just use it (chop off the EL)
            // or
            // nnn    => this needs to be multiplied by the v_factor to get the eqiuvalent EL
            let electrical_length_m =
                get_electrical_length(metafits_row.length_string, coax_v_factor);

            let vcs_order = get_vcs_order(metafits_row.input);
            let subfile_order = get_mwax_order(metafits_row.antenna, metafits_row.pol);

            rf_inputs.push(Self {
                input: metafits_row.input,
                antenna: metafits_row.antenna,
                tile_id: metafits_row.tile_id,
                tile_name: metafits_row.tile_name,
                pol: metafits_row.pol,
                electrical_length_m,
                north_m: metafits_row.north_m,
                east_m: metafits_row.east_m,
                height_m: metafits_row.height_m,
                vcs_order,
                subfile_order,
                flagged: metafits_row.flag == 1,
                gains: metafits_row.gains,
                delays: metafits_row.delays,
                receiver_number: metafits_row.rx,
                receiver_slot_number: metafits_row.slot,
            })
        }
        Ok(rf_inputs)
    }
}

fn read_cell_value<T: fitsio::tables::ReadsCol>(
    metafits_fptr: &mut fitsio::FitsFile,
    metafits_tile_table_hdu: &fitsio::hdu::FitsHdu,
    col_name: &str,
    row: usize,
) -> Result<T, RfinputError> {
    match metafits_tile_table_hdu.read_cell_value(metafits_fptr, col_name, row) {
        Ok(c) => Ok(c),
        Err(_) => Err(RfinputError::ReadCell {
            fits_filename: metafits_fptr.filename.clone(),
            hdu_num: metafits_tile_table_hdu.number + 1,
            row_num: row,
            col_name: col_name.to_string(),
        }),
    }
}

/// Pull out the array-in-a-cell values. This function assumes that the output
/// datatype is i16, and that the fits datatype is TINT, so it is not to be used
/// generally!
fn read_cell_array(
    metafits_fptr: &mut fitsio::FitsFile,
    metafits_tile_table_hdu: &fitsio::hdu::FitsHdu,
    col_name: &str,
    row: i64,
    n_elem: i64,
) -> Result<Vec<i16>, RfinputError> {
    unsafe {
        // With the column name, get the column number.
        let mut status = 0;
        let mut col_num = -1;
        let keyword = std::ffi::CString::new(col_name).unwrap();
        fitsio_sys::ffgcno(
            metafits_fptr.as_raw(),
            0,
            keyword.as_ptr() as *mut libc::c_char,
            &mut col_num,
            &mut status,
        );
        // Check the status.
        if status != 0 {
            return Err(RfinputError::CellArray {
                fits_filename: metafits_fptr.filename.clone(),
                hdu_num: metafits_tile_table_hdu.number + 1,
                row_num: row,
                col_name: col_name.to_string(),
            });
        }

        // Now get the specified row from that column.
        // cfitsio is stupid. The data we want fits in i16, but we're forced to
        // unpack it into i32. Demote the data at the end.
        let mut array: Vec<u32> = vec![0; n_elem as usize];
        array.shrink_to_fit();
        let array_ptr = array.as_mut_ptr();
        fitsio_sys::ffgcv(
            metafits_fptr.as_raw(),
            31,
            col_num,
            row + 1,
            1,
            n_elem,
            std::ptr::null_mut(),
            array_ptr as *mut core::ffi::c_void,
            &mut 0,
            &mut status,
        );

        // Check the status.
        match status {
            0 => {
                // Re-assemble the raw array into a Rust Vector.
                let v = std::slice::from_raw_parts(array_ptr, n_elem as usize);
                Ok(v.iter().map(|v| *v as _).collect())
            }
            _ => Err(RfinputError::CellArray {
                fits_filename: metafits_fptr.filename.clone(),
                hdu_num: metafits_tile_table_hdu.number + 1,
                row_num: row,
                col_name: col_name.to_string(),
            }),
        }
    }
}

/// Implements fmt::Debug for RFInput struct
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
impl fmt::Debug for RFInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.tile_name, self.pol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use fitsio::tables::{ColumnDataType, ColumnDescription};

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
        assert_eq!(0, get_mwax_order(0, Pol::X));
        assert_eq!(1, get_mwax_order(0, Pol::Y));
        assert_eq!(32, get_mwax_order(16, Pol::X));
        assert_eq!(33, get_mwax_order(16, Pol::Y));
        assert_eq!(120, get_mwax_order(60, Pol::X));
        assert_eq!(121, get_mwax_order(60, Pol::Y));
        assert_eq!(254, get_mwax_order(127, Pol::X));
        assert_eq!(255, get_mwax_order(127, Pol::Y));
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

    #[test]
    fn test_read_cell_array() {
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
        let mut fptr = fits_open!(&metafits_filename).unwrap();
        let hdu = fits_open_hdu!(&mut fptr, 1).unwrap();

        let delays = read_cell_array(&mut fptr, &hdu, "Delays", 0, 16);
        assert!(delays.is_ok());

        let gains = read_cell_array(&mut fptr, &hdu, "Gains", 0, 24);
        assert!(gains.is_ok());

        let asdf = read_cell_array(&mut fptr, &hdu, "NotReal", 0, 24);
        assert!(asdf.is_err());
    }

    #[test]
    fn test_read_metafits_values_from_row_0() {
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
        let mut metafits_fptr = fits_open!(&metafits_filename).unwrap();

        let metafits_tile_table_hdu = fits_open_hdu!(&mut metafits_fptr, 1).unwrap();

        // Get values from row 1
        let row: RFInputMetafitsTableRow =
            RFInput::read_metafits_values(&mut metafits_fptr, &metafits_tile_table_hdu, 0).unwrap();
        assert_eq!(row.input, 0);
        assert_eq!(row.antenna, 75);
        assert_eq!(row.tile_id, 104);
        assert_eq!(row.tile_name, "Tile104");
        assert_eq!(row.pol, Pol::Y);
        assert_eq!(row.length_string, "EL_-756.49");
        assert!(float_cmp::approx_eq!(
            f64,
            row.north_m,
            -101.529_998_779_296_88 as f64,
            float_cmp::F64Margin::default()
        ));
        assert!(float_cmp::approx_eq!(
            f64,
            row.east_m,
            -585.674_987_792_968_8 as f64,
            float_cmp::F64Margin::default()
        ));
        assert!(float_cmp::approx_eq!(
            f64,
            row.height_m,
            375.212_005_615_234_4 as f64,
            float_cmp::F64Margin::default()
        ));
        assert_eq!(row.flag, 1);
        assert_eq!(row.rx, 10);
        assert_eq!(row.slot, 4);
    }

    #[test]
    fn test_read_metafits_values_from_invalid_metafits() {
        let metafits_filename = "read_metafits_values_from_invalid_metafits.metafits";

        crate::misc::with_new_temp_fits_file(&metafits_filename, |metafits_fptr| {
            // Create a tiledata hdu
            let first_description = ColumnDescription::new("A")
                .with_type(ColumnDataType::Int)
                .create()
                .unwrap();
            let second_description = ColumnDescription::new("B")
                .with_type(ColumnDataType::Long)
                .create()
                .unwrap();
            let descriptions = [first_description, second_description];

            metafits_fptr
                .create_table("TILEDATA".to_string(), &descriptions)
                .unwrap();

            let metafits_tile_table_hdu = fits_open_hdu!(metafits_fptr, 1).unwrap();

            // Get values from row 1
            let metafits_result =
                RFInput::read_metafits_values(metafits_fptr, &metafits_tile_table_hdu, 0);

            assert_eq!(metafits_result.is_err(), true);
        });
    }

    #[test]
    fn test_populate_rf_inputs() {
        /* populate_rf_inputs(
            num_inputs: usize,
            metafits_fptr: &mut fitsio::FitsFile,
            metafits_tile_table_hdu: fitsio::hdu::FitsHdu,
            coax_v_factor: f64,
        ) -> Result<Vec<Self>, RfinputError>*/
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
        let mut metafits_fptr = fits_open!(&metafits_filename).unwrap();
        let metafits_tile_table_hdu = fits_open_hdu!(&mut metafits_fptr, 1).unwrap();
        let result =
            RFInput::populate_rf_inputs(256, &mut metafits_fptr, metafits_tile_table_hdu, 1.204);

        assert!(result.is_ok());

        let rfinput = result.unwrap();

        assert_eq!(rfinput[0].input, 0);
        assert_eq!(rfinput[0].antenna, 75);
        assert_eq!(rfinput[0].tile_id, 104);
        assert_eq!(rfinput[0].tile_name, "Tile104");
        assert_eq!(rfinput[0].pol, Pol::Y);
        assert_eq!(rfinput[0].electrical_length_m, -756.49);
        assert!(float_cmp::approx_eq!(
            f64,
            rfinput[0].north_m,
            -101.529_998_779_296_88 as f64,
            float_cmp::F64Margin::default()
        ));
        assert!(float_cmp::approx_eq!(
            f64,
            rfinput[0].east_m,
            -585.674_987_792_968_8 as f64,
            float_cmp::F64Margin::default()
        ));
        assert!(float_cmp::approx_eq!(
            f64,
            rfinput[0].height_m,
            375.212_005_615_234_4 as f64,
            float_cmp::F64Margin::default()
        ));
        assert_eq!(rfinput[0].flagged, true);
        assert_eq!(
            rfinput[0].gains,
            vec![
                74, 73, 73, 72, 71, 70, 68, 67, 66, 65, 65, 65, 66, 66, 65, 65, 64, 64, 64, 65, 65,
                66, 67, 68
            ]
        );
        assert_eq!(
            rfinput[0].delays,
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(rfinput[0].receiver_number, 10);
        assert_eq!(rfinput[0].receiver_slot_number, 4);
    }

    /*#[test]
    fn test_read_metafits_values_invalid_data() {
        let metafits_filename = "read_metafits_values_from_invalid_metafits.metafits";

        crate::misc::with_new_temp_fits_file(&metafits_filename, |metafits_fptr| {
            // Create a tiledata hdu
            let col_input = ColumnDescription::new("Input")
                .with_type(ColumnDataType::Int)
                .create()
                .unwrap();
            let col_antenna = ColumnDescription::new("Antenna")
                .with_type(ColumnDataType::Int)
                .create()
                .unwrap();
            let col_tile = ColumnDescription::new("Tile")
                .with_type(ColumnDataType::Int)
                .create()
                .unwrap();
            let col_tilename = ColumnDescription::new("TileName")
                .with_type(ColumnDataType::String)
                .create()
                .unwrap();
            let col_pol = ColumnDescription::new("Pol")
                .with_type(ColumnDataType::String)
                .create()
                .unwrap();
            let descriptions = [col_input, col_antenna, col_tile, col_tilename, col_pol];

            metafits_fptr
                .create_table("TILEDATA".to_string(), &descriptions)
                .unwrap();

            let metafits_tile_table_hdu = fits_open_hdu!(metafits_fptr, 1).unwrap();

            metafits_tile_table_hdu.write_col(&mut metafits_fptr, "Input", vec![0]);
            metafits_tile_table_hdu.write_col(&mut metafits_fptr, "Antenna", vec![0]);
            metafits_tile_table_hdu.write_col(&mut metafits_fptr, "Tile", &vec![0]);
            metafits_tile_table_hdu.write_col(&mut metafits_fptr, "TileName", vec!["Tile1"]);
            metafits_tile_table_hdu.write_col(&mut metafits_fptr, "Pol", vec!["Z"]); // this is invalid!

            // Get values from row 1
            let metafits_result =
                RFInput::read_metafits_values(metafits_fptr, &metafits_tile_table_hdu, 0);

            assert_eq!(metafits_result.is_err(), true);
        });
    }
    */
}
