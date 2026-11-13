#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

#
# See README.md on how to build and run the tests.
#
import hashlib
import itertools
import os
from concurrent.futures import ThreadPoolExecutor

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
        mwax_cc.read_by_baseline("0", 10)  # ty: ignore[invalid-argument-type]

    with pytest.raises(TypeError):
        mwax_cc.read_by_baseline(0.5, 10)  # ty: ignore[invalid-argument-type]


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


#
# Regression tests for reading visibilities and weights.
#
# These are here to make it safe to change how data is read and converted. They check every element of the data
# (rather than just a sum), that reads can be done from many threads at once, and the layout of what is returned.
#

# The timestep and coarse channel which have data in each set of test files
READ_INDICES = {"legacy": (0, 0), "mwax": (0, 10)}

# Golden values: the SHA-256 of the data read from the test files, as read by the current implementation.
# If one of these changes, the data we return has changed (e.g. the order of the data).
GOLDEN_SHA256 = {
    ("legacy", "read_by_baseline"): "6c1b1f7cfad0498e9cdcbac1fa1128c8ef11361aa552c7b0f76df756bf99eeda",
    ("legacy", "read_by_frequency"): "ff0de1c63f489fb9505e094a8d8e8b9217d710b6ceb1eadec7167e0b4054e775",
    ("legacy", "read_weights_by_baseline"): "f9173bb3de6158229c3aece6855aa66f30a7120bc24afbf630fecd8440da2b15",
    ("mwax", "read_by_baseline"): "701935e299a53ca42f51259c9899eb2f37cb2b1e56393d48120031e97bb8c328",
    ("mwax", "read_by_frequency"): "ae0620dfb966b23a7729bab9ec005e5be450fcdc1ddcfda3e8ea3c1df190c245",
    ("mwax", "read_weights_by_baseline"): "f9173bb3de6158229c3aece6855aa66f30a7120bc24afbf630fecd8440da2b15",
}

READ_METHODS = ["read_by_baseline", "read_by_frequency", "read_weights_by_baseline"]


def sha256_of(data: np.ndarray) -> str:
    # Adding zero turns -0.0 into 0.0, so the hash does not depend on the sign of a zero
    return hashlib.sha256((data + np.float32(0.0)).tobytes()).hexdigest()


@pytest.mark.parametrize("read_method", READ_METHODS)
@pytest.mark.parametrize("kind", ["legacy", "mwax"])
def test_corr_context_read_golden(kind: str, read_method: str, request):
    cc: mwalib.CorrelatorContext = request.getfixturevalue(f"{kind}_cc")
    timestep, coarse_chan = READ_INDICES[kind]

    data = getattr(cc, read_method)(timestep, coarse_chan)

    assert data.dtype == np.float32
    assert sha256_of(data) == GOLDEN_SHA256[(kind, read_method)]


@pytest.mark.parametrize("kind", ["legacy", "mwax"])
def test_corr_context_read_layout(kind: str, request):
    cc: mwalib.CorrelatorContext = request.getfixturevalue(f"{kind}_cc")
    timestep, coarse_chan = READ_INDICES[kind]
    mc = cc.metafits_context

    shapes = {
        "read_by_baseline": (mc.num_baselines, mc.num_corr_fine_chans_per_coarse, mc.num_visibility_pols * 2),
        "read_by_frequency": (mc.num_corr_fine_chans_per_coarse, mc.num_baselines, mc.num_visibility_pols * 2),
        "read_weights_by_baseline": (mc.num_baselines, mc.num_visibility_pols),
    }

    for read_method, shape in shapes.items():
        data = getattr(cc, read_method)(timestep, coarse_chan)

        assert data.shape == shape
        assert data.dtype == np.float32
        assert data.flags.c_contiguous
        assert data.flags.writeable


@pytest.mark.parametrize("read_method", READ_METHODS)
@pytest.mark.parametrize("kind", ["legacy", "mwax"])
def test_corr_context_read_returns_independent_arrays(kind: str, read_method: str, request):
    cc: mwalib.CorrelatorContext = request.getfixturevalue(f"{kind}_cc")
    timestep, coarse_chan = READ_INDICES[kind]

    first = getattr(cc, read_method)(timestep, coarse_chan)
    first[...] = np.float32(-1.0)

    # Changing what we were given does not change what we get next time
    second = getattr(cc, read_method)(timestep, coarse_chan)

    assert sha256_of(second) == GOLDEN_SHA256[(kind, read_method)]


@pytest.mark.parametrize("kind", ["legacy", "mwax"])
def test_corr_context_read_from_many_threads(kind: str, request):
    cc: mwalib.CorrelatorContext = request.getfixturevalue(f"{kind}_cc")
    timestep, coarse_chan = READ_INDICES[kind]

    expected = {m: getattr(cc, m)(timestep, coarse_chan) for m in READ_METHODS}

    def read(read_method: str) -> np.ndarray:
        return getattr(cc, read_method)(timestep, coarse_chan)

    # Many reads at once, of each of the read methods
    methods = READ_METHODS * 8
    with ThreadPoolExecutor(max_workers=4) as executor:
        results = list(executor.map(read, methods))

    for read_method, data in zip(methods, results):
        assert np.array_equal(data, expected[read_method])


def test_corr_context_read_last_valid_indices_have_no_data(mwax_cc: mwalib.CorrelatorContext):
    # The last valid indices are valid, but there is no data for them
    with pytest.raises(mwalib.GpuboxErrorNoDataForTimeStepCoarseChannel):
        mwax_cc.read_by_baseline(mwax_cc.num_timesteps - 1, 10)

    with pytest.raises(mwalib.GpuboxErrorNoDataForTimeStepCoarseChannel):
        mwax_cc.read_by_baseline(0, mwax_cc.num_coarse_chans - 1)

    # One more than the last is not valid
    with pytest.raises(mwalib.GpuboxErrorInvalidTimeStepIndex):
        mwax_cc.read_by_baseline(mwax_cc.num_timesteps, 10)

    with pytest.raises(mwalib.GpuboxErrorInvalidCoarseChanIndex):
        mwax_cc.read_by_baseline(0, mwax_cc.num_coarse_chans)


def test_corr_context_fine_chan_freqs_every_coarse_chan(any_cc: mwalib.CorrelatorContext):
    mc = any_cc.metafits_context
    width = mc.corr_fine_chan_width_hz

    for index, coarse_chan in enumerate(any_cc.coarse_chans):
        freqs = any_cc.get_fine_chan_freqs_hz_array([index])

        assert len(freqs) == mc.num_corr_fine_chans_per_coarse
        assert all(upper - lower == width for lower, upper in itertools.pairwise(freqs))
        assert coarse_chan.chan_start_hz <= freqs[0]
        assert freqs[-1] <= coarse_chan.chan_end_hz

    # Any order of channels is the same as asking for the channels one at a time
    indices = list(reversed(range(any_cc.num_coarse_chans)))
    one_at_a_time = [f for i in indices for f in any_cc.get_fine_chan_freqs_hz_array([i])]

    assert any_cc.get_fine_chan_freqs_hz_array(indices) == one_at_a_time


#
# Tests for the read methods that read into an existing array (`*_into_buffer`)
#


def read_shapes(cc: mwalib.CorrelatorContext) -> dict[str, tuple[int, ...]]:
    """Get the shape of the result of each read method.

    Args:
        cc: The correlator context.

    Returns:
        The shape of the result of each read method, by method name.
    """
    mc = cc.metafits_context
    return {
        "read_by_baseline": (mc.num_baselines, mc.num_corr_fine_chans_per_coarse, mc.num_visibility_pols * 2),
        "read_by_frequency": (mc.num_corr_fine_chans_per_coarse, mc.num_baselines, mc.num_visibility_pols * 2),
        "read_weights_by_baseline": (mc.num_baselines, mc.num_visibility_pols),
    }


# Suffix of the read methods that read into an existing array
INTO_BUFFER = "_into_buffer"

# Value used to fill buffers, so we can see whether the read overwrote them
BUFFER_SENTINEL = np.float32(-1234.5)


@pytest.mark.parametrize("read_method", READ_METHODS)
@pytest.mark.parametrize("kind", ["legacy", "mwax"])
def test_corr_context_read_into_buffer(kind: str, read_method: str, request):
    cc: mwalib.CorrelatorContext = request.getfixturevalue(f"{kind}_cc")
    timestep, coarse_chan = READ_INDICES[kind]
    buffer = np.full(read_shapes(cc)[read_method], BUFFER_SENTINEL, dtype=np.float32)

    # Twice, so the second read overwrites the data of the first one
    for _ in range(2):
        result = getattr(cc, read_method + INTO_BUFFER)(timestep, coarse_chan, buffer)

        # Like Rust and C, nothing is returned; the data is in the buffer
        assert result is None
        assert sha256_of(buffer) == GOLDEN_SHA256[(kind, read_method)]


@pytest.mark.parametrize("read_method", READ_METHODS)
def test_corr_context_read_into_buffer_as_keyword_argument(mwax_cc: mwalib.CorrelatorContext, read_method: str):
    timestep, coarse_chan = READ_INDICES["mwax"]
    buffer = np.zeros(read_shapes(mwax_cc)[read_method], dtype=np.float32)

    getattr(mwax_cc, read_method + INTO_BUFFER)(timestep, coarse_chan, buffer=buffer)

    assert sha256_of(buffer) == GOLDEN_SHA256[("mwax", read_method)]


@pytest.mark.parametrize("read_method", READ_METHODS)
def test_corr_context_read_into_buffer_with_wrong_shape(mwax_cc: mwalib.CorrelatorContext, read_method: str):
    timestep, coarse_chan = READ_INDICES["mwax"]
    shape = read_shapes(mwax_cc)[read_method]

    # Bigger, smaller and the same size in a different shape
    wrong_shapes = [
        (shape[0] + 1, *shape[1:]),
        (shape[0] - 1, *shape[1:]),
        (shape[1], shape[0], *shape[2:]),
    ]

    for wrong_shape in wrong_shapes:
        buffer = np.full(wrong_shape, BUFFER_SENTINEL, dtype=np.float32)

        with pytest.raises(ValueError, match="shape"):
            getattr(mwax_cc, read_method + INTO_BUFFER)(timestep, coarse_chan, buffer)

        # Nothing was written
        assert np.all(buffer == BUFFER_SENTINEL)


@pytest.mark.parametrize("read_method", READ_METHODS)
def test_corr_context_read_into_buffer_with_wrong_number_of_dimensions(
    mwax_cc: mwalib.CorrelatorContext, read_method: str
):
    timestep, coarse_chan = READ_INDICES["mwax"]
    buffer = np.zeros(int(np.prod(read_shapes(mwax_cc)[read_method])), dtype=np.float32)

    with pytest.raises(TypeError):
        getattr(mwax_cc, read_method + INTO_BUFFER)(timestep, coarse_chan, buffer)


@pytest.mark.parametrize("read_method", READ_METHODS)
def test_corr_context_read_into_buffer_with_wrong_dtype(mwax_cc: mwalib.CorrelatorContext, read_method: str):
    timestep, coarse_chan = READ_INDICES["mwax"]
    buffer = np.zeros(read_shapes(mwax_cc)[read_method], dtype=np.float64)

    with pytest.raises(TypeError):
        getattr(mwax_cc, read_method + INTO_BUFFER)(timestep, coarse_chan, buffer)


@pytest.mark.parametrize("read_method", READ_METHODS)
def test_corr_context_read_into_buffer_that_is_not_c_contiguous(mwax_cc: mwalib.CorrelatorContext, read_method: str):
    timestep, coarse_chan = READ_INDICES["mwax"]
    shape = read_shapes(mwax_cc)[read_method]

    # Fortran order is contiguous, but in the wrong order; a strided view is not contiguous at all
    fortran = np.full(shape, BUFFER_SENTINEL, dtype=np.float32, order="F")
    strided = np.full((shape[0] * 2, *shape[1:]), BUFFER_SENTINEL, dtype=np.float32)[::2]
    assert strided.shape == shape

    for buffer in [fortran, strided]:
        with pytest.raises(ValueError, match="C contiguous"):
            getattr(mwax_cc, read_method + INTO_BUFFER)(timestep, coarse_chan, buffer)

        assert np.all(buffer == BUFFER_SENTINEL)


@pytest.mark.parametrize("read_method", READ_METHODS)
def test_corr_context_read_into_buffer_that_is_read_only(mwax_cc: mwalib.CorrelatorContext, read_method: str):
    timestep, coarse_chan = READ_INDICES["mwax"]
    buffer = np.full(read_shapes(mwax_cc)[read_method], BUFFER_SENTINEL, dtype=np.float32)
    buffer.flags.writeable = False

    with pytest.raises(ValueError, match="written"):
        getattr(mwax_cc, read_method + INTO_BUFFER)(timestep, coarse_chan, buffer)

    assert np.all(buffer == BUFFER_SENTINEL)


def test_corr_context_read_into_buffer_with_invalid_timestep(mwax_cc: mwalib.CorrelatorContext):
    _, coarse_chan = READ_INDICES["mwax"]
    buffer = np.full(read_shapes(mwax_cc)["read_by_baseline"], BUFFER_SENTINEL, dtype=np.float32)

    with pytest.raises(mwalib.GpuboxErrorInvalidTimeStepIndex):
        mwax_cc.read_by_baseline_into_buffer(len(mwax_cc.timesteps), coarse_chan, buffer)

    assert np.all(buffer == BUFFER_SENTINEL)


@pytest.mark.parametrize("kind", ["legacy", "mwax"])
def test_corr_context_read_into_buffer_from_many_threads(kind: str, request):
    cc: mwalib.CorrelatorContext = request.getfixturevalue(f"{kind}_cc")
    timestep, coarse_chan = READ_INDICES[kind]
    shapes = read_shapes(cc)

    def read(read_method: str) -> np.ndarray:
        buffer = np.zeros(shapes[read_method], dtype=np.float32)
        getattr(cc, read_method + INTO_BUFFER)(timestep, coarse_chan, buffer)
        return buffer

    # Many reads at once, of each of the read methods, each into its own array
    methods = READ_METHODS * 8
    with ThreadPoolExecutor(max_workers=4) as executor:
        results = list(executor.map(read, methods))

    for read_method, data in zip(methods, results, strict=True):
        assert sha256_of(data) == GOLDEN_SHA256[(kind, read_method)]


#
# Phase 2 (safety): errors instead of panics
#


def test_corr_context_fine_chan_freqs_invalid_index(any_cc: mwalib.CorrelatorContext):
    num_coarse_chans = len(any_cc.coarse_chans)

    for indices in ([num_coarse_chans], [0, num_coarse_chans + 10]):
        with pytest.raises(mwalib.GpuboxErrorInvalidCoarseChanIndex):
            any_cc.get_fine_chan_freqs_hz_array(indices)
