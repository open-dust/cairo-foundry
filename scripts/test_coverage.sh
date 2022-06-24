#!/bin/bash

rm -rf ./target/debug/coverage/*
rm -rf ./target/debug/profraw/*

# Nightly Rust is required to use grcov for Rust coverage.
# Alternatively, you can export `RUSTC_BOOTSTRAP=1`, which basically turns your stable rustc into a Nightly one.
export RUSTC_BOOTSTRAP=1
export RUSTFLAGS="-Zinstrument-coverage"

cargo build

# Ensure each test runs gets its own profile information by defining the LLVM_PROFILE_FILE environment variable
# (%p will be replaced by the process ID, and %m by the binary signature)
export LLVM_PROFILE_FILE="./target/debug/profraw/%p-%m.profraw"

cargo test

# In the CWD, you will see a `.profraw` file has been generated.
# This contains the profiling information that grcov will parse, alongside with your binaries.


# Generate html report
grcov . --binary-path ./target/debug -s src -t html --branch --ignore-not-existing --ignore "/*" -o ./target/debug/coverage/

# Consult the coverage
open ./target/debug/coverage/index.html
