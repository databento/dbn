# dbz

[![build](https://github.com/databento/dbz/actions/workflows/build.yml/badge.svg)](https://github.com/databento/dbz/actions/workflows/build.yml)
![license](https://img.shields.io/github/license/databento/dbz?color=blue)

A library (`dbz-lib`) and CLI tool (`dbz-cli`) for working with Databento Binary
Encoding (DBZ) files.
Python bindings for `dbz-lib` are provided in the `dbz-python` package.

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
- [`dbz-cli`](src/dbz-cli/README.md)
- [`dbz-lib`](src/dbz-lib/README.md)
- [`dbz-python`](src/dbz-python/README.md)

## Building

`dbz` is written in Rust, so you'll need to have [Rust installed](https://www.rust-lang.org/)
first.

To build, run the following commands:
```sh
git clone https://github.com/databento/dbz
cd dbz
cargo build --release
./target/release/dbz --help
```

For the Python bindings, you'll need [Maturin](https://github.com/PyO3/maturin) installed as well.
To install the bindings for your active Python environment, run the following:

```sh
maturin develop --release
python -c 'import dbz_python'""
```

## Testing

Tests are run through `cargo test`.

# License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
