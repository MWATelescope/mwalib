#!/bin/bash

set -eux

#
# Build cfitsio (same as in make_cfitsio.sh but without sudo)
#

curl "https://heasarc.gsfc.nasa.gov/FTP/software/fitsio/c/cfitsio-3.49.tar.gz" -o cfitsio.tar.gz
tar -xf cfitsio.tar.gz
rm cfitsio.tar.gz
cd cfitsio-3.49
# Enabling SSE2/SSSE3 could cause portability problems, but it's unlikely that anyone
# is using such a CPU...
# https://stackoverflow.com/questions/52858556/most-recent-processor-without-support-of-ssse3-instructions
# Disabling curl just means you cannot fits_open() using a URL.
CFLAGS="-O3" ./configure --prefix=/usr/local --enable-reentrant --enable-sse2 --enable-ssse3 --disable-curl

if [[ "$OSTYPE" == "linux-gnu"* ]]; then    
    make -j
    make install
    ldconfig    

elif [[ "$OSTYPE" == "darwin"* ]]; then    
    make shared
    make install    
fi
cd ..

PATH=/root/.cargo/bin:$PATH

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