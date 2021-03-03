// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
The main interface to MWA data.
 */
use crate::coarse_channel::*;
use crate::error::*;
use crate::metafits_context::*;
use crate::timestep::*;
use crate::voltage_files::*;
use crate::*;
use std::fmt;

#[cfg(test)]
mod test;

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
    pub start_gps_time_ms: u64,
    /// `end_gps_time_ms` is the actual end time of the observation    
    /// i.e. start time of last common timestep plus length of a voltage file (1 sec for MWA Legacy, 8 secs for MWAX).
    pub end_gps_time_ms: u64,
    /// `start_gps_time_ms` but in UNIX time (milliseconds)
    pub start_unix_time_ms: u64,
    /// `end_gps_time_ms` but in UNIX time (milliseconds)
    pub end_unix_time_ms: u64,
    /// Total duration of observation (based on voltage files)
    pub duration_ms: u64,
    /// Number of timesteps in the observation
    pub num_timesteps: usize,
    /// length in millseconds of each timestep
    pub timestep_duration_ms: u64,
    /// Number of samples in each timestep
    pub num_samples_per_timestep: usize,
    /// This is an array of all timesteps we have data for
    pub timesteps: Vec<TimeStep>,
    /// Number of coarse channels after we've validated the input voltage files
    pub num_coarse_chans: usize,
    /// Vector of coarse channel structs
    pub coarse_chans: Vec<CoarseChannel>,
    /// Total bandwidth of observation (of the coarse channels we have)
    pub bandwidth_hz: u32,
    /// Bandwidth of each coarse channel
    pub coarse_chan_width_hz: u32,
    /// Volatge fine_chan_resolution (if applicable- MWA legacy is 10 kHz, MWAX is unchannelised i.e. the full coarse channel width)
    pub fine_chan_width_hz: u32,
    /// Number of fine channels in each coarse channel
    pub num_fine_chans_per_coarse: usize,

    /// `voltage_batches` *must* be sorted appropriately. See
    /// `voltage::determine_voltage_batches`. The order of the filenames
    /// corresponds directly to other voltage-related objects
    /// (e.g. `voltage_hdu_limits`). Structured:
    /// `voltage_batches[batch][filename]`.
    pub(crate) voltage_batches: Vec<VoltageFileBatch>,

    /// We assume as little as possible about the data layout in the voltage
    /// files; here, a `BTreeMap` contains each unique GPS time from every
    /// voltage file, which is associated with another `BTreeMap`, associating each
    /// voltage number with a voltage batch number and HDU index. The voltage
    /// number, batch number and HDU index are everything needed to find the
    /// correct HDU out of all voltage files.
    pub(crate) voltage_time_map: VoltageFileTimeMap,
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
    /// * Result containing a populated VoltageContext object if Ok.
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
        let (start_gps_time_ms, end_gps_time_ms, duration_ms) = {
            let o = determine_obs_times(
                &voltage_info.time_map,
                voltage_info.voltage_file_interval_ms,
            )?;
            (o.start_gps_time_ms, o.end_gps_time_ms, o.duration_ms)
        };

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
            voltage_info.corr_format,
            &metafits_coarse_chan_vec,
            metafits_coarse_chan_width_hz,
            None,
            Some(&voltage_info.time_map),
        )?;
        let num_coarse_chans = coarse_chans.len();

        // Fine-channel resolution. MWA Legacy is 10 kHz, MWAX is 1.28 MHz (unchannelised)
        let fine_chan_width_hz: u32 = match voltage_info.corr_format {
            CorrelatorVersion::Legacy => 10_000,
            CorrelatorVersion::OldLegacy => 10_000,
            CorrelatorVersion::V2 => {
                metafits_context.obs_bandwidth_hz / metafits_context.num_coarse_chans as u32
            }
        };

        // Determine the number of fine channels per coarse channel.
        let num_fine_chans_per_coarse =
            (metafits_coarse_chan_width_hz / fine_chan_width_hz) as usize;

        let bandwidth_hz = (num_coarse_chans as u32) * metafits_coarse_chan_width_hz;

        // We can unwrap here because the `voltage_time_map` can't be empty if
        // `voltages` isn't empty.
        let timesteps = TimeStep::populate_voltage_timesteps(
            start_gps_time_ms,
            end_gps_time_ms,
            voltage_info.voltage_file_interval_ms,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
        );

        // Convert the real start and end times to UNIX time
        let start_unix_time_ms = misc::convert_gpstime_to_unixtime(
            start_gps_time_ms,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
        );
        let end_unix_time_ms = misc::convert_gpstime_to_unixtime(
            end_gps_time_ms,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
        );

        // The number of samples this timestep represents. For correlator, this would be 1. For voltage capture it will be many.
        let num_samples_per_timestep = match voltage_info.corr_format {
            CorrelatorVersion::OldLegacy => 10000,
            CorrelatorVersion::Legacy => 10000,
            CorrelatorVersion::V2 => 10_240_000, // (64K per 50ms and there are 160 50ms blocks per 8 seconds of MWAX VCS)
        };

        // Length of this timestep in milliseconds
        let timestep_duration_ms = match voltage_info.corr_format {
            CorrelatorVersion::OldLegacy => 1000,
            CorrelatorVersion::Legacy => 1000,
            CorrelatorVersion::V2 => 8000,
        };

        // Get number of timesteps
        let num_timesteps = timesteps.len();
        Ok(VoltageContext {
            metafits_context,
            corr_version: voltage_info.corr_format,
            start_gps_time_ms,
            end_gps_time_ms,
            start_unix_time_ms,
            end_unix_time_ms,
            duration_ms,
            num_timesteps,
            timesteps,
            num_samples_per_timestep,
            timestep_duration_ms,
            num_coarse_chans,
            coarse_chans,
            fine_chan_width_hz,
            bandwidth_hz,
            coarse_chan_width_hz: metafits_coarse_chan_width_hz,
            num_fine_chans_per_coarse,
            voltage_batches: voltage_info.gpstime_batches,
            voltage_time_map: voltage_info.time_map,
        })
    }

    /*
    /// Read a single gps time / coarse channel worth of data
    /// The output data are in order:
    /// antenna[0]|pol[0]|s[0]...s[63999]|pol[0]
    /// Each sample is a byte.
    ///
    /// # Arguments
    ///
    /// * `timestep_index` - index within the timestep array for the desired timestep. This corresponds
    ///                      to the element within VoltageContext.timesteps. For mwa legacy each index
    ///                      represents 1 second increments, for mwax it is 8 second increments.
    ///
    /// * `coarse_chan_index` - index within the coarse_chan array for the desired coarse channel. This corresponds
    ///                      to the element within VoltageContext.coarse_chans.
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
        coarse_chan_index: usize,
    ) -> Result<Vec<u8>, VoltageFileError> {
        if self.voltage_batches.is_empty() {
            return Err(VoltageFileError::NoVoltageFiles);
        }

        // Lookup the coarse channel we need
        let coarse_chan = self.coarse_chans[coarse_chan_index].rec_chan_number;

        // Lookup the timestep we need
        let timestep = self.timesteps[timestep_index].gps_time_ms;

        // Get the filename for this timestep and coarse channel
        let filename: &String = &self.voltage_time_map[&timestep][&coarse_chan];

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
impl fmt::Display for VoltageContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            r#"VoltageContext (
            Metafits Context:         {metafits_context}
            Correlator version:       {corr_ver},

            Actual GPS start time:    {start_gps},
            Actual GPS end time:      {end_gps},
            Actual UNIX start time:   {start_unix},
            Actual UNIX end time:     {end_unix},
            Actual duration:          {duration} s,

            num timesteps:            {n_timesteps},
            timesteps:                {timesteps:?},
            num samples / ts:         {num_samples_per_timestep},
            timestep duration ms,     {timestep_duration_ms}

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
            start_unix = self.start_unix_time_ms as f64 / 1e3,
            end_unix = self.end_unix_time_ms as f64 / 1e3,
            start_gps = self.start_gps_time_ms as f64 / 1e3,
            end_gps = self.end_gps_time_ms as f64 / 1e3,
            duration = self.duration_ms as f64 / 1e3,
            n_timesteps = self.num_timesteps,
            timesteps = self.timesteps,
            num_samples_per_timestep = self.num_samples_per_timestep,
            timestep_duration_ms = self.timestep_duration_ms,
            n_ants = self.metafits_context.num_ants,
            obw = self.bandwidth_hz as f64 / 1e6,
            n_coarse = self.num_coarse_chans,
            coarse = self.coarse_chans,
            fcw = self.fine_chan_width_hz as f64 / 1e3,
            nfcpc = self.num_fine_chans_per_coarse,
            batches = self.voltage_batches,
        )
    }
}
