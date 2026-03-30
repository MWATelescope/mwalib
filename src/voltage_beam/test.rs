// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Unit tests for beam metadata
use crate::{
    coarse_channel, fits_open, fits_open_hdu, fits_open_hdu_by_name, voltage_beam, MetafitsContext,
    Rfinput,
};

#[test]
fn test_populate_beams_normal_metafits() {
    let filename = String::from("test_files/1447048976/1447048976_metafits.fits");
    const OBS_BEAMS: usize = 0;

    let mc = MetafitsContext::new(filename, Some(crate::MWAVersion::CorrMWAXv2))
        .expect("Error populating metafits");

    assert_eq!(mc.num_metafits_voltage_beams, OBS_BEAMS);
    assert_eq!(mc.metafits_voltage_beams, None);
}

#[test]
fn test_populate_beams_mwax_internal_metafits_old() {
    let filename = String::from("test_files/1449798840_bf/1449798840_metafits.fits");
    const OBS_CHANS: usize = 24;
    const OBS_TILES: usize = 256;
    const OBS_RFINPUTS: usize = OBS_TILES * 2;
    const OBS_BEAMS: usize = 9;
    const OBS_BANDWIDTH_HZ: u32 = 30720000;

    // open metafits file
    let mut fptr = fits_open!(filename).expect("Failed to open metafits file");

    // populate coarse channels
    // open primary HDU
    let primary_hdu = fits_open_hdu!(&mut fptr, 0).expect("Failed to open PRIMARY HDU");
    let (coarse_chan_mf_vec, coarse_chan_width) =
        coarse_channel::CoarseChannel::get_metafits_coarse_channel_info(
            &mut fptr,
            &primary_hdu,
            OBS_BANDWIDTH_HZ,
        )
        .expect("Failed to get coarse channel info");

    let coarse_chans = coarse_channel::CoarseChannel::populate_coarse_channels(
        crate::MWAVersion::CorrBeamformerMWAXv2,
        &coarse_chan_mf_vec,
        coarse_chan_width,
        None,
        None,
    )
    .expect("Failed to populate coarse chans");
    assert_eq!(coarse_chans.len(), OBS_CHANS);

    // populate rf_inputs
    // populate metafits tile table info
    let metafits_tile_table_hdu =
        fits_open_hdu_by_name!(&mut fptr, "TILEDATA").expect("Failed to open TILESET HDU");

    let rf_inputs = Rfinput::populate_rf_inputs(
        OBS_RFINPUTS,
        &mut fptr,
        &metafits_tile_table_hdu,
        1.03,
        OBS_CHANS,
        &None,
    )
    .expect("Failed to populate rfinputs");

    // populate ants
    let ants = crate::antenna::Antenna::populate_antennas(&rf_inputs);
    assert_eq!(ants.len(), OBS_TILES);

    // populate beams finally!
    // open beams HDU
    let beam_hdu =
        fits_open_hdu_by_name!(&mut fptr, "VOLTAGEBEAMS").expect("Failed to open VOLTAGEBEAMS HDU");
    let beams =
        voltage_beam::populate_voltage_beams(&mut fptr, &beam_hdu, None, &coarse_chans, &ants)
            .expect("Failed to populate beams");

    assert_eq!(beams.len(), OBS_BEAMS);
    assert_eq!(beams[0].number, 0);
    assert_eq!(beams[0].coherent, true);
    assert_eq!(beams[0].target_name, None);
    assert_eq!(beams[0].start_ra_deg, None);
    assert_eq!(beams[0].start_dec_deg, None);
    assert_eq!(beams[0].start_alt_deg, None);
    assert_eq!(beams[0].start_az_deg, None);

    assert_eq!(beams[8].number, 8);
    assert_eq!(beams[8].coherent, true);
    assert_eq!(beams[8].target_name, None);
    assert_eq!(beams[8].start_ra_deg, None);
    assert_eq!(beams[8].start_dec_deg, None);
    assert_eq!(beams[8].start_alt_deg, None);
    assert_eq!(beams[8].start_az_deg, None);
}

#[test]
fn test_populate_beams_internal_mwax_metafits() {
    let filename = String::from("test_files/1458492896_bf/1458492896_metafits.fits");
    const OBS_CHANS: usize = 24;
    const OBS_TILES: usize = 256;
    const OBS_RFINPUTS: usize = OBS_TILES * 2;
    const OBS_BEAMS: usize = 2;
    const OBS_BANDWIDTH_HZ: u32 = 30720000;

    // open metafits file
    let mut fptr = fits_open!(filename).expect("Failed to open metafits file");

    // populate coarse channels
    // open primary HDU
    let primary_hdu = fits_open_hdu!(&mut fptr, 0).expect("Failed to open PRIMARY HDU");
    let (coarse_chan_mf_vec, coarse_chan_width) =
        coarse_channel::CoarseChannel::get_metafits_coarse_channel_info(
            &mut fptr,
            &primary_hdu,
            OBS_BANDWIDTH_HZ,
        )
        .expect("Failed to get coarse channel info");

    let coarse_chans = coarse_channel::CoarseChannel::populate_coarse_channels(
        crate::MWAVersion::CorrBeamformerMWAXv2,
        &coarse_chan_mf_vec,
        coarse_chan_width,
        None,
        None,
    )
    .expect("Failed to populate coarse chans");
    assert_eq!(coarse_chans.len(), OBS_CHANS);

    // populate rf_inputs
    // populate metafits tile table info
    let metafits_tile_table_hdu =
        fits_open_hdu_by_name!(&mut fptr, "TILEDATA").expect("Failed to open TILESET HDU");

    let rf_inputs = Rfinput::populate_rf_inputs(
        OBS_RFINPUTS,
        &mut fptr,
        &metafits_tile_table_hdu,
        1.03,
        OBS_CHANS,
        &None,
    )
    .expect("Failed to populate rfinputs");

    // populate ants
    let ants = crate::antenna::Antenna::populate_antennas(&rf_inputs);
    assert_eq!(ants.len(), OBS_TILES);

    // populate beams finally!
    // open beams HDU
    let beam_hdu =
        fits_open_hdu_by_name!(&mut fptr, "VOLTAGEBEAMS").expect("Failed to open VOLTAGEBEAMS HDU");
    let beamaltaz_hdu =
        fits_open_hdu_by_name!(&mut fptr, "BEAMALTAZ").expect("Failed to open BEAMALTAZ HDU");

    let beams = voltage_beam::populate_voltage_beams(
        &mut fptr,
        &beam_hdu,
        Some(&beamaltaz_hdu),
        &coarse_chans,
        &ants,
    )
    .expect("Failed to populate beams");

    assert_eq!(beams.len(), OBS_BEAMS);
    assert_eq!(beams[0].number, 0);
    assert_eq!(beams[0].coherent, false);
    assert_eq!(beams[0].target_name, Some(String::from("PSR J1455-3330")));
    assert_eq!(beams[0].start_ra_deg, Some(223.94990412920006));
    assert_eq!(beams[0].start_dec_deg, Some(-33.5128896667));
    assert_eq!(beams[0].start_alt_deg, Some(62.54499154240908));
    assert_eq!(beams[0].start_az_deg, Some(112.09423355582292));

    assert_eq!(beams[1].number, 1);
    assert_eq!(beams[1].coherent, true);
    assert_eq!(beams[1].target_name, Some(String::from("PSR J1455-3330")));
    assert_eq!(beams[1].start_ra_deg, Some(223.9499));
    assert_eq!(beams[1].start_dec_deg, Some(-33.51289));
    assert_eq!(beams[1].start_alt_deg, Some(62.54499492637739));
    assert_eq!(beams[1].start_az_deg, Some(112.09423510597645));
}
