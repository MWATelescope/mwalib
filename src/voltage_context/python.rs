// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! VoltageContext methods for Python
#[cfg(feature = "python")]
use super::*;
#[cfg(feature = "python")]
use ndarray::Array;
#[cfg(feature = "python")]
use ndarray::Dim;
#[cfg(feature = "python")]
use numpy::PyArray;
#[cfg(feature = "python")]
use pyo3_stub_gen_derive::gen_stub_pymethods;

#[cfg_attr(feature = "python", gen_stub_pymethods)]
#[cfg_attr(feature = "python", pymethods)]
#[cfg(feature = "python")]
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
    fn pyo3_new(metafits_filename: PyObject, voltage_filenames: Vec<PyObject>) -> PyResult<Self> {
        // Convert the voltage filenames.
        let voltage_filenames: Vec<String> = voltage_filenames
            .into_iter()
            .map(|g| g.to_string())
            .collect();
        let c: VoltageContext =
            VoltageContext::new(metafits_filename.to_string(), &voltage_filenames)?;
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
    fn pyo3_get_fine_chan_freqs_hz_array(&self, volt_coarse_chan_indices: Vec<usize>) -> Vec<f64> {
        self.get_fine_chan_freqs_hz_array(&volt_coarse_chan_indices)
    }

    /// Read a single timestep / coarse channel worth of data
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
    ) -> PyResult<Bound<'py, PyArray<i8, Dim<[usize; 6]>>>> {
        // Use the existing Rust method.
        let mut data: Vec<i8> = vec![
            0;
            self.num_voltage_blocks_per_timestep
                * self.metafits_context.num_rf_inputs
                * self.num_fine_chans_per_coarse
                * self.num_samples_per_voltage_block
                * self.sample_size_bytes as usize
        ];
        self.read_file(volt_timestep_index, volt_coarse_chan_index, &mut data)?;

        // Convert the vector to a ND array (this is free).
        let data = match self.mwa_version {
            MWAVersion::VCSLegacyRecombined => Array::from_shape_vec(
                (
                    1, // There is 1 second per timestep for Legacy VCS 
                    self.num_samples_per_voltage_block,
                    self.num_fine_chans_per_coarse,
                    self.metafits_context.num_ants,
                    self.metafits_context.num_ant_pols,
                    self.sample_size_bytes as usize,
                ),
                data,
            )
            .expect("shape of data should match expected dimensions of Legacy VCS Recombined data (num_samples_per_voltage_block, num_fine_chans_per_coarse, num_ants, num_ant_pols, 1)"),
            MWAVersion::VCSMWAXv2 => Array::from_shape_vec(
                (
                    8, // There are 8 seconds in a timestep for MWAX VCS
                    self.num_voltage_blocks_per_second,
                    self.metafits_context.num_ants,
                    self.metafits_context.num_ant_pols,
                    self.num_samples_per_voltage_block,
                    self.sample_size_bytes as usize,
                ),
                data,
            )
            .expect("shape of data should match expected dimensions of MWAX VCS data (num_voltage_blocks_per_timestep, num_ants, num_ant_pols, num_samples_per_voltage_block, 2)"),
            _ => {
                return Err(voltage_files::error::PyVoltageErrorInvalidMwaVersion::new_err(
                    "Invalid MwaVersion",
                ));
            }
        };
        // Convert to a numpy array.
        let data = PyArray::from_owned_array(py, data);
        Ok(data)
    }

    /// Read a single or multiple seconds of data for a coarse channel
    ///
    /// Args:
    ///     gps_second_start (int): GPS second within the observation to start returning data.
    ///     gps_second_count (int): number of seconds of data to return.
    ///     volt_coarse_chan_index (int): index within the coarse_chan array for the desired coarse channel. This corresponds to the element within VoltageContext.coarse_chans.
    ///
    /// Returns:
    ///     data (numpy.typing.NDArray[numpy.int8]): A 6 dimensional ndarray of signed bytes containing the data, if Ok.
    ///
    /// NOTE: The shape is different between LegacyVCS and MWAX VCS
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
    ) -> PyResult<Bound<'py, PyArray<i8, Dim<[usize; 6]>>>> {
        // Use the existing Rust method.
        let mut data: Vec<i8> = match self.mwa_version {
            MWAVersion::VCSMWAXv2 => vec![
                0;
                self.num_voltage_blocks_per_second
                    * self.metafits_context.num_rf_inputs
                    * self.num_samples_per_voltage_block
                    * self.metafits_context.num_volt_fine_chans_per_coarse
                    * self.sample_size_bytes as usize
                    * gps_second_count
            ],
            MWAVersion::VCSLegacyRecombined => {
                vec![
                    0;
                    self.num_voltage_blocks_per_second
                        * self.metafits_context.num_rf_inputs
                        * self.num_samples_per_voltage_block
                        * self.metafits_context.num_volt_fine_chans_per_coarse
                        * self.sample_size_bytes as usize
                        * gps_second_count
                ]
            }
            _ => {
                return Err(
                    voltage_files::error::PyVoltageErrorInvalidMwaVersion::new_err(
                        "Invalid MwaVersion",
                    ),
                );
            }
        };
        self.read_second(
            gps_second_start,
            gps_second_count,
            volt_coarse_chan_index,
            &mut data,
        )?;

        // Convert the vector to an nd array (this is free).
        let data = match self.mwa_version {
            MWAVersion::VCSLegacyRecombined => Array::from_shape_vec(
                (
                    gps_second_count,
                    self.num_samples_per_voltage_block,
                    self.num_fine_chans_per_coarse,
                    self.metafits_context.num_ants,
                    self.metafits_context.num_ant_pols,
                    self.sample_size_bytes as usize,
                ),
                data,
            )
            .expect("shape of data should match expected dimensions of Legacy VCS Recombined data (gps_second_count, num_samples_per_voltage_block, num_fine_chans_per_coarse, num_ants, num_ant_pols, 1)"),
            MWAVersion::VCSMWAXv2 => Array::from_shape_vec(
                (
                    gps_second_count,
                    self.num_voltage_blocks_per_second,
                    self.metafits_context.num_ants,
                    self.metafits_context.num_ant_pols,
                    self.num_samples_per_voltage_block,
                    self.sample_size_bytes as usize,
                ),
                data,
            )
            .expect("shape of data should match expected dimensions of MWAX VCS data (gps_second_count, num_voltage_blocks_per_timestep, num_ants, num_ant_pols, num_samples_per_voltage_block, 2)"),
            _ => {
                return Err(voltage_files::error::PyVoltageErrorInvalidMwaVersion::new_err(
                    "Invalid MwaVersion",
                ));
            }
        };
        // Convert to a numpy array.
        let data = PyArray::from_owned_array(py, data);
        Ok(data)
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
