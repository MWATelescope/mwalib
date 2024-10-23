#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

#
# See README.md on how to build and run the tests.
#
import datetime
import mwalib
import pytest
import math

MWAX_CORRELATOR_METAFITS = "test_files/1244973688_1_timestep/1244973688.metafits"
SIGNAL_CHAIN_LEGACY_CORRELATOR_METAFITS = "test_files/metafits_signal_chain_corr/1096952256_metafits.fits"
LEGACY_CORRELATOR_METAFITS = "test_files/1101503312_1_timestep/1101503312.metafits"


@pytest.fixture
def mwax_mc() -> mwalib.MetafitsContext:
    return mwalib.MetafitsContext(MWAX_CORRELATOR_METAFITS, mwalib.MWAVersion.CorrMWAXv2)


@pytest.fixture
def mwax_sc_mc() -> mwalib.MetafitsContext:
    return mwalib.MetafitsContext(SIGNAL_CHAIN_LEGACY_CORRELATOR_METAFITS, None)


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
