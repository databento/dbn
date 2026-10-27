//! Decoding DBN version 4 streams.
//!
//! Construct these decoders with `with_fsm()` and a [`DbnFsm`](super::fsm::DbnFsm)
//! built with `header::<v4::RecordHeader>()`.
//!
//! # Examples
//! Decoding a version 3 stream upgrades it to version 4, where `ts_out` is a field in
//! the stream layout instead of a [`WithTsOut`](crate::WithTsOut) wrapper:
//!
//! ```
//! use dbn::{
//!     decode::{
//!         dbn::{fsm::DbnFsm, v4::Decoder},
//!         DbnMetadata,
//!     },
//!     encode::{dbn::Encoder, EncodeRecord},
//!     layout::DynFieldAccess,
//!     v4::{self, fields},
//!     Dataset, Metadata, SType, Schema, TradeMsg, WithTsOut,
//! };
//!
//! const TS_OUT: u64 = 1_609_160_400_000_013_500;
//! const PRICE: i64 = 4_215_500_000_000;
//!
//! let metadata = Metadata::builder()
//!     .dataset(Dataset::XnasItch)
//!     .schema(Some(Schema::Trades))
//!     .start(1_609_160_400_000_000_000)
//!     .stype_in(Some(SType::RawSymbol))
//!     .stype_out(SType::InstrumentId)
//!     .ts_out(true)
//!     .build();
//! let trade = TradeMsg {
//!     price: PRICE,
//!     ..Default::default()
//! };
//! let mut encoded = Vec::new();
//! Encoder::new(&mut encoded, &metadata)?.encode_record(&WithTsOut::new(trade, TS_OUT))?;
//!
//! let fsm = DbnFsm::builder().header::<v4::RecordHeader>().build()?;
//! let mut decoder = Decoder::with_fsm(encoded.as_slice(), fsm)?;
//! assert_eq!(decoder.metadata().version, v4::DBN_VERSION);
//! let index = decoder.metadata().layout_index().unwrap();
//! let rec = decoder.decode_ref()?.unwrap();
//!
//! assert_eq!(rec.try_get::<v4::TradeMsg>()?.price, PRICE);
//! assert_eq!(rec.field(&index, fields::TS_OUT).unwrap().0, TS_OUT);
//! # Ok::<_, dbn::Error>(())
//! ```

use crate::v4::RecordHeader;

/// A decoder for DBN version 4 metadata and records.
pub type Decoder<R> = super::Decoder<R, RecordHeader>;
/// A decoder for DBN version 4 records without metadata.
pub type RecordDecoder<R> = super::RecordDecoder<R, RecordHeader>;
/// A decoder for DBN version 4 metadata.
pub type MetadataDecoder<R> = super::MetadataDecoder<R, RecordHeader>;

/// An async decoder for DBN version 4 metadata and records.
#[cfg(feature = "async")]
pub type AsyncDecoder<R> = super::AsyncDecoder<R, RecordHeader>;
/// An async decoder for DBN version 4 records without metadata.
#[cfg(feature = "async")]
pub type AsyncRecordDecoder<R> = super::AsyncRecordDecoder<R, RecordHeader>;
/// An async decoder for DBN version 4 metadata.
#[cfg(feature = "async")]
pub type AsyncMetadataDecoder<R> = super::AsyncMetadataDecoder<R, RecordHeader>;
