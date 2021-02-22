// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Errors associated with coarse channels.
*/

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoarseChannelError {
    #[error("No gpubox or voltage time_map supplied")]
    NoGpuboxOrVoltageTimeMapSupplied,
    #[error("Gpubox AND voltage time_map supplied, which is not valid")]
    BothGpuboxAndVoltageTimeMapSupplied,

    /// An error derived from `FitsError`.
    #[error("{0}")]
    Fits(#[from] crate::fits_read::error::FitsError),
}
