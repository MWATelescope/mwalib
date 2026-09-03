#!/usr/bin/env bash

set -eux

cargo build --features="cfitsio-static"

mkdir -p build
pushd build
cmake .. -DCMAKE_BUILD_TYPE=Debug 
make
popd

valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes ./build/mwalib-print-context ../test_files/metafits_signal_chain_corr/1096952256_metafits.fits > /dev/null

valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes ./build/mwalib-print-volt-context ../test_files/1370755832_mwax_vcs_os/1370755832_metafits.fits ../test_files/1370755832_mwax_vcs_os/1370755832_1370755832_123.sub ../test_files/1370755832_mwax_vcs_os/1370755832_1370755832_124.sub > /dev/null

valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes ./build/mwalib-sum-all-hdus ../test_files/1244973688_1_timestep/1244973688.metafits ../test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_000.fits  ../test_files/1244973688_1_timestep/1244973688_20190619100110_ch114_001.fits > /dev/null

valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes ./build/mwalib-sum-vcs ../test_files/1370755832_mwax_vcs_os/1370755832_metafits.fits ../test_files/1370755832_mwax_vcs_os/*_12*.sub > /dev/null

echo "Run the compiled binaries with some MWA files to test mwalib. NOTE: you may need to add the ../target/release path to your LD_LIBRARY_PATH env variable for the executables to work."