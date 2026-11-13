# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

"""Helpers to check that mwalib's Python classes are read-only."""

import pytest


def public_attributes(obj: object) -> list[str]:
    """Get the names of the public data attributes of an object.

    Args:
        obj: The object.

    Returns:
        The names of the attributes that are not methods and do not start with an underscore.
    """
    return [name for name in dir(obj) if not name.startswith("_") and not callable(getattr(obj, name))]


def assert_read_only(obj: object) -> None:
    """Assert that every public data attribute of an object cannot be set.

    Args:
        obj: The object to check.
    """
    names = public_attributes(obj)
    assert names, f"{type(obj).__name__} has no attributes to check"

    for name in names:
        value = getattr(obj, name)
        with pytest.raises(AttributeError):
            setattr(obj, name, value)
