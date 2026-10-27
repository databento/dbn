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

fn update_symbol_map<H: crate::record::RecordHeaderKind>(
    metadata: &crate::Metadata,
    symbol_map: &mut crate::TsSymbolMap,
    framing: crate::RecordRef<'_, H>,
) -> crate::Result<()> {
    use crate::v4::framing::{RecordGroupHeader, RecordGroupHeaderMsg};

    if framing.header().raw_rtype() != RecordGroupHeaderMsg::RTYPE {
        return Ok(());
    }
    let group = RecordGroupHeader::parse(framing.as_ref())?;
    symbol_map.retain_from(group.msg.start_ts);
    let is_inverse = metadata.is_inverse()?;
    let mut cur = crate::layout::wire::ByteReader::new(group.symbol_map);
    for interval in fsm::decode_symbol_intervals(&mut cur, group.msg.interval_count)? {
        symbol_map.insert_interval(&interval?, is_inverse)?;
    }
    let padding = cur.remaining();
    if padding.len() >= 8 || padding.iter().any(|&b| b != 0) {
        return Err(crate::Error::decode(format!(
            "record group header has {} bytes after its {} symbol intervals, which isn't zero padding",
            padding.len(),
            group.msg.interval_count
        )));
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::{
        decode::{dbn::fsm::DbnFsm, FromLittleEndianSlice},
        encode::dbn::v4::Encoder,
        v4, Dataset, Metadata, Result, SType, Schema, SymbolInterval, METADATA_FIXED_LEN,
        METADATA_PRELUDE_LEN,
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

    pub const DAY: u64 = 86_400_000_000_000;
    pub const ESZ3_ID: u64 = 5482;
    pub const NQZ3_ID: u64 = 42140870;

    pub fn symbol_interval(
        start_ts: u64,
        end_ts: u64,
        instrument_id: u64,
        raw_symbol: &str,
    ) -> SymbolInterval {
        SymbolInterval {
            start_ts,
            end_ts,
            instrument_id,
            stype_in_symbol: raw_symbol.to_owned(),
            stype_out_symbol: instrument_id.to_string(),
        }
    }

    pub fn trade_v4_at(instrument_id: u64, ts_recv: u64) -> v4::TradeMsg {
        let mut trade = trade_v4();
        trade.hd.instrument_id = instrument_id;
        trade.ts_recv = ts_recv;
        trade
    }

    pub struct RecordGroupStream {
        pub encoded: Vec<u8>,
        pub second_group_pos: usize,
        pub trades_with_symbols: Vec<(v4::TradeMsg, Option<&'static str>)>,
    }

    pub fn record_group_stream() -> Result<RecordGroupStream> {
        let day1 = metadata_v4(Schema::Trades).start;
        let day2 = day1 + DAY;
        let mut metadata = metadata_v4(Schema::Trades);
        metadata.intervals = vec![symbol_interval(day1, day2, ESZ3_ID, "ESZ3")];
        let trades = vec![
            (trade_v4_at(ESZ3_ID, day1 + 1), Some("ESZ3")),
            (trade_v4_at(ESZ3_ID, day2 + 1), Some("ESH4")),
            (trade_v4_at(NQZ3_ID, day2 + 2), Some("NQZ3")),
            (trade_v4_at(7, day2 + 3), None),
        ];
        let mut encoded = Vec::new();
        let mut encoder = Encoder::new(&mut encoded, &metadata)?;
        encoder.encode_record_group_header(day1, &[])?;
        encoder.encode(&trades[0].0)?;
        let second_group_pos = encoder.get_ref().len();
        encoder.encode_record_group_header(
            day2,
            &[
                symbol_interval(day2, day2 + DAY, ESZ3_ID, "ESH4"),
                symbol_interval(day2, day2 + DAY, NQZ3_ID, "NQZ3"),
            ],
        )?;
        for (trade, _) in &trades[1..] {
            encoder.encode(trade)?;
        }
        Ok(RecordGroupStream {
            encoded,
            second_group_pos,
            trades_with_symbols: trades,
        })
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
