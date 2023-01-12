# dbn

[![build](https://github.com/databento/dbn/actions/workflows/build.yaml/badge.svg)](https://github.com/databento/dbn/actions/workflows/build.yaml)
![license](https://img.shields.io/github/license/databento/dbn?color=blue)
[![Current Crates.io Version](https://img.shields.io/crates/v/dbn.svg)](https://crates.io/crates/dbn)
![pypi-version](https://img.shields.io/pypi/v/databento_dbn)

A library (`dbn`) and CLI tool (`dbn`) for working with Databento Binary
Encoding (DBN) files and streams.
Python bindings for `dbn` are provided in the `databento-dbn` package.

The **D**atabento **B**inary E**n**coding (DBN) is an efficient
highly-compressible binary encoding suitable for bulk financial time series data.

## Features

- Performant binary encoding and decoding
- Highly compressible with Zstandard
- Extendable fixed-width schemas
- When Zstd-compressed, an optional metadata header in a leading zstd skippable frame

## Usage

See the respective READMEs for usage details:
- [`dbn`](rust/dbn/README.md)
- [`dbn-cli`](rust/dbn-cli/README.md)
- [`databento-dbn`](python/README.md)

# License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
