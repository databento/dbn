//! Decoding of DBN files.
pub(super) const DBN_PREFIX: &[u8] = b"DBN";
pub(super) const DBN_PREFIX_LEN: usize = DBN_PREFIX.len();

/// Returns `true` if `bytes` starts with valid uncompressed DBN.
pub fn starts_with_prefix(bytes: &[u8]) -> bool {
    bytes.len() > DBN_PREFIX_LEN && &bytes[..DBN_PREFIX_LEN] == DBN_PREFIX
}

#[doc(hidden)]
pub mod aligned_buffer;
mod sync;
pub(crate) use sync::decode_iso8601;
pub use sync::{Decoder, MetadataDecoder, RecordDecoder};
pub mod fsm;

#[cfg(feature = "async")]
mod r#async;
#[cfg(feature = "async")]
pub use r#async::{
    decode_metadata_with_fsm as async_decode_metadata_with_fsm,
    decode_record_ref_with_fsm as async_decode_record_ref_with_fsm, Decoder as AsyncDecoder,
    MetadataDecoder as AsyncMetadataDecoder, RecordDecoder as AsyncRecordDecoder,
};
