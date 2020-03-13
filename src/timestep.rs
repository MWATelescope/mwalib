// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for timestep metadata
*/
use crate::*;
use std::collections::BTreeMap;
use std::fmt;

/// This is a struct for our coarse channels
#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct mwalibTimeStep {
    // UNIX time (in milliseconds to avoid floating point inaccuracy)
    pub unix_time_ms: u64,
}

impl mwalibTimeStep {
    pub fn new(unix_time_ms: u64) -> mwalibTimeStep {
        mwalibTimeStep { unix_time_ms }
    }
    pub fn populate_timesteps(
        gpubox_time_map: &BTreeMap<u64, BTreeMap<usize, (usize, usize)>>,
    ) -> Result<(Vec<mwalibTimeStep>, usize), ErrorKind> {
        let num_timesteps = gpubox_time_map.len();
        // Initialise the timstep vector of structs
        let mut timesteps: Vec<mwalibTimeStep> = Vec::with_capacity(num_timesteps);
        // Each item of the gpubox_time_map has the unixtime(in ms) and another BTtree of GPUBOX files
        for key in gpubox_time_map.iter() {
            timesteps.push(mwalibTimeStep::new(*key.0));
        }

        Ok((timesteps, num_timesteps))
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
impl fmt::Debug for mwalibTimeStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unix={:.3}", self.unix_time_ms as f64 / 1000.,)
    }
}
