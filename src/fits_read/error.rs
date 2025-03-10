// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Errors associated with reading in fits files.

use std::path::PathBuf;

use thiserror::Error;

/// FitsError subtypes - mainly used by CorrelatorContext
#[derive(Error, Debug)]
pub enum FitsError {
    /// CFITSIO was not compiled with the REENTRANT directive
    #[error("mwalib has been compiled with a CFITSIO library which was not built with the -DREENTRANT directive")]
    CfitsioIsNotReentrant,

    /// Error when opening a fits file.
    #[error("{source_file}:{source_line}\nCouldn't open {fits_filename}: {fits_error}")]
    Open {
        fits_error: fitsio::errors::Error,
        fits_filename: PathBuf,
        source_file: &'static str,
        source_line: u32,
    },

    /// Error describing a key that couldn't be found in a fits header.
    #[error("{source_file}:{source_line}\n{fits_filename} HDU {hdu_num}: Couldn't find key {key}")]
    MissingKey {
        key: String,
        fits_filename: PathBuf,
        hdu_num: usize,
        source_file: &'static str,
        source_line: u32,
    },

    /// Error describing a HDU that couldn't be used as an image (e.g. `HduInfo::ImageInfo`).
    #[error("{source_file}:{source_line}\n{fits_filename} HDU {hdu_num}: Tried to use as an image, but not an image")]
    NotImage {
        fits_filename: PathBuf,
        hdu_num: usize,
        source_file: &'static str,
        source_line: u32,
    },

    /// Failure to read a long string.
    #[error("{source_file}:{source_line}\n{fits_filename} HDU {hdu_num}: Couldn't read a long string from {key}")]
    LongString {
        key: String,
        fits_filename: PathBuf,
        hdu_num: usize,
        source_file: &'static str,
        source_line: u32,
    },

    /// A generic error associated with the fitsio crate.
    #[error("{source_file}:{source_line}\n{fits_filename} HDU {hdu_num}: {fits_error}")]
    Fitsio {
        fits_error: fitsio::errors::Error,
        fits_filename: PathBuf,
        hdu_num: usize,
        source_file: &'static str,
        source_line: u32,
    },

    /// An error associated with parsing a string into another type.
    #[error("{source_file}:{source_line}\nCouldn't parse {key} in {fits_filename} HDU {hdu_num}")]
    Parse {
        key: String,
        fits_filename: PathBuf,
        hdu_num: usize,
        source_file: &'static str,
        source_line: u32,
    },

    /// Error when reading from an MWA metafits table cell.
    #[error("{fits_filename} HDU {hdu_num}: Failed to read table row {row_num} for {col_name} from metafits")]
    ReadCell {
        fits_filename: PathBuf,
        hdu_num: usize,
        row_num: usize,
        col_name: String,
    },

    /// Error when attempting to read a cell array.
    #[error("{fits_filename} HDU {hdu_num}: Failed to read cell array from column {col_name}, row {row_num} from metafits")]
    CellArray {
        fits_filename: PathBuf,
        hdu_num: usize,
        row_num: i64,
        col_name: String,
    },
}
