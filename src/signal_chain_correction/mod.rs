use crate::rfinput::ReceiverType;
use crate::MAX_RECEIVER_CHANNELS;
use std::fmt;

#[cfg(feature = "python")]
use pyo3_stub_gen_derive::gen_stub_pyclass;

///
/// Signal chain correction table
///
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
pub struct SignalChainCorrection {
    /// Receiver Type    
    pub receiver_type: ReceiverType,

    /// Whitening Filter    
    pub whitening_filter: bool,

    /// Corrections    
    pub corrections: Vec<f64>,
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
