// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Functions for organising and checking the consistency of voltage files.
*/

pub mod error;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use regex::Regex;

use crate::*;
pub use error::VoltageFileError;

#[derive(Debug)]
pub(crate) struct ObsTimes {
    pub start_gps_time_ms: u64, // Start= start of first timestep
    pub end_gps_time_ms: u64,   // End  = start of last timestep + interval time
    pub duration_ms: u64,
    pub voltage_file_interval_ms: u64, // number of milliseconds between each voltage file
}

/// This represents one group of voltage files with the same "batch" identitifer (gps time).
/// e.g.
/// MWA Legacy: obsid_gpstime_datetime_chan
/// MWAX      : obsid_gpstime_datetime_chan
pub(crate) struct VoltageFileBatch {
    pub gps_time: u64,                   // 1234567890
    pub voltage_files: Vec<VoltageFile>, // Vector storing the details of each voltage file in this batch
}

impl VoltageFileBatch {
    pub fn new(gps_time: u64) -> Self {
        Self {
            gps_time,
            voltage_files: vec![],
        }
    }
}

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
pub(crate) struct VoltageFile {
    /// Filename of voltage file
    pub filename: String,
    /// channel number (receiver channel number 001..255)
    pub channel_identifier: usize,
}

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
    gps_time: u64,
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

/// A type alias for a horrible type:
/// `BTreeMap<u64, BTreeMap<usize, String>>`
///
/// The outer-most keys are GPS seconds, which correspond to the
/// unique times in supplied voltage files. Each of these
/// keys is associated with a tree; the keys of these trees are the
/// coarse-channel numbers, which then refer to the filename.
///
/// Example of what the map looks like with some sample data (2 gps times which have 1 coarse channel each - 120 -
/// plus 5 gps times with 2 coarse channels - 121 and 122):
/// 1065880128: {120: "1065880128_1065880128_ch120.dat"}
/// 1065880129: {121: "1065880128_1065880129_ch121.dat", 122: "1065880128_1065880129_ch122.dat"}
/// 1065880130: {121: "1065880128_1065880130_ch121.dat", 122: "1065880128_1065880130_ch122.dat"}
/// 1065880131: {121: "1065880128_1065880131_ch121.dat", 122: "1065880128_1065880131_ch122.dat"}
/// 1065880132: {121: "1065880128_1065880132_ch121.dat", 122: "1065880128_1065880132_ch122.dat"}
/// 1065880133: {121: "1065880128_1065880133_ch121.dat", 122: "1065880128_1065880133_ch122.dat"}
/// 1065880134: {120: "1065880128_1065880134_ch120.dat"}
pub(crate) type VoltageFileTimeMap = BTreeMap<u64, BTreeMap<usize, String>>;

/// A little struct to help us not get confused when dealing with the returned
/// values from complex functions.
#[derive(Debug)]
pub(crate) struct VoltageFileInfo {
    pub gpstime_batches: Vec<VoltageFileBatch>,
    pub corr_format: CorrelatorVersion,
    pub time_map: VoltageFileTimeMap,
    pub file_size: u64,
    pub voltage_file_interval_ms: u64,
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
) -> Result<HashMap<u64, VoltageFileBatch>, VoltageFileError> {
    // unwrap is safe as a check is performed above to ensure that there are
    // some files present.
    let batches = temp_voltage_files.iter().map(|g| g.gps_time);
    let mut voltage_file_batches: HashMap<u64, VoltageFileBatch> = HashMap::new();
    for b in batches {
        voltage_file_batches.insert(b, VoltageFileBatch::new(b as u64));
    }

    for temp_v in temp_voltage_files.iter() {
        let v = VoltageFile {
            filename: temp_v.filename.to_string(),
            channel_identifier: temp_v.channel_identifier,
        };
        let batch = voltage_file_batches.get_mut(&temp_v.gps_time).unwrap();
        batch.voltage_files.push(v);
    }

    // Ensure the output is properly sorted - each batch is sorted by
    // channel_identifier.
    for (_, v) in &mut voltage_file_batches {
        v.voltage_files
            .sort_unstable_by(|a, b| a.channel_identifier.cmp(&b.channel_identifier));
    }
    Ok(voltage_file_batches)
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
) -> Result<(Vec<TempVoltageFile>, CorrelatorVersion, usize, u64), VoltageFileError> {
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

    // Determine the interval between files
    let voltage_file_interval_seconds: u64 = match format.unwrap() {
        CorrelatorVersion::V2 => 8,
        CorrelatorVersion::Legacy => 1,
        CorrelatorVersion::OldLegacy => 1,
    };

    // Check batches are contiguous and have equal numbers of files.
    let mut batches_and_files: BTreeMap<u64, u8> = BTreeMap::new();
    for voltage_file in &temp_voltage_files {
        *batches_and_files.entry(voltage_file.gps_time).or_insert(0) += 1;
    }

    let mut file_count: Option<u8> = None;
    let mut prev_batch_num: u64 = 0;
    for (_, (batch_num, num_files)) in batches_and_files.iter().enumerate() {
        // Check that the previous batch + voltage_file_interval_seconds == the current batch number
        // This is our contiguity check
        if prev_batch_num != 0 {
            if prev_batch_num + voltage_file_interval_seconds != *batch_num {
                return Err(VoltageFileError::GpsTimeMissing {
                    expected: prev_batch_num + voltage_file_interval_seconds,
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

    Ok((
        temp_voltage_files,
        format.unwrap(),
        batches_and_files.len(),
        voltage_file_interval_seconds * 1000,
    ))
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
pub(crate) fn examine_voltage_files<T: AsRef<Path>>(
    metafits_context: &MetafitsContext,
    voltage_filenames: &[T],
) -> Result<VoltageFileInfo, VoltageFileError> {
    let (temp_voltage_files, corr_format, _, voltage_file_interval_ms) =
        determine_voltage_file_gpstime_batches(
            voltage_filenames,
            metafits_context.obs_id as usize,
        )?;

    let time_map = create_time_map(&temp_voltage_files)?;

    let mut gpstime_batches: HashMap<u64, VoltageFileBatch> =
        convert_temp_voltage_files(temp_voltage_files)?;

    // Determine the size of each voltage file. mwalib will throw an
    // error if this size is not consistent for all voltage files.
    let mut voltage_file_size: Option<u64> = None;
    for (_, b) in &mut gpstime_batches {
        for v in &mut b.voltage_files {
            let this_size;
            let metadata = std::fs::metadata(&v.filename);
            match metadata {
                Ok(m) => {
                    this_size = m.len();
                }
                Err(e) => {
                    return Err(VoltageFileError::VoltageFileError(
                        (*v.filename).to_string(),
                        String::from(format!("{}", e)),
                    ));
                }
            };
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
    // Not very rust like! I want to use gpstime_batches.into_values().collect(), but it is still listed as an "unstable" feature.
    let mut gpstime_batches_vec: Vec<VoltageFileBatch> = Vec::new();
    for (_, b) in gpstime_batches {
        gpstime_batches_vec.push(b);
    }
    gpstime_batches_vec.sort_by_key(|b| b.gps_time);

    Ok(VoltageFileInfo {
        gpstime_batches: gpstime_batches_vec,
        corr_format,
        time_map,
        file_size: voltage_file_size.unwrap(),
        voltage_file_interval_ms: voltage_file_interval_ms,
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
/// # Returns
///
/// * A struct containing the start and end times based on what we actually got, so all coarse channels match.
///
///
pub(crate) fn determine_obs_times(
    voltage_time_map: &VoltageFileTimeMap,
    voltage_file_interval_ms: u64,
) -> Result<ObsTimes, VoltageFileError> {
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

    //let proper_start_ms: u64 = *voltage_time_map.iter().next().unwrap().0 as u64;
    //let proper_end_ms: u64 = *voltage_time_map.iter().next_back().unwrap().0 as u64
    //    + timestep_interval_ms as u64;
    let proper_start_gps_time: u64 = *i.next().unwrap().0 as u64;
    let proper_end_gps_time: u64 = match i.last() {
        Some(s) => *s.0 as u64,
        None => {
            // Looks like we only have 1 hdu, so end
            proper_start_gps_time
        }
    };

    Ok(ObsTimes {
        start_gps_time_ms: proper_start_gps_time * 1000,
        end_gps_time_ms: (proper_end_gps_time * 1000) + voltage_file_interval_ms,
        duration_ms: ((proper_end_gps_time - proper_start_gps_time) * 1000)
            + voltage_file_interval_ms,
        voltage_file_interval_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{Error, Write};
    // Helper fuction to generate (small) test voltage files
    fn generate_test_voltage_file(
        temp_dir: &tempdir::TempDir,
        filename: &str,
        time_samples: usize,
        rf_inputs: usize,
    ) -> Result<String, Error> {
        let tdir_path = temp_dir.path();
        let full_filename = tdir_path.join(filename);

        let mut output_file = File::create(&full_filename)?;
        // Write out x time samples
        // Each sample has x rfinputs
        // and 1 float for real 1 float for imaginary
        let floats = time_samples * rf_inputs * 2;
        let mut buffer: Vec<f32> = vec![0.0; floats];

        let mut bptr: usize = 0;

        // This will write out the sequence:
        // 0.25, 0.75, 1.25, 1.75..511.25,511.75  (1024 floats in all)
        for t in 0..time_samples {
            for r in 0..rf_inputs {
                // real
                buffer[bptr] = ((t * rf_inputs) + r) as f32 + 0.25;
                bptr += 1;
                // imag
                buffer[bptr] = ((t * rf_inputs) + r) as f32 + 0.75;
                bptr += 1;
            }
        }
        output_file.write_all(misc::as_u8_slice(buffer.as_slice()))?;
        output_file.flush()?;

        Ok(String::from(full_filename.to_str().unwrap()))
    }

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
        let (temp_voltage_files, corr_format, num_gputimes, voltage_file_interval_ms) =
            result.unwrap();
        assert_eq!(corr_format, CorrelatorVersion::Legacy);
        assert_eq!(num_gputimes, 3);
        assert_eq!(voltage_file_interval_ms, 1000);

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
            "1065880128_1065880136_122.sub",
            "1065880128_1065880136_21.sub",
            "1065880128_1065880136_1.sub",
            "1065880128_1065880128_21.sub",
            "1065880128_1065880128_001.sub",
            "1065880128_1065880128_122.sub",
            "1065880128_1065880144_122.sub",
            "1065880128_1065880144_021.sub",
            "1065880128_1065880144_01.sub",
        ];
        let result = determine_voltage_file_gpstime_batches(&files, 1065880128);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
        let (temp_voltage_files, corr_format, num_gputimes, voltage_file_interval_ms) =
            result.unwrap();
        assert_eq!(corr_format, CorrelatorVersion::V2);
        assert_eq!(num_gputimes, 3);
        assert_eq!(voltage_file_interval_ms, 8000);

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
                filename: "1065880128_1065880136_1.sub",
                obs_id: 1065880128,
                gps_time: 1065880136,
                channel_identifier: 1,
            },
            TempVoltageFile {
                filename: "1065880128_1065880136_21.sub",
                obs_id: 1065880128,
                gps_time: 1065880136,
                channel_identifier: 21,
            },
            TempVoltageFile {
                filename: "1065880128_1065880136_122.sub",
                obs_id: 1065880128,
                gps_time: 1065880136,
                channel_identifier: 122,
            },
            TempVoltageFile {
                filename: "1065880128_1065880144_01.sub",
                obs_id: 1065880128,
                gps_time: 1065880144,
                channel_identifier: 1,
            },
            TempVoltageFile {
                filename: "1065880128_1065880144_021.sub",
                obs_id: 1065880128,
                gps_time: 1065880144,
                channel_identifier: 21,
            },
            TempVoltageFile {
                filename: "1065880128_1065880144_122.sub",
                obs_id: 1065880128,
                gps_time: 1065880144,
                channel_identifier: 122,
            },
        ];

        assert_eq!(temp_voltage_files, expected_voltage_files);
    }

    #[test]
    fn test_determine_voltage_file_gpstime_batches_chan_mismatch() {
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
    fn test_determine_voltage_file_gpstime_batches_no_files() {
        let files: Vec<String> = Vec::new();
        let result = determine_voltage_file_gpstime_batches(&files, 1065880128);
        assert!(matches!(
            result.unwrap_err(),
            VoltageFileError::NoVoltageFiles
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
            VoltageFileError::GpsTimeMissing {
                expected: _,
                got: _
            }
        ));
    }

    #[test]
    fn test_determine_obs_times_test_many_timesteps_legacy() {
        let common_times: Vec<u64> =
            vec![1065880129, 1065880130, 1065880131, 1065880132, 1065880133];
        let mut input = VoltageFileTimeMap::new();
        // insert a "dangling time" at the beginning (1065880128) which is not a common timestep
        let mut new_time_tree = BTreeMap::new();
        new_time_tree.insert(120, String::from("1065880128_1065880128_ch120.dat"));
        input.insert(1065880128, new_time_tree);

        // Add the common times to the data structure
        for time in common_times.iter() {
            input
                .entry(*time)
                .or_insert_with(BTreeMap::new)
                .entry(121)
                .or_insert(format!("1065880128_{}_ch121.dat", time));

            input
                .entry(*time)
                .or_insert_with(BTreeMap::new)
                .entry(122)
                .or_insert(format!("1065880128_{}_ch122.dat", time));
        }

        // insert a "dangling time" at the end (1065880134) which is not a common timestep
        new_time_tree = BTreeMap::new();
        new_time_tree.insert(120, String::from("1065880128_1065880134_ch120.dat"));
        input.insert(1065880134, new_time_tree);

        let expected_interval: u64 = 1000; // 1000 since we are Legacy
        let expected_start: u64 = *common_times.first().unwrap() * 1000;
        let expected_end: u64 = (*common_times.last().unwrap() * 1000) + expected_interval;
        // Duration = common end - common start + integration time
        // == 1065880133 - 1065880129 + 1
        let expected_duration = 5000;
        let voltage_file_interval_ms: u64 = 1000;

        let result = determine_obs_times(&input, voltage_file_interval_ms);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(
            result.start_gps_time_ms, expected_start,
            "start_gps_time_ms incorrect {:?}",
            result
        );
        assert_eq!(
            result.end_gps_time_ms, expected_end,
            "end_gps_time_ms incorrect {:?}",
            result
        );
        assert_eq!(
            result.duration_ms, expected_duration,
            "duration_ms incorrect {:?}",
            result
        );
        assert_eq!(
            result.voltage_file_interval_ms, expected_interval,
            "voltage_file_interval_ms incorrect {:?}",
            result
        );
    }

    #[test]
    fn test_determine_obs_times_test_many_timesteps_mwax() {
        let common_times: Vec<u64> =
            vec![1065880136, 1065880144, 1065880152, 1065880160, 1065880168];
        let mut input = VoltageFileTimeMap::new();
        // insert a "dangling time" at the beginning (1065880128) which is not a common timestep
        let mut new_time_tree = BTreeMap::new();
        new_time_tree.insert(120, String::from("1065880128_1065880128_120.sub"));
        input.insert(1065880128, new_time_tree);

        // Add the common times to the data structure
        for time in common_times.iter() {
            input
                .entry(*time)
                .or_insert_with(BTreeMap::new)
                .entry(121)
                .or_insert(format!("1065880128_{}_121.sub", time));

            input
                .entry(*time)
                .or_insert_with(BTreeMap::new)
                .entry(122)
                .or_insert(format!("1065880128_{}_122.sub", time));
        }

        // insert a "dangling time" at the end (1065880176) which is not a common timestep
        new_time_tree = BTreeMap::new();
        new_time_tree.insert(120, String::from("1065880128_1065880176_120.sub"));
        input.insert(1065880176, new_time_tree);

        let expected_interval: u64 = 8000; // 8000 since we are MWAX
        let expected_start: u64 = *common_times.first().unwrap() * 1000;
        let expected_end: u64 = (*common_times.last().unwrap() * 1000) + expected_interval;
        // Duration = common end - common start + integration time
        // == 1065880168 - 1065880136 + 8
        let expected_duration = 40000;
        let voltage_file_interval_ms: u64 = 8000;

        let result = determine_obs_times(&input, voltage_file_interval_ms);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(
            result.start_gps_time_ms, expected_start,
            "start_gps_time_ms incorrect {:?}",
            result
        );
        assert_eq!(
            result.end_gps_time_ms, expected_end,
            "end_gps_time_ms incorrect {:?}",
            result
        );
        assert_eq!(
            result.duration_ms, expected_duration,
            "duration_ms incorrect {:?}",
            result
        );
        assert_eq!(
            result.voltage_file_interval_ms, expected_interval,
            "voltage_file_interval_ms incorrect {:?}",
            result
        );
    }

    #[test]
    fn test_voltage_file_batch_new() {
        let new_batch = VoltageFileBatch::new(1234567890);

        // Check the new batch is created ok
        assert_eq!(new_batch.gps_time, 1234567890);
        assert_eq!(new_batch.voltage_files.len(), 0);
    }

    #[test]
    fn test_voltage_file_batch_partialeq() {
        let mut batch1 = VoltageFileBatch::new(1234567890);
        let voltage_file1 = VoltageFile {
            filename: String::from("test.dat"),
            channel_identifier: 123,
        };
        batch1.voltage_files.push(voltage_file1);

        // Should be == to batch1
        let mut batch2 = VoltageFileBatch::new(1234567890);
        let voltage_file2 = VoltageFile {
            filename: String::from("test.dat"),
            channel_identifier: 123,
        };
        batch2.voltage_files.push(voltage_file2);

        // Should be != batch1 (filename)
        let mut batch3 = VoltageFileBatch::new(1234567890);
        let voltage_file3 = VoltageFile {
            filename: String::from("test1.dat"),
            channel_identifier: 123,
        };
        batch3.voltage_files.push(voltage_file3);

        // Should be != batch1 (gpstime)
        let mut batch4 = VoltageFileBatch::new(9876543210);
        let voltage_file4 = VoltageFile {
            filename: String::from("test.dat"),
            channel_identifier: 123,
        };
        batch4.voltage_files.push(voltage_file4);

        // Check the eq works
        assert_eq!(batch1, batch2);

        assert_ne!(batch1, batch3);

        assert_ne!(batch1, batch4);
    }

    #[test]
    fn test_convert_temp_voltage_files() {
        let mut temp_voltage_files: Vec<TempVoltageFile> = Vec::new();

        temp_voltage_files.push(TempVoltageFile {
            filename: "1234567000_1234567000_123.sub",
            obs_id: 1234567000,
            channel_identifier: 123,
            gps_time: 1234567000,
        });

        temp_voltage_files.push(TempVoltageFile {
            filename: "1234567890_1234567008_124.sub",
            obs_id: 1234567000,
            channel_identifier: 124,
            gps_time: 1234567008,
        });

        temp_voltage_files.push(TempVoltageFile {
            filename: "1234567890_1234567008_123.sub",
            obs_id: 1234567000,
            channel_identifier: 123,
            gps_time: 1234567008,
        });

        temp_voltage_files.push(TempVoltageFile {
            filename: "1234567890_1234567008_125.sub",
            obs_id: 1234567000,
            channel_identifier: 125,
            gps_time: 1234567008,
        });

        temp_voltage_files.push(TempVoltageFile {
            filename: "1234567000_1234567000_124.sub",
            obs_id: 1234567000,
            channel_identifier: 124,
            gps_time: 1234567000,
        });

        let result = convert_temp_voltage_files(temp_voltage_files);

        // The resulting VoltageFileBatches should:
        // * have 2 batches
        // * batches sorted by gpstime
        // * each batch sorted by coarse channel indentifier
        assert!(result.is_ok());

        let batches: HashMap<u64, VoltageFileBatch> = result.unwrap();

        assert_eq!(
            batches.len(),
            2,
            "Error - number of batches is incorrect: {} should be 2.",
            batches.len()
        );
        assert_eq!(batches.get(&1234567000).unwrap().voltage_files.len(), 2);
        assert_eq!(batches.get(&1234567008).unwrap().voltage_files.len(), 3);
    }

    #[test]
    fn test_examine_voltage_files_valid() {
        // Get a metafits context
        // Open the metafits file
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

        //
        // Read the observation using mwalib
        //
        // Open a context and load in a test metafits
        let context =
            MetafitsContext::new(&metafits_filename).expect("Failed to create MetafitsContext");

        // Create a temp dir for the temp files
        // Once out of scope the temp dir and it's contents will be deleted
        let temp_dir = tempdir::TempDir::new("voltage_test").unwrap();

        // Populate vector of filenames
        let mut voltage_filenames: Vec<String> = Vec::new();
        voltage_filenames.push(String::from("1101503312_1101503312_123.sub"));
        voltage_filenames.push(String::from("1101503312_1101503312_124.sub"));
        voltage_filenames.push(String::from("1101503312_1101503320_123.sub"));
        voltage_filenames.push(String::from("1101503312_1101503320_124.sub"));

        let mut temp_filenames: Vec<String> = Vec::new();

        for f in voltage_filenames.iter() {
            temp_filenames.push(generate_test_voltage_file(&temp_dir, f, 2, 256).unwrap());
        }
        let result = examine_voltage_files(&context, &temp_filenames);

        assert!(
            result.is_ok(),
            "examine_voltage_files failed {:?} - temp filenames: {:?}",
            result,
            temp_filenames
        );
    }

    #[test]
    fn test_examine_voltage_files_error_mismatched_sizes() {
        // Get a metafits context
        // Open the metafits file
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

        //
        // Read the observation using mwalib
        //
        // Open a context and load in a test metafits
        let context =
            MetafitsContext::new(&metafits_filename).expect("Failed to create MetafitsContext");

        // Create a temp dir for the temp files
        // Once out of scope the temp dir and it's contents will be deleted
        let temp_dir = tempdir::TempDir::new("voltage_test").unwrap();

        // Populate vector of filenames
        let mut voltage_filenames: Vec<String> = Vec::new();
        voltage_filenames.push(String::from("1101503312_1101503312_123.sub"));
        voltage_filenames.push(String::from("1101503312_1101503312_124.sub"));
        voltage_filenames.push(String::from("1101503312_1101503320_123.sub"));
        voltage_filenames.push(String::from("1101503312_1101503320_124.sub"));

        let mut temp_filenames: Vec<String> = Vec::new();

        for f in voltage_filenames.iter() {
            temp_filenames.push(generate_test_voltage_file(&temp_dir, f, 2, 256).unwrap());
        }
        // Now add a gps time batch which is a different size
        temp_filenames.push(
            generate_test_voltage_file(&temp_dir, "1101503312_1101503328_123.sub", 1, 256).unwrap(),
        );
        temp_filenames.push(
            generate_test_voltage_file(&temp_dir, "1101503312_1101503328_124.sub", 1, 256).unwrap(),
        );

        let result = examine_voltage_files(&context, &temp_filenames);

        assert!(result.is_err());

        assert!(matches!(
            result.unwrap_err(),
            VoltageFileError::UnequalFileSizes
        ));
    }

    #[test]
    fn test_examine_voltage_files_error_gpstime_gaps() {
        // Get a metafits context
        // Open the metafits file
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

        //
        // Read the observation using mwalib
        //
        // Open a context and load in a test metafits
        let context =
            MetafitsContext::new(&metafits_filename).expect("Failed to create MetafitsContext");

        // Create a temp dir for the temp files
        // Once out of scope the temp dir and it's contents will be deleted
        let temp_dir = tempdir::TempDir::new("voltage_test").unwrap();

        // Populate vector of filenames
        let mut voltage_filenames: Vec<String> = Vec::new();
        voltage_filenames.push(String::from("1101503312_1101503312_123.sub"));
        voltage_filenames.push(String::from("1101503312_1101503312_124.sub"));
        // Gap of 8 seconds here
        voltage_filenames.push(String::from("1101503312_1101503328_123.sub"));
        voltage_filenames.push(String::from("1101503312_1101503328_124.sub"));

        let mut temp_filenames: Vec<String> = Vec::new();

        for f in voltage_filenames.iter() {
            temp_filenames.push(generate_test_voltage_file(&temp_dir, f, 2, 256).unwrap());
        }

        let result = examine_voltage_files(&context, &temp_filenames);

        assert!(result.is_err());

        assert!(matches!(
            result.unwrap_err(),
            VoltageFileError::GpsTimeMissing {
                expected: _,
                got: _
            }
        ));
    }

    #[test]
    fn test_examine_voltage_files_error_file_not_found() {
        // Get a metafits context
        // Open the metafits file
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

        //
        // Read the observation using mwalib
        //
        // Open a context and load in a test metafits
        let context =
            MetafitsContext::new(&metafits_filename).expect("Failed to create MetafitsContext");

        // Populate vector of filenames
        let mut voltage_filenames: Vec<String> = Vec::new();
        voltage_filenames.push(String::from(
            "test_files_invalid/1101503312_1101503312_123.sub",
        ));
        voltage_filenames.push(String::from(
            "test_files_invalid/1101503312_1101503312_124.sub",
        ));
        voltage_filenames.push(String::from(
            "test_files_invalid/1101503312_1101503320_123.sub",
        ));
        voltage_filenames.push(String::from(
            "test_files_invalid/1101503312_1101503320_124.sub",
        ));

        let result = examine_voltage_files(&context, &voltage_filenames);

        assert!(result.is_err());

        assert!(matches!(
            result.unwrap_err(),
            VoltageFileError::VoltageFileError(_, _)
        ));
    }
}
