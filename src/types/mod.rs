// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use num_derive::FromPrimitive;
use std::fmt;

#[cfg(test)]
mod test;

#[cfg(feature = "python-stubgen")]
use pyo3_stub_gen_derive::gen_stub_pyclass_enum;

/// Enum for all of the known variants of file format based on Correlator version
///
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "python-stubgen", gen_stub_pyclass_enum)]
#[cfg_attr(
    any(feature = "python", feature = "python-stubgen"),
    pyo3::pyclass(eq, eq_int)
)]
pub enum MWAVersion {
    /// MWA correlator (v1.0), having data files without any batch numbers.
    CorrOldLegacy = 1,
    /// MWA correlator (v1.0), having data files with "gpubox" and batch numbers in their names.
    CorrLegacy = 2,
    /// MWAX correlator (v2.0)
    CorrMWAXv2 = 3,
    /// Legacy VCS Recombined
    VCSLegacyRecombined = 4,
    /// MWAX VCS
    VCSMWAXv2 = 5,
}

/// Implements fmt::Display for MWAVersion enum
///
/// # Arguments
///
/// * `f` - A fmt::Formatter
///
///
/// # Returns
///
/// * `fmt::Result` - Result of this method
///
///
impl fmt::Display for MWAVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MWAVersion::CorrOldLegacy => "Correlator v1 old Legacy (no file indices)",
                MWAVersion::CorrLegacy => "Correlator v1 Legacy",
                MWAVersion::CorrMWAXv2 => "Correlator v2 MWAX",
                MWAVersion::VCSLegacyRecombined => "VCS Legacy Recombined",
                MWAVersion::VCSMWAXv2 => "VCS MWAX v2",
            }
        )
    }
}

/// Visibility polarisations
///
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "python-stubgen", gen_stub_pyclass_enum)]
#[cfg_attr(
    any(feature = "python", feature = "python-stubgen"),
    pyo3::pyclass(eq, eq_int)
)]
pub enum VisPol {
    XX = 1,
    XY = 2,
    YX = 3,
    YY = 4,
}
/// Implements fmt::Display for VisPol enum
///
/// # Arguments
///
/// * `f` - A fmt::Formatter
///
///
/// # Returns
///
/// * `fmt::Result` - Result of this method
///
///
impl fmt::Display for VisPol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                VisPol::XX => "XX",
                VisPol::XY => "XY",
                VisPol::YX => "YX",
                VisPol::YY => "YY",
            }
        )
    }
}

/// The type of geometric delays applied to the data
///
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
#[cfg_attr(feature = "python-stubgen", gen_stub_pyclass_enum)]
#[cfg_attr(
    any(feature = "python", feature = "python-stubgen"),
    pyo3::pyclass(eq, eq_int)
)]
pub enum GeometricDelaysApplied {
    No = 0,
    Zenith = 1,
    TilePointing = 2,
    AzElTracking = 3,
}

/// Implements fmt::Display for GeometricDelaysApplied enum
///
/// # Arguments
///
/// * `f` - A fmt::Formatter
///
///
/// # Returns
///
/// * `fmt::Result` - Result of this method
///
///
impl fmt::Display for GeometricDelaysApplied {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GeometricDelaysApplied::No => "No",
                GeometricDelaysApplied::Zenith => "Zenith",
                GeometricDelaysApplied::TilePointing => "Tile Pointing",
                GeometricDelaysApplied::AzElTracking => "Az/El Tracking",
            }
        )
    }
}

/// Implements str::FromStr for GeometricDelaysApplied enum
///
/// # Arguments
///
/// * `input` - A &str which we want to convert to an enum
///
///
/// # Returns
///
/// * `Result<GeometricDelaysApplied, Err>` - Result of this method
///
///
impl std::str::FromStr for GeometricDelaysApplied {
    type Err = ();

    fn from_str(input: &str) -> Result<GeometricDelaysApplied, Self::Err> {
        match input {
            "No" => Ok(GeometricDelaysApplied::No),
            "Zenith" => Ok(GeometricDelaysApplied::Zenith),
            "Tile Pointing" => Ok(GeometricDelaysApplied::TilePointing),
            "Az/El Tracking" => Ok(GeometricDelaysApplied::AzElTracking),
            _ => Err(()),
        }
    }
}

/// The type of cable delays applied to the data
///
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
#[cfg_attr(feature = "python-stubgen", gen_stub_pyclass_enum)]
#[cfg_attr(
    any(feature = "python", feature = "python-stubgen"),
    pyo3::pyclass(eq, eq_int)
)]
pub enum CableDelaysApplied {
    NoCableDelaysApplied = 0,
    CableAndRecClock = 1,
    CableAndRecClockAndBeamformerDipoleDelays = 2,
}

/// Implements fmt::Display for CableDelaysApplied enum
///
/// # Arguments
///
/// * `f` - A fmt::Formatter
///
///
/// # Returns
///
/// * `fmt::Result` - Result of this method
///
///
impl fmt::Display for CableDelaysApplied {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CableDelaysApplied::NoCableDelaysApplied => "No",
                CableDelaysApplied::CableAndRecClock => "Cable and receiver clock cable length",
                CableDelaysApplied::CableAndRecClockAndBeamformerDipoleDelays =>
                    "Cable, receiver clock cable and pointing-dependent beamformer dipole delays",
            }
        )
    }
}

impl std::str::FromStr for CableDelaysApplied {
    type Err = ();

    fn from_str(input: &str) -> Result<CableDelaysApplied, Self::Err> {
        match input {
            "No" => Ok(CableDelaysApplied::NoCableDelaysApplied),
            "Cable and receiver clock cable length" => Ok(CableDelaysApplied::CableAndRecClock),
            "Cable, receiver clock cable and pointing-dependent beamformer dipole delays" => {
                Ok(CableDelaysApplied::CableAndRecClockAndBeamformerDipoleDelays)
            }
            _ => Err(()),
        }
    }
}

/// The MODE the system was in for this observation
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[cfg_attr(feature = "python-stubgen", gen_stub_pyclass_enum)]
#[cfg_attr(
    any(feature = "python", feature = "python-stubgen"),
    pyo3::pyclass(eq, eq_int)
)]
pub enum MWAMode {
    No_Capture = 0,
    Burst_Vsib = 1,
    Sw_Cor_Vsib = 2,
    Hw_Cor_Pkts = 3,
    Rts_32t = 4,
    Hw_Lfiles = 5,
    Hw_Lfiles_Nomentok = 6,
    Sw_Cor_Vsib_Nomentok = 7,
    Burst_Vsib_Synced = 8,
    Burst_Vsib_Raw = 9,
    Lfiles_Client = 16,
    No_Capture_Burst = 17,
    Enter_Burst = 18,
    Enter_Channel = 19,
    Voltage_Raw = 20,
    Corr_Mode_Change = 21,
    Voltage_Start = 22,
    Voltage_Stop = 23,
    Voltage_Buffer = 24,
    Mwax_Correlator = 30,
    Mwax_Vcs = 31,
    Mwax_Buffer = 32,
    Mwax_Beamformer = 33,
    Mwax_Corr_Bf = 34,
}

/// Implements fmt::Display for MWAMode enum
///
/// # Arguments
///
/// * `f` - A fmt::Formatter
///
///
/// # Returns
///
/// * `fmt::Result` - Result of this method
///
///
impl fmt::Display for MWAMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MWAMode::No_Capture => "NO_CAPTURE",
                MWAMode::Burst_Vsib => "BURST_VSIB",
                MWAMode::Sw_Cor_Vsib => "SW_COR_VSIB",
                MWAMode::Hw_Cor_Pkts => "HW_COR_PKTS",
                MWAMode::Rts_32t => "RTS_32T",
                MWAMode::Hw_Lfiles => "HW_LFILES",
                MWAMode::Hw_Lfiles_Nomentok => "HW_LFILES_NOMENTOK",
                MWAMode::Sw_Cor_Vsib_Nomentok => "SW_COR_VSIB_NOMENTOK",
                MWAMode::Burst_Vsib_Synced => "BURST_VSIB_SYNCED",
                MWAMode::Burst_Vsib_Raw => "BURST_VSIB_RAW",
                MWAMode::Lfiles_Client => "LFILES_CLIENT",
                MWAMode::No_Capture_Burst => "NO_CAPTURE_BURST",
                MWAMode::Enter_Burst => "ENTER_BURST",
                MWAMode::Enter_Channel => "ENTER_CHANNEL",
                MWAMode::Voltage_Raw => "VOLTAGE_RAW",
                MWAMode::Corr_Mode_Change => "CORR_MODE_CHANGE",
                MWAMode::Voltage_Start => "VOLTAGE_START",
                MWAMode::Voltage_Stop => "VOLTAGE_STOP",
                MWAMode::Voltage_Buffer => "VOLTAGE_BUFFER",
                MWAMode::Mwax_Correlator => "MWAX_CORRELATOR",
                MWAMode::Mwax_Vcs => "MWAX_VCS",
                MWAMode::Mwax_Buffer => "MWAX_BUFFER",
                MWAMode::Mwax_Beamformer => "MWAX_BEAMFORMER",
                MWAMode::Mwax_Corr_Bf => "MWAX_CORR_BF",
            }
        )
    }
}

impl std::str::FromStr for MWAMode {
    type Err = ();

    fn from_str(input: &str) -> Result<MWAMode, Self::Err> {
        match input {
            "NO_CAPTURE" => Ok(MWAMode::No_Capture),
            "BURST_VSIB" => Ok(MWAMode::Burst_Vsib),
            "SW_COR_VSIB" => Ok(MWAMode::Sw_Cor_Vsib),
            "HW_COR_PKTS" => Ok(MWAMode::Hw_Cor_Pkts),
            "RTS_32T" => Ok(MWAMode::Rts_32t),
            "HW_LFILES" => Ok(MWAMode::Hw_Lfiles),
            "HW_LFILES_NOMENTOK" => Ok(MWAMode::Hw_Lfiles_Nomentok),
            "SW_COR_VSIB_NOMENTOK" => Ok(MWAMode::Sw_Cor_Vsib_Nomentok),
            "BURST_VSIB_SYNCED" => Ok(MWAMode::Burst_Vsib_Synced),
            "BURST_VSIB_RAW" => Ok(MWAMode::Burst_Vsib_Raw),
            "LFILES_CLIENT" => Ok(MWAMode::Lfiles_Client),
            "NO_CAPTURE_BURST" => Ok(MWAMode::No_Capture_Burst),
            "ENTER_BURST" => Ok(MWAMode::Enter_Burst),
            "ENTER_CHANNEL" => Ok(MWAMode::Enter_Channel),
            "VOLTAGE_RAW" => Ok(MWAMode::Voltage_Raw),
            "CORR_MODE_CHANGE" => Ok(MWAMode::Corr_Mode_Change),
            "VOLTAGE_START" => Ok(MWAMode::Voltage_Start),
            "VOLTAGE_STOP" => Ok(MWAMode::Voltage_Stop),
            "VOLTAGE_BUFFER" => Ok(MWAMode::Voltage_Buffer),
            "MWAX_CORRELATOR" => Ok(MWAMode::Mwax_Correlator),
            "MWAX_VCS" => Ok(MWAMode::Mwax_Vcs),
            "MWAX_BUFFER" => Ok(MWAMode::Mwax_Buffer),
            "MWAX_BEAMFORMER" => Ok(MWAMode::Mwax_Beamformer),
            "MWAX_CORR_BF" => Ok(MWAMode::Mwax_Corr_Bf),
            _ => Err(()),
        }
    }
}

/// Instrument polarisation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    any(feature = "python", feature = "python-stubgen"),
    pyo3::pyclass(eq, eq_int)
)]
#[cfg_attr(feature = "python-stubgen", gen_stub_pyclass_enum)]
pub enum Pol {
    X,
    Y,
}

/// Implements fmt::Display for Pol
///
/// # Arguments
///
/// * `f` - A fmt::Formatter
///
///
/// # Returns
///
/// * `fmt::Result` - Result of this method
///
///
impl fmt::Display for Pol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Pol::X => "X",
                Pol::Y => "Y",
            }
        )
    }
}

/// ReceiverType enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
#[cfg_attr(
    any(feature = "python", feature = "python-stubgen"),
    pyo3::pyclass(eq, eq_int)
)]
#[cfg_attr(feature = "python-stubgen", gen_stub_pyclass_enum)]
#[allow(clippy::upper_case_acronyms)]
pub enum ReceiverType {
    Unknown,
    RRI,
    NI,
    Pseudo,
    SHAO,
    EDA2,
}

/// Implements fmt::Display for ReceiverType
///
/// # Arguments
///
/// * `f` - A fmt::Formatter
///
///
/// # Returns
///
/// * `fmt::Result` - Result of this method
///
///
impl fmt::Display for ReceiverType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ReceiverType::Unknown => "Unknown",
                ReceiverType::RRI => "RRI",
                ReceiverType::NI => "NI",
                ReceiverType::Pseudo => "Pseudo",
                ReceiverType::SHAO => "SHAO",
                ReceiverType::EDA2 => "EDA2",
            }
        )
    }
}

/// Implements str::FromStr for ReceiverType enum.
/// Non uppercase values are coverted to uppercase for comparision.
///
/// # Arguments
///
/// * `input` - A &str which we want to convert to an enum
///
///
/// # Returns
///
/// * `Result<ReceiverType, Err>` - Result of this method
///
///
impl std::str::FromStr for ReceiverType {
    type Err = ();

    fn from_str(input: &str) -> Result<ReceiverType, Self::Err> {
        match input.to_uppercase().as_str() {
            "RRI" => Ok(ReceiverType::RRI),
            "NI" => Ok(ReceiverType::NI),
            "PSEUDO" => Ok(ReceiverType::Pseudo),
            "SHAO" => Ok(ReceiverType::SHAO),
            "EDA2" => Ok(ReceiverType::EDA2),
            _ => Ok(ReceiverType::Unknown),
        }
    }
}

/// DataFileType enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[repr(C)]
#[cfg_attr(
    any(feature = "python", feature = "python-stubgen"),
    pyo3::pyclass(eq, eq_int)
)]
#[cfg_attr(feature = "python-stubgen", gen_stub_pyclass_enum)]
#[allow(clippy::upper_case_acronyms)]
pub enum DataFileType {
    Unknown = 0,
    Vdif = 1,
    Filterbank = 2,
}

/// Implements fmt::Display for DataFileType
///
/// # Arguments
///
/// * `f` - A fmt::Formatter
///
///
/// # Returns
///
/// * `fmt::Result` - Result of this method
///
///
impl fmt::Display for DataFileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DataFileType::Unknown => "Unknown",
                DataFileType::Vdif => "VDIF",
                DataFileType::Filterbank => "Filterbank",
            }
        )
    }
}

/// Implements str::FromStr for DataFileType enum.
/// Non uppercase values are coverted to uppercase for comparision.
///
/// # Arguments
///
/// * `input` - A &str which we want to convert to an enum
///
///
/// # Returns
///
/// * `Result<DataFileType, Err>` - Result of this method
///
///
impl std::str::FromStr for DataFileType {
    type Err = ();

    fn from_str(input: &str) -> Result<DataFileType, Self::Err> {
        match input.to_uppercase().as_str() {
            "VDIF" => Ok(DataFileType::Vdif),
            "Filterbank" => Ok(DataFileType::Filterbank),
            _ => Ok(DataFileType::Unknown),
        }
    }
}
