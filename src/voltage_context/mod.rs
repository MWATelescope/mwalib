// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
The main interface to MWA voltage data.
 */
use crate::coarse_channel::*;
use crate::error::*;
use crate::metafits_context::*;
use crate::timestep::*;
use crate::voltage_files::*;
use crate::*;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::io::{Read, Seek, SeekFrom};

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

    /// Number of bytes in each sample (a sample is a complex, thus includes r and i)
    pub sample_size_bytes: u64,
    /// Number of voltage blocks per timestep
    pub num_voltage_blocks_per_timestep: u64,
    /// Number of voltage blocks of samples in each second of data    
    pub num_voltage_blocks_per_second: u64,
    /// Number of samples in each voltage_blocks for each second of data per rf_input * fine_chans * real|imag
    pub num_samples_per_voltage_block: u64,
    /// The size of each voltage block    
    pub voltage_block_size_bytes: u64,
    /// Number of bytes used to store delays - for MWAX this is the same as a voltage block size, for legacy it is 0
    pub delay_block_size_bytes: u64,
    /// The amount of bytes to skip before getting into real data within the voltage files
    pub data_file_header_size_bytes: u64,
    /// Expected voltage file size
    pub expected_voltage_data_file_size_bytes: u64,

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

        // Length of each timestep in milliseconds
        let timestep_duration_ms = match voltage_info.corr_format {
            CorrelatorVersion::OldLegacy => 1000,
            CorrelatorVersion::Legacy => 1000,
            CorrelatorVersion::V2 => 8000,
        };

        // Number of bytes in each sample
        let sample_size_bytes: u64 = match voltage_info.corr_format {
            CorrelatorVersion::OldLegacy => 1, // 4 bits real, 4 bits imag
            CorrelatorVersion::Legacy => 1,    // 4 bits real, 4 bits imag
            CorrelatorVersion::V2 => 2,        // 8 bits real, 8 bits imag
        };

        // Number of voltage blocks per timestep
        let num_voltage_blocks_per_timestep: u64 = match voltage_info.corr_format {
            CorrelatorVersion::OldLegacy => 50,
            CorrelatorVersion::Legacy => 50,
            CorrelatorVersion::V2 => 160,
        };

        // Number of voltage blocks of samples in each second of data
        let num_voltage_blocks_per_second: u64 =
            num_voltage_blocks_per_timestep / (timestep_duration_ms / 1000);

        // Number of samples in each voltage_blocks for each second of data per rf_input * fine_chans * real|imag
        // TODO: verify with VCS team
        let num_samples_per_rf_chain_fine_chan_in_a_voltage_block: u64 =
            match voltage_info.corr_format {
                CorrelatorVersion::OldLegacy => 200,
                CorrelatorVersion::Legacy => 200,
                CorrelatorVersion::V2 => 64_000, // 64000 per rf_inpit x real|imag (no fine chans)
            };

        // The size of each voltage block
        let voltage_block_size_bytes: u64 = num_samples_per_rf_chain_fine_chan_in_a_voltage_block
            * metafits_context.num_rf_inputs as u64
            * num_fine_chans_per_coarse as u64
            * sample_size_bytes;

        // Determine delay_block_size - for MWAX this is the same as a voltage block size, for legacy it is 0
        let delay_block_size_bytes: u64 = match voltage_info.corr_format {
            CorrelatorVersion::OldLegacy => 0,
            CorrelatorVersion::Legacy => 0,
            CorrelatorVersion::V2 => voltage_block_size_bytes,
        };

        // The amount of bytes to skip before getting into real data within the voltage files
        let data_file_header_size_bytes: u64 = match voltage_info.corr_format {
            CorrelatorVersion::OldLegacy => 0,
            CorrelatorVersion::Legacy => 0,
            CorrelatorVersion::V2 => 4096,
        };

        // Expected voltage file size
        // MWAX 128T should be 4096+32768000+(32768000 * 160) == 5275652096
        let expected_voltage_data_file_size_bytes: u64 = data_file_header_size_bytes
            + delay_block_size_bytes
            + (voltage_block_size_bytes * num_voltage_blocks_per_timestep);

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
            timestep_duration_ms,
            num_coarse_chans,
            coarse_chans,
            fine_chan_width_hz,
            bandwidth_hz,
            coarse_chan_width_hz: metafits_coarse_chan_width_hz,
            num_fine_chans_per_coarse,
            sample_size_bytes,
            num_voltage_blocks_per_timestep,
            num_voltage_blocks_per_second,
            num_samples_per_voltage_block: num_samples_per_rf_chain_fine_chan_in_a_voltage_block,
            voltage_block_size_bytes,
            delay_block_size_bytes,
            data_file_header_size_bytes,
            expected_voltage_data_file_size_bytes,
            voltage_batches: voltage_info.gpstime_batches,
            voltage_time_map: voltage_info.time_map,
        })
    }

    /// Read a single timestep / coarse channel worth of data
    /// The output data are in the format:
    /// MWA Recombined VCS:
    /// antenna[0]|pol[0]|finechan[0]|sample[0]...sample[63999]
    /// antenna[0]|pol[0]|finechan[1]|sample[0]...sample[63999]
    /// ...
    /// antenna[0]|pol[1]|finechan[0]|sample[0]...sample[63999]
    /// antenna[0]|pol[1]|finechan[1]|sample[0]...sample[63999]
    /// ...
    /// antenna[1]|pol[0]|finechan[0]|sample[0_real][0_imag]...sample[63999_real][63999_imag]
    /// antenna[1]|pol[0]|finechan[1]|sample[0_real][0_imag]...sample[63999_real][63999_imag]
    ///
    ///
    /// MWAX:
    /// antenna[0]|pol[0]|sample[0]...sample[63999]
    /// antenna[0]|pol[1]|sample[0]...sample[63999]
    /// antenna[1]|pol[0]|sample[0]...sample[63999]
    /// antenna[1]|pol[1]|sample[0]...sample[63999]
    /// ...
    ///
    /// File format information:
    /// type    tiles   pols    fine ch bytes/samp  samples/block   block size  blocks  header  delay size  data size   file size   seconds/file    size/sec
    /// =====================================================================================================================================================
    /// Lgeacy  128     2       128     1           200             6553600     50      0       0           327680000   327680000   1               327680000
    /// MWAX    128     2       1       2           64000           32768000    160     4096    32768000    5242880000  5275652096  8               659456512
    /// NOTE: 'sample' refers to a complex value per tile/pol/chan/time. So legacy stores r/i as a byte (4bits r + 4bits i), mwax as 1 byte real, 1 byte imag.
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
    /// * A Result containing vector of bytes containing the data, if Ok.
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

        // Validate the timestep
        if timestep_index > self.num_timesteps - 1 {
            return Err(VoltageFileError::InvalidTimeStepIndex(
                self.num_timesteps - 1,
            ));
        }

        // Validate the coarse chan
        if coarse_chan_index > self.num_coarse_chans - 1 {
            return Err(VoltageFileError::InvalidCoarseChanIndex(
                self.num_coarse_chans - 1,
            ));
        }

        // Work out how much to read at once
        let chunk_size: usize = self.voltage_block_size_bytes as usize; // This will be the size of a voltage block

        // Get the filename for this timestep and coarse channel
        let filename: &String =
            &self.voltage_batches[timestep_index].voltage_files[coarse_chan_index].filename;

        // Open the file
        let file_handle = File::open(&filename).expect("no file found");

        // Obtain metadata
        let metadata = std::fs::metadata(&filename).expect("unable to read metadata");

        // Check file is as big as we expect
        // normally we would compare the file len to context.expected_voltage_data_file_size_bytes,
        // but in our tests we override the voltage_block_size_bytes because our test files only have 1 tile
        assert_eq!(
            metadata.len(),
            self.data_file_header_size_bytes
                + self.delay_block_size_bytes
                + (self.voltage_block_size_bytes * self.num_voltage_blocks_per_timestep)
        );

        // Create an output buffer big enough
        let mut buffer = vec![
            0;
            (self.voltage_block_size_bytes * self.num_voltage_blocks_per_timestep)
                as usize
        ];

        // Open a buffer reader
        let mut reader = BufReader::with_capacity(chunk_size, file_handle);

        // Skip header
        reader
            .by_ref()
            .seek(SeekFrom::Start(
                self.data_file_header_size_bytes + self.delay_block_size_bytes,
            ))
            .expect("Unable to seek to data in voltage file");

        // Read the data into the final output buffer in blocks, spaced out with delay blocks (possibly)
        let mut start_pos: usize = 0;
        let mut end_pos: usize = chunk_size as usize;

        // Loop until all data is read into our buffer
        for _ in 0..self.num_voltage_blocks_per_timestep {
            let chunk = &mut buffer[start_pos..end_pos];
            let bytes_read = reader.by_ref().read(chunk).expect("Error");

            assert_eq!(bytes_read, chunk_size);

            // Set new start and end pos
            start_pos = end_pos;
            end_pos = start_pos + chunk_size;
        }

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
            timestep duration ms:     {timestep_duration_ms} ms,

            num antennas:             {n_ants},

            observation bandwidth:    {obw} MHz,
            num coarse channels,      {n_coarse},
            coarse channels:          {coarse:?},

            fine channel resolution:  {fcw} Hz,
            num fine channels/coarse: {nfcpc},

            Number of bytes/sample:          {ssb} bytes,
            Voltage block/timestep:          {vbpts},
            Voltage blocks/sec:              {vbps}, 
            Samples per voltage_blocks for each second of data per rf_input,fine_chan,r|i: {sprffcvb},
            Size per voltage block:          {vbsb} bytes,
            Delay block size:                {dbsb} bytes,
            Data file header size:           {dfhsb} bytes,
            Expected voltage data file size: {evdfsb} bytes,
            
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
            timestep_duration_ms = self.timestep_duration_ms,
            n_ants = self.metafits_context.num_ants,
            obw = self.bandwidth_hz as f64 / 1e6,
            n_coarse = self.num_coarse_chans,
            coarse = self.coarse_chans,
            fcw = self.fine_chan_width_hz as f64 / 1e3,
            nfcpc = self.num_fine_chans_per_coarse,
            ssb = self.sample_size_bytes,
            vbpts = self.num_voltage_blocks_per_timestep,
            vbps = self.num_voltage_blocks_per_second,
            sprffcvb = self.num_samples_per_voltage_block,
            vbsb = self.voltage_block_size_bytes,
            dbsb = self.delay_block_size_bytes,
            dfhsb = self.data_file_header_size_bytes,
            evdfsb = self.expected_voltage_data_file_size_bytes,
            batches = self.voltage_batches,
        )
    }
}
