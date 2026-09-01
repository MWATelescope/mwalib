// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! pyo3 conversions between [`super::PyTimestamp`] and Python's `datetime.datetime`.
//!
//! `jiff::Timestamp` already has working `IntoPyObject`/`FromPyObject` via pyo3's own
//! `jiff-02` feature, so these impls are thin delegations to those - this module exists only
//! to attach a `PyStubType` implementation to `PyTimestamp`, which `Timestamp` itself lacks.

use super::PyTimestamp;
use jiff::Timestamp;
use pyo3::prelude::*;

impl<'py> IntoPyObject<'py> for PyTimestamp {
    type Target = <Timestamp as IntoPyObject<'py>>::Target;
    type Output = <Timestamp as IntoPyObject<'py>>::Output;
    type Error = <Timestamp as IntoPyObject<'py>>::Error;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.0.into_pyobject(py)
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for PyTimestamp {
    type Error = <Timestamp as FromPyObject<'a, 'py>>::Error;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        Timestamp::extract(obj).map(PyTimestamp)
    }
}

#[cfg(feature = "python-stubgen")]
impl pyo3_stub_gen::PyStubType for PyTimestamp {
    fn type_output() -> pyo3_stub_gen::TypeInfo {
        pyo3_stub_gen::TypeInfo::with_module("datetime.datetime", "datetime".into())
    }
}
