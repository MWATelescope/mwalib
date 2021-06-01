// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for timestep metadata
*/
use crate::misc;
use crate::{metafits_context, MWAVersion, MetafitsContext};
use crate::{MWA_VCS_LEGACY_RECOMBINED_FILE_SECONDS, MWA_VCS_MWAXV2_SUBFILE_SECONDS};
use std::collections::BTreeMap;
use std::fmt;

#[cfg(test)]
mod test;

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
            unix_time_ms,
            gps_time_ms,
        }
    }

    /// Creates a new, populated vector of TimeStep structs
    ///
    /// # Arguments
    ///
    /// * `gpubox_time_map` - BTree structure containing the map of what gpubox
    ///   files and timesteps we were supplied by the client.
    ///
    /// * `metafits_timesteps' - Reference to populated
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
        metafits_timesteps: &[TimeStep],
        scheduled_starttime_gps_ms: u64,
        scheduled_starttime_unix_ms: u64,
    ) -> Option<Vec<Self>> {
        if gpubox_time_map.is_empty() {
            return None;
        }
        // Create timestep vector from metafits timesteps
        let mut timesteps: Vec<TimeStep> = metafits_timesteps.to_vec();

        // Iterate through the gpubox map and insert any missing timesteps
        for (unix_time_ms, _) in gpubox_time_map.iter() {
            if !&timesteps.iter().any(|t| t.unix_time_ms == *unix_time_ms) {
                let gps_time_ms = misc::convert_unixtime_to_gpstime(
                    *unix_time_ms,
                    scheduled_starttime_gps_ms,
                    scheduled_starttime_unix_ms,
                );
                timesteps.push(Self::new(*unix_time_ms, gps_time_ms));
            }
        }

        // Now sort by unix time
        timesteps.sort_by_key(|t| t.unix_time_ms);

        Some(timesteps)
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
        mwa_version: metafits_context::MWAVersion,
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
