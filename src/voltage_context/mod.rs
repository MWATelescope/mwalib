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
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

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
    /// length in millseconds of each timestep
    pub timestep_duration_milliseconds: u64,
    /// Number of samples in each timestep
    pub num_samples_per_timestep: usize,
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
        // Populate coarse channels
        let (coarse_channels, num_coarse_channels, coarse_channel_width_hz) =
            coarse_channel::CoarseChannel::populate_voltage_coarse_channels(
                &mut metafits_fptr,
                &metafits_hdu,
                voltage_info.corr_format,
                metafits_context.observation_bandwidth_hz,
                &voltage_info.time_map,
            )?;

        // Fine-channel resolution. MWA Legacy is 10 kHz, MWAX is 1.28 MHz (unchannelised)
        let fine_channel_width_hz: u32 = match voltage_info.corr_format {
            CorrelatorVersion::Legacy => 10_000,
            CorrelatorVersion::OldLegacy => 10_000,
            CorrelatorVersion::V2 => {
                metafits_context.observation_bandwidth_hz
                    / metafits_context.num_coarse_channels as u32
            }
        };

        // Determine the number of fine channels per coarse channel.
        let num_fine_channels_per_coarse =
            (coarse_channel_width_hz / fine_channel_width_hz) as usize;

        let bandwidth_hz = (num_coarse_channels as u32) * coarse_channel_width_hz;

        // We can unwrap here because the `voltage_time_map` can't be empty if
        // `voltages` isn't empty.
        let timesteps = TimeStep::populate_voltage_timesteps(
            start_gps_time_milliseconds,
            end_gps_time_milliseconds,
            voltage_info.voltage_file_interval_milliseconds,
        );

        // The number of samples this timestep represents. For correlator, this would be 1. For voltage capture it will be many.
        let num_samples_per_timestep = match voltage_info.corr_format {
            CorrelatorVersion::OldLegacy => 1, // TODO: find out the legacy VCS value
            CorrelatorVersion::Legacy => 1,    // TODO: find out the legacy VCS value
            CorrelatorVersion::V2 => 160,
        };

        // Length of this timestep in milliseconds
        let timestep_duration_milliseconds = match voltage_info.corr_format {
            CorrelatorVersion::OldLegacy => 1000,
            CorrelatorVersion::Legacy => 1000,
            CorrelatorVersion::V2 => 8000,
        };

        // Get number of timesteps
        let num_timesteps = timesteps.len();
        Ok(VoltageContext {
            metafits_context,
            corr_version: voltage_info.corr_format,
            start_gps_time_milliseconds,
            end_gps_time_milliseconds,
            duration_milliseconds,
            num_timesteps,
            timesteps,
            num_samples_per_timestep,
            timestep_duration_milliseconds,
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

    /// Read a single gps time / coarse channel worth of data
    /// The output data are in order:
    /// antenna[0]|pol[0]|s[0]...s[63999]|pol[0]
    /// Each sample is a byte.
    ///
    /// # Arguments
    ///
    /// * `timestep_index` - index within the timestep array for the desired timestep. This corresponds
    ///                      to the element within VoltageContext.timesteps.
    ///
    /// * `coarse_channel_index` - index within the coarse_channel array for the desired coarse channel. This corresponds
    ///                      to the element within VoltageContext.coarse_channels.
    ///
    ///
    /// # Returns
    ///
    /// * A Result containing vector of bytes containing the data in antenna[0]|pol[0]|s[0]...s[63999]|pol[0] order, if Ok.
    ///
    ///
    pub fn read(
        &mut self,
        timestep_index: usize,
        coarse_channel_index: usize,
    ) -> Result<Vec<u8>, VoltageFileError> {
        if self.voltage_batches.is_empty() {
            return Err(VoltageFileError::NoVoltageFiles);
        }

        // Lookup the coarse channel we need
        let coarse_channel = self.coarse_channels[coarse_channel_index].receiver_channel_number;

        // Lookup the timestep we need
        let timestep = self.timesteps[timestep_index].gps_time_milliseconds;

        // Get the filename for this timestep and coarse channel
        let filename: &String = &self.voltage_time_map[&timestep][&coarse_channel];

        // Open the file
        let mut file_handle = File::open(&filename).expect("no file found");

        // Obtain metadata
        let metadata = std::fs::metadata(&filename).expect("unable to read metadata");

        // Create an output buffer big enough
        let mut buffer = vec![0; metadata.len() as usize];

        // Determine delay_block_size
        let delay_block_size: u64 = self.metafits_context.num_rf_inputs as u64 * 128000;

        // Determine where the data starts
        let skip_pos = match self.corr_version {
            CorrelatorVersion::V2 => 4096 + delay_block_size,
            CorrelatorVersion::Legacy => 0,
            CorrelatorVersion::OldLegacy => 0,
        };

        // Skip to the data
        file_handle
            .seek(SeekFrom::Start(skip_pos))
            .expect("Unable to seek to data in voltage file");

        // Read data into the buffer
        file_handle.read(&mut buffer).expect("buffer overflow");

        Ok(buffer)
    }
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
            num samples / ts:         {num_samples_per_timestep},
            timestep duration ms,     {timestep_duration_milliseconds}

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
            num_samples_per_timestep = self.num_samples_per_timestep,
            timestep_duration_milliseconds = self.timestep_duration_milliseconds,
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
    use std::fs::File;
    use std::io::{Error, Write};
    // Helper fuction to generate (small) test voltage files
    fn generate_test_voltage_file(
        temp_dir: &tempdir::TempDir,
        filename: &str,
        time_samples: usize,
        rf_inputs: usize,
    ) -> Result<String, Error> {
        let tdir_path = temp_dir.path();
        let full_filename = tdir_path.join(filename);

        let mut output_file = File::create(&full_filename)?;
        // Write out x time samples
        // Each sample has x rfinputs
        // and 1 float for real 1 float for imaginary
        let floats = time_samples * rf_inputs * 2;
        let mut buffer: Vec<f32> = vec![0.0; floats];

        let mut bptr: usize = 0;

        // This will write out the sequence:
        // 0.25, 0.75, 1.25, 1.75..511.25,511.75  (1024 floats in all)
        for t in 0..time_samples {
            for r in 0..rf_inputs {
                // real
                buffer[bptr] = ((t * rf_inputs) + r) as f32 + 0.25;
                bptr += 1;
                // imag
                buffer[bptr] = ((t * rf_inputs) + r) as f32 + 0.75;
                bptr += 1;
            }
        }
        output_file.write_all(misc::as_u8_slice(buffer.as_slice()))?;
        output_file.flush()?;

        Ok(String::from(full_filename.to_str().unwrap()))
    }

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

    #[test]
    fn test_context_new_invalid_metafits() {
        let metafits_filename = "invalid.metafits";
        let filename = "test_files/1101503312_1_timestep/1101503312_1101503312_ch123.dat";
        let voltage_files = vec![filename];

        // No gpubox files provided
        let context = VoltageContext::new(&metafits_filename, &voltage_files);

        assert!(context.is_err());
    }

    #[test]
    fn test_context_legacy_v1() {
        // Open the test mwax file
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
        // Create some test files
        // Create a temp dir for the temp files
        // Once out of scope the temp dir and it's contents will be deleted
        let temp_dir = tempdir::TempDir::new("voltage_test").unwrap();

        // Populate vector of filenames
        let mut temp_filenames: Vec<&str> = Vec::new();
        let tvf1 = generate_test_voltage_file(&temp_dir, "1101503312_1101503312_ch123.dat", 2, 256)
            .unwrap();
        temp_filenames.push(&tvf1);
        let tvf2 = generate_test_voltage_file(&temp_dir, "1101503312_1101503312_ch124.dat", 2, 256)
            .unwrap();
        temp_filenames.push(&tvf2);
        let tvf3 = generate_test_voltage_file(&temp_dir, "1101503312_1101503313_ch123.dat", 2, 256)
            .unwrap();
        temp_filenames.push(&tvf3);
        let tvf4 = generate_test_voltage_file(&temp_dir, "1101503312_1101503313_ch124.dat", 2, 256)
            .unwrap();
        temp_filenames.push(&tvf4);

        //
        // Read the observation using mwalib
        //
        // Open a context and load in a test metafits and gpubox file
        let context = VoltageContext::new(&metafits_filename, &temp_filenames)
            .expect("Failed to create VoltageContext");

        // Test the properties of the context object match what we expect
        // Correlator version:       v1 Legacy,
        assert_eq!(context.corr_version, CorrelatorVersion::Legacy);

        // Actual gps start time:   1_101_503_312,
        assert_eq!(context.start_gps_time_milliseconds, 1_101_503_312_000);

        // Actual gps end time:     1_101_503_314,
        assert_eq!(context.end_gps_time_milliseconds, 1_101_503_314_000);

        // Actual duration:          2 s,
        assert_eq!(context.duration_milliseconds, 2_000);

        // num timesteps:            2,
        assert_eq!(context.num_timesteps, 2);

        // timesteps:
        assert_eq!(
            context.timesteps[0].gps_time_milliseconds,
            1_101_503_312_000
        );
        assert_eq!(
            context.timesteps[1].gps_time_milliseconds,
            1_101_503_313_000
        );

        // num coarse channels,      2,
        assert_eq!(context.num_coarse_channels, 2);

        // observation bandwidth:    2.56 MHz,
        assert_eq!(context.bandwidth_hz, 1_280_000 * 2);

        // coarse channels:
        assert_eq!(context.coarse_channels[0].receiver_channel_number, 123);
        assert_eq!(context.coarse_channels[0].channel_centre_hz, 157_440_000);
        assert_eq!(context.coarse_channels[1].receiver_channel_number, 124);
        assert_eq!(context.coarse_channels[1].channel_centre_hz, 158_720_000);
        // fine channel resolution:  10 kHz,
        assert_eq!(context.fine_channel_width_hz, 10_000);
        // num fine channels/coarse: 128,
        assert_eq!(context.num_fine_channels_per_coarse, 128);
        assert_eq!(context.voltage_batches.len(), 2);
    }

    #[test]
    fn test_context_mwax_v2() {
        // Open the test mwax file
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";
        // Create some test files
        // Create a temp dir for the temp files
        // Once out of scope the temp dir and it's contents will be deleted
        let temp_dir = tempdir::TempDir::new("voltage_test").unwrap();

        // Populate vector of filenames
        let mut temp_filenames: Vec<&str> = Vec::new();
        let tvf1 =
            generate_test_voltage_file(&temp_dir, "1101503312_1101503312_123.sub", 2, 256).unwrap();
        temp_filenames.push(&tvf1);
        let tvf2 =
            generate_test_voltage_file(&temp_dir, "1101503312_1101503312_124.sub", 2, 256).unwrap();
        temp_filenames.push(&tvf2);
        let tvf3 =
            generate_test_voltage_file(&temp_dir, "1101503312_1101503320_123.sub", 2, 256).unwrap();
        temp_filenames.push(&tvf3);
        let tvf4 =
            generate_test_voltage_file(&temp_dir, "1101503312_1101503320_124.sub", 2, 256).unwrap();
        temp_filenames.push(&tvf4);

        //
        // Read the observation using mwalib
        //
        // Open a context and load in a test metafits and gpubox file
        let context = VoltageContext::new(&metafits_filename, &temp_filenames)
            .expect("Failed to create VoltageContext");

        // Test the properties of the context object match what we expect
        // Correlator version:       v2 mwax,
        assert_eq!(context.corr_version, CorrelatorVersion::V2);

        // Actual gps start time:   1_101_503_312,
        assert_eq!(context.start_gps_time_milliseconds, 1_101_503_312_000);

        // Actual gps end time:     1_101_503_328,
        assert_eq!(context.end_gps_time_milliseconds, 1_101_503_328_000);

        // Actual duration:          16 s,
        assert_eq!(context.duration_milliseconds, 16_000);

        // num timesteps:            2,
        assert_eq!(context.num_timesteps, 2);

        // timesteps:
        assert_eq!(
            context.timesteps[0].gps_time_milliseconds,
            1_101_503_312_000
        );
        assert_eq!(
            context.timesteps[1].gps_time_milliseconds,
            1_101_503_320_000
        );

        // num coarse channels,      2,
        assert_eq!(context.num_coarse_channels, 2);

        // observation bandwidth:    2.56 MHz,
        assert_eq!(context.bandwidth_hz, 1_280_000 * 2);

        // coarse channels:
        assert_eq!(context.coarse_channels[0].receiver_channel_number, 123);
        assert_eq!(context.coarse_channels[0].channel_centre_hz, 157_440_000);
        assert_eq!(context.coarse_channels[1].receiver_channel_number, 124);
        assert_eq!(context.coarse_channels[1].channel_centre_hz, 158_720_000);
        // fine channel resolution:  1.28 MHz,
        assert_eq!(context.fine_channel_width_hz, 1_280_000);
        // num fine channels/coarse: 1,
        assert_eq!(context.num_fine_channels_per_coarse, 1);
        assert_eq!(context.voltage_batches.len(), 2);
    }
}
/*
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
