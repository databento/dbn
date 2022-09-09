# dbz-lib

The official library for working with the Databento Binary Encoding (DBZ) format.

## Usage

The primary point for entrypoint for `dbz_lib` is the `Dbz` object, which
represents the contents of one DBZ file or byte stream.
To read a DBZ file with MBO data and print each row:
```rust
use databento_defs::tick::TickMsg;
use dbz_lib::Dbz;

let dbz = Dbz::from_file("20201228.dbz")?;
for tick in dbz.try_into_iter::<TickMsg>() {
    println!("{tick:?}");
}
```

The documentation provides an overview of all methods and features.

## Documentation

FIXME: when prepping to release to crates.io

## Building

`dbz` is written in Rust, so you'll need to have [Rust installed](https://www.rust-lang.org/)
first.

To build, run the following commands:
```sh
git clone https://github.com/databento/dbz
cd dbz
cargo build --release
```

### Python bindings

To also include the optional Python bindings when running any `cargo` command,
pass the `--all-features` flag or `--features=python`.
For example, to build all of dbz with Python bindings, run:
```sh
cargo build --all-features
```

To create a Python package from `dbz-lib`, you'll need to install [Maturin](https://github.com/PyO3/maturin).
For a Python package to develop against, run:
```
maturin develop -F python
```
This will install a package named `dbz_lib` in your current Python environment.

## Testing

Tests are run through `cargo test` and are located within each module.

## License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
