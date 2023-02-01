# Changelog

## 0.3.0
- Rename DBZ to DBN
  - Add ability to migrate legacy DBZ to DBN through CLI
- Rename python package to `databento-dbn`
- Drop requirement that DBN be Zstandard-compressed
- Move metadata out of skippable frame
- Fold in `databento-defs`
- Add billable size calculation to `dbn` CLI
- Add `MetadataBuilder` to assist with defaults
- Refactor into encoders and decoders

## 0.2.1 - 2022-12-02
- Add Python DBZ writing example
- Depend on crates.io version of [databento-defs](https://crates.io/crates/databento-defs)

## 0.2.0 - 2022-11-28
- Change JSON output to NDJSON
- Quote nanosecond timestamps in JSON to avoid loss of precision when parsing
- Change DBZ decoding to use [streaming-iterator](https://crates.io/crates/streaming-iterator)
- Enable Zstd checksums
- Add interface for writing DBZ files

## 0.1.5 - 2022-09-14
- Initial release
