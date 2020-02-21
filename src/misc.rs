// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub fn get_baseline_count(antennas: u16) -> u16 {
    return antennas * (antennas + 1) / 2;
}

pub fn get_antennas_from_baseline(baseline: usize, num_antennas: usize) -> (usize, usize) {
    let mut baseline_index = 0;
    for ant1 in 0..num_antennas {
        for ant2 in ant1..num_antennas {
            if baseline_index == baseline {
                return (ant1, ant2);
            } else {
                baseline_index += 1;
            }
        }
    }

    return (0, 0);
}
