#!/bin/bash

set -eux

#
# Build cfitsio (same as in make_cfitsio.sh but without sudo)
#

curl "https://heasarc.gsfc.nasa.gov/FTP/software/fitsio/c/cfitsio-3.49.tar.gz" -o cfitsio.tar.gz
tar -xf cfitsio.tar.gz
rm cfitsio.tar.gz
cd cfitsio-3.49
# Disabling curl just means you cannot fits_open() using a URL.
./configure --prefix=/usr/local --enable-reentrant --disable-curl

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

pip3 install --upgrade pip

# Setup maturin
pip3 install maturin==1.2.3

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Build a release for each x86_64 microarchitecture level. v4 can't be
    # compiled on GitHub for some reason.
    for level in "x86-64" "x86-64-v2" "x86-64-v3"; do
        export RUSTFLAGS="-C target-cpu=${level}"
    
        # Build python first
        MWALIB_LINK_STATIC_CFITSIO=1 maturin build --release --features python --strip -i 3.7 3.8 3.9 3.10 3.11

        # Build C objects
        MWALIB_LINK_STATIC_CFITSIO=1 cargo build --release --features examples
                
        # Create new release asset tarballs
        mv target/wheels/*.whl target/release/libmwalib.{a,so} include/mwalib.h .
        tar -acvf mwalib-$(git describe --tags)-linux-${level}.tar.gz \
            LICENSE LICENSE-cfitsio README.md CHANGELOG.md \
            libmwalib.{a,so} mwalib.h
        tar -acvf mwalib-$(git describe --tags)-linux-python-${level}.tar.gz \
            LICENSE LICENSE-cfitsio README.md CHANGELOG.md \
            ./*.whl
    done
elif [[ "$OSTYPE" == "darwin"* ]]; then            
    brew install automake

    # Build python first
    MWALIB_LINK_STATIC_CFITSIO=1 maturin build --release --features python --strip -i 3.7 3.8 3.9 3.10 3.11

    # Build C objects
    MWALIB_LINK_STATIC_CFITSIO=1 cargo build --release --features examples
    
    # Create new release asset tarballs
    mv target/wheels/*.whl target/release/libmwalib.{a,dylib} include/mwalib.h .
    tar -acvf mwalib-$(git describe --tags)-macosx-x86-64.tar.gz \
            LICENSE LICENSE-cfitsio README.md CHANGELOG.md \
            libmwalib.{a,dylib} mwalib.h
    tar -acvf mwalib-$(git describe --tags)-macosx-python-x86-64.tar.gz \
        LICENSE LICENSE-cfitsio README.md CHANGELOG.md \
        ./*.whl
fi