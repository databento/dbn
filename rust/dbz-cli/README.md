# dbz-cli

[![build](https://github.com/databento/dbz/actions/workflows/build.yml/badge.svg)](https://github.com/databento/dbz/actions/workflows/build.yml)
![license](https://img.shields.io/github/license/databento/dbz?color=blue)
[![Current Crates.io Version](https://img.shields.io/crates/v/dbz-cli.svg)](https://crates.io/crates/dbz-cli)

This crate provides a CLI tool `dbz` for converting the Databento Binary
Encoding (DBZ) files to text formats. This tool is heavily inspired by the
[`zstd` CLI](https://github.com/facebook/zstd).

## Usage

`dbz` currently supports CSV and JSON (technically [newline-delimited JSON](http://ndjson.org/))
as output formats.
By default `dbz` outputs the result to standard output for ease of use with
text-based command-line utilities.
Running
```sh
dbz some.dbz --csv | head -n 5
```
will print the first the header row and 4 data rows in `some.dbz` in CSV format to the console.
Similarly, running
```sh
dbz ohlcv-1d.dbz --json | jq '.high'
```
Will extract only the high prices from `ohlcv-1d.dbz`.

You can also save the results directly to another file by running
```sh
dbz some.dbz --json --output some.json
```
`dbz` will create a new file `some.csv` with the data from `some.dbz`
formatted as JSON.

When the file name passed `--output` or `-o` ends in `.json` or `.csv`, you
can omit the `--json` and `--csv` flags.
```sh
dbz another.dbz -o data.csv
```
This writes the contents of `another.dbz` to `data.json` in CSV format.

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
