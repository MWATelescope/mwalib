// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Errors associated with reading in rfinput data.

use std::path::PathBuf;

use thiserror::Error;

/// RfinputError subtypes
#[derive(Error, Debug)]
pub enum RfinputError {
    /// Error when reading in a Rfinput's polarisation.
    #[error("{fits_filename} HDU {hdu_num}: Did not recognise the polarisation at in row {row_num} ({got}); expected X or Y")]
    UnrecognisedPol {
        fits_filename: PathBuf,
        hdu_num: usize,
        row_num: usize,
        got: String,
    },

    /// An error derived from `FitsError`.
    #[error("{0}")]
    Fits(#[from] crate::fits_read::error::FitsError),
}
