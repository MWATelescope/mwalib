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
mod voltage_context;
mod voltage_files;

/// the velocity factor of electic fields in RG-6 like coax cable
pub const MWA_COAX_V_FACTOR: f64 = 1.204;
/// the number of seconds per file in MWA Legacy Recmbined VCS
pub(crate) const MWA_VCS_LEGACY_RECOMBINED_FILE_SECONDS: u64 = 1;
/// the number of seconds per subfile in MWAX v2 VCS
pub(crate) const MWA_VCS_MWAXV2_SUBFILE_SECONDS: u64 = 8;

// Include the generated built.rs code into our library
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

// Re-exports (public to other crates and in a flat structure)
pub use antenna::Antenna;
pub use baseline::Baseline;
pub use coarse_channel::CoarseChannel;
pub use correlator_context::CorrelatorContext;
pub use error::MwalibError;
pub use fits_read::*;
pub use metafits_context::{GeometricDelaysApplied, MWAMode, MWAVersion, MetafitsContext, VisPol};
pub use misc::*;
pub use rfinput::{Pol, Rfinput};
pub use timestep::TimeStep;
pub use voltage_context::VoltageContext;

// So that callers don't use a different version of fitsio, export them here.
pub use fitsio;
pub use fitsio_sys;
