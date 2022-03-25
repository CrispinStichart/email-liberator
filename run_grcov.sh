#!/bin/bash
set -euo pipefail
IFS=$'\n\t'

# Only nightly supports intrument-coverage
rustup default nightly

# export these so they're actually used
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="email_liberator-%p-%m.profraw"

# we delete them at the end, but they might have been created from outside
# the script
rm -f *.profraw

# As of this writing, there's some shared-state behavior -- specifically, 
# the path to last_message_seen is hardcoded and the tests that rely on it
# need to be run sequentially. 
cargo test -- --test-threads=1

# grcov will overwrite files without problem, but issues could
# arise if I renamed or removed source files.
rm -rf coverage/

# generate HTML report
grcov . --binary-path ./target/debug/ -s . -t html \
--branch --ignore-not-existing --keep-only 'src/*' -o coverage/

# these files are annoying an useless after we generate the report.
rm -f *.profraw
