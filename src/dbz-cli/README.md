# dbz-cli

This crate provides a CLI tool `dbz` for converting the Databento Binary
Encoding (DBZ) files to text formats. This tool is heavily inspired by the
[`zstd` CLI](https://github.com/facebook/zstd).

## Usage

`dbz` currently supports CSV and JSON as output formats.
Running
```sh
dbz some.dbz --encoding csv | head -n 5
```
will print the first 5 rows in `some.dbz` in CSV format to the console.

You can also save the results directly to another file by running
```sh
dbz some.dbz -e csv --output some.csv
```
`dbz` will output the a new file `some.csv` with the data from `some.dbz`
formatted as a CSV.

When the file name passed `--output` or `-o` ends in `.json` or `.csv`, you
can omit the `--encoding` or `-e` flag.
```sh
dbz another.dbz -o data.json
```
This writes the contents of `another.dbz` to `data.json` in JSON format.

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
All integration tests are located in [integration_tests.rs](tests/integration_tests.rs).

## License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
