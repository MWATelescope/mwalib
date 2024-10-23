// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! MetafitsContext methods for Python
#[cfg(feature = "python")]
use super::*;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3_stub_gen_derive::gen_stub_pymethods;

#[cfg_attr(feature = "python", gen_stub_pymethods)]
#[cfg_attr(feature = "python", pymethods)]
#[cfg(feature = "python")]
impl MetafitsContext {
    #[new]
    #[pyo3(signature = (metafits_filename, mwa_version=None), text_signature = "(metafits_filename: str, mwa_version: typing.Optional[MWAVersion]=None)")]
    /// From a path to a metafits file, create a `MetafitsContext`.
    ///
    /// Args:
    ///     metafits_filename (str): filename of metafits file.
    ///     mwa_version (Optional[MWAVersion]): the MWA version the metafits should be interpreted as. Pass None to have mwalib guess based on the MODE in the metafits.
    ///
    /// Returns:
    ///     metafits_contex (MetafitsContex): a populated MetafitsContext object if Ok.
    fn pyo3_new(metafits_filename: &str, mwa_version: Option<MWAVersion>) -> pyo3::PyResult<Self> {
        let m = Self::new(metafits_filename, mwa_version)?;
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
