// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Definitions for what we expose to the library
Public items will be exposed as mwalib::module.
*/
mod antenna;
mod baseline;
mod coarse_channel;
mod convert;
mod correlator_context;
mod error;
mod ffi;
mod fits_read;
mod gpubox_files;
mod metafits_context;
mod misc;
mod rfinput;
mod timestep;
mod visibility_pol;
mod voltage_context;
mod voltage_files;

/// The MWA's latitude on Earth in radians. This is -26d42m11.94986s.
pub const MWA_LATITUDE_RADIANS: f64 = -0.4660608448386394;
/// The MWA's longitude on Earth in radians. This is 116d40m14.93485s.
pub const MWA_LONGITUDE_RADIANS: f64 = 2.0362898668561042;
/// The MWA's altitude in metres.
pub const MWA_ALTITUDE_METRES: f64 = 377.827;
/// the velocity factor of electic fields in RG-6 like coax cable
pub const COAX_V_FACTOR: f64 = 1.204;

// Re-exports (public to other crates and in a flat structure)
pub use antenna::Antenna;
pub use baseline::Baseline;
pub use coarse_channel::CoarseChannel;
pub use correlator_context::CorrelatorContext;
pub use error::MwalibError;
pub use fits_read::*;
pub use metafits_context::{CorrelatorVersion, MetafitsContext};
pub use misc::*;
pub use rfinput::{Pol, RFInput};
pub use timestep::TimeStep;
pub use visibility_pol::VisibilityPol;
pub use voltage_context::VoltageContext;

// So that callers don't use a different version of fitsio, export them here.
pub use fitsio;
pub use fitsio_sys;
