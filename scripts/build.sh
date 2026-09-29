#! /usr/bin/env bash
set -e

cargo --version
cargo build --all-features
cargo build --tests --no-default-features -p dbn
