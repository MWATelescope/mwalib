// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for baseline metadata
*/
use crate::misc;
use std::fmt;

/// This is a struct for our baselines, so callers know the antenna ordering
#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct mwalibBaseline {
    /// Index in the mwalibContext.antenna array for antenna1 for this baseline
    pub antenna1_index: usize,
    /// Index in the mwalibContext.antenna array for antenna2 for this baseline
    pub antenna2_index: usize,
}

impl mwalibBaseline {
    /// Creates a new, populated mwalibBaseline struct
    ///
    /// # Arguments
    ///
    /// * `num_antennas` - The number of antennas in this observation
    ///
    ///
    /// # Returns
    ///
    /// * A populated mwalibBaseline struct
    ///    
    pub fn populate_baselines(num_antennas: usize) -> Vec<Self> {
        let num_baselines = misc::get_baseline_count(num_antennas);
        let mut bls: Vec<mwalibBaseline> = vec![
            mwalibBaseline {
                antenna1_index: 0,
                antenna2_index: 0
            };
            num_baselines
        ];
        let mut bl_index = 0;

        for a1 in 0..num_antennas {
            for a2 in a1..num_antennas {
                bls[bl_index].antenna1_index = a1;
                bls[bl_index].antenna2_index = a2;

                bl_index += 1;
            }
        }

        bls
    }
}

/// Implements fmt::Debug for mwalibBaseline struct
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
impl fmt::Debug for mwalibBaseline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.antenna1_index, self.antenna2_index,)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_populate_baselines() {
        let num_antennas = 128;
        let bls = mwalibBaseline::populate_baselines(num_antennas);

        assert_eq!(bls.len(), 8256);

        assert_eq!(bls[0].antenna1_index, 0);
        assert_eq!(bls[0].antenna2_index, 0);
        assert_eq!(bls[128].antenna1_index, 1);
        assert_eq!(bls[128].antenna2_index, 1);
        assert_eq!(bls[129].antenna1_index, 1);
        assert_eq!(bls[129].antenna2_index, 2);
        assert_eq!(bls[8255].antenna1_index, 127);
        assert_eq!(bls[8255].antenna2_index, 127);
    }
}
