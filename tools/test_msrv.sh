#!/usr/bin/env bash

#
# Usage:
# ./test_msrv.sh [--features=...]
# If --features=xxx is ommited it will use all features except python-stubgen since that is only used in development

# Fail the script on any error
set -e

# Check if a command line argument was provided
if [ -n "$1" ]; then
    FEATURES="$1"
else
    FEATURES="--features=python,cfitsio-static,examples"
fi

echo "Running tests using $FEATURES"

#
# This is a helper script so that when mwalib is ready to be
# released, this can check to ensure that it works in the
# minimum Rust version (MSRV) as specified in Cargo.toml
#
# It assumes:
# 1. You run this from inside the "tools" directory
# 2. You have rustup installed
# 3. You are in a uv controlled Python virtual environment
#

# Sync Python
uv sync --locked

source .venv/bin/activate

# Switch to the root mwalib dir
pushd ..

# update rust
echo "Updating rust..."
rustup update

# Ensure MSRV version of rust is installed
MIN_RUST=$(grep -m1 "rust-version" Cargo.toml | sed 's|.*\"\(.*\)\"|\1|')
echo "Installing MSRV ${MIN_RUST}..."
rustup install ${MIN_RUST}

# Clear everything
cargo clean
rm -rf target Cargo.lock

# Update dependencies
echo "Updating cargo dependencies..."
RUSTUP_TOOLCHAIN=${MIN_RUST} cargo update --verbose

# Build and run rust tests
echo "Building and running tests..."
RUSTUP_TOOLCHAIN=${MIN_RUST} cargo test $FEATURES --release

# Install mwalib python wheel
echo "Installing mwalib python wheel..."
RUSTUP_TOOLCHAIN=${MIN_RUST} maturin develop $FEATURES --release --strip

# Run python tests
echo "Running python tests..."
pytest

# Build C examples
echo "Building C examples..."
pushd examples
./build_c_examples.sh
popd

# Switch back to this dir
popd