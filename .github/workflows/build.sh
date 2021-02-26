#!/bin/bash

set -eux

# I don't know why, but I need to reinstall Rust. Probably something to do with
# GitHub overriding env variables.
#curl https://sh.rustup.rs -sSf | sh -s -- -y

# Install dependencies
curl "https://heasarc.gsfc.nasa.gov/FTP/software/fitsio/c/cfitsio-3.48.tar.gz" -o cfitsio.tar.gz
tar -xf cfitsio.tar.gz
rm cfitsio.tar.gz
cd cfitsio-3.48
# Enabling SSSE3 could cause portability problems, but it's unlikely that anyone
# is using such a CPU...
# https://stackoverflow.com/questions/52858556/most-recent-processor-without-support-of-ssse3-instructions
CFLAGS="-O3" ./configure --prefix=/usr/local --enable-reentrant --enable-ssse3
make -j8
make install
ldconfig

# Build
MWALIB_LINK_STATIC_CFITSIO=1 cargo build --release