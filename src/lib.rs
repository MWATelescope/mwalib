#[macro_use]
extern crate lazy_static;

pub mod data_buffer;
pub mod error;
pub mod ffi;
pub mod fits_read;
pub mod gpubox;
pub mod misc;
pub mod obs_context;

// Re-exports.
use anyhow::Context;
pub use error::ErrorKind;
pub use data_buffer::mwalibBuffer;
pub use obs_context::mwalibObsContext;
