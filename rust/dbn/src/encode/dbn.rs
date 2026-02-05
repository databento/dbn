//! Encoding DBN records into DBN, Zstandard-compressed or not.

/// Maximum number of `IoSlice`s to use in a single vectored write call.
const MAX_SLICES: usize = 128;

/// Increment added to a record's header `length` field (in `LENGTH_MULTIPLIER` units)
/// when appending a `ts_out` timestamp.
const TS_OUT_LEN: u8 = (std::mem::size_of::<u64>() / crate::RecordHeader::LENGTH_MULTIPLIER) as u8;

mod sync;
pub use sync::{Encoder, MetadataEncoder, RecordEncoder};

#[cfg(feature = "async")]
mod r#async;
#[cfg(feature = "async")]
pub use r#async::{
    Encoder as AsyncEncoder, MetadataEncoder as AsyncMetadataEncoder,
    RecordEncoder as AsyncRecordEncoder,
};
