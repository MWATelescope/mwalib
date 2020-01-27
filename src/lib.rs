#[macro_use]
extern crate lazy_static;

pub mod error;
pub mod ffi;
pub mod fits_read;
pub mod gpubox;
pub mod obs_context;
pub mod types;

// Re-exports.
pub use types::mwalibObsContext;
pub use error::ErrorKind;
