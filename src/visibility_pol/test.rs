// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Unit tests for visibility pol metadata
*/
#[cfg(test)]
use super::*;

#[test]
fn test_visibility_pol_populate() {
    let vp = VisibilityPol::populate_visibility_pols();

    assert_eq!(vp[0].polarisation, "XX");
    assert_eq!(vp[1].polarisation, "XY");
    assert_eq!(vp[2].polarisation, "YX");
    assert_eq!(vp[3].polarisation, "YY");
}
