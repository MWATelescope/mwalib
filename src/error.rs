// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Structs and helper methods for Error handling

use thiserror::Error;

/// MwalibError subtypes
#[derive(Error, Debug)]
pub enum MwalibError {
    /* // An error derived from `CfitsioError`.
    #[error("{0}")]
    Cfitsio(#[from] crate::fits_read::error::CfitsioError),*/
    /// An error derived from `FitsError`.
    #[error("{0}")]
    Fits(#[from] crate::fits_read::error::FitsError),

    /// An error derived from `RfinputError`.
    #[error("{0}")]
    CoarseChannel(#[from] crate::coarse_channel::error::CoarseChannelError),

    /// An error dervied from `MetafitsError`.
    #[error("{0}")]
    Metafits(#[from] crate::metafits_context::error::MetafitsError),

    /// An error derived from `RfinputError`.
    #[error("{0}")]
    Rfinput(#[from] crate::rfinput::error::RfinputError),

    /// An error derived from `GpuboxError`.
    #[error("{0}")]
    Gpubox(#[from] crate::gpubox_files::error::GpuboxError),

    /// An error derived from `VoltageFileError`.
    #[error("{0}")]
    Voltage(#[from] crate::voltage_files::error::VoltageFileError),

    // An error associated with parsing a string into another type.
    #[error("{source_file}:{source_line}\nCouldn't parse {key} in {fits_filename} HDU {hdu_num}")]
    Parse {
        key: String,
        fits_filename: String,
        hdu_num: usize,
        source_file: String,
        source_line: u32,
    },
}
