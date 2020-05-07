// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Functions for organising and checking the consistency of gpubox files.
*/
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Debug;
use std::string::ToString;

use fitsio::{hdu::FitsHdu, FitsFile};
use regex::Regex;

use crate::fits_read::*;
use crate::*;

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

#[cfg_attr(tarpaulin, skip)]
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
    pub filename: String,          // Filename of gpubox file
    pub channel_identifier: usize, // channel number (Legacy==gpubox host number 01..24; V2==receiver channel number 001..255)
    pub fptr: Option<FitsFile>,    // Pointer to fits file
}

impl GPUBoxFile {
    pub fn new(filename: String, channel_identifier: usize) -> Self {
        Self {
            filename,
            channel_identifier,
            fptr: None,
        }
    }
}

#[cfg_attr(tarpaulin, skip)]
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

lazy_static::lazy_static! {
    static ref RE_MWAX: Regex =
        Regex::new(r"\d{10}_\d{8}(.)?\d{6}_ch(?P<channel>\d{3})_(?P<batch>\d{3}).fits").unwrap();
    static ref RE_LEGACY_BATCH: Regex =
        Regex::new(r"\d{10}_\d{14}_gpubox(?P<band>\d{2})_(?P<batch>\d{2}).fits").unwrap();
    static ref RE_OLD_LEGACY_FORMAT: Regex =
        Regex::new(r"\d{10}_\d{14}_gpubox(?P<band>\d{2}).fits").unwrap();
    static ref RE_BAND: Regex = Regex::new(r"\d{10}_\d{14}_(ch|gpubox)(?P<band>\d+)").unwrap();
}

/// Group input gpubox files into a vector of vectors containing their
/// batches. A "gpubox batch" refers to the number XX in a gpubox filename
/// (e.g. `1065880128_20131015134930_gpubox01_XX.fits`). Fail if the number of
/// files in each batch is not equal.
///
/// Some older files might have a "batchless" format
/// (e.g. `1065880128_20131015134930_gpubox01.fits`); in this case, this
/// function will just return one sub-vector for one batch.
/// From a path to a metafits file and paths to gpubox files, create an `mwalibContext`.
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
/// * A Result containing a vector of GPUBoxBatch structs as well as the Correlator version
///   (which is based on the filename ONLY - we check this in `create_time_map()`) and the
///   number of GPUBoxes supplied.
///
///
pub fn determine_gpubox_batches<T: AsRef<str> + ToString + Debug>(
    gpubox_filenames: &[T],
) -> Result<(Vec<GPUBoxBatch>, CorrelatorVersion, usize), ErrorKind> {
    if gpubox_filenames.is_empty() {
        return Err(ErrorKind::Custom(
            "determine_gpubox_batches: gpubox / mwax fits files missing".to_string(),
        ));
    }
    let num_gpubox_files = gpubox_filenames.len();
    let mut format = None;
    let mut out_gpubox_batches: Vec<GPUBoxBatch> = vec![];

    for g in gpubox_filenames {
        match RE_MWAX.captures(g.as_ref()) {
            Some(caps) => {
                // Check if we've already matched any files as being the old
                // format. If so, then we've got a mix, and we should exit
                // early.
                match format {
                    None => format = Some(CorrelatorVersion::V2),
                    Some(CorrelatorVersion::V2) => (),
                    _ => {
                        return Err(ErrorKind::Custom(format!(
                            "There are a mixture of gpubox filename types in {:?}",
                            gpubox_filenames
                        )))
                    }
                }

                let batch: usize = caps["batch"].parse()?;
                let channel: usize = caps["channel"].parse()?;
                if !&out_gpubox_batches.iter().any(|b| b.batch_number == batch) {
                    // Enlarge the output vector if we need to.
                    out_gpubox_batches.push(GPUBoxBatch::new(batch));
                }
                // This finds the correct batch and then adds the gpubox file to it
                out_gpubox_batches
                    .iter_mut()
                    .find(|b| b.batch_number == batch)
                    .unwrap()
                    .gpubox_files
                    .push(GPUBoxFile::new(g.to_string(), channel));
            }

            // Try to match the legacy format.
            None => match RE_LEGACY_BATCH.captures(g.as_ref()) {
                Some(caps) => {
                    match format {
                        None => format = Some(CorrelatorVersion::Legacy),
                        Some(CorrelatorVersion::Legacy) => (),
                        _ => {
                            return Err(ErrorKind::Custom(format!(
                                "There are a mixture of gpubox filename types in {:?}",
                                gpubox_filenames
                            )))
                        }
                    }

                    let batch: usize = caps["batch"].parse()?;
                    let channel: usize = caps["band"].parse()?;
                    if !out_gpubox_batches.iter().any(|b| b.batch_number == batch) {
                        // Enlarge the output vector if we need to.
                        out_gpubox_batches.push(GPUBoxBatch::new(batch));
                    }

                    // This finds the correct batch and then adds the gpubox file to it
                    out_gpubox_batches
                        .iter_mut()
                        .find(|b| b.batch_number == batch)
                        .unwrap()
                        .gpubox_files
                        .push(GPUBoxFile::new(g.to_string(), channel));
                }

                // Try to match the old legacy format.
                None => match RE_OLD_LEGACY_FORMAT.captures(g.as_ref()) {
                    Some(caps) => {
                        match format {
                            None => format = Some(CorrelatorVersion::OldLegacy),
                            Some(CorrelatorVersion::OldLegacy) => (),
                            _ => {
                                return Err(ErrorKind::Custom(format!(
                                    "There are a mixture of gpubox filename types in {:?}",
                                    gpubox_filenames
                                )))
                            }
                        }

                        let channel: usize = caps["band"].parse()?;

                        // There's only one batch.
                        if !out_gpubox_batches.iter().any(|b| b.batch_number == 0) {
                            // Enlarge the output vector if we need to.
                            out_gpubox_batches.push(GPUBoxBatch::new(0));
                        }
                        out_gpubox_batches[0]
                            .gpubox_files
                            .push(GPUBoxFile::new(g.to_string(), channel));
                    }
                    None => {
                        return Err(ErrorKind::Custom(format!(
                            "Could not identify the gpubox filename structure for {:?}",
                            g
                        )))
                    }
                },
            },
        }
    }

    // Ensure the output is properly sorted - each batch is sorted by
    // channel_identifier.
    for v in &mut out_gpubox_batches {
        v.gpubox_files
            .sort_unstable_by(|a, b| a.channel_identifier.cmp(&b.channel_identifier));
    }

    // Sort the batches by batch number
    out_gpubox_batches.sort_by_key(|b| b.batch_number);

    // Check batches are contiguous
    for (i, batch) in out_gpubox_batches.iter().enumerate() {
        if i != batch.batch_number {
            return Err(ErrorKind::Custom(format!(
                "There is an entire gpubox batch missing (expected batch {} got {}).\n{}",
                i,
                batch.batch_number,
                out_gpubox_batches
                    .iter()
                    .enumerate()
                    .map(|(i, x)| format!("Batch {}: {}", i, x.gpubox_files.len()))
                    .collect::<Vec<String>>()
                    .join(", ")
            )));
        }
    }

    // Check that an equal number of files are in each batch.
    if !out_gpubox_batches
        .iter()
        .all(|x| x.gpubox_files.len() == out_gpubox_batches[0].gpubox_files.len())
    {
        return Err(ErrorKind::Custom(format!(
            "There are an uneven number of files in the gpubox batches.\n{}",
            out_gpubox_batches
                .iter()
                .enumerate()
                .map(|(i, x)| format!("Batch {}: {}", i, x.gpubox_files.len()))
                .collect::<Vec<String>>()
                .join(", ")
        )));
    }

    Ok((out_gpubox_batches, format.unwrap(), num_gpubox_files))
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
) -> Result<u64, fitsio::errors::Error> {
    let start_unix_time: i64 = metafits_hdu_fptr.read_key(gpubox_fptr, "TIME")?;
    let start_unix_millitime: i64 = metafits_hdu_fptr.read_key(gpubox_fptr, "MILLITIM")?;
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
) -> Result<BTreeMap<u64, usize>, fitsio::errors::Error> {
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
        let hdu = gpubox_fptr.hdu(hdu_index)?;
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
/// * A Result containing `Ok` or an `ErrorKind` if it fails validation.
///
///
pub fn validate_gpubox_metadata_correlator_version(
    gpubox_fptr: &mut FitsFile,
    gpubox_primary_hdu: &FitsHdu,
    gpubox_filename: &str,
    correlator_version: CorrelatorVersion,
) -> Result<(), ErrorKind> {
    // New correlator files include a version - check that it is present.
    // For pre v2, ensure the key isn't present
    let gpu_corr_version =
        get_optional_fits_key::<u8>(gpubox_fptr, &gpubox_primary_hdu, "CORR_VER").unwrap();

    match correlator_version {
        CorrelatorVersion::V2 => {
            match gpu_corr_version {
                None => Err(ErrorKind::Custom(format!(
                    "gpubox::validate_gpubox_metadata_correlator_version: Failed to read key CORR_VER from {}",
                    gpubox_filename
                ))),
                Some(gpu_corr_version_value) => {
                    match gpu_corr_version_value {
                        2 => Ok(()),
                        _ => Err(ErrorKind::Custom(format!(
                            "gpubox::validate_gpubox_metadata_correlator_version: CORR_VER {} from {} does not match expected value of 2",
                            gpu_corr_version_value, gpubox_filename
                        )))
                    }
                }
            }
        }

        CorrelatorVersion::OldLegacy | CorrelatorVersion::Legacy => {
            match gpu_corr_version {
                None => Ok(()),
                Some(gpu_corr_version_value) => Err(ErrorKind::Custom(format!(
                    "gpubox::validate_gpubox_metadata_correlator_version: Correlator version mismatch: gpubox filenames indicate OldLegacy or 
                    Legacy but {} has CORR_VER = {:?}", gpubox_filename, gpu_corr_version_value),
                ))
            }
        }
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
/// * A Result containing `Ok` or an `ErrorKind` if it fails validation.
///
///
pub fn validate_gpubox_metadata_obs_id(
    gpubox_fptr: &mut FitsFile,
    gpubox_primary_hdu: &FitsHdu,
    gpubox_filename: &str,
    metafits_obsid: u32,
) -> Result<(), ErrorKind> {
    // Get the OBSID- if not present, this is probably not an MWA fits file!
    let gpu_obs_id = get_required_fits_key::<u32>(gpubox_fptr, &gpubox_primary_hdu, "OBSID")
        .with_context(|| {
            format!(
                "gpubox::validate_gpubox_metadata_obs_id: Failed to read OBSID for {} - is this an MWA fits file?",
                gpubox_filename
            )
        })?;

    if gpu_obs_id != metafits_obsid {
        Err(ErrorKind::Custom(format!(
            "gpubox::validate_gpubox_metadata_obs_id: OBSID {} from {} does not match expected value of obs_id from metafits file {} 
            maybe you have a mix of different files?",
            gpu_obs_id, gpubox_filename, metafits_obsid
        )))
    } else {
        Ok(())
    }
}

/// Returns a BTree structure consisting of:
/// BTree of timesteps. Each timestep is a BTree for a course channel.
/// Each coarse channel then contains the batch number and hdu index.
/// Since we are reading each gpubox file, we also confirm that the
/// correlator version we got from the filenames matches what's in the
/// primary HDU, and we also check that the metafits obs_id matches
/// the primary HDU obs_id value.
///
/// # Arguments
///
/// * `gpubox_batches` - vector of structs describing each gpubox "batch"
///
/// * `metafits_obs_id` - obs_id as read from metafits
///
/// * `correlator_version` - enum telling us which correlator version the observation was created by (based on filename).
///
/// # Returns
///
/// * A Result containing the GPUBox Time Map or an error.
///
///
pub fn create_time_map(
    gpubox_batches: &mut Vec<GPUBoxBatch>,
    metafits_obs_id: u32,
    correlator_version: CorrelatorVersion,
) -> Result<(BTreeMap<u64, BTreeMap<usize, (usize, usize)>>, usize), ErrorKind> {
    // Open all the files.
    //let mut gpubox_fptrs = Vec::with_capacity(gpubox_batches.len());
    let mut gpubox_time_map = BTreeMap::new();
    // Keep track of the gpubox HDU size and the number of gpubox files.
    let mut size = 0;
    for (batch_num, batch) in gpubox_batches.iter_mut().enumerate() {
        for gpubox_file in &mut batch.gpubox_files {
            let mut fptr = FitsFile::open(&gpubox_file.filename)
                .with_context(|| format!("Failed to open {:?}", gpubox_file))?;

            let hdu = fptr
                .hdu(0)
                .with_context(|| format!("Failed to open HDU 1 of {:?}", gpubox_file))?;

            // Validate gpubox file header against metafits and other known info
            validate_gpubox_metadata_obs_id(
                &mut fptr,
                &hdu,
                &gpubox_file.filename,
                metafits_obs_id,
            )?;

            validate_gpubox_metadata_correlator_version(
                &mut fptr,
                &hdu,
                &gpubox_file.filename,
                correlator_version,
            )?;

            // Store the FITS file pointer for later.
            gpubox_file.fptr = Some(fptr);
        }

        // Because of the way `fitsio` uses the mutable reference to the
        // file handle, it's easier to do another loop here than use `fptr`
        // above.
        for gpubox_file in batch.gpubox_files.iter_mut() {
            // Determine gpubox number. This is from 0..N
            // and will map to the receiver channel numbers in the metafits*
            // (* except for legacy and old legacy if rec channel is > 128 in which case it is reversed), but at this point
            // we don't care about that.
            // Legacy- number is 2 digits and represents the physical gpubox that produced this file. These are mapped in ascending order
            // to the metafits COARSE_CHANNEL list of reciver channels
            // V2- number if 3 digits and referes to the receiver channel number
            let channel_identifier: usize = gpubox_file.channel_identifier;

            let time_map =
                map_unix_times_to_hdus((gpubox_file.fptr.as_mut()).unwrap(), correlator_version)?;
            for (time, hdu_index) in time_map {
                // For the current `time`, check if it's in the map. If not,
                // insert it and a new tree. Then check if `gpubox_num` is
                // in the sub-map for this `time`, etc.
                let new_time_tree = BTreeMap::new();
                gpubox_time_map
                    .entry(time)
                    .or_insert(new_time_tree)
                    .entry(channel_identifier)
                    .or_insert((batch_num, hdu_index));
            }

            // Determine the size of the gpubox HDU image. mwalib will panic
            // if this size is not consistent for all HDUs in all gpubox
            // files.
            let this_size = get_hdu_image_size((gpubox_file.fptr.as_mut()).unwrap())?
                .iter()
                .product();
            if size != 0 && size != this_size {
                return Err(ErrorKind::Custom(
                    "mwalibBuffer::read: Error: HDU sizes in gpubox files are not equal"
                        .to_string(),
                ));
            } else {
                size = this_size;
            }
        }
    }

    Ok((gpubox_time_map, size))
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
) -> Result<ObsTimes, ErrorKind> {
    // Find the maximum number of gpubox files, and assume that this is the
    // total number of input gpubox files.
    let size = match gpubox_time_map.iter().map(|(_, submap)| submap.len()).max() {
        Some(m) => m,
        None => {
            return Err(ErrorKind::Custom(
                "determine_obs_times: Input BTreeMap was empty".to_string(),
            ))
        }
    };

    // Filter the first elements that don't satisfy `submap.len() == size`. The
    // first and last of the submaps that satisfy this condition are the proper
    // start and end of the observation.

    // Is there a way to iterate only once?
    let mut i = gpubox_time_map
        .iter()
        .filter(|(_, submap)| submap.len() == size);
    let proper_start_millisec = match i.next().map(|(time, _)| *time) {
        Some(s) => s,
        None => {
            return Err(ErrorKind::Custom(
                "determine_obs_times: proper_start_millisec was not set".to_string(),
            ))
        }
    };
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
        let (gpubox_batches, corr_format, num_gpubox_files) = result.unwrap();
        assert_eq!(gpubox_batches.len(), 3);
        assert_eq!(corr_format, CorrelatorVersion::Legacy);
        assert_eq!(num_gpubox_files, 3);

        let mut expected_batches: Vec<GPUBoxBatch> = vec![
            GPUBoxBatch::new(0),
            GPUBoxBatch::new(1),
            GPUBoxBatch::new(2),
        ];
        expected_batches[0].gpubox_files.push(GPUBoxFile::new(
            String::from("1065880128_20131015134930_gpubox01_00.fits"),
            1,
        ));

        expected_batches[1].gpubox_files.push(GPUBoxFile::new(
            String::from("1065880128_20131015134930_gpubox20_01.fits"),
            20,
        ));

        expected_batches[2].gpubox_files.push(GPUBoxFile::new(
            String::from("1065880128_20131015134930_gpubox15_02.fits"),
            15,
        ));

        assert_eq!(gpubox_batches, expected_batches);
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
        let (gpubox_batches, corr_format, num_gpubox_files) = result.unwrap();
        assert_eq!(gpubox_batches.len(), 3);
        assert_eq!(corr_format, CorrelatorVersion::Legacy);
        assert_eq!(num_gpubox_files, 3);
        let mut expected_batches: Vec<GPUBoxBatch> = vec![
            GPUBoxBatch::new(0),
            GPUBoxBatch::new(1),
            GPUBoxBatch::new(2),
        ];

        expected_batches[0].gpubox_files.push(GPUBoxFile::new(
            String::from("/home/chj/1065880128_20131015134930_gpubox01_00.fits"),
            1,
        ));

        expected_batches[1].gpubox_files.push(GPUBoxFile::new(
            String::from("/home/gs/1065880128_20131015134930_gpubox20_01.fits"),
            20,
        ));

        expected_batches[2].gpubox_files.push(GPUBoxFile::new(
            String::from("/var/cache/1065880128_20131015134930_gpubox15_02.fits"),
            15,
        ));

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
        let (gpubox_batches, corr_format, num_gpubox_files) = result.unwrap();
        assert_eq!(gpubox_batches.len(), 3);
        assert_eq!(corr_format, CorrelatorVersion::Legacy);
        assert_eq!(num_gpubox_files, 6);

        let mut expected_batches: Vec<GPUBoxBatch> = vec![
            GPUBoxBatch::new(0),
            GPUBoxBatch::new(1),
            GPUBoxBatch::new(2),
        ];

        expected_batches[0].gpubox_files.push(GPUBoxFile::new(
            String::from("/home/chj/1065880128_20131015134930_gpubox01_00.fits"),
            1,
        ));
        expected_batches[0].gpubox_files.push(GPUBoxFile::new(
            String::from("/home/chj/1065880128_20131015134930_gpubox02_00.fits"),
            2,
        ));

        expected_batches[1].gpubox_files.push(GPUBoxFile::new(
            String::from("/home/chj/1065880128_20131015134930_gpubox19_01.fits"),
            19,
        ));
        expected_batches[1].gpubox_files.push(GPUBoxFile::new(
            String::from("/home/chj/1065880128_20131015134930_gpubox20_01.fits"),
            20,
        ));

        expected_batches[2].gpubox_files.push(GPUBoxFile::new(
            String::from("/home/chj/1065880128_20131015134930_gpubox14_02.fits"),
            14,
        ));
        expected_batches[2].gpubox_files.push(GPUBoxFile::new(
            String::from("/home/chj/1065880128_20131015134930_gpubox15_02.fits"),
            15,
        ));

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
        let (gpubox_batches, corr_format, num_gpubox_files) = result.unwrap();
        assert_eq!(gpubox_batches.len(), 3);
        assert_eq!(corr_format, CorrelatorVersion::Legacy);
        assert_eq!(num_gpubox_files, 6);

        let mut expected_batches: Vec<GPUBoxBatch> = vec![
            GPUBoxBatch::new(0),
            GPUBoxBatch::new(1),
            GPUBoxBatch::new(2),
        ];
        expected_batches[0].gpubox_files.push(GPUBoxFile::new(
            String::from("/home/chj/1065880128_20131015134930_gpubox01_00.fits"),
            1,
        ));
        expected_batches[0].gpubox_files.push(GPUBoxFile::new(
            String::from("/home/chj/1065880128_20131015134929_gpubox02_00.fits"),
            2,
        ));

        expected_batches[1].gpubox_files.push(GPUBoxFile::new(
            String::from("/home/chj/1065880128_20131015134930_gpubox19_01.fits"),
            19,
        ));
        expected_batches[1].gpubox_files.push(GPUBoxFile::new(
            String::from("/home/chj/1065880128_20131015134929_gpubox20_01.fits"),
            20,
        ));

        expected_batches[2].gpubox_files.push(GPUBoxFile::new(
            String::from("/home/chj/1065880128_20131015134931_gpubox14_02.fits"),
            14,
        ));
        expected_batches[2].gpubox_files.push(GPUBoxFile::new(
            String::from("/home/chj/1065880128_20131015134930_gpubox15_02.fits"),
            15,
        ));
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
        let (gpubox_batches, corr_format, num_gpubox_files) = result.unwrap();
        assert_eq!(gpubox_batches.len(), 1);
        assert_eq!(corr_format, CorrelatorVersion::OldLegacy);
        assert_eq!(num_gpubox_files, 3);

        let mut expected_batches: Vec<GPUBoxBatch> = vec![GPUBoxBatch::new(0)];
        expected_batches[0].gpubox_files.push(GPUBoxFile::new(
            String::from("1065880128_20131015134930_gpubox01.fits"),
            1,
        ));
        expected_batches[0].gpubox_files.push(GPUBoxFile::new(
            String::from("1065880128_20131015134930_gpubox15.fits"),
            15,
        ));
        expected_batches[0].gpubox_files.push(GPUBoxFile::new(
            String::from("1065880128_20131015134930_gpubox20.fits"),
            20,
        ));

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
        let (gpubox_batches, corr_format, num_gpubox_files) = result.unwrap();
        assert_eq!(gpubox_batches.len(), 2);
        assert_eq!(corr_format, CorrelatorVersion::V2);
        assert_eq!(num_gpubox_files, 4);

        let mut expected_batches: Vec<GPUBoxBatch> = vec![GPUBoxBatch::new(0), GPUBoxBatch::new(1)];
        expected_batches[0].gpubox_files.push(GPUBoxFile::new(
            String::from("1065880128_20131015134930_ch101_000.fits"),
            101,
        ));
        expected_batches[0].gpubox_files.push(GPUBoxFile::new(
            String::from("1065880128_20131015134930_ch102_000.fits"),
            102,
        ));
        expected_batches[1].gpubox_files.push(GPUBoxFile::new(
            String::from("1065880128_20131015135030_ch101_001.fits"),
            101,
        ));
        expected_batches[1].gpubox_files.push(GPUBoxFile::new(
            String::from("1065880128_20131015135030_ch102_001.fits"),
            102,
        ));

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
