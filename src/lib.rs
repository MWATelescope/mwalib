// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! A library to simplify reading Murchison Widefield Array (MWA) raw visibilities, voltages and metadata.

// Definitions for what we expose to the library
// Public items will be exposed as mwalib::module.
mod antenna;
mod baseline;
mod calibration_fit;
mod coarse_channel;
mod convert;
mod correlator_context;
mod error;
mod ffi;
mod fits_read;
mod gpubox_files;
mod metafits_context;
pub mod misc;
mod rfinput;
mod signal_chain_correction;
mod timestep;
mod voltage_context;
mod voltage_files;

////////////////////////////////////////////////////////////////////////////
/// NOTE: the below constants are here for FFI compatibility
/// If you are using `Marlu`, then it's recommended to use
/// the constants from that library
///
/// The MWA's latitude on Earth in radians. This is -26d42m11.94986s.
pub const MWALIB_MWA_LATITUDE_RADIANS: f64 = -0.4660608448386394;
/// The MWA's longitude on Earth in radians. This is 116d40m14.93485s.
pub const MWALIB_MWA_LONGITUDE_RADIANS: f64 = 2.0362898668561042;
/// The MWA's altitude in metres.
pub const MWALIB_MWA_ALTITUDE_METRES: f64 = 377.827;
/// speed of light in m/s
pub const MWALIB_SPEED_OF_LIGHT_IN_VACUUM_M_PER_S: f64 = 299792458.0;
/////////////////////////////////////////////////////////////////////////////

/// the velocity factor of electic fields in RG-6 like coax cable
pub const MWALIB_MWA_COAX_V_FACTOR: f64 = 1.204;
/// the number of seconds per file in MWA Legacy Recmbined VCS
pub(crate) const MWA_VCS_LEGACY_RECOMBINED_FILE_SECONDS: u64 = 1;
/// the number of seconds per subfile in MWAX v2 VCS
pub(crate) const MWA_VCS_MWAXV2_SUBFILE_SECONDS: u64 = 8;
/// Total number of MWA Receiver coarse channels
pub(crate) const MAX_RECEIVER_CHANNELS: usize = 256;

// Include the generated built.rs code into our library
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

// Re-exports (public to other crates and in a flat structure)
pub use antenna::Antenna;
pub use baseline::Baseline;
pub use calibration_fit::error::CalibrationFitError;
pub use calibration_fit::CalibrationFit;
pub use coarse_channel::error::CoarseChannelError;
pub use coarse_channel::CoarseChannel;
pub use correlator_context::CorrelatorContext;
pub use error::MwalibError;
pub use fits_read::*;
pub use gpubox_files::GpuboxError;
pub use metafits_context::error::MetafitsError;
pub use metafits_context::{
    CableDelaysApplied, GeometricDelaysApplied, MWAMode, MWAVersion, MetafitsContext, VisPol,
};
pub use misc::*;
pub use rfinput::{error::RfinputError, Pol, ReceiverType, Rfinput};
pub use signal_chain_correction::*;
pub use timestep::TimeStep;
pub use voltage_context::VoltageContext;
pub use voltage_files::error::VoltageFileError;

// So that callers don't use a different version of fitsio, export them here.
pub use fitsio;
pub use fitsio_sys;

#[cfg(any(feature = "python", feature = "python-stubgen"))]
pub mod python;
