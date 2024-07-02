# databento-dbn

[![build](https://github.com/databento/dbn/actions/workflows/build.yaml/badge.svg)](https://github.com/databento/dbn/actions/workflows/build.yaml)
![python](https://img.shields.io/badge/python-3.8+-blue.svg)
![license](https://img.shields.io/github/license/databento/dbn?color=blue)
[![pypi-version](https://img.shields.io/pypi/v/databento_dbn)](https://pypi.org/project/databento-dbn)

Python bindings for the `dbn` Rust library, used by the [Databento Python client library](https://github.com/databento/databento-python).
For more information about the encoding, read our [introduction to DBN](https://databento.com/docs/standards-and-conventions/databento-binary-encoding).

Using this library is for advanced users and is not fully documented or supported.

## Installation

To install the latest stable version from PyPI:
```sh
pip install -U databento-dbn
```

## Usage and documentation

See the [documentation](https://databento.com/docs/quickstart?historical=python&live=python) for the Python client library.

## Building

`databento-dbn` is written in Rust, so you'll need to have [Rust installed](https://www.rust-lang.org/)
as well as [Maturin](https://github.com/PyO3/maturin).

To build, run the following commands:
```sh
git clone https://github.com/databento/dbn
cd dbn
maturin build
```

To build the Python package and install it for the active Python interpreter in your `PATH`, run:
```sh
maturin develop
```
This will install a package named `databento-dbn` in your current Python environment.

## License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
