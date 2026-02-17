// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Unit tests for types
use super::*;
use std::str::FromStr;

#[test]
fn test_mwa_version_display_corr_mwaxv2() {
    let cv = MWAVersion::CorrMWAXv2;

    assert_eq!(format!("{}", cv), "Correlator v2 MWAX");
}

#[test]
fn test_mwa_version_display_corr_legacy() {
    let cv = MWAVersion::CorrLegacy;

    assert_eq!(format!("{}", cv), "Correlator v1 Legacy");
}

#[test]
fn test_mwa_version_display_corr_old_legacy() {
    let cv = MWAVersion::CorrOldLegacy;

    assert_eq!(
        format!("{}", cv),
        "Correlator v1 old Legacy (no file indices)"
    );
}

#[test]
fn test_mwa_version_display_vcs_legacy_recombined() {
    let cv = MWAVersion::VCSLegacyRecombined;

    assert_eq!(format!("{}", cv), "VCS Legacy Recombined");
}

#[test]
fn test_mwa_version_display_vcs_mwaxv2() {
    let cv = MWAVersion::VCSMWAXv2;

    assert_eq!(format!("{}", cv), "VCS MWAX v2");
}

#[test]
fn test_geometric_delays_applied_enum() {
    let none = GeometricDelaysApplied::No;
    let zen = GeometricDelaysApplied::Zenith;
    let tile = GeometricDelaysApplied::TilePointing;
    let azel = GeometricDelaysApplied::AzElTracking;

    assert_eq!(format!("{}", none), "No");
    assert_eq!(format!("{}", zen), "Zenith");
    assert_eq!(format!("{}", tile), "Tile Pointing");
    assert_eq!(format!("{}", azel), "Az/El Tracking");

    assert!(GeometricDelaysApplied::from_str("No").is_ok());
    assert!(GeometricDelaysApplied::from_str("Zenith").is_ok());
    assert!(GeometricDelaysApplied::from_str("Tile Pointing").is_ok());
    assert!(GeometricDelaysApplied::from_str("Az/El Tracking").is_ok());
    assert!(GeometricDelaysApplied::from_str("something invalid").is_err());

    let i32_none: GeometricDelaysApplied = num_traits::FromPrimitive::from_i32(0).unwrap();
    let i32_zen: GeometricDelaysApplied = num_traits::FromPrimitive::from_i32(1).unwrap();
    let i32_tile: GeometricDelaysApplied = num_traits::FromPrimitive::from_i32(2).unwrap();
    let i32_azel: GeometricDelaysApplied = num_traits::FromPrimitive::from_i32(3).unwrap();

    assert_eq!(i32_none, GeometricDelaysApplied::No);
    assert_eq!(i32_zen, GeometricDelaysApplied::Zenith);
    assert_eq!(i32_tile, GeometricDelaysApplied::TilePointing);
    assert_eq!(i32_azel, GeometricDelaysApplied::AzElTracking);

    let geo_delay: GeometricDelaysApplied = match Some(1) {
        Some(g) => num_traits::FromPrimitive::from_i32(g).unwrap(),
        None => GeometricDelaysApplied::No,
    };
    assert_eq!(geo_delay, GeometricDelaysApplied::Zenith);
}

#[test]
fn test_mode_enum() {
    let no_capture = MWAMode::No_Capture;
    let burst_vsib = MWAMode::Burst_Vsib;
    let sw_cor_vsib = MWAMode::Sw_Cor_Vsib;
    let hw_cor_pkts = MWAMode::Hw_Cor_Pkts;
    let rts_32t = MWAMode::Rts_32t;
    let hw_lfiles = MWAMode::Hw_Lfiles;
    let hw_lfiles_nomentok = MWAMode::Hw_Lfiles_Nomentok;
    let sw_cor_vsib_nomentok = MWAMode::Sw_Cor_Vsib_Nomentok;
    let burst_vsib_synced = MWAMode::Burst_Vsib_Synced;
    let burst_vsib_raw = MWAMode::Burst_Vsib_Raw;
    let lfiles_client = MWAMode::Lfiles_Client;
    let no_capture_burst = MWAMode::No_Capture_Burst;
    let enter_burst = MWAMode::Enter_Burst;
    let enter_channel = MWAMode::Enter_Channel;
    let voltage_raw = MWAMode::Voltage_Raw;
    let corr_mode_change = MWAMode::Corr_Mode_Change;
    let voltage_start = MWAMode::Voltage_Start;
    let voltage_stop = MWAMode::Voltage_Stop;
    let voltage_buffer = MWAMode::Voltage_Buffer;
    let mwax_correlator = MWAMode::Mwax_Correlator;
    let mwax_vcs = MWAMode::Mwax_Vcs;
    let mwax_buffer = MWAMode::Mwax_Buffer;
    let mwax_beamformer = MWAMode::Mwax_Beamformer;
    let mwax_corr_bf = MWAMode::Mwax_Corr_Bf;

    assert_eq!(format!("{}", no_capture), "NO_CAPTURE");
    assert_eq!(format!("{}", burst_vsib), "BURST_VSIB");
    assert_eq!(format!("{}", sw_cor_vsib), "SW_COR_VSIB");
    assert_eq!(format!("{}", hw_cor_pkts), "HW_COR_PKTS");
    assert_eq!(format!("{}", rts_32t), "RTS_32T");
    assert_eq!(format!("{}", hw_lfiles), "HW_LFILES");
    assert_eq!(format!("{}", hw_lfiles_nomentok), "HW_LFILES_NOMENTOK");
    assert_eq!(format!("{}", sw_cor_vsib_nomentok), "SW_COR_VSIB_NOMENTOK");
    assert_eq!(format!("{}", burst_vsib_synced), "BURST_VSIB_SYNCED");
    assert_eq!(format!("{}", burst_vsib_raw), "BURST_VSIB_RAW");
    assert_eq!(format!("{}", lfiles_client), "LFILES_CLIENT");
    assert_eq!(format!("{}", no_capture_burst), "NO_CAPTURE_BURST");
    assert_eq!(format!("{}", enter_burst), "ENTER_BURST");
    assert_eq!(format!("{}", enter_channel), "ENTER_CHANNEL");
    assert_eq!(format!("{}", voltage_raw), "VOLTAGE_RAW");
    assert_eq!(format!("{}", corr_mode_change), "CORR_MODE_CHANGE");
    assert_eq!(format!("{}", voltage_start), "VOLTAGE_START");
    assert_eq!(format!("{}", voltage_stop), "VOLTAGE_STOP");
    assert_eq!(format!("{}", voltage_buffer), "VOLTAGE_BUFFER");
    assert_eq!(format!("{}", mwax_correlator), "MWAX_CORRELATOR");
    assert_eq!(format!("{}", mwax_vcs), "MWAX_VCS");
    assert_eq!(format!("{}", mwax_buffer), "MWAX_BUFFER");
    assert_eq!(format!("{}", mwax_beamformer), "MWAX_BEAMFORMER");
    assert_eq!(format!("{}", mwax_corr_bf), "MWAX_CORR_BF");

    assert!(MWAMode::from_str("NO_CAPTURE").is_ok());
    assert!(MWAMode::from_str("BURST_VSIB").is_ok());
    assert!(MWAMode::from_str("SW_COR_VSIB").is_ok());
    assert!(MWAMode::from_str("HW_COR_PKTS").is_ok());
    assert!(MWAMode::from_str("RTS_32T").is_ok());
    assert!(MWAMode::from_str("HW_LFILES").is_ok());
    assert!(MWAMode::from_str("HW_LFILES_NOMENTOK").is_ok());
    assert!(MWAMode::from_str("SW_COR_VSIB_NOMENTOK").is_ok());
    assert!(MWAMode::from_str("BURST_VSIB_SYNCED").is_ok());
    assert!(MWAMode::from_str("BURST_VSIB_RAW").is_ok());
    assert!(MWAMode::from_str("LFILES_CLIENT").is_ok());
    assert!(MWAMode::from_str("NO_CAPTURE_BURST").is_ok());
    assert!(MWAMode::from_str("ENTER_BURST").is_ok());
    assert!(MWAMode::from_str("ENTER_CHANNEL").is_ok());
    assert!(MWAMode::from_str("VOLTAGE_RAW").is_ok());
    assert!(MWAMode::from_str("CORR_MODE_CHANGE").is_ok());
    assert!(MWAMode::from_str("VOLTAGE_START").is_ok());
    assert!(MWAMode::from_str("VOLTAGE_STOP").is_ok());
    assert!(MWAMode::from_str("VOLTAGE_BUFFER").is_ok());
    assert!(MWAMode::from_str("MWAX_CORRELATOR").is_ok());
    assert!(MWAMode::from_str("MWAX_VCS").is_ok());
    assert!(MWAMode::from_str("MWAX_BUFFER").is_ok());
    assert!(MWAMode::from_str("MWAX_BEAMFORMER").is_ok());
    assert!(MWAMode::from_str("MWAX_CORR_BF").is_ok());
    assert!(MWAMode::from_str("something invalid").is_err());
}

#[test]
fn test_data_file_type_enum() {
    let vdif = DataFileType::Vdif;
    let filterbank = DataFileType::Filterbank;
    let unknown: DataFileType = DataFileType::UnknownType;

    assert_eq!(format!("{}", vdif), "VDIF");
    assert_eq!(format!("{}", filterbank), "Filterbank");
    assert_eq!(format!("{}", unknown), "Unknown");

    let i32_vdif: DataFileType = num_traits::FromPrimitive::from_i32(19).unwrap();
    let i32_filterbank: DataFileType = num_traits::FromPrimitive::from_i32(20).unwrap();

    assert_eq!(i32_vdif, DataFileType::Vdif);
    assert_eq!(i32_filterbank, DataFileType::Filterbank);

    let dft: DataFileType = match Some(19) {
        Some(d) => num_traits::FromPrimitive::from_i32(d).unwrap(),
        None => DataFileType::UnknownType,
    };
    assert_eq!(dft, DataFileType::Vdif);

    let dft: DataFileType = match Some(20) {
        Some(d) => num_traits::FromPrimitive::from_i32(d).unwrap(),
        None => DataFileType::UnknownType,
    };
    assert_eq!(dft, DataFileType::Filterbank);
}
