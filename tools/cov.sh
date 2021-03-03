#
# This is a script to generate test coverage reports
# it assumes your default rust is NOT nightly and that you have the nightly rust installed.
# It will ignore modules ending in test.rs and any rustc or cargo stuff that is not mwalib
# 
cd ..
rm *.profraw
rustup run nightly cargo cov -- report --use-color --ignore-filename-regex='(/.cargo/registry|/rustc|test.rs$)' \
$( \
      for file in \
       $( \
            RUSTFLAGS="-Zinstrument-coverage" LLVM_PROFILE_FILE="json5format-%m.profraw" rustup run nightly cargo test --lib --no-run --message-format=json \
                | jq -r "select(.profile.test == true) | .filenames[]" \
                | grep -v dSYM - \
        ); \
      do \
          printf "%s %s " -object $file; \
      done \
) \
--instr-profile=json5format.profdata --summary-only

#rustup run nightly cargo cov -- show --use-color --ignore-filename-regex='/.cargo/registry' --instr-profile=json5format.profdata \
#$( \
#      for file in \
#       $( \
#           RUSTFLAGS="-Zinstrument-coverage" LLVM_PROFILE_FILE="json5format-%m.profraw" rustup run nightly cargo test --lib --no-run --message-format=json \
#                | jq -r "select(.profile.test == true) | .filenames[]" \
#                | grep -v dSYM - \
#        ); \
#       do \
#          printf "%s %s " -object $file; \
#       done \
#) --show-instantiations --show-line-counts-or-regions --Xdemangler=rustfilt | less -R
