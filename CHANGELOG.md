# Changelog

## 0.11.1 - TBD
### Bug fixes
- Changed DBN stream detection to ignore the DBN version

## 0.11.0 - 2023-09-21
### Enhancements
- Added new `EncodeRecordTextExt` trait which is implemented for the CSV and JSON
  encoders. It adds two methods for encoding a `symbol` field along side the rest of the
  record fields, matching the behavior of `map_symbols` in the historical API
- Added `encode_header` and `encode_header_for_schema` methods to `CsvEncoder` and
  `DynEncoder` to give more flexibility for encoding CSV headers
- Added `from_file` and `from_zstd_file` functions to `AsyncDbnDecoder` to match
  synchronous decoder
- Implemented `Copy` for `RecordRef` to make it behave more like a reference
- Added `AsyncDbnEncoder` for simpler DBN encoding and to match sync API
- Added `RecordEnum` and `RecordRefEnum` to more easily be able to pattern match on
  records of different types
- Added `ARCX.PILLAR.ARCX` publisher
- Added `From` DBN records for `RecordRef`
- Added re-exports to the top level of the crate for all enums and records for simpler
  imports
- Added `ClosePrice` and `NetChange` `StatType`s used in the `OPRA.PILLAR` dataset

### Breaking changes
- Split `encode_record_ref` into a safe method with no arguments and an unsafe method
  with a `ts_out` parameter to reduce `unsafe` usage when not working with live data
  that may contain `ts_out`

### Bug fixes
- Fixed `dbn` CLI not writing CSV header when using `--fragment` and `--zstd-fragment`
  flags
- Fixed lifetime on return value from `RecordRef::get_unchecked`
- Fixed missing check for `stype_out` before building `Metadata` symbology maps

## 0.10.2 - 2023-09-12
### Bug fixes
- Fixed query range checking in `Metadata::symbol_map_for_date`
- Added `debug_assert_eq!` check for alignment in `RecordRef::new`

## 0.10.1 - 2023-09-07
### Bug fixes
- Changed `Metadata::symbol_map` and `symbol_map_for_date` to return `String` values
  instead of `&str`, which made it difficult to use

## 0.10.0 - 2023-09-07
### Enhancements
- Added `start` and `end` getters to `Metadata` that return `time::OffsetDateTime`
- Added `symbol_map` and `symbol_map_for_date` methods to `Metadata` to aid historical
  symbology mapping from the instrument IDs in records
- Added `DynReader` struct for being agnostic about whether an input stream is
  zstd-compressed
- Improved safety of `RecordRef::get` by adding length check
- Added Python DBN `Transcoder` class for converting DBN to JSON and CSV with optional
  zstd compression
- Added optional `has_metadata` parameter to Python `DBNDecoder` to allow
  decoding plain records by passing `False`. By default `DBNDecoder` expects a complete
  DBN stream, which begins with metadata
- Added `get_ref` methods to `dbn::Decoder` and `dbn::RecordDecoder` which return a
  reference to the inner reader
- Added `UNDEF_PRICE`, `UNDEF_ORDER_SIZE`, `UNDEF_STAT_QUANTITY`, and `UNDEF_TIMESTAMP`
  constants to `databento_dbn` Python package to make it easier to filter null values
- Added `Metadata::builder()` function to create a new builder instance

### Breaking changes
- Split out `EncodeRecordRef` trait from `EncodeDbn` to have a boxable trait (i.e.
  `Box<dyn EncodeRecordRef>`) for dynamic encoding
- Split out `EncodeRecord` trait from `EncodeDbn`
- Split out `DecodeRecordRef` trait from `DecodeDbn` to have a boxable trait (i.e.
  `Box<dyn DecodeRecordRef>`) for dynamic decoding
- Changed `DynWriter` from an enum to a struct with only private fields

### Bug fixes
- Fixed typo in `BATY.PITCH.BATY` publisher
- Fixed typo in `README.md` (credit: @thomas-k-cameron)

## 0.9.0 - 2023-08-24
### Enhancements
- Added `publisher` method to `RecordHeader` and all record types for converting
  the `publisher_id` to an enum
- Added getters that return `time::OffsetDateTime` for the following fields:
  `ts_event`, `ts_recv`, `ts_ref`, `activation`, `expiration`, `start_ts`, `end_ts`,
  `ts_out`
- Added getters for `ts_in_delta` that return `time::Duration`

## 0.8.3 - 2023-08-15
### Bug fixes
- Fixed missing `raw_instrument_id` field in Python `InstrumentDefMsg`
- Fixed missing `OHLCV_EOD` variant in Python `Schema` type hint

## 0.8.2 - 2023-08-10
### Enhancements
- Added new `OhlcvEod` schema variant for future use with OHLCV bars based around the
  end of the trading session
- Implemented `std::fmt::Display` for publisher enums (`Publisher`, `Dataset`, and
  `Venue`)

### Bug fixes
- Fixed Python type hint for `Encoding.variants()`

## 0.8.1 - 2023-08-02
### Enhancements
- Added `raw_instrument_id` field to `InstrumentDefMsg` (definition schema) for use in
  future datasets consolidated from multiple publishers
- Added new `OHLCV_EOD` rtype for future daily OHLCV schema based on the trading
  session
- Added new `SType::Nasdaq` and `SType::Cms` to support querying US equities datasets
  using either convention, regardless of the original convention of the dataset.
- Relaxed `pyo3`, `tokio`, and `zstd` dependency version requirements
- Added `FIXED_PRICE_SCALE` constant to `databento_dbn` Python package
- Added generated field metadata for each record type to aid in pandas DataFrame
  creation

### Breaking changes
- Changed `size_hint` class method to class attribute for Python records

### Bug fixes
- Fixed multi-frame Zstd decoding for async decoders

## 0.8.0 - 2023-07-19
### Enhancements
- Switched from `anyhow::Error` to custom `dbn::Error` for all public fallible functions
  and methods. This should make it easier to disambiguate between error types.
- `EncodeDbn::encode_record` and `EncodeDbn::record_record_ref` no longer treat a
  `BrokenPipe` error differently
- Added `AsyncDbnDecoder`
- Added `pretty::Px` and `pretty::Ts` newtypes to expose price and timestamp formatting
  logic outside of CSV and JSON encoding
- Added interning for Python strings
- Added `rtype` to encoded JSON and CSV to aid differeniating between different record types.
  This is particularly important when working with live data.
- Added `pretty_` Python attributes for DBN price fields
- Added `pretty_` Python attributes for DBN UTC timestamp fields

### Breaking changes
- All fallible operations now return a `dbn::Error` instead of an `anyhow::Error`
- Updated serialization order to serialize `ts_recv` and `ts_event` first
- Moved header fields (`rtype`, `publisher_id`, `instrument_id`, and `ts_event`) to
  nested object under the key `hd` in JSON encoding to match structure definitions
- Changed JSON encoding of all 64-bit integers to strings to avoid loss of precision
- Updated `MboMsg` serialization order to serialize `action`, `side`, and `channel_id`
  earlier given their importance
- Updated `Mbp1Msg`, `Mbp10Msg`, and `TradeMsg` serialization order to serialize
  `action`, `side`, and `depth` earlier given their importance
- Updated `InstrumentDefMsg` serialization order to serialize `raw_symbol`,
  `security_update_action`, and `instrument_class` earlier given their importance
- Removed `bool` return value from `EncodeDbn::encode_record` and
  `EncodeDbn::record_record_ref`. These now return `dbn::Result<()>`.

### Bug fixes
- Fixed handling of NUL byte when encoding DBN to CSV and JSON
- Fixed handling of broken pipe in `dbn` CLI tool

## 0.7.1 - 2023-06-26
- Added Python `variants` method to return an iterator over the enum variants for
  `Compression`, `Encoding`, `Schema`, and `SType`
- Improved Python enum conversions for `Compression`, `Encoding`, `Schema`, and `SType`

## 0.7.0 - 2023-06-20
### Enhancements
- Added publishers enums
- Added export to Python for `Compression`, `Encoding`, `SType`, and `Schema`
  enums
- Improved Python string representation of `ErrorMsg` and `SystemMsg`
- Added async JSON encoder

### Breaking changes
- Dropped support for Python 3.7

### Bug fixes
- Fixed pretty timestamp formatting to match API

## 0.6.1 - 2023-06-02
- Added `--fragment` and `--zstd-fragment` CLI arguments to read DBN streams
  without metadata
- Added `csv::Decoder::get_ref` that returns reference to the underlying writer
- Added missing Python getter for `InstrumentDefMsg::group`
- Added dataset constants
- Changed `c_char` fields to be exposed to Python as `str`

## 0.6.0 - 2023-05-26
### Enhancements
- Added `--limit NUM` CLI argument to output only the first `NUM` records
- Added `AsRef<[u8]>` implementation for `RecordRef`
- Added Python `size_hint` classmethod for DBN records
- Improved DBN encoding performance of `RecordRef`s
- Added `use_pretty_px` for price formatting and `use_pretty_ts` for datetime formatting
  to CSV and JSON encoders
- Added `UNDEF_TIMESTAMP` constant for when timestamp fields are unset

### Breaking changes
- Renamed `booklevel` MBP field to `levels` for brevity and consistent naming
- Renamed `--pretty-json` CLI flag to `--pretty` and added support for CSV. Passing this
  flag now also enables `use_pretty_px` and `use_pretty_ts`
- Removed `open_interest_qty` and `cleared_volume` fields that were always unset from
  definition schema
- Changed Python `DBNDecoder.decode` to return records with a `ts_out` attribute, instead
  of a tuple
- Rename Python `DbnDecoder` to `DBNDecoder`

### Bug fixes
- Fixed `Action` conversion methods (credit: @thomas-k-cameron)

## 0.5.1 - 2023-05-05
- Added `F`ill action type for MBO messages
- Added Python type stub for `StatMsg`

## 0.5.0 - 2023-04-25
### Enhancements
- Added support for Statistics schema
- Added `RType` enum for exhaustive pattern matching
- Added `&str` getters for more `c_char` array record fields
- Changed `DbnDecoder.decode` to always return a list of tuples

### Breaking changes
- Changed `schema` and `stype_in` to optional in `Metadata` to support live data
- Renamed `SType::ProductId` to `SType::InstrumentId` and `SType::Native` to `SType::RawSymbol`
- Renamed `RecordHeader::product_id` to `instrument_id`
- Renamed `InstrumentDefMsg::symbol` to `raw_symbol`
- Renamed `SymbolMapping::native_symbol` to `raw_symbol`
- Deprecated `SType::Smart` to split into `SType::Parent` and `SType::Continuous`

### Bug fixes
- Fixed value associated with `Side::None`
- Fixed issue with decoding partial records in Python `DbnDecoder`
- Fixed missing type hint for Metadata bytes support
- Added support for equality comparisons in Python classes

## 0.4.3 - 2023-04-07
- Fixed typo in Python type stubs

## 0.4.2 - 2023-04-06
- Fixed support for `ErrorMsg`, `SystemMsg`, and `SymbolMappingMsg` in Python

## 0.4.1 - 2023-04-05
### Enhancements
- Added enums `MatchAlgorithm`, `UserDefinedInstrument`
- Added constants `UNDEF_PRICE` and `UNDEF_ORDER_SIZE`
- Added Python type stubs for `databento_dbn` package

### Bug fixes
- Fixed `Metadata.__bytes__` method to return valid DBN
- Fixed panics when decoding invalid records
- Fixed issue with attempting to decode partial records in Python `DbnDecoder`
- Fixed support for `ImbalanceMsg` in Python `DbnDecoder`

## 0.4.0 - 2023-03-24
### Enhancements
- Added support for Imbalance schema
- Updated `InstrumentDefMsg` to include options-related fields and `instrument_class`
- Added support for encoding and decoding `ts_out`
- Added `ts_out` to `Metadata`
- Improved enum API
- Relaxed requirement for slice passed to `RecordRef::new` to be mutable
- Added error forwarding from `DecodeDbn` methods
- Added `SystemMsg` record
- Exposed constructor and additional methods for DBN records and `Metadata` to Python
- Made `RecordRef` implement `Sync` and `Send`

### Breaking changes
- Introduced separate rtypes for each OHLCV schema
- Removed `record_count` from `Metadata`
- Changed serialization of `c_char` fields to strings instead of ints
- Renamed `dbn::RecordDecoder::decode_record` to `decode`
- Renamed `dbn::RecordDecoder::decode_record_ref` to `decode_ref`
- Renamed `HasRType::size` to `record_size` to avoid confusion with order size fields
- Stopped serializing `related` and `related_security_id` fields in `InstrumentDefMsg`

## 0.3.2 - 2023-03-01
### Enhancements
- Added records and `Metadata` as exports of `databento_dbn` Python package
- Improved how `Metadata` appears in Python and added `__repr__`

### Bug fixes
- Fixed bug where `dbn` CLI tool didn't truncate existing files

## 0.3.1 - 2023-02-27
### Enhancements
- Added improved Python bindings for decoding DBN
- Standardized documentation for `start`, `end`, and `limit`

### Bug fixes
- Fixed bug with `encode_metadata` Python function

## 0.3.0 - 2023-02-22
### Enhancements
- Added ability to migrate legacy DBZ to DBN through CLI
- Relaxed requirement that DBN be Zstandard-compressed
- Folded in `databento-defs`
- Added support for async encoding and decoding
- Added billable size calculation to `dbn` CLI
- Added `MetadataBuilder` to assist with defaults
- Refactored into encoder and decoder types

### Breaking changes
- Renamed DBZ to DBN
- Renamed python package to `databento-dbn`
- Moved metadata out of skippable frame

## 0.2.1 - 2022-12-02
- Added Python DBZ writing example
- Changed [databento-defs](https://crates.io/crates/databento-defs) dependency to crates.io version

## 0.2.0 - 2022-11-28
### Enhancements
- Added interface for writing DBZ files
- Enabled Zstd checksums
- Changed DBZ decoding to use [streaming-iterator](https://crates.io/crates/streaming-iterator)

### Breaking changes
- Changed JSON output to NDJSON

### Bug fixes
- Change nanosecond timestamps to strings in JSON to avoid loss of precision when parsing

## 0.1.5 - 2022-09-14
- Initial release
