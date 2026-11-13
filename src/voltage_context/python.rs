// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! VoltageContext methods for Python
#[cfg(any(feature = "python", feature = "python-stubgen"))]
use super::*;
#[cfg(any(feature = "python", feature = "python-stubgen"))]
use crate::python::borrow_buffer;
#[cfg(any(feature = "python", feature = "python-stubgen"))]
use ndarray::Array;
#[cfg(any(feature = "python", feature = "python-stubgen"))]
use ndarray::Ix6;
#[cfg(any(feature = "python", feature = "python-stubgen"))]
use numpy::PyArray;
#[cfg(any(feature = "python", feature = "python-stubgen"))]
use pyo3::exceptions::PyRuntimeError;
#[cfg(feature = "python-stubgen")]
use pyo3_stub_gen_derive::gen_stub_pymethods;

/// Number of seconds in a legacy VCS timestep (file)
#[cfg(any(feature = "python", feature = "python-stubgen"))]
const LEGACY_VCS_SECONDS_PER_TIMESTEP: usize = 1;

/// Number of seconds in an MWAX VCS timestep (file)
#[cfg(any(feature = "python", feature = "python-stubgen"))]
const MWAX_VCS_SECONDS_PER_TIMESTEP: usize = 8;

#[cfg_attr(feature = "python-stubgen", gen_stub_pymethods)]
#[cfg_attr(any(feature = "python", feature = "python-stubgen"), pymethods)]
#[cfg(any(feature = "python", feature = "python-stubgen"))]
impl VoltageContext {
    /// From a path to a metafits file and paths to voltage files, create a `VoltageContext`.
    ///
    /// Args:
    ///     metafits_filename (str): filename of metafits file as a path or string.
    ///     voltage_filenames (list[str]): list of filenames of voltage files.
    ///
    /// Returns:
    ///     voltage_context (VoltageContext): a populated VoltageContext object if Ok.
    #[new]
    #[pyo3(signature = (metafits_filename, voltage_filenames), text_signature = "(metafits_filename: str, mwa_version: list[voltage_filenames])")]
    fn pyo3_new(metafits_filename: &str, voltage_filenames: Vec<String>) -> PyResult<Self> {
        // Convert the voltage filenames.
        let voltage_filenames: Vec<String> = voltage_filenames
            .into_iter()
            .map(|g| g.to_string())
            .collect();
        let c: VoltageContext = VoltageContext::new(metafits_filename, &voltage_filenames)?;
        Ok(c)
    }

    /// For a given list of voltage coarse channel indices, return a list of the center frequencies for all the fine channels in the given coarse channels.
    ///
    /// Args:
    ///     volt_coarse_chan_indices (list[int]): a list containing correlator coarse channel indices for which you want fine channels for. Does not need to be contiguous.
    ///
    /// Returns:
    ///     fine_chan_freqs_hz_array (list[float]): a vector of floats containing the centre sky frequencies of all the fine channels for the given coarse channels.
    #[pyo3(name = "get_fine_chan_freqs_hz_array")]
    fn pyo3_get_fine_chan_freqs_hz_array(
        &self,
        volt_coarse_chan_indices: Vec<usize>,
    ) -> PyResult<Vec<f64>> {
        Ok(self.get_fine_chan_freqs_hz_array(&volt_coarse_chan_indices)?)
    }

    /// Read a single timestep / coarse channel worth of data
    ///
    /// The GIL is released during the read.
    ///
    /// Args:
    ///     volt_timestep_index (int): index within the timestep array for the desired timestep. This corresponds to the element within VoltageContext.timesteps. For mwa legacy each index represents 1 second increments, for mwax it is 8 second increments.
    ///     volt_coarse_chan_index (int): index within the coarse_chan array for the desired coarse channel. This corresponds to the element within VoltageContext.coarse_chans.
    ///
    /// Returns:
    ///     data (numpy.typing.NDArray[numpy.int8]): A 6 dimensional ndarray of signed bytes containing the data, if Ok.
    ///
    /// NOTE: The shape of the ndarray is different between LegacyVCS and MWAX VCS
    /// Legacy: [second],[time sample],[chan],[ant],[pol],[complexity]
    ///         where complexity is a byte (first 4 bits for real, second 4 bits for imaginary) in 2's compliment
    /// MWAX  : [second],[voltage_block],[antenna],[pol],[sample],[r,i]
    ///
    #[pyo3(
        name = "read_file",
        text_signature = "(self, volt_timestep_index, volt_coarse_chan_index)"
    )]
    fn pyo3_read_file<'py>(
        &self,
        py: Python<'py>,
        volt_timestep_index: usize,
        volt_coarse_chan_index: usize,
    ) -> PyResult<Bound<'py, PyArray<i8, Ix6>>> {
        let shape = self.read_file_shape()?;
        let mut data: Vec<i8> = vec![0; shape.iter().product()];

        // Read without the GIL, so other Python threads can run (and read) at the same time.
        py.detach(|| self.read_file(volt_timestep_index, volt_coarse_chan_index, &mut data))?;

        // Convert the vector to a ND array (this is free).
        let data = Array::from_shape_vec(shape, data).map_err(|e| {
            PyRuntimeError::new_err(format!(
                "shape of data should match the expected dimensions {shape:?}: {e}"
            ))
        })?;
        Ok(PyArray::from_owned_array(py, data))
    }

    /// Read a single timestep / coarse channel worth of data into an existing array. The data is the same as
    /// `read_file` gives.
    ///
    /// The GIL is released during the read; do not change `buffer` from another thread until it returns.
    ///
    /// This does not make a new array, so reading many times into one array is faster than `read_file`.
    ///
    /// Args:
    ///     volt_timestep_index (int): index within the timestep array for the desired timestep. This corresponds to the element within VoltageContext.timesteps. For mwa legacy each index represents 1 second increments, for mwax it is 8 second increments.
    ///     volt_coarse_chan_index (int): index within the coarse_chan array for the desired coarse channel. This corresponds to the element within VoltageContext.coarse_chans.
    ///     buffer (numpy.typing.NDArray[numpy.int8]): 6 dimensional array to read into, with the shape read_file gives. It must be int8, C contiguous, writeable and have exactly this shape, else ValueError is raised.
    #[pyo3(
        name = "read_file_into_buffer",
        text_signature = "(self, volt_timestep_index, volt_coarse_chan_index, buffer)"
    )]
    fn pyo3_read_file_into_buffer<'py>(
        &self,
        volt_timestep_index: usize,
        volt_coarse_chan_index: usize,
        buffer: Bound<'py, PyArray<i8, Ix6>>,
    ) -> PyResult<()> {
        let shape = self.read_file_shape()?;
        let mut buffer_rw = borrow_buffer(&buffer, &shape)?;
        let buffer_slice = buffer_rw.as_slice_mut()?;

        // Read without the GIL. `buffer_rw` keeps the array borrowed (for numpy's borrow checking) until the read is
        // done.
        buffer
            .py()
            .detach(|| self.read_file(volt_timestep_index, volt_coarse_chan_index, buffer_slice))?;
        Ok(())
    }

    /// Read a single or multiple seconds of data for a coarse channel
    ///
    /// The GIL is released during the read.
    ///
    /// Args:
    ///     gps_second_start (int): GPS second within the observation to start returning data.
    ///     gps_second_count (int): number of seconds of data to return.
    ///     volt_coarse_chan_index (int): index within the coarse_chan array for the desired coarse channel. This corresponds to the element within VoltageContext.coarse_chans.
    ///
    /// Returns:
    ///     data (numpy.typing.NDArray[numpy.int8]): A 6 dimensional ndarray of signed bytes containing the data, if Ok.
    ///
    /// NOTE: The shape of the ndarray is different between LegacyVCS and MWAX VCS
    /// Legacy: [second],[time sample],[chan],[ant],[pol],[complexity]
    ///         where complexity is a byte (first 4 bits for real, second 4 bits for imaginary) in 2's compliment
    /// MWAX  : [second],[voltage_block],[antenna],[pol],[sample],[r,i]
    #[pyo3(
        name = "read_second",
        text_signature = "(self, gps_second_start, gps_second_count, volt_coarse_chan_index)"
    )]
    fn pyo3_read_second<'py>(
        &self,
        py: Python<'py>,
        gps_second_start: u64,
        gps_second_count: usize,
        volt_coarse_chan_index: usize,
    ) -> PyResult<Bound<'py, PyArray<i8, Ix6>>> {
        let shape = self.read_second_shape(gps_second_count)?;
        let mut data: Vec<i8> = vec![0; shape.iter().product()];

        // Read without the GIL, so other Python threads can run (and read) at the same time.
        py.detach(|| {
            self.read_second(
                gps_second_start,
                gps_second_count,
                volt_coarse_chan_index,
                &mut data,
            )
        })?;

        // Convert the vector to an nd array (this is free).
        let data = Array::from_shape_vec(shape, data).map_err(|e| {
            PyRuntimeError::new_err(format!(
                "shape of data should match the expected dimensions {shape:?}: {e}"
            ))
        })?;
        Ok(PyArray::from_owned_array(py, data))
    }

    /// Read a single or multiple seconds of data for a coarse channel into an existing array. The data is the same as
    /// `read_second` gives.
    ///
    /// The GIL is released during the read; do not change `buffer` from another thread until it returns.
    ///
    /// This does not make a new array, so reading many times into one array is faster than `read_second`.
    ///
    /// Args:
    ///     gps_second_start (int): GPS second within the observation to start returning data.
    ///     gps_second_count (int): number of seconds of data to return.
    ///     volt_coarse_chan_index (int): index within the coarse_chan array for the desired coarse channel. This corresponds to the element within VoltageContext.coarse_chans.
    ///     buffer (numpy.typing.NDArray[numpy.int8]): 6 dimensional array to read into, with the shape read_second (for this gps_second_count) gives. It must be int8, C contiguous, writeable and have exactly this shape, else ValueError is raised.
    #[pyo3(
        name = "read_second_into_buffer",
        text_signature = "(self, gps_second_start, gps_second_count, volt_coarse_chan_index, buffer)"
    )]
    fn pyo3_read_second_into_buffer<'py>(
        &self,
        gps_second_start: u64,
        gps_second_count: usize,
        volt_coarse_chan_index: usize,
        buffer: Bound<'py, PyArray<i8, Ix6>>,
    ) -> PyResult<()> {
        let shape = self.read_second_shape(gps_second_count)?;
        let mut buffer_rw = borrow_buffer(&buffer, &shape)?;
        let buffer_slice = buffer_rw.as_slice_mut()?;

        // Read without the GIL. `buffer_rw` keeps the array borrowed (for numpy's borrow checking) until the read is
        // done.
        buffer.py().detach(|| {
            self.read_second(
                gps_second_start,
                gps_second_count,
                volt_coarse_chan_index,
                buffer_slice,
            )
        })?;
        Ok(())
    }

    // https://pyo3.rs/v0.17.3/class/object.html#string-representations
    fn __repr__(&self) -> String {
        format!("{}", self)
    }

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

#[cfg(any(feature = "python", feature = "python-stubgen"))]
impl VoltageContext {
    /// Shape of the array `read_file` gives.
    ///
    /// # Returns
    ///
    /// * The shape, or an error for an MWA version that is not a voltage version.
    ///
    fn read_file_shape(&self) -> PyResult<[usize; 6]> {
        match self.mwa_version {
            MWAVersion::VCSLegacyRecombined => Ok([
                LEGACY_VCS_SECONDS_PER_TIMESTEP,
                self.num_samples_per_voltage_block,
                self.num_fine_chans_per_coarse,
                self.metafits_context.num_ants,
                self.metafits_context.num_ant_pols,
                self.sample_size_bytes as usize,
            ]),
            MWAVersion::VCSMWAXv2 => Ok([
                MWAX_VCS_SECONDS_PER_TIMESTEP,
                self.num_voltage_blocks_per_second,
                self.metafits_context.num_ants,
                self.metafits_context.num_ant_pols,
                self.num_samples_per_voltage_block,
                self.sample_size_bytes as usize,
            ]),
            _ => Err(
                voltage_files::error::PyVoltageErrorInvalidMwaVersion::new_err(
                    "Invalid MwaVersion",
                ),
            ),
        }
    }

    /// Shape of the array `read_second` gives for `gps_second_count` seconds.
    ///
    /// # Arguments
    ///
    /// * `gps_second_count` - Number of seconds to read.
    ///
    /// # Returns
    ///
    /// * The shape, or an error for an MWA version that is not a voltage version.
    ///
    fn read_second_shape(&self, gps_second_count: usize) -> PyResult<[usize; 6]> {
        match self.mwa_version {
            MWAVersion::VCSLegacyRecombined => Ok([
                gps_second_count,
                self.num_samples_per_voltage_block,
                self.num_fine_chans_per_coarse,
                self.metafits_context.num_ants,
                self.metafits_context.num_ant_pols,
                self.sample_size_bytes as usize,
            ]),
            MWAVersion::VCSMWAXv2 => Ok([
                gps_second_count,
                self.num_voltage_blocks_per_second,
                self.metafits_context.num_ants,
                self.metafits_context.num_ant_pols,
                self.num_samples_per_voltage_block,
                self.sample_size_bytes as usize,
            ]),
            _ => Err(
                voltage_files::error::PyVoltageErrorInvalidMwaVersion::new_err(
                    "Invalid MwaVersion",
                ),
            ),
        }
    }
}
