#!/usr/bin/env bash

set -eux

cargo build --release --features=cfitsio-static

# Compile the example C code
gcc -O3 \
    mwalib-print-context.c \
    -o mwalib-print-context \
    -I ../include \
    -lm -lpthread -ldl \
    -L../target/release/ \
    -lmwalib

gcc -O3 \
    mwalib-sum-all-hdus.c \
    -o mwalib-sum-all-hdus \
    -I ../include \
    -lm -lpthread -ldl \
    -L../target/release/ \
    -lmwalib

gcc -O3 \
    mwalib-print-volt-context.c \
    -o mwalib-print-volt-context \
    -I ../include \
    -lm -lpthread -ldl \
    -L../target/release/ \
    -lmwalib

gcc -O3 \
    mwalib-sum-vcs.c \
    -o mwalib-sum-vcs \
    -I ../include \
    -lm -lpthread -ldl \
    -L../target/release/ \
    -lmwalib

echo "Run the compiled binaries with some MWA files to test mwalib. NOTE: you may need to add the ../target/release path to your LD_LIBRARY_PATH env variable for the executables to work."
