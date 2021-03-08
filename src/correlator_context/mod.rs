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
    pub corr_version: CorrelatorVersion,
    /// The proper start of the observation (the time that is common to all
    /// provided gpubox files).
    pub start_unix_time_ms: u64,
    /// `end_unix_time_ms` is the actual end time of the observation
    /// i.e. start time of last common timestep plus integration time.
    pub end_unix_time_ms: u64,
    /// `start_unix_time_ms` but in GPS milliseconds
    pub start_gps_time_ms: u64,
    /// `end_unix_time_ms` but in GPS milliseconds
    pub end_gps_time_ms: u64,
    /// Total duration of observation (based on gpubox files)
    pub duration_ms: u64,
    /// Number of timesteps in the observation
    pub num_timesteps: usize,
    /// This is an array of all timesteps we have data for
    pub timesteps: Vec<TimeStep>,
    /// Number of coarse channels after we've validated the input gpubox files
    pub num_coarse_chans: usize,
    /// Vector of coarse channel structs
    pub coarse_chans: Vec<CoarseChannel>,
    /// Total bandwidth of the common coarse channels which have been provided (which may be less than or equal to the bandwith in the MetafitsContext)
    pub bandwidth_hz: u32,
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
    pub(crate) gpubox_batches: Vec<GpuBoxBatch>,
    /// We assume as little as possible about the data layout in the gpubox
    /// files; here, a `BTreeMap` contains each unique UNIX time from every
    /// gpubox, which is associated with another `BTreeMap`, associating each
    /// gpubox number with a gpubox batch number and HDU index. The gpubox
    /// number, batch number and HDU index are everything needed to find the
    /// correct HDU out of all gpubox files.
    pub(crate) gpubox_time_map: BTreeMap<u64, BTreeMap<usize, (usize, usize)>>,
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
        let metafits_context = MetafitsContext::new(metafits_filename)?;

        // Re-open metafits file
        let mut metafits_fptr = fits_open!(&metafits_filename)?;
        let metafits_hdu = fits_open_hdu!(&mut metafits_fptr, 0)?;

        if gpubox_filenames.is_empty() {
            return Err(MwalibError::Gpubox(
                gpubox_files::error::GpuboxError::NoGpuboxes,
            ));
        }
        // Do gpubox stuff only if we have gpubox files.
        let gpubox_info = examine_gpubox_files(&gpubox_filenames, metafits_context.obs_id)?;
        // We can unwrap here because the `gpubox_time_map` can't be empty if
        // `gpuboxes` isn't empty.
        let timesteps = TimeStep::populate_correlator_timesteps(
            &gpubox_info.time_map,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
        )
        .unwrap();

        let num_timesteps = timesteps.len();

        // Populate coarse channels
        // Get metafits info
        let (metafits_coarse_chan_vec, metafits_coarse_chan_width_hz) =
            CoarseChannel::get_metafits_coarse_channel_info(
                &mut metafits_fptr,
                &metafits_hdu,
                metafits_context.obs_bandwidth_hz,
            )?;

        // Process the channels based on the gpubox files we have
        let coarse_chans = CoarseChannel::populate_coarse_channels(
            gpubox_info.corr_format,
            &metafits_coarse_chan_vec,
            metafits_coarse_chan_width_hz,
            Some(&gpubox_info.time_map),
            None,
        )?;

        let num_coarse_chans = coarse_chans.len();
        let bandwidth_hz = (num_coarse_chans as u32) * metafits_coarse_chan_width_hz;

        // We have enough information to validate HDU matches metafits
        if !gpubox_filenames.is_empty() {
            let coarse_chan = coarse_chans[0].gpubox_number;
            let (batch_index, _) = gpubox_info.time_map[&timesteps[0].unix_time_ms][&coarse_chan];

            let mut fptr = fits_open!(&gpubox_info.batches[batch_index].gpubox_files[0].filename)?;

            CorrelatorContext::validate_first_hdu(
                gpubox_info.corr_format,
                metafits_context.num_corr_fine_chans_per_coarse,
                metafits_context.num_baselines,
                metafits_context.num_visibility_pols,
                &mut fptr,
            )?;
        }

        // Populate the start and end times of the observation.
        // Start= start of first timestep
        // End  = start of last timestep + integration time
        let (start_unix_time_ms, end_unix_time_ms, duration_ms) = {
            let o = determine_obs_times(&gpubox_info.time_map, metafits_context.corr_int_time_ms)?;
            (o.start_millisec, o.end_millisec, o.duration_millisec)
        };

        // Convert the real start and end times to GPS time
        let start_gps_time_ms = misc::convert_unixtime_to_gpstime(
            start_unix_time_ms,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
        );
        let end_gps_time_ms = misc::convert_unixtime_to_gpstime(
            end_unix_time_ms,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
        );

        // Prepare the conversion array to convert legacy correlator format into mwax format
        // or just leave it empty if we're in any other format
        let legacy_conversion_table: Vec<LegacyConversionBaseline> = match gpubox_info.corr_format {
            CorrelatorVersion::OldLegacy | CorrelatorVersion::Legacy => {
                convert::generate_conversion_array(&mut metafits_context.rf_inputs.clone())
            }
            _ => Vec::new(),
        };

        Ok(CorrelatorContext {
            metafits_context,
            corr_version: gpubox_info.corr_format,
            start_unix_time_ms,
            end_unix_time_ms,
            start_gps_time_ms,
            end_gps_time_ms,
            duration_ms,
            num_timesteps,
            timesteps,
            num_coarse_chans,
            coarse_chans,
            bandwidth_hz,
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
    /// [baseline][frequency][pol][r][i]
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
        &mut self,
        timestep_index: usize,
        coarse_chan_index: usize,
    ) -> Result<Vec<f32>, GpuboxError> {
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

        // Output buffer for read in data
        let output_buffer: Vec<f32>;

        // Lookup the coarse channel we need
        let coarse_chan = self.coarse_chans[coarse_chan_index].gpubox_number;
        let (batch_index, hdu_index) =
            self.gpubox_time_map[&self.timesteps[timestep_index].unix_time_ms][&coarse_chan];

        if self.gpubox_batches.is_empty() {
            return Err(GpuboxError::NoGpuboxes);
        }
        let mut fptr =
            fits_open!(&self.gpubox_batches[batch_index].gpubox_files[coarse_chan_index].filename)?;
        let hdu = fits_open_hdu!(&mut fptr, hdu_index)?;
        output_buffer = get_fits_image!(&mut fptr, &hdu)?;
        // If legacy correlator, then convert the HDU into the correct output format
        if self.corr_version == CorrelatorVersion::OldLegacy
            || self.corr_version == CorrelatorVersion::Legacy
        {
            // Prepare temporary buffer, if we are reading legacy correlator files
            let mut temp_buffer = vec![
                0.;
                self.metafits_context.num_corr_fine_chans_per_coarse
                    * self.metafits_context.num_visibility_pols
                    * self.metafits_context.num_baselines
                    * 2
            ];

            convert::convert_legacy_hdu_to_mwax_baseline_order(
                &self.legacy_conversion_table,
                &output_buffer,
                &mut temp_buffer,
                self.metafits_context.num_corr_fine_chans_per_coarse,
            );

            Ok(temp_buffer)
        } else {
            Ok(output_buffer)
        }
    }

    /// Read a single timestep for a single coarse channel
    /// The output visibilities are in order:
    /// [frequency][baseline][pol][r][i]
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
    /// * A Result containing vector of 32 bit floats containing the data in [frequency][baseline][pol][r][i] order, if Ok.
    ///
    ///
    pub fn read_by_frequency(
        &mut self,
        timestep_index: usize,
        coarse_chan_index: usize,
    ) -> Result<Vec<f32>, GpuboxError> {
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

        // Output buffer for read in data
        let output_buffer: Vec<f32>;

        // Prepare temporary buffer
        let mut temp_buffer = vec![
            0.;
            self.metafits_context.num_corr_fine_chans_per_coarse
                * self.metafits_context.num_visibility_pols
                * self.metafits_context.num_baselines
                * 2
        ];

        // Lookup the coarse channel we need
        let coarse_chan = self.coarse_chans[coarse_chan_index].gpubox_number;
        let (batch_index, hdu_index) =
            self.gpubox_time_map[&self.timesteps[timestep_index].unix_time_ms][&coarse_chan];

        if self.gpubox_batches.is_empty() {
            return Err(GpuboxError::NoGpuboxes);
        }
        let mut fptr =
            fits_open!(&self.gpubox_batches[batch_index].gpubox_files[coarse_chan_index].filename)?;
        let hdu = fits_open_hdu!(&mut fptr, hdu_index)?;
        output_buffer = get_fits_image!(&mut fptr, &hdu)?;
        // If legacy correlator, then convert the HDU into the correct output format
        if self.corr_version == CorrelatorVersion::OldLegacy
            || self.corr_version == CorrelatorVersion::Legacy
        {
            convert::convert_legacy_hdu_to_mwax_frequency_order(
                &self.legacy_conversion_table,
                &output_buffer,
                &mut temp_buffer,
                self.metafits_context.num_corr_fine_chans_per_coarse,
            );

            Ok(temp_buffer)
        } else {
            // Do conversion for mwax (it is in baseline order, we want it in freq order)
            convert::convert_mwax_hdu_to_frequency_order(
                &output_buffer,
                &mut temp_buffer,
                self.metafits_context.num_baselines,
                self.metafits_context.num_corr_fine_chans_per_coarse,
                self.metafits_context.num_visibility_pols,
            );

            Ok(temp_buffer)
        }
    }

    /// Validates the first HDU of a gpubox file against metafits metadata
    ///
    /// In this case we call `validate_hdu_axes()`
    ///
    /// # Arguments
    ///
    /// * `corr_version` - Correlator version of this gpubox file.
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
        corr_version: CorrelatorVersion,
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
            corr_version,
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
    /// * `corr_version` - Correlator version of this gpubox file.
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
        corr_version: CorrelatorVersion,
        metafits_fine_chans_per_coarse: usize,
        metafits_baselines: usize,
        visibility_pols: usize,
        naxis1: usize,
        naxis2: usize,
    ) -> Result<(), GpuboxError> {
        // We have different values depending on the version of the correlator
        match corr_version {
            CorrelatorVersion::OldLegacy | CorrelatorVersion::Legacy => {
                // NAXIS1 = baselines * visibility_pols * 2
                // NAXIS2 = fine channels
                let calculated_naxis1: i32 = metafits_baselines as i32 * visibility_pols as i32 * 2;
                let calculated_naxis2: i32 = metafits_fine_chans_per_coarse as i32;

                if calculated_naxis1 != naxis1 as i32 {
                    return Err(GpuboxError::LegacyNAXIS1Mismatch {
                        naxis1,
                        calculated_naxis1,
                        metafits_baselines,
                        visibility_pols,
                        naxis2,
                    });
                }
                if calculated_naxis2 != naxis2 as i32 {
                    return Err(GpuboxError::LegacyNAXIS2Mismatch {
                        naxis2,
                        calculated_naxis2,
                        metafits_fine_chans_per_coarse,
                    });
                }
            }
            CorrelatorVersion::V2 => {
                // NAXIS1 = fine channels * visibility pols * 2
                // NAXIS2 = baselines
                let calculated_naxis1: i32 =
                    metafits_fine_chans_per_coarse as i32 * visibility_pols as i32 * 2;
                let calculated_naxis2: i32 = metafits_baselines as i32;

                if calculated_naxis1 != naxis1 as i32 {
                    return Err(GpuboxError::MwaxNAXIS1Mismatch {
                        naxis1,
                        calculated_naxis1,
                        metafits_fine_chans_per_coarse,
                        visibility_pols,
                        naxis2,
                    });
                }
                if calculated_naxis2 != naxis2 as i32 {
                    return Err(GpuboxError::MwaxNAXIS2Mismatch {
                        naxis2,
                        calculated_naxis2,
                        metafits_baselines,
                    });
                }
            }
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
            Metafits Context:         {metafits_context}
            Correlator version:       {corr_ver},

            Actual UNIX start time:   {start_unix},
            Actual UNIX end time:     {end_unix},
            Actual GPS start time:    {start_gps},
            Actual GPS end time:      {end_gps},
            Actual duration:          {duration} s,

            num timesteps:            {n_timesteps},
            timesteps:                {timesteps:?},           

            observation bandwidth:    {obw} MHz,
            num coarse channels,      {n_coarse},
            coarse channels:          {coarse:?},            

            gpubox HDU size:          {hdu_size} MiB,
            Memory usage per scan:    {scan_size} MiB,

            gpubox batches:           {batches:#?},
        )"#,
            metafits_context = self.metafits_context,
            corr_ver = self.corr_version,
            start_unix = self.start_unix_time_ms as f64 / 1e3,
            end_unix = self.end_unix_time_ms as f64 / 1e3,
            start_gps = self.start_gps_time_ms as f64 / 1e3,
            end_gps = self.end_gps_time_ms as f64 / 1e3,
            duration = self.duration_ms as f64 / 1e3,
            n_timesteps = self.num_timesteps,
            timesteps = self.timesteps,
            obw = self.bandwidth_hz as f64 / 1e6,
            n_coarse = self.num_coarse_chans,
            coarse = self.coarse_chans,
            hdu_size = size,
            scan_size = size * self.num_gpubox_files as f64,
            batches = self.gpubox_batches,
        )
    }
}
