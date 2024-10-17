#!/usr/bin/env bash

# Fail the script on any error
set -e

#
# This is a helper script so that when mwalib is ready to be
# released, this can check to ensure that it works in the
# minimum Rust version (MSRV) as specified in Cargo.toml
#
# It assumes:
# 1. You run this from inside the "tools" directory
# 2. You have rustup installed
# 3. You are in a Python virtual environment
#

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
RUSTUP_TOOLCHAIN=${MIN_RUST} cargo test --release --all-features

# Install mwalib python wheel
echo "Installing mwalib python wheel..."
RUSTUP_TOOLCHAIN=${MIN_RUST} maturin develop --all-features --release --strip

# Run python tests
echo "Running python tests..."
pytest

# Switch back to this dir
popd