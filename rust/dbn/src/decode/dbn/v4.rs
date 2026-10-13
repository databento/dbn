//! Decoding DBN version 4 streams.
//!
//! Construct these decoders with `with_fsm()` and a [`DbnFsm`](super::fsm::DbnFsm)
//! built with `header::<v4::RecordHeader>()`.

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
