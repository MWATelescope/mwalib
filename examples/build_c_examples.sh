#!/usr/bin/env bash

set -eux

cargo build --release --features="cfitsio-static"

mkdir -p build
pushd build
cmake .. -DCMAKE_BUILD_TYPE=Release 
make
popd

echo "Run the compiled binaries with some MWA files to test mwalib."
