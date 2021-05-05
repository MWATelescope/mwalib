#!/usr/bin/env bash

set -eux

cargo build --release

# Compile the example C code
gcc -O3 \
    mwalib-print-obs-context.c \
    -o mwalib-print-obs-context \
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
    mwalib-print-voltage-context.c \
    -o mwalib-print-voltage-context \
    -I ../include \
    -lm -lpthread -ldl \
    -L../target/release/ \
    -lmwalib

echo "Run the compiled binaries with some MWA files to test mwalib. NOTE: you may need to add the ../target/release path to your LD_LIBRARY env variable for the executables to work."
