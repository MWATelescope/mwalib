// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! CorrelatorContext methods for Python
#[cfg(any(feature = "python", feature = "python-stubgen"))]
use super::*;

#[cfg(any(feature = "python", feature = "python-stubgen"))]
use ndarray::Array2;
#[cfg(any(feature = "python", feature = "python-stubgen"))]
use ndarray::Array3;
#[cfg(any(feature = "python", feature = "python-stubgen"))]
use numpy::PyArray2;
#[cfg(any(feature = "python", feature = "python-stubgen"))]
use numpy::PyArray3;
#[cfg(feature = "python-stubgen")]
use pyo3_stub_gen_derive::gen_stub_pymethods;

#[cfg_attr(feature = "python-stubgen", gen_stub_pymethods)]
#[cfg_attr(feature = "python", pymethods)]
#[cfg(any(feature = "python", feature = "python-stubgen"))]
impl CorrelatorContext {
    /// From a path to a metafits file and paths to gpubox files, create a `CorrelatorContext`.
    ///
    /// Args:
    ///     metafits_filename (str): filename of metafits file as a path or string.
    ///     gpubox_filenames (list[str]): list of filenames of gpubox files.
    ///
    /// Returns:
    ///     correlator_context (CorelatorContext): a populated CorrelatorContext object if Ok.
    #[new]
    #[pyo3(signature = (metafits_filename, gpubox_filenames), text_signature = "(metafits_filename: str, mwa_version: list[gpubox_filenames])")]
    fn pyo3_new(metafits_filename: &str, gpubox_filenames: Vec<String>) -> PyResult<Self> {
        // Convert the gpubox filenames.
        let gpubox_filenames: Vec<String> = gpubox_filenames
            .into_iter()
            .map(|g| g.to_string())
            .collect();
        let c = CorrelatorContext::new(metafits_filename, &gpubox_filenames)?;
        Ok(c)
    }

    /// For a given list of correlator coarse channel indices, return a list of the center frequencies for all the fine channels in the given coarse channels
    ///
    /// Args:
    ///     corr_coarse_chan_indices (list[int]): a list containing correlator coarse channel indices for which you want fine channels for. Does not need to be contiguous.
    ///
    /// Returns:
    ///     fine_chan_freqs_hz_array (list[float]): a vector of floats containing the centre sky frequencies of all the fine channels for the given coarse channels.
    #[pyo3(name = "get_fine_chan_freqs_hz_array")]
    fn pyo3_get_fine_chan_freqs_hz_array(&self, corr_coarse_chan_indices: Vec<usize>) -> Vec<f64> {
        self.get_fine_chan_freqs_hz_array(&corr_coarse_chan_indices)
    }

    /// Read a single timestep for a single coarse channel. The output visibilities are in order: baseline,frequency,pol,r,i
    ///
    /// Args:
    ///     corr_timestep_index (int): index within the CorrelatorContext timestep array for the desired timestep. This corresponds to the element within CorrelatorContext.timesteps.
    ///     corr_coarse_chan_index (int): index within the CorrelatorContext coarse_chan array for the desired coarse channel. This corresponds to the element within CorrelatorContext.coarse_chans.
    ///
    /// Returns:
    ///     data (numpy.typing.NDArray[numpy.float32]): 3 dimensional ndarray of 32 bit floats containing the data in [baseline],[frequency],[pol,r,i] order, if Ok.
    #[pyo3(
        name = "read_by_baseline",
        text_signature = "(self, corr_timestep_index, corr_coarse_chan_index)"
    )]
    fn pyo3_read_by_baseline<'py>(
        &self,
        py: Python<'py>,
        corr_timestep_index: usize,
        corr_coarse_chan_index: usize,
    ) -> PyResult<Bound<'py, PyArray3<f32>>> {
        // Use the existing Rust method.
        let data = self.read_by_baseline(corr_timestep_index, corr_coarse_chan_index)?;
        // Convert the vector to a 3D array (this is free).
        let data = Array3::from_shape_vec(
            (
                self.metafits_context.num_baselines,
                self.metafits_context.num_corr_fine_chans_per_coarse,
                8,
            ),
            data,
        )
        .expect("shape of data should match expected dimensions (num_baselines, num_corr_fine_chans_per_coarse, visibility_pols * 2)");
        // Convert to a numpy array.
        let data = PyArray3::from_owned_array(py, data);
        Ok(data)
    }

    /// Read a single timestep for a single coarse channel. The output visibilities are in order: frequency,baseline,pol,r,i
    ///
    /// Args:
    ///     corr_timestep_index (int): index within the CorrelatorContext timestep array for the desired timestep. This corresponds to the element within CorrelatorContext.timesteps.
    ///     corr_coarse_chan_index (int): index within the CorrelatorContext coarse_chan array for the desired coarse channel. This corresponds to the element within CorrelatorContext.coarse_chans.
    ///
    /// Returns:
    ///     data (numpy.typing.NDArray[numpy.float32]): 3 dimensional ndarray of 32 bit floats containing the data in [frequency],[baseline],[pol,r,i] order, if Ok.
    #[pyo3(
        name = "read_by_frequency",
        text_signature = "(self, corr_timestep_index, corr_coarse_chan_index)"
    )]
    fn pyo3_read_by_frequency<'py>(
        &self,
        py: Python<'py>,
        corr_timestep_index: usize,
        corr_coarse_chan_index: usize,
    ) -> PyResult<Bound<'py, PyArray3<f32>>> {
        // Use the existing Rust method.
        let data = self.read_by_frequency(corr_timestep_index, corr_coarse_chan_index)?;
        // Convert the vector to a 3D array (this is free).
        let data = Array3::from_shape_vec(
            (
                self.metafits_context.num_corr_fine_chans_per_coarse,
                self.metafits_context.num_baselines,
                8,
            ),
            data,
        )
        .expect("shape of data should match expected dimensions (num_corr_fine_chans_per_coarse, num_baselines, visibility_pols * 2)");
        // Convert to a numpy array.
        let data = PyArray3::from_owned_array(py, data);
        Ok(data)
    }

    /// Read weights for a single timestep for a single coarse channel. The output weights are in order: baseline,pol
    ///
    /// Args:
    ///     corr_timestep_index (int): index within the CorrelatorContext timestep array for the desired timestep. This corresponds to the element within CorrelatorContext.timesteps.
    ///     corr_coarse_chan_index (int): index within the CorrelatorContext coarse_chan array for the desired coarse channel. This corresponds to the element within CorrelatorContext.coarse_chans.
    ///
    /// Returns:
    ///     data (numpy.typing.NDArray[numpy.float32]): A 2 dimensional ndarray of 32 bit floats containing the data in [baseline],[pol] order, if Ok.
    #[pyo3(
        name = "read_weights_by_baseline",
        text_signature = "(self, corr_timestep_index, corr_coarse_chan_index)"
    )]
    fn pyo3_read_weights_by_baseline<'py>(
        &self,
        py: Python<'py>,
        corr_timestep_index: usize,
        corr_coarse_chan_index: usize,
    ) -> PyResult<Bound<'py, PyArray2<f32>>> {
        // Use the existing Rust method.
        let data = self.read_weights_by_baseline(corr_timestep_index, corr_coarse_chan_index)?;
        // Convert the vector to a 3D array (this is free).
        let data = Array2::from_shape_vec(
            (
                self.metafits_context.num_baselines,
                self.metafits_context.num_visibility_pols,
            ),
            data,
        )
        .expect("shape of data should match expected dimensions (num_baselines, visibility_pols)");
        // Convert to a numpy array.
        let data = PyArray2::from_owned_array(py, data);
        Ok(data)
    }

    // https://pyo3.rs/v0.17.3/class/object.html#string-representations
    fn __repr__(&self) -> String {
        format!("{}", self)
    }

    #[pyo3()]
    fn __enter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __exit__(
        &mut self,
        _exc_type: &Bound<PyAny>,
        _exc_value: &Bound<PyAny>,
        _traceback: &Bound<PyAny>,
    ) {
    }
}
