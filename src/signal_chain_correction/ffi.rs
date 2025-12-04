// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::ReceiverType;

///
/// C Representation of a `SignalChainCorrection` struct
///
///
/// Signal chain correction table
///
#[repr(C)]
pub struct SignalChainCorrection {
    /// Receiver Type
    pub receiver_type: ReceiverType,

    /// Whitening Filter
    pub whitening_filter: bool,

    /// Corrections
    pub corrections: *mut f64,
}
