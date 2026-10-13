// `OHLCV_DEPRECATED` is still a compiled rtype
#![allow(deprecated)]

use std::{
    mem::{offset_of, size_of},
    os::raw::c_char,
};

use rstest::rstest;

use super::{fields, layouts::default_layout, rtype, *};
use crate::{
    layout::{DynFieldAccess, LayoutIndex, Verdict},
    record::str_to_c_chars,
    v4::types::Decimal,
    ErrorCode, StatType, StatusAction, SystemCode,
};

/// `(registry ID, offset)` pairs for the named fields of a struct.
macro_rules! expected_fields {
    ($rec:ty { $($id:ident: $field:ident),* $(,)? }) => {
        vec![$((fields::$id.id(), offset_of!($rec, $field))),*]
    };
}

#[rstest]
#[case::mbo(rtype::MBO, size_of::<MboMsg>())]
#[case::trades(rtype::MBP_0, size_of::<TradeMsg>())]
#[case::mbp1(rtype::MBP_1, size_of::<Mbp1Msg>())]
#[case::tbbo(rtype::TBBO, size_of::<Mbp1Msg>())]
#[case::mbp10(rtype::MBP_10, size_of::<Mbp10Msg>())]
#[case::bbo_1s(rtype::BBO_1S, size_of::<BboMsg>())]
#[case::bbo_1m(rtype::BBO_1M, size_of::<BboMsg>())]
#[case::cmbp1(rtype::CMBP_1, size_of::<Cmbp1Msg>())]
#[case::tcbbo(rtype::TCBBO, size_of::<Cmbp1Msg>())]
#[case::cbbo_1s(rtype::CBBO_1S, size_of::<CbboMsg>())]
#[case::cbbo_1m(rtype::CBBO_1M, size_of::<CbboMsg>())]
#[case::ohlcv_1s(rtype::OHLCV_1S, size_of::<OhlcvMsg>())]
#[case::ohlcv_1m(rtype::OHLCV_1M, size_of::<OhlcvMsg>())]
#[case::ohlcv_1h(rtype::OHLCV_1H, size_of::<OhlcvMsg>())]
#[case::ohlcv_1d(rtype::OHLCV_1D, size_of::<OhlcvMsg>())]
#[case::ohlcv_eod(rtype::OHLCV_EOD, size_of::<OhlcvMsg>())]
#[case::ohlcv_deprecated(rtype::OHLCV_DEPRECATED, size_of::<OhlcvMsg>())]
#[case::status(rtype::STATUS, size_of::<StatusMsg>())]
#[case::imbalance(rtype::IMBALANCE, size_of::<ImbalanceMsg>())]
#[case::statistics(rtype::STATISTICS, size_of::<StatMsg>())]
#[case::error(rtype::ERROR, size_of::<ErrorMsg>())]
#[case::system(rtype::SYSTEM, size_of::<SystemMsg>())]
fn default_layout_base_matches_struct(#[case] rtype: u16, #[case] size: usize) {
    let rec = default_layout().record_layout(rtype).unwrap();
    assert_eq!(usize::from(rec.base_record_size), size);
}

#[test]
fn default_layout_holds_one_layout_per_rtype() {
    assert_eq!(default_layout().record_layouts.len(), 24);
}

#[test]
fn default_layout_is_compiled_for_every_rtype() {
    let layout = default_layout();
    let index = LayoutIndex::new(layout, layout);
    for rec in &layout.record_layouts {
        assert_eq!(
            index.verdict(rec.rtype),
            Some(Verdict::Compiled),
            "{}",
            rec.name
        );
    }
}

#[rstest]
#[case::mbo(&[rtype::MBO], expected_fields!(MboMsg {
    ORDER_ID: order_id, PRICE: price, SIZE: size, FLAGS: flags, CHANNEL_ID: channel_id,
    ACTION: action, SIDE: side, TS_RECV: ts_recv, TS_IN_DELTA: ts_in_delta,
    SEQUENCE: sequence,
}))]
#[case::trades(&[rtype::MBP_0], expected_fields!(TradeMsg {
    PRICE: price, SIZE: size, ACTION: action, SIDE: side, FLAGS: flags, TS_RECV: ts_recv,
    TS_IN_DELTA: ts_in_delta, SEQUENCE: sequence,
}))]
#[case::mbp1(&[rtype::MBP_1, rtype::TBBO], expected_fields!(Mbp1Msg {
    PRICE: price, SIZE: size, ACTION: action, SIDE: side, FLAGS: flags, TS_RECV: ts_recv,
    TS_IN_DELTA: ts_in_delta, SEQUENCE: sequence, LEVELS_1: levels,
}))]
#[case::mbp10(&[rtype::MBP_10], expected_fields!(Mbp10Msg {
    PRICE: price, SIZE: size, ACTION: action, SIDE: side, FLAGS: flags, DEPTH: depth,
    TS_RECV: ts_recv, TS_IN_DELTA: ts_in_delta, SEQUENCE: sequence, LEVELS_10: levels,
}))]
#[case::bbo(&[rtype::BBO_1S, rtype::BBO_1M], expected_fields!(BboMsg {
    PRICE: price, SIZE: size, SIDE: side, FLAGS: flags, TS_RECV: ts_recv,
    SEQUENCE: sequence, LEVELS_1: levels,
}))]
#[case::cmbp1(&[rtype::CMBP_1, rtype::TCBBO], expected_fields!(Cmbp1Msg {
    PRICE: price, SIZE: size, ACTION: action, SIDE: side, FLAGS: flags, TS_RECV: ts_recv,
    TS_IN_DELTA: ts_in_delta, CONSOLIDATED_LEVELS_1: levels,
}))]
#[case::cbbo(&[rtype::CBBO_1S, rtype::CBBO_1M], expected_fields!(CbboMsg {
    PRICE: price, SIZE: size, SIDE: side, FLAGS: flags, TS_RECV: ts_recv,
    CONSOLIDATED_LEVELS_1: levels,
}))]
#[case::ohlcv(
    &[
        rtype::OHLCV_1S,
        rtype::OHLCV_1M,
        rtype::OHLCV_1H,
        rtype::OHLCV_1D,
        rtype::OHLCV_EOD,
        rtype::OHLCV_DEPRECATED,
    ],
    expected_fields!(OhlcvMsg {
        OPEN: open, HIGH: high, LOW: low, CLOSE: close, VOLUME: volume,
    })
)]
#[case::status(&[rtype::STATUS], expected_fields!(StatusMsg {
    TS_RECV: ts_recv, STATUS_ACTION: action, REASON: reason, TRADING_EVENT: trading_event,
    IS_TRADING: is_trading, IS_QUOTING: is_quoting,
    IS_SHORT_SELL_RESTRICTED: is_short_sell_restricted,
}))]
#[case::definition(&[rtype::INSTRUMENT_DEF], expected_fields!(InstrumentDefMsg {
    TS_RECV: ts_recv, MIN_PRICE_INCREMENT: min_price_increment,
    DISPLAY_FACTOR: display_factor, RAW_INSTRUMENT_ID: raw_instrument_id,
    MIN_LOT_SIZE_ROUND_LOT: min_lot_size_round_lot,
    SECURITY_UPDATE_ACTION: security_update_action, INSTRUMENT_CLASS: instrument_class,
    MATCH_ALGORITHM: match_algorithm, CHANNEL_ID: channel_id,
}))]
#[case::imbalance(&[rtype::IMBALANCE], expected_fields!(ImbalanceMsg {
    TS_RECV: ts_recv, REF_PRICE: ref_price, CONT_BOOK_CLR_PRICE: cont_book_clr_price,
    AUCT_INTEREST_CLR_PRICE: auct_interest_clr_price, PAIRED_QTY: paired_qty,
    TOTAL_IMBALANCE_QTY: total_imbalance_qty, AUCTION_TYPE: auction_type, SIDE: side,
}))]
#[case::statistics(&[rtype::STATISTICS], expected_fields!(StatMsg {
    TS_RECV: ts_recv, TS_REF: ts_ref, PRICE: price, QUANTITY: quantity,
    SEQUENCE: sequence, TS_IN_DELTA: ts_in_delta, STAT_TYPE: stat_type,
    CHANNEL_ID: channel_id, UPDATE_ACTION: update_action, STAT_FLAGS: stat_flags,
}))]
#[case::error(&[rtype::ERROR], expected_fields!(ErrorMsg {
    SUBSCRIPTION_ID: subscription_id, SCHEMA: schema, ERROR_CODE: code, IS_LAST: is_last,
    ERR: err,
}))]
#[case::symbol_mapping(&[rtype::SYMBOL_MAPPING], expected_fields!(SymbolMappingMsg {
    START_TS: start_ts, END_TS: end_ts, STYPE_IN: stype_in, STYPE_OUT: stype_out,
}))]
#[case::system(&[rtype::SYSTEM], expected_fields!(SystemMsg {
    SUBSCRIPTION_ID: subscription_id, SCHEMA: schema, SYSTEM_CODE: code, MSG: msg,
}))]
fn default_layout_places_every_field(
    #[case] rtypes: &[u16],
    #[case] mut expected: Vec<(u16, usize)>,
) {
    expected.sort_unstable();
    for &rtype in rtypes {
        let rec = default_layout().record_layout(rtype).unwrap();
        let mut actual: Vec<_> = rec
            .field_defs
            .iter()
            .filter(|f| {
                !f.is_padding() && f.field_id != fields::HD.id() && f.type_id != fields::SYMBOL_ID
            })
            .map(|f| (f.field_id, usize::from(f.offset)))
            .collect();
        actual.sort_unstable();
        assert_eq!(actual, expected, "{} {rtype:#06X}", rec.name);
    }
}

fn index() -> LayoutIndex {
    LayoutIndex::new(default_layout(), default_layout())
}

#[rstest]
#[case::mbp1(rtype::MBP_1, true, false)]
#[case::mbp10(rtype::MBP_10, false, true)]
fn levels_resolve_only_at_their_depth(
    #[case] rtype: u16,
    #[case] has_depth_1: bool,
    #[case] has_depth_10: bool,
) {
    let index = index();
    assert_eq!(fields::LEVELS_1.id(), fields::LEVELS_10.id());
    assert_eq!(
        index.offset_of(rtype, fields::LEVELS_1).is_some(),
        has_depth_1
    );
    assert_eq!(
        index.offset_of(rtype, fields::LEVELS_10).is_some(),
        has_depth_10
    );
}

#[test]
fn reads_ohlcv_close() {
    let ohlcv = OhlcvMsg {
        hd: RecordHeader::new::<OhlcvMsg>(rtype::OHLCV_1D, 1, 2, 3),
        open: 1,
        high: 4,
        low: 0,
        close: 3_250_000_000,
        volume: 10,
    };
    assert_eq!(
        ohlcv.field(&index(), fields::CLOSE),
        Some(Decimal {
            raw: 3_250_000_000,
            scale: -9
        })
    );
}

#[test]
fn reads_status_action() {
    let status = StatusMsg {
        hd: RecordHeader::new::<StatusMsg>(rtype::STATUS, 1, 2, 3),
        ts_recv: 4,
        action: StatusAction::PreOpen as u16,
        reason: 0,
        trading_event: 0,
        is_trading: b'Y' as c_char,
        is_quoting: b'~' as c_char,
        is_short_sell_restricted: b'N' as c_char,
        _reserved: [0; 7],
    };
    let action = status.field(&index(), fields::STATUS_ACTION).unwrap();
    assert_eq!(action.get().unwrap(), StatusAction::PreOpen);
}

#[test]
fn reads_error_code_and_message() {
    let error = ErrorMsg {
        hd: RecordHeader::new::<ErrorMsg>(rtype::ERROR, 0, 0, 3),
        subscription_id: 5,
        schema: u16::MAX,
        code: ErrorCode::AuthFailed as u8,
        is_last: 1,
        _reserved: [0; 2],
        err: str_to_c_chars("bad key").unwrap(),
    };
    let index = index();
    assert_eq!(
        error
            .field(&index, fields::ERROR_CODE)
            .unwrap()
            .get()
            .unwrap(),
        ErrorCode::AuthFailed
    );
    assert_eq!(error.field(&index, fields::ERR), Some("bad key"));
}

#[test]
fn reads_system_message() {
    let system = SystemMsg {
        hd: RecordHeader::new::<SystemMsg>(rtype::SYSTEM, 0, 0, 3),
        subscription_id: 5,
        schema: u16::MAX,
        code: SystemCode::SubscriptionAck as u8,
        _reserved: [0; 2],
        msg: str_to_c_chars("subscribed").unwrap(),
    };
    assert_eq!(system.field(&index(), fields::MSG), Some("subscribed"));
}

#[test]
fn reads_stat_type() {
    let stat = StatMsg {
        hd: RecordHeader::new::<StatMsg>(rtype::STATISTICS, 1, 2, 3),
        ts_recv: 4,
        ts_ref: 5,
        price: 6,
        quantity: 7,
        sequence: 8,
        ts_in_delta: 9,
        stat_type: StatType::SettlementPrice as u16,
        channel_id: 1,
        update_action: 1,
        stat_flags: 0,
        _reserved: [0; 3],
    };
    assert_eq!(
        stat.field(&index(), fields::STAT_TYPE)
            .unwrap()
            .get()
            .unwrap(),
        StatType::SettlementPrice
    );
}

#[rstest]
#[case::definition(
    rtype::INSTRUMENT_DEF,
    size_of::<InstrumentDefMsg>(),
    &[fields::RAW_SYMBOL]
)]
#[case::symbol_mapping(
    rtype::SYMBOL_MAPPING,
    size_of::<SymbolMappingMsg>(),
    &[fields::STYPE_IN_SYMBOL, fields::STYPE_OUT_SYMBOL]
)]
fn required_symbols_follow_the_struct(
    #[case] rtype: u16,
    #[case] struct_size: usize,
    #[case] symbols: &[fields::Field<types::Symbol>],
) {
    let width = crate::compat::SYMBOL_CSTR_LEN_V3 as u16;
    let rec = default_layout().record_layout(rtype).unwrap();
    for (i, symbol) in symbols.iter().enumerate() {
        let def = rec
            .field_defs
            .iter()
            .find(|f| f.field_id == symbol.id())
            .unwrap();
        assert_eq!(def.type_id, fields::SYMBOL_ID);
        assert_eq!(def.size, width);
        assert_eq!(
            usize::from(def.offset),
            struct_size + i * usize::from(width)
        );
    }
    assert_eq!(
        usize::from(rec.base_record_size),
        (struct_size + symbols.len() * usize::from(width)).next_multiple_of(8)
    );
    let err = crate::layout::validate(default_layout(), width + 1).unwrap_err();
    assert!(err.to_string().contains("symbol width"), "{err}");
}

mod decoded_layout {
    use std::os::raw::c_char;

    use rstest::*;

    use crate::{
        decode::{
            dbn::{fsm::DbnFsm, Decoder},
            DbnMetadata,
        },
        encode::dbn::v4::Encoder,
        layout::{
            DynFieldAccess, RecordLayoutBuilder, RecordLayoutDesc, StreamLayout,
            StreamLayoutBuilder, Verdict,
        },
        v4::{
            fields::{self, Field, FieldType},
            rtype,
            types::Char,
            RecordBuf, RecordHeader, TradeMsg,
        },
        Dataset, FlagSet, Metadata, SType, Schema, SYMBOL_CSTR_LEN,
    };

    const SYM_LEN: u16 = SYMBOL_CSTR_LEN as u16;
    const TS_RECV: u64 = 1_609_160_400_000_000_000;
    const TS_OUT: u64 = TS_RECV + 1_000;

    fn conditions<T: FieldType>() -> Field<T> {
        Field::new(fields::CONDITIONS.id())
    }

    fn trades_with_tail(declare: impl FnOnce(&mut RecordLayoutBuilder<'_>)) -> StreamLayout {
        let mut sb = StreamLayoutBuilder::new(1, SYM_LEN);
        {
            let mut r = sb.record(rtype::MBP_0, Schema::Trades as u16, "TradeMsg");
            TradeMsg::add_fields(&mut r);
            declare(&mut r);
            r.finish();
        }
        sb.build().unwrap()
    }

    fn metadata(layout: StreamLayout, ts_out: bool) -> Metadata {
        let mut metadata = Metadata::builder()
            .version(crate::v4::DBN_VERSION)
            .dataset(Dataset::GlbxMdp3)
            .schema(Some(Schema::Trades))
            .start(TS_RECV)
            .end(None)
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .ts_out(ts_out)
            .build();
        metadata.symbol_cstr_len = SYM_LEN as usize;
        metadata.layout = Some(layout);
        metadata
    }

    fn trade() -> TradeMsg {
        TradeMsg {
            hd: RecordHeader::new::<TradeMsg>(rtype::MBP_0, 1, 555, TS_RECV - 100),
            price: 123_000_000_000,
            size: 7,
            action: b'T' as c_char,
            side: b'B' as c_char,
            flags: FlagSet::new(0b1010),
            _reserved: 0,
            ts_recv: TS_RECV,
            ts_in_delta: 9,
            sequence: 88,
        }
    }

    fn trade_with_tail(layout: &StreamLayout, tail: &[u8]) -> RecordBuf<128> {
        let total = layout.record_layout(rtype::MBP_0).unwrap().record_size() as usize;
        let mut buf = RecordBuf::<128>::from(trade());
        let raw = buf.raw_buf_mut();
        raw[size_of::<TradeMsg>()..][..tail.len()].copy_from_slice(tail);
        raw[..2].copy_from_slice(&((total - size_of::<RecordHeader>()) as u16).to_le_bytes());
        buf
    }

    #[rstest]
    #[case::opra_conditions(
        trades_with_tail(|r| { r.ext_field(conditions::<Char>(), "conditions"); }),
        false,
        b"@".to_vec()
    )]
    #[case::sip_conditions(
        trades_with_tail(|r| { r.ext_field(conditions::<[Char; 4]>(), "conditions"); }),
        false,
        b"@ TI".to_vec()
    )]
    #[case::ts_out(
        trades_with_tail(|r| { r.ext_field(fields::TS_OUT, "ts_out"); }),
        true,
        TS_OUT.to_le_bytes().to_vec()
    )]
    fn extension_resolves_from_the_decoded_layout(
        #[case] layout: StreamLayout,
        #[case] ts_out: bool,
        #[case] tail: Vec<u8>,
    ) -> crate::Result<()> {
        let metadata = metadata(layout.clone(), ts_out);
        let sent = trade_with_tail(&layout, &tail);
        let mut encoded = Vec::new();
        Encoder::new(&mut encoded, &metadata)?.encode_ref(sent.as_rec_ref())?;

        let fsm = DbnFsm::builder().header::<RecordHeader>().build()?;
        let mut decoder = Decoder::with_fsm(encoded.as_slice(), fsm)?;
        assert_eq!(decoder.metadata().layout.as_ref(), Some(&layout));
        let index = decoder
            .metadata()
            .layout_index()
            .expect("version 4 carries a layout");
        assert_eq!(index.verdict(rtype::MBP_0), Some(Verdict::Compiled));
        let rec = decoder.decode_ref()?.expect("one trade record");

        assert_eq!(rec.as_ref(), sent.as_rec_ref().as_ref());
        match tail.len() {
            1 => assert_eq!(
                rec.field(&index, conditions::<Char>()),
                Some(Char(b'@' as c_char))
            ),
            4 => assert_eq!(
                rec.field(&index, conditions::<[Char; 4]>()),
                Some((*b"@ TI").map(|b| Char(b as c_char)))
            ),
            _ => assert_eq!(rec.field(&index, fields::TS_OUT).map(|t| t.0), Some(TS_OUT)),
        }
        if ts_out {
            assert!(rec.field(&index, conditions::<Char>()).is_none());
        } else {
            assert!(rec.field(&index, fields::TS_OUT).is_none());
        }
        Ok(())
    }

    #[test]
    fn a_record_that_does_not_match_the_layout_is_refused() {
        let metadata = metadata(
            trades_with_tail(|r| {
                r.ext_field(conditions::<[Char; 4]>(), "conditions");
            }),
            false,
        );
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf, &metadata).unwrap();

        let err = encoder.encode(&trade()).unwrap_err().to_string();
        assert!(err.contains("in this stream's layout"), "{err}");
    }
}
