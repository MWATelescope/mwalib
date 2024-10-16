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
fn mwalib(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<MetafitsContext>()?;
    m.add_class::<CorrelatorContext>()?;
    m.add_class::<VoltageContext>()?;
    m.add_class::<CableDelaysApplied>()?;
    m.add_class::<GeometricDelaysApplied>()?;
    m.add_class::<MWAVersion>()?;
    m.add_class::<MWAMode>()?;
    m.add_class::<Pol>()?;
    m.add_class::<ReceiverType>()?;
    m.add("MwalibError", py.get_type_bound::<MwalibError>())?;
    m.add(
        "GpuboxErrorBatchMissing",
        py.get_type_bound::<PyGpuboxErrorBatchMissing>(),
    )?;
    m.add(
        "GpuboxErrorCorrVerMismatch",
        py.get_type_bound::<PyGpuboxErrorCorrVerMismatch>(),
    )?;
    m.add(
        "GpuboxErrorEmptyBTreeMap",
        py.get_type_bound::<PyGpuboxErrorEmptyBTreeMap>(),
    )?;
    m.add("GpuboxErrorFits", py.get_type_bound::<PyGpuboxErrorFits>())?;
    m.add(
        "GpuboxErrorInvalidCoarseChanIndex",
        py.get_type_bound::<PyGpuboxErrorInvalidCoarseChanIndex>(),
    )?;
    m.add(
        "GpuboxErrorInvalidMwaVersion",
        py.get_type_bound::<PyGpuboxErrorInvalidMwaVersion>(),
    )?;
    m.add(
        "GpuboxErrorInvalidTimeStepIndex",
        py.get_type_bound::<PyGpuboxErrorInvalidTimeStepIndex>(),
    )?;
    m.add(
        "GpuboxErrorLegacyNaxis1Mismatch",
        py.get_type_bound::<PyGpuboxErrorLegacyNaxis1Mismatch>(),
    )?;
    m.add(
        "GpuboxErrorLegacyNaxis2Mismatch",
        py.get_type_bound::<PyGpuboxErrorLegacyNaxis2Mismatch>(),
    )?;
    m.add(
        "GpuboxErrorMissingObsid",
        py.get_type_bound::<PyGpuboxErrorMissingObsid>(),
    )?;
    m.add(
        "GpuboxErrorMixture",
        py.get_type_bound::<PyGpuboxErrorMixture>(),
    )?;
    m.add(
        "GpuboxErrorMwaxNaxis1Mismatch",
        py.get_type_bound::<PyGpuboxErrorMwaxNaxis1Mismatch>(),
    )?;
    m.add(
        "GpuboxErrorMwaxNaxis2Mismatch",
        py.get_type_bound::<PyGpuboxErrorMwaxNaxis2Mismatch>(),
    )?;
    m.add(
        "GpuboxErrorMwaxCorrVerMismatch",
        py.get_type_bound::<PyGpuboxErrorMwaxCorrVerMismatch>(),
    )?;
    m.add(
        "GpuboxErrorMwaxCorrVerMissing",
        py.get_type_bound::<PyGpuboxErrorMwaxCorrVerMissing>(),
    )?;
    m.add(
        "GpuboxErrorNoDataForTimeStepCoarseChannel",
        py.get_type_bound::<PyGpuboxErrorNoDataForTimeStepCoarseChannel>(),
    )?;
    m.add(
        "GpuboxErrorNoDataHDUsInGpuboxFile",
        py.get_type_bound::<PyGpuboxErrorNoDataHDUsInGpuboxFile>(),
    )?;
    m.add(
        "GpuboxErrorNoGpuboxes",
        py.get_type_bound::<PyGpuboxErrorNoGpuboxes>(),
    )?;
    m.add(
        "GpuboxErrorObsidMismatch",
        py.get_type_bound::<PyGpuboxErrorObsidMismatch>(),
    )?;
    m.add(
        "GpuboxErrorUnequalHduSizes",
        py.get_type_bound::<PyGpuboxErrorUnequalHduSizes>(),
    )?;
    m.add(
        "GpuboxErrorUnevenCountInBatches",
        py.get_type_bound::<PyGpuboxErrorUnevenCountInBatches>(),
    )?;
    m.add(
        "GpuboxErrorUnrecognised",
        py.get_type_bound::<PyGpuboxErrorUnrecognised>(),
    )?;

    m.add(
        "VoltageErrorInvalidTimeStepIndex",
        py.get_type_bound::<PyVoltageErrorInvalidTimeStepIndex>(),
    )?;
    m.add(
        "VoltageErrorInvalidCoarseChanIndex",
        py.get_type_bound::<PyVoltageErrorInvalidCoarseChanIndex>(),
    )?;
    m.add(
        "VoltageErrorNoVoltageFiles",
        py.get_type_bound::<PyVoltageErrorNoVoltageFiles>(),
    )?;
    m.add(
        "VoltageErrorInvalidBufferSize",
        py.get_type_bound::<PyVoltageErrorInvalidBufferSize>(),
    )?;
    m.add(
        "VoltageErrorInvalidGpsSecondStart",
        py.get_type_bound::<PyVoltageErrorInvalidGpsSecondStart>(),
    )?;
    m.add(
        "VoltageErrorInvalidVoltageFileSize",
        py.get_type_bound::<PyVoltageErrorInvalidVoltageFileSize>(),
    )?;
    m.add(
        "VoltageErrorInvalidGpsSecondCount",
        py.get_type_bound::<PyVoltageErrorInvalidGpsSecondCount>(),
    )?;
    m.add("VoltageError", py.get_type_bound::<PyVoltageError>())?;
    m.add(
        "VoltageErrorMixture",
        py.get_type_bound::<PyVoltageErrorMixture>(),
    )?;
    m.add(
        "VoltageErrorGpsTimeMissing",
        py.get_type_bound::<PyVoltageErrorGpsTimeMissing>(),
    )?;
    m.add(
        "VoltageErrorUnevenChannelsForGpsTime",
        py.get_type_bound::<PyVoltageErrorUnevenChannelsForGpsTime>(),
    )?;
    m.add(
        "VoltageErrorUnrecognised",
        py.get_type_bound::<PyVoltageErrorUnrecognised>(),
    )?;
    m.add(
        "VoltageErrorMissingObsid",
        py.get_type_bound::<PyVoltageErrorMissingObsid>(),
    )?;
    m.add(
        "VoltageErrorUnequalFileSizes",
        py.get_type_bound::<PyVoltageErrorUnequalFileSizes>(),
    )?;
    m.add(
        "VoltageErrorMetafitsObsidMismatch",
        py.get_type_bound::<PyVoltageErrorMetafitsObsidMismatch>(),
    )?;
    m.add(
        "VoltageErrorObsidMismatch",
        py.get_type_bound::<PyVoltageErrorObsidMismatch>(),
    )?;
    m.add(
        "VoltageErrorEmptyBTreeMap",
        py.get_type_bound::<PyVoltageErrorEmptyBTreeMap>(),
    )?;
    m.add(
        "VoltageErrorInvalidMwaVersion",
        py.get_type_bound::<PyVoltageErrorInvalidMwaVersion>(),
    )?;
    m.add(
        "VoltageErrorNoDataForTimeStepCoarseChannel",
        py.get_type_bound::<PyVoltageErrorNoDataForTimeStepCoarseChannel>(),
    )?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
