// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
The main interface to MWA data.
 */
use std::collections::BTreeMap;
use std::fmt;

use crate::coarse_channel::*;
use crate::convert::*;
use crate::error::*;
use crate::gpubox_files::*;
use crate::metafits_context::*;
use crate::timestep::*;
use crate::*;

#[cfg(test)]
mod test;

///
/// `mwalib` correlator observation context. This represents the basic metadata for a correlator observation.
///
#[derive(Debug)]
pub struct CorrelatorContext {
    /// Observation Metadata
    pub metafits_context: MetafitsContext,
    /// Version of the correlator format
    pub mwa_version: MWAVersion,
    /// This is an array of all known timesteps (union of metafits and provided timesteps from data files)
    pub timesteps: Vec<TimeStep>,
    /// Number of timesteps in the timesteps vector
    pub num_timesteps: usize,
    /// Vector of coarse channel structs which is the same as the metafits provided coarse channels
    pub coarse_chans: Vec<CoarseChannel>,
    /// Number of coarse channels in the coarse channel vector
    pub num_coarse_chans: usize,

    /// Vector of (in)common timestep indices
    pub common_timestep_indices: Vec<usize>,
    // Number of common timesteps
    pub num_common_timesteps: usize,
    /// Vector of (in)common coarse channel indices
    pub common_coarse_chan_indices: Vec<usize>,
    // Number of common coarse channels
    pub num_common_coarse_chans: usize,

    /// The start of the observation (the time that is common to all
    /// provided gpubox files).
    pub common_start_unix_time_ms: u64,
    /// `end_unix_time_ms` is the common end time of the observation
    /// i.e. start time of last common timestep plus integration time.
    pub common_end_unix_time_ms: u64,
    /// `start_unix_time_ms` but in GPS milliseconds
    pub common_start_gps_time_ms: u64,
    /// `end_unix_time_ms` but in GPS milliseconds
    pub common_end_gps_time_ms: u64,
    /// Total duration of common timesteps
    pub common_duration_ms: u64,
    /// Total bandwidth of the common coarse channels
    pub common_bandwidth_hz: u32,

    /// Vector of (in)common timestep indices only including timesteps after the quack time
    pub common_good_timestep_indices: Vec<usize>,
    // Number of common timesteps only including timesteps after the quack time
    pub num_common_good_timesteps: usize,
    /// Vector of (in)common coarse channel indices only including timesteps after the quack time
    pub common_good_coarse_chan_indices: Vec<usize>,
    // Number of common coarse channels only including timesteps after the quack time
    pub num_common_good_coarse_chans: usize,
    /// The start of the observation (the time that is common to all
    /// provided gpubox files) only including timesteps after the quack time
    pub common_good_start_unix_time_ms: u64,
    /// `end_unix_time_ms` is the common end time of the observation only including timesteps after the quack time
    /// i.e. start time of last common timestep plus integration time.
    pub common_good_end_unix_time_ms: u64,
    /// `common_good_start_unix_time_ms` but in GPS milliseconds
    pub common_good_start_gps_time_ms: u64,
    /// `common_good_end_unix_time_ms` but in GPS milliseconds
    pub common_good_end_gps_time_ms: u64,
    /// Total duration of common_good timesteps
    pub common_good_duration_ms: u64,
    /// Total bandwidth of the common coarse channels only including timesteps after the quack time
    pub common_good_bandwidth_hz: u32,

    /// The number of bytes taken up by a scan/timestep in each gpubox file.
    pub num_timestep_coarse_chan_bytes: usize,
    /// The number of floats in each gpubox HDU.
    pub num_timestep_coarse_chan_floats: usize,
    /// This is the number of gpubox files *per batch*.
    pub num_gpubox_files: usize,
    /// `gpubox_batches` *must* be sorted appropriately. See
    /// `gpubox::determine_gpubox_batches`. The order of the filenames
    /// corresponds directly to other gpubox-related objects
    /// (e.g. `gpubox_hdu_limits`). Structured:
    /// `gpubox_batches[batch][filename]`.
    pub gpubox_batches: Vec<GpuBoxBatch>,
    /// We assume as little as possible about the data layout in the gpubox
    /// files; here, a `BTreeMap` contains each unique UNIX time from every
    /// gpubox, which is associated with another `BTreeMap`, associating each
    /// gpubox number with a gpubox batch number and HDU index. The gpubox
    /// number, batch number and HDU index are everything needed to find the
    /// correct HDU out of all gpubox files.
    pub gpubox_time_map: BTreeMap<u64, BTreeMap<usize, (usize, usize)>>,
    /// A conversion table to optimise reading of legacy MWA HDUs
    pub(crate) legacy_conversion_table: Vec<LegacyConversionBaseline>,
}

impl CorrelatorContext {
    /// From a path to a metafits file and paths to gpubox files, create an `CorrelatorContext`.
    ///
    /// The traits on the input parameters allow flexibility to input types.
    ///
    /// # Arguments
    ///
    /// * `metafits_filename` - filename of metafits file as a path or string.
    ///
    /// * `gpubox_filenames` - slice of filenames of gpubox files as paths or strings.
    ///
    ///
    /// # Returns
    ///
    /// * Result containing a populated CorrelatorContext object if Ok.
    ///
    ///
    pub fn new<T: AsRef<std::path::Path>>(
        metafits_filename: &T,
        gpubox_filenames: &[T],
    ) -> Result<Self, MwalibError> {
        let mut metafits_context = MetafitsContext::new_internal(metafits_filename)?;

        if gpubox_filenames.is_empty() {
            return Err(MwalibError::Gpubox(
                gpubox_files::error::GpuboxError::NoGpuboxes,
            ));
        }
        // Do gpubox stuff only if we have gpubox files.
        let gpubox_info = examine_gpubox_files(&gpubox_filenames, metafits_context.obs_id)?;

        // Populate metafits coarse channels and timesteps now that we know what MWA Version we are dealing with
        // Populate the coarse channels
        metafits_context.populate_expected_coarse_channels(gpubox_info.mwa_version)?;

        // Populate the timesteps
        metafits_context.populate_expected_timesteps(gpubox_info.mwa_version)?;

        // We can unwrap here because the `gpubox_time_map` can't be empty if
        // `gpuboxes` isn't empty.
        let timesteps = TimeStep::populate_correlator_timesteps(
            &gpubox_info.time_map,
            &metafits_context.metafits_timesteps,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
            metafits_context.corr_int_time_ms,
        )
        .unwrap();

        let num_timesteps = timesteps.len();

        // Populate coarse channels
        // First- the "main" coarse channel vector is simply the metafits coarse channels
        let coarse_chans = metafits_context.metafits_coarse_chans.clone();
        let num_coarse_chans = coarse_chans.len();

        // We have enough information to validate HDU matches metafits for the first batch/first coarse channel we have data for
        if !gpubox_filenames.is_empty() {
            let mut fptr = fits_open!(&gpubox_info.batches[0].gpubox_files[0].filename)?;

            CorrelatorContext::validate_first_hdu(
                gpubox_info.mwa_version,
                metafits_context.num_corr_fine_chans_per_coarse,
                metafits_context.num_baselines,
                metafits_context.num_visibility_pols,
                &mut fptr,
            )?;
        }

        // Populate the start and end times of the observation based on common channels.
        // Start= start of first timestep
        // End  = start of last timestep + integration time
        let (
            common_start_unix_time_ms,
            common_end_unix_time_ms,
            common_duration_ms,
            common_coarse_chan_identifiers,
        ) = {
            let o = determine_common_obs_times_and_chans(
                &gpubox_info.time_map,
                metafits_context.corr_int_time_ms,
                None,
            )?;
            (
                o.start_time_unix_ms,
                o.end_time_unix_ms,
                o.duration_ms,
                o.coarse_chan_identifiers,
            )
        };

        // Populate the common coarse_chan indices vector
        let mut common_coarse_chan_indices: Vec<usize> = common_coarse_chan_identifiers
            .iter()
            .map(|chan_identifier| {
                coarse_chans
                    .iter()
                    .position(|corr_coarse_chan| corr_coarse_chan.gpubox_number == *chan_identifier)
                    .unwrap()
            })
            .collect::<Vec<usize>>();

        common_coarse_chan_indices.sort_unstable();

        // Populate the common timestep indices vector
        let mut common_timestep_indices: Vec<usize> =
            vec![0; (common_duration_ms / metafits_context.corr_int_time_ms) as usize];

        // Ugly, but this will populate a vector of the indices of the common timesteps
        for (cts_index, cts_value) in common_timestep_indices.iter_mut().enumerate() {
            *cts_value = timesteps
                .iter()
                .position(|t| {
                    t.unix_time_ms
                        == common_start_unix_time_ms
                            + (cts_index as u64 * metafits_context.corr_int_time_ms)
                })
                .unwrap();
        }
        let num_common_timesteps = common_timestep_indices.len();

        // Determine some other "common" attributes
        let num_common_coarse_chans = common_coarse_chan_indices.len();
        let common_bandwidth_hz =
            (num_common_coarse_chans as u32) * metafits_context.coarse_chan_width_hz;

        // Convert the real start and end times to GPS time
        let common_start_gps_time_ms = misc::convert_unixtime_to_gpstime(
            common_start_unix_time_ms,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
        );
        let common_end_gps_time_ms = misc::convert_unixtime_to_gpstime(
            common_end_unix_time_ms,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
        );

        // Populate the start and end times of the observation based on common channels after the quack time.
        // Start= start of first timestep
        // End  = start of last timestep + integration time
        let (
            common_good_start_unix_time_ms,
            common_good_end_unix_time_ms,
            common_good_duration_ms,
            common_good_coarse_chan_identifiers,
        ) = {
            let o = determine_common_obs_times_and_chans(
                &gpubox_info.time_map,
                metafits_context.corr_int_time_ms,
                Some(metafits_context.good_time_unix_ms),
            )?;
            (
                o.start_time_unix_ms,
                o.end_time_unix_ms,
                o.duration_ms,
                o.coarse_chan_identifiers,
            )
        };

        // Populate the common good coarse_chan indices vector
        let mut common_good_coarse_chan_indices: Vec<usize> = common_good_coarse_chan_identifiers
            .iter()
            .map(|chan_identifier| {
                coarse_chans
                    .iter()
                    .position(|corr_coarse_chan| corr_coarse_chan.gpubox_number == *chan_identifier)
                    .unwrap()
            })
            .collect::<Vec<usize>>();

        common_good_coarse_chan_indices.sort_unstable();

        // Populate the common timestep indices vector
        let mut common_good_timestep_indices: Vec<usize> =
            vec![0; (common_good_duration_ms / metafits_context.corr_int_time_ms) as usize];

        // Ugly, but this will populate a vector of the indices of the common timesteps
        for (cts_index, cts_value) in common_good_timestep_indices.iter_mut().enumerate() {
            *cts_value = timesteps
                .iter()
                .position(|t| {
                    t.unix_time_ms
                        == common_good_start_unix_time_ms
                            + (cts_index as u64 * metafits_context.corr_int_time_ms)
                })
                .unwrap();
        }
        let num_common_good_timesteps = common_good_timestep_indices.len();

        // Determine some other "common good" attributes
        let num_common_good_coarse_chans = common_good_coarse_chan_indices.len();
        let common_good_bandwidth_hz =
            (num_common_good_coarse_chans as u32) * metafits_context.coarse_chan_width_hz;

        // Convert the real start and end times to GPS time
        let common_good_start_gps_time_ms = misc::convert_unixtime_to_gpstime(
            common_good_start_unix_time_ms,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
        );
        let common_good_end_gps_time_ms = misc::convert_unixtime_to_gpstime(
            common_good_end_unix_time_ms,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
        );

        // Prepare the conversion array to convert legacy correlator format into mwax format
        // or just leave it empty if we're in any other format
        let legacy_conversion_table: Vec<LegacyConversionBaseline> = match gpubox_info.mwa_version {
            MWAVersion::CorrOldLegacy | MWAVersion::CorrLegacy => {
                convert::generate_conversion_array(&mut metafits_context.rf_inputs.clone())
            }
            _ => Vec::new(),
        };

        Ok(CorrelatorContext {
            metafits_context,
            mwa_version: gpubox_info.mwa_version,
            num_timesteps,
            timesteps,
            num_coarse_chans,
            coarse_chans,
            num_common_timesteps,
            common_timestep_indices,
            num_common_coarse_chans,
            common_coarse_chan_indices,
            common_start_unix_time_ms,
            common_end_unix_time_ms,
            common_start_gps_time_ms,
            common_end_gps_time_ms,
            common_duration_ms,
            common_bandwidth_hz,
            num_common_good_timesteps,
            common_good_timestep_indices,
            num_common_good_coarse_chans,
            common_good_coarse_chan_indices,
            common_good_start_unix_time_ms,
            common_good_end_unix_time_ms,
            common_good_start_gps_time_ms,
            common_good_end_gps_time_ms,
            common_good_duration_ms,
            common_good_bandwidth_hz,
            gpubox_batches: gpubox_info.batches,
            gpubox_time_map: gpubox_info.time_map,
            num_timestep_coarse_chan_bytes: gpubox_info.hdu_size * 4,
            num_timestep_coarse_chan_floats: gpubox_info.hdu_size,
            num_gpubox_files: gpubox_filenames.len(),
            legacy_conversion_table,
        })
    }

    /// Read a single timestep for a single coarse channel
    /// The output visibilities are in order:
    /// baseline,frequency,pol,r,i
    ///
    /// # Arguments
    ///
    /// * `timestep_index` - index within the timestep array for the desired timestep. This corresponds
    ///                      to the element within mwalibContext.timesteps.
    ///
    /// * `coarse_chan_index` - index within the coarse_chan array for the desired coarse channel. This corresponds
    ///                      to the element within mwalibContext.coarse_chans.
    ///
    ///
    /// # Returns
    ///
    /// * A Result containing vector of 32 bit floats containing the data in [baseline][frequency][pol][r][i] order, if Ok.
    ///
    ///
    pub fn read_by_baseline(
        &self,
        timestep_index: usize,
        coarse_chan_index: usize,
    ) -> Result<Vec<f32>, GpuboxError> {
        let mut return_buffer: Vec<f32> = vec![0.; self.num_timestep_coarse_chan_floats];

        self.read_by_baseline_into_buffer(timestep_index, coarse_chan_index, &mut return_buffer)?;

        Ok(return_buffer)
    }

    /// Read a single timestep for a single coarse channel
    /// The output visibilities are in order:
    /// frequency,baseline,pol,r,i
    ///
    /// # Arguments
    ///
    /// * `timestep_index` - index within the timestep array for the desired timestep. This corresponds
    ///                      to the element within mwalibContext.timesteps.
    ///
    /// * `coarse_chan_index` - index within the coarse_chan array for the desired coarse channel. This corresponds
    ///                      to the element within mwalibContext.coarse_chans.
    ///
    ///
    /// # Returns
    ///
    /// * A Result containing vector of 32 bit floats containing the data in frequency,baseline,pol,r,i order, if Ok.
    ///
    ///
    pub fn read_by_frequency(
        &self,
        timestep_index: usize,
        coarse_chan_index: usize,
    ) -> Result<Vec<f32>, GpuboxError> {
        let mut return_buffer: Vec<f32> = vec![0.; self.num_timestep_coarse_chan_floats];

        self.read_by_frequency_into_buffer(timestep_index, coarse_chan_index, &mut return_buffer)?;

        Ok(return_buffer)
    }

    /// Validate input timestep_index and coarse_chan_index and return the fits_filename, batch index and hdu of the corresponding data
    ///
    /// # Arguments
    ///
    /// * `timestep_index` - index within the timestep array for the desired timestep. This corresponds
    ///                      to the element within Context.timesteps.
    ///
    /// * `coarse_chan_index` - index within the coarse_chan array for the desired coarse channel. This corresponds
    ///                      to the element within Context.coarse_chans.        
    ///
    /// # Returns
    ///
    /// * A Result of Ok wrapping the fits_filename, batch_index and hdu_index if success or a GpuboxError on failure.
    ///
    fn get_fits_filename_and_batch_and_hdu(
        &self,
        timestep_index: usize,
        coarse_chan_index: usize,
    ) -> Result<(&str, usize, usize), GpuboxError> {
        // Validate the timestep
        if timestep_index > self.num_timesteps - 1 {
            return Err(GpuboxError::InvalidTimeStepIndex(self.num_timesteps - 1));
        }

        // Validate the coarse chan
        if coarse_chan_index > self.num_coarse_chans - 1 {
            return Err(GpuboxError::InvalidCoarseChanIndex(
                self.num_coarse_chans - 1,
            ));
        }

        if self.gpubox_batches.is_empty() {
            return Err(GpuboxError::NoGpuboxes);
        }

        // Lookup the coarse channel we need
        let channel_identifier = self.coarse_chans[coarse_chan_index].gpubox_number;

        // Get the batch index & hdu based on unix time of the timestep
        let (batch_index, hdu_index) = match self
            .gpubox_time_map
            .get(&self.timesteps[timestep_index].unix_time_ms)
        {
            Some(t) => match t.get(&channel_identifier) {
                Some(c) => c,
                None => {
                    return Err(GpuboxError::NoDataForTimeStepCoarseChannel {
                        timestep_index,
                        coarse_chan_index,
                    })
                }
            },
            None => {
                return Err(GpuboxError::NoDataForTimeStepCoarseChannel {
                    timestep_index,
                    coarse_chan_index,
                })
            }
        };

        // For the batch number and coarse channel identifier, find the fits filename we need
        let fits_filename = match &self.gpubox_batches[*batch_index]
            .gpubox_files
            .iter()
            .find(|&gf| gf.channel_identifier == channel_identifier)
        {
            Some(gpuboxfile) => &gpuboxfile.filename,
            None => {
                return Err(GpuboxError::NoDataForTimeStepCoarseChannel {
                    timestep_index,
                    coarse_chan_index,
                })
            }
        };

        Ok((fits_filename, *batch_index, *hdu_index))
    }

    /// Read a single timestep for a single coarse channel
    /// The output visibilities are in order:
    /// baseline,frequency,pol,r,i
    ///
    /// # Arguments
    ///
    /// * `timestep_index` - index within the timestep array for the desired timestep. This corresponds
    ///                      to the element within Context.timesteps.
    ///
    /// * `coarse_chan_index` - index within the coarse_chan array for the desired coarse channel. This corresponds
    ///                      to the element within Context.coarse_chans.
    ///
    /// * `buffer` - Float buffer as a slice which will be filled with data from the HDU read in [baseline][frequency][pol][r][i] order.
    ///
    /// # Returns
    ///
    /// * A Result of Ok if success or a GpuboxError on failure.
    ///
    pub fn read_by_baseline_into_buffer(
        &self,
        timestep_index: usize,
        coarse_chan_index: usize,
        buffer: &mut [f32],
    ) -> Result<(), GpuboxError> {
        // Validate input timestep_index and coarse_chan_index and return the fits_filename, batch index and hdu of the corresponding data
        let (fits_filename, _, hdu_index) =
            self.get_fits_filename_and_batch_and_hdu(timestep_index, coarse_chan_index)?;

        // Open the fits file
        let mut fptr = fits_open!(&fits_filename)?;
        let hdu = fits_open_hdu!(&mut fptr, hdu_index)?;

        // If legacy correlator, then convert the HDU into the correct output format
        if self.mwa_version == MWAVersion::CorrOldLegacy
            || self.mwa_version == MWAVersion::CorrLegacy
        {
            // Prepare temporary buffer, if we are reading legacy correlator files
            let mut temp_buffer = vec![
                0.;
                self.metafits_context.num_corr_fine_chans_per_coarse
                    * self.metafits_context.num_visibility_pols
                    * self.metafits_context.num_baselines
                    * 2
            ];

            // Read into temp buffer
            get_fits_float_image_into_buffer!(&mut fptr, &hdu, &mut temp_buffer)?;

            convert::convert_legacy_hdu_to_mwax_baseline_order(
                &self.legacy_conversion_table,
                &temp_buffer,
                buffer,
                self.metafits_context.num_corr_fine_chans_per_coarse,
            );

            Ok(())
        } else {
            // Read into caller's buffer
            get_fits_float_image_into_buffer!(&mut fptr, &hdu, buffer)?;

            Ok(())
        }
    }

    /// Read a single timestep for a single coarse channel into a supplied buffer
    /// The output visibilities are in order:
    /// frequency,baseline,pol,r,i
    ///
    /// # Arguments
    ///
    /// * `timestep_index` - index within the timestep array for the desired timestep. This corresponds
    ///                      to the element within mwalibContext.timesteps.
    ///
    /// * `coarse_chan_index` - index within the coarse_chan array for the desired coarse channel. This corresponds
    ///                      to the element within mwalibContext.coarse_chans.
    ///
    /// * `buffer` - Float buffer as a slice which will be filled with data from the HDU read in [baseline][frequency][pol][r][i] order.
    ///
    /// # Returns
    ///
    /// * A Result of Ok if success or a GpuboxError on failure.
    ///
    pub fn read_by_frequency_into_buffer(
        &self,
        timestep_index: usize,
        coarse_chan_index: usize,
        buffer: &mut [f32],
    ) -> Result<(), GpuboxError> {
        // Validate input timestep_index and coarse_chan_index and return the fits_filename, batch index and hdu of the corresponding data
        let (fits_filename, _, hdu_index) =
            self.get_fits_filename_and_batch_and_hdu(timestep_index, coarse_chan_index)?;

        // Open the fits file
        let mut fptr = fits_open!(&fits_filename)?;
        let hdu = fits_open_hdu!(&mut fptr, hdu_index)?;

        // Prepare temporary buffer
        let mut temp_buffer = vec![
            0.;
            self.metafits_context.num_corr_fine_chans_per_coarse
                * self.metafits_context.num_visibility_pols
                * self.metafits_context.num_baselines
                * 2
        ];

        // Read the hdu into our temp buffer
        get_fits_float_image_into_buffer!(&mut fptr, &hdu, &mut temp_buffer)?;

        // If legacy correlator, then convert the HDU into the correct output format
        if self.mwa_version == MWAVersion::CorrOldLegacy
            || self.mwa_version == MWAVersion::CorrLegacy
        {
            convert::convert_legacy_hdu_to_mwax_frequency_order(
                &self.legacy_conversion_table,
                &temp_buffer,
                buffer,
                self.metafits_context.num_corr_fine_chans_per_coarse,
            );

            Ok(())
        } else {
            // Do conversion for mwax (it is in baseline order, we want it in freq order)
            convert::convert_mwax_hdu_to_frequency_order(
                &temp_buffer,
                buffer,
                self.metafits_context.num_baselines,
                self.metafits_context.num_corr_fine_chans_per_coarse,
                self.metafits_context.num_visibility_pols,
            );

            Ok(())
        }
    }

    /// Validates the first HDU of a gpubox file against metafits metadata
    ///
    /// In this case we call `validate_hdu_axes()`
    ///
    /// # Arguments
    ///
    /// * `mwa_version` - Correlator version of this gpubox file.
    ///
    /// * `metafits_fine_chans_per_coarse` - the number of fine chan per coarse as calculated using info from metafits.
    ///
    /// * `metafits_baselines` - the number of baselines as reported by the metafits file.
    ///
    /// * `visibility_pols` - the number of pols produced by the correlator (always 4 for MWA)
    ///
    /// * `gpubox_fptr` - FITSFile pointer to an MWA GPUbox file
    ///
    /// # Returns
    ///
    /// * Result containing `Ok` if it is valid, or a custom `MwalibError` if not valid.
    ///
    ///
    fn validate_first_hdu(
        mwa_version: MWAVersion,
        metafits_fine_chans_per_coarse: usize,
        metafits_baselines: usize,
        visibility_pols: usize,
        gpubox_fptr: &mut fitsio::FitsFile,
    ) -> Result<(), GpuboxError> {
        // Get NAXIS1 and NAXIS2 from a gpubox file first image HDU
        let hdu = fits_open_hdu!(gpubox_fptr, 1)?;
        let dimensions = get_hdu_image_size!(gpubox_fptr, &hdu)?;
        let naxis1 = dimensions[1];
        let naxis2 = dimensions[0];

        Self::validate_hdu_axes(
            mwa_version,
            metafits_fine_chans_per_coarse,
            metafits_baselines,
            visibility_pols,
            naxis1,
            naxis2,
        )
    }

    /// Validates the first HDU of a gpubox file against metafits metadata
    ///
    /// In this case we check that NAXIS1 = the correct value and NAXIS2 = the correct value calculated from the metafits
    ///
    /// # Arguments
    ///
    /// * `mwa_version` - Correlator version of this gpubox file.
    ///
    /// * `metafits_fine_chans_per_coarse` - the number of fine chan per coarse as calculated using info from metafits.
    ///
    /// * `metafits_baselines` - the number of baselines as reported by the metafits file.
    ///
    /// * `visibility_pols` - the number of pols produced by the correlator (always 4 for MWA)
    ///
    /// * `naxis1` - NAXIS1 keyword read from HDU1 of a Gpubox file
    ///
    /// * `naxis2` - NAXIS2 keyword read from HDU1 of a Gpubox file
    ///
    /// # Returns
    ///
    /// * Result containing `Ok` if it is valid, or a custom `MwalibError` if not valid.
    ///
    ///
    fn validate_hdu_axes(
        mwa_version: MWAVersion,
        metafits_fine_chans_per_coarse: usize,
        metafits_baselines: usize,
        visibility_pols: usize,
        naxis1: usize,
        naxis2: usize,
    ) -> Result<(), GpuboxError> {
        // We have different values depending on the version of the correlator
        match mwa_version {
            MWAVersion::CorrOldLegacy | MWAVersion::CorrLegacy => {
                // NAXIS1 = baselines * visibility_pols * 2
                // NAXIS2 = fine channels
                let calculated_naxis1: i32 = metafits_baselines as i32 * visibility_pols as i32 * 2;
                let calculated_naxis2: i32 = metafits_fine_chans_per_coarse as i32;

                if calculated_naxis1 != naxis1 as i32 {
                    return Err(GpuboxError::LegacyNaxis1Mismatch {
                        naxis1,
                        calculated_naxis1,
                        metafits_baselines,
                        visibility_pols,
                        naxis2,
                    });
                }
                if calculated_naxis2 != naxis2 as i32 {
                    return Err(GpuboxError::LegacyNaxis2Mismatch {
                        naxis2,
                        calculated_naxis2,
                        metafits_fine_chans_per_coarse,
                    });
                }
            }
            MWAVersion::CorrMWAXv2 => {
                // NAXIS1 = fine channels * visibility pols * 2
                // NAXIS2 = baselines
                let calculated_naxis1: i32 =
                    metafits_fine_chans_per_coarse as i32 * visibility_pols as i32 * 2;
                let calculated_naxis2: i32 = metafits_baselines as i32;

                if calculated_naxis1 != naxis1 as i32 {
                    return Err(GpuboxError::MwaxNaxis1Mismatch {
                        naxis1,
                        calculated_naxis1,
                        metafits_fine_chans_per_coarse,
                        visibility_pols,
                        naxis2,
                    });
                }
                if calculated_naxis2 != naxis2 as i32 {
                    return Err(GpuboxError::MwaxNaxis2Mismatch {
                        naxis2,
                        calculated_naxis2,
                        metafits_baselines,
                    });
                }
            }
            _ => return Err(GpuboxError::InvalidMwaVersion { mwa_version }),
        }

        Ok(())
    }
}

/// Implements fmt::Display for CorrelatorContext struct
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
impl fmt::Display for CorrelatorContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // `size` is the number of floats (self.gpubox_hdu_size) multiplied by 4
        // bytes per float, divided by 1024^2 to get MiB.
        let size = (self.num_timestep_coarse_chan_floats * 4) as f64 / (1024 * 1024) as f64;

        writeln!(
            f,
            r#"CorrelatorContext (
            Metafits Context:           {metafits_context}
            Correlator version:         {corr_ver},
            
            num timesteps:              {n_timesteps},
            timesteps:                  {timesteps:?},
            num coarse channels,        {n_coarse},
            coarse channels:            {coarse:?},

            Common timestep indices:    {num_common_timesteps}: {common_ts:?},
            Common coarse chan indices: {num_common_coarse_chans}: {common_chans:?},
            Common UNIX start time:     {common_start_unix},
            Common UNIX end time:       {common_end_unix},
            Common GPS start time:      {common_start_gps},
            Common GPS end time:        {common_end_gps},
            Common duration:            {common_duration} s,
            Common bandwidth:           {common_bw} MHz,     
            
            Common/Good timestep indices:    {num_common_good_timesteps}: {common_good_ts:?},
            Common/Good coarse chan indices: {num_common_good_coarse_chans}: {common_good_chans:?},
            Common/Good UNIX start time:     {common_good_start_unix},
            Common/Good UNIX end time:       {common_good_end_unix},
            Common/Good GPS start time:      {common_good_start_gps},
            Common/Good GPS end time:        {common_good_end_gps},
            Common/Good duration:            {common_good_duration} s,
            Common/Good bandwidth:           {common_good_bw} MHz,     

            gpubox HDU size:            {hdu_size} MiB,
            Memory usage per scan:      {scan_size} MiB,

            gpubox batches:             {batches:#?},
        )"#,
            metafits_context = self.metafits_context,
            corr_ver = self.mwa_version,
            n_timesteps = self.num_timesteps,
            timesteps = self.timesteps,
            n_coarse = self.num_coarse_chans,
            coarse = self.coarse_chans,
            common_ts = self.common_timestep_indices,
            num_common_timesteps = self.num_common_timesteps,
            common_chans = self.common_coarse_chan_indices,
            num_common_coarse_chans = self.num_common_coarse_chans,
            common_start_unix = self.common_start_unix_time_ms as f64 / 1e3,
            common_end_unix = self.common_end_unix_time_ms as f64 / 1e3,
            common_start_gps = self.common_start_gps_time_ms as f64 / 1e3,
            common_end_gps = self.common_end_gps_time_ms as f64 / 1e3,
            common_duration = self.common_duration_ms as f64 / 1e3,
            common_bw = self.common_bandwidth_hz as f64 / 1e6,
            common_good_ts = self.common_good_timestep_indices,
            num_common_good_timesteps = self.num_common_good_timesteps,
            common_good_chans = self.common_good_coarse_chan_indices,
            num_common_good_coarse_chans = self.num_common_good_coarse_chans,
            common_good_start_unix = self.common_good_start_unix_time_ms as f64 / 1e3,
            common_good_end_unix = self.common_good_end_unix_time_ms as f64 / 1e3,
            common_good_start_gps = self.common_good_start_gps_time_ms as f64 / 1e3,
            common_good_end_gps = self.common_good_end_gps_time_ms as f64 / 1e3,
            common_good_duration = self.common_good_duration_ms as f64 / 1e3,
            common_good_bw = self.common_good_bandwidth_hz as f64 / 1e6,
            hdu_size = size,
            scan_size = size * self.num_gpubox_files as f64,
            batches = self.gpubox_batches,
        )
    }
}
