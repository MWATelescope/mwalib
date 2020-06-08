// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for timestep metadata
*/

use std::collections::BTreeMap;
use std::fmt;

/// This is a struct for our timesteps
#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct mwalibTimeStep {
    /// UNIX time (in milliseconds to avoid floating point inaccuracy)
    pub unix_time_ms: u64,
}

impl mwalibTimeStep {
    /// Creates a new, populated mwalibTimeStep struct
    ///
    /// # Arguments
    ///
    /// * `unix_time_ms` - The UNIX time for this timestep, in milliseconds
    ///
    ///
    /// # Returns
    ///
    /// * A populated mwalibTimeStep struct
    ///
    pub fn new(unix_time_ms: u64) -> Self {
        mwalibTimeStep { unix_time_ms }
    }

    /// Creates a new, populated vector of mwalibTimeStep structs
    ///
    /// # Arguments
    ///
    /// * `gpubox_time_map` - BTree structure containing the map of what gpubox
    ///   files and timesteps we were supplied by the client.
    ///
    ///
    /// # Returns
    ///
    /// * A populated vector of mwalibTimeStep structs inside an Option. Only
    ///   timesteps *common to all* gpubox files are included. If the Option has
    ///   a value of None, then `gpubox_time_map` is empty.
    ///
    pub fn populate_timesteps(
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
        let mut timesteps: Vec<mwalibTimeStep> = vec![];
        for (key, m) in gpubox_time_map.iter() {
            if m.len() == num_gpubox_files {
                timesteps.push(Self::new(*key));
            }
        }

        Some(timesteps)
    }
}

/// Implements fmt::Debug for mwalibTimeStep struct
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
#[cfg_attr(tarpaulin, skip)]
impl fmt::Debug for mwalibTimeStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unix={:.3}", self.unix_time_ms as f64 / 1000.,)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_populate_timesteps() {
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
        let timesteps = mwalibTimeStep::populate_timesteps(&gpubox_time_map).unwrap();

        // Check
        assert_eq!(6, timesteps.len());
        assert_eq!(timesteps[0].unix_time_ms, 1_381_844_923_000);
        assert_eq!(timesteps[5].unix_time_ms, 1_381_844_925_500);
    }

    #[test]
    fn test_populate_timesteps_none() {
        // Create a dummy BTree GPUbox map
        let gpubox_time_map = BTreeMap::new();
        // Get a vector timesteps
        let timesteps = mwalibTimeStep::populate_timesteps(&gpubox_time_map);

        // Check
        assert!(timesteps.is_none());
    }

    #[test]
    fn test_timestep_new() {
        // This test is a bit of a waste right now but it will be useful once
        // julian date and possibly UTC conversions are done in the new() method
        let timestep = mwalibTimeStep {
            unix_time_ms: 1_234_567_890_123,
        };
        let new_timestep = mwalibTimeStep::new(1_234_567_890_123);

        assert_eq!(timestep.unix_time_ms, new_timestep.unix_time_ms);
    }
}
