#!/usr/bin/env bash
echo "Making release..."
echo "Cleaning up previous release files..."
rm -rf ../target
rm -rf release
echo "Building mwalib..."
export MWALIB_LINK_STATIC_CFITSIO=1
cargo build --release -v
cbindgen -l c .. > ../include/mwalib.h
echo "mwalib.h has been regenerated in ../include/mwalib.h"
echo "Packaging up mwalib..."
mkdir -p release
mkdir -p release/lib
mkdir -p release/include
cp ../target/release/libmwalib.a release/lib/.
cp ../target/release/libmwalib.so release/lib/.
cp ../LICENSE release/.
cp ../LICENSE-cfitsio release/.
cp ../CHANGELOG.md release/.
cp ../include/mwalib.h release/include/.
cd release
echo "Taring files..."
tar -czvf libmwalib-0.4.3-linux_x86_64.tar.gz lib/libmwalib.a lib/libmwalib.so include/mwalib.h LICENSE LICENSE-cfitsio CHANGELOG.md
echo "Release complete!"
