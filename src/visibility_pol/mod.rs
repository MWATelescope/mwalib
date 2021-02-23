// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for visibility polarisations metadata
*/

use std::fmt;

/// This is a struct for our visibility polarisations
#[derive(Clone)]
pub struct VisibilityPol {
    /// Polarisation (e.g. "XX" or "XY" or "YX" or "YY")
    pub polarisation: String,
}

impl VisibilityPol {
    /// Creates a new, populated vector of VisibilityPol structs
    ///
    /// # Arguments
    ///        
    ///
    /// # Returns
    ///
    /// * A populated vector of visibility polarisations for the MWA
    ///
    pub fn populate_visibility_pols() -> Vec<Self> {
        let mut pols: Vec<VisibilityPol> = Vec::with_capacity(4);
        pols.push(VisibilityPol {
            polarisation: String::from("XX"),
        });
        pols.push(VisibilityPol {
            polarisation: String::from("XY"),
        });
        pols.push(VisibilityPol {
            polarisation: String::from("YX"),
        });
        pols.push(VisibilityPol {
            polarisation: String::from("YY"),
        });

        pols
    }
}

/// Implements fmt::Debug for VisibilityPol struct
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
impl fmt::Debug for VisibilityPol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pol={}", self.polarisation,)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_pol_populate() {
        let vp = VisibilityPol::populate_visibility_pols();

        assert_eq!(vp[0].polarisation, "XX");
        assert_eq!(vp[1].polarisation, "XY");
        assert_eq!(vp[2].polarisation, "YX");
        assert_eq!(vp[3].polarisation, "YY");
    }
}
