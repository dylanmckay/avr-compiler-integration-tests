#! /bin/bash
#
# Runs all of the tests within this repository.

if [ -z "$LLVM_SYSROOT" ]; then echo "please set \$LLVM_SYSROOT" && exit 1; fi

set -e

# Build all executables so we can use them from the tests.
cargo build

# Run internal Rust tests
cargo check

# Run the integration tests with avr-gcc.
# We expect this to always pass. If this fails, something is likely
# wrong with the tests
echo "Running tests with avr-gcc"
cargo run --bin avr-lit --quiet -- --avr-gcc tests/

echo "Running tests with LLVM"
cargo run --bin avr-lit --quiet -- --llvm-sysroot "$LLVM_SYSROOT" tests/

