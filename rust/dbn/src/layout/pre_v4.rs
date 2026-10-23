//! Layout descriptions of the DBN version 1-3 record structs, the source side of an
//! upgrade to DBN version 4.

use crate::{
    layout::{RecordLayout, RecordLayoutBuilder, StreamLayoutBuilder},
    v4::{fields, rtype},
};

pub(crate) fn record_layouts(version: u8, ts_out: bool) -> Vec<RecordLayout> {
    match version {
        1 => version_layouts!(v1, ts_out),
        2 => version_layouts!(v2, ts_out),
        3 => version_layouts!(v3, ts_out),
        _ => Vec::new(),
    }
}

// In DBN versions 1-3, `ts_out` directly follows the record
fn finish_with_ts_out(mut r: RecordLayoutBuilder<'_>, ts_out: bool) {
    if ts_out {
        r.finish_base();
        r.field(fields::TS_OUT, "ts_out");
    }
    r.finish();
}

macro_rules! version_layouts {
    ($v:ident, $ts_out:expr) => {{
        use crate::$v::*;
        let mut sb = StreamLayoutBuilder::pre_v4(SYMBOL_CSTR_LEN as u16);
        finish_with_ts_out(sb.layout_of::<MboMsg>(rtype::MBO), $ts_out);
        finish_with_ts_out(sb.layout_of::<TradeMsg>(rtype::MBP_0), $ts_out);
        finish_with_ts_out(sb.layout_of::<Mbp1Msg>(rtype::MBP_1), $ts_out);
        finish_with_ts_out(sb.layout_of::<Mbp1Msg>(rtype::TBBO), $ts_out);
        finish_with_ts_out(sb.layout_of::<Mbp10Msg>(rtype::MBP_10), $ts_out);
        finish_with_ts_out(sb.layout_of::<BboMsg>(rtype::BBO_1S), $ts_out);
        finish_with_ts_out(sb.layout_of::<BboMsg>(rtype::BBO_1M), $ts_out);
        finish_with_ts_out(sb.layout_of::<Cmbp1Msg>(rtype::CMBP_1), $ts_out);
        finish_with_ts_out(sb.layout_of::<Cmbp1Msg>(rtype::TCBBO), $ts_out);
        finish_with_ts_out(sb.layout_of::<CbboMsg>(rtype::CBBO_1S), $ts_out);
        finish_with_ts_out(sb.layout_of::<CbboMsg>(rtype::CBBO_1M), $ts_out);
        finish_with_ts_out(sb.layout_of::<OhlcvMsg>(rtype::OHLCV_1S), $ts_out);
        finish_with_ts_out(sb.layout_of::<OhlcvMsg>(rtype::OHLCV_1M), $ts_out);
        finish_with_ts_out(sb.layout_of::<OhlcvMsg>(rtype::OHLCV_1H), $ts_out);
        finish_with_ts_out(sb.layout_of::<OhlcvMsg>(rtype::OHLCV_1D), $ts_out);
        finish_with_ts_out(sb.layout_of::<OhlcvMsg>(rtype::OHLCV_EOD), $ts_out);
        finish_with_ts_out(sb.layout_of::<StatusMsg>(rtype::STATUS), $ts_out);
        finish_with_ts_out(
            sb.layout_of::<InstrumentDefMsg>(rtype::INSTRUMENT_DEF),
            $ts_out,
        );
        finish_with_ts_out(sb.layout_of::<ImbalanceMsg>(rtype::IMBALANCE), $ts_out);
        finish_with_ts_out(sb.layout_of::<StatMsg>(rtype::STATISTICS), $ts_out);
        finish_with_ts_out(sb.layout_of::<ErrorMsg>(rtype::ERROR), $ts_out);
        finish_with_ts_out(
            sb.layout_of::<SymbolMappingMsg>(rtype::SYMBOL_MAPPING),
            $ts_out,
        );
        finish_with_ts_out(sb.layout_of::<SystemMsg>(rtype::SYSTEM), $ts_out);
        #[allow(deprecated)]
        finish_with_ts_out(sb.layout_of::<OhlcvMsg>(rtype::OHLCV_DEPRECATED), $ts_out);
        sb.into_unvalidated_record_layouts()
    }};
}
use version_layouts;

#[cfg(test)]
mod tests {
    use std::{ffi::c_char, mem::size_of};

    use rstest::rstest;

    use super::*;
    use crate::{
        layout::upgrade::UpgradePlan,
        v4::{self, fields},
    };

    fn record_layout(version: u8, rtype: u16) -> Option<RecordLayout> {
        record_layouts(version, false)
            .into_iter()
            .find(|rec| rec.rtype == rtype)
    }

    const RTYPES: &[u16] = &[
        rtype::MBO,
        rtype::MBP_0,
        rtype::MBP_1,
        rtype::TBBO,
        rtype::MBP_10,
        rtype::BBO_1S,
        rtype::BBO_1M,
        rtype::CMBP_1,
        rtype::TCBBO,
        rtype::CBBO_1S,
        rtype::CBBO_1M,
        rtype::OHLCV_1S,
        rtype::OHLCV_1M,
        rtype::OHLCV_1H,
        rtype::OHLCV_1D,
        rtype::OHLCV_EOD,
        rtype::STATUS,
        rtype::INSTRUMENT_DEF,
        rtype::IMBALANCE,
        rtype::STATISTICS,
        rtype::ERROR,
        rtype::SYMBOL_MAPPING,
        rtype::SYSTEM,
    ];

    #[rstest]
    fn every_record_has_a_layout(#[values(1, 2, 3)] version: u8) {
        for &rtype in RTYPES {
            let rec = record_layout(version, rtype)
                .unwrap_or_else(|| panic!("no v{version} layout for {rtype:#04x}"));
            let hd = rec
                .field_defs
                .iter()
                .find(|f| f.field_id == fields::HD.id())
                .unwrap_or_else(|| panic!("no v{version} header for {rtype:#04x}"));
            assert_eq!(hd.size as usize, size_of::<crate::RecordHeader>());
        }
    }

    #[rstest]
    #[case::v1_def(1, rtype::INSTRUMENT_DEF, size_of::<crate::v1::InstrumentDefMsg>())]
    #[case::v1_stat(1, rtype::STATISTICS, size_of::<crate::v1::StatMsg>())]
    #[case::v1_error(1, rtype::ERROR, size_of::<crate::v1::ErrorMsg>())]
    #[case::v2_def(2, rtype::INSTRUMENT_DEF, size_of::<crate::v2::InstrumentDefMsg>())]
    #[case::v3_def(3, rtype::INSTRUMENT_DEF, size_of::<crate::v3::InstrumentDefMsg>())]
    #[case::v3_mbp_10(3, rtype::MBP_10, size_of::<crate::v3::Mbp10Msg>())]
    fn a_layout_spans_its_struct(#[case] version: u8, #[case] rtype: u16, #[case] size: usize) {
        assert_eq!(
            record_layout(version, rtype).unwrap().base_record_size as usize,
            size
        );
    }

    #[rstest]
    fn ts_out_follows_every_record(#[values(1, 2, 3)] version: u8) {
        for (rec, with_ts_out) in record_layouts(version, false)
            .iter()
            .zip(&record_layouts(version, true))
        {
            let ts_out = with_ts_out
                .field_defs
                .iter()
                .find(|f| f.field_id == fields::TS_OUT.id())
                .unwrap_or_else(|| panic!("no v{version} ts_out for {}", rec.name));
            assert_eq!(ts_out.offset, rec.base_record_size, "{}", rec.name);
        }
    }

    #[rstest]
    fn every_v4_default_field_upgrades(#[values(1, 2, 3)] version: u8) {
        let dst_layout = v4::layouts::default_layout();
        let src_layouts = record_layouts(version, false);
        let missing: Vec<(u16, &str)> = RTYPES
            .iter()
            .flat_map(|&rtype| {
                let src = src_layouts.iter().find(|rec| rec.rtype == rtype).unwrap();
                let dst = dst_layout.record_layout(rtype).unwrap();
                let plan = UpgradePlan::new(src, dst);
                dst.field_defs
                    .iter()
                    .zip(&dst.field_names)
                    .filter(|(f, _)| !f.is_padding() && f.field_id != fields::HD.id())
                    .filter(|(f, _)| src.field_defs.iter().any(|s| s.field_id == f.field_id))
                    .filter(move |(f, _)| !plan.writes_start_of(f))
                    .map(move |(_, name)| (rtype, name.as_str()))
            })
            .collect();
        assert!(missing.is_empty(), "v{version} drops {missing:?}");
    }

    fn upgrade<D>(version: u8, rtype: u16, src: &[u8]) -> (D, Vec<u8>) {
        let dst_layout = v4::layouts::default_layout().record_layout(rtype).unwrap();
        let plan = UpgradePlan::new(&record_layout(version, rtype).unwrap(), dst_layout);
        let mut dst = vec![0; dst_layout.base_record_size as usize];
        plan.apply(src, &mut dst);
        // SAFETY: `D` is a `repr(C)` record of plain data, and `dst` holds at least
        // `size_of::<D>()` bytes.
        let out = unsafe { dst.as_ptr().cast::<D>().read_unaligned() };
        (out, dst[size_of::<D>()..].to_vec())
    }

    #[test]
    fn a_v1_stat_upgrades() {
        let src = crate::v1::StatMsg {
            ts_recv: 1,
            ts_ref: 2,
            price: 3,
            quantity: crate::v1::UNDEF_STAT_QUANTITY,
            sequence: 4,
            ts_in_delta: -5,
            stat_type: crate::StatType::OpeningPrice as u16,
            channel_id: 6,
            update_action: crate::StatUpdateAction::New as u8,
            stat_flags: 7,
            ..Default::default()
        };
        let (out, _) = upgrade::<v4::StatMsg>(1, rtype::STATISTICS, src.as_ref());
        assert_eq!(
            out,
            v4::StatMsg {
                hd: out.hd,
                ts_recv: 1,
                ts_ref: 2,
                price: 3,
                quantity: crate::UNDEF_STAT_QUANTITY,
                sequence: 4,
                ts_in_delta: -5,
                stat_type: crate::StatType::OpeningPrice as u16,
                channel_id: 6,
                update_action: crate::StatUpdateAction::New as u8,
                stat_flags: 7,
                _reserved: [0; 3],
            }
        );
    }

    #[test]
    fn a_v1_definition_upgrades() {
        let src = crate::v1::InstrumentDefMsg {
            ts_recv: 1,
            min_price_increment: 2,
            display_factor: 3,
            raw_instrument_id: u32::MAX,
            min_lot_size_round_lot: 4,
            security_update_action: crate::SecurityUpdateAction::Modify,
            instrument_class: crate::InstrumentClass::Future as c_char,
            match_algorithm: crate::MatchAlgorithm::Fifo as c_char,
            channel_id: u16::MAX,
            raw_symbol: crate::record::str_to_c_chars("ESZ5").unwrap(),
            ..Default::default()
        };
        let (out, symbol) = upgrade::<v4::InstrumentDefMsg>(1, rtype::INSTRUMENT_DEF, src.as_ref());
        assert_eq!(
            out,
            v4::InstrumentDefMsg {
                hd: out.hd,
                ts_recv: 1,
                min_price_increment: 2,
                display_factor: 3,
                raw_instrument_id: u64::from(u32::MAX),
                min_lot_size_round_lot: 4,
                security_update_action: crate::SecurityUpdateAction::Modify as c_char,
                instrument_class: crate::InstrumentClass::Future as c_char,
                match_algorithm: crate::MatchAlgorithm::Fifo as c_char,
                channel_id: u8::MAX,
            }
        );
        assert_eq!(symbol[..4], *b"ESZ5");
        assert!(symbol[4..crate::SYMBOL_CSTR_LEN].iter().all(|&b| b == 0));
    }

    #[test]
    fn a_v3_mbp_10_upgrades() {
        let src = crate::v3::Mbp10Msg {
            price: 1,
            size: 2,
            action: crate::Action::Add as c_char,
            side: crate::Side::Bid as c_char,
            flags: crate::FlagSet::new(crate::flags::LAST),
            depth: 3,
            ts_recv: 4,
            ts_in_delta: 5,
            sequence: 6,
            levels: std::array::from_fn(|i| crate::BidAskPair {
                bid_px: i as i64,
                ask_sz: i as u32,
                ..Default::default()
            }),
            ..Default::default()
        };
        let (out, _) = upgrade::<v4::Mbp10Msg>(3, rtype::MBP_10, src.as_ref());
        assert_eq!(
            out,
            v4::Mbp10Msg {
                hd: out.hd,
                price: 1,
                size: 2,
                action: crate::Action::Add as c_char,
                side: crate::Side::Bid as c_char,
                flags: crate::FlagSet::new(crate::flags::LAST),
                depth: 3,
                ts_recv: 4,
                ts_in_delta: 5,
                sequence: 6,
                levels: src.levels,
            }
        );
    }
}
