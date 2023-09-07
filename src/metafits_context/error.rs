// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Errors associated with reading in metafits files.

use crate::MWAMode;
use thiserror::Error;

/// Metafits error subtypes - used by MetafitsContext
#[derive(Error, Debug)]
pub enum MetafitsError {
    #[error("Unable to determine MWA Version from MODE keyword {0} from metafits")]
    UnableToDetermineMWAVersionFromMode(MWAMode),
}
