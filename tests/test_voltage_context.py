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

MWAX_VCS_METAFITS = "../test_files/1101503312_mwax_vcs/1101503312.metafits"
MWAX_VCS_VOLTAGE_FILES = [
    "../test_files/1101503312_mwax_vcs/1101503312_1101503312_123.sub",
    "../test_files/1101503312_mwax_vcs/1101503312_1101503312_124.sub",
    "../test_files/1101503312_mwax_vcs/1101503312_1101503320_123.sub",
    "../test_files/1101503312_mwax_vcs/1101503312_1101503320_124.sub",
]


LEGACY_VCS_METAFITS = "../test_files/1101503312_vcs/1101503312.metafits"
LEGACY_VCS_VOLTAGE_FILES = [
    "../test_files/1101503312_vcs/1101503312_1101503312_ch123.dat",
    "../test_files/1101503312_vcs/1101503312_1101503312_ch124.dat",
    "../test_files/1101503312_vcs/1101503312_1101503313_ch123.dat",
    "../test_files/1101503312_vcs/1101503312_1101503313_ch124.dat",
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

    assert np.sum(data_by_gpsecond, dtype=np.int64) == 5222400000

    assert data_by_gpsecond.shape == (
        gps_seconds,
        mwax_vc.num_voltage_blocks_per_second,
        mwax_vc.metafits_context.num_ants,
        mwax_vc.metafits_context.num_ant_pols,
        mwax_vc.num_samples_per_voltage_block,
        mwax_vc.sample_size_bytes,
    )

    # Check the sums are equial
    assert np.sum(data_by_file, dtype=np.float64) == np.sum(
        data_by_gpsecond, dtype=np.float64
    )

    # Check data detail
    # second: 0, block: 0, ant: 0, pol: 0, sample: 0, value: 0
    assert data_by_gpsecond[0, 0, 0, 0, 0, 0] == 0

    # second: 0, block: 0, ant: 0, pol: 0, sample: 1, value: 1
    assert data_by_gpsecond[0, 0, 0, 0, 1, 1] == 253

    # second: 0, block: 0, ant: 0, pol: 0, sample: 255, value: 0
    assert data_by_gpsecond[0, 0, 0, 0, 255, 0] == 254

    # second: 0, block: 0, ant: 0, pol: 0, sample: 256, value: 1
    assert data_by_gpsecond[0, 0, 0, 0, 256, 1] == 255

    # second: 0, block: 1, ant: 0, pol: 0, sample: 2, value: 0
    assert data_by_gpsecond[0, 1, 0, 0, 2, 0] == 9

    # second: 0, block: 159, ant: 0, pol: 1, sample: 63999, value: 1
    # second: 7, block: 19, ant: 0, pol: 1, sample: 63999, value: 1
    assert data_by_gpsecond[7, 19, 0, 1, 63999, 1] == 226

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

    assert np.sum(data_by_gpsecond, dtype=np.int64) == 326353664

    assert data_by_gpsecond.shape == (
        gps_seconds,
        legacy_vc.num_samples_per_voltage_block,
        legacy_vc.num_fine_chans_per_coarse,
        legacy_vc.metafits_context.num_ants,
        legacy_vc.metafits_context.num_ant_pols,
        legacy_vc.sample_size_bytes,
    )

    # Check the sums are equial
    assert np.sum(data_by_file, dtype=np.float64) == np.sum(
        data_by_gpsecond, dtype=np.float64
    )

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
    assert data_by_gpsecond[0, 10, 32, 0, 1, 0] == 138

    # second: 0, sample: 9999, fine_chan: 127, ant: 0, pol: 1, sample
    assert data_by_gpsecond[0, 9999, 127, 0, 1, 0] == 187
