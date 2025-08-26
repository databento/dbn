#![cfg(test)]

use crate::{SType, StatType};

use super::*;

#[test]
fn invalid_rtype_error() {
    let header = RecordHeader::new::<MboMsg>(0xE, 1, 2, 3);
    assert_eq!(
        header.rtype().unwrap_err().to_string(),
        "couldn't convert 0x0E to dbn::enums::RType"
    );
}

#[test]
fn debug_mbo() {
    let rec = MboMsg {
        hd: RecordHeader::new::<MboMsg>(
            rtype::MBO,
            Publisher::OpraPillarXcbo as u16,
            678,
            1704468548242628731,
        ),
        flags: FlagSet::empty().set_last().set_bad_ts_recv(),
        price: 4_500_500_000_000,
        side: b'B' as c_char,
        action: b'A' as c_char,
        ..Default::default()
    };
    assert_eq!(
            format!("{rec:?}"),
            "MboMsg { hd: RecordHeader { length: 14, rtype: Mbo, publisher_id: OpraPillarXcbo, \
            instrument_id: 678, ts_event: 1704468548242628731 }, order_id: 0, \
            price: 4500.500000000, size: 4294967295, flags: LAST | BAD_TS_RECV (136), channel_id: 255, \
            action: 'A', side: 'B', ts_recv: 18446744073709551615, ts_in_delta: 0, sequence: 0 }"
        );
}

#[test]
fn debug_stats() {
    let rec = StatMsg {
        stat_type: StatType::OpenInterest as u16,
        update_action: StatUpdateAction::New as u8,
        quantity: 5,
        stat_flags: 0b00000010,
        ..Default::default()
    };
    assert_eq!(
            format!("{rec:?}"),
            "StatMsg { hd: RecordHeader { length: 20, rtype: Statistics, publisher_id: 0, \
            instrument_id: 0, ts_event: 18446744073709551615 }, ts_recv: 18446744073709551615, \
            ts_ref: 18446744073709551615, price: UNDEF_PRICE, quantity: 5, sequence: 0, ts_in_delta: 0, \
            stat_type: OpenInterest, channel_id: 65535, update_action: New, stat_flags: 0b00000010 }"
        );
}

#[test]
fn debug_instrument_err() {
    let rec = ErrorMsg {
        err: str_to_c_chars("Missing stype_in").unwrap(),
        ..Default::default()
    };
    assert_eq!(
            format!("{rec:?}"),
            "ErrorMsg { hd: RecordHeader { length: 80, rtype: Error, publisher_id: 0, \
            instrument_id: 0, ts_event: 18446744073709551615 }, err: \"Missing stype_in\", code: 255, is_last: 255 }"
        );
}

#[test]
fn debug_instrument_sys() {
    let rec = SystemMsg::heartbeat(123);
    assert_eq!(
        format!("{rec:?}"),
        "SystemMsg { hd: RecordHeader { length: 80, rtype: System, publisher_id: 0, \
            instrument_id: 0, ts_event: 123 }, msg: \"Heartbeat\", code: Heartbeat }"
    );
}

#[test]
fn debug_instrument_symbol_mapping() {
    let rec = SymbolMappingMsg {
        hd: RecordHeader::new::<SymbolMappingMsg>(
            rtype::SYMBOL_MAPPING,
            0,
            5602,
            1704466940331347283,
        ),
        stype_in: SType::RawSymbol as u8,
        stype_in_symbol: str_to_c_chars("ESM4").unwrap(),
        stype_out: SType::RawSymbol as u8,
        stype_out_symbol: str_to_c_chars("ESM4").unwrap(),
        ..Default::default()
    };
    assert_eq!(
            format!("{rec:?}"),
            "SymbolMappingMsg { hd: RecordHeader { length: 44, rtype: SymbolMapping, publisher_id: 0, instrument_id: 5602, ts_event: 1704466940331347283 }, stype_in: RawSymbol, stype_in_symbol: \"ESM4\", stype_out: RawSymbol, stype_out_symbol: \"ESM4\", start_ts: 18446744073709551615, end_ts: 18446744073709551615 }"
        );
}
