# dbn-cli

[![build](https://github.com/databento/dbn/actions/workflows/build.yaml/badge.svg)](https://github.com/databento/dbn/actions/workflows/build.yaml)
![license](https://img.shields.io/github/license/databento/dbn?color=blue)
[![Current Crates.io Version](https://img.shields.io/crates/v/dbn-cli.svg)](https://crates.io/crates/dbn-cli)

This crate provides a CLI tool `dbn` for converting the Databento Binary
Encoding (DBN) files to text formats, as well as updating legacy DBZ files to
DBN.

This tool is heavily inspired by the [`zstd` CLI](https://github.com/facebook/zstd).

## Usage

`dbn` currently supports CSV and JSON (technically [newline-delimited JSON](http://ndjson.org/))
as output formats.
By default, `dbn` outputs the result to standard output for ease of use with
text-based command-line utilities.
Running
```sh
dbn some.dbn --csv | head -n 5
```
will print the header row and 4 data rows in `some.dbn` in CSV format to the console.
Similarly, running
```sh
dbn ohlcv-1d.dbn.zst --json | jq '.high'
```
Will extract only the high prices from `ohlcv-1d.dbn.zst`.
`dbn` works with both uncompressed and Zstandard-compressed DBN files.

You can also save the results directly to another file by running
```sh
dbn some.dbn.zst --json --output some.json
```
`dbn` will create a new file `some.csv` with the data from `some.dbn.zst`
formatted as JSON.

When the file name passed `--output` or `-o` ends in `.json` or `.csv`, you
can omit the `--json` and `--csv` flags.
```sh
dbn another.dbn.zst -o data.csv
```
This writes the contents of `another.dbn.zst` to `data.json` in CSV format.

By default, `dbn` will not overwrite an existing file.
To replace the contents of an existing file and allow overwriting files, pass
the `-f` or `--force` flag.

### Compressing the output
In addition to reading Zstandard-compressed files, `dbn` can also write compressed JSON and CSV.

```sh
dbn ohlcv-1d.dbn -o ohclv-1d.json.zst
```

or explicitly
```
dbn ohlcv-1d.dbn --json --zstd > ohlcv-1d.json.zst
```

### Converting DBZ files to DBN

DBN is an evolution of DBZ, which required Zstandard.
To update an old DBZ file to Zstandard-compressed DBN, run
```sh
dbn 20221212.mbo.dbz -o 20221212.dbn.zst
```
or pass `--dbn` to set the output encoding explicitly.

## License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
