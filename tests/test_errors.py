#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

#
# See README.md on how to build and run the tests.
#
import pytest

import mwalib

MWAX_CORRELATOR_METAFITS = "test_files/1244973688_1_timestep/1244973688.metafits"
MWAX_CORRELATOR_GPUBOX_FILES = [
    "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits",
    "test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_001.fits",
]
MWAX_VCS_METAFITS = "test_files/1101503312_mwax_vcs/1101503312.metafits"
NOT_A_FILE = "test_files/this_file_does_not_exist.fits"


def test_mwalib_error_is_an_exception():
    assert issubclass(mwalib.MwalibError, Exception)


def test_metafits_context_file_not_found():
    with pytest.raises(mwalib.MwalibError, match="Couldn't open"):
        mwalib.MetafitsContext(NOT_A_FILE, None)


def test_metafits_context_filename_is_a_directory():
    with pytest.raises(mwalib.MwalibError):
        mwalib.MetafitsContext("test_files", None)


def test_metafits_context_filename_wrong_type():
    with pytest.raises(TypeError):
        mwalib.MetafitsContext(None, None)  # type: ignore[arg-type]

    with pytest.raises(TypeError):
        mwalib.MetafitsContext(1234, None)  # type: ignore[arg-type]


def test_metafits_context_version_wrong_type():
    with pytest.raises(TypeError):
        mwalib.MetafitsContext(MWAX_CORRELATOR_METAFITS, "CorrMWAXv2")  # type: ignore[arg-type]


def test_correlator_context_metafits_not_found():
    with pytest.raises(mwalib.MwalibError):
        mwalib.CorrelatorContext(NOT_A_FILE, MWAX_CORRELATOR_GPUBOX_FILES)


def test_correlator_context_no_gpubox_files():
    with pytest.raises(mwalib.MwalibError, match="No gpubox"):
        mwalib.CorrelatorContext(MWAX_CORRELATOR_METAFITS, [])


def test_correlator_context_unrecognised_gpubox_filename():
    with pytest.raises(mwalib.MwalibError, match="gpubox filename structure"):
        mwalib.CorrelatorContext(MWAX_CORRELATOR_METAFITS, ["not_a_gpubox_file.fits"])


def test_correlator_context_gpubox_files_wrong_type():
    with pytest.raises(TypeError):
        mwalib.CorrelatorContext(MWAX_CORRELATOR_METAFITS, MWAX_CORRELATOR_GPUBOX_FILES[0])  # type: ignore[arg-type]


def test_voltage_context_metafits_not_found():
    with pytest.raises(mwalib.MwalibError):
        mwalib.VoltageContext(NOT_A_FILE, ["1101503312_1101503312_ch123.dat"])


def test_voltage_context_no_voltage_files():
    with pytest.raises(mwalib.MwalibError, match="No voltage files"):
        mwalib.VoltageContext(MWAX_VCS_METAFITS, [])


def test_voltage_context_unrecognised_voltage_filename():
    with pytest.raises(mwalib.MwalibError, match="voltage filename structure"):
        mwalib.VoltageContext(MWAX_VCS_METAFITS, ["not_a_voltage_file.sub"])
