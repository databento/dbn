# dbn

[![build](https://github.com/databento/dbn/actions/workflows/build.yaml/badge.svg)](https://github.com/databento/dbn/actions/workflows/build.yaml)
[![license](https://img.shields.io/github/license/databento/dbn?color=blue)](./LICENSE)
[![Current Crates.io Version](https://img.shields.io/crates/v/dbn.svg)](https://crates.io/crates/dbn)
[![pypi-version](https://img.shields.io/pypi/v/databento_dbn)](https://pypi.org/project/databento-dbn)

Libraries and a CLI tool for working with Databento Binary
Encoding (DBN) files and streams.
Python bindings for `dbn` are provided in the `databento-dbn` package.

The **D**atabento **B**inary E**n**coding (DBN) is an efficient
highly-compressible binary encoding suitable for bulk financial time series data.

## Features

- Performant binary encoding and decoding
- Highly compressible with Zstandard
- Extendable fixed-width schemas

## Usage

See the respective READMEs for usage details:
- [`dbn`](rust/dbn/README.md): Rust library crate
- [`dbn-cli`](rust/dbn-cli/README.md): CLI crate providing a `dbn` binary
- [`databento-dbn`](python/README.md): Python package

# License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
