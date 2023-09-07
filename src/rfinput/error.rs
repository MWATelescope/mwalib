// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Errors associated with reading in rfinput data.

use std::path::PathBuf;

use thiserror::Error;

/// EfinputError subtypes
#[derive(Error, Debug)]
pub enum RfinputError {
    /// Error when reading from an MWA metafits table cell.
    #[error("{fits_filename} HDU {hdu_num}: Failed to read table row {row_num} for {col_name} from metafits")]
    ReadCell {
        fits_filename: PathBuf,
        hdu_num: usize,
        row_num: usize,
        col_name: String,
    },

    /// Error when reading in a Rfinput's polarisation.
    #[error("{fits_filename} HDU {hdu_num}: Did not recognise the polarisation at in row {row_num} ({got}); expected X or Y")]
    UnrecognisedPol {
        fits_filename: PathBuf,
        hdu_num: usize,
        row_num: usize,
        got: String,
    },

    /// Error when attempting to read a cell array.
    #[error("{fits_filename} HDU {hdu_num}: Failed to read cell array from column {col_name}, row {row_num} from metafits")]
    CellArray {
        fits_filename: PathBuf,
        hdu_num: usize,
        row_num: i64,
        col_name: String,
    },

    /// An error derived from `FitsError`.
    #[error("{0}")]
    Fits(#[from] crate::fits_read::error::FitsError),
}
