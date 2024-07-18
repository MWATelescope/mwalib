// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Structs and helper methods for rf_input metadata

pub mod error;
use crate::misc::has_whitening_filter;
use crate::{_open_hdu, fits_open_hdu};
use error::RfinputError;
use log::trace;
use std::fmt;

#[cfg(test)]
mod test;

/// VCS_ORDER is the order that comes out of PFB and into the correlator (for legacy observations)
/// It can be calculated, so we do that, rather than make the user get a newer metafits (only metafits after mid 2018
/// have this column pre populated). The VCS_ORDER since it relates to the 256 input PFB has no value for inputs >255.
/// Therefore we simply return `input` if `input` is > 255
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
    if input < 256 {
        (input & 0xC0) | ((input & 0x30) >> 4) | ((input & 0x0F) << 2)
    } else {
        input
    }
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
    // `u32::from` converts the boolean to a number; in this case, 1 if pol is
    // Y, 0 otherwise.
    (antenna * 2) + u32::from(pol == Pol::Y)
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
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
struct RfInputMetafitsTableRow {
    /// This is the ordinal index of the rf_input in the metafits file
    input: u32,
    /// This is the antenna number.
    /// Nominally this is the field we sort by to get the desired output order of antenna.
    /// X and Y have the same antenna number. This is the sorted ordinal order of the antenna.None
    /// e.g. 0...N-1
    antenna: u32,
    /// Numeric part of tile_name for the antenna. Each pol has the same value
    /// e.g. tile_name "tile011" has a tile_id of 11
    tile_id: u32,
    /// Human readable name of the antenna
    /// X and Y have the same name
    tile_name: String,
    /// Polarisation - X or Y
    pol: Pol,
    /// Electrical length in metres for this antenna and polarisation to the receiver
    length_string: String,
    /// Antenna position North from the array centre (metres)
    north_m: f64,
    /// Antenna position East from the array centre (metres)
    east_m: f64,
    /// Antenna height from the array centre (metres)
    height_m: f64,
    /// Is this rf_input flagged out (due to tile error, etc from metafits)
    flag: i32,
    /// Digital gains
    /// Digital gains read from metafits need to be divided by 64 and stored in this vec
    digital_gains: Vec<f64>,
    /// Dipole delays
    dipole_delays: Vec<u32>,
    /// Dipole gains.
    ///
    /// These are either 1 or 0 (on or off), depending on the dipole delay; a
    /// dipole delay of 32 corresponds to "dead dipole", so the dipole gain of 0
    /// reflects that. All other dipoles are assumed to be "live". The values
    /// are made floats for easy use in beam code.
    dipole_gains: Vec<f64>,
    /// Receiver number
    rx: u32,
    /// Receiver slot number
    slot: u32,
    /// Receiver type
    rx_type: String,
    /// (cable) flavour
    flavour: String,
}

/// ReceiverType enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[allow(clippy::upper_case_acronyms)]
pub enum ReceiverType {
    Unknown,
    RRI,
    NI,
    Pseudo,
    SHAO,
    EDA2,
}

/// Implements fmt::Display for ReceiverType
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
impl fmt::Display for ReceiverType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ReceiverType::Unknown => "Unknown",
                ReceiverType::RRI => "RRI",
                ReceiverType::NI => "NI",
                ReceiverType::Pseudo => "Pseudo",
                ReceiverType::SHAO => "SHAO",
                ReceiverType::EDA2 => "EDA2",
            }
        )
    }
}

/// Implements str::FromStr for ReceiverType enum.
/// Non uppercase values are coverted to uppercase for comparision.
///
/// # Arguments
///
/// * `input` - A &str which we want to convert to an enum
///
///
/// # Returns
///
/// * `Result<ReceiverType, Err>` - Result of this method
///
///
impl std::str::FromStr for ReceiverType {
    type Err = ();

    fn from_str(input: &str) -> Result<ReceiverType, Self::Err> {
        match input.to_uppercase().as_str() {
            "RRI" => Ok(ReceiverType::RRI),
            "NI" => Ok(ReceiverType::NI),
            "PSEUDO" => Ok(ReceiverType::Pseudo),
            "SHAO" => Ok(ReceiverType::SHAO),
            "EDA2" => Ok(ReceiverType::EDA2),
            _ => Ok(ReceiverType::Unknown),
        }
    }
}

/// Structure to hold one row of the metafits calibdata table
struct RfInputMetafitsCalibDataTableRow {
    /// This is the ordinal index of the rf_input in the metafits file
    antenna: u32,
    tile: u32,
    tilename: String,
    pol: Pol,
    calib_delay: f32,
    calib_gains: Vec<f32>,
}

/// Structure for storing MWA rf_chains (tile with polarisation) information from the metafits file
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all))]
pub struct Rfinput {
    /// This is the metafits order (0-n inputs)
    pub input: u32,
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
    /// metafits digital gains will be divided by 64
    /// Digital gains are in mwalib metafits coarse channel order (ascending sky frequency order)
    pub digital_gains: Vec<f64>,
    /// Dipole gains.
    ///
    /// These are either 1 or 0 (on or off), depending on the dipole delay; a
    /// dipole delay of 32 corresponds to "dead dipole", so the dipole gain of 0
    /// reflects that. All other dipoles are assumed to be "live". The values
    /// are made floats for easy use in beam code.
    pub dipole_gains: Vec<f64>,
    /// Dipole delays
    pub dipole_delays: Vec<u32>,
    /// Receiver number
    pub rec_number: u32,
    /// Receiver slot number
    pub rec_slot_number: u32,
    /// Receiver type
    pub rec_type: ReceiverType,
    /// Flavour
    pub flavour: String,
    /// Has whitening filter (depends on flavour)
    pub has_whitening_filter: bool,
    /// Calibration delay in meters (if provided)
    pub calib_delay: Option<f32>,
    /// Calibration gains (vector- 1 per coarse channel) if provided
    pub calib_gains: Option<Vec<f32>>,
}

impl Rfinput {
    /// This method just reads a row from the metafits tiledata table and returns the values in a struct
    ///
    /// # Arguments
    ///
    /// * `metafits_fptr` - reference to the FitsFile representing the metafits file.
    ///
    /// * `metafits_tile_table_hdu` - reference to the HDU containing the TILEDATA table.
    ///
    /// * `row` - row index to read from the TILEDATA table in the metafits.
    ///
    /// * `num_coarse_chans` - the number of coarse channels in this observation.
    ///
    ///
    /// # Returns
    ///
    /// * An Result containing a populated vector of RFInputMetafitsTableRow structss or an Error
    ///
    fn read_metafits_tiledata_values(
        metafits_fptr: &mut fitsio::FitsFile,
        metafits_tile_table_hdu: &fitsio::hdu::FitsHdu,
        row: usize,
        num_coarse_chans: usize,
    ) -> Result<RfInputMetafitsTableRow, RfinputError> {
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

        // Digital gains values in metafits need to be divided by 64
        // Digital gains are in mwalib metafits coarse channel order (ascending sky frequency order)
        let digital_gains = read_cell_array_u32(
            metafits_fptr,
            metafits_tile_table_hdu,
            "Gains",
            row as i64,
            num_coarse_chans,
        )?
        .iter()
        .map(|gains| *gains as f64 / 64.0)
        .collect();

        let dipole_delays = read_cell_array_u32(
            metafits_fptr,
            metafits_tile_table_hdu,
            "Delays",
            row as i64,
            16,
        )?;
        let rx = read_cell_value(metafits_fptr, metafits_tile_table_hdu, "Rx", row)?;
        let slot = read_cell_value(metafits_fptr, metafits_tile_table_hdu, "Slot", row)?;

        let dipole_gains = dipole_delays
            .iter()
            .map(|&delay| if delay == 32 { 0.0 } else { 1.0 })
            .collect();

        // Many old metafits will not have this column. So
        // if it does not exist, don't panic, just return
        // en empty string. The enum value will end up
        // being set to ReceiverType::Unknown
        let rx_type: String = read_cell_value(
            metafits_fptr,
            metafits_tile_table_hdu,
            "Receiver_Types",
            row,
        )
        .unwrap_or_default();

        // Many old metafits will not have this column. So
        // if it does not exist, don't panic, just return
        // en empty string.
        let flavour: String =
            read_cell_value(metafits_fptr, metafits_tile_table_hdu, "Flavors", row)
                .unwrap_or_default();

        Ok(RfInputMetafitsTableRow {
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
            digital_gains,
            dipole_delays,
            dipole_gains,
            rx,
            slot,
            rx_type,
            flavour,
        })
    }

    /// This method just reads a row from the metafits calibdata table and returns the values in a struct
    ///
    /// # Arguments
    ///
    /// * `metafits_fptr` - reference to the FitsFile representing the metafits file.
    ///
    /// * `metafits_calibdata_table_hdu` - reference to the HDU containing the CALIBDATA table.
    ///
    /// * `row` - row index to read from the CALIBDATA table in the metafits.
    ///
    /// * `num_coarse_chans` - the number of coarse channels in this observation.
    ///
    ///
    /// # Returns
    ///
    /// * An Result containing a populated vector of RFInputMetafitsTableRow structss or an Error
    ///
    fn read_metafits_calibdata_values(
        metafits_fptr: &mut fitsio::FitsFile,
        metafits_calibdata_table_hdu: &Option<fitsio::hdu::FitsHdu>,
        row: usize,
        num_coarse_chans: usize,
    ) -> Result<Option<RfInputMetafitsCalibDataTableRow>, RfinputError> {
        if let Some(metafits_cal_hdu) = metafits_calibdata_table_hdu {
            let antenna = read_cell_value(metafits_fptr, metafits_cal_hdu, "Antenna", row)?;
            let tile_id = read_cell_value(metafits_fptr, metafits_cal_hdu, "Tile", row)?;
            let tile_name = read_cell_value(metafits_fptr, metafits_cal_hdu, "TileName", row)?;
            let pol = {
                let p: String = read_cell_value(metafits_fptr, metafits_cal_hdu, "Pol", row)?;
                match p.as_str() {
                    "X" => Pol::X,
                    "Y" => Pol::Y,
                    _ => {
                        return Err(RfinputError::UnrecognisedPol {
                            fits_filename: metafits_fptr.filename.clone(),
                            hdu_num: metafits_cal_hdu.number + 1,
                            row_num: row,
                            got: p,
                        })
                    }
                }
            };

            let calib_delay_m: f32 =
                read_cell_value(metafits_fptr, metafits_cal_hdu, "Calib_Delay", row)?;

            // calib gains are in mwalib metafits coarse channel order (ascending sky frequency order)
            let calib_gains: Vec<f32> = read_cell_array_f32(
                metafits_fptr,
                metafits_cal_hdu,
                "Calib_Gains",
                row as i64,
                num_coarse_chans,
            )?;

            Ok(Some(RfInputMetafitsCalibDataTableRow {
                antenna,
                tile: tile_id,
                tilename: tile_name,
                pol,
                calib_delay: calib_delay_m,
                calib_gains,
            }))
        } else {
            Ok(None)
        }
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
    pub(crate) fn populate_rf_inputs(
        num_inputs: usize,
        metafits_fptr: &mut fitsio::FitsFile,
        metafits_tile_table_hdu: fitsio::hdu::FitsHdu,
        coax_v_factor: f64,
        num_coarse_chans: usize,
    ) -> Result<Vec<Self>, RfinputError> {
        let mut rf_inputs: Vec<Self> = Vec::with_capacity(num_inputs);

        // The calibration HDU is not always present, so we will check the result when we need to use it.
        // is_ok() means it exists, is_err() = True means it does not.
        let metafits_cal_hdu_result = fits_open_hdu!(metafits_fptr, 2);

        let metafits_cal_hdu_option = match metafits_cal_hdu_result {
            Ok(m) => Some(m),
            Err(_) => None,
        };

        for input in 0..num_inputs {
            // Note fits row numbers start at 1
            let metafits_row = Self::read_metafits_tiledata_values(
                metafits_fptr,
                &metafits_tile_table_hdu,
                input,
                num_coarse_chans,
            )?;

            // The metafits TILEDATA table contains 2 rows for each antenna.
            // Some metafits will have
            // EL_nnn => this is the electrical length. Just use it (chop off the EL)
            // or
            // nnn    => this needs to be multiplied by the v_factor to get the eqiuvalent EL
            let electrical_length_m =
                get_electrical_length(metafits_row.length_string, coax_v_factor);

            let vcs_order = get_vcs_order(metafits_row.input);
            let subfile_order = get_mwax_order(metafits_row.antenna, metafits_row.pol);

            let has_whitening_filter: bool = has_whitening_filter(&metafits_row.flavour);

            // Get data from calibration hdu if it exists
            let calibdata_row = Self::read_metafits_calibdata_values(
                metafits_fptr,
                &metafits_cal_hdu_option,
                input,
                num_coarse_chans,
            )?;

            let calib_delay: Option<f32>;
            let calib_gains: Option<Vec<f32>>;
            if calibdata_row.is_some() {
                let calibdata_row = calibdata_row.unwrap();

                // Check to ensure we have the same order of rf_inputs between this rf_input and the calibration hdu
                assert_eq!(calibdata_row.antenna, metafits_row.antenna);
                assert_eq!(calibdata_row.tile, metafits_row.tile_id);
                assert_eq!(calibdata_row.tilename, metafits_row.tile_name);
                assert_eq!(calibdata_row.pol, metafits_row.pol);

                // Grab the delays and gains
                calib_delay = Some(calibdata_row.calib_delay);
                calib_gains = Some(calibdata_row.calib_gains);
            } else {
                calib_delay = None;
                calib_gains = None;
            }

            rf_inputs.push(Self {
                input: metafits_row.input,
                ant: metafits_row.antenna,
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
                digital_gains: metafits_row.digital_gains,
                dipole_gains: metafits_row.dipole_gains,
                dipole_delays: metafits_row.dipole_delays,
                rec_number: metafits_row.rx,
                rec_slot_number: metafits_row.slot,
                rec_type: metafits_row.rx_type.parse::<ReceiverType>().unwrap(),
                flavour: metafits_row.flavour,
                has_whitening_filter,
                calib_delay,
                calib_gains,
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
        Ok(c) => {
            trace!(
                "read_cell_value() filename: '{}' hdu: {} col_name: '{}' row '{}'",
                metafits_fptr.filename.display(),
                metafits_tile_table_hdu.number,
                col_name,
                row
            );
            Ok(c)
        }
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
fn read_cell_array_u32(
    metafits_fptr: &mut fitsio::FitsFile,
    metafits_tile_table_hdu: &fitsio::hdu::FitsHdu,
    col_name: &str,
    row: i64,
    n_elem: usize,
) -> Result<Vec<u32>, RfinputError> {
    unsafe {
        // With the column name, get the column number.
        let mut status = 0;
        let mut col_num = -1;
        let keyword = std::ffi::CString::new(col_name).unwrap().into_raw();
        fitsio_sys::ffgcno(
            metafits_fptr.as_raw(),
            0,
            keyword,
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
        drop(std::ffi::CString::from_raw(keyword));

        // Now get the specified row from that column.
        // cfitsio is stupid. The data we want fits in i16, but we're forced to
        // unpack it into i32. Demote the data at the end.
        let mut array: Vec<u32> = vec![0; n_elem];
        array.shrink_to_fit();
        let array_ptr = array.as_mut_ptr();
        fitsio_sys::ffgcv(
            metafits_fptr.as_raw(),
            31,
            col_num,
            row + 1,
            1,
            n_elem as i64,
            std::ptr::null_mut(),
            array_ptr as *mut core::ffi::c_void,
            &mut 0,
            &mut status,
        );

        // Check the status.
        match status {
            0 => {
                // Re-assemble the raw array into a Rust Vector.
                let v = std::slice::from_raw_parts(array_ptr, n_elem);

                trace!(
                    "read_cell_array() filename: '{}' hdu: {} col_name: '{}' row '{}'",
                    metafits_fptr.filename.display(),
                    metafits_tile_table_hdu.number,
                    col_name,
                    row
                );

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

/// Pull out the array-in-a-cell values. This function assumes that the output
/// datatype is f32, and that the fits datatype is E (f32), so it is not to be used
/// generally!
fn read_cell_array_f32(
    metafits_fptr: &mut fitsio::FitsFile,
    metafits_tile_table_hdu: &fitsio::hdu::FitsHdu,
    col_name: &str,
    row: i64,
    n_elem: usize,
) -> Result<Vec<f32>, RfinputError> {
    unsafe {
        // With the column name, get the column number.
        let mut status = 0;
        let mut col_num = -1;
        let keyword = std::ffi::CString::new(col_name).unwrap().into_raw();
        fitsio_sys::ffgcno(
            metafits_fptr.as_raw(),
            0,
            keyword,
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
        drop(std::ffi::CString::from_raw(keyword));

        // Now get the specified row from that column.
        // cfitsio is stupid. The data we want fits in i16, but we're forced to
        // unpack it into i32. Demote the data at the end.
        let mut array: Vec<f32> = vec![0.0; n_elem];
        array.shrink_to_fit();
        let array_ptr = array.as_mut_ptr();
        fitsio_sys::ffgcv(
            metafits_fptr.as_raw(),
            42,
            col_num,
            row + 1,
            1,
            n_elem as i64,
            std::ptr::null_mut(),
            array_ptr as *mut core::ffi::c_void,
            &mut 0,
            &mut status,
        );

        // Check the status.
        match status {
            0 => {
                // Re-assemble the raw array into a Rust Vector.
                let v = std::slice::from_raw_parts(array_ptr, n_elem);

                trace!(
                    "read_cell_array() filename: '{}' hdu: {} col_name: '{}' row '{}'",
                    metafits_fptr.filename.display(),
                    metafits_tile_table_hdu.number,
                    col_name,
                    row
                );

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
impl fmt::Debug for Rfinput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.tile_name, self.pol)
    }
}
