#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

#
# See README.md on how to build and run the tests.
#
import mwalib
import numpy as np
import pytest

MWAX_CORRELATOR_METAFITS = "../test_files/1244973688_1_timestep/1244973688.metafits"
MWAX_CORRELATOR_GPUBOX_FILES = [
    "../test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits",
    "../test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_001.fits",
]

LEGACY_CORRELATOR_METAFITS = "../test_files/1101503312_1_timestep/1101503312.metafits"
LEGACY_CORRELATOR_GPUBOX_FILES = [
    "../test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
]


@pytest.fixture
def mwax_cc() -> mwalib.CorrelatorContext:
    return mwalib.CorrelatorContext(
        MWAX_CORRELATOR_METAFITS, MWAX_CORRELATOR_GPUBOX_FILES
    )


@pytest.fixture
def legacy_cc() -> mwalib.CorrelatorContext:
    return mwalib.CorrelatorContext(
        LEGACY_CORRELATOR_METAFITS, LEGACY_CORRELATOR_GPUBOX_FILES
    )


def test_mwax_corr_context_mwa_version(mwax_cc: mwalib.CorrelatorContext):
    assert mwax_cc.mwa_version == mwalib.MWAVersion.CorrMWAXv2


def test_mwax_corr_context_read_visibilities(mwax_cc: mwalib.CorrelatorContext):
    timestep = 0
    coarse_chan = 10

    data_by_bl = mwax_cc.read_by_baseline(timestep, coarse_chan)

    assert np.sum(data_by_bl, dtype=np.float64) == 1389140766690.4983

    assert data_by_bl.shape == (
        mwax_cc.metafits_context.num_baselines,
        mwax_cc.metafits_context.num_corr_fine_chans_per_coarse,
        mwax_cc.metafits_context.num_visibility_pols * 2,
    )

    data_by_freq = mwax_cc.read_by_frequency(timestep, coarse_chan)

    assert data_by_freq.shape == (
        mwax_cc.metafits_context.num_corr_fine_chans_per_coarse,
        mwax_cc.metafits_context.num_baselines,
        mwax_cc.metafits_context.num_visibility_pols * 2,
    )

    # Check the sums are equial
    assert np.sum(data_by_bl, dtype=np.float64) == np.sum(
        data_by_freq, dtype=np.float64
    )


def test_mwax_corr_context_read_weights_by_baseline(mwax_cc: mwalib.CorrelatorContext):
    timestep = 0
    coarse_chan = 10

    data = mwax_cc.read_weights_by_baseline(timestep, coarse_chan)

    assert data.shape == (
        mwax_cc.metafits_context.num_baselines,
        mwax_cc.metafits_context.num_visibility_pols,
    )


def test_legacy_corr_context_get_fine_chan_freqs_hz_array(
    legacy_cc: mwalib.CorrelatorContext,
):
    chans = [10, 20]

    fine_chan_freqs = legacy_cc.get_fine_chan_freqs_hz_array(chans)

    assert len(fine_chan_freqs) == 256
    assert fine_chan_freqs[0] == 151680000.0
    assert fine_chan_freqs[128] == 164480000.0
