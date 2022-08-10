# dbz-cli

This crate provides a CLI tool `dbz` for converting the Databento Binary
Encoding (DBZ) files to text formats. This tool is heavily inspired by the
[`zstd` CLI](https://github.com/facebook/zstd).

## Usage

`dbz` currently supports CSV and JSON as output formats. Running
```sh
dbz some.mbo.dbz --encoding csv
```
will create a new file `some.mbo.csv` with the data from `some.mbo.dbz`
formatted as a CSV.

You may also specify an output file name:
```sh
dbz some.mbo.dbz --output a_different_name.mbo.json
```
If the output file name has a `.json` or `.csv` extension, the encoding is
implied and no `--encoding` argument is required, but it can still be used as an
override.

If you want to view the contents of a DBZ file in the terminal or pipe the
output to another program, pass the `-c` or `--stdout` flag. For example, to
print the first five rows to the terminal, you'd run:
```sh
dbz some.mbo.dbz --encoding csv --stdout | head -n 5
```

By default, `dbz` will not overwrite an existing file.
To replace the contents of an existing file and allow overwriting files, pass
the `-f` or `--force` flag.

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

## Testing

Tests are run through `cargo test`.
All integration tests are located in [tests.rs](tests/tests.rs).

## License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
