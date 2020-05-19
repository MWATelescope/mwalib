#!/usr/bin/env bash

set -eux

cargo build --release

# Requires cbindgen to be installed. Run "cargo install cbindgen" to do that,
# or, if available, install it via your package manager.
cbindgen -l c .. > ../include/mwalib.h

echo "mwalib.h has been regenerated in ../include/mwalib.h"
