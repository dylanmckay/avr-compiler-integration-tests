#! /bin/bash
#
# Runs all of the tests within this repository.
#

set -e

# Build all executables so we can use them from the tests.
cargo build

# Run internal Rust tests
cargo check

# Run the integration tests.
cargo run --bin avr-lit tests/

