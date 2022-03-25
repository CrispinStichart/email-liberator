#!/usr/bin/env bash

rustup default nightly
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="your_name-%p-%m.profraw"

