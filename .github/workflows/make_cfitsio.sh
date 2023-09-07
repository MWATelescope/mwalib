#!/bin/bash

set -eux

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

if [[ "$OSTYPE" == "linux-gnu"* ]]; then    
    make -j
    sudo make install
    sudo ldconfig    

elif [[ "$OSTYPE" == "darwin"* ]]; then    
    sudo make shared
    sudo make install    
fi

cd ..