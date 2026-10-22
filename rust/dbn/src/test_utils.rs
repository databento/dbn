use std::os::raw::c_char;

use fallible_streaming_iterator::FallibleStreamingIterator;

use crate::{
    decode::{dbn::tests::metadata_v4, private::LastRecord, DecodeRecordRef},
    layout::{RecordLayoutDesc, StreamLayoutBuilder},
    v3,
    v4::{self, fields},
    BidAskPair, Error, FlagSet, Metadata, RecordRef, SType, Schema,
};

/// A testing shim to get a streaming iterator from a [`Vec`].
pub struct VecStream<T> {
    vec: Vec<T>,
    idx: isize,
}

impl<T> VecStream<T> {
    pub fn new(vec: Vec<T>) -> Self {
        // initialize at -1 because `advance()` is always called before
        // `get()`.
        Self { vec, idx: -1 }
    }
}

impl<T> FallibleStreamingIterator for VecStream<T> {
    type Item = T;
    type Error = Error;

    fn advance(&mut self) -> Result<(), Error> {
        self.idx += 1;
        Ok(())
    }

    fn get(&self) -> Option<&Self::Item> {
        self.vec.get(self.idx as usize)
    }
}

impl<T> DecodeRecordRef for VecStream<T>
where
    T: v3::HasRType,
{
    fn decode_record_ref(&mut self) -> crate::Result<Option<crate::RecordRef<'_>>> {
        self.idx += 1;
        let Some(rec) = self.vec.get(self.idx as usize) else {
            return Ok(None);
        };
        Ok(Some(RecordRef::from(rec)))
    }
}

impl<T> LastRecord for VecStream<T>
where
    T: v3::HasRType + AsRef<[u8]>,
{
    fn last_record(&self) -> Option<RecordRef<'_>> {
        if self.vec.is_empty() {
            None
        } else {
            Some(RecordRef::from(self.vec.get(self.idx as usize).unwrap()))
        }
    }
}

pub fn mbo_v4() -> v4::MboMsg {
    v4::MboMsg {
        hd: v4::RecordHeader::new::<v4::MboMsg>(v4::rtype::MBO, 1, 5482, 1_697_240_529_000_000_000),
        order_id: 16,
        price: 5500,
        size: 3,
        flags: FlagSet::empty(),
        channel_id: 14,
        action: b'A' as c_char,
        side: b'B' as c_char,
        ts_recv: 1_697_240_529_000_000_100,
        ts_in_delta: 22_000,
        sequence: 1_002_375,
    }
}

pub fn mbo_v3() -> v3::MboMsg {
    let v4 = mbo_v4();
    v3::MboMsg {
        hd: v3::RecordHeader::new::<v3::MboMsg>(
            crate::rtype::MBO,
            v4.hd.publisher_id,
            u32::try_from(v4.hd.instrument_id).unwrap(),
            v4.hd.ts_event,
        ),
        order_id: v4.order_id,
        price: v4.price,
        size: v4.size,
        flags: v4.flags,
        channel_id: v4.channel_id,
        action: v4.action,
        side: v4.side,
        ts_recv: v4.ts_recv,
        ts_in_delta: v4.ts_in_delta,
        sequence: v4.sequence,
    }
}

pub fn trade_v4() -> v4::TradeMsg {
    v4::TradeMsg {
        hd: v4::RecordHeader::new::<v4::TradeMsg>(
            v4::rtype::MBP_0,
            1,
            5482,
            1_697_240_529_000_000_000,
        ),
        price: 5500,
        size: 7,
        action: b'T' as c_char,
        side: b'B' as c_char,
        flags: FlagSet::empty(),
        _reserved: 0,
        ts_recv: 1_697_240_529_000_000_100,
        ts_in_delta: 9,
        sequence: 88,
    }
}

pub fn as_ref(buf: &[u64]) -> RecordRef<'_, v4::RecordHeader> {
    // SAFETY: `buf` holds a whole record starting with an aligned v4 header whose
    // `length` covers the rest of `buf`.
    unsafe { RecordRef::unchecked_from_header(buf.as_ptr().cast()) }
}

pub fn mbp10_v4() -> v4::Mbp10Msg {
    v4::Mbp10Msg {
        hd: v4::RecordHeader::new::<v4::Mbp10Msg>(
            v4::rtype::MBP_10,
            1,
            5482,
            1_697_240_529_000_000_000,
        ),
        price: crate::UNDEF_PRICE,
        size: 3,
        action: b'A' as c_char,
        side: 0,
        flags: FlagSet::empty().set_last(),
        depth: 4,
        ts_recv: crate::UNDEF_TIMESTAMP,
        ts_in_delta: -5,
        sequence: 12,
        levels: levels(),
    }
}

pub fn mbp10_v3() -> v3::Mbp10Msg {
    let v4 = mbp10_v4();
    v3::Mbp10Msg {
        hd: v3::RecordHeader::new::<v3::Mbp10Msg>(
            crate::rtype::MBP_10,
            1,
            5482,
            1_697_240_529_000_000_000,
        ),
        price: v4.price,
        size: v4.size,
        action: v4.action,
        side: v4.side,
        flags: v4.flags,
        depth: v4.depth,
        ts_recv: v4.ts_recv,
        ts_in_delta: v4.ts_in_delta,
        sequence: v4.sequence,
        levels: levels(),
    }
}

pub fn symbol_mapping_v4() -> Vec<u64> {
    let mapping = symbol_mapping();
    let layout = v4::layouts::default_layout();
    let rec = layout.record_layout(v4::rtype::SYMBOL_MAPPING).unwrap();
    let mut bytes = vec![0; usize::from(rec.record_size())];
    bytes[..size_of::<v4::SymbolMappingMsg>()].copy_from_slice(RecordRef::from(&mapping).as_ref());
    // v4 `length` counts the bytes after the header
    let length = u16::try_from(bytes.len() - size_of::<v4::RecordHeader>()).unwrap();
    bytes[..2].copy_from_slice(&length.to_le_bytes());
    for (field, symbol) in [
        (fields::STYPE_IN_SYMBOL, "ESZ6"),
        (fields::STYPE_OUT_SYMBOL, "5482"),
    ] {
        let def = rec
            .field_defs
            .iter()
            .find(|f| f.field_id == field.id())
            .unwrap();
        let start = usize::from(def.offset);
        bytes[start..start + symbol.len()].copy_from_slice(symbol.as_bytes());
    }
    aligned(&bytes)
}

pub fn symbol_mapping_v3() -> v3::SymbolMappingMsg {
    let mapping = symbol_mapping();
    v3::SymbolMappingMsg::new(
        5482,
        7,
        SType::RawSymbol,
        "ESZ6",
        SType::InstrumentId,
        "5482",
        mapping.start_ts,
        mapping.end_ts,
    )
    .unwrap()
}

pub fn trades_with_sale_condition<T: fields::FixedWidth>() -> Metadata {
    let mut sb = StreamLayoutBuilder::new(1, crate::compat::SYMBOL_CSTR_LEN_V3 as u16);
    {
        let mut r = sb.record(v4::rtype::MBP_0, Schema::Trades as u16, v4::TradeMsg::NAME);
        v4::TradeMsg::add_fields(&mut r);
        r.finish_base();
        r.field(
            fields::Field::<T>::new(fields::SALE_CONDITION.id()),
            "sale_condition",
        );
        r.finish();
    }
    let mut metadata = metadata_v4(Schema::Trades);
    metadata.layout = Some(sb.build().unwrap());
    metadata
}

pub fn trade_v4_with_sale_condition(metadata: &Metadata, sale_condition: &[u8]) -> Vec<u64> {
    let rec = &metadata.layout.as_ref().unwrap().record_layouts[0];
    let trade = trade_v4();
    let mut bytes = vec![0; usize::from(rec.record_size())];
    bytes[..size_of::<v4::TradeMsg>()].copy_from_slice(RecordRef::from(&trade).as_ref());
    let length = u16::try_from(bytes.len() - size_of::<v4::RecordHeader>()).unwrap();
    bytes[..2].copy_from_slice(&length.to_le_bytes());
    let start = size_of::<v4::TradeMsg>();
    bytes[start..start + sale_condition.len()].copy_from_slice(sale_condition);
    aligned(&bytes)
}

pub fn trade_v3() -> v3::TradeMsg {
    let v4 = trade_v4();
    v3::TradeMsg {
        hd: v3::RecordHeader::new::<v3::TradeMsg>(
            crate::rtype::MBP_0,
            v4.hd.publisher_id,
            u32::try_from(v4.hd.instrument_id).unwrap(),
            v4.hd.ts_event,
        ),
        price: v4.price,
        size: v4.size,
        action: v4.action,
        side: v4.side,
        flags: v4.flags,
        depth: 0,
        ts_recv: v4.ts_recv,
        ts_in_delta: v4.ts_in_delta,
        sequence: v4.sequence,
    }
}

/// [`trade_v3()`] upgraded to v4, which keeps v3's `depth` as an extension.
pub fn trade_v3_upgraded() -> v4::TradeMsg {
    let trade = trade_v4();
    let record_size = crate::layout::upgrade_layout(false)
        .record_layout(v4::rtype::MBP_0)
        .unwrap()
        .record_size();
    v4::TradeMsg {
        hd: v4::RecordHeader::with_record_size(
            trade.hd.rtype,
            trade.hd.publisher_id,
            trade.hd.instrument_id,
            trade.hd.ts_event,
            usize::from(record_size),
        )
        .unwrap(),
        ..trade
    }
}

fn levels() -> [BidAskPair; 10] {
    std::array::from_fn(|i| BidAskPair {
        bid_px: 5_000_000_000 - i as i64 * 250_000_000,
        ask_px: if i == 9 {
            crate::UNDEF_PRICE
        } else {
            5_250_000_000 + i as i64 * 250_000_000
        },
        bid_sz: 10 + i as u32,
        ask_sz: 20 + i as u32,
        bid_ct: i as u32,
        ask_ct: 2 * i as u32,
    })
}

fn symbol_mapping() -> v4::SymbolMappingMsg {
    v4::SymbolMappingMsg {
        hd: v4::RecordHeader::new::<v4::SymbolMappingMsg>(v4::rtype::SYMBOL_MAPPING, 0, 5482, 7),
        start_ts: 1_697_155_200_000_000_000,
        end_ts: 1_697_241_600_000_000_000,
        stype_in: SType::RawSymbol as u8,
        stype_out: SType::InstrumentId as u8,
        _reserved: [0; 6],
    }
}

fn aligned(bytes: &[u8]) -> Vec<u64> {
    bytes
        .chunks(8)
        .map(|chunk| u64::from_le_bytes(chunk.try_into().unwrap()))
        .collect()
}
