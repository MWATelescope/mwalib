// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
Errors associated with reading in gpubox files.
*/

use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::upper_case_acronyms)]
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

    #[error("Failed to read key CORR_VER from MWAX gpubox file {0}")]
    MWAXCorrVerMissing(String),

    #[error("MWAX gpubox file {0} had a CORR_VER not equal to 2")]
    MWAXCorrVerMismatch(String),

    #[error("HDU sizes in gpubox files are not equal")]
    UnequalHduSizes,

    #[error("There is an entire gpubox batch missing (expected batch {expected} got {got})")]
    BatchMissing { expected: usize, got: usize },

    #[error("There are an uneven number of files in the gpubox batches ({expected} vs {got})")]
    UnevenCountInBatches { expected: u8, got: u8 },

    #[error("Input BTreeMap was empty")]
    EmptyBTreeMap,

    #[error("NAXIS1 in first gpubox image HDU {naxis1} does not match expected value {calculated_naxis1} (metafits baselines [{metafits_baselines}] * pols [{visibility_pols}] * 2 [r,i]). NAXIS2={naxis2}")]
    LegacyNAXIS1Mismatch {
        naxis1: usize,
        calculated_naxis1: i32,
        metafits_baselines: usize,
        visibility_pols: usize,
        naxis2: usize,
    },

    #[error("NAXIS2 in first gpubox image HDU {naxis2} does not match expected value {calculated_naxis2} (metafits fine chans per coarse [{metafits_fine_chans_per_coarse}])")]
    LegacyNAXIS2Mismatch {
        naxis2: usize,
        calculated_naxis2: i32,
        metafits_fine_chans_per_coarse: usize,
    },

    #[error("NAXIS1 in first gpubox image HDU {naxis1} does not match expected value {calculated_naxis1} (metafits fine chans per coarse [{metafits_fine_chans_per_coarse}] * pols [{visibility_pols}] * 2 [r,i]. NAXIS2={naxis2})")]
    MwaxNAXIS1Mismatch {
        naxis1: usize,
        calculated_naxis1: i32,
        metafits_fine_chans_per_coarse: usize,
        visibility_pols: usize,
        naxis2: usize,
    },

    #[error("NAXIS2 in first gpubox image HDU {naxis2} does not match expected value {calculated_naxis2} (metafits baselines [{metafits_baselines}]")]
    MwaxNAXIS2Mismatch {
        naxis2: usize,
        calculated_naxis2: i32,
        metafits_baselines: usize,
    },

    /// An error derived from `FitsError`.
    #[error("{0}")]
    Fits(#[from] crate::fits_read::error::FitsError),
}
