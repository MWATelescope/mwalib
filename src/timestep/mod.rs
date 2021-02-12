// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for timestep metadata
*/
use std::collections::BTreeMap;
use std::fmt;

/// This is a struct for our timesteps
/// NOTE: correlator timesteps use unix time, voltage timesteps use gpstime
/// TODO: convert from unix to gps and gps to unix. For now it's ok to leave the one you are not using as 0.
#[derive(Clone)]
pub struct TimeStep {
    /// UNIX time (in milliseconds to avoid floating point inaccuracy)
    pub unix_time_milliseconds: u64,
    /// gps time (in milliseconds)
    pub gps_time_milliseconds: u64,
}

impl TimeStep {
    /// Creates a new, populated TimeStep struct
    ///
    /// # Arguments
    ///
    /// * `unix_time_milliseconds` - The UNIX time for this timestep, in milliseconds
    ///
    /// * `gps_time_milliseconds` - The gps time for this timestep, in milliseconds
    ///
    ///
    /// # Returns
    ///
    /// * A populated TimeStep struct
    ///
    pub fn new(unix_time_milliseconds: u64, gps_time_milliseconds: u64) -> Self {
        TimeStep {
            unix_time_milliseconds,
            gps_time_milliseconds,
        }
    }

    /// Creates a new, populated vector of TimeStep structs
    ///
    /// # Arguments
    ///
    /// * `gpubox_time_map` - BTree structure containing the map of what gpubox
    ///   files and timesteps we were supplied by the client.
    ///
    ///
    /// # Returns
    ///
    /// * A populated vector of TimeStep structs inside an Option. Only
    ///   timesteps *common to all* gpubox files are included. If the Option has
    ///   a value of None, then `gpubox_time_map` is empty.
    ///
    pub fn populate_correlator_timesteps(
        gpubox_time_map: &BTreeMap<u64, BTreeMap<usize, (usize, usize)>>,
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
        for (key, m) in gpubox_time_map.iter() {
            if m.len() == num_gpubox_files {
                let gps_time_milliseconds = 0; // TODO: convert UNIX time to gps seconds
                timesteps.push(Self::new(*key, gps_time_milliseconds));
            }
        }

        Some(timesteps)
    }

    /// Creates a new, populated vector of TimeStep structs
    ///
    /// # Arguments
    ///
    /// * `start_gps_time_milliseconds` - GPS time (in ms) of first common voltage file.
    ///
    /// * `end_gps_time_milliseconds` - GPS time (in ms) of last common voltage file + voltage_file_interval_milliseconds.
    ///
    ///
    ///
    /// # Returns
    ///
    /// * A populated vector of TimeStep structs from start to end, spaced by voltage_file_interval_milliseconds.
    ///
    pub fn populate_voltage_timesteps(
        start_gps_time_milliseconds: u64,
        end_gps_time_milliseconds: u64,
        voltage_file_interval_milliseconds: u64,
    ) -> Vec<Self> {
        let mut timesteps: Vec<TimeStep> = vec![];
        for gps_time in (start_gps_time_milliseconds..end_gps_time_milliseconds)
            .step_by(voltage_file_interval_milliseconds as usize)
        {
            let unix_time_milliseconds = 0; // TODO: convert UNIX time to gps seconds

            timesteps.push(Self::new(unix_time_milliseconds, gps_time));
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
#[cfg(not(tarpaulin_include))]
impl fmt::Debug for TimeStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unix={:.3}", self.unix_time_milliseconds as f64 / 1000.,)
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
        let timesteps = TimeStep::populate_correlator_timesteps(&gpubox_time_map).unwrap();

        // Check
        assert_eq!(6, timesteps.len());
        assert_eq!(timesteps[0].unix_time_milliseconds, 1_381_844_923_000);
        assert_eq!(timesteps[5].unix_time_milliseconds, 1_381_844_925_500);
    }

    #[test]
    fn test_populate_correlator_timesteps_none() {
        // Create a dummy BTree GPUbox map
        let gpubox_time_map = BTreeMap::new();
        // Get a vector timesteps
        let timesteps = TimeStep::populate_correlator_timesteps(&gpubox_time_map);

        // Check
        assert!(timesteps.is_none());
    }

    #[test]
    fn test_timestep_new() {
        // This test is a bit of a waste right now but it will be useful once
        // julian date and possibly UTC conversions are done in the new() method
        let timestep = TimeStep {
            unix_time_milliseconds: 1_234_567_890_123,
            gps_time_milliseconds: 0,
        };
        let new_timestep = TimeStep::new(1_234_567_890_123, 0);

        assert_eq!(
            timestep.unix_time_milliseconds,
            new_timestep.unix_time_milliseconds,
        );
        assert_eq!(
            timestep.gps_time_milliseconds,
            new_timestep.gps_time_milliseconds
        );
    }

    #[test]
    fn test_populate_voltage_timesteps_oldlegacy() {
        let timesteps =
            TimeStep::populate_voltage_timesteps(1_234_567_000_000, 1_234_567_004_000, 1000);
        assert_eq!(timesteps.len(), 4);
        assert_eq!(timesteps[0].gps_time_milliseconds, 1_234_567_000_000);
        assert_eq!(timesteps[1].gps_time_milliseconds, 1_234_567_001_000);
        assert_eq!(timesteps[2].gps_time_milliseconds, 1_234_567_002_000);
        assert_eq!(timesteps[3].gps_time_milliseconds, 1_234_567_003_000);
    }

    #[test]
    fn test_populate_voltage_timesteps_legacy() {
        let timesteps =
            TimeStep::populate_voltage_timesteps(1_234_567_000_000, 1_234_567_004_000, 1000);
        assert_eq!(timesteps.len(), 4);
        assert_eq!(timesteps[0].gps_time_milliseconds, 1_234_567_000_000);
        assert_eq!(timesteps[1].gps_time_milliseconds, 1_234_567_001_000);
        assert_eq!(timesteps[2].gps_time_milliseconds, 1_234_567_002_000);
        assert_eq!(timesteps[3].gps_time_milliseconds, 1_234_567_003_000);
    }

    #[test]
    fn test_populate_voltage_timesteps_mwax() {
        let timesteps =
            TimeStep::populate_voltage_timesteps(1_234_567_000_000, 1_234_567_032_000, 8000);

        assert_eq!(timesteps.len(), 4);
        assert_eq!(timesteps[0].gps_time_milliseconds, 1_234_567_000_000);
        assert_eq!(timesteps[1].gps_time_milliseconds, 1_234_567_008_000);
        assert_eq!(timesteps[2].gps_time_milliseconds, 1_234_567_016_000);
        assert_eq!(timesteps[3].gps_time_milliseconds, 1_234_567_024_000);
    }
}
