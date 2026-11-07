#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

#
# See README.md on how to build and run the tests.
#
import pytest

import mwalib

# Enums and (at least) the members we rely on. Only a subset is listed for the large enums, so adding a new member
# does not break these tests.
ENUM_MEMBERS = [
    pytest.param(mwalib.Pol, {"X", "Y"}, id="Pol"),
    pytest.param(mwalib.VisPol, {"XX", "XY", "YX", "YY"}, id="VisPol"),
    pytest.param(mwalib.HeightCode, {"EllipsoidHeight", "ArrayCentreOffset"}, id="HeightCode"),
    pytest.param(
        mwalib.ReceiverType,
        {"Unknown", "RRI", "NI", "Pseudo", "SHAO", "EDA2", "CIRA"},
        id="ReceiverType",
    ),
    pytest.param(
        mwalib.MWAVersion,
        {"CorrOldLegacy", "CorrLegacy", "CorrMWAXv2", "VCSLegacyRecombined", "VCSMWAXv2"},
        id="MWAVersion",
    ),
    pytest.param(
        mwalib.DelayMode,
        {"NoDelays", "Cable", "CableZenith", "FullCal", "FullChain", "FullTrack", "SignalChain", "TileBeam"},
        id="DelayMode",
    ),
    pytest.param(
        mwalib.GeometricDelaysApplied,
        {"No", "Zenith", "TilePointing", "AzElTracking"},
        id="GeometricDelaysApplied",
    ),
    pytest.param(
        mwalib.CableDelaysApplied,
        {"NoCableDelaysApplied", "CableAndRecClock", "CableAndRecClockAndBeamformerDipoleDelays"},
        id="CableDelaysApplied",
    ),
]


@pytest.mark.parametrize(("enum_type", "expected_members"), ENUM_MEMBERS)
def test_enum_has_expected_members(enum_type, expected_members: set[str]):
    members = {name for name in dir(enum_type) if not name.startswith("_")}

    assert expected_members <= members


@pytest.mark.parametrize(("enum_type", "expected_members"), ENUM_MEMBERS)
def test_enum_members_equal_themselves_only(enum_type, expected_members: set[str]):
    for name in expected_members:
        member = getattr(enum_type, name)
        assert member == member
        assert not member != member

        for other_name in expected_members - {name}:
            assert member != getattr(enum_type, other_name)


def test_enum_members_of_different_types_are_not_equal():
    assert mwalib.Pol.X != mwalib.VisPol.XX
    assert mwalib.Pol.X != mwalib.ReceiverType.Unknown


def test_pol_values():
    assert mwalib.Pol.X == 0
    assert mwalib.Pol.Y == 1


def test_height_code_values():
    # These values are the codes in the metafits "Height_Code" column
    assert mwalib.HeightCode.EllipsoidHeight == 0
    assert mwalib.HeightCode.ArrayCentreOffset == 1
