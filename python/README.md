# databento-dbz

[![build](https://github.com/databento/dbz/actions/workflows/build.yml/badge.svg)](https://github.com/databento/dbz/actions/workflows/build.yml)
![license](https://img.shields.io/github/license/databento/dbz?color=blue)
![pypi-version](https://img.shields.io/pypi/v/databento_dbz)

Python bindings for the `dbz` Rust library.
Used by the [Databento Python client library](https://github.com/databento/databento-python).

Using this crate is for advanced users and is not fully documented or supported.

## Installation

To install the latest stable version from PyPI:
```sh
pip install -U databento-dbz
```

## Usage

To read the metadata from a DBZ file into a `dict`, read the raw bytes and pass them to `decode_metadata`.
```python
from databento_dbz import decode_metadata

with open("my.dbz", "rb") as fin:
    metadata = decode_metadata(fin.read())
# Print symbology mappings
print(metadata["mappings"])
```

You can write DBZ files using `write_dbz_file`:
```python
from databento_dbz import write_dbz_file

records = [
    {"rtype": 160, "publisher_id": 1, "product_id": 1, "ts_event": 647784973705, "order_id": 1,
     "price": 3723000000000, "size": 1, "flags": -128, "channel_id": 0, "action": ord('C'),
     "side": ord('A'), "ts_recv": 1609160400000704060, "ts_in_delta": 0, "sequence": 1170352}
]
with open("my.dbz", "wb") as out:
    write_dbz_file(file=out, schema="mbo", dataset="custom", records=records, stype="product_id")
```
Note that the keys in the dictionaries in `records` must match the field names of the schema, or
the function will raise a `KeyError`.

## Building

`databento-dbz` is written in Rust, so you'll need to have [Rust installed](https://www.rust-lang.org/)
as well as [Maturin](https://github.com/PyO3/maturin).

To build, run the following commands:
```sh
git clone https://github.com/databento/dbz
cd dbz
maturin build
```

To build the Python package and install it for the active Python interpreter in your `PATH`, run:
```sh
maturin develop
```
This will install a package named `databento-dbz` in your current Python environment.

## License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
