#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

#
# See README.md on how to build and run the tests.
#
import datetime
import itertools
import math

import pytest
from read_only_helpers import assert_read_only

import mwalib

MWAX_CORRELATOR_METAFITS = "test_files/1244973688_1_timestep/1244973688.metafits"
SIGNAL_CHAIN_LEGACY_CORRELATOR_METAFITS = "test_files/metafits_signal_chain_corr/1096952256_metafits.fits"
LEGACY_CORRELATOR_METAFITS = "test_files/1101503312_1_timestep/1101503312.metafits"
VOLTAGE_BEAM_METAFITS = "test_files/1458492896_bf/1458492896_metafits.fits"
HEIGHT_CODE_METAFITS = "test_files/metafits_tests/1478034080_metafits_height_code.fits"
DERIPPLE_METAFITS = "test_files/metafits_tests/1370752512_metafits_deripple_os.fits"
CAL_SOL_METAFITS = "test_files/metafits_cal_sol/1111842752_metafits.fits"


@pytest.fixture
def mwax_mc() -> mwalib.MetafitsContext:
    return mwalib.MetafitsContext(MWAX_CORRELATOR_METAFITS, mwalib.MWAVersion.CorrMWAXv2)


@pytest.fixture
def mwax_sc_mc() -> mwalib.MetafitsContext:
    return mwalib.MetafitsContext(SIGNAL_CHAIN_LEGACY_CORRELATOR_METAFITS, None)


@pytest.fixture
def mwax_vb_mc() -> mwalib.MetafitsContext:
    return mwalib.MetafitsContext(VOLTAGE_BEAM_METAFITS, None)


@pytest.fixture
def height_code_mc() -> mwalib.MetafitsContext:
    return mwalib.MetafitsContext(HEIGHT_CODE_METAFITS, None)


@pytest.fixture
def legacy_mc() -> mwalib.MetafitsContext:
    return mwalib.MetafitsContext(LEGACY_CORRELATOR_METAFITS, None)


def test_legacy_metafits_context(legacy_mc: mwalib.MetafitsContext):
    # Checking some attributes at random
    assert legacy_mc.obs_id == 1101503312
    assert legacy_mc.corr_fine_chan_width_hz == 10_000
    assert len(legacy_mc.metafits_fine_chan_freqs_hz) == 3072
    assert len(legacy_mc.metafits_fine_chan_freqs_hz) == legacy_mc.num_metafits_fine_chan_freqs
    # this tests MWAMode enum
    assert legacy_mc.mode == mwalib.MWAMode.Hw_Lfiles
    # this tests datetimes
    assert legacy_mc.sched_start_utc == datetime.datetime(2014, 12, 1, 21, 8, 16, tzinfo=datetime.timezone.utc)
    # testing an Option<f64>
    assert legacy_mc.ra_phase_center_degrees is None

    # receivers
    assert legacy_mc.receivers == [
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8,
        9,
        10,
        11,
        12,
        13,
        14,
        15,
        16,
    ]

    # delays
    assert legacy_mc.delays == [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]


def test_mwax_metafits_context(
    mwax_mc: mwalib.MetafitsContext,
):
    assert mwax_mc.mwa_version == mwalib.MWAVersion.CorrMWAXv2

    assert mwax_mc.cable_delays_applied == mwalib.CableDelaysApplied.NoCableDelaysApplied
    assert mwax_mc.geometric_delays_applied == mwalib.GeometricDelaysApplied.No

    # This metafits has a DELAYMOD card but with an undefined value, which mwalib
    # treats the same as the key being absent
    assert mwax_mc.delay_mode is None
    assert mwax_mc.delay_mode_description is None
    assert mwax_mc.digital_gains_applied is False

    # No beamformer delay mode keys in this metafits
    assert mwax_mc.bf_delay_mode is None
    assert mwax_mc.bf_geometric_delays_applied is None
    assert mwax_mc.bf_cable_delays_applied is None
    assert mwax_mc.bf_calibration_delays_and_gains_applied is None
    assert mwax_mc.bf_signal_chain_corrections_applied is None
    assert mwax_mc.bf_digital_gains_applied is None
    assert mwax_mc.bf_delay_mode_description is None


def test_voltage_beam_metafits_context_delay_modes(
    mwax_vb_mc: mwalib.MetafitsContext,
):
    # This beamformer metafits has the correlator delay mode keys populated
    assert mwax_vb_mc.delay_mode == mwalib.DelayMode.FullTrack
    assert mwax_vb_mc.delay_mode_description == "Phase up to track source"
    assert mwax_vb_mc.cable_delays_applied == mwalib.CableDelaysApplied.CableAndRecClock
    assert mwax_vb_mc.geometric_delays_applied == mwalib.GeometricDelaysApplied.AzElTracking

    # But no beamformer (BDELMOD) keys, so callers should fall back to the above
    assert mwax_vb_mc.bf_delay_mode is None
    assert mwax_vb_mc.bf_geometric_delays_applied is None
    assert mwax_vb_mc.bf_cable_delays_applied is None


def test_mwax_metafits_context_rf_inputs(
    mwax_mc: mwalib.MetafitsContext,
):
    # this tests lists
    assert len(mwax_mc.rf_inputs) == 256

    rfinput: mwalib.Rfinput = mwax_mc.rf_inputs[0]

    # this tests strings
    assert rfinput.tile_name == "Tile051"

    # this tests enums
    assert rfinput.pol == mwalib.Pol.X


def test_mwax_metafits_context_antennas(
    mwax_mc: mwalib.MetafitsContext,
):
    # this tests lists
    assert len(mwax_mc.antennas) == 128
    assert mwax_mc.num_ants == 128

    ant: mwalib.Antenna = mwax_mc.antennas[0]

    # this tests strings
    assert ant.tile_name == "Tile051"

    # this tests enums and objects as attributes
    assert ant.rfinput_x.pol == mwalib.Pol.X
    assert ant.rfinput_y.pol == mwalib.Pol.Y


def test_mwax_metafits_context_baselines(
    mwax_mc: mwalib.MetafitsContext,
):
    assert len(mwax_mc.baselines) == 8256
    assert mwax_mc.num_baselines == 8256

    assert mwax_mc.baselines[0].ant1_index == 0
    assert mwax_mc.baselines[0].ant2_index == 0
    assert mwax_mc.baselines[128].ant1_index == 1
    assert mwax_mc.baselines[128].ant2_index == 1
    assert mwax_mc.baselines[129].ant1_index == 1
    assert mwax_mc.baselines[129].ant2_index == 2
    assert mwax_mc.baselines[8255].ant1_index == 127
    assert mwax_mc.baselines[8255].ant2_index == 127


def test_mwax_metafits_context_coarse_chans(
    mwax_mc: mwalib.MetafitsContext,
):
    assert len(mwax_mc.metafits_coarse_chans) == 24
    assert mwax_mc.num_metafits_coarse_chans == 24

    assert mwax_mc.metafits_coarse_chans[0].rec_chan_number == 104
    assert mwax_mc.metafits_coarse_chans[23].rec_chan_number == 127


def test_mwax_metafits_context_timesteps(
    mwax_mc: mwalib.MetafitsContext,
):
    assert len(mwax_mc.metafits_timesteps) == 120
    assert mwax_mc.num_metafits_timesteps == 120

    assert mwax_mc.metafits_timesteps[0].gps_time_ms == 1244973688000
    assert mwax_mc.metafits_timesteps[119].gps_time_ms == 1244973807000


def test_mwax_metafits_context_signal_chain_corrections(
    mwax_sc_mc: mwalib.MetafitsContext,
):
    assert mwax_sc_mc.signal_chain_corrections is not None

    assert len(mwax_sc_mc.signal_chain_corrections) == 8
    assert mwax_sc_mc.rf_inputs[0].signal_chain_corrections_index == 0
    assert (
        mwax_sc_mc.signal_chain_corrections[mwax_sc_mc.rf_inputs[0].signal_chain_corrections_index].whitening_filter
        is False
    )
    assert (
        mwax_sc_mc.signal_chain_corrections[mwax_sc_mc.rf_inputs[0].signal_chain_corrections_index].receiver_type
        == mwalib.ReceiverType.RRI
    )
    assert math.isclose(
        mwax_sc_mc.signal_chain_corrections[mwax_sc_mc.rf_inputs[0].signal_chain_corrections_index].corrections[0],
        0.16073910960211837,
    )
    assert math.isclose(
        mwax_sc_mc.signal_chain_corrections[mwax_sc_mc.rf_inputs[0].signal_chain_corrections_index].corrections[255],
        0.7598147243238643,
    )


def test_mwax_metafits_context_no_signal_chain_corrections(
    mwax_mc: mwalib.MetafitsContext,
):
    assert mwax_mc.signal_chain_corrections is None
    assert mwax_mc.rf_inputs[0].signal_chain_corrections_index is None


def test_mwax_metafits_context_voltage_beams(mwax_vb_mc: mwalib.MetafitsContext):
    assert mwax_vb_mc.num_metafits_voltage_beams == 2

    assert mwax_vb_mc.metafits_voltage_beams is not None

    assert mwax_vb_mc.metafits_voltage_beams[0].coherent is False
    assert mwax_vb_mc.metafits_voltage_beams[0].number == 0
    assert mwax_vb_mc.metafits_voltage_beams[0].target_name == "PSR J1455-3330"
    assert mwax_vb_mc.metafits_voltage_beams[0].start_ra_deg == 223.94990412920006
    assert mwax_vb_mc.metafits_voltage_beams[0].start_dec_deg == -33.5128896667
    assert mwax_vb_mc.metafits_voltage_beams[0].start_alt_deg == 62.54499154240908
    assert mwax_vb_mc.metafits_voltage_beams[0].start_az_deg == 112.09423355582292

    assert mwax_vb_mc.metafits_voltage_beams[1].number == 1
    assert mwax_vb_mc.metafits_voltage_beams[1].coherent is True
    assert mwax_vb_mc.metafits_voltage_beams[1].target_name == "PSR J1455-3330"
    assert mwax_vb_mc.metafits_voltage_beams[1].start_ra_deg == 223.9499
    assert mwax_vb_mc.metafits_voltage_beams[1].start_dec_deg == -33.51289
    assert mwax_vb_mc.metafits_voltage_beams[1].start_alt_deg == 62.54499492637739
    assert mwax_vb_mc.metafits_voltage_beams[1].start_az_deg == 112.09423510597645


def test_height_code_in_metafits(height_code_mc: mwalib.MetafitsContext):
    # LB tiles have raw heights (code 0), all other tiles have code 1
    assert len(height_code_mc.rf_inputs) == 480
    for rfinput in height_code_mc.rf_inputs:
        if rfinput.tile_name.startswith("LB"):
            assert rfinput.height_code == mwalib.HeightCode.EllipsoidHeight
        else:
            assert rfinput.height_code == mwalib.HeightCode.ArrayCentreOffset

    assert len(height_code_mc.antennas) == 240
    for ant in height_code_mc.antennas:
        assert ant.height_code == ant.rfinput_x.height_code
        assert ant.height_code == ant.rfinput_y.height_code
    assert sum(a.height_code == mwalib.HeightCode.EllipsoidHeight for a in height_code_mc.antennas) == 40


def test_height_code_default_when_no_column(mwax_mc: mwalib.MetafitsContext):
    # This metafits has no Height_Code column, so the default rule is used:
    # ArrayCentreOffset, unless the tile name starts with "LB" (EllipsoidHeight)
    for rfinput in mwax_mc.rf_inputs:
        if rfinput.tile_name.startswith("LB"):
            assert rfinput.height_code == mwalib.HeightCode.EllipsoidHeight
        else:
            assert rfinput.height_code == mwalib.HeightCode.ArrayCentreOffset
    for ant in mwax_mc.antennas:
        assert ant.height_code == ant.rfinput_x.height_code


# The number of milliseconds between the unix epoch (1970-01-01) and the GPS epoch (1980-01-06) minus the 18 leap
# seconds that had been added by 2019.
GPS_TO_UNIX_OFFSET_2019_MS = 315_964_782_000

# A selection of metafits, covering each type of observation we have test files for
ALL_METAFITS = [
    pytest.param((LEGACY_CORRELATOR_METAFITS, None), id="legacy"),
    pytest.param((MWAX_CORRELATOR_METAFITS, mwalib.MWAVersion.CorrMWAXv2), id="mwax"),
    pytest.param((SIGNAL_CHAIN_LEGACY_CORRELATOR_METAFITS, None), id="signal_chain"),
    pytest.param((VOLTAGE_BEAM_METAFITS, None), id="voltage_beam"),
    pytest.param((HEIGHT_CODE_METAFITS, None), id="height_code"),
    pytest.param((DERIPPLE_METAFITS, None), id="deripple"),
    pytest.param((CAL_SOL_METAFITS, mwalib.MWAVersion.CorrLegacy), id="cal_sol"),
]


@pytest.fixture(params=ALL_METAFITS)
def any_mc(request) -> mwalib.MetafitsContext:
    filename, mwa_version = request.param
    return mwalib.MetafitsContext(filename, mwa_version)


@pytest.fixture
def cal_sol_mc() -> mwalib.MetafitsContext:
    return mwalib.MetafitsContext(CAL_SOL_METAFITS, mwalib.MWAVersion.CorrLegacy)


def test_metafits_context_counts_are_consistent(any_mc: mwalib.MetafitsContext):
    num_ants = any_mc.num_ants

    assert num_ants > 0
    assert any_mc.num_ant_pols == 2
    assert any_mc.num_rf_inputs == num_ants * any_mc.num_ant_pols
    assert len(any_mc.rf_inputs) == any_mc.num_rf_inputs
    assert len(any_mc.antennas) == num_ants

    # One baseline for each pair of antennas, including auto correlations
    assert any_mc.num_baselines == num_ants * (num_ants + 1) // 2
    assert len(any_mc.baselines) == any_mc.num_baselines

    assert len(any_mc.metafits_coarse_chans) == any_mc.num_metafits_coarse_chans
    assert len(any_mc.metafits_timesteps) == any_mc.num_metafits_timesteps
    assert len(any_mc.metafits_fine_chan_freqs_hz) == any_mc.num_metafits_fine_chan_freqs
    assert len(any_mc.receivers) == any_mc.num_receivers
    assert len(any_mc.delays) == any_mc.num_delays
    assert len(any_mc.signal_chain_corrections or []) == any_mc.num_signal_chain_corrections
    assert len(any_mc.calibration_fits or []) == any_mc.num_calibration_fits


def test_metafits_context_str_contains_obs_id(any_mc: mwalib.MetafitsContext):
    assert str(any_mc.obs_id) in str(any_mc)


def test_metafits_context_is_a_context_manager(any_mc: mwalib.MetafitsContext):
    with any_mc as mc:
        assert mc is any_mc
        assert mc.num_ants > 0


def test_metafits_context_baselines(any_mc: mwalib.MetafitsContext):
    pairs = [(b.ant1_index, b.ant2_index) for b in any_mc.baselines]

    # Baselines start with the first antenna's auto correlation and end with the last antenna's
    assert pairs[0] == (0, 0)
    assert pairs[-1] == (any_mc.num_ants - 1, any_mc.num_ants - 1)

    # Each baseline is unique, ant1 <= ant2 and both are valid antenna indices
    assert len(set(pairs)) == len(pairs)
    assert all(0 <= ant1 <= ant2 < any_mc.num_ants for ant1, ant2 in pairs)

    # Baselines are in ant1, then ant2 order
    assert pairs == sorted(pairs)


def test_metafits_context_antennas_match_rf_inputs(any_mc: mwalib.MetafitsContext):
    assert sorted(a.ant for a in any_mc.antennas) == list(range(any_mc.num_ants))

    for ant in any_mc.antennas:
        assert ant.rfinput_x.pol == mwalib.Pol.X
        assert ant.rfinput_y.pol == mwalib.Pol.Y

        # Both pols share the antenna's details
        for rfinput in (ant.rfinput_x, ant.rfinput_y):
            assert rfinput.ant == ant.ant
            assert rfinput.tile_id == ant.tile_id
            assert rfinput.tile_name == ant.tile_name

        assert ant.electrical_length_m == ant.rfinput_x.electrical_length_m
        assert ant.north_m == ant.rfinput_x.north_m
        assert ant.east_m == ant.rfinput_x.east_m
        assert ant.height_m == ant.rfinput_x.height_m
        assert ant.height_code == ant.rfinput_x.height_code


def test_metafits_context_rf_input_orders(any_mc: mwalib.MetafitsContext):
    # Input is unique to each rf_input. subfile_order is the order we want the data in, so it is a permutation of 0..N-1
    assert len({r.input for r in any_mc.rf_inputs}) == any_mc.num_rf_inputs
    assert sorted(r.subfile_order for r in any_mc.rf_inputs) == list(range(any_mc.num_rf_inputs))

    # Each antenna has one X and one Y
    assert sum(r.pol == mwalib.Pol.X for r in any_mc.rf_inputs) == any_mc.num_ants
    assert sum(r.pol == mwalib.Pol.Y for r in any_mc.rf_inputs) == any_mc.num_ants


def test_metafits_context_rf_input_gains_and_delays(any_mc: mwalib.MetafitsContext):
    for rfinput in any_mc.rf_inputs:
        assert len(rfinput.digital_gains) == any_mc.num_metafits_coarse_chans

        # 16 dipoles in a tile. A delay of 32 means the dipole is dead.
        assert len(rfinput.dipole_delays) == 16
        assert len(rfinput.dipole_gains) == 16
        for delay, gain in zip(rfinput.dipole_delays, rfinput.dipole_gains):
            assert gain == (0.0 if delay == 32 else 1.0)


def test_metafits_context_rf_input_receivers(any_mc: mwalib.MetafitsContext):
    for rfinput in any_mc.rf_inputs:
        assert isinstance(rfinput.rec_type, mwalib.ReceiverType)
        assert isinstance(rfinput.flagged, bool)
        assert isinstance(rfinput.has_whitening_filter, bool)
        assert isinstance(rfinput.height_code, mwalib.HeightCode)


def test_metafits_context_coarse_channels(any_mc: mwalib.MetafitsContext):
    chans = sorted(any_mc.metafits_coarse_chans, key=lambda c: c.chan_start_hz)

    for chan in chans:
        assert chan.chan_width_hz == any_mc.coarse_chan_width_hz
        assert chan.chan_end_hz - chan.chan_start_hz == chan.chan_width_hz
        assert chan.chan_centre_hz == chan.chan_start_hz + chan.chan_width_hz // 2

    # Channels do not overlap
    for lower, upper in itertools.pairwise(chans):
        assert upper.chan_start_hz >= lower.chan_end_hz


def test_metafits_context_timesteps_are_increasing(any_mc: mwalib.MetafitsContext):
    gps_times = [t.gps_time_ms for t in any_mc.metafits_timesteps]
    unix_times = [t.unix_time_ms for t in any_mc.metafits_timesteps]

    assert gps_times == sorted(set(gps_times))
    assert unix_times == sorted(set(unix_times))

    # The gps to unix offset only changes on a leap second, so it is constant across a (short) observation.
    # (Some metafits, e.g. beamformer observations, have no correlator timesteps)
    if gps_times:
        assert len({u - g for g, u in zip(gps_times, unix_times)}) == 1


def test_metafits_context_scheduled_times(any_mc: mwalib.MetafitsContext):
    assert any_mc.sched_end_gps_time_ms - any_mc.sched_start_gps_time_ms == any_mc.sched_duration_ms
    assert any_mc.sched_end_unix_time_ms - any_mc.sched_start_unix_time_ms == any_mc.sched_duration_ms

    # The utc datetimes are the same instants as the unix times
    for utc, unix_ms in (
        (any_mc.sched_start_utc, any_mc.sched_start_unix_time_ms),
        (any_mc.sched_end_utc, any_mc.sched_end_unix_time_ms),
    ):
        assert utc == datetime.datetime.fromtimestamp(unix_ms / 1000, tz=datetime.timezone.utc)


def test_mwax_metafits_context_scheduled_times(mwax_mc: mwalib.MetafitsContext):
    assert mwax_mc.obs_id == 1244973688
    assert mwax_mc.sched_start_gps_time_ms == mwax_mc.obs_id * 1000
    assert mwax_mc.sched_start_unix_time_ms - mwax_mc.sched_start_gps_time_ms == GPS_TO_UNIX_OFFSET_2019_MS
    assert mwax_mc.sched_start_utc == datetime.datetime(2019, 6, 19, 10, 1, 10, tzinfo=datetime.timezone.utc)

    # The first timestep is at the start of the observation
    assert mwax_mc.metafits_timesteps[0].gps_time_ms == mwax_mc.sched_start_gps_time_ms
    assert mwax_mc.metafits_timesteps[0].unix_time_ms == mwax_mc.sched_start_unix_time_ms


def test_mwax_metafits_context_fine_channels(mwax_mc: mwalib.MetafitsContext):
    freqs = mwax_mc.metafits_fine_chan_freqs_hz

    assert len(freqs) == mwax_mc.num_metafits_coarse_chans * mwax_mc.num_corr_fine_chans_per_coarse

    # Fine channels are evenly spaced
    assert all(upper - lower == mwax_mc.corr_fine_chan_width_hz for lower, upper in itertools.pairwise(freqs))


def test_mwax_metafits_context_has_no_calibration(mwax_mc: mwalib.MetafitsContext):
    assert mwax_mc.num_calibration_fits == 0
    assert not mwax_mc.calibration_fits

    assert all(r.calib_delay is None for r in mwax_mc.rf_inputs)
    assert all(r.calib_gains is None for r in mwax_mc.rf_inputs)


def test_metafits_context_calibration_fits(cal_sol_mc: mwalib.MetafitsContext):
    assert cal_sol_mc.calibration_fits is not None
    assert cal_sol_mc.num_calibration_fits == len(cal_sol_mc.calibration_fits)
    assert cal_sol_mc.num_calibration_fits > 0

    for fit in cal_sol_mc.calibration_fits:
        assert fit.num_gains == len(fit.gains)
        assert fit.num_gain_polynomial_fit0 == len(fit.gain_polynomial_fit0)
        assert fit.num_gain_polynomial_fit1 == len(fit.gain_polynomial_fit1)


def test_metafits_context_calibration_in_rf_inputs(cal_sol_mc: mwalib.MetafitsContext):
    assert cal_sol_mc.rf_inputs[151].tile_name == "Tile104"
    assert cal_sol_mc.rf_inputs[151].calib_delay == pytest.approx(-135.49985, abs=1e-3)

    for rfinput in cal_sol_mc.rf_inputs:
        assert rfinput.calib_gains is not None
        assert len(rfinput.calib_gains) == cal_sol_mc.num_metafits_coarse_chans


def test_deripple_metafits_context():
    mc = mwalib.MetafitsContext(DERIPPLE_METAFITS, None)

    assert mc.mwa_version == mwalib.MWAVersion.CorrMWAXv2
    assert mc.deripple_applied
    assert mc.oversampled
    assert (mc.num_ants, mc.num_rf_inputs, mc.num_baselines) == (16, 32, 136)


def test_metafits_context_without_deripple(mwax_mc: mwalib.MetafitsContext):
    assert not mwax_mc.deripple_applied
    assert not mwax_mc.oversampled


#
# Phase 2 (safety): all classes are read-only
#


def test_metafits_context_classes_are_read_only():
    mc = mwalib.MetafitsContext(SIGNAL_CHAIN_LEGACY_CORRELATOR_METAFITS, None)
    assert mc.signal_chain_corrections is not None

    for obj in (
        mc,
        mc.antennas[0],
        mc.baselines[0],
        mc.rf_inputs[0],
        mc.metafits_coarse_chans[0],
        mc.metafits_timesteps[0],
        mc.signal_chain_corrections[0],
    ):
        assert_read_only(obj)


def test_metafits_context_voltage_beam_and_calibration_fit_are_read_only():
    vb_mc = mwalib.MetafitsContext(VOLTAGE_BEAM_METAFITS, None)
    cal_mc = mwalib.MetafitsContext(CAL_SOL_METAFITS, None)
    assert vb_mc.metafits_voltage_beams is not None
    assert cal_mc.calibration_fits is not None

    assert_read_only(vb_mc.metafits_voltage_beams[0])
    assert_read_only(cal_mc.calibration_fits[0])
