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
pub mod v4;

#[cfg(feature = "async")]
mod r#async;
#[cfg(feature = "async")]
#[expect(deprecated)]
pub use r#async::decode_record_ref_with_fsm as async_decode_record_ref_with_fsm;
#[cfg(feature = "async")]
pub use r#async::{
    decode_metadata_with_fsm as async_decode_metadata_with_fsm, Decoder as AsyncDecoder,
    MetadataDecoder as AsyncMetadataDecoder, RecordDecoder as AsyncRecordDecoder,
};

#[cfg(test)]
pub(crate) mod tests {
    use crate::{
        decode::{dbn::fsm::DbnFsm, FromLittleEndianSlice},
        encode::dbn::v4::Encoder,
        v4, Dataset, Metadata, Result, SType, Schema, METADATA_FIXED_LEN, METADATA_PRELUDE_LEN,
    };

    pub use crate::test_utils::{mbo_v4, trade_v3, trade_v3_upgraded, trade_v4};

    pub fn metadata_v4(schema: Schema) -> Metadata {
        Metadata::builder()
            .version(v4::DBN_VERSION)
            .dataset(Dataset::GlbxMdp3)
            .schema(Some(schema))
            .start(1697240529000000000)
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .layout(v4::layouts::default_layout().clone())
            .build()
    }

    pub fn fsm_v4() -> Result<DbnFsm<v4::RecordHeader>> {
        DbnFsm::builder().header::<v4::RecordHeader>().build()
    }

    /// Returns a version 4 stream of three [`trade_v4()`] records.
    pub fn trades_v4() -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        let mut encoder = Encoder::new(&mut encoded, &metadata_v4(Schema::Trades))?;
        for _ in 0..3 {
            encoder.encode(&trade_v4())?;
        }
        Ok(encoded)
    }

    /// Replaces the stream layout section of the version 4 stream `encoded` with
    /// `layout`, bypassing the encoder's validation.
    pub fn replace_layout(encoded: &[u8], layout: &[u8]) -> Vec<u8> {
        const LAYOUT_LEN_POS: usize = METADATA_PRELUDE_LEN + METADATA_FIXED_LEN;
        const LAYOUT_POS: usize = LAYOUT_LEN_POS + size_of::<u32>();
        let old_len = u32::from_le_slice(&encoded[LAYOUT_LEN_POS..]);
        let metadata_len = u32::from_le_slice(&encoded[4..]);
        let new_len = layout.len() as u32;
        let mut replaced = encoded[..LAYOUT_LEN_POS].to_vec();
        replaced[4..8].copy_from_slice(&(metadata_len - old_len + new_len).to_le_bytes());
        replaced.extend_from_slice(&new_len.to_le_bytes());
        replaced.extend_from_slice(layout);
        replaced.extend_from_slice(&encoded[LAYOUT_POS + old_len as usize..]);
        replaced
    }
}
