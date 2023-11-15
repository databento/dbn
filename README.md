# dbn

[![build](https://github.com/databento/dbn/actions/workflows/build.yaml/badge.svg)](https://github.com/databento/dbn/actions/workflows/build.yaml)
[![Documentation](https://img.shields.io/docsrs/dbn)](https://docs.rs/dbn/latest/dbn/)
[![license](https://img.shields.io/github/license/databento/dbn?color=blue)](./LICENSE)
[![Current Crates.io Version](https://img.shields.io/crates/v/dbn.svg)](https://crates.io/crates/dbn)
[![pypi-version](https://img.shields.io/pypi/v/databento_dbn)](https://pypi.org/project/databento-dbn)
[![Slack](https://img.shields.io/badge/join_Slack-community-darkblue.svg?logo=slack)](https://join.slack.com/t/databento-hq/shared_invite/zt-24oqyrub9-MellISM2cdpQ7s_7wcXosw)

**D**atabento **B**inary E**n**coding (DBN) is an extremely fast message encoding and storage format for normalized market data.
The DBN specification includes a simple, self-describing metadata header and a fixed set of struct definitions, which enforce a standardized way to normalize market data.

All official Databento client libraries use DBN under the hood, both as a data interchange format and for in-memory representation of data.
DBN is also the default encoding for all Databento APIs, including live data streaming, historical data streaming, and batch flat files.

This repository contains both  libraries and a CLI tool for working with DBN files and streams.
Python bindings for `dbn` are provided in the `databento_dbn` package.

For more details, read our [introduction to DBN](https://docs.databento.com/knowledge-base/new-users/dbn-encoding/getting-started-with-dbn).

## Features

- Performant binary encoding and decoding
- Highly compressible with Zstandard
- Extendable fixed-width schemas

## Usage

See the respective READMEs for usage details:
- [`dbn`](rust/dbn/README.md): Rust library crate
- [`dbn-cli`](rust/dbn-cli/README.md): CLI crate providing a `dbn` binary
- [`databento-dbn`](python/README.md): Python package

## License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
