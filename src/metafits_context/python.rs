// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! MetafitsContext methods for Python
#[cfg(feature = "python")]
use super::*;
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymethods]
impl MetafitsContext {
    /// From a path to a metafits file, create a `MetafitsContext`.
    ///
    /// # Arguments
    ///
    /// * `metafits_filename` - filename of metafits file as a path or string.
    ///
    /// * `mwa_version` - (Optional) the MWA version the metafits should be interpreted as. Pass None to have mwalib guess based on the MODE in the metafits.
    ///
    /// # Returns
    ///
    /// * A populated MetafitsContext object if Ok.
    ///
    #[new]
    #[pyo3(signature = (metafits_filename, mwa_version=None))]
    fn pyo3_new(
        metafits_filename: pyo3::PyObject,
        mwa_version: Option<MWAVersion>,
    ) -> pyo3::PyResult<Self> {
        let m = Self::new(metafits_filename.to_string(), mwa_version)?;
        Ok(m)
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
