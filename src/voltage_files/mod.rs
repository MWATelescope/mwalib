// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Functions for organising and checking the consistency of voltage files.

pub mod error;
use crate::*;
pub use error::VoltageFileError;
use regex::Regex;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::path::Path;

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3_stub_gen_derive::gen_stub_pyclass;

#[cfg(test)]
mod test;

#[derive(Debug)]
pub(crate) struct ObsTimesAndChans {
    pub start_gps_time_ms: u64, // Start= start of first timestep
    pub end_gps_time_ms: u64,   // End  = start of last timestep + interval time
    pub duration_ms: u64,
    pub coarse_chan_identifiers: Vec<usize>, // Vector of Correlator Coarse Chan identifiers (gpubox number or rec chan number)
}

/// This represents one group of voltage files with the same "batch" identitifer (gps time).
/// e.g.
/// MWA Legacy: obsid_gpstime_datetime_chan
/// MWAX      : obsid_gpstime_datetime_chan
#[cfg_attr(feature = "python", gen_stub_pyclass, pyclass(get_all, set_all))]
#[derive(Clone)]
pub struct VoltageFileBatch {
    // GPS second of this observation. e.g. 1234567890
    pub gps_time_seconds: u64,

    /// Vector storing the details of each voltage file in this batch
    pub voltage_files: Vec<VoltageFile>,
}

impl VoltageFileBatch {
    pub fn new(gps_time: u64) -> Self {
        Self {
            gps_time_seconds: gps_time,
            voltage_files: vec![],
        }
    }
}

impl fmt::Debug for VoltageFileBatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "gps_time={} voltage_files={:?}",
            self.gps_time_seconds, self.voltage_files,
        )
    }
}

/// This represents one voltage file
#[cfg_attr(feature = "python", gen_stub_pyclass, pyclass(get_all, set_all))]
#[derive(Clone)]
pub struct VoltageFile {
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
        self.gps_time_seconds == other.gps_time_seconds && self.voltage_files == other.voltage_files
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
    gps_time_seconds: u64,
}

impl std::cmp::PartialEq for TempVoltageFile<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
            && self.obs_id == other.obs_id
            && self.channel_identifier == other.channel_identifier
            && self.gps_time_seconds == other.gps_time_seconds
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
    #[allow(dead_code)]
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
    let batches = temp_voltage_files.iter().map(|g| g.gps_time_seconds);
    let mut voltage_file_batches: HashMap<u64, VoltageFileBatch> = HashMap::new();
    for b in batches {
        voltage_file_batches.insert(b, VoltageFileBatch::new(b));
    }

    for temp_v in temp_voltage_files.iter() {
        let v = VoltageFile {
            filename: temp_v.filename.to_string(),
            channel_identifier: temp_v.channel_identifier,
        };
        let batch = voltage_file_batches
            .get_mut(&temp_v.gps_time_seconds)
            .unwrap();
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
///   all of the voltage filenames provided by the client.
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
    voltage_filenames: &'_ [T],
    metafits_obs_id: usize,
) -> Result<(Vec<TempVoltageFile<'_>>, MWAVersion, usize, u64), VoltageFileError> {
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
                        gps_time_seconds: caps["gpstime"].parse().unwrap(),
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
                            gps_time_seconds: caps["gpstime"].parse().unwrap(),
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
        MWAVersion::VCSMWAXv2 => MWA_VCS_MWAXV2_SUBFILE_SECONDS,
        MWAVersion::VCSLegacyRecombined => MWA_VCS_LEGACY_RECOMBINED_FILE_SECONDS,
        _ => return Err(VoltageFileError::InvalidMwaVersion { mwa_version }),
    };

    // Check batches are contiguous and have equal numbers of files.
    let mut batches_and_files: BTreeMap<u64, u8> = BTreeMap::new();
    for voltage_file in &temp_voltage_files {
        *batches_and_files
            .entry(voltage_file.gps_time_seconds)
            .or_insert(0) += 1;
    }

    let mut file_count: Option<u8> = None;
    let mut prev_batch_num: u64 = 0;
    for (batch_num, num_files) in batches_and_files.iter() {
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
    temp_voltage_files.sort_unstable_by_key(|v| (v.gps_time_seconds, v.channel_identifier));

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
///   containing all of the voltage filenames provided by the client.
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

    let time_map: VoltageFileTimeMap = create_time_map(&temp_voltage_files);

    let mut gpstime_batches: HashMap<u64, VoltageFileBatch> =
        convert_temp_voltage_files(temp_voltage_files);

    // Determine the size of each voltage file. mwalib will throw an
    // error if this size is not consistent for all voltage files.
    let mut voltage_file_size: Option<u64> = None;
    for b in gpstime_batches.values_mut() {
        for v in &mut b.voltage_files {
            let metadata = std::fs::metadata(&v.filename);
            let this_size = match metadata {
                Ok(m) => m.len(),
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
    gpstime_batches_vec.sort_by_key(|b| b.gps_time_seconds);

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
            .entry(voltage_file.gps_time_seconds)
            .or_insert_with(BTreeMap::new)
            .entry(voltage_file.channel_identifier)
            .or_insert_with(|| voltage_file.filename.to_string());
    }

    voltage_time_map
}

/// Returns a vector of timestep indicies which exist in the VoltageFileTimeMap (i.e. the user has provided at least some data files for these timesteps)
///
/// # Arguments
///
/// * `voltage_time_map` - BTree structure containing the map of what voltage data files and timesteps we were supplied by the client.
///
/// * `volt_timesteps` - Vector of Voltage Context TimeStep structs.
///
/// # Returns
///
/// * A vector of timestep indices for which at least some data files have been provided
///
///
pub(crate) fn populate_provided_timesteps(
    voltage_time_map: &VoltageFileTimeMap,
    volt_timesteps: &[TimeStep],
) -> Vec<usize> {
    // populate a vector with the indicies of corr_timesteps that correspond to the gps times which are in
    // the first level of the voltage_time_map. This represents all of the timesteps we have at least some data for
    let mut return_vec: Vec<usize> = voltage_time_map
        .iter()
        .map(|t| {
            volt_timesteps
                .iter()
                .position(|v| v.gps_time_ms == *t.0 * 1000)
                .unwrap()
        })
        .collect();

    // Ensure vector is sorted
    return_vec.sort_unstable();

    return_vec
}

/// Returns a vector of coarse chan indicies which exist in the VoltageFileTimeMap (i.e. the user has provided at least some data files for these coarse channels)
///
/// # Arguments
///
/// * `voltage_time_map` - BTree structure containing the map of what voltage data files and timesteps we were supplied by the client.
///
/// * `volt_coarse_chans` - Vector of Voltage Context CoarseChannel structs.
///
/// # Returns
///
/// * A vector of coarse channel indices for which at least some data files have been provided
///
///
pub(crate) fn populate_provided_coarse_channels(
    voltage_time_map: &VoltageFileTimeMap,
    volt_coarse_chans: &[CoarseChannel],
) -> Vec<usize> {
    // Go through all timesteps in the GpuBoxTimeMap.
    // For each timestep get each coarse channel identifier and add it into the HashSet
    let chans: HashSet<usize> = voltage_time_map
        .iter()
        .flat_map(|ts| ts.1.iter().map(|ch| *ch.0))
        .collect::<HashSet<usize>>();

    // We should now have a small HashSet of coarse channel identifiers
    // Get the index of each item in the hashset from the correlator coarse channels passed in and add that to the return vector
    let mut return_vec: Vec<usize> = chans
        .iter()
        .map(|c| {
            volt_coarse_chans
                .iter()
                .position(|v| v.gpubox_number == *c)
                .unwrap()
        })
        .collect();

    // Ensure vector is sorted
    return_vec.sort_unstable();

    return_vec
}

/// Determine the common start and end times of an observation. In this context,
/// "common" refers to a time that is common to the all of provided coarse channels and contiguous. e.g.
///
/// ```text
/// time:   0123456789abcdef
/// chan01: ###############
/// chan02:  #############
/// chan03: #####    ######
/// chan04:   ############
/// chan05: ###############
/// chan06:                #
/// chans07-24: <none>
/// ```
/// Example 1:
/// In the above example, there is at least some timesteps from coarse channels 01-06. But there are NO timesteps that contain all 6 channels so this function
/// would return a None.
///
/// Example 2:
/// If you were to remove timestep "f", then there are 5 coarse channels, and the timesteps 2-4 inclusive are the common timesteps.
///
/// # Arguments
///
/// * `voltage_time_map` - BTree structure containing the map of what voltage files/channels and timesteps we were supplied by the client.
///
/// * `timestep_duration_ms` - the gap between timesteps in ms
///
/// * `good_time_gps_time_ms` - Option- Some is the 'good' time (i.e. the first time which is not part of the quack time). None means that
///   times during the quack time are ok to be included.
///
/// # Returns
///
/// * A Result which contains an Option containing a struct containing the start and end times based on what we actually got, so all coarse channels match, or None; or an Error.
///
///
pub(crate) fn determine_common_obs_times_and_chans(
    voltage_time_map: &VoltageFileTimeMap,
    timestep_duration_ms: u64,
    good_time_gps_time_ms: Option<u64>,
) -> Result<Option<ObsTimesAndChans>, VoltageFileError> {
    // If we pass in Some(good_time_gps_time_ms) then restrict the gpubox time map to times AFTER the quack time
    let timemap = match good_time_gps_time_ms {
        Some(good_time) => voltage_time_map
            .clone()
            .into_iter()
            .filter(|ts| ts.0 * 1000 >= good_time)
            .collect(),

        None => voltage_time_map.clone(),
    };

    // Go through all timesteps in the GpuBoxTimeMap.
    // For each timestep get each coarse channel identifier and add it into the HashSet, then dump them into a vector
    // get the length of the vector - we will use this to test each entry in the VoltageTimeMap
    let max_chans = voltage_time_map
        .iter()
        .flat_map(|ts| ts.1.iter().map(|ch| *ch.0))
        .collect::<HashSet<usize>>()
        .into_iter()
        .len();

    // Filter only the timesteps that have the same coarse channels
    let mut filtered_timesteps = timemap
        .into_iter()
        .filter(|(_, submap)| submap.len() == max_chans);

    // Get the first timestep where the num chans matches the provided channels. If we get None, then we did not find any timesteps which contain all the coarse channels
    let first_ts = match filtered_timesteps.next() {
        Some(ts) => ts,
        None => return Ok(None),
    };

    // Now for refernce lets get what the coarse channels are for this timestep- we will use it below when iterating through the filtered collection of timesteps
    let first_ts_chans = first_ts
        .1
        .iter()
        .map(|ts_chans| *ts_chans.0)
        .collect::<Vec<usize>>();
    let common_start_gps_ms = first_ts.0 * 1000;

    // In case there was only 1 timestep in the filtered timesteps, set the common end time now
    let mut common_end_gps_ms = common_start_gps_ms + timestep_duration_ms;

    // Iterate over the filtered timemap
    // Go to the next timestep unless:
    // * It is not contiguous with the previous
    // * It does not have that same max number of channels
    let mut prev_ts_gps_ms = common_start_gps_ms;
    loop {
        let next_item = filtered_timesteps.next();

        match next_item {
            Some(ts) => {
                // Check ts and prev ts are contiguous and channels match
                if (ts.0 * 1000 == prev_ts_gps_ms + timestep_duration_ms)
                    && first_ts_chans.len() == ts.1.len()
                {
                    // Update the end time
                    common_end_gps_ms = (ts.0 * 1000) + timestep_duration_ms;
                    prev_ts_gps_ms = ts.0 * 1000;
                } else {
                    break;
                }
            }
            None => break,
        }
    }

    Ok(Some(ObsTimesAndChans {
        start_gps_time_ms: common_start_gps_ms,
        end_gps_time_ms: common_end_gps_ms,
        duration_ms: common_end_gps_ms - common_start_gps_ms,
        coarse_chan_identifiers: first_ts_chans,
    }))
}
