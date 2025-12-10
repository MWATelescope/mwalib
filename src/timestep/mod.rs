// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Structs and helper methods for timestep metadata
use crate::gpubox_files::GpuboxTimeMap;
use crate::misc;
use crate::voltage_files::VoltageFileTimeMap;
use crate::{MWAVersion, MetafitsContext};
use crate::{MWA_VCS_LEGACY_RECOMBINED_FILE_SECONDS, MWA_VCS_MWAXV2_SUBFILE_SECONDS};
use std::fmt;

pub mod ffi;

#[cfg(test)]
mod test;

#[cfg(any(feature = "python", feature = "python-stubgen"))]
use pyo3::prelude::*;
#[cfg(feature = "python-stubgen")]
use pyo3_stub_gen_derive::gen_stub_pyclass;

/// This is a struct for our timesteps
/// NOTE: correlator timesteps use unix time, voltage timesteps use gpstime, but we convert the two depending on what we are given
#[cfg_attr(feature = "python-stubgen", gen_stub_pyclass)]
#[cfg_attr(
    any(feature = "python", feature = "python-stubgen"),
    pyclass(get_all, set_all)
)]
#[derive(Clone)]
pub struct TimeStep {
    /// UNIX time (in milliseconds to avoid floating point inaccuracy)
    pub unix_time_ms: u64,
    /// gps time (in milliseconds)
    pub gps_time_ms: u64,
}

impl TimeStep {
    /// Creates a new, populated TimeStep struct
    ///
    /// # Arguments
    ///
    /// * `unix_time_ms` - The UNIX time for this timestep, in milliseconds
    ///
    /// * `gps_time_ms` - The gps time for this timestep, in milliseconds
    ///
    ///
    /// # Returns
    ///
    /// * A populated TimeStep struct
    ///
    pub(crate) fn new(unix_time_ms: u64, gps_time_ms: u64) -> Self {
        TimeStep {
            unix_time_ms,
            gps_time_ms,
        }
    }

    /// Creates a new, populated vector of correlator TimeStep structs.
    ///    
    ///
    /// # Arguments
    ///
    /// * `gpubox_time_map` - BTree structure containing the map of what gpubox
    ///   files and timesteps we were supplied by the client.
    ///
    /// * `metafits_timesteps' - Reference to populated metafits timesteps.
    ///
    /// * `scheduled_starttime_gps_ms` - Scheduled start time of the observation based on GPSTIME in the metafits (obsid).
    ///
    /// * `scheduled_starttime_unix_ms` - Scheduled start time of the observation based on GOODTIME-QUACKTIM in the metafits.
    ///
    /// * `corr_int_time_ms` The correlator integration time in ms between each timestep.
    ///
    /// # Returns
    ///
    /// * A populated vector of TimeStep structs inside an Option. Only
    ///   timesteps. If the Option has a value of None, then `gpubox_time_map` is empty.
    ///
    pub(crate) fn populate_correlator_timesteps(
        gpubox_time_map: &GpuboxTimeMap,
        metafits_timesteps: &[TimeStep],
        scheduled_starttime_gps_ms: u64,
        scheduled_starttime_unix_ms: u64,
        corr_int_time_ms: u64,
    ) -> Option<Vec<Self>> {
        if gpubox_time_map.is_empty() {
            return None;
        }
        // Create timestep vector from metafits timesteps
        let mut timesteps: Vec<TimeStep> = Vec::new();

        // Iterate through the gpubox map and insert all timesteps
        for (unix_time_ms, _) in gpubox_time_map.iter() {
            let gps_time_ms = misc::convert_unixtime_to_gpstime(
                *unix_time_ms,
                scheduled_starttime_gps_ms,
                scheduled_starttime_unix_ms,
            );
            timesteps.push(Self::new(*unix_time_ms, gps_time_ms));
        }

        // Now that we have finished with the correlator specific GpuBoxTimeMap, pass the details into the generic function to populate the superset
        // of timesteps
        Some(TimeStep::populate_metafits_provided_superset_of_timesteps(
            timesteps,
            metafits_timesteps,
            scheduled_starttime_gps_ms,
            scheduled_starttime_unix_ms,
            corr_int_time_ms,
        ))
    }

    /// Creates a new, populated vector of voltage TimeStep structs.    
    ///
    /// # Arguments
    ///
    /// * `voltage_time_map` - BTree structure containing the map of what voltage
    ///   files and timesteps we were supplied by the client.
    ///
    /// * `metafits_timesteps' - Reference to populated metafits timesteps.
    ///
    /// * `scheduled_starttime_gps_ms` - Scheduled start time of the observation based on GPSTIME in the metafits (obsid).
    ///
    /// * `scheduled_starttime_unix_ms` - Scheduled start time of the observation based on GOODTIME-QUACKTIM in the metafits.
    ///
    /// * `voltage_timestep_duration_ms` The time in ms between each timestep.
    ///
    /// # Returns
    ///
    /// * A populated vector of TimeStep structs inside an Option. Only
    ///   timesteps. If the Option has a value of None, then `voltage_time_map` is empty.
    ///
    pub(crate) fn populate_voltage_timesteps(
        voltage_time_map: &VoltageFileTimeMap,
        metafits_timesteps: &[TimeStep],
        scheduled_starttime_gps_ms: u64,
        scheduled_starttime_unix_ms: u64,
        voltage_timestep_duration_ms: u64,
    ) -> Option<Vec<Self>> {
        if voltage_time_map.is_empty() {
            return None;
        }
        // Create timestep vector from metafits timesteps
        let mut timesteps: Vec<TimeStep> = Vec::new();

        // Iterate through the voltage time map and insert all timesteps
        for (gps_time_seconds, _) in voltage_time_map.iter() {
            let unix_time_ms = misc::convert_gpstime_to_unixtime(
                *gps_time_seconds * 1000,
                scheduled_starttime_gps_ms,
                scheduled_starttime_unix_ms,
            );
            timesteps.push(Self::new(unix_time_ms, *gps_time_seconds * 1000));
        }

        // Now that we have finished with the voltage specific VoltageTimeMap, pass the details into the generic function to populate the superset
        // of timesteps
        Some(TimeStep::populate_metafits_provided_superset_of_timesteps(
            timesteps,
            metafits_timesteps,
            scheduled_starttime_gps_ms,
            scheduled_starttime_unix_ms,
            voltage_timestep_duration_ms,
        ))
    }

    /// Generic helper function for both Correlator and Voltage contexts to, given a provided set of timesteps (from all data files),
    /// create a new vector of timesteps which is a contiguous superset of metafits and provided timesteps.
    ///
    /// This code tries to populate timesteps which:
    /// * Covers all data provided
    /// * Does it's best to go from the metafits scheduled start to scheduled end
    ///   NOTE: this is involved, because in legacy obs, the metafits correlator timesteps can be offset by fractions of an integration from the data timesteps. E.g.
    ///   metafits timesteps = [0, 2, 4, 6, ..., 30]
    ///   provided timesteps = [3, 5, 7, 9, ..., 29, 31]
    ///
    ///  In this example the superset of timesteps will be:
    ///  timesteps [1, 3, 5, 7, 9, ..., 29, 31]
    ///
    /// # Arguments
    ///
    /// * `provided_timesteps` - Vector of timesteps which have been found based on the data files provided.
    ///
    /// * `metafits_timesteps' - Reference to populated metafits timesteps.
    ///
    /// * `scheduled_starttime_gps_ms` - Scheduled start time of the observation based on GPSTIME in the metafits (obsid).
    ///
    /// * `scheduled_starttime_unix_ms` - Scheduled start time of the observation based on GOODTIME-QUACKTIM in the metafits.
    ///
    /// * `timestep_duration_ms` The time in ms between each timestep.
    ///
    /// # Returns
    ///
    /// * A populated vector of superset of TimeSteps.
    ///
    fn populate_metafits_provided_superset_of_timesteps(
        provided_timesteps: Vec<TimeStep>,
        metafits_timesteps: &[TimeStep],
        scheduled_starttime_gps_ms: u64,
        scheduled_starttime_unix_ms: u64,
        timestep_duration_ms: u64,
    ) -> Vec<TimeStep> {
        let mut timesteps: Vec<TimeStep> = provided_timesteps;

        let first_data_timestep_unix_ms: u64 = timesteps[0].unix_time_ms;
        let last_data_timestep_unix_ms: u64 = timesteps[timesteps.len() - 1].unix_time_ms;

        // Go backwards from the first provided timestep to the scheduled start time of the obs and fill in any missing timesteps
        // but only if the first provided timestep is AFTER the start time of the obs.
        if first_data_timestep_unix_ms > metafits_timesteps[0].unix_time_ms {
            let mut current_timestep_unix_ms: u64 =
                first_data_timestep_unix_ms - timestep_duration_ms;

            while current_timestep_unix_ms >= metafits_timesteps[0].unix_time_ms {
                // Create a new timestep
                let gps_time_ms = misc::convert_unixtime_to_gpstime(
                    current_timestep_unix_ms,
                    scheduled_starttime_gps_ms,
                    scheduled_starttime_unix_ms,
                );
                timesteps.push(Self::new(current_timestep_unix_ms, gps_time_ms));

                // Move back by the correlator integration time
                current_timestep_unix_ms -= timestep_duration_ms;
            }
        }

        // Go forwards from the last provided timestep to the scheduled end time of the obs and fill in any missing timesteps
        // but only if the last provided timestep is BEFORE the end time of the obs.
        if last_data_timestep_unix_ms
            < metafits_timesteps[metafits_timesteps.len() - 1].unix_time_ms
        {
            let mut current_timestep_unix_ms: u64 =
                last_data_timestep_unix_ms + timestep_duration_ms;

            while current_timestep_unix_ms
                <= metafits_timesteps[metafits_timesteps.len() - 1].unix_time_ms
            {
                // Create a new timestep
                let gps_time_ms = misc::convert_unixtime_to_gpstime(
                    current_timestep_unix_ms,
                    scheduled_starttime_gps_ms,
                    scheduled_starttime_unix_ms,
                );
                timesteps.push(Self::new(current_timestep_unix_ms, gps_time_ms));

                // Move forward by the correlator integration time
                current_timestep_unix_ms += timestep_duration_ms;
            }
        }

        // Now sort by unix time
        timesteps.sort_by_key(|t| t.unix_time_ms);

        // We have extended out the first and last provided timesteps
        // Now we can go from first to last and fill in any gaps
        for timestep_unix_time_ms in (timesteps[0].unix_time_ms
            ..timesteps[timesteps.len() - 1].unix_time_ms)
            .step_by(timestep_duration_ms as usize)
        {
            if !&timesteps
                .iter()
                .any(|t| t.unix_time_ms == timestep_unix_time_ms)
            {
                let gps_time_ms = misc::convert_unixtime_to_gpstime(
                    timestep_unix_time_ms,
                    scheduled_starttime_gps_ms,
                    scheduled_starttime_unix_ms,
                );
                timesteps.push(Self::new(timestep_unix_time_ms, gps_time_ms));
            }
        }

        // Now sort by unix time one final time
        timesteps.sort_by_key(|t| t.unix_time_ms);

        timesteps
    }

    /// This creates a populated vector of `TimeStep` structs
    ///
    /// # Arguments    
    ///
    /// * `metafits_context` - Reference to populated MetafitsContext
    ///
    /// * `mwa_version` - enum representing the version of the correlator this observation was created with.
    ///
    /// * `start_gps_time_ms` - GPS time (in ms) of first common voltage file.
    ///
    /// * `duration_ms` - Duration (in ms).        
    ///
    /// * `scheduled_starttime_gps_ms` - Scheduled start time of the observation based on GPSTIME in the metafits (obsid).
    ///
    /// * `scheduled_starttime_unix_ms` - Scheduled start time of the observation based on GOODTIME-QUACKTIM in the metafits.
    ///
    /// # Returns
    ///
    /// * A populated vector of TimeStep structs from start to end.
    ///
    pub(crate) fn populate_timesteps(
        metafits_context: &MetafitsContext,
        mwa_version: MWAVersion,
        start_gps_time_ms: u64,
        duration_ms: u64,
        scheduled_starttime_gps_ms: u64,
        scheduled_starttime_unix_ms: u64,
    ) -> Vec<Self> {
        // Determine the interval between timesteps
        let interval_ms: u64 = match mwa_version {
            MWAVersion::CorrOldLegacy | MWAVersion::CorrLegacy | MWAVersion::CorrMWAXv2 => {
                metafits_context.corr_int_time_ms
            }
            MWAVersion::VCSLegacyRecombined => MWA_VCS_LEGACY_RECOMBINED_FILE_SECONDS * 1000,
            MWAVersion::VCSMWAXv2 => MWA_VCS_MWAXV2_SUBFILE_SECONDS * 1000,
        };

        // Init our vector
        let mut timesteps_vec: Vec<Self> = vec![];

        // Populate the vector (note use of ..= here for an INCLUSIVE for loop)
        for gps_time in
            (start_gps_time_ms..start_gps_time_ms + duration_ms).step_by(interval_ms as usize)
        {
            let unix_time_ms = misc::convert_gpstime_to_unixtime(
                gps_time,
                scheduled_starttime_gps_ms,
                scheduled_starttime_unix_ms,
            );

            timesteps_vec.push(Self::new(unix_time_ms, gps_time));
        }

        timesteps_vec
    }

    /// This creates a populated vector of indices from the passed in `all_timesteps' slice of TimeSteps between start and end time.
    ///
    /// # Arguments    
    ///
    /// * `all_timesteps` - Reference to a slice containing all the timesteps        
    ///
    /// * `start_unix_time_ms` - Start time of the first timestep you want the index put in the final array..
    ///
    /// * `end_unix_time_ms` - The time (including the timestep duration) up to which you want in the final array.
    ///
    /// # Returns
    ///
    /// * A populated vector of timestep indices based on the start and end times passed in.
    ///
    pub(crate) fn get_timstep_indicies(
        all_timesteps: &[Self],
        start_unix_time_ms: u64,
        end_unix_time_ms: u64,
    ) -> Vec<usize> {
        let mut timestep_indices: Vec<usize> = all_timesteps
            .iter()
            .filter(|f| f.unix_time_ms >= start_unix_time_ms && f.unix_time_ms < end_unix_time_ms)
            .map(|t| {
                all_timesteps
                    .iter()
                    .position(|v| v.unix_time_ms == t.unix_time_ms)
                    .unwrap()
            })
            .collect();
        timestep_indices.sort_unstable();

        timestep_indices
    }
}

/// Implements fmt::Debug for TimeStep struct
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
impl fmt::Debug for TimeStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unix={:.3}, gps={:.3}",
            self.unix_time_ms as f64 / 1000.,
            self.gps_time_ms as f64 / 1000.,
        )
    }
}
