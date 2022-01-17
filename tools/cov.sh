#
# This is a script to generate test coverage reports in lcov format
# it assumes your default rust is NOT nightly and that you have the nightly rust installed.
# 
echo This should be run from the mwalib base directory!

export LD_LIBRARY_PATH=/usr/local/lib/
export CARGO_INCREMENTAL=0
#export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Coverflow-checks=off"
export RUSTDOCFLAGS="-Cpanic=abort"
export LLVM_PROFILE_FILE=json5format-%m.profraw

mkdir -p coverage
rustup run nightly cargo build
rustup run nightly cargo test
zip -0 ccov.zip `find . \( -name "mwalib*.gc*" \) -print`
grcov ccov.zip -s . -t lcov --llvm --branch --ignore-not-existing --ignore "/*" -o coverage/coverage.lcov
