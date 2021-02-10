// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Functions for organising and checking the consistency of voltage files.
*/

pub mod error;

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use regex::Regex;

use crate::*;
pub use error::VoltageFileError;

#[derive(Debug)]
pub struct ObsTimes {
    pub start_gps_time_milliseconds: u64, // Start= start of first timestep
    pub end_gps_time_milliseconds: u64,   // End  = start of last timestep + interval time
    pub duration_milliseconds: u64,
    pub voltage_file_interval_milliseconds: u64, // number of milliseconds between each voltage file
}

/// This represents one group of voltage files with the same "batch" identitifer (gps time).
/// e.g.
/// MWA Legacy: obsid_gpstime_datetime_chan
/// MWAX      : obsid_gpstime_datetime_chan
pub struct VoltageFileBatch {
    pub gps_time: usize,                 // 1234567890
    pub voltage_files: Vec<VoltageFile>, // Vector storing the details of each voltage file in this batch
}

impl VoltageFileBatch {
    pub fn new(gps_time: usize) -> Self {
        Self {
            gps_time,
            voltage_files: vec![],
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl fmt::Debug for VoltageFileBatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "gps_time={} voltage_files={:?}",
            self.gps_time, self.voltage_files,
        )
    }
}

/// This represents one voltage file
pub struct VoltageFile {
    /// Filename of voltage file
    pub filename: String,
    /// channel number (receiver channel number 001..255)
    pub channel_identifier: usize,
}

#[cfg(not(tarpaulin_include))]
impl fmt::Debug for VoltageFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "filename={} channelidentifier={}",
            self.filename, self.channel_identifier,
        )
    }
}

impl std::cmp::PartialEq for VoltageFileBatch {
    fn eq(&self, other: &Self) -> bool {
        self.gps_time == other.gps_time && self.voltage_files == other.voltage_files
    }
}

impl std::cmp::PartialEq for VoltageFile {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename && self.channel_identifier == other.channel_identifier
    }
}

/// A temporary representation of a voltage file
#[derive(Clone, Debug)]
struct TempVoltageFile<'a> {
    /// Filename of gpubox file
    filename: &'a str,
    /// obsid
    obs_id: usize,
    /// Channel number (Legacy==gpubox host number 01..24; V2==receiver channel number 001..255)
    channel_identifier: usize,
    /// GPS time (aka Batch number)
    gps_time: usize,
}

impl<'a> std::cmp::PartialEq for TempVoltageFile<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
            && self.obs_id == other.obs_id
            && self.channel_identifier == other.channel_identifier
            && self.gps_time == other.gps_time
    }
}

lazy_static::lazy_static! {
    // 1234567890_1234567890_123.sub
    // obsid        subobsid  chan
    static ref RE_MWAX_VCS: Regex =
        Regex::new(r"(?P<obs_id>\d{10})_(?P<gpstime>\d{10})_(?P<channel>\d{1,3})\.sub").unwrap();
    // 1234567890_1234567890_123.dat
    // obsid        subobsid  chan
    static ref RE_LEGACY_VCS_RECOMBINED: Regex =
        Regex::new(r"(?P<obs_id>\d{10})_(?P<gpstime>\d{10})_ch(?P<channel>\d{1,3})\.dat").unwrap();
}

/// Convert `Vec<TempVoltageFile>` to `Vec<VoltageFileBatch>`. This requires the voltage
/// files to actually be present.
///
/// Fail if
///
/// * no files were supplied;
/// * the files specified by the `TempVoltageFile`s can't be opened.
///
///
/// # Arguments
///
/// * `temp_voltage_files` - A vector of `TempVoltageFile` to be converted.
///
///
/// # Returns
///
/// * A Result containing a vector of `VoltageFileBatch`.
///
///
fn convert_temp_voltage_files(
    temp_voltage_files: Vec<TempVoltageFile>,
) -> Result<Vec<VoltageFileBatch>, VoltageFileError> {
    // unwrap is safe as a check is performed above to ensure that there are
    // some files present.
    let num_batches = temp_voltage_files.iter().map(|g| g.gps_time).max().unwrap() + 1;
    let mut voltage_file_batches: Vec<VoltageFileBatch> = Vec::with_capacity(num_batches);
    for b in 0..num_batches {
        voltage_file_batches.push(VoltageFileBatch::new(b));
    }

    for temp_v in temp_voltage_files.into_iter() {
        let v = VoltageFile {
            filename: temp_v.filename.to_string(),
            channel_identifier: temp_v.channel_identifier,
        };
        voltage_file_batches[temp_v.gps_time].voltage_files.push(v);
    }

    // Ensure the output is properly sorted - each batch is sorted by
    // channel_identifier.
    for v in &mut voltage_file_batches {
        v.voltage_files
            .sort_unstable_by(|a, b| a.channel_identifier.cmp(&b.channel_identifier));
    }

    // Sort the batches by batch number
    voltage_file_batches.sort_by_key(|b| b.gps_time);

    Ok(voltage_file_batches)
}

/// A type alias for a horrible type:
/// `BTreeMap<u64, BTreeMap<usize, (usize, usize)>>`
///
/// The outer-most keys are GPS seconds, which correspond to the
/// unique times in supplied voltage files. Each of these
/// keys is associated with a tree; the keys of these trees are the
/// coarse-channel numbers, which then refer to the filename.
pub type VoltageFileTimeMap = BTreeMap<usize, BTreeMap<usize, String>>;

/// A little struct to help us not get confused when dealing with the returned
/// values from complex functions.
pub struct VoltageFileInfo {
    pub gpstime_batches: Vec<VoltageFileBatch>,
    pub corr_format: CorrelatorVersion,
    pub time_map: VoltageFileTimeMap,
    pub file_size: u64,
}

/// Group input voltage files into gpstime_batches. A "voltage batch" refers to the sub_obs_id
/// in a voltage filename (second 10 digit number in filename).
/// (e.g. `1065880128_XXXXXXXXXX_123.sub`). Older / Legacy VCS files
/// have a similar format (e.g. `1065880128_XXXXXXXXXX_ch123.dat`).
///
///
/// Fail if
///
/// * no files were supplied;
/// * there is a mixture of the types of voltage files supplied (e.g. different correlator
///   versions);
/// * a voltage filename's structure could not be identified;
/// * the gpstime batch numbers are not contiguous;
/// * the number of files in each gpstime batch is not equal;
///
///
/// # Arguments
///
/// * `voltage_filenames` - A vector or slice of strings or references to strings containing
///                        all of the voltage filenames provided by the client.
///
/// * `metafits_obs_id` - The obs_id of the observation from the metafits.
///
///
/// # Returns
///
/// * A Result containing a vector of `TempVoltageFile` structs as well as a
///   `CorrelatorVersion`, the number of voltage files supplied, and the number of
///   gps time batches.
///
///
fn determine_voltage_file_gpstime_batches<T: AsRef<Path>>(
    voltage_filenames: &[T],
    metafits_obs_id: usize,
) -> Result<(Vec<TempVoltageFile>, CorrelatorVersion, usize), VoltageFileError> {
    if voltage_filenames.is_empty() {
        return Err(VoltageFileError::NoVoltageFiles);
    }
    let mut format = None;
    let mut temp_voltage_files: Vec<TempVoltageFile> = Vec::with_capacity(voltage_filenames.len());

    for v_path in voltage_filenames {
        // So that we can pass along useful error messages, convert the input
        // filename type to a string slice. This will fail if the filename is
        // not UTF-8 compliant.
        let v = v_path
            .as_ref()
            .to_str()
            .expect("Voltage filename is not UTF-8 compliant");

        let new_temp_voltage_file: TempVoltageFile = {
            match RE_MWAX_VCS.captures(v) {
                Some(caps) => {
                    // Check if we've already matched any files as being the old
                    // format. If so, then we've got a mix, and we should exit
                    // early.
                    match format {
                        None => format = Some(CorrelatorVersion::V2),
                        Some(CorrelatorVersion::V2) => (),
                        _ => return Err(VoltageFileError::Mixture),
                    }

                    // The following unwraps are safe, because the regex wouldn't
                    // work if they couldn't be parsed into ints.
                    TempVoltageFile {
                        filename: v,
                        obs_id: caps["obs_id"].parse().unwrap(),
                        channel_identifier: caps["channel"].parse().unwrap(),
                        gps_time: caps["gpstime"].parse().unwrap(),
                    }
                }

                // Try to match the legacy format.
                None => match RE_LEGACY_VCS_RECOMBINED.captures(v) {
                    Some(caps) => {
                        match format {
                            None => format = Some(CorrelatorVersion::Legacy),
                            Some(CorrelatorVersion::Legacy) => (),
                            _ => return Err(VoltageFileError::Mixture),
                        }

                        TempVoltageFile {
                            filename: v,
                            obs_id: caps["obs_id"].parse().unwrap(),
                            channel_identifier: caps["channel"].parse().unwrap(),
                            gps_time: caps["gpstime"].parse().unwrap(),
                        }
                    }
                    None => return Err(VoltageFileError::Unrecognised(v.to_string())),
                },
            }
        };

        // Does this file have the same obs_id in the filename as we have in the metafits?
        if new_temp_voltage_file.obs_id == metafits_obs_id {
            temp_voltage_files.push(new_temp_voltage_file);
        } else {
            return Err(VoltageFileError::MetafitsObsidMismatch);
        }
    }

    // Check batches are contiguous and have equal numbers of files.
    let mut batches_and_files: BTreeMap<usize, u8> = BTreeMap::new();
    for voltage_file in &temp_voltage_files {
        *batches_and_files.entry(voltage_file.gps_time).or_insert(0) += 1;
    }

    let mut file_count: Option<u8> = None;
    let mut prev_batch_num: usize = 0;
    for (_, (batch_num, num_files)) in batches_and_files.iter().enumerate() {
        // Check that the previous batch + 1 == the current batch number
        // This is our contiguity check
        if prev_batch_num != 0 {
            if prev_batch_num + 1 != *batch_num {
                return Err(VoltageFileError::GpsTimeMissing {
                    expected: prev_batch_num + 1,
                    got: *batch_num,
                });
            }
        }
        prev_batch_num = *batch_num;

        match file_count {
            None => file_count = Some(*num_files),
            Some(c) => {
                if c != *num_files {
                    return Err(VoltageFileError::UnevenChannelsForGpsTime {
                        expected: c,
                        got: *num_files,
                    });
                }
            }
        }
    }

    // Ensure the output is properly sorted - each batch is sorted by batch
    // number, then channel identifier.
    temp_voltage_files.sort_unstable_by_key(|v| (v.gps_time, v.channel_identifier));

    Ok((temp_voltage_files, format.unwrap(), batches_and_files.len()))
}

/// This function unpacks the metadata associated with input voltage files. The
/// input filenames are grouped into into gps time batches. A "gpstime batch" refers to
/// the sub_obs_id in a voltage filename
/// (e.g. `1065880128_XXXXXXXXXX_123.sub`). Some older files might
/// have a different format
/// (e.g. `1065880128_XXXXXXXXXX_ch123.dat`). These details are
/// reflected in the returned `CorrelatorVersion`.
///
/// Fail if
///
/// * no files were supplied;
/// * there is a mixture of the types of voltage files supplied (e.g. different
///   correlator versions);
/// * a voltage filename's structure could not be identified;
/// * the volatge gpstimes are not contiguous;
/// * the number of files in each batch of gpstimes is not equal;
/// * the amount of data in each file is not equal.
///
///
/// # Arguments
///
/// * `metafits_context`  - A reference to a populated metafits context we can use to verify voltage file metadata against.
///
/// * `voltage_filenames` - A vector or slice of strings or references to strings
///                         containing all of the voltage filenames provided by the client.
///
///
/// # Returns
///
/// * A Result containing a vector of VoltageBatch structs, the MWA Correlator
///   version, the GPS times paired with filenames, and the amount of
///   data in each HDU.
///
///
pub fn examine_voltage_files<T: AsRef<Path>>(
    metafits_context: &MetafitsContext,
    voltage_filenames: &[T],
) -> Result<VoltageFileInfo, VoltageFileError> {
    let (temp_voltage_files, corr_format, _) =
        determine_voltage_file_gpstime_batches(voltage_filenames, metafits_context.obsid as usize)?;

    let time_map = create_time_map(&temp_voltage_files)?;

    let mut gpstime_batches = convert_temp_voltage_files(temp_voltage_files)?;

    // Determine the size of each voltage file. mwalib will throw an
    // error if this size is not consistent for all voltage files.
    let mut voltage_file_size: Option<u64> = None;
    for b in &mut gpstime_batches {
        for v in &mut b.voltage_files {
            let this_size = std::fs::metadata(&v.filename).unwrap().len();
            match voltage_file_size {
                None => voltage_file_size = Some(this_size),
                Some(s) => {
                    if s != this_size {
                        return Err(VoltageFileError::UnequalFileSizes);
                    }
                }
            }
        }
    }

    // `determine_voltage_file_batches` fails if no voltage files are supplied, so it
    // is safe to unwrap voltage_file_size.
    Ok(VoltageFileInfo {
        gpstime_batches,
        corr_format,
        time_map,
        file_size: voltage_file_size.unwrap(),
    })
}

/// Returns a BTree structure consisting of:
/// BTree of timesteps. Each timestep is a BTree for a course channel.
/// Each coarse channel then contains the filenames of voltage files.
///
/// # Arguments
///
/// * `voltage_file_batches` - vector of structs describing each voltage "batch" of gpstimes
///
/// * `correlator_version` - enum telling us which correlator version the observation was created by.
///
///
/// # Returns
///
/// * A Result containing the Voltage File Time Map or an error.
///
///
fn create_time_map(
    voltage_file_batches: &[TempVoltageFile],
) -> Result<VoltageFileTimeMap, VoltageFileError> {
    // create a map
    let mut voltage_time_map = BTreeMap::new();
    for voltage_file in voltage_file_batches.iter() {
        voltage_time_map
            .entry(voltage_file.gps_time)
            .or_insert_with(BTreeMap::new)
            .entry(voltage_file.channel_identifier)
            .or_insert(voltage_file.filename.to_string());
    }

    Ok(voltage_time_map)
}

/// Determine the proper start and end times of an observation. In this context,
/// "proper" refers to a time that is common to all voltage files. Depending on
/// the correlator version, the last timestep is incremented by either 1 or 8 seconds.
///
///
/// # Arguments
///
/// * `voltage_time_map` - BTree structure containing the map of what voltage files and timesteps we were supplied by the client.
///
/// * `correlator_version` - Correlator dump time (so we know the gap between timesteps)
///
/// # Returns
///
/// * A struct containing the start and end times based on what we actually got, so all coarse channels match.
///
///
pub fn determine_obs_times(
    voltage_time_map: &VoltageFileTimeMap,
    correlator_version: CorrelatorVersion,
) -> Result<ObsTimes, VoltageFileError> {
    let voltage_file_interval_milliseconds: u64 = match correlator_version {
        CorrelatorVersion::V2 => 8_000,
        CorrelatorVersion::Legacy => 1_000,
        CorrelatorVersion::OldLegacy => 1_000,
    };

    // Find the maximum number of voltage files for a timestep, and assume that this is the
    // total number of input voltage files.
    let size = match voltage_time_map
        .iter()
        .map(|(_, submap)| submap.len())
        .max()
    {
        Some(m) => m,
        None => return Err(VoltageFileError::EmptyBTreeMap),
    };

    // Filter the first elements that don't satisfy `submap.len() == size`. The
    // first and last of the submaps that satisfy this condition are the proper
    // start and end of the observation.
    let mut i = voltage_time_map
        .iter()
        .filter(|(_, submap)| submap.len() == size);

    //let proper_start_milliseconds: u64 = *voltage_time_map.iter().next().unwrap().0 as u64;
    //let proper_end_milliseconds: u64 = *voltage_time_map.iter().next_back().unwrap().0 as u64
    //    + timestep_interval_milliseconds as u64;
    let proper_start_gps_time: u64 = *i.next().unwrap().0 as u64;
    let proper_end_gps_time: u64 = match i.last() {
        Some(s) => *s.0 as u64,
        None => {
            // Looks like we only have 1 hdu, so end
            proper_start_gps_time
        }
    };

    Ok(ObsTimes {
        start_gps_time_milliseconds: proper_start_gps_time * 1000,
        end_gps_time_milliseconds: (proper_end_gps_time * 1000)
            + voltage_file_interval_milliseconds,
        duration_milliseconds: ((proper_end_gps_time - proper_start_gps_time) * 1000)
            + voltage_file_interval_milliseconds,
        voltage_file_interval_milliseconds,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_voltage_file_unrecognised_files() {
        let files = vec![
            "1065880128_106588012_ch123.dat",
            "106588012_1065880129_ch121.dat",
            "1065880128_1065880128__ch121.dat",
            "1065880128_1065880130_ch124.txt",
        ];
        let result = determine_voltage_file_gpstime_batches(&files, 1065880128);

        assert!(matches!(
            result.unwrap_err(),
            VoltageFileError::Unrecognised(_)
        ));
    }

    #[test]
    fn test_determine_voltage_file_gpstime_batches_proper_legacy_format() {
        let files = vec![
            "1065880128_1065880129_ch122.dat",
            "1065880128_1065880129_ch21.dat",
            "1065880128_1065880129_ch1.dat",
            "1065880128_1065880128_ch21.dat",
            "1065880128_1065880128_ch001.dat",
            "1065880128_1065880128_ch122.dat",
            "1065880128_1065880130_ch122.dat",
            "1065880128_1065880130_ch021.dat",
            "1065880128_1065880130_ch01.dat",
        ];
        let result = determine_voltage_file_gpstime_batches(&files, 1065880128);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
        let (temp_voltage_files, corr_format, num_gputimes) = result.unwrap();
        assert_eq!(corr_format, CorrelatorVersion::Legacy);
        assert_eq!(num_gputimes, 3);

        let expected_voltage_files = vec![
            TempVoltageFile {
                filename: "1065880128_1065880128_ch001.dat",
                obs_id: 1065880128,
                gps_time: 1065880128,
                channel_identifier: 1,
            },
            TempVoltageFile {
                filename: "1065880128_1065880128_ch21.dat",
                obs_id: 1065880128,
                gps_time: 1065880128,
                channel_identifier: 21,
            },
            TempVoltageFile {
                filename: "1065880128_1065880128_ch122.dat",
                obs_id: 1065880128,
                gps_time: 1065880128,
                channel_identifier: 122,
            },
            TempVoltageFile {
                filename: "1065880128_1065880129_ch1.dat",
                obs_id: 1065880128,
                gps_time: 1065880129,
                channel_identifier: 1,
            },
            TempVoltageFile {
                filename: "1065880128_1065880129_ch21.dat",
                obs_id: 1065880128,
                gps_time: 1065880129,
                channel_identifier: 21,
            },
            TempVoltageFile {
                filename: "1065880128_1065880129_ch122.dat",
                obs_id: 1065880128,
                gps_time: 1065880129,
                channel_identifier: 122,
            },
            TempVoltageFile {
                filename: "1065880128_1065880130_ch01.dat",
                obs_id: 1065880128,
                gps_time: 1065880130,
                channel_identifier: 1,
            },
            TempVoltageFile {
                filename: "1065880128_1065880130_ch021.dat",
                obs_id: 1065880128,
                gps_time: 1065880130,
                channel_identifier: 21,
            },
            TempVoltageFile {
                filename: "1065880128_1065880130_ch122.dat",
                obs_id: 1065880128,
                gps_time: 1065880130,
                channel_identifier: 122,
            },
        ];

        assert_eq!(temp_voltage_files, expected_voltage_files);
    }

    #[test]
    fn test_determine_voltage_file_gpstime_batches_proper_mwax_format() {
        let files = vec![
            "1065880128_1065880129_122.sub",
            "1065880128_1065880129_21.sub",
            "1065880128_1065880129_1.sub",
            "1065880128_1065880128_21.sub",
            "1065880128_1065880128_001.sub",
            "1065880128_1065880128_122.sub",
            "1065880128_1065880130_122.sub",
            "1065880128_1065880130_021.sub",
            "1065880128_1065880130_01.sub",
        ];
        let result = determine_voltage_file_gpstime_batches(&files, 1065880128);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
        let (temp_voltage_files, corr_format, num_gputimes) = result.unwrap();
        assert_eq!(corr_format, CorrelatorVersion::V2);
        assert_eq!(num_gputimes, 3);

        let expected_voltage_files = vec![
            TempVoltageFile {
                filename: "1065880128_1065880128_001.sub",
                obs_id: 1065880128,
                gps_time: 1065880128,
                channel_identifier: 1,
            },
            TempVoltageFile {
                filename: "1065880128_1065880128_21.sub",
                obs_id: 1065880128,
                gps_time: 1065880128,
                channel_identifier: 21,
            },
            TempVoltageFile {
                filename: "1065880128_1065880128_122.sub",
                obs_id: 1065880128,
                gps_time: 1065880128,
                channel_identifier: 122,
            },
            TempVoltageFile {
                filename: "1065880128_1065880129_1.sub",
                obs_id: 1065880128,
                gps_time: 1065880129,
                channel_identifier: 1,
            },
            TempVoltageFile {
                filename: "1065880128_1065880129_21.sub",
                obs_id: 1065880128,
                gps_time: 1065880129,
                channel_identifier: 21,
            },
            TempVoltageFile {
                filename: "1065880128_1065880129_122.sub",
                obs_id: 1065880128,
                gps_time: 1065880129,
                channel_identifier: 122,
            },
            TempVoltageFile {
                filename: "1065880128_1065880130_01.sub",
                obs_id: 1065880128,
                gps_time: 1065880130,
                channel_identifier: 1,
            },
            TempVoltageFile {
                filename: "1065880128_1065880130_021.sub",
                obs_id: 1065880128,
                gps_time: 1065880130,
                channel_identifier: 21,
            },
            TempVoltageFile {
                filename: "1065880128_1065880130_122.sub",
                obs_id: 1065880128,
                gps_time: 1065880130,
                channel_identifier: 122,
            },
        ];

        assert_eq!(temp_voltage_files, expected_voltage_files);
    }

    #[test]
    fn test_determine_voltage_file_gpstime_batches_channel_mismatch() {
        let files = vec![
            "1065880128_1065880129_ch123.dat",
            "1065880128_1065880129_ch121.dat",
            "1065880128_1065880128_ch121.dat",
            "1065880128_1065880130_ch124.dat",
        ];
        let result = determine_voltage_file_gpstime_batches(&files, 1065880128);
        assert!(matches!(
            result.unwrap_err(),
            VoltageFileError::UnevenChannelsForGpsTime {
                expected: _,
                got: _
            }
        ));
    }

    #[test]
    fn test_determine_voltage_file_correlator_version_mismatch() {
        let files = vec![
            "1065880128_1065880129_ch123.dat",
            "1065880128_1065880129_121.sub",
            "1065880128_1065880128_ch121.dat",
            "1065880128_1065880130_ch124.dat",
        ];
        let result = determine_voltage_file_gpstime_batches(&files, 1065880128);
        assert!(matches!(result.unwrap_err(), VoltageFileError::Mixture));
    }

    #[test]
    fn test_determine_voltage_file_metafits_obs_id_mismatch() {
        let files = vec![
            "1065880128_1065880128_ch121.dat",
            "1065880128_1065880129_ch121.dat",
            "1065880128_1065880130_ch121.dat",
        ];
        let result = determine_voltage_file_gpstime_batches(&files, 1234567890);
        assert!(matches!(
            result.unwrap_err(),
            VoltageFileError::MetafitsObsidMismatch
        ));
    }

    #[test]
    fn test_determine_voltage_file_gpstime_missing() {
        let files = vec![
            "1065880128_1065880128_ch121.dat",
            "1065880128_1065880130_ch121.dat",
        ];
        let result = determine_voltage_file_gpstime_batches(&files, 1065880128);
        assert!(matches!(
            result.unwrap_err(),
            VoltageFileError::GpsTimeMissing { expected: _, got: _ }
        ));
    }

    #[test]
    fn test_determine_obs_times_test_many_timesteps_legacy() {
        let common_times: Vec<u64> =
            vec![1065880129, 1065880130, 1065880131, 1065880132, 1065880133];
        let mut input = VoltageFileTimeMap::new();
        // insert a "dangling time" at the beginning (1065880128) which is not a common timestep
        let mut new_time_tree = BTreeMap::new();
        new_time_tree.insert(121, String::from("1065880128_1065880128_ch121.dat"));
        input.insert(1065880128, new_time_tree);

        // Add the common times to the data structure
        for time in common_times.iter() {
            let mut new_time_tree = BTreeMap::new();
            new_time_tree.insert(121, format!("1065880128_{}_ch121.dat", time));
            input.insert(*time as usize, new_time_tree);

            let mut new_time_tree2 = BTreeMap::new();
            new_time_tree2.insert(122, format!("1065880128_{}_ch122.dat", time));
            input.insert(*time as usize, new_time_tree2);
        }

        // insert a "dangling time" at the end (1065880134) which is not a common timestep
        new_time_tree = BTreeMap::new();
        new_time_tree.insert(121, String::from("1065880128_1065880134_ch121.dat"));
        input.insert(1065880134, new_time_tree);

        let expected_interval: u64 = 1000; // 1000 since we are Legacy
        let expected_start: u64 = *common_times.first().unwrap() * 1000;
        let expected_end: u64 = (*common_times.last().unwrap() + expected_interval) * 1000;
        // Duration = common end - common start + integration time
        // == 1065880133 - 1065880129 + 1000
        let expected_duration = 5000;

        let result = determine_obs_times(&input, CorrelatorVersion::Legacy);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(
            result.start_gps_time_milliseconds, expected_start,
            "{:?}",
            result
        );
        assert_eq!(
            result.end_gps_time_milliseconds, expected_end,
            "{:?}",
            result
        );
        assert_eq!(
            result.duration_milliseconds, expected_duration,
            "{:?}",
            result
        );
        assert_eq!(
            result.voltage_file_interval_milliseconds, expected_interval,
            "{:?}",
            result
        );
    }
}
/*
    #[test]
    fn test_determine_obs_times_test_one_timestep() {
        // Create two files, with 1 overlapping times, but also a little
        // dangling at the start and end.
        let common_times: Vec<u64> = vec![1_381_844_923_500];
        let integration_time_ms = 500;

        let mut input = BTreeMap::new();
        let mut new_time_tree = BTreeMap::new();
        new_time_tree.insert(0, (0, 1));
        // Add a dangling time before the common time
        input.insert(1_381_844_923_000, new_time_tree);

        for (i, time) in common_times.iter().enumerate() {
            let mut new_time_tree = BTreeMap::new();
            // gpubox 0.
            new_time_tree.insert(0, (0, i + 2));
            // gpubox 1.
            new_time_tree.insert(1, (0, i + 1));
            input.insert(*time, new_time_tree);
        }

        let mut new_time_tree = BTreeMap::new();
        new_time_tree.insert(1, (0, common_times.len() + 1));
        // Add a dangling time after the common time
        input.insert(1_381_844_924_000, new_time_tree);

        let expected_start = *common_times.first().unwrap();
        let expected_end = *common_times.last().unwrap() + integration_time_ms;
        // Duration = common end - common start + integration time
        // == 1_381_844_923_500 - 1_381_844_923_500 + 500
        let expected_duration = 500;

        let result = determine_obs_times(&input, integration_time_ms);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.start_millisec, expected_start);
        assert_eq!(result.end_millisec, expected_end);
        assert_eq!(result.duration_millisec, expected_duration);
    }
}
*/
