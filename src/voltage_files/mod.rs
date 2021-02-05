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
    pub duration_seconds: u64,
    pub timestep_interval_milliseconds: u64, // number of milliseconds between each timestep
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
    for (i, (batch_num, num_files)) in batches_and_files.iter().enumerate() {
        if i != *batch_num {
            return Err(VoltageFileError::GpsTimeMissing {
                expected: i,
                got: *batch_num,
            });
        }

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

    let time_map = create_time_map(&temp_voltage_files, corr_format)?;

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
    correlator_version: CorrelatorVersion,
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
/// "proper" refers to a time that is common to all gpubox files. Because gpubox
/// files may not all start and end at the same time, anything "dangling" is
/// trimmed. e.g.
///
/// ```text
/// time:     0123456789abcdef
/// gpubox01: ################
/// gpubox02:  ###############
/// gpubox03: ################
/// gpubox04:   ##############
/// gpubox05: ###############
/// gpubox06: ################
/// ```
///
/// In this example, we start collecting data from time=2, and end at time=e,
/// because these are the first and last places that all gpubox files have
/// data. All dangling data is ignored.
///
/// See tests of this function or `obs_context.rs` for examples of constructing
/// the input to this function.
///
///
/// # Arguments
///
/// * `gpubox_time_map` - BTree structure containing the map of what gpubox files and timesteps we were supplied by the client.
///
/// * `integration_time_ms` - Correlator dump time (so we know the gap between timesteps)
///
/// # Returns
///
/// * A struct containing the start and end times based on what we actually got, so all coarse channels match.
///
///
pub fn determine_obs_times(
    voltage_time_map: &BTreeMap<usize, BTreeMap<usize, String>>,
    correlator_version: CorrelatorVersion,
) -> Result<ObsTimes, VoltageFileError> {
    let timestep_interval_milliseconds: u64 = match correlator_version {
        CorrelatorVersion::V2 => 8_000,
        CorrelatorVersion::Legacy => 1_000,
    };
    let proper_start_milliseconds: u64 = *voltage_time_map.iter().next().unwrap().0 as u64;
    let proper_end_milliseconds: u64 = *voltage_time_map.iter().next_back().unwrap().0 as u64
        + timestep_interval_milliseconds as u64;

    Ok(ObsTimes {
        start_gps_time_milliseconds: proper_start_milliseconds,
        end_gps_time_milliseconds: proper_end_milliseconds,
        duration_seconds: (proper_end_milliseconds - proper_start_milliseconds) as u64,
        timestep_interval_milliseconds,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_voltage_file_gpstime_batches_proper_format() {
        let files = vec![
            "1065880128_1065880129_ch123.dat",
            "1065880128_1065880128_ch121.dat",
            "1065880128_1065880130_ch124.dat",
        ];
        let result = determine_voltage_file_gpstime_batches(&files, 1065880128);
        assert!(result.is_ok());
        let (temp_voltage_files, corr_format, num_gputimes) = result.unwrap();
        assert_eq!(corr_format, CorrelatorVersion::Legacy);
        assert_eq!(num_gputimes, 3);

        let expected_voltage_files = vec![
            TempVoltageFile {
                filename: "1065880128_1065880128_ch121.dat",
                obs_id: 1065880128,
                gps_time: 1065880128,
                channel_identifier: 121,
            },
            TempVoltageFile {
                filename: "1065880128_1065880129_ch123.dat",
                obs_id: 1065880128,
                gps_time: 1065880129,
                channel_identifier: 123,
            },
            TempVoltageFile {
                filename: "1065880128_1065880128_ch121.dat",
                obs_id: 1065880128,
                gps_time: 1065880128,
                channel_identifier: 121,
            },
        ];

        assert_eq!(temp_voltage_files, expected_voltage_files);
    }
}
/*
    #[test]
    fn test_determine_gpubox_batches_proper_format2() {
        let files = vec![
            "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
            "/home/gs/1065880128_20131015134930_gpubox20_01.fits",
            "/var/cache/1065880128_20131015134930_gpubox15_02.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_ok());
        let (gpubox_batches, corr_format, num_batches) = result.unwrap();
        assert_eq!(corr_format, CorrelatorVersion::Legacy);
        assert_eq!(num_batches, 3);
        let expected_batches = vec![
            TempGPUBoxFile {
                filename: "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
                channel_identifier: 1,
                batch_number: 0,
            },
            TempGPUBoxFile {
                filename: "/home/gs/1065880128_20131015134930_gpubox20_01.fits",
                channel_identifier: 20,
                batch_number: 1,
            },
            TempGPUBoxFile {
                filename: "/var/cache/1065880128_20131015134930_gpubox15_02.fits",
                channel_identifier: 15,
                batch_number: 2,
            },
        ];

        assert_eq!(gpubox_batches, expected_batches);
    }

    #[test]
    fn test_determine_gpubox_batches_proper_format3() {
        let files = vec![
            "/home/chj/1065880128_20131015134930_gpubox02_00.fits",
            "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
            "/home/chj/1065880128_20131015134930_gpubox20_01.fits",
            "/home/chj/1065880128_20131015134930_gpubox19_01.fits",
            "/home/chj/1065880128_20131015134930_gpubox14_02.fits",
            "/home/chj/1065880128_20131015134930_gpubox15_02.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_ok());
        let (gpubox_batches, corr_format, num_batches) = result.unwrap();
        assert_eq!(corr_format, CorrelatorVersion::Legacy);
        assert_eq!(num_batches, 3);

        let expected_batches = vec![
            TempGPUBoxFile {
                filename: "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
                channel_identifier: 1,
                batch_number: 0,
            },
            TempGPUBoxFile {
                filename: "/home/chj/1065880128_20131015134930_gpubox02_00.fits",
                channel_identifier: 2,
                batch_number: 0,
            },
            TempGPUBoxFile {
                filename: "/home/chj/1065880128_20131015134930_gpubox19_01.fits",
                channel_identifier: 19,
                batch_number: 1,
            },
            TempGPUBoxFile {
                filename: "/home/chj/1065880128_20131015134930_gpubox20_01.fits",
                channel_identifier: 20,
                batch_number: 1,
            },
            TempGPUBoxFile {
                filename: "/home/chj/1065880128_20131015134930_gpubox14_02.fits",
                channel_identifier: 14,
                batch_number: 2,
            },
            TempGPUBoxFile {
                filename: "/home/chj/1065880128_20131015134930_gpubox15_02.fits",
                channel_identifier: 15,
                batch_number: 2,
            },
        ];

        assert_eq!(gpubox_batches, expected_batches);
    }

    #[test]
    fn test_determine_gpubox_batches_proper_format4() {
        let files = vec![
            "/home/chj/1065880128_20131015134929_gpubox02_00.fits",
            "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
            "/home/chj/1065880128_20131015134929_gpubox20_01.fits",
            "/home/chj/1065880128_20131015134930_gpubox19_01.fits",
            "/home/chj/1065880128_20131015134931_gpubox14_02.fits",
            "/home/chj/1065880128_20131015134930_gpubox15_02.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_ok());
        let (gpubox_batches, corr_format, num_batches) = result.unwrap();
        assert_eq!(corr_format, CorrelatorVersion::Legacy);
        assert_eq!(num_batches, 3);

        let expected_batches = vec![
            TempGPUBoxFile {
                filename: "/home/chj/1065880128_20131015134930_gpubox01_00.fits",
                channel_identifier: 1,
                batch_number: 0,
            },
            TempGPUBoxFile {
                filename: "/home/chj/1065880128_20131015134929_gpubox02_00.fits",
                channel_identifier: 2,
                batch_number: 0,
            },
            TempGPUBoxFile {
                filename: "/home/chj/1065880128_20131015134930_gpubox19_01.fits",
                channel_identifier: 19,
                batch_number: 1,
            },
            TempGPUBoxFile {
                filename: "/home/chj/1065880128_20131015134929_gpubox20_01.fits",
                channel_identifier: 20,
                batch_number: 1,
            },
            TempGPUBoxFile {
                filename: "/home/chj/1065880128_20131015134931_gpubox14_02.fits",
                channel_identifier: 14,
                batch_number: 2,
            },
            TempGPUBoxFile {
                filename: "/home/chj/1065880128_20131015134930_gpubox15_02.fits",
                channel_identifier: 15,
                batch_number: 2,
            },
        ];

        assert_eq!(gpubox_batches, expected_batches);
    }

    #[test]
    fn test_determine_gpubox_batches_invalid_filename() {
        let files = vec!["1065880128_20131015134930_gpubox0100.fits"];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }

    #[test]
    fn test_determine_gpubox_batches_invalid_filename2() {
        let files = vec!["1065880128x_20131015134930_gpubox01_00.fits"];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }

    #[test]
    fn test_determine_gpubox_batches_invalid_filename3() {
        let files = vec!["1065880128_920131015134930_gpubox01_00.fits"];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }

    #[test]
    fn test_determine_gpubox_batches_invalid_count() {
        // There are no gpubox files for batch "01".
        let files = vec![
            "1065880128_20131015134930_gpubox01_00.fits",
            "1065880128_20131015134930_gpubox15_02.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }

    #[test]
    fn test_determine_gpubox_batches_invalid_count2() {
        // There are not enough gpubox files for batch "02".
        let files = vec![
            "1065880128_20131015134930_gpubox01_00.fits",
            "1065880128_20131015134930_gpubox02_00.fits",
            "1065880128_20131015134930_gpubox01_01.fits",
            "1065880128_20131015134930_gpubox02_01.fits",
            "1065880128_20131015134930_gpubox15_02.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }

    #[test]
    fn test_determine_gpubox_batches_old_format() {
        let files = vec![
            "1065880128_20131015134930_gpubox01.fits",
            "1065880128_20131015134930_gpubox20.fits",
            "1065880128_20131015134930_gpubox15.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_ok());
        let (gpubox_batches, corr_format, num_batches) = result.unwrap();
        assert_eq!(corr_format, CorrelatorVersion::OldLegacy);
        assert_eq!(num_batches, 1);

        let expected_batches = vec![
            TempGPUBoxFile {
                filename: "1065880128_20131015134930_gpubox01.fits",
                channel_identifier: 1,
                batch_number: 0,
            },
            TempGPUBoxFile {
                filename: "1065880128_20131015134930_gpubox15.fits",
                channel_identifier: 15,
                batch_number: 0,
            },
            TempGPUBoxFile {
                filename: "1065880128_20131015134930_gpubox20.fits",
                channel_identifier: 20,
                batch_number: 0,
            },
        ];

        assert_eq!(gpubox_batches, expected_batches);
    }

    #[test]
    fn test_determine_gpubox_batches_new_format() {
        let files = vec![
            "1065880128_20131015134930_ch101_000.fits",
            "1065880128_20131015134930_ch102_000.fits",
            "1065880128_20131015135030_ch101_001.fits",
            "1065880128_20131015135030_ch102_001.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_ok());
        let (gpubox_batches, corr_format, num_batches) = result.unwrap();
        assert_eq!(corr_format, CorrelatorVersion::V2);
        assert_eq!(num_batches, 2);

        let expected_batches = vec![
            TempGPUBoxFile {
                filename: "1065880128_20131015134930_ch101_000.fits",
                channel_identifier: 101,
                batch_number: 0,
            },
            TempGPUBoxFile {
                filename: "1065880128_20131015134930_ch102_000.fits",
                channel_identifier: 102,
                batch_number: 0,
            },
            TempGPUBoxFile {
                filename: "1065880128_20131015135030_ch101_001.fits",
                channel_identifier: 101,
                batch_number: 1,
            },
            TempGPUBoxFile {
                filename: "1065880128_20131015135030_ch102_001.fits",
                channel_identifier: 102,
                batch_number: 1,
            },
        ];

        assert_eq!(gpubox_batches, expected_batches);
    }

    #[test]
    fn test_determine_gpubox_batches_mix() {
        let files = vec![
            "1065880128_20131015134930_gpubox01.fits",
            "1065880128_20131015134930_gpubox15_01.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_err());
    }

    #[test]
    fn test_determine_hdu_time_test1() {
        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        with_new_temp_fits_file("determine_hdu_time_test1.fits", |fptr| {
            let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

            // Write the TIME and MILLITIM keys. Key types must be i64 to get any
            // sort of sanity.
            hdu.write_key(fptr, "TIME", 1_434_494_061)
                .expect("Couldn't write key 'TIME'");
            hdu.write_key(fptr, "MILLITIM", 0)
                .expect("Couldn't write key 'MILLITIM'");

            let result = determine_hdu_time(fptr, &hdu);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 1_434_494_061_000);
        });
    }

    #[test]
    fn test_determine_hdu_time_test2() {
        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        with_new_temp_fits_file("determine_hdu_time_test2.fits", |fptr| {
            let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

            hdu.write_key(fptr, "TIME", 1_381_844_923)
                .expect("Couldn't write key 'TIME'");
            hdu.write_key(fptr, "MILLITIM", 500)
                .expect("Couldn't write key 'MILLITIM'");

            let result = determine_hdu_time(fptr, &hdu);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 1_381_844_923_500);
        });
    }

    #[test]
    fn test_determine_hdu_time_test3() {
        // Use the current UNIX time.
        let current = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Err(e) => panic!("Something is wrong with time on your system: {}", e),
            Ok(n) => n.as_secs(),
        };

        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        with_new_temp_fits_file("determine_hdu_time_test3.fits", |fptr| {
            let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

            hdu.write_key(fptr, "TIME", current)
                .expect("Couldn't write key 'TIME'");
            hdu.write_key(fptr, "MILLITIM", 500)
                .expect("Couldn't write key 'MILLITIM'");

            let result = determine_hdu_time(fptr, &hdu);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), current * 1000 + 500);
        });
    }

    #[test]
    fn test_map_unix_times_to_hdus_test() {
        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        with_new_temp_fits_file("map_unix_times_to_hdus_test.fits", |fptr| {
            let times: Vec<(u64, u64)> =
                vec![(1_381_844_923, 500), (1_381_844_924, 0), (1_381_844_950, 0)];
            let mut expected = BTreeMap::new();
            let image_description = ImageDescription {
                data_type: ImageType::Float,
                dimensions: &[100, 100],
            };
            for (i, (time, millitime)) in times.iter().enumerate() {
                let hdu = fptr
                    .create_image("EXTNAME".to_string(), &image_description)
                    .expect("Couldn't create image");
                hdu.write_key(fptr, "TIME", *time)
                    .expect("Couldn't write key 'TIME'");
                hdu.write_key(fptr, "MILLITIM", *millitime)
                    .expect("Couldn't write key 'MILLITIM'");

                expected.insert(time * 1000 + millitime, i + 1);
            }

            let result = map_unix_times_to_hdus(fptr, CorrelatorVersion::Legacy);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), expected);
        });
    }

    #[test]
    fn test_determine_obs_times_test_many_timesteps() {
        // Create two files, with mostly overlapping times, but also a little
        // dangling at the start and end.
        let common_times: Vec<u64> = vec![
            1_381_844_923_500,
            1_381_844_924_000,
            1_381_844_924_500,
            1_381_844_925_000,
            1_381_844_925_500,
        ];
        let integration_time_ms = 500;

        let mut input = BTreeMap::new();
        let mut new_time_tree = BTreeMap::new();
        new_time_tree.insert(0, (0, 1));
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
        input.insert(1_381_844_926_000, new_time_tree);

        let expected_start = *common_times.first().unwrap();
        let expected_end = *common_times.last().unwrap() + integration_time_ms;
        // Duration = common end - common start + integration time
        // == 1_381_844_925_500 - 1_381_844_923_500 + 500
        let expected_duration = 2500;

        let result = determine_obs_times(&input, integration_time_ms);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.start_millisec, expected_start);
        assert_eq!(result.end_millisec, expected_end);
        assert_eq!(result.duration_millisec, expected_duration);
    }

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

    #[test]
    fn test_validate_gpubox_metadata_correlator_version() {
        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        with_new_temp_fits_file(
            "test_validate_gpubox_metadata_correlator_version.fits",
            |fptr| {
                let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

                // This should succeed- LegacyOld should NOT have CORR_VER key
                assert!(validate_gpubox_metadata_correlator_version(
                    fptr,
                    &hdu,
                    &String::from("test_file.fits"),
                    CorrelatorVersion::OldLegacy,
                )
                .is_ok());

                // This should succeed- Legacy should NOT have CORR_VER key
                assert!(validate_gpubox_metadata_correlator_version(
                    fptr,
                    &hdu,
                    &String::from("test_file.fits"),
                    CorrelatorVersion::Legacy,
                )
                .is_ok());

                // This should fail- V2 needs CORR_VER key
                assert!(validate_gpubox_metadata_correlator_version(
                    fptr,
                    &hdu,
                    &String::from("test_file.fits"),
                    CorrelatorVersion::V2,
                )
                .is_err());

                // Now put in a corr version
                hdu.write_key(fptr, "CORR_VER", 2)
                    .expect("Couldn't write key 'CORR_VER'");

                // This should succeed- V2 should have CORR_VER key
                assert!(validate_gpubox_metadata_correlator_version(
                    fptr,
                    &hdu,
                    &String::from("test_file.fits"),
                    CorrelatorVersion::V2,
                )
                .is_ok());

                // This should fail- OldLegacy should NOT have CORR_VER key
                assert!(validate_gpubox_metadata_correlator_version(
                    fptr,
                    &hdu,
                    &String::from("test_file.fits"),
                    CorrelatorVersion::OldLegacy,
                )
                .is_err());

                // This should fail- Legacy should NOT have CORR_VER key
                assert!(validate_gpubox_metadata_correlator_version(
                    fptr,
                    &hdu,
                    &String::from("test_file.fits"),
                    CorrelatorVersion::Legacy,
                )
                .is_err());
            },
        );

        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        // This section tests CORR_VER where it is != 2
        with_new_temp_fits_file(
            "test_validate_gpubox_metadata_correlator_version.fits",
            |fptr| {
                let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

                // This should not succeed- CORR_VER key if it exists should be 2
                // CORR_VER did not exist in OldLegacy or Legacy correlator
                // Now put in a corr version
                hdu.write_key(fptr, "CORR_VER", 1)
                    .expect("Couldn't write key 'CORR_VER'");

                assert!(validate_gpubox_metadata_correlator_version(
                    fptr,
                    &hdu,
                    &String::from("test_file.fits"),
                    CorrelatorVersion::V2,
                )
                .is_err());
            },
        );
    }

    #[test]
    fn test_validate_gpubox_metadata_obsid() {
        // with_temp_file creates a temp dir and temp file, then removes them once out of scope
        with_new_temp_fits_file(
            "test_validate_gpubox_metadata_correlator_version.fits",
            |fptr| {
                let hdu = fptr.hdu(0).expect("Couldn't open HDU 0");

                // OBSID is not there, this should be an error
                assert!(validate_gpubox_metadata_obs_id(
                    fptr,
                    &hdu,
                    &String::from("test_file.fits"),
                    1_234_567_890,
                )
                .is_err());

                // Now add the key
                hdu.write_key(fptr, "OBSID", 1_234_567_890)
                    .expect("Couldn't write key 'OBSID'");

                // OBSID is there, but does not match metafits- this should be an error
                assert!(validate_gpubox_metadata_obs_id(
                    fptr,
                    &hdu,
                    &String::from("test_file.fits"),
                    2_345_678_901,
                )
                .is_err());

                // OBSID is there, and it matches
                assert!(validate_gpubox_metadata_obs_id(
                    fptr,
                    &hdu,
                    &String::from("test_file.fits"),
                    1_234_567_890,
                )
                .is_ok());
            },
        );
    }
}
*/
