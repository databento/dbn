#! /usr/bin/env bash

source "$(dirname "$0")/config.sh"
cd "${PROJECT_ROOT_DIR}/tests/data"
find . -name '*.dbn*' -exec cargo run -- {} --output {} --force \;
