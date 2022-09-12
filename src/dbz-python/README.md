# dbz-python

Python bindings for the `dbz-lib` Rust library.

## Building

`dbz-python` is written in Rust, so you'll need to have [Rust installed](https://www.rust-lang.org/)
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
This will install a package named `dbz_python` in your current Python environment.

## License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
