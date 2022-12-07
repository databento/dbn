#! /usr/bin/env bash
set -e
cargo test
cd rust/dbz
cargo test --features python-test
