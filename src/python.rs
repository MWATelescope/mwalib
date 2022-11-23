// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Python interface to mwalib via pyo3.

use pyo3::prelude::*;

use crate::{CorrelatorContext, MetafitsContext};

// Add a python exception for mwalib.
pyo3::create_exception!(mwalib, MwalibError, pyo3::exceptions::PyException);
impl std::convert::From<crate::MwalibError> for PyErr {
    fn from(err: crate::MwalibError) -> PyErr {
        MwalibError::new_err(err.to_string())
    }
}

#[cfg_attr(feature = "python", pymodule)]
fn mwalib(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<MetafitsContext>()?;
    m.add_class::<CorrelatorContext>()?;
    m.add("MwalibError", py.get_type::<MwalibError>())?;
    Ok(())
}
