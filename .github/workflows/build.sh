#!/bin/bash

set -eux

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    #
    # Build cfitsio
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
    make -j
    sudo make install
    sudo ldconfig
    cd ..
    
    PATH=/root/.cargo/bin:$PATH
    # 1.63 is the newest rustc version that can use glibc >= 2.11, and we use it
    # because newer versions require glibc >= 2.17 (which this container
    # deliberately doesn't have; we want maximum compatibility, so we use an old
    # glibc).
    rustup install 1.64 --no-self-update
    rustup default 1.64

    # Setup maturin
    pip3 install maturin==1.2.3

    # Build a release for each x86_64 microarchitecture level. v4 can't be
    # compiled on GitHub for some reason.
    for level in "x86-64" "x86-64-v2" "x86-64-v3"; do
        export RUSTFLAGS="-C target-cpu=${level}"

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
    done
elif [[ "$OSTYPE" == "darwin"* ]]; then
    #
    # Build cfitsio      
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
    sudo make shared
    sudo make install
    cd ..

    # Setup maturin
    pip3 install maturin==1.2.3
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