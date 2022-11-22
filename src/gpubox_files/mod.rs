// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Functions for organising and checking the consistency of gpubox files.

pub mod error;

use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fmt;
use std::path::Path;

use fitsio::{hdu::FitsHdu, FitsFile};
use rayon::prelude::*;
use regex::Regex;

use crate::*;
pub use error::GpuboxError;

#[cfg(test)]
mod test;

/// This struct is used to return the common or common good timesteps and coarse channels
#[derive(Debug)]
pub(crate) struct ObsTimesAndChans {
    pub start_time_unix_ms: u64, // Start= start of first timestep
    pub end_time_unix_ms: u64,   // End  = start of last timestep + integration time
    pub duration_ms: u64,
    pub coarse_chan_identifiers: Vec<usize>, // Vector of Correlator Coarse Chan identifiers (gpubox number or rec chan number)
}

/// This represents one group of gpubox files with the same "batch" identitifer.
/// e.g. obsid_datetime_chan_batch
pub struct GpuBoxBatch {
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
pub struct GpuBoxFile {
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
///                                      Unix          Chan    Batch  Hdu
pub(crate) type GpuboxTimeMap = BTreeMap<u64, BTreeMap<usize, (usize, usize)>>;

/// A little struct to help us not get confused when dealing with the returned
/// values from complex functions.
pub(crate) struct GpuboxInfo {
    pub batches: Vec<GpuBoxBatch>,
    pub mwa_version: MWAVersion,
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
/// reflected in the returned `MWAVersion`.
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
    let (temp_gpuboxes, corr_format) = determine_gpubox_batches(gpubox_filenames)?;

    let time_map: GpuboxTimeMap = create_time_map(&temp_gpuboxes, corr_format)?;

    let mut batches = convert_temp_gpuboxes(temp_gpuboxes);

    // Determine the size of each gpubox's image on HDU 1. mwalib will throw an
    // error if this size is not consistent for all gpubox files.
    let mut hdu_size: Option<usize> = None;
    for b in &mut batches {
        for g in &mut b.gpubox_files {
            let mut fptr = fits_open!(&g.filename)?;

            // Check that there are some HDUs (apart from just the primary)
            // Assuming it does have some, open the first one
            let hdu = match fptr.iter().count() {
                1 => {
                    return Err(GpuboxError::NoDataHDUsInGpuboxFile {
                        gpubox_filename: g.filename.clone(),
                    })
                }
                _ => fits_open_hdu!(&mut fptr, 1)?,
            };

            let this_size = get_hdu_image_size!(&mut fptr, &hdu)?.iter().product();
            match hdu_size {
                None => hdu_size = Some(this_size),
                Some(s) => {
                    if s != this_size {
                        return Err(GpuboxError::UnequalHduSizes);
                    }
                }
            }

            // Do another check by looking in the header of each fits file and checking the mwa_version is correct
            let primary_hdu = fits_open_hdu!(&mut fptr, 0)?;
            validate_gpubox_metadata_mwa_version(
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
        mwa_version: corr_format,
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
///   `MWAVersion`.
///
///
fn determine_gpubox_batches<T: AsRef<Path>>(
    gpubox_filenames: &[T],
) -> Result<(Vec<TempGpuBoxFile>, MWAVersion), GpuboxError> {
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
                    None => format = Some(MWAVersion::CorrMWAXv2),
                    Some(MWAVersion::CorrMWAXv2) => (),
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
                        None => format = Some(MWAVersion::CorrLegacy),
                        Some(MWAVersion::CorrLegacy) => (),
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
                            None => format = Some(MWAVersion::CorrOldLegacy),
                            Some(MWAVersion::CorrOldLegacy) => (),
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

    // Ensure the output is properly sorted - each batch is sorted by batch
    // number, then channel identifier.
    temp_gpuboxes.sort_unstable_by_key(|g| (g.batch_number, g.channel_identifier));

    Ok((temp_gpuboxes, format.unwrap()))
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
    Ok(start_unix_time * 1000 + start_unix_millitime)
}

/// Iterate over each HDU of the given gpubox file, tracking which UNIX times
/// are associated with which HDU numbers.
///
///
/// # Arguments
///
/// * `gpubox_fptr` - A FitsFile reference to this gpubox file.
///
/// * `mwa_version` - enum telling us which correlator version the observation was created by.
///
///
/// # Returns
///
/// * A BTree representing time and hdu index this gpubox file.
///
///
fn map_unix_times_to_hdus(
    gpubox_fptr: &mut FitsFile,
    mwa_version: MWAVersion,
) -> Result<BTreeMap<u64, usize>, FitsError> {
    let mut map = BTreeMap::new();
    let last_hdu_index = gpubox_fptr.iter().count();
    // The new correlator has a "weights" HDU in each alternating HDU. Skip
    // those.
    let step_size = if mwa_version == MWAVersion::CorrMWAXv2 {
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
/// * `mwa_version` - enum telling us which correlator version the observation was created by.
///
///
/// # Returns
///
/// * A Result containing `Ok` or an `MwalibError` if it fails validation.
///
///
fn validate_gpubox_metadata_mwa_version(
    gpubox_fptr: &mut FitsFile,
    gpubox_primary_hdu: &FitsHdu,
    gpubox_filename: &str,
    mwa_version: MWAVersion,
) -> Result<(), GpuboxError> {
    // New correlator files include a version - check that it is present.
    // For pre v2, ensure the key isn't present
    let gpu_mwa_version: Option<u8> =
        get_optional_fits_key!(gpubox_fptr, gpubox_primary_hdu, "CORR_VER")?;

    match mwa_version {
        MWAVersion::CorrMWAXv2 => match gpu_mwa_version {
            None => Err(GpuboxError::MwaxCorrVerMissing(gpubox_filename.to_string())),
            Some(gpu_mwa_version_value) => match gpu_mwa_version_value {
                2 => Ok(()),
                _ => Err(GpuboxError::MwaxCorrVerMismatch(
                    gpubox_filename.to_string(),
                )),
            },
        },

        MWAVersion::CorrOldLegacy | MWAVersion::CorrLegacy => match gpu_mwa_version {
            None => Ok(()),
            Some(gpu_corr_version_value) => Err(GpuboxError::CorrVerMismatch {
                gpubox_filename: gpubox_filename.to_string(),
                gpu_corr_version_value,
            }),
        },
        _ => Err(GpuboxError::InvalidMwaVersion { mwa_version }),
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
/// * `mwa_version` - enum telling us which correlator version the observation was created by.
///
///
/// # Returns
///
/// * A Result containing the GPUBox Time Map or an error.
///
///
fn create_time_map(
    gpuboxes: &[TempGpuBoxFile],
    mwa_version: MWAVersion,
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
            if mwa_version == MWAVersion::CorrMWAXv2 {
                let v: u8 = get_required_fits_key!(&mut fptr, &hdu, "CORR_VER")?;
                if v != 2 {
                    return Err(GpuboxError::MwaxCorrVerMismatch(g.filename.to_string()));
                }
            }

            // Get the UNIX times from each of the HDUs of this `FitsFile`.
            map_unix_times_to_hdus(&mut fptr, mwa_version).map_err(GpuboxError::from)
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

/// Returns a vector of timestep indicies which exist in the GpuBoxTimeMap (i.e. the user has provided at least some data files for these timesteps)
///
/// # Arguments
///
/// * `gpubox_time_map` - BTree structure containing the map of what gpubox files and timesteps we were supplied by the client.
///
/// * `corr_timesteps` - Vector of Correlator Context TimeStep structs.
///
/// # Returns
///
/// * A vector of timestep indices for which at least some data files have been provided
///
///
pub(crate) fn populate_provided_timesteps(
    gpubox_time_map: &GpuboxTimeMap,
    corr_timesteps: &[TimeStep],
) -> Vec<usize> {
    // populate a vector with the indicies of corr_timesteps that correspond to the unix times which are in
    // the first level of the gpuboxtimemap. This represents all of the timesteps we have at least some data for
    let mut return_vec: Vec<usize> = gpubox_time_map
        .iter()
        .map(|t| {
            corr_timesteps
                .iter()
                .position(|v| v.unix_time_ms == *t.0)
                .unwrap()
        })
        .collect();

    // Ensure vector is sorted
    return_vec.sort_unstable();

    return_vec
}

/// Returns a vector of coarse chan indicies which exist in the GpuBoxTimeMap (i.e. the user has provided at least some data files for these coarse channels)
///
/// # Arguments
///
/// * `gpubox_time_map` - BTree structure containing the map of what gpubox files and timesteps we were supplied by the client.
///
/// * `corr_coarse_chans` - Vector of Correlator Context CoarseChannel structs.
///
/// # Returns
///
/// * A vector of coarse channel indices for which at least some data files have been provided
///
///
pub(crate) fn populate_provided_coarse_channels(
    gpubox_time_map: &GpuboxTimeMap,
    corr_coarse_chans: &[CoarseChannel],
) -> Vec<usize> {
    // Go through all timesteps in the GpuBoxTimeMap.
    // For each timestep get each coarse channel identifier and add it into the HashSet
    let chans: HashSet<usize> = gpubox_time_map
        .iter()
        .flat_map(|ts| ts.1.iter().map(|ch| *ch.0))
        .collect::<HashSet<usize>>();

    // We should now have a small HashSet of coarse channel identifiers
    // Get the index of each item in the hashset from the correlator coarse channels passed in and add that to the return vector
    let mut return_vec: Vec<usize> = chans
        .iter()
        .map(|c| {
            corr_coarse_chans
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
/// time:     0123456789abcdef
/// gpubox01: ###############
/// gpubox02:  #############
/// gpubox03: #####    ######
/// gpubox04:   ############
/// gpubox05: ###############
/// gpubox06:                #
/// gpubox07-24: <none>
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
/// * `gpubox_time_map` - BTree structure containing the map of what gpubox files and timesteps we were supplied by the client.
///
/// * `integration_time_ms` - Correlator dump time (so we know the gap between timesteps)
///
/// * `good_time_unix_time_ms` - Option- Some is the 'good' time (i.e. the first time which is not part of the quack time). None means that
///                              times during the quack time are ok to be included.
///
/// # Returns
///
/// * A Result which contains an Option containing a struct containing the start and end times based on what we actually got, so all coarse channels match, or None; or an Error.
///
///
pub(crate) fn determine_common_obs_times_and_chans(
    gpubox_time_map: &GpuboxTimeMap,
    integration_time_ms: u64,
    good_time_unix_time_ms: Option<u64>,
) -> Result<Option<ObsTimesAndChans>, GpuboxError> {
    // If we pass in Some(good_time_unix_time_ms) then restrict the gpubox time map to times AFTER the quack time
    let timemap = match good_time_unix_time_ms {
        Some(good_time) => gpubox_time_map
            .clone()
            .into_iter()
            .filter(|ts| ts.0 >= good_time)
            .collect(),

        None => gpubox_time_map.clone(),
    };

    // Go through all timesteps in the GpuBoxTimeMap.
    // For each timestep get each coarse channel identifier and add it into the HashSet, then dump them into a vector
    // get the length of the vector - we will use this to test each entry in the GpuboxTimeMap
    let max_chans = gpubox_time_map
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
    let common_start_unix_ms = first_ts.0;

    // In case there was only 1 timestep in the filtered timesteps, set the common end time now
    let mut common_end_unix_ms = common_start_unix_ms + integration_time_ms;

    // Iterate over the filtered timemap
    // Go to the next timestep unless:
    // * It is not contiguous with the previous
    // * It does not have that same max number of channels
    let mut prev_ts_unix_ms = common_start_unix_ms;
    loop {
        let next_item = filtered_timesteps.next();

        match next_item {
            Some(ts) => {
                // Check ts and prev ts are contiguous and channels match
                if (ts.0 == prev_ts_unix_ms + integration_time_ms)
                    && first_ts_chans.len() == ts.1.len()
                {
                    // Update the end time
                    common_end_unix_ms = ts.0 + integration_time_ms;
                    prev_ts_unix_ms = ts.0;
                } else {
                    break;
                }
            }
            None => break,
        }
    }

    Ok(Some(ObsTimesAndChans {
        start_time_unix_ms: common_start_unix_ms,
        end_time_unix_ms: common_end_unix_ms,
        duration_ms: common_end_unix_ms - common_start_unix_ms,
        coarse_chan_identifiers: first_ts_chans,
    }))
}

#[cfg(test)]
mod tests {}
