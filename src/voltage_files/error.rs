// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Errors associated with reading in voltage files.
*/
use thiserror::Error;

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
        "Invalid gps_second_count (gps second start {0} + count {1} cannot be greater than {1})"
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
}
