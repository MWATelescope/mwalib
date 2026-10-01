#!/usr/bin/env bash
#
# Regenerates the Python type stubs, mwa_giant_squid.pyi, from the
# pyo3-stub-gen annotations in src/python. Run it after a change to the
# Python API, then review and commit the result:
#
#     tools/generate_stubs.sh
#     git diff mwa_giant_squid.pyi
#
# pyo3-stub-gen writes `typing.Optional[X]` and its own layout, so ruff
# then changes the stub to the project style (`X | None`, line length 120).
# It assumes cargo and uv are on the PATH. It can run from any directory.
# Linux only: on macOS the library path variable is different, and this
# script has not been tested there.
#
# stub_gen is a normal program that embeds Python, so it is linked against
# a libpython. The script gives pyo3 the project's Python (the one uv uses,
# usually .venv) and puts that Python's library directory on the library
# path: a uv-managed Python keeps libpython in its own directory, which the
# dynamic loader does not search.

# Fail the script on any error
set -euo pipefail

cd "$(dirname "$0")/.."

readonly STUB_FILE="mwalib.pyi"

cargo build --features python,python-stubgen
target/debug/stub_gen
uv run ruff check --fix --quiet "${STUB_FILE}"
uv run ruff format --quiet "${STUB_FILE}"
