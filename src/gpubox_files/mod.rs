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

#[cfg(test)]
mod test;

#[derive(Debug)]
pub(crate) struct ObsTimes {
    pub start_millisec: u64, // Start= start of first timestep
    pub end_millisec: u64,   // End  = start of last timestep + integration time
    pub duration_millisec: u64,
}

/// This represents one group of gpubox files with the same "batch" identitifer.
/// e.g. obsid_datetime_chan_batch
pub(crate) struct GpuBoxBatch {
    pub batch_number: usize,           // 00,01,02..n
    pub gpubox_files: Vec<GpuBoxFile>, // Vector storing the details of each gpubox file in this batch
}

impl GpuBoxBatch {
    pub fn new(batch_number: usize) -> Self {
        Self {
            batch_number,
            gpubox_files: vec![],
        }
    }
}

impl fmt::Debug for GpuBoxBatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "batch_number={} gpubox_files={:?}",
            self.batch_number, self.gpubox_files,
        )
    }
}

/// This represents one gpubox file
pub(crate) struct GpuBoxFile {
    /// Filename of gpubox file
    pub filename: String,
    /// channel number (Legacy==gpubox host number 01..24; V2==receiver channel number 001..255)
    pub channel_identifier: usize,
}

impl fmt::Debug for GpuBoxFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "filename={} channelidentifier={}",
            self.filename, self.channel_identifier,
        )
    }
}

impl std::cmp::PartialEq for GpuBoxBatch {
    fn eq(&self, other: &Self) -> bool {
        self.batch_number == other.batch_number && self.gpubox_files == other.gpubox_files
    }
}

impl std::cmp::PartialEq for GpuBoxFile {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename && self.channel_identifier == other.channel_identifier
    }
}

/// A temporary representation of a gpubox file
#[derive(Clone, Debug)]
struct TempGpuBoxFile<'a> {
    /// Filename of gpubox file
    filename: &'a str,
    /// Channel number (Legacy==gpubox host number 01..24; V2==receiver channel number 001..255)
    channel_identifier: usize,
    /// Batch number (00,01,02..n)
    batch_number: usize,
}

impl<'a> std::cmp::PartialEq for TempGpuBoxFile<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
            && self.channel_identifier == other.channel_identifier
            && self.batch_number == other.batch_number
    }
}

lazy_static::lazy_static! {
    // MWAX: 1234567890_1234567890123_ch123_123.fits
    //          obsid   datetime      chan  batch
    static ref RE_MWAX: Regex =
        Regex::new(r"\d{10}_\d{8}(.)?\d{6}_ch(?P<channel>\d{3})_(?P<batch>\d{3}).fits").unwrap();
    // Legacy MWA: 1234567890_1234567890123_gpubox12_12.fits
    //                 obsid     datetime       chan batch
    static ref RE_LEGACY_BATCH: Regex =
        Regex::new(r"\d{10}_\d{14}_gpubox(?P<band>\d{2})_(?P<batch>\d{2}).fits").unwrap();
    // Old Legacy MWA: 1234567890_1234567890123_gpubox12.fits
    //                    obsid      datetime        chan
    static ref RE_OLD_LEGACY_FORMAT: Regex =
        Regex::new(r"\d{10}_\d{14}_gpubox(?P<band>\d{2}).fits").unwrap();
    static ref RE_BAND: Regex = Regex::new(r"\d{10}_\d{14}_(ch|gpubox)(?P<band>\d+)").unwrap();
}

/// A type alias for a horrible type:
/// `BTreeMap<u64, BTreeMap<usize, (usize, usize)>>`
///
/// The outer-most keys are UNIX times in milliseconds, which correspond to the
/// unique times available to HDU files in supplied gpubox files. Each of these
/// keys is associated with a tree; the keys of these trees are the gpubox
/// coarse-channel numbers, which then refer to gpubox batch numbers and HDU
/// indices.
pub(crate) type GpuboxTimeMap = BTreeMap<u64, BTreeMap<usize, (usize, usize)>>;

/// A little struct to help us not get confused when dealing with the returned
/// values from complex functions.
pub(crate) struct GpuboxInfo {
    pub batches: Vec<GpuBoxBatch>,
    pub corr_format: CorrelatorVersion,
    pub time_map: GpuboxTimeMap,
    pub hdu_size: usize,
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
fn convert_temp_gpuboxes(temp_gpuboxes: Vec<TempGpuBoxFile>) -> Vec<GpuBoxBatch> {
    // unwrap is safe as a check is performed above to ensure that there are
    // some files present.
    let num_batches = temp_gpuboxes.iter().map(|g| g.batch_number).max().unwrap() + 1;
    let mut gpubox_batches: Vec<GpuBoxBatch> = Vec::with_capacity(num_batches);
    for b in 0..num_batches {
        gpubox_batches.push(GpuBoxBatch::new(b));
    }

    for temp_g in temp_gpuboxes.into_iter() {
        let g = GpuBoxFile {
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

    gpubox_batches
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
/// * `metafits_obs_id` - The obs_id reported from the metafits file primary HDU
///
/// # Returns
///
/// * A Result containing a vector of GPUBoxBatch structs, the MWA Correlator
///   version, the UNIX times paired with gpubox HDU numbers, and the amount of
///   data in each HDU.
///
///
pub(crate) fn examine_gpubox_files<T: AsRef<Path>>(
    gpubox_filenames: &[T],
    metafits_obs_id: u32,
) -> Result<GpuboxInfo, GpuboxError> {
    let (temp_gpuboxes, corr_format, _) = determine_gpubox_batches(gpubox_filenames)?;

    let time_map = create_time_map(&temp_gpuboxes, corr_format)?;

    let mut batches = convert_temp_gpuboxes(temp_gpuboxes);

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

            // Do another check by looking in the header of each fits file and checking the corr_version is correct
            let primary_hdu = fits_open_hdu!(&mut fptr, 0)?;
            validate_gpubox_metadata_correlator_version(
                &mut fptr,
                &primary_hdu,
                &g.filename,
                corr_format,
            )?;

            // Do another check to ensure the obsid in the metafits matches that in the gpubox files
            validate_gpubox_metadata_obs_id(&mut fptr, &primary_hdu, &g.filename, metafits_obs_id)?;
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
) -> Result<(Vec<TempGpuBoxFile>, CorrelatorVersion, usize), GpuboxError> {
    if gpubox_filenames.is_empty() {
        return Err(GpuboxError::NoGpuboxes);
    }
    let mut format = None;
    let mut temp_gpuboxes: Vec<TempGpuBoxFile> = Vec::with_capacity(gpubox_filenames.len());

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
                temp_gpuboxes.push(TempGpuBoxFile {
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

                    temp_gpuboxes.push(TempGpuBoxFile {
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

                        temp_gpuboxes.push(TempGpuBoxFile {
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
/// * `gpubox_hdu_fptr` - A reference to the HDU we are finding the time of.
///
///
/// # Returns
///
/// * A Result containing the full start unix time (in milliseconds) or an error.
///
///
fn determine_hdu_time(
    gpubox_fptr: &mut FitsFile,
    gpubox_hdu_fptr: &FitsHdu,
) -> Result<u64, FitsError> {
    let start_unix_time: u64 = get_required_fits_key!(gpubox_fptr, gpubox_hdu_fptr, "TIME")?;
    let start_unix_millitime: u64 =
        get_required_fits_key!(gpubox_fptr, gpubox_hdu_fptr, "MILLITIM")?;
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
fn map_unix_times_to_hdus(
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
fn validate_gpubox_metadata_correlator_version(
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
            None => Err(GpuboxError::MwaxCorrVerMissing(gpubox_filename.to_string())),
            Some(gpu_corr_version_value) => match gpu_corr_version_value {
                2 => Ok(()),
                _ => Err(GpuboxError::MwaxCorrVerMismatch(
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
fn validate_gpubox_metadata_obs_id(
    gpubox_fptr: &mut FitsFile,
    gpubox_primary_hdu: &FitsHdu,
    gpubox_filename: &str,
    metafits_obs_id: u32,
) -> Result<(), GpuboxError> {
    // Get the OBSID- if not present, this is probably not an MWA fits file!
    let gpu_obs_id: u32 = match get_required_fits_key!(gpubox_fptr, gpubox_primary_hdu, "OBSID") {
        Ok(o) => o,
        Err(_) => return Err(GpuboxError::MissingObsid(gpubox_filename.to_string())),
    };

    if gpu_obs_id != metafits_obs_id {
        Err(GpuboxError::ObsidMismatch {
            obsid: metafits_obs_id,
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
    gpuboxes: &[TempGpuBoxFile],
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
                    return Err(GpuboxError::MwaxCorrVerMismatch(g.filename.to_string()));
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
pub(crate) fn determine_obs_times(
    gpubox_time_map: &GpuboxTimeMap,
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
mod tests {}
