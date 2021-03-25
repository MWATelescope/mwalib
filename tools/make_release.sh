#!/usr/bin/env bash

# Must be run with the version number as the only param. e.g.
# ./make_release.sh 0.6.0
if [[ $# -eq 0 ]] ; then
    echo 'You must provide the release version number. E.g. ./make_release.sh 0.6.0'
    exit 1
fi

echo "Making release... $1"
echo "Cleaning up previous release files..."
rm -rf ../target
rm -rf release
echo "Building mwalib..."
export MWALIB_LINK_STATIC_CFITSIO=1
cargo build --release
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
tar -czvf libmwalib-$1-linux_x86_64.tar.gz lib/libmwalib.a lib/libmwalib.so include/mwalib.h LICENSE LICENSE-cfitsio CHANGELOG.md
echo "Release complete!"
