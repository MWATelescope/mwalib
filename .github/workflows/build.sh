#!/bin/bash

set -eux

#
# Build cfitsio
#
./make_cfitsio.sh

PATH=/root/.cargo/bin:$PATH
# 1.63 is the newest rustc version that can use glibc >= 2.11, and we use it
# because newer versions require glibc >= 2.17 (which this container
# deliberately doesn't have; we want maximum compatibility, so we use an old
# glibc).
rustup install 1.63 --no-self-update
rustup default 1.63

# Setup maturin
pip3 install maturin==1.2.3

# Build a release for each x86_64 microarchitecture level. v4 can't be
# compiled on GitHub for some reason.
for level in "x86-64" "x86-64-v2" "x86-64-v3"; do
    export RUSTFLAGS="-C target-cpu=${level}"

    if [[ "$OSTYPE" == "linux-gnu"* ]]; then        
        # Build python first
        maturin build --release --features=python,cfitsio-static --strip -i 3.7 3.8 3.9 3.10

        # Build C objects
        cargo build --release --features cfitsio-static,examples
                
        # Create new release asset tarballs
        mv target/wheels/*.whl target/release/libmwalib.{a,so} include/mwalib.h .
        tar -acvf mwalib-$(git describe --tags)-linux-${level}.tar.gz \
            LICENSE LICENSE-cfitsio README.md CHANGELOG.md \
            libmwalib.{a,so} mwalib.h
        tar -acvf mwalib-$(git describe --tags)-linux-python-${level}.tar.gz \
            LICENSE LICENSE-cfitsio README.md CHANGELOG.md \
            ./*.whl        
    elif [[ "$OSTYPE" == "darwin"* ]]; then            
        brew install automake

        # Build python first
        maturin build --release --features=python,cfitsio-static --strip

        # Build C objects
        cargo build --release --features cfitsio-static,examples
        
        # Create new release asset tarballs
        mv target/wheels/*.whl target/release/libmwalib.{a,dylib} include/mwalib.h .
        tar -acvf mwalib-$(git describe --tags)-macosx-${level}.tar.gz \
                LICENSE LICENSE-cfitsio README.md CHANGELOG.md \
                libmwalib.{a,so} mwalib.h
        tar -acvf mwalib-$(git describe --tags)-macosx-python-${level}.tar.gz \
            LICENSE LICENSE-cfitsio README.md CHANGELOG.md \
            ./*.whl
    fi
done