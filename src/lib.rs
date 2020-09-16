// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Definitions for what we expose to the library
*/
pub mod antenna;
pub mod baseline;
pub mod coarse_channel;
pub mod context;
pub mod convert;
pub mod error;
pub mod ffi;
pub mod fits_read;
pub mod gpubox;
pub mod misc;
pub mod rfinput;
pub mod timestep;
pub mod visibility_pol;

/// The MWA's latitude on Earth in radians. This is -26d42m11.94986s.
pub const MWA_LATITUDE_RADIANS: f64 = -0.4660608448386394;
/// The MWA's longitude on Earth in radians. This is 116d40m14.93485s.
pub const MWA_LONGITUDE_RADIANS: f64 = 2.0362898668561042;
/// The MWA's altitude in metres.
pub const MWA_ALTITUDE_METRES: f64 = 377.827;

// Re-exports.
pub use antenna::*;
pub use context::{mwalibContext, CorrelatorVersion};
pub use error::MwalibError;
pub use fits_read::*;
pub use rfinput::*;
