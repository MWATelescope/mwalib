#
# This is a script to generate test coverage reports in lcov format
# it assumes your default rust is NOT nightly and that you have the nightly rust installed.
# 
cd ..
rustup run nightly cargo clean
RUSTFLAGS="-Zinstrument-coverage" LLVM_PROFILE_FILE="json5format-%m.profraw" rustup run nightly cargo test --tests
rustup run nightly cargo profdata -- merge -sparse json5format-*.profraw -o json5format.profdata
rustup run nightly cargo cov -- export --format=lcov --ignore-filename-regex='(/.cargo/registry|/rustc|test.rs$)' --instr-profile=json5format.profdata \
$( \
     for file in \
            $( \
                 RUSTFLAGS="-Zinstrument-coverage" LLVM_PROFILE_FILE="json5format-%m.profraw" rustup run nightly cargo test --tests --no-run --message-format=json \
	                 | jq -r "select(.profile.test == true) | .filenames[]" \
	                 | grep -v dSYM - \
         ); \
       do \
          printf "%s %s " -object $file; \
       done \
) > coverage/coverage.lcov
rm *.profraw
rm *.profdata
