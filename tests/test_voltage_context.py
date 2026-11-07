#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

#
# See README.md on how to build and run the tests.
#
import hashlib
import itertools
from concurrent.futures import ThreadPoolExecutor

import numpy as np
import pytest

import mwalib

MWAX_VCS_METAFITS = "test_files/1101503312_mwax_vcs/1101503312.metafits"
MWAX_VCS_VOLTAGE_FILES = [
    "test_files/1101503312_mwax_vcs/1101503312_1101503312_123.sub",
    "test_files/1101503312_mwax_vcs/1101503312_1101503312_124.sub",
    "test_files/1101503312_mwax_vcs/1101503312_1101503320_123.sub",
    "test_files/1101503312_mwax_vcs/1101503312_1101503320_124.sub",
]


LEGACY_VCS_METAFITS = "test_files/1101503312_vcs/1101503312.metafits"
LEGACY_VCS_VOLTAGE_FILES = [
    "test_files/1101503312_vcs/1101503312_1101503312_ch123.dat",
    "test_files/1101503312_vcs/1101503312_1101503312_ch124.dat",
    "test_files/1101503312_vcs/1101503312_1101503313_ch123.dat",
    "test_files/1101503312_vcs/1101503312_1101503313_ch124.dat",
]


@pytest.fixture
def mwax_vc() -> mwalib.VoltageContext:
    return mwalib.VoltageContext(MWAX_VCS_METAFITS, MWAX_VCS_VOLTAGE_FILES)


@pytest.fixture
def legacy_vc() -> mwalib.VoltageContext:
    return mwalib.VoltageContext(LEGACY_VCS_METAFITS, LEGACY_VCS_VOLTAGE_FILES)


def test_mwax_voltage_context(mwax_vc: mwalib.VoltageContext):
    assert mwax_vc.mwa_version == mwalib.MWAVersion.VCSMWAXv2

    # Number of bytes in each sample
    assert mwax_vc.sample_size_bytes == 2
    # Number of voltage blocks per timestep
    assert mwax_vc.num_voltage_blocks_per_timestep == 160
    # Number of voltage blocks of samples in each second of data
    assert mwax_vc.num_voltage_blocks_per_second == 20
    # Number of samples in each voltage_blocks for each second of data per rf_input * fine_chans * real|imag
    assert mwax_vc.num_samples_per_voltage_block == 64_000
    # The size of each voltage block
    assert mwax_vc.voltage_block_size_bytes == 256_000
    # Number of bytes used to store delays - for MWAX this is the same as a voltage block size, for legacy it is 0
    assert mwax_vc.delay_block_size_bytes == mwax_vc.voltage_block_size_bytes

    # The amount of bytes to skip before getting into real data within the voltage files
    assert mwax_vc.data_file_header_size_bytes == 4096
    # Expected voltage file size
    assert mwax_vc.expected_voltage_data_file_size_bytes == 41_220_096
    # Check number of batches
    assert len(mwax_vc.voltage_batches) == 2


def test_legacy_voltage_context(legacy_vc: mwalib.VoltageContext):
    assert legacy_vc.mwa_version == mwalib.MWAVersion.VCSLegacyRecombined

    # Number of bytes in each sample
    assert legacy_vc.sample_size_bytes == 1
    # Number of voltage blocks per timestep
    assert legacy_vc.num_voltage_blocks_per_timestep == 1
    # Number of voltage blocks of samples in each second of data
    assert legacy_vc.num_voltage_blocks_per_second == 1
    # Number of samples in each voltage_blocks for each second of data per rf_input * fine_chans * real|imag
    assert legacy_vc.num_samples_per_voltage_block == 10000
    # The size of each voltage block
    assert legacy_vc.voltage_block_size_bytes == 2560000
    # Number of bytes used to store delays - for MWAX this is the same as a voltage block size, for legacy it is 0
    assert legacy_vc.delay_block_size_bytes == 0
    # The amount of bytes to skip before getting into real data within the voltage files
    assert legacy_vc.data_file_header_size_bytes == 0
    # Expected voltage file size
    assert legacy_vc.expected_voltage_data_file_size_bytes == 2_560_000
    # Check batches
    assert len(legacy_vc.voltage_batches) == 2


def test_mwax_vcs_context_read_data(mwax_vc: mwalib.VoltageContext):
    timestep = 0
    coarse_chan = 14

    data_by_file = mwax_vc.read_file(timestep, coarse_chan)

    assert data_by_file.shape == (
        8,
        mwax_vc.num_voltage_blocks_per_second,
        mwax_vc.metafits_context.num_ants,
        mwax_vc.metafits_context.num_ant_pols,
        mwax_vc.num_samples_per_voltage_block,
        mwax_vc.sample_size_bytes,
    )

    gps_start = 1101503312
    gps_seconds = 8

    data_by_gpsecond = mwax_vc.read_second(gps_start, gps_seconds, coarse_chan)

    assert data_by_gpsecond.shape == (
        gps_seconds,
        mwax_vc.num_voltage_blocks_per_second,
        mwax_vc.metafits_context.num_ants,
        mwax_vc.metafits_context.num_ant_pols,
        mwax_vc.num_samples_per_voltage_block,
        mwax_vc.sample_size_bytes,
    )

    # Check the sums are equial
    assert np.sum(data_by_file, dtype=np.float64) == np.sum(data_by_gpsecond, dtype=np.float64)

    # Check data detail
    # second: 0, block: 0, ant: 0, pol: 0, sample: 0, value: 0
    assert data_by_gpsecond[0, 0, 0, 0, 0, 0] == 0

    # second: 0, block: 0, ant: 0, pol: 0, sample: 1, value: 1
    assert data_by_gpsecond[0, 0, 0, 0, 1, 1] == -3

    # second: 0, block: 0, ant: 0, pol: 0, sample: 255, value: 0
    assert data_by_gpsecond[0, 0, 0, 0, 255, 0] == -2

    # second: 0, block: 0, ant: 0, pol: 0, sample: 256, value: 1
    assert data_by_gpsecond[0, 0, 0, 0, 256, 1] == -1

    # second: 0, block: 1, ant: 0, pol: 0, sample: 2, value: 0
    assert data_by_gpsecond[0, 1, 0, 0, 2, 0] == 9

    # second: 0, block: 159, ant: 0, pol: 1, sample: 63999, value: 1
    # second: 7, block: 19, ant: 0, pol: 1, sample: 63999, value: 1
    assert data_by_gpsecond[7, 19, 0, 1, 63999, 1] == -30

    # second: 0, block: 120, ant: 0, pol: 0, sample: 0, value: 0
    # second: 6, block: 0, ant: 0, pol: 0, sample: 0, value: 0
    assert data_by_gpsecond[6, 0, 0, 0, 0, 0] == 88


def test_legacy_vcs_context_read_data(legacy_vc: mwalib.VoltageContext):
    timestep = 0
    coarse_chan = 14

    data_by_file = legacy_vc.read_file(timestep, coarse_chan)

    assert data_by_file.shape == (
        1,
        legacy_vc.num_samples_per_voltage_block,
        legacy_vc.num_fine_chans_per_coarse,
        legacy_vc.metafits_context.num_ants,
        legacy_vc.metafits_context.num_ant_pols,
        legacy_vc.sample_size_bytes,
    )

    gps_start = 1101503312
    gps_seconds = 1

    data_by_gpsecond = legacy_vc.read_second(gps_start, gps_seconds, coarse_chan)

    assert data_by_gpsecond.shape == (
        gps_seconds,
        legacy_vc.num_samples_per_voltage_block,
        legacy_vc.num_fine_chans_per_coarse,
        legacy_vc.metafits_context.num_ants,
        legacy_vc.metafits_context.num_ant_pols,
        legacy_vc.sample_size_bytes,
    )

    # Check the sums are equial
    assert np.sum(data_by_file, dtype=np.float64) == np.sum(data_by_gpsecond, dtype=np.float64)

    # Check detailed data
    # second: 0, sample: 0, fine_chan: 0, ant: 0, pol: 0, sample
    assert data_by_gpsecond[0, 0, 0, 0, 0, 0] == 0

    # second: 0, sample: 0, fine_chan: 0, ant: 0, pol: 1, sample
    assert data_by_gpsecond[0, 0, 0, 0, 1, 0] == 2

    # second: 0, sample: 0, fine_chan: 1, ant: 0, pol: 1, sample
    assert data_by_gpsecond[0, 0, 1, 0, 1, 0] == 5

    # second: 0, sample: 0, fine_chan: 127, ant: 0, pol: 0, sample
    assert data_by_gpsecond[0, 0, 127, 0, 0, 0] == 125

    # second: 0, sample: 10, fine_chan: 32, ant: 0, pol: 1, sample
    assert data_by_gpsecond[0, 10, 32, 0, 1, 0] == -118

    # second: 0, sample: 9999, fine_chan: 127, ant: 0, pol: 1, sample
    assert data_by_gpsecond[0, 9999, 127, 0, 1, 0] == -69


@pytest.fixture(params=["legacy_vc", "mwax_vc"])
def any_vc(request) -> mwalib.VoltageContext:
    return request.getfixturevalue(request.param)


def test_voltage_context_is_a_context_manager(any_vc: mwalib.VoltageContext):
    with any_vc as vc:
        assert vc is any_vc


def test_voltage_context_indices_are_consistent(any_vc: mwalib.VoltageContext):
    assert len(any_vc.common_timestep_indices) == any_vc.num_common_timesteps
    assert len(any_vc.common_coarse_chan_indices) == any_vc.num_common_coarse_chans
    assert len(any_vc.provided_timestep_indices) == any_vc.num_provided_timesteps
    assert len(any_vc.provided_coarse_chan_indices) == any_vc.num_provided_coarse_chans

    # The common indices are always provided, and all indices are valid
    assert set(any_vc.common_timestep_indices) <= set(any_vc.provided_timestep_indices)
    assert set(any_vc.common_coarse_chan_indices) <= set(any_vc.provided_coarse_chan_indices)
    assert all(0 <= i < any_vc.num_timesteps for i in any_vc.provided_timestep_indices)
    assert all(0 <= i < any_vc.num_coarse_chans for i in any_vc.provided_coarse_chan_indices)

    assert any_vc.num_common_timesteps > 0
    assert any_vc.num_common_coarse_chans > 0


def test_voltage_context_read_file_dtype_and_shape(any_vc: mwalib.VoltageContext):
    timestep = any_vc.common_timestep_indices[0]
    coarse_chan = any_vc.common_coarse_chan_indices[0]
    mc = any_vc.metafits_context

    data = any_vc.read_file(timestep, coarse_chan)

    # Signed bytes. The shape is not the same between legacy and mwax, but the antenna and pol axes are
    assert data.dtype == np.int8
    assert data.ndim == 6 or data.ndim == 5
    assert mc.num_ants in data.shape
    assert mc.num_ant_pols in data.shape


def test_mwax_vcs_context_read_file_matches_read_second_for_each_coarse_chan(mwax_vc: mwalib.VoltageContext):
    gps_start = 1101503312
    gps_seconds = 8

    for coarse_chan in mwax_vc.provided_coarse_chan_indices:
        by_file = mwax_vc.read_file(0, coarse_chan)
        by_second = mwax_vc.read_second(gps_start, gps_seconds, coarse_chan)

        assert np.array_equal(by_file, by_second)


def test_mwax_vcs_context_read_second_part_of_a_file(mwax_vc: mwalib.VoltageContext):
    gps_start = 1101503312
    coarse_chan = mwax_vc.provided_coarse_chan_indices[0]

    whole_file = mwax_vc.read_file(0, coarse_chan)

    # Reading seconds 2 and 3 only is the same as slicing those seconds out of the whole file
    two_seconds = mwax_vc.read_second(gps_start + 2, 2, coarse_chan)

    assert two_seconds.shape[0] == 2
    assert np.array_equal(two_seconds, whole_file[2:4])


def test_voltage_context_read_file_invalid_timestep(any_vc: mwalib.VoltageContext):
    with pytest.raises(mwalib.VoltageErrorInvalidTimeStepIndex):
        any_vc.read_file(any_vc.num_timesteps, 0)


def test_voltage_context_read_file_invalid_coarse_chan(any_vc: mwalib.VoltageContext):
    with pytest.raises(mwalib.VoltageErrorInvalidCoarseChanIndex):
        any_vc.read_file(0, any_vc.num_coarse_chans)


def test_voltage_context_read_second_invalid_start(mwax_vc: mwalib.VoltageContext):
    coarse_chan = mwax_vc.provided_coarse_chan_indices[0]

    # The start must be inside the observation
    with pytest.raises(mwalib.VoltageErrorInvalidGpsSecondStart):
        mwax_vc.read_second(0, 1, coarse_chan)


def test_voltage_context_read_second_invalid_coarse_chan(mwax_vc: mwalib.VoltageContext):
    with pytest.raises(mwalib.VoltageErrorInvalidCoarseChanIndex):
        mwax_vc.read_second(1101503312, 1, mwax_vc.num_coarse_chans)


def test_voltage_context_read_bad_argument_types(mwax_vc: mwalib.VoltageContext):
    with pytest.raises(OverflowError):
        mwax_vc.read_file(-1, 0)

    with pytest.raises(TypeError):
        mwax_vc.read_file("0", 0)  # ty: ignore[invalid-argument-type]


def test_voltage_context_fine_chan_freqs(any_vc: mwalib.VoltageContext):
    mc = any_vc.metafits_context
    coarse_chans = any_vc.provided_coarse_chan_indices

    freqs = any_vc.get_fine_chan_freqs_hz_array(coarse_chans)

    assert len(freqs) == len(coarse_chans) * mc.num_volt_fine_chans_per_coarse

    # Within a coarse channel the fine channels are evenly spaced
    per_coarse = mc.num_volt_fine_chans_per_coarse
    for i in range(len(coarse_chans)):
        chan_freqs = freqs[i * per_coarse : (i + 1) * per_coarse]
        assert all(upper - lower == mc.volt_fine_chan_width_hz for lower, upper in itertools.pairwise(chan_freqs))


def test_voltage_context_fine_chan_freqs_empty(any_vc: mwalib.VoltageContext):
    assert any_vc.get_fine_chan_freqs_hz_array([]) == []


#
# Regression tests for reading voltages.
#
# These check every element of the data (rather than just a sum), that reads can be done from many threads at once,
# and the layout of what is returned.
#

# Golden values: the SHA-256 of the data read from the test files, as read by the current implementation.
# If one of these changes, the data we return has changed (e.g. the order of the data).
GOLDEN_VOLTAGE_SHA256 = {
    ("mwax", 14): "a68004258cfc5cc0d542b48c13637cbe79102a7fae7925960a7c34e664b83e00",
    ("mwax", 15): "3f7c4a964dca93a031494110ac833291521936473fd07f73e59ae919653615b8",
    ("legacy", 14): "c4cf71e052d3e7002f730dd93a7e407908227516c100f7532ce311e84429cfbf",
}


@pytest.mark.parametrize(("kind", "coarse_chan"), list(GOLDEN_VOLTAGE_SHA256))
def test_voltage_context_read_file_golden(kind: str, coarse_chan: int, request):
    vc: mwalib.VoltageContext = request.getfixturevalue(f"{kind}_vc")

    data = vc.read_file(0, coarse_chan)

    assert data.dtype == np.int8
    assert data.flags.c_contiguous
    assert data.flags.writeable
    assert hashlib.sha256(data.tobytes()).hexdigest() == GOLDEN_VOLTAGE_SHA256[(kind, coarse_chan)]


def test_voltage_context_read_returns_independent_arrays(mwax_vc: mwalib.VoltageContext):
    first = mwax_vc.read_file(0, 14)
    first[...] = np.int8(-1)

    # Changing what we were given does not change what we get next time
    second = mwax_vc.read_file(0, 14)

    assert hashlib.sha256(second.tobytes()).hexdigest() == GOLDEN_VOLTAGE_SHA256[("mwax", 14)]


def test_voltage_context_read_from_many_threads(mwax_vc: mwalib.VoltageContext):
    gps_start = 1101503312
    expected_by_file = {c: mwax_vc.read_file(0, c) for c in (14, 15)}
    expected_by_second = mwax_vc.read_second(gps_start, 8, 14)

    def read_file(coarse_chan: int):
        return mwax_vc.read_file(0, coarse_chan)

    def read_second(_: int):
        return mwax_vc.read_second(gps_start, 8, 14)

    with ThreadPoolExecutor(max_workers=4) as executor:
        files = list(executor.map(read_file, [14, 15] * 4))
        seconds = list(executor.map(read_second, range(8)))

    for coarse_chan, data in zip([14, 15] * 4, files):
        assert np.array_equal(data, expected_by_file[coarse_chan])

    for data in seconds:
        assert np.array_equal(data, expected_by_second)
