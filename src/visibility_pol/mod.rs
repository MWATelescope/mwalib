// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Structs and helper methods for visibility polarisations metadata
*/
use std::fmt;

#[cfg(test)]
mod test;

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
    pub(crate) fn populate_visibility_pols() -> Vec<Self> {
        vec![
            VisibilityPol {
                polarisation: String::from("XX"),
            },
            VisibilityPol {
                polarisation: String::from("XY"),
            },
            VisibilityPol {
                polarisation: String::from("YX"),
            },
            VisibilityPol {
                polarisation: String::from("YY"),
            },
        ]
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
