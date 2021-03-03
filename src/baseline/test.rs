// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Unit tests for baseline metadata
*/
#[cfg(test)]
use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_populate_baselines() {
        let num_ants = 128;
        let bls = Baseline::populate_baselines(num_ants);

        assert_eq!(bls.len(), 8256);

        assert_eq!(bls[0].ant1_index, 0);
        assert_eq!(bls[0].ant2_index, 0);
        assert_eq!(bls[128].ant1_index, 1);
        assert_eq!(bls[128].ant2_index, 1);
        assert_eq!(bls[129].ant1_index, 1);
        assert_eq!(bls[129].ant2_index, 2);
        assert_eq!(bls[8255].ant1_index, 127);
        assert_eq!(bls[8255].ant2_index, 127);
    }
}
