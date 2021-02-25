// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for timestep metadata
*/
use crate::misc;
use std::collections::BTreeMap;
use std::fmt;

/// This is a struct for our timesteps
/// NOTE: correlator timesteps use unix time, voltage timesteps use gpstime, but we convert the two depending on what we are given
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
    fn new(unix_time_ms: u64, gps_time_ms: u64) -> Self {
        TimeStep {
            unix_time_ms: unix_time_ms,
            gps_time_ms: gps_time_ms,
        }
    }

    /// Creates a new, populated vector of TimeStep structs
    ///
    /// # Arguments
    ///
    /// * `gpubox_time_map` - BTree structure containing the map of what gpubox
    ///   files and timesteps we were supplied by the client.
    ///
    /// * `scheduled_starttime_gps_ms` - Scheduled start time of the observation based on GPSTIME in the metafits (obsid).
    ///
    /// * `scheduled_starttime_unix_ms` - Scheduled start time of the observation based on GOODTIME-QUACKTIM in the metafits.
    ///
    /// # Returns
    ///
    /// * A populated vector of TimeStep structs inside an Option. Only
    ///   timesteps *common to all* gpubox files are included. If the Option has
    ///   a value of None, then `gpubox_time_map` is empty.
    ///
    pub(crate) fn populate_correlator_timesteps(
        gpubox_time_map: &BTreeMap<u64, BTreeMap<usize, (usize, usize)>>,
        scheduled_starttime_gps_ms: u64,
        scheduled_starttime_unix_ms: u64,
    ) -> Option<Vec<Self>> {
        if gpubox_time_map.is_empty() {
            return None;
        }
        // We need to determine the timesteps that are common to all gpubox
        // files. First, determine the maximum number of gpubox files by
        // inspecting the length of the BTreeMaps associated with each key of
        // `gpubox_time_map`.
        let num_gpubox_files: usize = gpubox_time_map.iter().map(|(_, m)| m.len()).max().unwrap();
        // Now we find all keys with lengths equal to `num_gpubox_files`.
        let mut timesteps: Vec<TimeStep> = vec![];
        for (unix_time_ms, m) in gpubox_time_map.iter() {
            if m.len() == num_gpubox_files {
                let gps_time_ms = misc::convert_unixtime_to_gpstime(
                    *unix_time_ms,
                    scheduled_starttime_gps_ms,
                    scheduled_starttime_unix_ms,
                );
                timesteps.push(Self::new(*unix_time_ms, gps_time_ms));
            }
        }

        Some(timesteps)
    }

    /// Creates a new, populated vector of TimeStep structs
    ///
    /// # Arguments
    ///
    /// * `start_gps_time_ms` - GPS time (in ms) of first common voltage file.
    ///
    /// * `end_gps_time_ms` - GPS time (in ms) of last common voltage file + voltage_file_interval_ms.
    ///
    /// * `voltage_file_interval_ms` - Time interval (in ms) each voltage file represents.
    ///
    /// * `scheduled_starttime_gps_ms` - Scheduled start time of the observation based on GPSTIME in the metafits (obsid).
    ///
    /// * `scheduled_starttime_unix_ms` - Scheduled start time of the observation based on GOODTIME-QUACKTIM in the metafits.
    ///
    /// # Returns
    ///
    /// * A populated vector of TimeStep structs from start to end, spaced by voltage_file_interval_ms.
    ///
    pub(crate) fn populate_voltage_timesteps(
        start_gps_time_ms: u64,
        end_gps_time_ms: u64,
        voltage_file_interval_ms: u64,
        scheduled_starttime_gps_ms: u64,
        scheduled_starttime_unix_ms: u64,
    ) -> Vec<Self> {
        let mut timesteps: Vec<TimeStep> = vec![];
        for gps_time in
            (start_gps_time_ms..end_gps_time_ms).step_by(voltage_file_interval_ms as usize)
        {
            let unix_time_ms = misc::convert_gpstime_to_unixtime(
                gps_time,
                scheduled_starttime_gps_ms,
                scheduled_starttime_unix_ms,
            );

            timesteps.push(Self::new(unix_time_ms, gps_time));
        }

        timesteps
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_populate_correlator_timesteps() {
        // Create a dummy BTree GPUbox map
        let mut gpubox_time_map = BTreeMap::new();

        let times: Vec<u64> = vec![
            1_381_844_923_000,
            1_381_844_923_500,
            1_381_844_924_000,
            1_381_844_924_500,
            1_381_844_925_000,
            1_381_844_925_500,
        ];

        for (i, time) in times.iter().enumerate() {
            let mut new_time_tree = BTreeMap::new();
            // gpubox 0.
            new_time_tree.insert(0, (0, i));
            // gpubox 1.
            new_time_tree.insert(1, (0, i + 1));
            gpubox_time_map.insert(*time, new_time_tree);
        }

        // Get a vector timesteps
        let scheduled_start_gpstime_ms = 1065880139_000;
        let scheduled_start_unix_ms = 1381844923_000;
        let timesteps = TimeStep::populate_correlator_timesteps(
            &gpubox_time_map,
            scheduled_start_gpstime_ms,
            scheduled_start_unix_ms,
        )
        .unwrap();

        // Check
        assert_eq!(6, timesteps.len());
        assert_eq!(timesteps[0].unix_time_ms, 1_381_844_923_000);
        assert_eq!(timesteps[0].gps_time_ms, 1_065_880_139_000);
        assert_eq!(timesteps[5].unix_time_ms, 1_381_844_925_500);
        assert_eq!(timesteps[5].gps_time_ms, 1_065_880_141_500);
    }

    #[test]
    fn test_populate_correlator_timesteps_none() {
        // Create a dummy BTree GPUbox map
        let gpubox_time_map = BTreeMap::new();
        // Get a vector timesteps
        let scheduled_start_gpstime_ms = 0;
        let scheduled_start_unix_ms = 0;
        let timesteps = TimeStep::populate_correlator_timesteps(
            &gpubox_time_map,
            scheduled_start_gpstime_ms,
            scheduled_start_unix_ms,
        );

        // Check
        assert!(timesteps.is_none());
    }

    #[test]
    fn test_timestep_new() {
        // This test is a bit of a waste right now but it will be useful once
        // julian date and possibly UTC conversions are done in the new() method
        let timestep = TimeStep {
            unix_time_ms: 1_381_844_923_000,
            gps_time_ms: 1_065_880_139_000,
        };
        let new_timestep = TimeStep::new(1_381_844_923_000, 1_065_880_139_000);

        assert_eq!(timestep.unix_time_ms, new_timestep.unix_time_ms,);
        assert_eq!(timestep.gps_time_ms, new_timestep.gps_time_ms);
    }

    #[test]
    fn test_populate_voltage_timesteps_oldlegacy() {
        let scheduled_start_gpstime_ms = 1_065_880_139_000;
        let scheduled_start_unix_ms = 1_381_844_923_000;
        let timesteps = TimeStep::populate_voltage_timesteps(
            1_065_880_139_000,
            1_065_880_143_000,
            1000,
            scheduled_start_gpstime_ms,
            scheduled_start_unix_ms,
        );
        assert_eq!(timesteps.len(), 4);
        assert_eq!(timesteps[0].gps_time_ms, 1_065_880_139_000);
        assert_eq!(timesteps[0].unix_time_ms, 1_381_844_923_000);
        assert_eq!(timesteps[1].gps_time_ms, 1_065_880_140_000);
        assert_eq!(timesteps[1].unix_time_ms, 1_381_844_924_000);
        assert_eq!(timesteps[2].gps_time_ms, 1_065_880_141_000);
        assert_eq!(timesteps[2].unix_time_ms, 1_381_844_925_000);
        assert_eq!(timesteps[3].gps_time_ms, 1_065_880_142_000);
        assert_eq!(timesteps[3].unix_time_ms, 1_381_844_926_000);
    }

    #[test]
    fn test_populate_voltage_timesteps_legacy() {
        let scheduled_start_gpstime_ms = 1_065_880_139_000;
        let scheduled_start_unix_ms = 1_381_844_923_000;

        let timesteps = TimeStep::populate_voltage_timesteps(
            1_065_880_139_000,
            1_065_880_143_000,
            1000,
            scheduled_start_gpstime_ms,
            scheduled_start_unix_ms,
        );
        assert_eq!(timesteps.len(), 4);
        assert_eq!(timesteps[0].gps_time_ms, 1_065_880_139_000);
        assert_eq!(timesteps[0].unix_time_ms, 1_381_844_923_000);
        assert_eq!(timesteps[1].gps_time_ms, 1_065_880_140_000);
        assert_eq!(timesteps[1].unix_time_ms, 1_381_844_924_000);
        assert_eq!(timesteps[2].gps_time_ms, 1_065_880_141_000);
        assert_eq!(timesteps[2].unix_time_ms, 1_381_844_925_000);
        assert_eq!(timesteps[3].gps_time_ms, 1_065_880_142_000);
        assert_eq!(timesteps[3].unix_time_ms, 1_381_844_926_000);
    }

    #[test]
    fn test_populate_voltage_timesteps_mwax() {
        let scheduled_start_gpstime_ms = 1_065_880_139_000;
        let scheduled_start_unix_ms = 1_381_844_923_000;
        let timesteps = TimeStep::populate_voltage_timesteps(
            1_065_880_139_000,
            1_065_880_171_000,
            8000,
            scheduled_start_gpstime_ms,
            scheduled_start_unix_ms,
        );

        assert_eq!(timesteps.len(), 4);
        assert_eq!(timesteps[0].gps_time_ms, 1_065_880_139_000);
        assert_eq!(timesteps[0].unix_time_ms, 1_381_844_923_000);
        assert_eq!(timesteps[1].gps_time_ms, 1_065_880_147_000);
        assert_eq!(timesteps[1].unix_time_ms, 1_381_844_931_000);
        assert_eq!(timesteps[2].gps_time_ms, 1_065_880_155_000);
        assert_eq!(timesteps[2].unix_time_ms, 1_381_844_939_000);
        assert_eq!(timesteps[3].gps_time_ms, 1_065_880_163_000);
        assert_eq!(timesteps[3].unix_time_ms, 1_381_844_947_000);
    }
}
