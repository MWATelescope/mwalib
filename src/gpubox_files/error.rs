// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Errors associated with reading in gpubox files.

#[cfg(any(feature = "python", feature = "python-stubgen"))]
use pyo3::create_exception;

#[cfg(any(feature = "python", feature = "python-stubgen"))]
use pyo3::prelude::*;

use thiserror::Error;

use crate::MWAVersion;

/// GpuboxError subtypes - mainly used by CorrelatorContext
#[derive(Error, Debug)]
pub enum GpuboxError {
    #[error("Invalid timestep index provided. The timestep index must be between 0 and {0}")]
    InvalidTimeStepIndex(usize),

    #[error("Invalid coarse chan index provided. The coarse chan index must be between 0 and {0}")]
    InvalidCoarseChanIndex(usize),

    #[error("No gpubox / mwax fits files were supplied")]
    NoGpuboxes,

    #[error("There are a mixture of gpubox filename types!")]
    Mixture,

    #[error("Could not identify the gpubox filename structure for {0:?}")]
    Unrecognised(String),

    #[error("Failed to read OBSID from {0} - is this an MWA fits file?")]
    MissingObsid(String),

    #[error(r#"OBSID {gpubox_obsid} from {gpubox_filename} does not match expected value of obs_id from metafits file {obsid}
maybe you have a mix of different files?"#)]
    ObsidMismatch {
        obsid: u32,
        gpubox_filename: String,
        gpubox_obsid: u32,
    },

    #[error("Correlator version mismatch: gpubox filenames indicate OldLegacy or Legacy but {gpubox_filename} has CORR_VER = {gpu_corr_version_value}")]
    CorrVerMismatch {
        gpubox_filename: String,
        gpu_corr_version_value: u8,
    },

    #[error("The gpubox file {gpubox_filename} has no data HDUs")]
    NoDataHDUsInGpuboxFile { gpubox_filename: String },

    #[error("Failed to read key CORR_VER from MWAX gpubox file {0}")]
    MwaxCorrVerMissing(String),

    #[error("MWAX gpubox file {0} had a CORR_VER not equal to 2")]
    MwaxCorrVerMismatch(String),

    #[error("HDU sizes in gpubox files are not equal")]
    UnequalHduSizes,

    #[error("There is an entire gpubox batch missing (expected batch {expected} got {got})")]
    BatchMissing { expected: usize, got: usize },

    #[error("There are an uneven number of files in the gpubox batches ({expected} vs {got})")]
    UnevenCountInBatches { expected: u8, got: u8 },

    #[error("Input BTreeMap was empty")]
    EmptyBTreeMap,

    #[error("NAXIS1 in first gpubox image HDU {naxis1} does not match expected value {calculated_naxis1} (metafits baselines [{metafits_baselines}] * pols [{visibility_pols}] * 2 [r,i]). NAXIS2={naxis2}")]
    LegacyNaxis1Mismatch {
        naxis1: usize,
        calculated_naxis1: i32,
        metafits_baselines: usize,
        visibility_pols: usize,
        naxis2: usize,
    },

    #[error("NAXIS2 in first gpubox image HDU {naxis2} does not match expected value {calculated_naxis2} (metafits fine chans per coarse [{metafits_fine_chans_per_coarse}])")]
    LegacyNaxis2Mismatch {
        naxis2: usize,
        calculated_naxis2: i32,
        metafits_fine_chans_per_coarse: usize,
    },

    #[error("NAXIS1 in first gpubox image HDU {naxis1} does not match expected value {calculated_naxis1} (metafits fine chans per coarse [{metafits_fine_chans_per_coarse}] * pols [{visibility_pols}] * 2 [r,i]. NAXIS2={naxis2})")]
    MwaxNaxis1Mismatch {
        naxis1: usize,
        calculated_naxis1: i32,
        metafits_fine_chans_per_coarse: usize,
        visibility_pols: usize,
        naxis2: usize,
    },

    #[error("NAXIS2 in first gpubox image HDU {naxis2} does not match expected value {calculated_naxis2} (metafits baselines [{metafits_baselines}]")]
    MwaxNaxis2Mismatch {
        naxis2: usize,
        calculated_naxis2: i32,
        metafits_baselines: usize,
    },

    #[error("Invalid MWA Version value ({mwa_version}) for this method. Only 'Corr' enum values are allowed here.")]
    InvalidMwaVersion { mwa_version: MWAVersion },

    #[error("No data exists for the provided timestep {timestep_index} and coarse channel {coarse_chan_index}.")]
    NoDataForTimeStepCoarseChannel {
        timestep_index: usize,
        coarse_chan_index: usize,
    },

    /// An error derived from `FitsError`.
    #[error("{0}")]
    Fits(#[from] crate::fits_read::error::FitsError),
}

//
// Create Python Exceptions for rust errors
//
// Add exception for PyGpuboxErrorBatchMissing
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorBatchMissing,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorCorrVerMismatch
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorCorrVerMismatch,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorEmptyBTreeMap
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorEmptyBTreeMap,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorFits
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(mwalib, PyGpuboxErrorFits, pyo3::exceptions::PyException);

// Add exception for PyGpuboxErrorInvalidCoarseChanIndex
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorInvalidCoarseChanIndex,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorInvalidMwaVersion
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorInvalidMwaVersion,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorInvalidTimeStepIndex
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorInvalidTimeStepIndex,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorLegacyNaxis1Mismatch
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorLegacyNaxis1Mismatch,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorLegacyNaxis2Mismatch
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorLegacyNaxis2Mismatch,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorMissingObsid
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorMissingObsid,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorMixture
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(mwalib, PyGpuboxErrorMixture, pyo3::exceptions::PyException);

// Add exception for PyGpuboxErrorMwaxNaxis1Mismatch
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorMwaxNaxis1Mismatch,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorMwaxNaxis2Mismatch
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorMwaxNaxis2Mismatch,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorMwaxCorrVerMismatch
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorMwaxCorrVerMismatch,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorMwaxCorrVerMissing
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorMwaxCorrVerMissing,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorNoDataForTimeStepCoarseChannel
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorNoDataForTimeStepCoarseChannel,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorNoDataHDUsInGpuboxFile
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorNoDataHDUsInGpuboxFile,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorNoGpuboxes
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorNoGpuboxes,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorObsidMismatch
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorObsidMismatch,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorUnequalHduSizes
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorUnequalHduSizes,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorUnevenCountInBatches
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorUnevenCountInBatches,
    pyo3::exceptions::PyException
);

// Add exception for PyGpuboxErrorUnrecognised
#[cfg(any(feature = "python", feature = "python-stubgen"))]
create_exception!(
    mwalib,
    PyGpuboxErrorUnrecognised,
    pyo3::exceptions::PyException
);

// Convert a rust GpuBoxError to a python exception
#[cfg(any(feature = "python", feature = "python-stubgen"))]
impl std::convert::From<GpuboxError> for PyErr {
    fn from(err: GpuboxError) -> PyErr {
        match &err {
            GpuboxError::BatchMissing { .. } => PyGpuboxErrorBatchMissing::new_err(err.to_string()),
            GpuboxError::CorrVerMismatch { .. } => {
                PyGpuboxErrorCorrVerMismatch::new_err(err.to_string())
            }
            GpuboxError::EmptyBTreeMap => PyGpuboxErrorEmptyBTreeMap::new_err(err.to_string()),
            GpuboxError::Fits(_) => PyGpuboxErrorFits::new_err(err.to_string()),
            GpuboxError::InvalidCoarseChanIndex(_) => {
                PyGpuboxErrorInvalidCoarseChanIndex::new_err(err.to_string())
            }
            GpuboxError::InvalidMwaVersion { .. } => {
                PyGpuboxErrorInvalidMwaVersion::new_err(err.to_string())
            }
            GpuboxError::InvalidTimeStepIndex(_) => {
                PyGpuboxErrorInvalidTimeStepIndex::new_err(err.to_string())
            }
            GpuboxError::LegacyNaxis1Mismatch { .. } => {
                PyGpuboxErrorLegacyNaxis1Mismatch::new_err(err.to_string())
            }
            GpuboxError::LegacyNaxis2Mismatch { .. } => {
                PyGpuboxErrorLegacyNaxis2Mismatch::new_err(err.to_string())
            }
            GpuboxError::MissingObsid(_) => PyGpuboxErrorMissingObsid::new_err(err.to_string()),
            GpuboxError::Mixture => PyGpuboxErrorMixture::new_err(err.to_string()),
            GpuboxError::MwaxNaxis1Mismatch { .. } => {
                PyGpuboxErrorMwaxNaxis1Mismatch::new_err(err.to_string())
            }
            GpuboxError::MwaxNaxis2Mismatch { .. } => {
                PyGpuboxErrorMwaxNaxis2Mismatch::new_err(err.to_string())
            }
            GpuboxError::MwaxCorrVerMismatch(_) => {
                PyGpuboxErrorMwaxCorrVerMismatch::new_err(err.to_string())
            }
            GpuboxError::MwaxCorrVerMissing(_) => {
                PyGpuboxErrorMwaxCorrVerMissing::new_err(err.to_string())
            }
            GpuboxError::NoDataForTimeStepCoarseChannel { .. } => {
                PyGpuboxErrorNoDataForTimeStepCoarseChannel::new_err(err.to_string())
            }
            GpuboxError::NoDataHDUsInGpuboxFile { gpubox_filename: _ } => {
                PyGpuboxErrorNoDataHDUsInGpuboxFile::new_err(err.to_string())
            }
            GpuboxError::NoGpuboxes => PyGpuboxErrorNoGpuboxes::new_err(err.to_string()),
            GpuboxError::ObsidMismatch { .. } => {
                PyGpuboxErrorObsidMismatch::new_err(err.to_string())
            }
            GpuboxError::UnequalHduSizes => PyGpuboxErrorUnequalHduSizes::new_err(err.to_string()),
            GpuboxError::UnevenCountInBatches { .. } => {
                PyGpuboxErrorUnevenCountInBatches::new_err(err.to_string())
            }
            GpuboxError::Unrecognised(_) => PyGpuboxErrorUnrecognised::new_err(err.to_string()),
        }
    }
}
