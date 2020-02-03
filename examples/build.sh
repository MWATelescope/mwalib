#!/usr/bin/env bash

set -eux

cargo build --release

# Requires cbindgen to be installed. Run "cargo install cbindgen" to do that,
# or, if available, install it via your package manager.
cbindgen -l c .. > mwalib.h

gcc -O3 \
    mwalib-print-obs-context.c \
    -o mwalib-print-obs-context \
    -lcfitsio -lm -lpthread -ldl \
    ../target/release/libmwalib.a

gcc -O3 \
    mwalib-sum-all-hdus.c \
    -o mwalib-sum-all-hdus \
    -lcfitsio -lm -lpthread -ldl \
    ../target/release/libmwalib.a

echo "Run the compiled binaries with some MWA files to test mwalib."
