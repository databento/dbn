# dbz

[![build](https://github.com/databento/dbz/actions/workflows/build.yml/badge.svg)](https://github.com/databento/dbz/actions/workflows/build.yml)
![license](https://img.shields.io/github/license/databento/dbz?color=blue)
[![Current Crates.io Version](https://img.shields.io/crates/v/dbz.svg)](https://crates.io/crates/dbz)
![pypi-version](https://img.shields.io/pypi/v/databento_dbz)

A library (`dbz`) and CLI tool (`dbz`) for working with Databento Binary
Encoding (DBZ) files.
Python bindings for `dbz` are provided in the `databento-dbz` package.

The **D**atabento **B**inary Encoding + **Z**standard compression (DBZ) is an efficient
highly compressible binary encoding suitable for bulk financial time series data,
which includes a metadata header.

## Features

- Performant binary encoding and decoding
- Highly compressible with Zstandard
- Extendable fixed-width schemas
- Metadata header in a leading zstd skippable frame

The DBZ format relies on a compliant Zstandard decompressor to read the data.
The basic metadata can be read without the need for zstd, as it is not
compressed, however the symbology portion is.

## Usage

See the respective READMEs for usage details:
- [`dbz`](rust/dbz/README.md)
- [`dbz-cli`](rust/dbz-cli/README.md)
- [`databento-dbz`](python/README.md)

# License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
