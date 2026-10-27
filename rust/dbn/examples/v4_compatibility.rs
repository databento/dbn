//! Forward and backward compatibility of DBN version 4 record layouts.
//!
//! A v4 stream's metadata carries the layout of every record type it holds. A
//! [`LayoutIndex`] compares those layouts against the structs this build compiled. A
//! build is forward compatible with streams from newer producers and backward
//! compatible with streams from older ones:
//!
//! 1. Forward compatibility: a newer producer grew the trades base and appended an
//!    extension field. This build still casts to its `TradeMsg` and reads both new
//!    fields by ID.
//! 2. Dynamic access: one accessor per field across every rtype in a stream, including
//!    an rtype with no compiled struct, next to what the same accessors take in DBN
//!    version 3.

use std::{ffi::c_char, mem::size_of};

use dbn::{
    decode::{
        dbn::{fsm::DbnFsm, v4::Decoder},
        DbnMetadata,
    },
    encode::dbn::v4::Encoder,
    layout::{DynFieldAccess, LayoutIndex, RecordLayout, StreamLayout, StreamLayoutBuilder},
    pretty::Px,
    rtype_dispatch,
    v4::{
        self,
        fields::{self, Field},
        layouts::default_layout,
        rtype,
        types::{Char, Decimal},
        MboMsg, RecordHeader, TradeMsg,
    },
    Action, Dataset, FlagSet, HasRType, Metadata, RType, Record, SType, Schema, Side,
};

const TS_EVENT: u64 = 1_609_160_400_000_000_000; // 2020-12-28T13:00:00Z
const TS_RECV: u64 = TS_EVENT + 1_500;
const TS_OUT: u64 = TS_RECV + 12_000;
const SYMBOL_CSTR_LEN: u16 = dbn::SYMBOL_CSTR_LEN as u16;
// A repacked layout takes a new rtype. This one carries fractional sizes.
const RTYPE_FRACTIONAL_TRADES: u16 = 0x01A0;

fn main() -> dbn::Result<()> {
    forward_compatible()?;
    dynamic_access()
}

fn forward_compatible() -> dbn::Result<()> {
    println!("\n== Forward compatibility: the producer is newer ==");

    // A later release moved `channel_id` into the trades base and populates OPRA's
    // last-sale `sale_condition` as an extension field. Growing the base keeps the rtype.
    let mut sb = StreamLayoutBuilder::new(1, SYMBOL_CSTR_LEN);
    let mut trades = sb.layout_of::<TradeMsg>(rtype::MBP_0);
    trades.field(fields::CHANNEL_ID, "channel_id");
    trades.finish_base();
    trades.field(fields::SALE_CONDITION, "sale_condition");
    trades.finish();
    let layout = sb.build()?;
    let declared = layout.record_layout(rtype::MBP_0).unwrap().clone();

    let mut trade = v4::RecordBuf::from(sample_trade());
    write_record(
        &mut trade,
        &declared,
        &[
            (fields::CHANNEL_ID.id(), &[3]),
            (fields::SALE_CONDITION.id(), b"@"),
        ],
    );
    let encoded = encode(layout, false, &[trade.as_rec_ref()])?;
    let mut decoder = decoder(&encoded)?;
    let index = decoder.metadata().layout_index().unwrap();
    let rec = decoder.decode_ref()?.unwrap();

    // The hash changed, but every field the compiled struct names is at the same offset,
    // size, and type, and the wider base clears the cast's length check
    let compiled = default_layout().record_layout(rtype::MBP_0).unwrap();
    assert_ne!(declared.layout_hash, compiled.layout_hash);
    println!(
        "  hash {:#018X} != compiled {:#018X}, verdict {:?}",
        declared.layout_hash,
        compiled.layout_hash,
        index.verdict(rtype::MBP_0).unwrap()
    );

    // Bytes past `size_of::<TradeMsg>()` are invisible to the cast
    let base = rec.try_get::<TradeMsg>()?;
    assert_eq!(base.price, sample_trade().price);

    let channel_id = rec.field(&index, fields::CHANNEL_ID).unwrap();
    let sale_condition = rec.field(&index, fields::SALE_CONDITION).unwrap();
    assert_eq!(channel_id, 3);
    assert_eq!(sale_condition, Char(b'@' as c_char));
    println!(
        "  record {} bytes, base {} (compiled {})",
        rec.record_size(),
        declared.base_record_size,
        size_of::<TradeMsg>(),
    );
    println!(
        "  base cast price {}, grown base channel_id {channel_id}, extension sale_condition {:?}",
        Px(base.price),
        sale_condition.0 as u8 as char
    );
    Ok(())
}

fn ts_out(rec: v4::RecordRef, index: &LayoutIndex) -> Option<u64> {
    rec.field(index, fields::TS_OUT).map(|t| t.0)
}

fn price(rec: v4::RecordRef, index: &LayoutIndex) -> Option<Decimal> {
    rec.field(index, fields::PRICE)
}

fn action(rec: v4::RecordRef, index: &LayoutIndex) -> Option<Action> {
    rec.field(index, fields::ACTION)?.get().ok()
}

// Needs `Metadata::ts_out` checked out of band, a dispatch arm for every rtype, and
// `record_size()` guards to choose between the v1, v2, and v3 versions of a struct
fn ts_out_v3(rec: dbn::RecordRef) -> Option<u64> {
    fn take_ts_out<T: HasRType>(rec: &dbn::WithTsOut<T>) -> u64 {
        rec.ts_out
    }
    rtype_dispatch!(rec, ts_out: true, take_ts_out()).ok()
}

// No generic price accessor exists before v4, so it takes an arm per rtype
fn price_v3(rec: dbn::RecordRef) -> Option<i64> {
    match rec.rtype().ok()? {
        RType::Mbp0 => Some(rec.get::<dbn::TradeMsg>()?.price),
        RType::Mbp1 => Some(rec.get::<dbn::Mbp1Msg>()?.price),
        RType::Mbp10 => Some(rec.get::<dbn::Mbp10Msg>()?.price),
        RType::Mbo => Some(rec.get::<dbn::MboMsg>()?.price),
        RType::Cmbp1 | RType::Tcbbo => Some(rec.get::<dbn::Cmbp1Msg>()?.price),
        RType::Bbo1S | RType::Bbo1M => Some(rec.get::<dbn::BboMsg>()?.price),
        RType::Cbbo1S | RType::Cbbo1M => Some(rec.get::<dbn::CbboMsg>()?.price),
        // ...plus imbalance and statistics
        _ => None,
    }
}

fn dynamic_access() -> dbn::Result<()> {
    println!("\n== Fully dynamic access, no rtype dispatch ==");

    // MBO, trades, and a repacked trades variant with no compiled struct, no `RType`
    // variant, and no dispatch arm anywhere, each with `ts_out` as an extension
    let mut sb = StreamLayoutBuilder::new(1, SYMBOL_CSTR_LEN);
    let mut mbo = sb.layout_of::<MboMsg>(rtype::MBO);
    mbo.finish_base();
    mbo.field(fields::TS_OUT, "ts_out");
    mbo.finish();
    let mut trades = sb.layout_of::<TradeMsg>(rtype::MBP_0);
    trades.finish_base();
    trades.field(fields::TS_OUT, "ts_out");
    trades.finish();
    let mut fractional = sb.record(
        RTYPE_FRACTIONAL_TRADES,
        Schema::Trades as u16,
        "FractionalTradeMsg",
    );
    fractional
        .field(fields::PRICE, "price")
        .field(Field::<Decimal>::new(fields::SIZE.id()), "size")
        .field(fields::ACTION, "action")
        .field(fields::SIDE, "side")
        .field(fields::FLAGS, "flags")
        .field(fields::TS_RECV, "ts_recv")
        .field(fields::TS_IN_DELTA, "ts_in_delta")
        .field(fields::SEQUENCE, "sequence")
        .finish_base();
    fractional.field(fields::TS_OUT, "ts_out");
    fractional.finish();
    let layout = sb.build()?;

    let ts_out_value = TS_OUT.to_le_bytes();
    let with_ts_out = [(fields::TS_OUT.id(), ts_out_value.as_slice())];
    let mut mbo = v4::RecordBuf::from(sample_mbo());
    write_record(
        &mut mbo,
        layout.record_layout(rtype::MBO).unwrap(),
        &with_ts_out,
    );
    let mut trade = v4::RecordBuf::from(sample_trade());
    write_record(
        &mut trade,
        layout.record_layout(rtype::MBP_0).unwrap(),
        &with_ts_out,
    );
    let fractional = fractional_trade(&layout, ts_out_value);
    let encoded = encode(
        layout,
        true,
        &[
            mbo.as_rec_ref(),
            trade.as_rec_ref(),
            fractional.as_rec_ref(),
        ],
    )?;

    let mut decoder = decoder(&encoded)?;
    let index = decoder.metadata().layout_index().unwrap();
    println!(
        "  {:<6} {:>8}  {:>14}  {:>6}",
        "rtype", "ts_out", "price", "action"
    );
    while let Some(rec) = decoder.decode_ref()? {
        println!(
            "  {:#06X} {:>8}  {:>14}  {:>6}",
            rec.raw_rtype(),
            // As a delta from ts_recv to keep the column narrow
            ts_out(rec, &index).map_or("-".to_owned(), |t| (t - TS_RECV).to_string()),
            price(rec, &index).map_or("-".to_owned(), |p| Px(p.raw).to_string()),
            action(rec, &index).map_or("-".to_owned(), |a| (a as u8 as char).to_string()),
        );
        assert_eq!(ts_out(rec, &index), Some(TS_OUT));
        if rec.raw_rtype() == RTYPE_FRACTIONAL_TRADES {
            assert_eq!(price(rec, &index).unwrap().raw, sample_trade().price);
            assert_eq!(action(rec, &index), Some(Action::Trade));
        }
    }

    let legacy = legacy_trade_with_ts_out();
    let legacy_rec = dbn::RecordRef::from(&legacy);
    assert_eq!(ts_out_v3(legacy_rec), Some(TS_OUT));
    assert_eq!(price_v3(legacy_rec), Some(sample_trade().price));
    println!(
        "  v3: ts_out_v3 {:?}, price_v3 {:?}",
        ts_out_v3(legacy_rec).map(|t| t - TS_RECV),
        price_v3(legacy_rec).map(Px)
    );
    Ok(())
}

fn encode(layout: StreamLayout, ts_out: bool, records: &[v4::RecordRef]) -> dbn::Result<Vec<u8>> {
    let metadata = Metadata::builder()
        .version(v4::DBN_VERSION)
        .dataset(Dataset::OpraPillar)
        .schema(Some(Schema::Trades))
        .start(TS_EVENT)
        .stype_in(Some(SType::RawSymbol))
        .stype_out(SType::InstrumentId)
        .ts_out(ts_out)
        .layout(layout)
        .build();
    let mut encoded = Vec::new();
    Encoder::new(&mut encoded, &metadata)?.encode_refs(records)?;
    Ok(encoded)
}

fn decoder(encoded: &[u8]) -> dbn::Result<Decoder<&[u8]>> {
    Decoder::with_fsm(encoded, DbnFsm::builder().header::<RecordHeader>().build()?)
}

// Sets the header length to the layout's record size and writes `values` at the
// offsets the layout declares, as a producer populating fields past the compiled
// struct does
fn write_record(rec: &mut v4::RecordBuf, layout: &RecordLayout, values: &[(u16, &[u8])]) {
    let raw = rec.raw_buf_mut();
    let length = layout.record_size() - size_of::<RecordHeader>() as u16;
    raw[..2].copy_from_slice(&length.to_le_bytes());
    for &(field_id, value) in values {
        let at = usize::from(
            layout
                .field_defs
                .iter()
                .find(|f| f.field_id == field_id)
                .expect("declared field")
                .offset,
        );
        raw[at..][..value.len()].copy_from_slice(value);
    }
}

fn fractional_trade(layout: &StreamLayout, ts_out: [u8; 8]) -> v4::RecordBuf {
    let trade = sample_trade();
    let mut rec = v4::RecordBuf::from(TradeMsg {
        hd: RecordHeader::new::<TradeMsg>(RTYPE_FRACTIONAL_TRADES, 1, 42_000, TS_EVENT),
        ..Default::default()
    });
    rec.raw_buf_mut()[size_of::<RecordHeader>()..].fill(0);
    write_record(
        &mut rec,
        layout.record_layout(RTYPE_FRACTIONAL_TRADES).unwrap(),
        &[
            (fields::PRICE.id(), &trade.price.to_le_bytes()),
            // Widened in place: a fractional Fixed where default TradeMsg has a u32
            (fields::SIZE.id(), &i64::from(trade.size).to_le_bytes()),
            (fields::ACTION.id(), &[trade.action as u8]),
            (fields::SIDE.id(), &[trade.side as u8]),
            (fields::FLAGS.id(), &[trade.flags.raw()]),
            (fields::TS_RECV.id(), &trade.ts_recv.to_le_bytes()),
            (fields::TS_IN_DELTA.id(), &trade.ts_in_delta.to_le_bytes()),
            (fields::SEQUENCE.id(), &trade.sequence.to_le_bytes()),
            (fields::TS_OUT.id(), &ts_out),
        ],
    );
    rec
}

fn sample_trade() -> TradeMsg {
    TradeMsg {
        hd: RecordHeader::new::<TradeMsg>(rtype::MBP_0, 1, 42_000, TS_EVENT),
        price: 4_215_500_000_000,
        size: 7,
        action: Action::Trade as c_char,
        side: Side::Bid as c_char,
        flags: FlagSet::empty(),
        _reserved: 0,
        ts_recv: TS_RECV,
        ts_in_delta: 1_200,
        sequence: 88,
    }
}

fn sample_mbo() -> MboMsg {
    MboMsg {
        hd: RecordHeader::new::<MboMsg>(rtype::MBO, 1, 42_000, TS_EVENT),
        order_id: 90_112,
        price: 4_215_250_000_000,
        size: 3,
        flags: FlagSet::empty(),
        channel_id: 0,
        action: Action::Add as c_char,
        side: Side::Ask as c_char,
        ts_recv: TS_RECV,
        ts_in_delta: 900,
        sequence: 87,
    }
}

fn legacy_trade_with_ts_out() -> dbn::WithTsOut<dbn::TradeMsg> {
    let trade = sample_trade();
    dbn::WithTsOut::new(
        dbn::TradeMsg {
            hd: dbn::RecordHeader::new::<dbn::TradeMsg>(rtype::MBP_0 as u8, 1, 42_000, TS_EVENT),
            price: trade.price,
            size: trade.size,
            action: trade.action,
            side: trade.side,
            flags: trade.flags,
            depth: 0,
            ts_recv: trade.ts_recv,
            ts_in_delta: trade.ts_in_delta,
            sequence: trade.sequence,
        },
        TS_OUT,
    )
}
