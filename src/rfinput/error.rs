// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Errors associated with reading in rfinput data.
*/

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RfinputError {
    /// Error when opening a fits file.
    #[error("{fits_filename} HDU {hdu_num}: Failed to read table row {row_num} for {col_name} from metafits")]
    Read {
        fits_filename: String,
        hdu_num: usize,
        row_num: usize,
        col_name: String,
    },

    /// An error derived from `FitsError`.
    #[error("{0}")]
    Fits(#[from] crate::fits_read::error::FitsError),
}
