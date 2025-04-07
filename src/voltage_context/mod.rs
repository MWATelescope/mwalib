// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The main interface to MWA voltage data.

use crate::coarse_channel::*;
use crate::error::*;
use crate::metafits_context::*;
use crate::timestep::*;
use crate::voltage_files::*;
use crate::*;
use std::fmt;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3_stub_gen_derive::gen_stub_pyclass;

#[cfg(feature = "python")]
mod python;

#[cfg(test)]
pub(crate) mod test; // It's pub crate because I reuse some test code in the ffi tests.

///
/// This represents the basic metadata and methods for an MWA voltage capture system (VCS) observation.
///
#[cfg_attr(feature = "python", gen_stub_pyclass, pyclass(get_all, set_all))]
#[derive(Debug)]
pub struct VoltageContext {
    /// Observation Metadata obtained from the metafits file
    pub metafits_context: MetafitsContext,
    /// MWA version, derived from the files passed in
    pub mwa_version: MWAVersion,
    /// This is an array of all known timesteps (union of metafits and provided timesteps from data files). The only exception is when the metafits timesteps are
    /// offset from the provided timesteps, in which case see description in `timestep::populate_metafits_provided_superset_of_timesteps`.
    pub timesteps: Vec<TimeStep>,
    /// Number of timesteps in the timesteps vector
    pub num_timesteps: usize,
    /// length in millseconds of each timestep
    pub timestep_duration_ms: u64,

    /// Vector of coarse channel structs which is the same as the metafits provided coarse channels
    pub coarse_chans: Vec<CoarseChannel>,
    /// Number of coarse channels in coarse chans struct
    pub num_coarse_chans: usize,

    /// Vector of (in)common timestep indices
    pub common_timestep_indices: Vec<usize>,
    /// Number of common timesteps
    pub num_common_timesteps: usize,
    /// Vector of (in)common coarse channel indices
    pub common_coarse_chan_indices: Vec<usize>,
    /// Number of common coarse channels
    pub num_common_coarse_chans: usize,
    /// The start of the observation (the time that is common to all
    /// provided data files).
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
    /// Number of common timesteps only including timesteps after the quack time
    pub num_common_good_timesteps: usize,
    /// Vector of (in)common coarse channel indices only including timesteps after the quack time
    pub common_good_coarse_chan_indices: Vec<usize>,
    /// Number of common coarse channels only including timesteps after the quack time
    pub num_common_good_coarse_chans: usize,
    /// The start of the observation (the time that is common to all
    /// provided data files) only including timesteps after the quack time
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

    /// The indices of any timesteps which we have *some* data for
    pub provided_timestep_indices: Vec<usize>,
    /// Number of provided timestep indices we have at least *some* data for
    pub num_provided_timesteps: usize,
    /// The indices of any coarse channels which we have *some* data for
    pub provided_coarse_chan_indices: Vec<usize>,
    /// Number of provided coarse channel indices we have at least *some* data for
    pub num_provided_coarse_chans: usize,

    /// Bandwidth of each coarse channel
    pub coarse_chan_width_hz: u32,
    /// Volatge fine_chan_resolution (if applicable- MWA legacy is 10 kHz, MWAX is unchannelised i.e. the full coarse channel width)
    pub fine_chan_width_hz: u32,
    /// Number of fine channels in each coarse channel
    pub num_fine_chans_per_coarse: usize,

    /// Number of bytes in each sample (a sample is a complex, thus includes r and i)
    pub sample_size_bytes: u64,
    /// Number of voltage blocks per timestep
    pub num_voltage_blocks_per_timestep: usize,
    /// Number of voltage blocks of samples in each second of data    
    pub num_voltage_blocks_per_second: usize,
    /// Number of samples in each voltage_blocks for each second of data per rf_input * fine_chans * real|imag
    pub num_samples_per_voltage_block: usize,
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
    pub voltage_batches: Vec<VoltageFileBatch>,

    /// We assume as little as possible about the data layout in the voltage
    /// files; here, a `BTreeMap` contains each unique GPS time from every
    /// voltage file, which is associated with another `BTreeMap`, associating each
    /// voltage number with a voltage batch number and HDU index. The voltage
    /// number, batch number and HDU index are everything needed to find the
    /// correct HDU out of all voltage files.
    #[allow(dead_code)]
    pub(crate) voltage_time_map: VoltageFileTimeMap,
}

impl VoltageContext {
    /// From a path to a metafits file and paths to voltage files, create an `VoltageContext`.
    ///
    /// The traits on the input parameters allow flexibility to input types.
    ///
    /// # Arguments
    ///
    /// * `metafits_filename` - filename of metafits file as a path or string.
    ///
    /// * `voltage_filenames` - slice of filenames of voltage files as paths or strings.
    ///
    ///
    /// # Returns
    ///
    /// * Result containing a populated VoltageContext object if Ok.
    ///
    ///
    pub fn new<P: AsRef<Path>, P2: AsRef<Path>>(
        metafits_filename: P,
        voltage_filenames: &[P2],
    ) -> Result<Self, MwalibError> {
        // Check CFITSIO is reentrant before proceeding
        if !fits_read::is_fitsio_reentrant() {
            return Err(MwalibError::Fits(FitsError::CfitsioIsNotReentrant));
        }

        Self::new_inner(metafits_filename.as_ref(), voltage_filenames)
    }

    fn new_inner<P: AsRef<Path>>(
        metafits_filename: &Path,
        voltage_filenames: &[P],
    ) -> Result<Self, MwalibError> {
        let mut metafits_context = MetafitsContext::new_internal(metafits_filename)?;

        // Do voltage stuff only if we have voltage files.
        if voltage_filenames.is_empty() {
            return Err(MwalibError::Voltage(VoltageFileError::NoVoltageFiles));
        }
        let voltage_info = examine_voltage_files(&metafits_context, voltage_filenames)?;

        // Update the metafits now that we know the mwa_version
        metafits_context.mwa_version = Some(voltage_info.mwa_version);

        // Update the voltage fine channel size now that we know which mwaversion we are using
        if voltage_info.mwa_version == MWAVersion::VCSMWAXv2 {
            // MWAX VCS- the data is unchannelised so coarse chan width == fine chan width
            metafits_context.volt_fine_chan_width_hz = metafits_context.coarse_chan_width_hz;
            metafits_context.num_volt_fine_chans_per_coarse = 1;
        }

        // Populate metafits coarse channels and timesteps now that we know what MWA Version we are dealing with
        // Populate the coarse channels
        metafits_context.populate_expected_coarse_channels(voltage_info.mwa_version)?;

        // Now populate the fine channels
        metafits_context.metafits_fine_chan_freqs_hz =
            CoarseChannel::get_fine_chan_centres_array_hz(
                voltage_info.mwa_version,
                &metafits_context.metafits_coarse_chans,
                metafits_context.volt_fine_chan_width_hz,
                metafits_context.num_volt_fine_chans_per_coarse,
            );
        metafits_context.num_metafits_fine_chan_freqs =
            metafits_context.metafits_fine_chan_freqs_hz.len();

        // Populate the timesteps
        metafits_context.populate_expected_timesteps(voltage_info.mwa_version)?;

        // We can unwrap here because the `voltage_time_map` can't be empty if
        // `voltages` isn't empty.
        let timesteps = TimeStep::populate_voltage_timesteps(
            &voltage_info.time_map,
            &metafits_context.metafits_timesteps,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
            voltage_info.voltage_file_interval_ms,
        )
        .unwrap();
        let num_timesteps = timesteps.len();

        // Determine the "provided" timesteps- which corr_timestep indices did we get *some* data for?
        let provided_timestep_indices: Vec<usize> =
            voltage_files::populate_provided_timesteps(&voltage_info.time_map, &timesteps);
        let num_provided_timestep_indices = provided_timestep_indices.len();

        // Populate voltage coarse channels based on the metafits channels
        let coarse_chans = metafits_context.metafits_coarse_chans.clone();
        let num_coarse_chans = coarse_chans.len();

        // Determine the "provided" coarse channels- which corr_coarse_chan indices did we get *some* data for?
        let provided_coarse_chan_indices: Vec<usize> =
            voltage_files::populate_provided_coarse_channels(&voltage_info.time_map, &coarse_chans);
        let num_provided_coarse_chan_indices = provided_coarse_chan_indices.len();

        // Fine-channel resolution & number of fine chans per coarse
        let fine_chan_width_hz = metafits_context.volt_fine_chan_width_hz;
        let num_fine_chans_per_coarse = metafits_context.num_volt_fine_chans_per_coarse;

        let coarse_chan_width_hz = metafits_context.coarse_chan_width_hz;

        // Populate the start and end times of the observation based on common channels.
        // Start= start of first timestep
        // End  = start of last timestep + integration time
        let (
            common_start_gps_time_ms,
            common_end_gps_time_ms,
            common_duration_ms,
            common_coarse_chan_identifiers,
        ) = {
            match determine_common_obs_times_and_chans(
                &voltage_info.time_map,
                voltage_info.voltage_file_interval_ms,
                None,
            )? {
                Some(o) => (
                    o.start_gps_time_ms,
                    o.end_gps_time_ms,
                    o.duration_ms,
                    o.coarse_chan_identifiers,
                ),
                None => (0, 0, 0, vec![]),
            }
        };

        // Convert the real start and end times to GPS time
        let common_start_unix_time_ms = misc::convert_gpstime_to_unixtime(
            common_start_gps_time_ms,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
        );
        let common_end_unix_time_ms = misc::convert_gpstime_to_unixtime(
            common_end_gps_time_ms,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
        );

        // Populate the common coarse_chan indices vector
        let common_coarse_chan_indices: Vec<usize> =
            CoarseChannel::get_coarse_chan_indicies(&coarse_chans, &common_coarse_chan_identifiers);
        let num_common_coarse_chans = common_coarse_chan_indices.len();

        // Populate a vector containing the indicies of all the common timesteps (based on the correlator context timesteps vector)
        let common_timestep_indices: Vec<usize> = TimeStep::get_timstep_indicies(
            &timesteps,
            common_start_unix_time_ms,
            common_end_unix_time_ms,
        );
        let num_common_timesteps = common_timestep_indices.len();

        let common_bandwidth_hz = (num_common_coarse_chans as u32) * coarse_chan_width_hz;

        // Populate the start and end times of the observation based on common channels (excluding any timesteps during/before the quacktime).
        // Start= start of first timestep
        // End  = start of last timestep + integration time
        let (
            common_good_start_gps_time_ms,
            common_good_end_gps_time_ms,
            common_good_duration_ms,
            common_good_coarse_chan_identifiers,
        ) = {
            match determine_common_obs_times_and_chans(
                &voltage_info.time_map,
                voltage_info.voltage_file_interval_ms,
                Some(metafits_context.good_time_gps_ms),
            )? {
                Some(o) => (
                    o.start_gps_time_ms,
                    o.end_gps_time_ms,
                    o.duration_ms,
                    o.coarse_chan_identifiers,
                ),
                None => (0, 0, 0, vec![]),
            }
        };

        // Convert the real start and end times to GPS time
        let common_good_start_unix_time_ms = misc::convert_gpstime_to_unixtime(
            common_good_start_gps_time_ms,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
        );
        let common_good_end_unix_time_ms = misc::convert_gpstime_to_unixtime(
            common_good_end_gps_time_ms,
            metafits_context.sched_start_gps_time_ms,
            metafits_context.sched_start_unix_time_ms,
        );

        // Populate the common coarse_chan indices vector
        let common_good_coarse_chan_indices: Vec<usize> = CoarseChannel::get_coarse_chan_indicies(
            &coarse_chans,
            &common_good_coarse_chan_identifiers,
        );
        let num_common_good_coarse_chans = common_good_coarse_chan_indices.len();

        // Populate a vector containing the indicies of all the common timesteps (based on the correlator context timesteps vector)
        let common_good_timestep_indices: Vec<usize> = TimeStep::get_timstep_indicies(
            &timesteps,
            common_good_start_unix_time_ms,
            common_good_end_unix_time_ms,
        );
        let num_common_good_timesteps = common_good_timestep_indices.len();

        let common_good_bandwidth_hz = (num_common_good_coarse_chans as u32) * coarse_chan_width_hz;

        // Number of bytes in each sample
        let sample_size_bytes: u64 = match voltage_info.mwa_version {
            MWAVersion::VCSLegacyRecombined => 1, // 4 bits real, 4 bits imag
            MWAVersion::VCSMWAXv2 => 2,           // 8 bits real, 8 bits imag
            _ => {
                return Err(MwalibError::Voltage(VoltageFileError::InvalidMwaVersion {
                    mwa_version: voltage_info.mwa_version,
                }))
            }
        };

        // Number of voltage blocks per timestep
        let num_voltage_blocks_per_timestep: usize = match voltage_info.mwa_version {
            MWAVersion::VCSLegacyRecombined => 1,
            MWAVersion::VCSMWAXv2 => 160,
            _ => {
                return Err(MwalibError::Voltage(VoltageFileError::InvalidMwaVersion {
                    mwa_version: voltage_info.mwa_version,
                }))
            }
        };

        // Number of voltage blocks of samples in each second of data
        let num_voltage_blocks_per_second: usize = num_voltage_blocks_per_timestep
            / (voltage_info.voltage_file_interval_ms as usize / 1000);

        // Number of samples in each voltage_blocks for each second of data per rf_input * fine_chans * real|imag
        let num_samples_per_rf_chain_fine_chan_in_a_voltage_block: usize =
            match voltage_info.mwa_version {
                MWAVersion::VCSLegacyRecombined => 10_000,
                MWAVersion::VCSMWAXv2 => match metafits_context.oversampled {
                    true => 81_920,
                    false => 64_000,
                }, // critically sampled = 64000 per rf_inpit x real|imag (no fine chans), or 81920 if oversampled
                _ => {
                    return Err(MwalibError::Voltage(VoltageFileError::InvalidMwaVersion {
                        mwa_version: voltage_info.mwa_version,
                    }))
                }
            };

        // The size of each voltage block
        let voltage_block_size_bytes: u64 = num_samples_per_rf_chain_fine_chan_in_a_voltage_block
            as u64
            * metafits_context.num_rf_inputs as u64
            * num_fine_chans_per_coarse as u64
            * sample_size_bytes;

        // Determine delay_block_size - for MWAX this is the same as a voltage block size, for legacy it is 0
        let delay_block_size_bytes: u64 = match voltage_info.mwa_version {
            MWAVersion::VCSLegacyRecombined => 0,
            MWAVersion::VCSMWAXv2 => voltage_block_size_bytes,
            _ => {
                return Err(MwalibError::Voltage(VoltageFileError::InvalidMwaVersion {
                    mwa_version: voltage_info.mwa_version,
                }))
            }
        };

        // The amount of bytes to skip before getting into real data within the voltage files
        let data_file_header_size_bytes: u64 = match voltage_info.mwa_version {
            MWAVersion::VCSLegacyRecombined => 0,
            MWAVersion::VCSMWAXv2 => 4096,
            _ => {
                return Err(MwalibError::Voltage(VoltageFileError::InvalidMwaVersion {
                    mwa_version: voltage_info.mwa_version,
                }))
            }
        };

        // Expected voltage file size
        // Legacy 128T should be    0+       0+(327,680,000 * 1 block)    == 327,680,000 bytes (for 1 sec of data)
        // MWAX 128T should be   4096+32768000+(32,768,000  * 160 blocks) == 5,275,652,096 bytes (for 8 secs of data)
        let expected_voltage_data_file_size_bytes: u64 = data_file_header_size_bytes
            + delay_block_size_bytes
            + (voltage_block_size_bytes * num_voltage_blocks_per_timestep as u64);

        // The rf inputs should be sorted depending on the CorrVersion
        match voltage_info.mwa_version {
            MWAVersion::VCSLegacyRecombined => {
                metafits_context.rf_inputs.sort_by_key(|k| k.vcs_order);
            }
            MWAVersion::VCSMWAXv2 => {}
            _ => {
                return Err(MwalibError::Voltage(VoltageFileError::InvalidMwaVersion {
                    mwa_version: voltage_info.mwa_version,
                }))
            }
        }

        Ok(VoltageContext {
            metafits_context,
            mwa_version: voltage_info.mwa_version,
            num_timesteps,
            timesteps,
            timestep_duration_ms: voltage_info.voltage_file_interval_ms,
            num_coarse_chans,
            coarse_chans,
            num_common_timesteps,
            common_timestep_indices,
            num_common_coarse_chans,
            common_coarse_chan_indices,
            common_start_gps_time_ms,
            common_end_gps_time_ms,
            common_start_unix_time_ms,
            common_end_unix_time_ms,
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
            provided_timestep_indices,
            num_provided_timesteps: num_provided_timestep_indices,
            provided_coarse_chan_indices,
            num_provided_coarse_chans: num_provided_coarse_chan_indices,
            fine_chan_width_hz,
            coarse_chan_width_hz,
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

    /// For a given slice of voltage coarse channel indices, return a vector of the center
    /// frequencies for all the fine channels in the given coarse channels
    ///
    /// # Arguments
    ///
    /// * `volt_coarse_chan_indices` - a slice containing voltage coarse channel indices
    ///   for which you want fine channels for. Does not need to be contiguous.
    ///
    ///
    /// # Returns
    ///
    /// * a vector of f64 containing the centre sky frequencies of all the fine channels for the
    ///   given coarse channels.
    ///
    pub fn get_fine_chan_freqs_hz_array(&self, volt_coarse_chan_indices: &[usize]) -> Vec<f64> {
        CoarseChannel::get_fine_chan_centres_array_hz_inner(
            self.mwa_version,
            volt_coarse_chan_indices
                .iter()
                .map(|c| &self.coarse_chans[*c]),
            self.metafits_context.volt_fine_chan_width_hz,
            self.metafits_context.num_volt_fine_chans_per_coarse,
        )
    }

    /// Validates gps time start and gps seconds count, and returns the end gps time or an Error.
    /// The gps end second is the START time of the end second, not the END of the second.
    /// e.g gpstart = 100, count = 1, therefore gpsend = 100.
    /// e.g gpstart = 100, count = 2, therefore gpsend = 101.
    ///
    /// # Arguments
    ///
    /// * `gps_second_start` - The gps start time
    ///
    /// * `gps_second_count` - The number of gps seconds
    ///    
    ///
    /// # Returns
    ///
    /// * A Result the end gps second if the gps time range is within the VoltageContext's range or Error if not.
    ///
    fn validate_gps_time_parameters(
        &self,
        gps_second_start: u64,
        gps_second_count: usize,
    ) -> Result<u64, VoltageFileError> {
        // Validate the gpstime
        let gps_time_observation_max_ms =
            self.timesteps[self.num_timesteps - 1].gps_time_ms + self.timestep_duration_ms;

        // Validate the start time
        if gps_second_start * 1000 < self.timesteps[0].gps_time_ms
            || gps_second_start * 1000 > gps_time_observation_max_ms
        {
            return Err(VoltageFileError::InvalidGpsSecondStart(
                self.timesteps[0].gps_time_ms / 1000,
                gps_time_observation_max_ms / 1000,
            ));
        }

        // Validate the end time
        let gps_second_end: u64 = gps_second_start + gps_second_count as u64 - 1;
        if gps_second_end * 1000 > gps_time_observation_max_ms {
            return Err(VoltageFileError::InvalidGpsSecondCount(
                self.timesteps[0].gps_time_ms / 1000,
                gps_second_count,
                gps_time_observation_max_ms / 1000,
            ));
        }

        Ok(gps_second_end)
    }

    /// Read a single or multiple seconds of data for a coarse channel
    ///
    /// # Arguments
    ///
    /// * `gps_second_start` - GPS second within the observation to start returning data.
    ///
    /// * `gps_second_count` - number of seconds of data to return.
    ///
    /// * `volt_coarse_chan_index` - index within the coarse_chan array for the desired coarse channel. This corresponds
    ///   to the element within VoltageContext.coarse_chans.
    ///
    /// * `buffer` - a mutable reference to an already exitsing, initialised slice `[i8]` which will be filled with data.
    ///
    /// # Returns
    ///
    /// * A Result of Ok or error.
    ///
    ///
    pub fn read_second(
        &self,
        gps_second_start: u64,
        gps_second_count: usize,
        volt_coarse_chan_index: usize,
        buffer: &mut [i8],
    ) -> Result<(), VoltageFileError> {
        if self.voltage_batches.is_empty() {
            return Err(VoltageFileError::NoVoltageFiles);
        }

        // Validate the coarse chan
        if volt_coarse_chan_index > self.num_coarse_chans - 1 {
            return Err(VoltageFileError::InvalidCoarseChanIndex(
                self.num_coarse_chans - 1,
            ));
        }

        // Determine the channel identifier, which we will use later to find the correct data file
        let channel_identifier = self.coarse_chans[volt_coarse_chan_index].gpubox_number;

        // Validate the gpstime
        let gps_second_end =
            VoltageContext::validate_gps_time_parameters(self, gps_second_start, gps_second_count)?;

        // Determine which timestep(s) we need to cover the start and end gps times.
        // NOTE: mwax has 8 gps seconds per timestep, legacy vcs has 1
        let timestep_index_start: usize = (((gps_second_start * 1000)
            - self.timesteps[0].gps_time_ms) as f64
            / self.timestep_duration_ms as f64)
            .floor() as usize;
        // Get end timestep which includes the end gps time
        let timestep_index_end: usize =
            ((((gps_second_end * 1000) - self.timesteps[0].gps_time_ms) as f64 + 1.)
                / self.timestep_duration_ms as f64)
                .floor() as usize;

        // Check output buffer is big enough
        let expected_buffer_size: usize = self.voltage_block_size_bytes as usize
            * self.num_voltage_blocks_per_second
            * gps_second_count;

        if buffer.len() != expected_buffer_size {
            return Err(VoltageFileError::InvalidBufferSize(
                buffer.len(),
                expected_buffer_size,
            ));
        }

        // Work out how much to read at once
        let chunk_size: usize = self.voltage_block_size_bytes as usize; // This will be the size of a voltage block

        // Variables to keep track of where in the buffer we are writing to
        let mut start_pos: usize = 0;
        let mut end_pos: usize = chunk_size;

        // Loop through the timesteps / files
        for timestep_index in timestep_index_start..timestep_index_end + 1 {
            // Find the batch relating to the timestep index or None if we don't have any data files for that timestep
            let batch = &self
                .voltage_batches
                .iter()
                .find(|f| f.gps_time_seconds * 1000 == self.timesteps[timestep_index].gps_time_ms);

            // find the filename if the timestep/coarse chan combo exists
            let filename_result = match batch {
                Some(b) => b
                    .voltage_files
                    .iter()
                    .find(|f| f.channel_identifier == channel_identifier),
                None => None,
            };

            // Get the filename for this timestep and coarse channel
            let filename: &String = match filename_result {
                Some(f) => &f.filename,
                None => {
                    return Err(VoltageFileError::NoDataForTimeStepCoarseChannel {
                        timestep_index,
                        coarse_chan_index: volt_coarse_chan_index,
                    });
                }
            };

            // Open the file
            let mut file_handle = File::open(filename).expect("no file found");

            // Obtain metadata
            let metadata = std::fs::metadata(filename).expect("unable to read metadata");

            // Check file is as big as we expect
            if metadata.len() != self.expected_voltage_data_file_size_bytes {
                return Err(VoltageFileError::InvalidVoltageFileSize(
                    metadata.len(),
                    String::from(filename),
                    self.expected_voltage_data_file_size_bytes,
                ));
            }

            // Loop until all data is read into our buffer
            //
            // in mwax 1 file	= 8 gps seconds broken into 20 voltage blocks per sec
            // in legacy 1 file	= 1 gps seconds
            //
            for block_index in 0..self.num_voltage_blocks_per_timestep {
                // We may only be reading a portion of this file, so determine if we want to read this block
                // i.e. is it part of second we care about?
                let current_gps_time = (self.timesteps[timestep_index].gps_time_ms / 1000)
                    + (block_index as f64 / self.num_voltage_blocks_per_second as f64).floor()
                        as u64;

                if current_gps_time >= gps_second_start && current_gps_time <= gps_second_end {
                    // Skip bytes in the file to this block
                    // We skip the header, delays and any intervening voltage blocks
                    file_handle
                        .by_ref()
                        .seek(SeekFrom::Start(
                            self.data_file_header_size_bytes
                                + self.delay_block_size_bytes
                                + (block_index as u64 * chunk_size as u64),
                        ))
                        .expect("Unable to seek to next data block in voltage file");

                    // Our input buffer is &mut [i8] because we want signed data, yet
                    // all the read functions seem to only read as [u8].
                    // So convert the chunk to &mut [u8] then read into it.
                    // This should just be a pointer cast and not cost anything.
                    let chunk: &mut [u8] = unsafe {
                        core::slice::from_raw_parts_mut(
                            buffer[start_pos..end_pos].as_mut_ptr() as *mut u8,
                            chunk_size,
                        )
                    };

                    let bytes_read = file_handle
                        .by_ref()
                        .take(chunk_size as u64)
                        .read(chunk)
                        .expect("Unable to read data block in voltage file");

                    assert_eq!(bytes_read, chunk_size);

                    // Set new start and end pos
                    start_pos = end_pos;
                    end_pos = start_pos + chunk_size;
                }
            }
        }
        Ok(())
    }

    /// Read a single or multiple seconds of data for a coarse channel
    ///
    /// # Arguments
    ///
    /// * `volt_timestep_index` - index within the timestep array for the desired timestep. This corresponds
    ///   to the element within VoltageContext.timesteps. For mwa legacy each index
    ///   represents 1 second increments, for mwax it is 8 second increments.
    ///
    /// * `volt_coarse_chan_index` - index within the coarse_chan array for the desired coarse channel. This corresponds
    ///   to the element within VoltageContext.coarse_chans.
    ///
    /// * `buffer` - a mutable reference to an already exitsing, initialised slice `[i8]` which will be filled with the data from one VCS data file.
    ///
    ///
    /// # Returns
    ///
    /// * A Result of Ok or error.
    ///    
    pub fn read_file(
        &self,
        volt_timestep_index: usize,
        volt_coarse_chan_index: usize,
        buffer: &mut [i8],
    ) -> Result<(), VoltageFileError> {
        if self.voltage_batches.is_empty() {
            return Err(VoltageFileError::NoVoltageFiles);
        }

        // Validate the timestep
        if volt_timestep_index > self.num_timesteps - 1 {
            return Err(VoltageFileError::InvalidTimeStepIndex(
                self.num_timesteps - 1,
            ));
        }

        // Validate the coarse chan
        if volt_coarse_chan_index > self.num_coarse_chans - 1 {
            return Err(VoltageFileError::InvalidCoarseChanIndex(
                self.num_coarse_chans - 1,
            ));
        }
        // Determine the channel identifier, which we will use later to find the correct data file
        let channel_identifier = self.coarse_chans[volt_coarse_chan_index].gpubox_number;

        // Work out how much to read at once
        let chunk_size: usize = self.voltage_block_size_bytes as usize; // This will be the size of a voltage block

        // Find the batch relating to the timestep index or None if we don't have any data files for that timestep
        let batch = &self
            .voltage_batches
            .iter()
            .find(|f| f.gps_time_seconds * 1000 == self.timesteps[volt_timestep_index].gps_time_ms);

        // find the filename if the timestep/coarse chan combo exists
        let filename_result = match batch {
            Some(b) => b
                .voltage_files
                .iter()
                .find(|f| f.channel_identifier == channel_identifier),
            None => None,
        };

        // Get the filename for this timestep and coarse channel
        let filename: &String = match filename_result {
            Some(f) => &f.filename,
            None => {
                return Err(VoltageFileError::NoDataForTimeStepCoarseChannel {
                    timestep_index: volt_timestep_index,
                    coarse_chan_index: volt_coarse_chan_index,
                });
            }
        };

        // Open the file
        let mut file_handle = File::open(filename).expect("no file found");

        // Obtain metadata
        let metadata = std::fs::metadata(filename).expect("unable to read metadata");

        // Check file is as big as we expect
        // normally we would compare the file len to context.expected_voltage_data_file_size_bytes,
        // but in our tests we override the voltage_block_size_bytes because our test files only have 1 tile
        // TODO: This should be an Error type
        assert_eq!(
            metadata.len(),
            self.data_file_header_size_bytes
                + self.delay_block_size_bytes
                + (self.voltage_block_size_bytes * self.num_voltage_blocks_per_timestep as u64),
            "header={} + delay={} + vb_size={} + vb_per_ts={}",
            self.data_file_header_size_bytes,
            self.delay_block_size_bytes,
            self.voltage_block_size_bytes,
            self.num_voltage_blocks_per_timestep
        );

        // Check buffer is big enough
        let expected_buffer_size =
            self.voltage_block_size_bytes as usize * self.num_voltage_blocks_per_timestep;

        if buffer.len() != expected_buffer_size {
            return Err(VoltageFileError::InvalidBufferSize(
                buffer.len(),
                expected_buffer_size,
            ));
        }

        // Skip header
        file_handle
            .by_ref()
            .seek(SeekFrom::Start(
                self.data_file_header_size_bytes + self.delay_block_size_bytes,
            ))
            .expect("Unable to seek to data in voltage file");

        // Read the data into the final output buffer in blocks, spaced out with delay blocks (possibly)
        let mut start_pos: usize = 0;
        let mut end_pos: usize = chunk_size;

        // Loop until all data is read into our buffer
        for _ in 0..self.num_voltage_blocks_per_timestep {
            //let chunk = &mut buffer[start_pos..end_pos];
            //let bytes_read = reader.by_ref().read(chunk).expect("Error");

            // Our input buffer is &mut [i8] because we want signed data, yet
            // all the read functions seem to only read as [u8].
            // So convert the chunk to &mut [u8] then read into it.
            // This should just be a pointer cast and not cost anything.
            let chunk: &mut [u8] = unsafe {
                core::slice::from_raw_parts_mut(
                    buffer[start_pos..end_pos].as_mut_ptr() as *mut u8,
                    chunk_size,
                )
            };

            //let chunk = &buffer[start_pos..end_pos];
            //let bytes_read = reader.by_ref().read(&mut chunk).expect("Error");
            let bytes_read = file_handle
                .by_ref()
                .take(chunk_size as u64)
                .read(chunk)
                .expect("Unable to read data block in voltage file");

            assert_eq!(bytes_read, chunk_size);

            // Set new start and end pos
            start_pos = end_pos;
            end_pos = start_pos + chunk_size;
        }

        Ok(())
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
            MWA version:              {corr_ver},            

            num timesteps:            {n_timesteps},
            timesteps:                {timesteps:?},            
            timestep duration ms:     {timestep_duration_ms} ms,
            num coarse channels,      {n_coarse},
            coarse channels:          {coarse:?},

            provided timesteps indices:   {num_provided_timesteps}: {provided_timesteps:?},
            provided coarse chan indices: {num_provided_coarse_chans}: {provided_coarse_chans:?},

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

            fine channel resolution:  {fcw} Hz,
            num fine channels/coarse: {nfcpc},

            Number of bytes/sample:          {ssb} bytes,
            Voltage block/timestep:          {vbpts},
            Voltage blocks/sec:              {vbps}, 
            Samples per voltage_blocks for each second of data per rf_input,fine_chan,r|i: {sprffcvb},
            Size per voltage block:          {vbsb} bytes,
            Delay block size:                {dbsb} bytes,
            Data file header size:           {dfhsb} bytes,
            Expected voltage data file size: {evdfsb} bytes,
            
            voltage batches:          {batches:#?},
        )"#,
            metafits_context = self.metafits_context,
            corr_ver = self.mwa_version,
            n_timesteps = self.num_timesteps,
            timesteps = self.timesteps,
            timestep_duration_ms = self.timestep_duration_ms,
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
            num_provided_timesteps = self.num_provided_timesteps,
            provided_timesteps = self.provided_timestep_indices,
            num_provided_coarse_chans = self.num_provided_coarse_chans,
            provided_coarse_chans = self.provided_coarse_chan_indices,
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
