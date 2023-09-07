// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Python interface to mwalib via pyo3.

use pyo3::prelude::*;

use crate::{
    gpubox_files::error::*,
    rfinput::{Pol, ReceiverType},
    voltage_files::error::*,
    CableDelaysApplied, CorrelatorContext, GeometricDelaysApplied, MWAMode, MWAVersion,
    MetafitsContext, VoltageContext,
};

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
    m.add_class::<VoltageContext>()?;
    m.add_class::<CableDelaysApplied>()?;
    m.add_class::<GeometricDelaysApplied>()?;
    m.add_class::<MWAVersion>()?;
    m.add_class::<MWAMode>()?;
    m.add_class::<Pol>()?;
    m.add_class::<ReceiverType>()?;
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

    m.add(
        "VoltageErrorInvalidTimeStepIndex",
        py.get_type::<PyVoltageErrorInvalidTimeStepIndex>(),
    )?;
    m.add(
        "VoltageErrorInvalidCoarseChanIndex",
        py.get_type::<PyVoltageErrorInvalidCoarseChanIndex>(),
    )?;
    m.add(
        "VoltageErrorNoVoltageFiles",
        py.get_type::<PyVoltageErrorNoVoltageFiles>(),
    )?;
    m.add(
        "VoltageErrorInvalidBufferSize",
        py.get_type::<PyVoltageErrorInvalidBufferSize>(),
    )?;
    m.add(
        "VoltageErrorInvalidGpsSecondStart",
        py.get_type::<PyVoltageErrorInvalidGpsSecondStart>(),
    )?;
    m.add(
        "VoltageErrorInvalidVoltageFileSize",
        py.get_type::<PyVoltageErrorInvalidVoltageFileSize>(),
    )?;
    m.add(
        "VoltageErrorInvalidGpsSecondCount",
        py.get_type::<PyVoltageErrorInvalidGpsSecondCount>(),
    )?;
    m.add("VoltageError", py.get_type::<PyVoltageError>())?;
    m.add(
        "VoltageErrorMixture",
        py.get_type::<PyVoltageErrorMixture>(),
    )?;
    m.add(
        "VoltageErrorGpsTimeMissing",
        py.get_type::<PyVoltageErrorGpsTimeMissing>(),
    )?;
    m.add(
        "VoltageErrorUnevenChannelsForGpsTime",
        py.get_type::<PyVoltageErrorUnevenChannelsForGpsTime>(),
    )?;
    m.add(
        "VoltageErrorUnrecognised",
        py.get_type::<PyVoltageErrorUnrecognised>(),
    )?;
    m.add(
        "VoltageErrorMissingObsid",
        py.get_type::<PyVoltageErrorMissingObsid>(),
    )?;
    m.add(
        "VoltageErrorUnequalFileSizes",
        py.get_type::<PyVoltageErrorUnequalFileSizes>(),
    )?;
    m.add(
        "VoltageErrorMetafitsObsidMismatch",
        py.get_type::<PyVoltageErrorMetafitsObsidMismatch>(),
    )?;
    m.add(
        "VoltageErrorObsidMismatch",
        py.get_type::<PyVoltageErrorObsidMismatch>(),
    )?;
    m.add(
        "VoltageErrorEmptyBTreeMap",
        py.get_type::<PyVoltageErrorEmptyBTreeMap>(),
    )?;
    m.add(
        "VoltageErrorInvalidMwaVersion",
        py.get_type::<PyVoltageErrorInvalidMwaVersion>(),
    )?;
    m.add(
        "VoltageErrorNoDataForTimeStepCoarseChannel",
        py.get_type::<PyVoltageErrorNoDataForTimeStepCoarseChannel>(),
    )?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
