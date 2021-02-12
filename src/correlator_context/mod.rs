// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
The main interface to MWA data.
 */
use std::collections::BTreeMap;
use std::fmt;

use crate::baseline::*;
use crate::coarse_channel::*;
use crate::convert::*;
use crate::gpubox_files::*;
use crate::metafits_context::*;
use crate::timestep::*;
use crate::visibility_pol::*;
use crate::*;

///
/// `mwalib` correlator observation context. This represents the basic metadata for a correlator observation.
///
pub struct CorrelatorContext {
    /// Observation Metadata
    pub metafits_context: MetafitsContext,
    /// Version of the correlator format
    pub corr_version: CorrelatorVersion,
    /// The proper start of the observation (the time that is common to all
    /// provided gpubox files).
    pub start_unix_time_milliseconds: u64,
    /// `end_unix_time_milliseconds` is the actual end time of the observation
    /// i.e. start time of last common timestep plus integration time.
    pub end_unix_time_milliseconds: u64,
    /// Total duration of observation (based on gpubox files)
    pub duration_milliseconds: u64,
    /// Number of timesteps in the observation
    pub num_timesteps: usize,
    /// This is an array of all timesteps we have data for
    pub timesteps: Vec<TimeStep>,
    /// Number of baselines stored. This is autos plus cross correlations
    pub num_baselines: usize,
    /// Baslines
    pub baselines: Vec<Baseline>,
    /// Number of polarisation combinations in the visibilities e.g. XX,XY,YX,YY == 4
    pub num_visibility_pols: usize,
    /// Visibility polarisations
    pub visibility_pols: Vec<VisibilityPol>,
    /// Correlator mode dump time
    pub integration_time_milliseconds: u64,
    /// Number of coarse channels after we've validated the input gpubox files
    pub num_coarse_channels: usize,
    /// Vector of coarse channel structs
    pub coarse_channels: Vec<CoarseChannel>,
    /// Total bandwidth of observation (of the coarse channels we have)
    pub bandwidth_hz: u32,
    /// Bandwidth of each coarse channel
    pub coarse_channel_width_hz: u32,
    /// Correlator fine_channel_resolution
    pub fine_channel_width_hz: u32,
    /// Number of fine channels in each coarse channel
    pub num_fine_channels_per_coarse: usize,

    /// `gpubox_batches` *must* be sorted appropriately. See
    /// `gpubox::determine_gpubox_batches`. The order of the filenames
    /// corresponds directly to other gpubox-related objects
    /// (e.g. `gpubox_hdu_limits`). Structured:
    /// `gpubox_batches[batch][filename]`.
    pub gpubox_batches: Vec<GPUBoxBatch>,

    /// We assume as little as possible about the data layout in the gpubox
    /// files; here, a `BTreeMap` contains each unique UNIX time from every
    /// gpubox, which is associated with another `BTreeMap`, associating each
    /// gpubox number with a gpubox batch number and HDU index. The gpubox
    /// number, batch number and HDU index are everything needed to find the
    /// correct HDU out of all gpubox files.
    pub gpubox_time_map: BTreeMap<u64, BTreeMap<usize, (usize, usize)>>,

    /// The number of bytes taken up by a scan/timestep in each gpubox file.
    pub num_timestep_coarse_channel_bytes: usize,
    /// The number of floats in each gpubox HDU.
    pub num_timestep_coarse_channel_floats: usize,
    /// This is the number of gpubox files *per batch*.
    pub num_gpubox_files: usize,
    /// A conversion table to optimise reading of legacy MWA HDUs
    pub legacy_conversion_table: Vec<LegacyConversionBaseline>,
}

impl CorrelatorContext {
    /// From a path to a metafits file and paths to gpubox files, create an `CorrelatorContext`.
    ///
    /// The traits on the input parameters allow flexibility to input types.
    ///
    /// # Arguments
    ///
    /// * `metafits` - filename of metafits file as a path or string.
    ///
    /// * `gpuboxes` - slice of filenames of gpubox files as paths or strings.
    ///
    ///
    /// # Returns
    ///
    /// * Result containing a populated mwalibContext object if Ok.
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

        // We need to get the correlator integration time
        let integration_time_milliseconds: u64 = {
            let it: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "INTTIME")?;
            (it * 1000.) as _
        };

        let (gpubox_info, timesteps) =
        // Do gpubox stuff only if we have gpubox files.
            if !gpubox_filenames.is_empty() {
                let gpubox_info = examine_gpubox_files(&gpubox_filenames)?;
                // We can unwrap here because the `gpubox_time_map` can't be empty if
                // `gpuboxes` isn't empty.
                let timesteps = TimeStep::populate_correlator_timesteps(&gpubox_info.time_map).unwrap();
                (gpubox_info, timesteps)
            } else {
                // If there are no gpubox files, then we need to use metafits info.
                let nscans: u64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "NSCANS")?;
                let timesteps: Vec<TimeStep> = (0..nscans)
                    .map(|i| {
                        let time = metafits_context.good_time_unix_milliseconds + i * integration_time_milliseconds;
                        TimeStep::new(time, 0)
                    })
                    .collect();

                // Make a fake `gpubox_time_map`.
                let channels: Vec<usize> = {
                    let s: String = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "CHANSEL")?;
                    s.replace(&['\'', '&'][..], "")
                        .split(',')
                        .map(|s| s.parse::<usize>().unwrap() + 1)
                        .collect()
                };

                let mut gpubox_time_map = BTreeMap::new();
                for (i, time) in timesteps.iter().enumerate() {
                    for channel in &channels {
                        gpubox_time_map
                            .entry(time.unix_time_milliseconds)
                            .or_insert_with(BTreeMap::new)
                            .entry(*channel)
                            .or_insert((0, i));
                    }
                }

                (GpuboxInfo {
                    batches: vec![],
                    corr_format: CorrelatorVersion::Legacy,
                    time_map: gpubox_time_map,
                    hdu_size: 0
                }, timesteps)
            };
        let num_timesteps = timesteps.len();

        // Populate baselines
        let baselines = Baseline::populate_baselines(metafits_context.num_antennas);

        // Populate the pols that come out of the correlator
        let visibility_pols = VisibilityPol::populate_visibility_pols();
        let num_visibility_pols = visibility_pols.len();

        // `num_baselines` is the number of cross-correlations + the number of
        // auto-correlations.
        let num_baselines =
            (metafits_context.num_antennas / 2) * (metafits_context.num_antennas + 1);

        // Populate coarse channels
        let (coarse_channels, num_coarse_channels, coarse_channel_width_hz) =
            coarse_channel::CoarseChannel::populate_correlator_coarse_channels(
                &mut metafits_fptr,
                &metafits_hdu,
                gpubox_info.corr_format,
                metafits_context.observation_bandwidth_hz,
                &gpubox_info.time_map,
            )?;

        let bandwidth_hz = (num_coarse_channels as u32) * coarse_channel_width_hz;

        // Fine-channel resolution. The FINECHAN value in the metafits is in units
        // of kHz - make it Hz.
        let fine_channel_width_hz: u32 = {
            let fc: f64 = get_required_fits_key!(&mut metafits_fptr, &metafits_hdu, "FINECHAN")?;
            (fc * 1000.).round() as _
        };
        // Determine the number of fine channels per coarse channel.
        let num_fine_channels_per_coarse =
            (coarse_channel_width_hz / fine_channel_width_hz) as usize;

        // We have enough information to validate HDU matches metafits
        if !gpubox_filenames.is_empty() {
            let coarse_channel = coarse_channels[0].gpubox_number;
            let (batch_index, _) =
                gpubox_info.time_map[&timesteps[0].unix_time_milliseconds][&coarse_channel];

            let mut fptr = fits_open!(&gpubox_info.batches[batch_index].gpubox_files[0].filename)?;

            CorrelatorContext::validate_first_hdu(
                gpubox_info.corr_format,
                num_fine_channels_per_coarse,
                num_baselines,
                num_visibility_pols,
                &mut fptr,
            )?;
        }

        // Populate the start and end times of the observation.
        // Start= start of first timestep
        // End  = start of last timestep + integration time
        let (start_unix_time_milliseconds, end_unix_time_milliseconds, duration_milliseconds) = {
            let o = determine_obs_times(&gpubox_info.time_map, integration_time_milliseconds)?;
            (o.start_millisec, o.end_millisec, o.duration_millisec)
        };

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
            start_unix_time_milliseconds,
            end_unix_time_milliseconds,
            duration_milliseconds,
            num_timesteps,
            timesteps,
            num_baselines,
            baselines,
            num_visibility_pols,
            visibility_pols,
            num_coarse_channels,
            coarse_channels,
            integration_time_milliseconds,
            fine_channel_width_hz,
            bandwidth_hz,
            coarse_channel_width_hz,
            num_fine_channels_per_coarse,
            gpubox_batches: gpubox_info.batches,
            gpubox_time_map: gpubox_info.time_map,
            num_timestep_coarse_channel_bytes: gpubox_info.hdu_size * 4,
            num_timestep_coarse_channel_floats: gpubox_info.hdu_size,
            num_gpubox_files: gpubox_filenames.len(),
            legacy_conversion_table,
        })
    }

    /// Validates the first HDU of a gpubox file against metafits metadata
    ///
    /// In this case we call `validate_hdu_axes()`
    ///
    /// # Arguments
    ///
    /// * `corr_version` - Correlator version of this gpubox file.
    ///
    /// * `metafits_fine_channels_per_coarse` - the number of fine chan per coarse as calculated using info from metafits.
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
    pub fn validate_first_hdu(
        corr_version: CorrelatorVersion,
        metafits_fine_channels_per_coarse: usize,
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
            metafits_fine_channels_per_coarse,
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
    /// * `metafits_fine_channels_per_coarse` - the number of fine chan per coarse as calculated using info from metafits.
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
    pub fn validate_hdu_axes(
        corr_version: CorrelatorVersion,
        metafits_fine_channels_per_coarse: usize,
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
                let calculated_naxis2: i32 = metafits_fine_channels_per_coarse as i32;

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
                        metafits_fine_channels_per_coarse,
                    });
                }
            }
            CorrelatorVersion::V2 => {
                // NAXIS1 = fine channels * visibility pols * 2
                // NAXIS2 = baselines
                let calculated_naxis1: i32 =
                    metafits_fine_channels_per_coarse as i32 * visibility_pols as i32 * 2;
                let calculated_naxis2: i32 = metafits_baselines as i32;

                if calculated_naxis1 != naxis1 as i32 {
                    return Err(GpuboxError::MwaxNAXIS1Mismatch {
                        naxis1,
                        calculated_naxis1,
                        metafits_fine_channels_per_coarse,
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

    /// Read a single timestep for a single coarse channel
    /// The output visibilities are in order:
    /// [baseline][frequency][pol][r][i]
    ///
    /// # Arguments
    ///
    /// * `timestep_index` - index within the timestep array for the desired timestep. This corresponds
    ///                      to the element within mwalibContext.timesteps.
    ///
    /// * `coarse_channel_index` - index within the coarse_channel array for the desired coarse channel. This corresponds
    ///                      to the element within mwalibContext.coarse_channels.
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
        coarse_channel_index: usize,
    ) -> Result<Vec<f32>, GpuboxError> {
        // Output buffer for read in data
        let output_buffer: Vec<f32>;

        // Prepare temporary buffer, if we are reading legacy correlator files
        let mut temp_buffer = if self.corr_version == CorrelatorVersion::OldLegacy
            || self.corr_version == CorrelatorVersion::Legacy
        {
            vec![
                0.;
                self.num_fine_channels_per_coarse
                    * self.num_visibility_pols
                    * self.num_baselines
                    * 2
            ]
        } else {
            Vec::new()
        };

        // Lookup the coarse channel we need
        let coarse_channel = self.coarse_channels[coarse_channel_index].gpubox_number;
        let (batch_index, hdu_index) = self.gpubox_time_map
            [&self.timesteps[timestep_index].unix_time_milliseconds][&coarse_channel];

        if self.gpubox_batches.is_empty() {
            return Err(GpuboxError::NoGpuboxes);
        }
        let mut fptr = fits_open!(
            &self.gpubox_batches[batch_index].gpubox_files[coarse_channel_index].filename
        )?;
        let hdu = fits_open_hdu!(&mut fptr, hdu_index)?;
        output_buffer = get_fits_image!(&mut fptr, &hdu)?;
        // If legacy correlator, then convert the HDU into the correct output format
        if self.corr_version == CorrelatorVersion::OldLegacy
            || self.corr_version == CorrelatorVersion::Legacy
        {
            convert::convert_legacy_hdu_to_mwax_baseline_order(
                &self.legacy_conversion_table,
                &output_buffer,
                &mut temp_buffer,
                self.num_fine_channels_per_coarse,
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
    /// * `coarse_channel_index` - index within the coarse_channel array for the desired coarse channel. This corresponds
    ///                      to the element within mwalibContext.coarse_channels.
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
        coarse_channel_index: usize,
    ) -> Result<Vec<f32>, GpuboxError> {
        // Output buffer for read in data
        let output_buffer: Vec<f32>;

        // Prepare temporary buffer, if we are reading legacy correlator files
        let mut temp_buffer = vec![
            0.;
            self.num_fine_channels_per_coarse
                * self.num_visibility_pols
                * self.num_baselines
                * 2
        ];

        // Lookup the coarse channel we need
        let coarse_channel = self.coarse_channels[coarse_channel_index].gpubox_number;
        let (batch_index, hdu_index) = self.gpubox_time_map
            [&self.timesteps[timestep_index].unix_time_milliseconds][&coarse_channel];

        if self.gpubox_batches.is_empty() {
            return Err(GpuboxError::NoGpuboxes);
        }
        let mut fptr = fits_open!(
            &self.gpubox_batches[batch_index].gpubox_files[coarse_channel_index].filename
        )?;
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
                self.num_fine_channels_per_coarse,
            );

            Ok(temp_buffer)
        } else {
            // Do conversion for mwax (it is in baseline order, we want it in freq order)
            convert::convert_mwax_hdu_to_frequency_order(
                &output_buffer,
                &mut temp_buffer,
                self.num_baselines,
                self.num_fine_channels_per_coarse,
                self.num_visibility_pols,
            );

            Ok(temp_buffer)
        }
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
#[cfg(not(tarpaulin_include))]
impl fmt::Display for CorrelatorContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // `size` is the number of floats (self.gpubox_hdu_size) multiplied by 4
        // bytes per float, divided by 1024^2 to get MiB.
        let size = (self.num_timestep_coarse_channel_floats * 4) as f64 / (1024 * 1024) as f64;

        writeln!(
            f,
            r#"CorrelatorContext (
            Metafits Context:         {metafits_context}
            Correlator version:       {corr_ver},

            Actual UNIX start time:   {start_unix},
            Actual UNIX end time:     {end_unix},
            Actual duration:          {duration} s,

            num timesteps:            {n_timesteps},
            timesteps:                {timesteps:?},

            num baselines:            {n_bls},
            baselines:                {bl01} v {bl02} to {bll1} v {bll2}
            num auto-correlations:    {n_ants},
            num cross-correlations:   {n_ccs},

            num visibility pols:      {n_vps},
            visibility pols:          {vp0}, {vp1}, {vp2}, {vp3},

            observation bandwidth:    {obw} MHz,
            num coarse channels,      {n_coarse},
            coarse channels:          {coarse:?},

            Correlator Mode:
            fine channel resolution:  {fcw} kHz,
            integration time:         {int_time:.2} s
            num fine channels/coarse: {nfcpc},

            gpubox HDU size:          {hdu_size} MiB,
            Memory usage per scan:    {scan_size} MiB,

            gpubox batches:           {batches:#?},
        )"#,
            metafits_context = self.metafits_context,
            corr_ver = self.corr_version,
            start_unix = self.start_unix_time_milliseconds as f64 / 1e3,
            end_unix = self.end_unix_time_milliseconds as f64 / 1e3,
            duration = self.duration_milliseconds as f64 / 1e3,
            n_timesteps = self.num_timesteps,
            timesteps = self.timesteps,
            n_bls = self.num_baselines,
            bl01 = self.baselines[0].antenna1_index,
            bl02 = self.baselines[0].antenna2_index,
            bll1 = self.baselines[self.num_baselines - 1].antenna1_index,
            bll2 = self.baselines[self.num_baselines - 1].antenna2_index,
            n_ants = self.metafits_context.num_antennas,
            n_ccs = self.num_baselines - self.metafits_context.num_antennas,
            n_vps = self.num_visibility_pols,
            vp0 = self.visibility_pols[0].polarisation,
            vp1 = self.visibility_pols[1].polarisation,
            vp2 = self.visibility_pols[2].polarisation,
            vp3 = self.visibility_pols[3].polarisation,
            obw = self.bandwidth_hz as f64 / 1e6,
            n_coarse = self.num_coarse_channels,
            coarse = self.coarse_channels,
            fcw = self.fine_channel_width_hz as f64 / 1e3,
            int_time = self.integration_time_milliseconds as f64 / 1e3,
            nfcpc = self.num_fine_channels_per_coarse,
            hdu_size = size,
            scan_size = size * self.num_gpubox_files as f64,
            batches = self.gpubox_batches,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::*;

    #[test]
    fn test_context_new_missing_gpubox_files() {
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
        let gpuboxfiles = Vec::new();

        // No gpubox files provided
        let context = CorrelatorContext::new(&metafits_filename, &gpuboxfiles);
        assert!(context.is_ok());
        let context = context.unwrap();

        assert!(context.gpubox_batches.is_empty());
    }

    #[test]
    fn test_context_new_invalid_metafits() {
        let metafits_filename = "invalid.metafits";
        let filename =
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";
        let gpuboxfiles = vec![filename];

        // No gpubox files provided
        let context = CorrelatorContext::new(&metafits_filename, &gpuboxfiles);

        assert!(context.is_err());
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn test_context_legacy_v1() {
        // Open the test mwax file
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
        let filename =
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";

        //
        // Read the observation using mwalib
        //
        // Open a context and load in a test metafits and gpubox file
        let gpuboxfiles = vec![filename];
        let context = CorrelatorContext::new(&metafits_filename, &gpuboxfiles)
            .expect("Failed to create mwalibContext");

        // Test the properties of the context object match what we expect
        // Correlator version:       v1 Legacy,
        assert_eq!(context.corr_version, CorrelatorVersion::Legacy);

        // Actual UNIX start time:   1417468096,
        assert_eq!(context.start_unix_time_milliseconds, 1_417_468_096_000);

        // Actual UNIX end time:     1417468098,
        assert_eq!(context.end_unix_time_milliseconds, 1_417_468_098_000);

        // Actual duration:          2 s,
        assert_eq!(context.duration_milliseconds, 2000);

        // num timesteps:            1,
        assert_eq!(context.num_timesteps, 1);

        // timesteps:                [unix=1417468096.000],
        assert_eq!(
            context.timesteps[0].unix_time_milliseconds,
            1_417_468_096_000
        );

        // num baselines:            8256,
        assert_eq!(context.num_baselines, 8256);

        // num visibility pols:      4,
        assert_eq!(context.num_visibility_pols, 4);

        // observation bandwidth:    1.28 MHz,
        assert_eq!(context.bandwidth_hz, 1_280_000);

        // num coarse channels,      1,
        assert_eq!(context.num_coarse_channels, 1);

        // coarse channels:          [gpu=1 corr=0 rec=109 @ 139.520 MHz],
        assert_eq!(context.coarse_channels[0].gpubox_number, 1);
        assert_eq!(context.coarse_channels[0].receiver_channel_number, 109);
        assert_eq!(context.coarse_channels[0].channel_centre_hz, 139_520_000);

        // Correlator Mode:
        // fine channel resolution:  10 kHz,
        assert_eq!(context.fine_channel_width_hz, 10_000);

        // integration time:         2.00 s
        assert_eq!(context.integration_time_milliseconds, 2000);

        // num fine channels/coarse: 128,
        assert_eq!(context.num_fine_channels_per_coarse, 128);

        // gpubox HDU size:          32.25 MiB,
        // Memory usage per scan:    32.25 MiB,

        // metafits filename:        ../test_files/1101503312_1_timestep/1101503312.metafits,
        // gpubox batches:           [
        // batch_number=0 gpubox_files=[filename=../test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits channelidentifier=1]
    }

    #[test]
    fn test_mwax_read() {
        // Open the test mwax file
        // a) directly using Fits  (data will be ordered [baseline][freq][pol][r][i])
        // b) using mwalib (by baseline) (data will be ordered the same as the raw fits file)
        // c) using mwalib (by frequency) (data will be ordered [freq][baseline][pol][r][i])
        // Then check b) is the same as a) and
        // that c) is the same size and sum as a) and b)
        let mwax_metafits_filename = "test_files/1244973688_1_timestep/1244973688.metafits";
        let mwax_filename =
            "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits";

        //
        // Read the mwax file using FITS
        //
        let mut fptr = fits_open!(&mwax_filename).unwrap();
        let fits_hdu = fits_open_hdu!(&mut fptr, 1).unwrap();

        // Read data from fits hdu into vector
        let fits_hdu_data: Vec<f32> = get_fits_image!(&mut fptr, &fits_hdu).unwrap();
        //
        // Read the mwax file by frequency using mwalib
        //
        // Open a context and load in a test metafits and gpubox file
        let gpuboxfiles = vec![mwax_filename];
        let mut context = CorrelatorContext::new(&mwax_metafits_filename, &gpuboxfiles)
            .expect("Failed to create CorrelatorContext");

        // Read and convert first HDU by baseline
        let mwalib_hdu_data_by_bl: Vec<f32> = context.read_by_baseline(0, 0).expect("Error!");

        // Read and convert first HDU by frequency
        let mwalib_hdu_data_by_freq: Vec<f32> = context.read_by_frequency(0, 0).expect("Error!");

        // First assert that the data vectors are the same size
        assert_eq!(fits_hdu_data.len(), mwalib_hdu_data_by_bl.len());
        assert_eq!(fits_hdu_data.len(), mwalib_hdu_data_by_freq.len());

        // Check all 3 sum to the same value
        let sum_fits: f64 = fits_hdu_data.iter().fold(0., |sum, x| sum + *x as f64);
        let sum_freq: f64 = mwalib_hdu_data_by_freq
            .iter()
            .fold(0., |sum, x| sum + *x as f64);
        let sum_bl: f64 = mwalib_hdu_data_by_bl
            .iter()
            .fold(0., |sum, x| sum + *x as f64);

        // Check sums match
        assert_eq!(
            approx_eq!(f64, sum_fits, sum_freq, F64Margin::default()),
            approx_eq!(f64, sum_fits, sum_bl, F64Margin::default())
        );

        // Check this block of floats matches
        assert_eq!(fits_hdu_data, mwalib_hdu_data_by_bl);
    }

    #[test]
    fn test_validate_first_hdu() {
        // Open the test mwax file
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
        let filename =
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";

        //
        // Read the observation using mwalib
        //
        // Open a context and load in a test metafits and gpubox file
        let gpuboxfiles = vec![filename];
        let context = CorrelatorContext::new(&metafits_filename, &gpuboxfiles)
            .expect("Failed to create CorrelatorContext");

        let coarse_channel = context.coarse_channels[0].gpubox_number;
        let (batch_index, _) =
            context.gpubox_time_map[&context.timesteps[0].unix_time_milliseconds][&coarse_channel];

        let mut fptr =
            fits_open!(&context.gpubox_batches[batch_index].gpubox_files[0].filename).unwrap();

        let result_valid = CorrelatorContext::validate_first_hdu(
            context.corr_version,
            context.num_fine_channels_per_coarse,
            context.num_baselines,
            context.num_visibility_pols,
            &mut fptr,
        );

        let result_invalid1 = CorrelatorContext::validate_first_hdu(
            context.corr_version,
            context.num_fine_channels_per_coarse + 1,
            context.num_baselines,
            context.num_visibility_pols,
            &mut fptr,
        );

        let result_invalid2 = CorrelatorContext::validate_first_hdu(
            context.corr_version,
            context.num_fine_channels_per_coarse,
            context.num_baselines + 1,
            context.num_visibility_pols,
            &mut fptr,
        );

        let result_invalid3 = CorrelatorContext::validate_first_hdu(
            context.corr_version,
            context.num_fine_channels_per_coarse,
            context.num_baselines,
            context.num_visibility_pols + 1,
            &mut fptr,
        );

        // This is valid
        assert!(result_valid.is_ok());

        assert!(result_invalid1.is_err());

        assert!(result_invalid2.is_err());

        assert!(result_invalid3.is_err());
    }
}
