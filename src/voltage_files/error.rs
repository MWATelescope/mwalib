// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Errors associated with reading in voltage files.

use crate::MWAVersion;
use thiserror::Error;

#[cfg(feature = "python")]
use pyo3::create_exception;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[derive(Error, Debug)]
pub enum VoltageFileError {
    #[error("Invalid timestep index provided. The timestep index must be between 0 and {0}")]
    InvalidTimeStepIndex(usize),

    #[error("Invalid coarse chan index provided. The coarse chan index must be between 0 and {0}")]
    InvalidCoarseChanIndex(usize),

    #[error("No voltage files were supplied")]
    NoVoltageFiles,

    #[error("Provided buffer of {0} bytes is not the correct size (should be {1} bytes)")]
    InvalidBufferSize(usize, usize),

    #[error("Invalid gps_second_start (should be between {0} and {1} inclusive)")]
    InvalidGpsSecondStart(u64, u64),

    #[error("Invalid voltage file size {0} (expected {1} bytes, was {2} bytes)")]
    InvalidVoltageFileSize(u64, String, u64),

    #[error(
        "Invalid gps_second_count (gps second start {0} + count {1} - 1 cannot be greater than {1})"
    )]
    InvalidGpsSecondCount(u64, usize, u64),

    #[error("Voltage file {0} error: {1}")]
    VoltageFileError(String, String),

    #[error("There are a mixture of voltage filename types!")]
    Mixture,

    #[error(r#"There are missing gps times- expected {expected} got {got}"#)]
    GpsTimeMissing { expected: u64, got: u64 },

    #[error(r#"There are an uneven number of channel (files) across all of the gps times- expected {expected} got {got}"#)]
    UnevenChannelsForGpsTime { expected: u8, got: u8 },

    #[error(r#"Could not identify the voltage filename structure for {0}"#)]
    Unrecognised(String),

    #[error("Failed to read OBSID from {0} - is this an MWA voltage file?")]
    MissingObsid(String),

    #[error("The provided voltage files are of different sizes and this is not supported")]
    UnequalFileSizes,

    #[error("The provided metafits obsid does not match the provided filenames obsid.")]
    MetafitsObsidMismatch,

    #[error(r#"OBSID {voltage_obsid} from {voltage_filename} does not match expected value of obs_id from metafits file {obsid}
maybe you have a mix of different files?"#)]
    ObsidMismatch {
        obsid: u32,
        voltage_filename: String,
        voltage_obsid: u32,
    },
    #[error("Input BTreeMap was empty")]
    EmptyBTreeMap,

    #[error("Invalid MWA Version value ({mwa_version}) for this method. Only 'VCS' enum values are allowed here.")]
    InvalidMwaVersion { mwa_version: MWAVersion },

    #[error("No data exists for the provided timestep {timestep_index} and coarse channel {coarse_chan_index}.")]
    NoDataForTimeStepCoarseChannel {
        timestep_index: usize,
        coarse_chan_index: usize,
    },
}

//
// Create Python Exceptions for rust errors
//
// Add exception for PyVoltageErrorInvalidTimeStepIndex
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorInvalidTimeStepIndex,
    pyo3::exceptions::PyException
);

// Add exception for PyVoltageErrorInvalidCoarseChanIndex
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorInvalidCoarseChanIndex,
    pyo3::exceptions::PyException
);

// Add exception for PyVoltageErrorNoVoltageFiles
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorNoVoltageFiles,
    pyo3::exceptions::PyException
);

// Add exception for PyVoltageErrorInvalidBufferSize
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorInvalidBufferSize,
    pyo3::exceptions::PyException
);

// Add exception for PyVoltageErrorInvalidGpsSecondStart
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorInvalidGpsSecondStart,
    pyo3::exceptions::PyException
);

// Add exception for PyVoltageErrorInvalidVoltageFileSize
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorInvalidVoltageFileSize,
    pyo3::exceptions::PyException
);

// Add exception for PyVoltageErrorInvalidGpsSecondCount
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorInvalidGpsSecondCount,
    pyo3::exceptions::PyException
);

// Add exception for PyVoltageError
#[cfg(feature = "python")]
create_exception!(mwalib, PyVoltageError, pyo3::exceptions::PyException);

// Add exception for PyVoltageErrorMixture
#[cfg(feature = "python")]
create_exception!(mwalib, PyVoltageErrorMixture, pyo3::exceptions::PyException);

// Add exception for PyVoltageErrorGpsTimeMissing
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorGpsTimeMissing,
    pyo3::exceptions::PyException
);

// Add exception for PyVoltageErrorUnevenChannelsForGpsTime
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorUnevenChannelsForGpsTime,
    pyo3::exceptions::PyException
);

// Add exception for PyVoltageErrorUnrecognised
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorUnrecognised,
    pyo3::exceptions::PyException
);

// Add exception for PyVoltageErrorMissingObsid
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorMissingObsid,
    pyo3::exceptions::PyException
);

// Add exception for PyVoltageErrorUnequalFileSizes
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorUnequalFileSizes,
    pyo3::exceptions::PyException
);

// Add exception for PyVoltageErrorMetafitsObsidMismatch
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorMetafitsObsidMismatch,
    pyo3::exceptions::PyException
);

// Add exception for PyVoltageErrorObsidMismatch
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorObsidMismatch,
    pyo3::exceptions::PyException
);

// Add exception for PyVoltageErrorEmptyBTreeMap
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorEmptyBTreeMap,
    pyo3::exceptions::PyException
);

// Add exception for PyVoltageErrorInvalidMwaVersion
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorInvalidMwaVersion,
    pyo3::exceptions::PyException
);

// Add exception for PyVoltageErrorNoDataForTimeStepCoarseChannel
#[cfg(feature = "python")]
create_exception!(
    mwalib,
    PyVoltageErrorNoDataForTimeStepCoarseChannel,
    pyo3::exceptions::PyException
);

// Convert a rust VoltageFileError to a python exception
#[cfg(feature = "python")]
impl std::convert::From<VoltageFileError> for PyErr {
    fn from(err: VoltageFileError) -> PyErr {
        match &err {
            VoltageFileError::InvalidTimeStepIndex(_) => {
                PyVoltageErrorInvalidTimeStepIndex::new_err(err.to_string())
            }
            VoltageFileError::InvalidCoarseChanIndex(_) => {
                PyVoltageErrorInvalidCoarseChanIndex::new_err(err.to_string())
            }

            VoltageFileError::NoVoltageFiles => {
                PyVoltageErrorNoVoltageFiles::new_err(err.to_string())
            }

            VoltageFileError::InvalidBufferSize(..) => {
                PyVoltageErrorInvalidBufferSize::new_err(err.to_string())
            }

            VoltageFileError::InvalidGpsSecondStart(..) => {
                PyVoltageErrorInvalidGpsSecondStart::new_err(err.to_string())
            }

            VoltageFileError::InvalidVoltageFileSize(..) => {
                PyVoltageErrorInvalidVoltageFileSize::new_err(err.to_string())
            }

            VoltageFileError::InvalidGpsSecondCount(..) => {
                PyVoltageErrorInvalidGpsSecondCount::new_err(err.to_string())
            }

            VoltageFileError::VoltageFileError(..) => PyVoltageError::new_err(err.to_string()),

            VoltageFileError::Mixture => PyVoltageErrorMixture::new_err(err.to_string()),

            VoltageFileError::GpsTimeMissing { .. } => {
                PyVoltageErrorGpsTimeMissing::new_err(err.to_string())
            }

            VoltageFileError::UnevenChannelsForGpsTime { .. } => {
                PyVoltageErrorUnevenChannelsForGpsTime::new_err(err.to_string())
            }

            VoltageFileError::Unrecognised(_) => {
                PyVoltageErrorUnrecognised::new_err(err.to_string())
            }

            VoltageFileError::MissingObsid(_) => {
                PyVoltageErrorMissingObsid::new_err(err.to_string())
            }

            VoltageFileError::UnequalFileSizes { .. } => {
                PyVoltageErrorUnequalFileSizes::new_err(err.to_string())
            }

            VoltageFileError::MetafitsObsidMismatch { .. } => {
                PyVoltageErrorMetafitsObsidMismatch::new_err(err.to_string())
            }

            VoltageFileError::ObsidMismatch { .. } => {
                PyVoltageErrorObsidMismatch::new_err(err.to_string())
            }

            VoltageFileError::EmptyBTreeMap { .. } => {
                PyVoltageErrorEmptyBTreeMap::new_err(err.to_string())
            }

            VoltageFileError::InvalidMwaVersion { .. } => {
                PyVoltageErrorInvalidMwaVersion::new_err(err.to_string())
            }

            VoltageFileError::NoDataForTimeStepCoarseChannel { .. } => {
                PyVoltageErrorNoDataForTimeStepCoarseChannel::new_err(err.to_string())
            }
        }
    }
}
