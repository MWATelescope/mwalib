// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Functions for organising and checking the consistency of gpubox files.
*/

pub mod error;

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use fitsio::{hdu::FitsHdu, FitsFile};
use rayon::prelude::*;
use regex::Regex;

use crate::*;
pub use error::GpuboxError;

#[derive(Debug)]
pub struct ObsTimes {
    pub start_millisec: u64, // Start= start of first timestep
    pub end_millisec: u64,   // End  = start of last timestep + integration time
    pub duration_millisec: u64,
}

/// This represents one group of gpubox files with the same "batch" identitifer.
/// e.g. obsid_datetime_chan_batch
pub struct GPUBoxBatch {
    pub batch_number: usize,           // 00,01,02..n
    pub gpubox_files: Vec<GPUBoxFile>, // Vector storing the details of each gpubox file in this batch
}

impl GPUBoxBatch {
    pub fn new(batch_number: usize) -> Self {
        Self {
            batch_number,
            gpubox_files: vec![],
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl fmt::Debug for GPUBoxBatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "batch_number={} gpubox_files={:?}",
            self.batch_number, self.gpubox_files,
        )
    }
}

/// This represents one gpubox file
pub struct GPUBoxFile {
    /// Filename of gpubox file
    pub filename: String,
    /// channel number (Legacy==gpubox host number 01..24; V2==receiver channel number 001..255)
    pub channel_identifier: usize,
}

#[cfg(not(tarpaulin_include))]
impl fmt::Debug for GPUBoxFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "filename={} channelidentifier={}",
            self.filename, self.channel_identifier,
        )
    }
}

impl std::cmp::PartialEq for GPUBoxBatch {
    fn eq(&self, other: &Self) -> bool {
        self.batch_number == other.batch_number && self.gpubox_files == other.gpubox_files
    }
}

impl std::cmp::PartialEq for GPUBoxFile {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename && self.channel_identifier == other.channel_identifier
    }
}

/// A temporary representation of a gpubox file
#[derive(Clone, Debug)]
struct TempGPUBoxFile<'a> {
    /// Filename of gpubox file
    filename: &'a str,
    /// Channel number (Legacy==gpubox host number 01..24; V2==receiver channel number 001..255)
    channel_identifier: usize,
    /// Batch number (00,01,02..n)
    batch_number: usize,
}

impl<'a> std::cmp::PartialEq for TempGPUBoxFile<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
            && self.channel_identifier == other.channel_identifier
            && self.batch_number == other.batch_number
    }
}

lazy_static::lazy_static! {
    static ref RE_MWAX: Regex =
        Regex::new(r"\d{10}_\d{8}(.)?\d{6}_ch(?P<channel>\d{3})_(?P<batch>\d{3}).fits").unwrap();
    static ref RE_LEGACY_BATCH: Regex =
        Regex::new(r"\d{10}_\d{14}_gpubox(?P<band>\d{2})_(?P<batch>\d{2}).fits").unwrap();
    static ref RE_OLD_LEGACY_FORMAT: Regex =
        Regex::new(r"\d{10}_\d{14}_gpubox(?P<band>\d{2}).fits").unwrap();
    static ref RE_BAND: Regex = Regex::new(r"\d{10}_\d{14}_(ch|gpubox)(?P<band>\d+)").unwrap();
}

/// Convert `Vec<TempGPUBoxFile>` to `Vec<GPUBoxBatch>`. This requires the fits
/// files to actually be present, as `GPUBoxFile`s need an open fits file
/// handle.
///
/// Fail if
///
/// * no files were supplied;
/// * the fits files specified by the `TempGPUBoxFile`s can't be opened.
///
///
/// # Arguments
///
/// * `temp_gpuboxes` - A vector of `TempGPUBoxFile` to be converted.
///
///
/// # Returns
///
/// * A Result containing a vector of `GPUBoxBatch`.
///
///
fn convert_temp_gpuboxes(
    temp_gpuboxes: Vec<TempGPUBoxFile>,
) -> Result<Vec<GPUBoxBatch>, FitsError> {
    // unwrap is safe as a check is performed above to ensure that there are
    // some files present.
    let num_batches = temp_gpuboxes.iter().map(|g| g.batch_number).max().unwrap() + 1;
    let mut gpubox_batches: Vec<GPUBoxBatch> = Vec::with_capacity(num_batches);
    for b in 0..num_batches {
        gpubox_batches.push(GPUBoxBatch::new(b));
    }

    for temp_g in temp_gpuboxes.into_iter() {
        let g = GPUBoxFile {
            filename: temp_g.filename.to_string(),
            channel_identifier: temp_g.channel_identifier,
        };
        gpubox_batches[temp_g.batch_number].gpubox_files.push(g);
    }

    // Ensure the output is properly sorted - each batch is sorted by
    // channel_identifier.
    for v in &mut gpubox_batches {
        v.gpubox_files
            .sort_unstable_by(|a, b| a.channel_identifier.cmp(&b.channel_identifier));
    }

    // Sort the batches by batch number
    gpubox_batches.sort_by_key(|b| b.batch_number);

    Ok(gpubox_batches)
}

/// A type alias for a horrible type:
/// `BTreeMap<u64, BTreeMap<usize, (usize, usize)>>`
///
/// The outer-most keys are UNIX times in milliseconds, which correspond to the
/// unique times available to HDU files in supplied gpubox files. Each of these
/// keys is associated with a tree; the keys of these trees are the gpubox
/// coarse-channel numbers, which then refer to gpubox batch numbers and HDU
/// indices.
pub type GpuboxTimeMap = BTreeMap<u64, BTreeMap<usize, (usize, usize)>>;

/// A little struct to help us not get confused when dealing with the returned
/// values from complex functions.
pub struct GpuboxInfo {
    pub batches: Vec<GPUBoxBatch>,
    pub corr_format: CorrelatorVersion,
    pub time_map: GpuboxTimeMap,
    pub hdu_size: usize,
}

/// This function unpacks the metadata associated with input gpubox files. The
/// input filenames are grouped into into batches. A "gpubox batch" refers to
/// the number XX in a gpubox filename
/// (e.g. `1065880128_20131015134930_gpubox01_XX.fits`). Some older files might
/// have a "batchless" format
/// (e.g. `1065880128_20131015134930_gpubox01.fits`). These details are
/// reflected in the returned `CorrelatorVersion`.
///
/// Fail if
///
/// * no files were supplied;
/// * there is a mixture of the types of gpubox files supplied (e.g. different
///   correlator versions);
/// * a gpubox filename's structure could not be identified;
/// * the gpubox batch numbers are not contiguous;
/// * the number of files in each batch is not equal;
/// * MWAX gpubox files don't have a CORR_VER key in HDU 0, or it is not equal
///   to 2;
/// * the amount of data in each HDU is not equal.
///
///
/// # Arguments
///
/// * `gpubox_filenames` - A vector or slice of strings or references to strings
///                        containing all of the gpubox filenames provided by the client.
///
///
/// # Returns
///
/// * A Result containing a vector of GPUBoxBatch structs, the MWA Correlator
///   version, the UNIX times paired with gpubox HDU numbers, and the amount of
///   data in each HDU.
///
///
pub fn examine_gpubox_files<T: AsRef<Path>>(
    gpubox_filenames: &[T],
) -> Result<GpuboxInfo, GpuboxError> {
    let (temp_gpuboxes, corr_format, _) = determine_gpubox_batches(gpubox_filenames)?;

    let time_map = create_time_map(&temp_gpuboxes, corr_format)?;

    let mut batches = convert_temp_gpuboxes(temp_gpuboxes)?;

    // Determine the size of each gpubox's image on HDU 1. mwalib will throw an
    // error if this size is not consistent for all gpubox files.
    let mut hdu_size: Option<usize> = None;
    for b in &mut batches {
        for g in &mut b.gpubox_files {
            let mut fptr = fits_open!(&g.filename)?;

            let hdu = fits_open_hdu!(&mut fptr, 1)?;
            let this_size = get_hdu_image_size!(&mut fptr, &hdu)?.iter().product();
            match hdu_size {
                None => hdu_size = Some(this_size),
                Some(s) => {
                    if s != this_size {
                        return Err(GpuboxError::UnequalHduSizes);
                    }
                }
            }
        }
    }

    // `determine_gpubox_batches` fails if no gpubox files are supplied, so it
    // is safe to unwrap hdu_size.
    Ok(GpuboxInfo {
        batches,
        corr_format,
        time_map,
        hdu_size: hdu_size.unwrap(),
    })
}

/// Group input gpubox files into batches. A "gpubox batch" refers to the number
/// XX in a gpubox filename
/// (e.g. `1065880128_20131015134930_gpubox01_XX.fits`). Some older files might
/// have a "batchless" format (e.g. `1065880128_20131015134930_gpubox01.fits`).
///
///
/// Fail if
///
/// * no files were supplied;
/// * there is a mixture of the types of gpubox files supplied (e.g. different correlator
///   versions);
/// * a gpubox filename's structure could not be identified;
/// * the gpubox batch numbers are not contiguous;
/// * the number of files in each batch is not equal;
///
///
/// # Arguments
///
/// * `gpubox_filenames` - A vector or slice of strings or references to strings containing
///                        all of the gpubox filenames provided by the client.
///
///
/// # Returns
///
/// * A Result containing a vector of `TempGPUBoxFile` structs as well as a
///   `CorrelatorVersion`, the number of GPUBoxes supplied, and the number of
///   gpubox batches.
///
///
fn determine_gpubox_batches<T: AsRef<Path>>(
    gpubox_filenames: &[T],
) -> Result<(Vec<TempGPUBoxFile>, CorrelatorVersion, usize), GpuboxError> {
    if gpubox_filenames.is_empty() {
        return Err(GpuboxError::NoGpuboxes);
    }
    let mut format = None;
    let mut temp_gpuboxes: Vec<TempGPUBoxFile> = Vec::with_capacity(gpubox_filenames.len());

    for g_path in gpubox_filenames {
        // So that we can pass along useful error messages, convert the input
        // filename type to a string slice. This will fail if the filename is
        // not UTF-8 compliant, but, I don't think cfitsio will work in that
        // case anyway.
        let g = g_path
            .as_ref()
            .to_str()
            .expect("gpubox filename is not UTF-8 compliant");
        match RE_MWAX.captures(g) {
            Some(caps) => {
                // Check if we've already matched any files as being the old
                // format. If so, then we've got a mix, and we should exit
                // early.
                match format {
                    None => format = Some(CorrelatorVersion::V2),
                    Some(CorrelatorVersion::V2) => (),
                    _ => return Err(GpuboxError::Mixture),
                }

                // The following unwraps are safe, because the regex wouldn't
                // work if they couldn't be parsed into ints.
                temp_gpuboxes.push(TempGPUBoxFile {
                    filename: g,
                    channel_identifier: caps["channel"].parse().unwrap(),
                    batch_number: caps["batch"].parse().unwrap(),
                });
            }

            // Try to match the legacy format.
            None => match RE_LEGACY_BATCH.captures(g) {
                Some(caps) => {
                    match format {
                        None => format = Some(CorrelatorVersion::Legacy),
                        Some(CorrelatorVersion::Legacy) => (),
                        _ => return Err(GpuboxError::Mixture),
                    }

                    temp_gpuboxes.push(TempGPUBoxFile {
                        filename: g,
                        channel_identifier: caps["band"].parse().unwrap(),
                        batch_number: caps["batch"].parse().unwrap(),
                    });
                }

                // Try to match the old legacy format.
                None => match RE_OLD_LEGACY_FORMAT.captures(g) {
                    Some(caps) => {
                        match format {
                            None => format = Some(CorrelatorVersion::OldLegacy),
                            Some(CorrelatorVersion::OldLegacy) => (),
                            _ => return Err(GpuboxError::Mixture),
                        }

                        temp_gpuboxes.push(TempGPUBoxFile {
                            filename: g,
                            channel_identifier: caps["band"].parse().unwrap(),
                            // There's only one batch.
                            batch_number: 0,
                        });
                    }
                    None => return Err(GpuboxError::Unrecognised(g.to_string())),
                },
            },
        }
    }

    // Check batches are contiguous and have equal numbers of files.
    let mut batches_and_files: BTreeMap<usize, u8> = BTreeMap::new();
    for gpubox in &temp_gpuboxes {
        *batches_and_files.entry(gpubox.batch_number).or_insert(0) += 1;
    }

    let mut file_count: Option<u8> = None;
    for (i, (batch_num, num_files)) in batches_and_files.iter().enumerate() {
        if i != *batch_num {
            return Err(GpuboxError::BatchMissing {
                expected: i,
                got: *batch_num,
            });
        }

        match file_count {
            None => file_count = Some(*num_files),
            Some(c) => {
                if c != *num_files {
                    return Err(GpuboxError::UnevenCountInBatches {
                        expected: c,
                        got: *num_files,
                    });
                }
            }
        }
    }

    // Ensure the output is properly sorted - each batch is sorted by batch
    // number, then channel identifier.
    temp_gpuboxes.sort_unstable_by_key(|g| (g.batch_number, g.channel_identifier));

    Ok((temp_gpuboxes, format.unwrap(), batches_and_files.len()))
}

/// Given a FITS file pointer and HDU, determine the time in units of
/// milliseconds.
///
///
/// # Arguments
///
/// * `gpubox_fptr` - A FitsFile reference to this gpubox file.
///
/// * `metafits_hdu_fptr` - A reference to the primary HDU in the metafits file
///                         where we read keyword/value pairs.
///
///
/// # Returns
///
/// * A Result containing the full start unix time (in milliseconds) or an error.
///
///
pub fn determine_hdu_time(
    gpubox_fptr: &mut FitsFile,
    metafits_hdu_fptr: &FitsHdu,
) -> Result<u64, FitsError> {
    let start_unix_time: u64 = get_required_fits_key!(gpubox_fptr, metafits_hdu_fptr, "TIME")?;
    let start_unix_millitime: u64 =
        get_required_fits_key!(gpubox_fptr, metafits_hdu_fptr, "MILLITIM")?;
    Ok((start_unix_time * 1000 + start_unix_millitime) as u64)
}

/// Iterate over each HDU of the given gpubox file, tracking which UNIX times
/// are associated with which HDU numbers.
///
///
/// # Arguments
///
/// * `gpubox_fptr` - A FitsFile reference to this gpubox file.
///
/// * `correlator_version` - enum telling us which correlator version the observation was created by.
///
///
/// # Returns
///
/// * A BTree representing time and hdu index this gpubox file.
///
///
pub fn map_unix_times_to_hdus(
    gpubox_fptr: &mut FitsFile,
    correlator_version: CorrelatorVersion,
) -> Result<BTreeMap<u64, usize>, FitsError> {
    let mut map = BTreeMap::new();
    let last_hdu_index = gpubox_fptr.iter().count();
    // The new correlator has a "weights" HDU in each alternating HDU. Skip
    // those.
    let step_size = if correlator_version == CorrelatorVersion::V2 {
        2
    } else {
        1
    };
    // Ignore the first HDU in all gpubox files; it contains only a little
    // metadata.
    for hdu_index in (1..last_hdu_index).step_by(step_size) {
        let hdu = fits_open_hdu!(gpubox_fptr, hdu_index)?;
        let time = determine_hdu_time(gpubox_fptr, &hdu)?;
        map.insert(time, hdu_index);
    }

    Ok(map)
}

/// Validate that the correlator version we worked out from the filename does not contradict
/// the CORR_VER key from MWAX files or absence of that key for legacy correlator.
///
///
/// # Arguments
///
/// * `gpubox_fptr` - A FitsFile reference to this gpubox file.
///
/// * `gpubox_primary_hdu` - The primary HDU of the gpubox file.
///
/// * `gpubox_filename` - The filename of the gpubox file being validated.
///
/// * `correlator_version` - enum telling us which correlator version the observation was created by.
///
///
/// # Returns
///
/// * A Result containing `Ok` or an `MwalibError` if it fails validation.
///
///
pub fn validate_gpubox_metadata_correlator_version(
    gpubox_fptr: &mut FitsFile,
    gpubox_primary_hdu: &FitsHdu,
    gpubox_filename: &str,
    correlator_version: CorrelatorVersion,
) -> Result<(), GpuboxError> {
    // New correlator files include a version - check that it is present.
    // For pre v2, ensure the key isn't present
    let gpu_corr_version: Option<u8> =
        get_optional_fits_key!(gpubox_fptr, &gpubox_primary_hdu, "CORR_VER")?;

    match correlator_version {
        CorrelatorVersion::V2 => match gpu_corr_version {
            None => Err(GpuboxError::MWAXCorrVerMissing(gpubox_filename.to_string())),
            Some(gpu_corr_version_value) => match gpu_corr_version_value {
                2 => Ok(()),
                _ => Err(GpuboxError::MWAXCorrVerMismatch(
                    gpubox_filename.to_string(),
                )),
            },
        },

        CorrelatorVersion::OldLegacy | CorrelatorVersion::Legacy => match gpu_corr_version {
            None => Ok(()),
            Some(gpu_corr_version_value) => Err(GpuboxError::CorrVerMismatch {
                gpubox_filename: gpubox_filename.to_string(),
                gpu_corr_version_value,
            }),
        },
    }
}

/// Validate that the obsid we got from the metafits does not contradict
/// the GPSTIME key (obsid) from gpubox files.
///
///
/// # Arguments
///
/// * `gpubox_fptr` - A FitsFile reference to this gpubox file.
///
/// * `gpubox_primary_hdu` - The primary HDU of the gpubox file.
///
/// * `gpubox_filename` - The filename of the gpubox file being validated.
///
/// * `metafits_obsid` - Obsid as determined by reading the metafits.
///
///
/// # Returns
///
/// * A Result containing `Ok` or an `MwalibError` if it fails validation.
///
///
pub fn validate_gpubox_metadata_obs_id(
    gpubox_fptr: &mut FitsFile,
    gpubox_primary_hdu: &FitsHdu,
    gpubox_filename: &str,
    metafits_obsid: u32,
) -> Result<(), GpuboxError> {
    // Get the OBSID- if not present, this is probably not an MWA fits file!
    let gpu_obs_id: u32 = match get_required_fits_key!(gpubox_fptr, gpubox_primary_hdu, "OBSID") {
        Ok(o) => o,
        Err(_) => return Err(GpuboxError::MissingObsid(gpubox_filename.to_string())),
    };

    if gpu_obs_id != metafits_obsid {
        Err(GpuboxError::ObsidMismatch {
            obsid: metafits_obsid,
            gpubox_filename: gpubox_filename.to_string(),
            gpubox_obsid: gpu_obs_id,
        })
    } else {
        Ok(())
    }
}

/// Returns a BTree structure consisting of:
/// BTree of timesteps. Each timestep is a BTree for a course channel.
/// Each coarse channel then contains the batch number and hdu index.
///
/// # Arguments
///
/// * `gpubox_batches` - vector of structs describing each gpubox "batch"
///
/// * `correlator_version` - enum telling us which correlator version the observation was created by.
///
///
/// # Returns
///
/// * A Result containing the GPUBox Time Map or an error.
///
///
fn create_time_map(
    gpuboxes: &[TempGPUBoxFile],
    correlator_version: CorrelatorVersion,
) -> Result<GpuboxTimeMap, GpuboxError> {
    // Ugly hack to open up all the HDUs of the gpubox files in parallel. We
    // can't do this over the `GPUBoxBatch` or `GPUBoxFile` structs because they
    // contain the `FitsFile` struct, which does not implement the `Send`
    // trait. `ThreadsafeFitsFile` does contain this, but does not allow
    // iteration. It seems like the smaller evil is to just iterate over the
    // filenames here and get the relevant info out of the HDUs before things
    // get too complicated elsewhere.

    // In parallel, open up all the fits files and get their HDU times. rayon
    // preserves the order of the input arguments, so there is no need to keep
    // the temporary gpubox files along with their times. In any case, handling
    // that would be difficult!
    let maps = gpuboxes
        .into_par_iter()
        .map(|g| {
            let mut fptr = fits_open!(&g.filename)?;
            let hdu = fits_open_hdu!(&mut fptr, 0)?;

            // New correlator files include a version - check that it is present.
            if correlator_version == CorrelatorVersion::V2 {
                let v: u8 = get_required_fits_key!(&mut fptr, &hdu, "CORR_VER")?;
                if v != 2 {
                    return Err(GpuboxError::MWAXCorrVerMismatch(g.filename.to_string()));
                }
            }

            // Get the UNIX times from each of the HDUs of this `FitsFile`.
            map_unix_times_to_hdus(&mut fptr, correlator_version).map_err(GpuboxError::from)
        })
        .collect::<Vec<Result<BTreeMap<u64, usize>, GpuboxError>>>();

    // Collapse all of the gpubox time maps into a single map.
    let mut gpubox_time_map = BTreeMap::new();
    for (map_maybe_error, gpubox) in maps.into_iter().zip(gpuboxes.iter()) {
        let map = map_maybe_error?;
        for (time, hdu_index) in map {
            gpubox_time_map
                .entry(time)
                .or_insert_with(BTreeMap::new)
                .entry(gpubox.channel_identifier)
                .or_insert((gpubox.batch_number, hdu_index));
        }
    }

    Ok(gpubox_time_map)
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
    gpubox_time_map: &BTreeMap<u64, BTreeMap<usize, (usize, usize)>>,
    integration_time_ms: u64,
) -> Result<ObsTimes, GpuboxError> {
    // Find the maximum number of gpubox files, and assume that this is the
    // total number of input gpubox files.
    let size = match gpubox_time_map.iter().map(|(_, submap)| submap.len()).max() {
        Some(m) => m,
        None => return Err(GpuboxError::EmptyBTreeMap),
    };

    // Filter the first elements that don't satisfy `submap.len() == size`. The
    // first and last of the submaps that satisfy this condition are the proper
    // start and end of the observation.

    let mut i = gpubox_time_map
        .iter()
        .filter(|(_, submap)| submap.len() == size);
    // unwrap is safe because an empty map is checked above.
    let proper_start_millisec = i.next().map(|(time, _)| *time).unwrap();
    let proper_end_millisec = match i.last().map(|(time, _)| *time) {
        Some(s) => s,
        None => {
            // Looks like we only have 1 hdu, so end
            proper_start_millisec
        }
    } + integration_time_ms;

    Ok(ObsTimes {
        start_millisec: proper_start_millisec,
        end_millisec: proper_end_millisec,
        duration_millisec: proper_end_millisec - proper_start_millisec,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::misc::*;
    use fitsio::images::{ImageDescription, ImageType};
    use std::time::SystemTime;

    #[test]
    fn test_determine_gpubox_batches_proper_format() {
        let files = vec![
            "1065880128_20131015134930_gpubox20_01.fits",
            "1065880128_20131015134930_gpubox01_00.fits",
            "1065880128_20131015134930_gpubox15_02.fits",
        ];
        let result = determine_gpubox_batches(&files);
        assert!(result.is_ok());
        let (temp_gpuboxes, corr_format, num_batches) = result.unwrap();
        assert_eq!(corr_format, CorrelatorVersion::Legacy);
        assert_eq!(num_batches, 3);

        let expected_gpuboxes = vec![
            TempGPUBoxFile {
                filename: "1065880128_20131015134930_gpubox01_00.fits",
                channel_identifier: 1,
                batch_number: 0,
            },
            TempGPUBoxFile {
                filename: "1065880128_20131015134930_gpubox20_01.fits",
                channel_identifier: 20,
                batch_number: 1,
            },
            TempGPUBoxFile {
                filename: "1065880128_20131015134930_gpubox15_02.fits",
                channel_identifier: 15,
                batch_number: 2,
            },
        ];

        assert_eq!(temp_gpuboxes, expected_gpuboxes);
    }

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
