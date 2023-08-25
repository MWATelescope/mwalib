// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Python interface to mwalib via pyo3.

use pyo3::prelude::*;

use crate::{gpubox_files::error::*, CorrelatorContext, MetafitsContext};

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
    m.add(
        "GpuboxErrorBatchMissing",
        py.get_type::<PyGpuboxErrorBatchMissing>(),
    )?;
    m.add(
        "GpuboxErrorCorrVerMismatch",
        py.get_type::<PyGpuboxErrorCorrVerMismatch>(),
    )?;
    m.add(
        "GpuboxErrorEmptyBTreeMap",
        py.get_type::<PyGpuboxErrorEmptyBTreeMap>(),
    )?;
    m.add("GpuboxErrorFits", py.get_type::<PyGpuboxErrorFits>())?;
    m.add(
        "GpuboxErrorInvalidCoarseChanIndex",
        py.get_type::<PyGpuboxErrorInvalidCoarseChanIndex>(),
    )?;
    m.add(
        "GpuboxErrorInvalidMwaVersion",
        py.get_type::<PyGpuboxErrorInvalidMwaVersion>(),
    )?;
    m.add(
        "GpuboxErrorInvalidTimeStepIndex",
        py.get_type::<PyGpuboxErrorInvalidTimeStepIndex>(),
    )?;
    m.add(
        "GpuboxErrorLegacyNaxis1Mismatch",
        py.get_type::<PyGpuboxErrorLegacyNaxis1Mismatch>(),
    )?;
    m.add(
        "GpuboxErrorLegacyNaxis2Mismatch",
        py.get_type::<PyGpuboxErrorLegacyNaxis2Mismatch>(),
    )?;
    m.add(
        "GpuboxErrorMissingObsid",
        py.get_type::<PyGpuboxErrorMissingObsid>(),
    )?;
    m.add("GpuboxErrorMixture", py.get_type::<PyGpuboxErrorMixture>())?;
    m.add(
        "GpuboxErrorMwaxNaxis1Mismatch",
        py.get_type::<PyGpuboxErrorMwaxNaxis1Mismatch>(),
    )?;
    m.add(
        "GpuboxErrorMwaxNaxis2Mismatch",
        py.get_type::<PyGpuboxErrorMwaxNaxis2Mismatch>(),
    )?;
    m.add(
        "GpuboxErrorMwaxCorrVerMismatch",
        py.get_type::<PyGpuboxErrorMwaxCorrVerMismatch>(),
    )?;
    m.add(
        "GpuboxErrorMwaxCorrVerMissing",
        py.get_type::<PyGpuboxErrorMwaxCorrVerMissing>(),
    )?;
    m.add(
        "GpuboxErrorNoDataForTimeStepCoarseChannel",
        py.get_type::<PyGpuboxErrorNoDataForTimeStepCoarseChannel>(),
    )?;
    m.add(
        "GpuboxErrorNoDataHDUsInGpuboxFile",
        py.get_type::<PyGpuboxErrorNoDataHDUsInGpuboxFile>(),
    )?;
    m.add(
        "GpuboxErrorNoGpuboxes",
        py.get_type::<PyGpuboxErrorNoGpuboxes>(),
    )?;
    m.add(
        "GpuboxErrorObsidMismatch",
        py.get_type::<PyGpuboxErrorObsidMismatch>(),
    )?;
    m.add(
        "GpuboxErrorUnequalHduSizes",
        py.get_type::<PyGpuboxErrorUnequalHduSizes>(),
    )?;
    m.add(
        "GpuboxErrorUnevenCountInBatches",
        py.get_type::<PyGpuboxErrorUnevenCountInBatches>(),
    )?;
    m.add(
        "GpuboxErrorUnrecognised",
        py.get_type::<PyGpuboxErrorUnrecognised>(),
    )?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
