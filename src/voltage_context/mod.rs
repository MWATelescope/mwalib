// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
The main interface to MWA data.
 */
use std::fmt;

use crate::coarse_channel::*;
use crate::error::*;
use crate::metafits_context::*;
use crate::timestep::*;
use crate::voltage_files::*;
use crate::*;

///
/// `mwalib` voltage captue system (VCS) observation context. This represents the basic metadata for a voltage capture observation.
///
#[derive(Debug)]
pub struct VoltageContext {
    /// Observation Metadata
    pub metafits_context: MetafitsContext,
    /// Version of the correlator format
    pub corr_version: CorrelatorVersion,
    /// The proper start of the observation (the time that is common to all
    /// provided voltage files).
    pub start_gps_time_milliseconds: u64,
    /// `end_gps_time_milliseconds` is the actual end time of the observation    
    /// i.e. start time of last common timestep plus length of a voltage file (1 sec for MWA Legacy, 8 secs for MWAX).
    pub end_gps_time_milliseconds: u64,
    /// Total duration of observation (based on voltage files)
    pub duration_milliseconds: u64,
    /// Number of timesteps in the observation
    pub num_timesteps: usize,
    /// This is an array of all timesteps we have data for
    pub timesteps: Vec<TimeStep>,
    /// Number of coarse channels after we've validated the input voltage files
    pub num_coarse_channels: usize,
    /// Vector of coarse channel structs
    pub coarse_channels: Vec<CoarseChannel>,
    /// Total bandwidth of observation (of the coarse channels we have)
    pub bandwidth_hz: u32,
    /// Bandwidth of each coarse channel
    pub coarse_channel_width_hz: u32,
    /// Volatge fine_channel_resolution (if applicable- MWA legacy is 10 kHz, MWAX is unchannelised i.e. the full coarse channel width)
    pub fine_channel_width_hz: u32,
    /// Number of fine channels in each coarse channel
    pub num_fine_channels_per_coarse: usize,

    /// `voltage_batches` *must* be sorted appropriately. See
    /// `voltage::determine_voltage_batches`. The order of the filenames
    /// corresponds directly to other voltage-related objects
    /// (e.g. `voltage_hdu_limits`). Structured:
    /// `voltage_batches[batch][filename]`.
    pub voltage_batches: Vec<VoltageFileBatch>,

    /// We assume as little as possible about the data layout in the voltage
    /// files; here, a `BTreeMap` contains each unique GPS time from every
    /// voltage file, which is associated with another `BTreeMap`, associating each
    /// voltage number with a voltage batch number and HDU index. The voltage
    /// number, batch number and HDU index are everything needed to find the
    /// correct HDU out of all voltage files.
    pub voltage_time_map: VoltageFileTimeMap,
}

impl VoltageContext {
    /// From a path to a metafits file and paths to voltage files, create an `VoltageContext`.
    ///
    /// The traits on the input parameters allow flexibility to input types.
    ///
    /// # Arguments
    ///
    /// * `metafits` - filename of metafits file as a path or string.
    ///
    /// * `voltages` - slice of filenames of voltage files as paths or strings.
    ///
    ///
    /// # Returns
    ///
    /// * Result containing a populated mwalibContext object if Ok.
    ///
    ///
    pub fn new<T: AsRef<std::path::Path>>(
        metafits_filename: &T,
        voltage_filenames: &[T],
    ) -> Result<Self, MwalibError> {
        let metafits_context = MetafitsContext::new(metafits_filename)?;

        // Re-open metafits file
        let mut metafits_fptr = fits_open!(&metafits_filename)?;
        let metafits_hdu = fits_open_hdu!(&mut metafits_fptr, 0)?;

        // Do voltage stuff only if we have voltage files.
        if voltage_filenames.is_empty() {
            return Err(MwalibError::Voltage(VoltageFileError::NoVoltageFiles));
        }
        let voltage_info = examine_voltage_files(&metafits_context, &voltage_filenames)?;
        // Populate the start and end times of the observation.
        // Start= start of first timestep
        // End  = start of last timestep + integration time
        let (start_gps_time_milliseconds, end_gps_time_milliseconds, duration_milliseconds) = {
            let o = determine_obs_times(
                &voltage_info.time_map,
                voltage_info.voltage_file_interval_milliseconds,
            )?;
            (
                o.start_gps_time_milliseconds,
                o.end_gps_time_milliseconds,
                o.duration_milliseconds,
            )
        };

        // We can unwrap here because the `voltage_time_map` can't be empty if
        // `voltages` isn't empty.
        let timesteps = TimeStep::populate_voltage_timesteps(
            start_gps_time_milliseconds,
            end_gps_time_milliseconds,
            voltage_info.voltage_file_interval_milliseconds,
        )
        .unwrap();
        // Get number of timesteps
        let num_timesteps = timesteps.len();
        // Fine-channel resolution. MWA Legacy is 10 kHz, MWAX is 1.28 MHz (unchannelised)
        let fine_channel_width_hz: u32 = match voltage_info.corr_format {
            CorrelatorVersion::Legacy => 10_000,
            CorrelatorVersion::OldLegacy => 10_000,
            CorrelatorVersion::V2 => metafits_context.observation_bandwidth_hz,
        };
        // Populate coarse channels
        let (coarse_channels, num_coarse_channels, coarse_channel_width_hz) =
            coarse_channel::CoarseChannel::populate_voltage_coarse_channels(
                &mut metafits_fptr,
                &metafits_hdu,
                voltage_info.corr_format,
                metafits_context.observation_bandwidth_hz,
                &voltage_info.time_map,
            )?;

        // Determine the number of fine channels per coarse channel.
        let num_fine_channels_per_coarse =
            (coarse_channel_width_hz / fine_channel_width_hz) as usize;

        let bandwidth_hz = (num_coarse_channels as u32) * coarse_channel_width_hz;
        Ok(VoltageContext {
            metafits_context,
            corr_version: voltage_info.corr_format,
            start_gps_time_milliseconds,
            end_gps_time_milliseconds,
            duration_milliseconds,
            num_timesteps,
            timesteps,
            num_coarse_channels,
            coarse_channels,
            fine_channel_width_hz,
            bandwidth_hz,
            coarse_channel_width_hz,
            num_fine_channels_per_coarse,
            voltage_batches: voltage_info.gpstime_batches,
            voltage_time_map: voltage_info.time_map,
        })
    }

    /*
    /// Validates the PSRDADA header of a voltage file against metafits metadata
    ///
    /// In this case we call `validate_hdu_axes()`
    ///
    /// # Arguments
    ///
    /// * `corr_version` - Correlator version of this voltage file.
    ///
    /// * `metafits_fine_channels_per_coarse` - the number of fine chan per coarse as calculated using info from metafits.
    ///
    /// * `metafits_baselines` - the number of baselines as reported by the metafits file.
    ///
    /// * `visibility_pols` - the number of pols produced by the correlator (always 4 for MWA)
    ///
    /// * `voltage_fptr` - FITSFile pointer to an MWA GPUbox file
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
        voltage_fptr: &mut fitsio::FitsFile,
    ) -> Result<(), GpuboxError> {
        // Get NAXIS1 and NAXIS2 from a voltage file first image HDU
        let hdu = fits_open_hdu!(voltage_fptr, 1)?;
        let dimensions = get_hdu_image_size!(voltage_fptr, &hdu)?;
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

    /// Validates the first HDU of a voltage file against metafits metadata
    ///
    /// In this case we check that NAXIS1 = the correct value and NAXIS2 = the correct value calculated from the metafits
    ///
    /// # Arguments
    ///
    /// * `corr_version` - Correlator version of this voltage file.
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
        let (batch_index, hdu_index) =
            self.gpubox_time_map[&self.timesteps[timestep_index].unix_time_ms][&coarse_channel];

        if self.voltage_batches.is_empty() {
            return Err(GpuboxError::NoGpuboxes);
        }
        let mut fptr = fits_open!(
            &self.voltage_batches[batch_index].gpubox_files[coarse_channel_index].filename
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
        let (batch_index, hdu_index) =
            self.gpubox_time_map[&self.timesteps[timestep_index].unix_time_ms][&coarse_channel];

        if self.voltage_batches.is_empty() {
            return Err(GpuboxError::NoGpuboxes);
        }
        let mut fptr = fits_open!(
            &self.voltage_batches[batch_index].gpubox_files[coarse_channel_index].filename
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
    }*/
}

/// Implements fmt::Display for VoltageContext struct
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
impl fmt::Display for VoltageContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            r#"VoltageContext (
            Metafits Context:         {metafits_context}
            Correlator version:       {corr_ver},

            Actual UNIX start time:   {start_unix},
            Actual UNIX end time:     {end_unix},
            Actual duration:          {duration} s,

            num timesteps:            {n_timesteps},
            timesteps:                {timesteps:?},

            num antennas:             {n_ants},

            observation bandwidth:    {obw} MHz,
            num coarse channels,      {n_coarse},
            coarse channels:          {coarse:?},

            fine channel resolution:  {fcw} Hz,
            num fine channels/coarse: {nfcpc},
            
            voltage batches:          {batches:#?},
        )"#,
            metafits_context = self.metafits_context,
            corr_ver = self.corr_version,
            start_unix = self.start_gps_time_milliseconds as f64 / 1e3,
            end_unix = self.end_gps_time_milliseconds as f64 / 1e3,
            duration = self.duration_milliseconds as f64 / 1e3,
            n_timesteps = self.num_timesteps,
            timesteps = self.timesteps,
            n_ants = self.metafits_context.num_antennas,
            obw = self.bandwidth_hz as f64 / 1e6,
            n_coarse = self.num_coarse_channels,
            coarse = self.coarse_channels,
            fcw = self.fine_channel_width_hz as f64 / 1e3,
            nfcpc = self.num_fine_channels_per_coarse,
            batches = self.voltage_batches,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //use float_cmp::*;

    #[test]
    fn test_context_new_missing_voltage_files() {
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
        let voltagefiles = Vec::new();

        // No gpubox files provided
        let context = VoltageContext::new(&metafits_filename, &voltagefiles);
        assert!(matches!(
            context.unwrap_err(),
            MwalibError::Voltage(VoltageFileError::NoVoltageFiles)
        ));
    }
}

/*
    #[test]
    fn test_context_new_invalid_metafits() {
        let metafits_filename = "invalid.metafits";
        let filename =
            "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits";
        let gpuboxfiles = vec![filename];

        // No gpubox files provided
        let context = VoltageContext::new(&metafits_filename, &gpuboxfiles);

        assert!(context.is_err());
    }
}*/
/*

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
        let context = VoltageContext::new(&metafits_filename, &gpuboxfiles)
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
        assert_eq!(context.timesteps[0].unix_time_ms, 1_417_468_096_000);

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
        let mut context = VoltageContext::new(&mwax_metafits_filename, &gpuboxfiles)
            .expect("Failed to create VoltageContext");

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
        let context = VoltageContext::new(&metafits_filename, &gpuboxfiles)
            .expect("Failed to create VoltageContext");

        let coarse_channel = context.coarse_channels[0].gpubox_number;
        let (batch_index, _) =
            context.gpubox_time_map[&context.timesteps[0].unix_time_ms][&coarse_channel];

        let mut fptr =
            fits_open!(&context.voltage_batches[batch_index].gpubox_files[0].filename).unwrap();

        let result_valid = VoltageContext::validate_first_hdu(
            context.corr_version,
            context.num_fine_channels_per_coarse,
            context.num_baselines,
            context.num_visibility_pols,
            &mut fptr,
        );

        let result_invalid1 = VoltageContext::validate_first_hdu(
            context.corr_version,
            context.num_fine_channels_per_coarse + 1,
            context.num_baselines,
            context.num_visibility_pols,
            &mut fptr,
        );

        let result_invalid2 = VoltageContext::validate_first_hdu(
            context.corr_version,
            context.num_fine_channels_per_coarse,
            context.num_baselines + 1,
            context.num_visibility_pols,
            &mut fptr,
        );

        let result_invalid3 = VoltageContext::validate_first_hdu(
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
*/
