# Changelog

## 0.5.0 - TBD
- Added support for Statistics schema
- Changed `schema` and `stype_in` to optional in `Metadata` to support live data
- Added `RType` enum for exhaustive pattern matching
- Added `&str` getters for more `c_char` array record fields
- Changed `DbnDecoder.decode` to always return a list of tuples
- Fixed value associated with `Side::None`
- Fixed issue with decoding partial records in Python `DbnDecoder`
- Fixed missing type hint for Metadata bytes support

## 0.4.3 - 2023-04-07
- Fixed typo in Python type stubs

## 0.4.2 - 2023-04-06
- Fixed support for `ErrorMsg`, `SystemMsg`, and `SymbolMappingMsg` in Python

## 0.4.1 - 2023-04-05
- Added enums `MatchAlgorithm`, `UserDefinedInstrument`
- Added constants `UNDEF_PRICE` and `UNDEF_ORDER_SIZE`
- Added Python type stubs for `databento_dbn` package
- Fixed `Metadata.__bytes__` method to return valid DBN
- Fixed panics when decoding invalid records
- Fixed issue with attempting to decode partial records in Python `DbnDecoder`
- Fixed support for `ImbalanceMsg` in Python `DbnDecoder`

## 0.4.0 - 2023-03-24
- Added support for Imbalance schema
- Updated `InstrumentDefMsg` to include options-related fields and `instrument_class`
- Added support for encoding and decoding `ts_out`
- Added `ts_out` to `Metadata`
- Improved enum API
- Introduced separate rtypes for each OHLCV schema
- Removed `record_count` from `Metadata`
- Changed serialization of `c_char` fields to strings instead of ints
- Dropped requirement for slice passed to `RecordRef::new` to be mutable
- Added error forwarding from `DecodeDbn` methods
- Added `SystemMsg` record
- Renamed `dbn::RecordDecoder::decode_record` to `decode`
- Renamed `dbn::RecordDecoder::decode_record_ref` to `decode_ref`
- Renamed `HasRType::size` to `record_size` to avoid confusion with order size fields
- Stopped serializing `related` and `related_security_id` fields in `InstrumentDefMsg`
- Exposed constructor and additional methods for DBN records and `Metadata` to Python
- Made `RecordRef` implement `Sync` and `Send`

## 0.3.2 - 2023-03-01
- Added records and `Metadata` as exports of `databento_dbn` Python package
- Improved how `Metadata` appears in Python and added `__repr__`
- Fixed bug where `dbn` CLI tool didn't truncate existing files

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
