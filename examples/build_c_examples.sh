#!/usr/bin/env bash

set -eux

cargo build --release --features="cfitsio-static"

make clean

make 

echo "Run the compiled binaries with some MWA files to test mwalib. NOTE: you may need to add the ../target/release path to your LD_LIBRARY_PATH env variable for the executables to work."
