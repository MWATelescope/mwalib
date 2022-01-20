#
# This is a script to generate test coverage reports in lcov format
# it assumes your default rust is NOT nightly and that you have the nightly rust installed.
# 
echo This should be run from the mwalib base directory!

rm -rf target

#
# Setup cargo-llvm-cov
# See: https://github.com/taiki-e/cargo-llvm-cov for more info
#
# Toolchain is pinned due to this issue with nightly >2022-01-14: https://github.com/rust-lang/rust/issues/93054
rustup toolchain install nightly-2022-01-14
rustup component add llvm-tools-preview --toolchain nightly-2022-01-14
rustup run nightly-2022-01-14 cargo install cargo-llvm-cov

# Generate coverage and show summary on the console
# The extra options specified:
# * --lib == only cover the library 
# * --ignore-filename-regex="test.rs" == skip tests
rustup run nightly-2022-01-14 cargo llvm-cov --lib --ignore-filename-regex="test.rs"