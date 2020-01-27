#!/usr/bin/env bash

cargo build --release

# Requires cbindgen to be installed. Run "cargo install cbindgen" to do that.
cbindgen -l c . > mwalib.h

gcc -O3 \
    -o test-via-c \
    test.c \
    -lcfitsio -lm -lpthread -ldl \
    ./target/release/libmwalib.a

echo "Run test-via-c with some MWA files to test mwalib."
