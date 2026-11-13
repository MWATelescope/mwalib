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
#[cfg(any(feature = "python", feature = "python-stubgen"))]
use numpy::{ndarray::Dimension, PyArray, PyArrayMethods, PyReadwriteArray, PyUntypedArrayMethods};
#[cfg(any(feature = "python", feature = "python-stubgen"))]
use pyo3::exceptions::PyValueError;
#[cfg(feature = "python-stubgen")]
use pyo3_stub_gen_derive::gen_stub_pymethods;

/// Number of floats in one complex visibility (real and imaginary)
#[cfg(any(feature = "python", feature = "python-stubgen"))]
const FLOATS_PER_COMPLEX: usize = 2;

/// Shape of the visibilities of one timestep and coarse channel in [baseline][frequency][pol,r,i] order.
///
/// # Arguments
///
/// * `mc` - The metafits context.
///
/// # Returns
///
/// * The shape.
///
#[cfg(any(feature = "python", feature = "python-stubgen"))]
fn by_baseline_shape(mc: &MetafitsContext) -> (usize, usize, usize) {
    (
        mc.num_baselines,
        mc.num_corr_fine_chans_per_coarse,
        mc.num_visibility_pols * FLOATS_PER_COMPLEX,
    )
}

/// Shape of the visibilities of one timestep and coarse channel in [frequency][baseline][pol,r,i] order.
///
/// # Arguments
///
/// * `mc` - The metafits context.
///
/// # Returns
///
/// * The shape.
///
#[cfg(any(feature = "python", feature = "python-stubgen"))]
fn by_frequency_shape(mc: &MetafitsContext) -> (usize, usize, usize) {
    (
        mc.num_corr_fine_chans_per_coarse,
        mc.num_baselines,
        mc.num_visibility_pols * FLOATS_PER_COMPLEX,
    )
}

/// Shape of the weights of one timestep and coarse channel in [baseline][pol] order.
///
/// # Arguments
///
/// * `mc` - The metafits context.
///
/// # Returns
///
/// * The shape.
///
#[cfg(any(feature = "python", feature = "python-stubgen"))]
fn weights_shape(mc: &MetafitsContext) -> (usize, usize) {
    (mc.num_baselines, mc.num_visibility_pols)
}

/// Check that a `buffer` array has the shape `expected_shape` and is C contiguous, then borrow it for writing.
///
/// # Arguments
///
/// * `buffer` - The array the caller gave to read into.
///
/// * `expected_shape` - The shape the read gives.
///
/// # Returns
///
/// * The borrowed array, or a `ValueError` if the shape is wrong, the array is not C contiguous, is not writeable or is
///   already borrowed.
///
#[cfg(any(feature = "python", feature = "python-stubgen"))]
fn borrow_buffer<'py, D: Dimension>(
    buffer: &Bound<'py, PyArray<f32, D>>,
    expected_shape: &[usize],
) -> PyResult<PyReadwriteArray<'py, f32, D>> {
    if buffer.shape() != expected_shape {
        return Err(PyValueError::new_err(format!(
            "buffer has shape {:?}, expected {:?}",
            buffer.shape(),
            expected_shape
        )));
    }

    // A Fortran ordered array is also contiguous, but its elements would be in the wrong order
    if !buffer.is_c_contiguous() {
        return Err(PyValueError::new_err("buffer must be C contiguous"));
    }

    buffer
        .try_readwrite()
        .map_err(|e| PyValueError::new_err(format!("buffer cannot be written to: {e}")))
}

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
        // Read without the GIL, so other Python threads can run (and read) at the same time.
        let data =
            py.detach(|| self.read_by_baseline(corr_timestep_index, corr_coarse_chan_index))?;
        // Convert the vector to a 3D array (this is free).
        let data = Array3::from_shape_vec(by_baseline_shape(&self.metafits_context), data)
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
        // Read without the GIL, so other Python threads can run (and read) at the same time.
        let data =
            py.detach(|| self.read_by_frequency(corr_timestep_index, corr_coarse_chan_index))?;
        // Convert the vector to a 3D array (this is free).
        let data = Array3::from_shape_vec(by_frequency_shape(&self.metafits_context), data)
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
        // Read without the GIL, so other Python threads can run (and read) at the same time.
        let data = py.detach(|| {
            self.read_weights_by_baseline(corr_timestep_index, corr_coarse_chan_index)
        })?;
        // Convert the vector to a 2D array (this is free).
        let data = Array2::from_shape_vec(weights_shape(&self.metafits_context), data).expect(
            "shape of data should match expected dimensions (num_baselines, visibility_pols)",
        );
        // Convert to a numpy array.
        let data = PyArray2::from_owned_array(py, data);
        Ok(data)
    }

    /// Read a single timestep for a single coarse channel into an existing array. The output visibilities are in order: baseline,frequency,pol,r,i
    ///
    /// The GIL is released during the read; do not change `buffer` from another thread until it returns.
    ///
    /// This does not make a new array, so reading many times into one array is faster than `read_by_baseline`.
    ///
    /// Args:
    ///     corr_timestep_index (int): index within the CorrelatorContext timestep array for the desired timestep. This corresponds to the element within CorrelatorContext.timesteps.
    ///     corr_coarse_chan_index (int): index within the CorrelatorContext coarse_chan array for the desired coarse channel. This corresponds to the element within CorrelatorContext.coarse_chans.
    ///     buffer (numpy.typing.NDArray[numpy.float32]): 3 dimensional array to read into, in [baseline],[frequency],[pol,r,i] order. It must be float32, C contiguous, writeable and have exactly this shape, else ValueError is raised.
    #[pyo3(
        name = "read_by_baseline_into_buffer",
        text_signature = "(self, corr_timestep_index, corr_coarse_chan_index, buffer)"
    )]
    fn pyo3_read_by_baseline_into_buffer<'py>(
        &self,
        corr_timestep_index: usize,
        corr_coarse_chan_index: usize,
        buffer: Bound<'py, PyArray3<f32>>,
    ) -> PyResult<()> {
        let (d0, d1, d2) = by_baseline_shape(&self.metafits_context);
        let mut buffer_rw = borrow_buffer(&buffer, &[d0, d1, d2])?;
        let buffer_slice = buffer_rw.as_slice_mut()?;
        // Read without the GIL, so other Python threads can run (and read) at the same time. `buffer_rw` keeps the
        // array borrowed (for numpy's borrow checking) until the read is done.
        buffer.py().detach(|| {
            self.read_by_baseline_into_buffer(
                corr_timestep_index,
                corr_coarse_chan_index,
                buffer_slice,
            )
        })?;
        Ok(())
    }

    /// Read a single timestep for a single coarse channel into an existing array. The output visibilities are in order: frequency,baseline,pol,r,i
    ///
    /// The GIL is released during the read; do not change `buffer` from another thread until it returns.
    ///
    /// This does not make a new array, so reading many times into one array is faster than `read_by_frequency`.
    ///
    /// Args:
    ///     corr_timestep_index (int): index within the CorrelatorContext timestep array for the desired timestep. This corresponds to the element within CorrelatorContext.timesteps.
    ///     corr_coarse_chan_index (int): index within the CorrelatorContext coarse_chan array for the desired coarse channel. This corresponds to the element within CorrelatorContext.coarse_chans.
    ///     buffer (numpy.typing.NDArray[numpy.float32]): 3 dimensional array to read into, in [frequency],[baseline],[pol,r,i] order. It must be float32, C contiguous, writeable and have exactly this shape, else ValueError is raised.
    #[pyo3(
        name = "read_by_frequency_into_buffer",
        text_signature = "(self, corr_timestep_index, corr_coarse_chan_index, buffer)"
    )]
    fn pyo3_read_by_frequency_into_buffer<'py>(
        &self,
        corr_timestep_index: usize,
        corr_coarse_chan_index: usize,
        buffer: Bound<'py, PyArray3<f32>>,
    ) -> PyResult<()> {
        let (d0, d1, d2) = by_frequency_shape(&self.metafits_context);
        let mut buffer_rw = borrow_buffer(&buffer, &[d0, d1, d2])?;
        let buffer_slice = buffer_rw.as_slice_mut()?;
        // Read without the GIL, so other Python threads can run (and read) at the same time. `buffer_rw` keeps the
        // array borrowed (for numpy's borrow checking) until the read is done.
        buffer.py().detach(|| {
            self.read_by_frequency_into_buffer(
                corr_timestep_index,
                corr_coarse_chan_index,
                buffer_slice,
            )
        })?;
        Ok(())
    }

    /// Read a single timestep for a single coarse channel into an existing array. The output weights are in order: baseline,pol
    ///
    /// The GIL is released during the read; do not change `buffer` from another thread until it returns.
    ///
    /// This does not make a new array, so reading many times into one array is faster than `read_weights_by_baseline`.
    ///
    /// Args:
    ///     corr_timestep_index (int): index within the CorrelatorContext timestep array for the desired timestep. This corresponds to the element within CorrelatorContext.timesteps.
    ///     corr_coarse_chan_index (int): index within the CorrelatorContext coarse_chan array for the desired coarse channel. This corresponds to the element within CorrelatorContext.coarse_chans.
    ///     buffer (numpy.typing.NDArray[numpy.float32]): 2 dimensional array to read into, in [baseline],[pol] order. It must be float32, C contiguous, writeable and have exactly this shape, else ValueError is raised.
    #[pyo3(
        name = "read_weights_by_baseline_into_buffer",
        text_signature = "(self, corr_timestep_index, corr_coarse_chan_index, buffer)"
    )]
    fn pyo3_read_weights_by_baseline_into_buffer<'py>(
        &self,
        corr_timestep_index: usize,
        corr_coarse_chan_index: usize,
        buffer: Bound<'py, PyArray2<f32>>,
    ) -> PyResult<()> {
        let (d0, d1) = weights_shape(&self.metafits_context);
        let mut buffer_rw = borrow_buffer(&buffer, &[d0, d1])?;
        let buffer_slice = buffer_rw.as_slice_mut()?;
        // Read without the GIL, so other Python threads can run (and read) at the same time. `buffer_rw` keeps the
        // array borrowed (for numpy's borrow checking) until the read is done.
        buffer.py().detach(|| {
            self.read_weights_by_baseline_into_buffer(
                corr_timestep_index,
                corr_coarse_chan_index,
                buffer_slice,
            )
        })?;
        Ok(())
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
