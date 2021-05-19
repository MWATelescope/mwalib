// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Unit tests for baseline metadata
*/
#[cfg(test)]
use super::*;
use crate::MetafitsContext;
use crate::Pol;

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

    #[test]
    fn test_get_rf_inputs() {
        // Get a metafits context so we can get the baselines, antennas and rfinputs
        // Open the test mwa v 1 metafits file
        let metafits_filename = "test_files/1101503312_1_timestep/1101503312.metafits";

        //
        // Read the observation using mwalib
        //
        // Open a context and load in a test metafits
        let context =
            MetafitsContext::new(&metafits_filename).expect("Failed to create MetafitsContext");

        let (rf_a1, rf_a2) = context.baselines[1].get_rf_inputs(VisPol::XX, &context.antennas);

        assert_eq!(rf_a1.pol, Pol::X);
        assert_eq!(rf_a2.pol, Pol::X);
        assert_eq!(rf_a1.input, context.rf_inputs[0].input);
        assert_eq!(rf_a2.input, context.rf_inputs[2].input);

        let (rf_b1, rf_b2) = context.baselines[129].get_rf_inputs(VisPol::YX, &context.antennas);

        assert_eq!(rf_b1.pol, Pol::Y);
        assert_eq!(rf_b2.pol, Pol::X);
        assert_eq!(rf_b1.input, context.rf_inputs[3].input);
        assert_eq!(rf_b2.input, context.rf_inputs[4].input);
    }
}
