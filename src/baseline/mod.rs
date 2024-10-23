// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Structs and helper methods for baseline metadata

use crate::misc;
use std::fmt;

#[cfg(feature = "python")]
use pyo3_stub_gen_derive::gen_stub_pyclass;

#[cfg(test)]
mod test;
/// This is a struct for our baselines, so callers know the antenna ordering
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
#[derive(Clone)]
pub struct Baseline {
    /// Index in the mwalibContext.antenna array for antenna1 for this baseline    
    pub ant1_index: usize,
    /// Index in the mwalibContext.antenna array for antenna2 for this baseline    
    pub ant2_index: usize,
}

impl Baseline {
    /// Creates a new, populated
    ///Baseline struct
    ///
    /// # Arguments
    ///
    /// * `num_ants` - The number of antennas in this observation
    ///
    ///
    /// # Returns
    ///
    /// * A populated Baseline struct
    ///    
    pub(crate) fn populate_baselines(num_ants: usize) -> Vec<Self> {
        let num_baselines = misc::get_baseline_count(num_ants);
        let mut bls: Vec<Baseline> = vec![
            Baseline {
                ant1_index: 0,
                ant2_index: 0
            };
            num_baselines
        ];
        let mut bl_index = 0;

        for a1 in 0..num_ants {
            for a2 in a1..num_ants {
                bls[bl_index].ant1_index = a1;
                bls[bl_index].ant2_index = a2;

                bl_index += 1;
            }
        }

        bls
    }
}
/// Implements fmt::Debug for Baseline struct
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
impl fmt::Debug for Baseline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.ant1_index, self.ant2_index,)
    }
}
