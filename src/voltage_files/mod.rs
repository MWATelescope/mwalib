// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Functions for organising and checking the consistency of voltage files.
*/
pub mod error;
use crate::*;
pub use error::VoltageFileError;
use regex::Regex;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

#[cfg(test)]
mod test;

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
    pub mwa_version: MWAVersion,
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
) -> HashMap<u64, VoltageFileBatch> {
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
    for v in voltage_file_batches.values_mut() {
        v.voltage_files
            .sort_unstable_by(|a, b| a.channel_identifier.cmp(&b.channel_identifier));
    }
    voltage_file_batches
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
///   `MWAVersion`, the number of voltage files supplied, and the number of
///   gps time batches.
///
///
fn determine_voltage_file_gpstime_batches<T: AsRef<Path>>(
    voltage_filenames: &[T],
    metafits_obs_id: usize,
) -> Result<(Vec<TempVoltageFile>, MWAVersion, usize, u64), VoltageFileError> {
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
                        None => format = Some(MWAVersion::VCSMWAXv2),
                        Some(MWAVersion::VCSMWAXv2) => (),
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
                            None => format = Some(MWAVersion::VCSLegacyRecombined),
                            Some(MWAVersion::VCSLegacyRecombined) => (),
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
    let mwa_version = format.unwrap();
    let voltage_file_interval_seconds: u64 = match mwa_version {
        MWAVersion::VCSMWAXv2 => 8,
        MWAVersion::VCSLegacyRecombined => 1,
        _ => return Err(VoltageFileError::InvalidMwaVersion { mwa_version }),
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
        if prev_batch_num != 0 && prev_batch_num + voltage_file_interval_seconds != *batch_num {
            return Err(VoltageFileError::GpsTimeMissing {
                expected: prev_batch_num + voltage_file_interval_seconds,
                got: *batch_num,
            });
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
/// reflected in the returned `MWAVersion`.
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
    let (temp_voltage_files, mwa_version, _, voltage_file_interval_ms) =
        determine_voltage_file_gpstime_batches(
            voltage_filenames,
            metafits_context.obs_id as usize,
        )?;

    let time_map = create_time_map(&temp_voltage_files);

    let mut gpstime_batches: HashMap<u64, VoltageFileBatch> =
        convert_temp_voltage_files(temp_voltage_files);

    // Determine the size of each voltage file. mwalib will throw an
    // error if this size is not consistent for all voltage files.
    let mut voltage_file_size: Option<u64> = None;
    for b in gpstime_batches.values_mut() {
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
                        format!("{}", e),
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
        mwa_version,
        time_map,
        file_size: voltage_file_size.unwrap(),
        voltage_file_interval_ms,
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
fn create_time_map(voltage_file_batches: &[TempVoltageFile]) -> VoltageFileTimeMap {
    // create a map
    let mut voltage_time_map = BTreeMap::new();
    for voltage_file in voltage_file_batches.iter() {
        voltage_time_map
            .entry(voltage_file.gps_time)
            .or_insert_with(BTreeMap::new)
            .entry(voltage_file.channel_identifier)
            .or_insert_with(|| voltage_file.filename.to_string());
    }

    voltage_time_map
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
