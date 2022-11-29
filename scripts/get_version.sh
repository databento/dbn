#! /usr/bin/env bash

source "$(dirname "$0")/config.sh"
grep -E '^version =' "${PROJECT_ROOT_DIR}/src/dbz-python/Cargo.toml" | cut -d'"' -f 2
