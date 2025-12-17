use crate::types::ReceiverType;
use crate::{
    read_cell_array_f64, read_cell_string, read_cell_value, FitsError, MAX_RECEIVER_CHANNELS,
};
use std::fmt;

use fitsio::hdu::{FitsHdu, HduInfo};
use fitsio::FitsFile;
#[cfg(any(feature = "python", feature = "python-stubgen"))]
use pyo3::prelude::*;
#[cfg(feature = "python-stubgen")]
use pyo3_stub_gen_derive::gen_stub_pyclass;

pub mod ffi;

#[cfg(test)]
pub(crate) mod ffi_test;

///
/// Signal chain correction table
///
#[cfg_attr(feature = "python-stubgen", gen_stub_pyclass)]
#[cfg_attr(
    any(feature = "python", feature = "python-stubgen"),
    pyclass(get_all, set_all)
)]
#[derive(Clone, Debug, PartialEq)]
pub struct SignalChainCorrection {
    /// Receiver Type    
    pub receiver_type: ReceiverType,

    /// Whitening Filter    
    pub whitening_filter: bool,

    /// Corrections    
    pub corrections: Vec<f64>,

    /// Number of corrections
    pub num_corrections: usize,
}

/// Implements fmt::Display for SignalChainCorrection
///
/// # Arguments
///
/// * `f` - A fmt::Formatter
///
///
/// # Returns
///
/// * `fmt::Result` - Result of this method
///
///
impl fmt::Display for SignalChainCorrection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let corr: String = if !self.corrections.is_empty() {
            format!(
                "[{}..{}]",
                self.corrections[0],
                self.corrections[MAX_RECEIVER_CHANNELS - 1]
            )
        } else {
            "[]".to_string()
        };

        write!(
            f,
            "Receiver Type: {} Whitening filter: {} Corrections: {}",
            self.receiver_type, self.whitening_filter, corr
        )
    }
}

/// Read the signal chain FitsHdu and return a populated vector of `SignalChainCorrection`s
///
/// # Arguments
///
/// * `metafits_fptr` - reference to the FitsFile representing the metafits file.
///
/// * `sig_chain_hdu` - The FitsHdu containing valid signal chain corrections data.
///
/// # Returns
///
/// * Result containing a vector of signal chain corrections read from the sig_chain_hdu HDU.
///
pub(crate) fn populate_signal_chain_corrections(
    metafits_fptr: &mut FitsFile,
    sig_chain_hdu: &FitsHdu,
) -> Result<Vec<SignalChainCorrection>, FitsError> {
    // Find out how many rows there are in the table
    let rows = match &sig_chain_hdu.info {
        HduInfo::TableInfo {
            column_descriptions: _,
            num_rows,
        } => *num_rows,
        _ => 0,
    };

    let mut sig_chain_vec: Vec<SignalChainCorrection> = Vec::new();

    for row in 0..rows {
        // unwrap_or(-1)
        let whitening_filter: bool =
            read_cell_value(metafits_fptr, sig_chain_hdu, "Whitening_Filter", row).unwrap_or(-1)
                == 1;

        let rx_type_str: String =
            read_cell_string(metafits_fptr, sig_chain_hdu, "Receiver_type", row)?;
        let receiver_type: ReceiverType = rx_type_str.parse::<ReceiverType>().unwrap();

        let corrections: Vec<f64> = read_cell_array_f64(
            metafits_fptr,
            sig_chain_hdu,
            "Corrections",
            row,
            MAX_RECEIVER_CHANNELS,
        )?;

        sig_chain_vec.push(SignalChainCorrection {
            receiver_type,
            whitening_filter,
            num_corrections: corrections.len(),
            corrections,
        });
    }

    Ok(sig_chain_vec)
}
