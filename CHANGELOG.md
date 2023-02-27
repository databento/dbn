# Changelog

## 0.3.1 - 2023-02-27
- Added improved Python bindings for decoding DBN
- Fixed bug with `encode_metadata` Python function
- Standardized documentation for `start`, `end`, and `limit`

## 0.3.0 - 2023-02-22
- Renamed DBZ to DBN
  - Added ability to migrate legacy DBZ to DBN through CLI
- Renamed python package to `databento-dbn`
- Dropped requirement that DBN be Zstandard-compressed
- Moved metadata out of skippable frame
- Folded in `databento-defs`
- Added support for async encoding and decoding
- Added billable size calculation to `dbn` CLI
- Added `MetadataBuilder` to assist with defaults
- Refactored into encoder and decoder types

## 0.2.1 - 2022-12-02
- Added Python DBZ writing example
- Changed [databento-defs](https://crates.io/crates/databento-defs) dependency to crates.io version

## 0.2.0 - 2022-11-28
- Changed JSON output to NDJSON
- Change nanosecond timestamps to strings in JSON to avoid loss of precision when parsing
- Changed DBZ decoding to use [streaming-iterator](https://crates.io/crates/streaming-iterator)
- Enabled Zstd checksums
- Added interface for writing DBZ files

## 0.1.5 - 2022-09-14
- Initial release
