#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

#
# See README.md on how to build and run the tests.
#
import itertools
import os

import numpy as np
import pytest

import mwalib

MWAX_CORRELATOR_METAFITS = "test_files/1244973688_1_timestep/1244973688.metafits"
MWAX_CORRELATOR_GPUBOX_FILES = [
    "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits",
    "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_001.fits",
]

LEGACY_CORRELATOR_METAFITS = "test_files/1101503312_1_timestep/1101503312.metafits"
LEGACY_CORRELATOR_GPUBOX_FILES = [
    "test_files/1101503312_1_timestep/1101503312_20141201210818_gpubox01_00.fits",
]


@pytest.fixture
def mwax_cc() -> mwalib.CorrelatorContext:
    return mwalib.CorrelatorContext(MWAX_CORRELATOR_METAFITS, MWAX_CORRELATOR_GPUBOX_FILES)


@pytest.fixture
def legacy_cc() -> mwalib.CorrelatorContext:
    return mwalib.CorrelatorContext(LEGACY_CORRELATOR_METAFITS, LEGACY_CORRELATOR_GPUBOX_FILES)


def test_legacy_corr_context_attributes(legacy_cc: mwalib.CorrelatorContext):
    assert legacy_cc.mwa_version == mwalib.MWAVersion.CorrLegacy
    assert legacy_cc.bscale == 0.5


def test_mwax_corr_context_attributes(mwax_cc: mwalib.CorrelatorContext):
    assert mwax_cc.mwa_version == mwalib.MWAVersion.CorrMWAXv2
    assert mwax_cc.bscale == 1.0


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
    assert np.sum(data_by_bl, dtype=np.float64) == np.sum(data_by_freq, dtype=np.float64)


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


@pytest.fixture(params=["legacy_cc", "mwax_cc"])
def any_cc(request) -> mwalib.CorrelatorContext:
    return request.getfixturevalue(request.param)


def test_corr_context_is_a_context_manager(any_cc: mwalib.CorrelatorContext):
    with any_cc as cc:
        assert cc is any_cc


def test_corr_context_indices_are_consistent(any_cc: mwalib.CorrelatorContext):
    assert len(any_cc.common_timestep_indices) == any_cc.num_common_timesteps
    assert len(any_cc.common_coarse_chan_indices) == any_cc.num_common_coarse_chans
    assert len(any_cc.provided_timestep_indices) == any_cc.num_provided_timesteps
    assert len(any_cc.provided_coarse_chan_indices) == any_cc.num_provided_coarse_chans

    # The common indices are always provided, and all indices are valid
    assert set(any_cc.common_timestep_indices) <= set(any_cc.provided_timestep_indices)
    assert set(any_cc.common_coarse_chan_indices) <= set(any_cc.provided_coarse_chan_indices)
    assert all(0 <= i < any_cc.num_timesteps for i in any_cc.provided_timestep_indices)
    assert all(0 <= i < any_cc.num_coarse_chans for i in any_cc.provided_coarse_chan_indices)

    # There must be some data to work with
    assert any_cc.num_common_timesteps > 0
    assert any_cc.num_common_coarse_chans > 0


def test_corr_context_gpubox_batches(any_cc: mwalib.CorrelatorContext, request):
    assert len(any_cc.gpubox_batches) > 0

    supplied = (
        MWAX_CORRELATOR_GPUBOX_FILES
        if request.node.callspec.params["any_cc"] == "mwax_cc"
        else LEGACY_CORRELATOR_GPUBOX_FILES
    )

    batch_filenames = [f.filename for batch in any_cc.gpubox_batches for f in batch.gpubox_files]

    # Every file we supplied is in exactly one batch
    assert sorted(os.path.basename(f) for f in batch_filenames) == sorted(os.path.basename(f) for f in supplied)


def test_mwax_corr_context_provided_coarse_chan_matches_filename(mwax_cc: mwalib.CorrelatorContext):
    # The supplied gpubox files are all for receiver channel 114 ("ch114" in the filename)
    assert mwax_cc.num_provided_coarse_chans == 1

    coarse_chan = mwax_cc.metafits_context.metafits_coarse_chans[mwax_cc.provided_coarse_chan_indices[0]]
    assert coarse_chan.rec_chan_number == 114


def test_corr_context_read_by_baseline_and_frequency_are_transposed(any_cc: mwalib.CorrelatorContext):
    timestep = any_cc.common_timestep_indices[0]
    coarse_chan = any_cc.common_coarse_chan_indices[0]
    mc = any_cc.metafits_context

    by_baseline = any_cc.read_by_baseline(timestep, coarse_chan)
    by_frequency = any_cc.read_by_frequency(timestep, coarse_chan)

    assert by_baseline.dtype == np.float32
    assert by_frequency.dtype == np.float32
    assert by_baseline.shape == (mc.num_baselines, mc.num_corr_fine_chans_per_coarse, mc.num_visibility_pols * 2)

    # Same data, just a different order: [baseline][fine_chan][pol] vs [fine_chan][baseline][pol]
    assert np.array_equal(np.swapaxes(by_baseline, 0, 1), by_frequency)


def test_corr_context_read_weights(any_cc: mwalib.CorrelatorContext):
    timestep = any_cc.common_timestep_indices[0]
    coarse_chan = any_cc.common_coarse_chan_indices[0]
    mc = any_cc.metafits_context

    weights = any_cc.read_weights_by_baseline(timestep, coarse_chan)

    assert weights.dtype == np.float32
    assert weights.shape == (mc.num_baselines, mc.num_visibility_pols)
    assert np.all(np.isfinite(weights))


def test_corr_context_read_is_repeatable(any_cc: mwalib.CorrelatorContext):
    timestep = any_cc.common_timestep_indices[0]
    coarse_chan = any_cc.common_coarse_chan_indices[0]

    first = any_cc.read_by_baseline(timestep, coarse_chan)
    second = any_cc.read_by_baseline(timestep, coarse_chan)

    assert np.array_equal(first, second)


def test_corr_context_read_invalid_timestep(mwax_cc: mwalib.CorrelatorContext):
    with pytest.raises(mwalib.GpuboxErrorInvalidTimeStepIndex):
        mwax_cc.read_by_baseline(mwax_cc.num_timesteps, 10)

    with pytest.raises(mwalib.GpuboxErrorInvalidTimeStepIndex):
        mwax_cc.read_by_frequency(mwax_cc.num_timesteps, 10)

    with pytest.raises(mwalib.GpuboxErrorInvalidTimeStepIndex):
        mwax_cc.read_weights_by_baseline(mwax_cc.num_timesteps, 10)


def test_corr_context_read_invalid_coarse_chan(mwax_cc: mwalib.CorrelatorContext):
    with pytest.raises(mwalib.GpuboxErrorInvalidCoarseChanIndex):
        mwax_cc.read_by_baseline(0, mwax_cc.num_coarse_chans)

    with pytest.raises(mwalib.GpuboxErrorInvalidCoarseChanIndex):
        mwax_cc.read_by_frequency(0, mwax_cc.num_coarse_chans)

    with pytest.raises(mwalib.GpuboxErrorInvalidCoarseChanIndex):
        mwax_cc.read_weights_by_baseline(0, mwax_cc.num_coarse_chans)


def test_corr_context_read_no_data_for_coarse_chan(mwax_cc: mwalib.CorrelatorContext):
    # Valid indices, but we did not supply a gpubox file for coarse channel 0
    assert 0 not in mwax_cc.provided_coarse_chan_indices

    with pytest.raises(mwalib.GpuboxErrorNoDataForTimeStepCoarseChannel):
        mwax_cc.read_by_baseline(0, 0)


def test_corr_context_read_bad_argument_types(mwax_cc: mwalib.CorrelatorContext):
    with pytest.raises(OverflowError):
        mwax_cc.read_by_baseline(-1, 10)

    with pytest.raises(TypeError):
        mwax_cc.read_by_baseline("0", 10)  # type: ignore[arg-type]

    with pytest.raises(TypeError):
        mwax_cc.read_by_baseline(0.5, 10)  # type: ignore[arg-type]


@pytest.mark.parametrize("coarse_chans", [[0], [10, 11], [0, 5, 23]])
def test_corr_context_fine_chan_freqs(mwax_cc: mwalib.CorrelatorContext, coarse_chans: list[int]):
    mc = mwax_cc.metafits_context

    freqs = mwax_cc.get_fine_chan_freqs_hz_array(coarse_chans)

    assert len(freqs) == len(coarse_chans) * mc.num_corr_fine_chans_per_coarse

    # Within a coarse channel the fine channels are evenly spaced
    per_coarse = mc.num_corr_fine_chans_per_coarse
    for i in range(len(coarse_chans)):
        chan_freqs = freqs[i * per_coarse : (i + 1) * per_coarse]
        assert all(upper - lower == mc.corr_fine_chan_width_hz for lower, upper in itertools.pairwise(chan_freqs))

        # and they sit inside the coarse channel
        coarse = mc.metafits_coarse_chans[coarse_chans[i]]
        assert coarse.chan_start_hz <= chan_freqs[0] <= chan_freqs[-1] <= coarse.chan_end_hz


def test_corr_context_fine_chan_freqs_empty(mwax_cc: mwalib.CorrelatorContext):
    assert mwax_cc.get_fine_chan_freqs_hz_array([]) == []
