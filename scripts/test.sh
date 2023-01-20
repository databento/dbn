#! /usr/bin/env bash
set -e
cargo test
cd rust/dbn
cargo test --features python-test
