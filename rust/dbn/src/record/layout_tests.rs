#![cfg(test)]

use mem::offset_of;
use rstest::rstest;
use type_layout::{Field, TypeLayout};

use crate::Schema;
use crate::UNDEF_TIMESTAMP;

use super::*;

const OHLCV_MSG: OhlcvMsg = OhlcvMsg {
    hd: RecordHeader {
        length: 56,
        rtype: rtype::OHLCV_1S,
        publisher_id: 1,
        instrument_id: 5482,
        ts_event: 1609160400000000000,
    },
    open: 372025000000000,
    high: 372050000000000,
    low: 372025000000000,
    close: 372050000000000,
    volume: 57,
};

#[test]
fn test_transmute_record_bytes() {
    unsafe {
        let ohlcv_bytes = std::slice::from_raw_parts(
            &OHLCV_MSG as *const OhlcvMsg as *const u8,
            mem::size_of::<OhlcvMsg>(),
        )
        .to_vec();
        let ohlcv = transmute_record_bytes::<OhlcvMsg>(ohlcv_bytes.as_slice()).unwrap();
        assert_eq!(*ohlcv, OHLCV_MSG);
    };
}

#[test]
#[should_panic]
fn test_transmute_record_bytes_small_buffer() {
    let source = OHLCV_MSG;
    unsafe {
        let slice = std::slice::from_raw_parts(
            &source as *const OhlcvMsg as *const u8,
            mem::size_of::<OhlcvMsg>() - 5,
        );
        transmute_record_bytes::<OhlcvMsg>(slice);
    };
}

#[test]
fn test_transmute_record() {
    let source = Box::new(OHLCV_MSG);
    let ohlcv_ref: &OhlcvMsg = unsafe { transmute_record(&source.hd) }.unwrap();
    assert_eq!(*ohlcv_ref, OHLCV_MSG);
}

#[test]
fn test_transmute_record_mut() {
    let mut source = Box::new(OHLCV_MSG);
    let ohlcv_ref: &OhlcvMsg = unsafe { transmute_record_mut(&mut source.hd) }.unwrap();
    assert_eq!(*ohlcv_ref, OHLCV_MSG);
}

#[rstest]
#[case::header(RecordHeader::default::<MboMsg>(rtype::MBO), 16)]
#[case::mbo(MboMsg::default(), 56)]
#[case::ba_pair(BidAskPair::default(), 32)]
#[case::cba_pair(ConsolidatedBidAskPair::default(), mem::size_of::<BidAskPair>())]
#[case::trade(TradeMsg::default(), 48)]
#[case::mbp1(Mbp1Msg::default(), mem::size_of::<TradeMsg>() + mem::size_of::<BidAskPair>())]
#[case::mbp10(Mbp10Msg::default(), mem::size_of::<TradeMsg>() + mem::size_of::<BidAskPair>() * 10)]
#[case::bbo(BboMsg::default_for_schema(Schema::Bbo1S), mem::size_of::<Mbp1Msg>())]
#[case::cmbp1(Cmbp1Msg::default_for_schema(Schema::Cmbp1), mem::size_of::<Mbp1Msg>())]
#[case::cbbo(CbboMsg::default_for_schema(Schema::Cbbo1S), mem::size_of::<Mbp1Msg>())]
#[case::ohlcv(OhlcvMsg::default_for_schema(Schema::Ohlcv1S), 56)]
#[case::status(StatusMsg::default(), 40)]
#[case::definition(InstrumentDefMsg::default(), 520)]
#[case::imbalance(ImbalanceMsg::default(), 112)]
#[case::stat(StatMsg::default(), 80)]
#[case::error(ErrorMsg::default(), 320)]
#[case::symbol_mapping(SymbolMappingMsg::default(), 176)]
#[case::system(SystemMsg::default(), 320)]
#[case::with_ts_out(WithTsOut::new(SystemMsg::default(), 0), mem::size_of::<SystemMsg>() + 8)]
fn test_sizes<R: Sized>(#[case] _rec: R, #[case] exp: usize) {
    assert_eq!(mem::size_of::<R>(), exp);
    assert!(mem::size_of::<R>() <= crate::MAX_RECORD_LEN);
}

#[rstest]
#[case::header(RecordHeader::default::<MboMsg>(rtype::MBO))]
#[case::mbo(MboMsg::default())]
#[case::ba_pair(BidAskPair::default())]
#[case::cba_pair(ConsolidatedBidAskPair::default())]
#[case::trade(TradeMsg::default())]
#[case::mbp1(Mbp1Msg::default())]
#[case::mbp10(Mbp10Msg::default())]
#[case::bbo(BboMsg::default_for_schema(Schema::Bbo1S))]
#[case::cmbp1(Cmbp1Msg::default_for_schema(Schema::Cmbp1))]
#[case::cbbo(CbboMsg::default_for_schema(Schema::Cbbo1S))]
#[case::ohlcv(OhlcvMsg::default_for_schema(Schema::Ohlcv1S))]
#[case::status(StatusMsg::default())]
#[case::definition(InstrumentDefMsg::default())]
#[case::imbalance(ImbalanceMsg::default())]
#[case::stat(StatMsg::default())]
#[case::error(ErrorMsg::default())]
#[case::symbol_mapping(SymbolMappingMsg::default())]
#[case::system(SystemMsg::default())]
fn test_alignment_and_no_padding<R: TypeLayout>(#[case] _rec: R) {
    let layout = R::type_layout();
    assert_eq!(layout.alignment, 8, "Unexpected alignment: {layout}");
    for field in layout.fields.iter() {
        assert!(
            matches!(field, Field::Field { .. }),
            "Detected padding: {layout}"
        );
    }
}

#[test]
fn test_bbo_alignment_matches_mbp1() {
    assert_eq!(offset_of!(BboMsg, hd), offset_of!(Mbp1Msg, hd));
    assert_eq!(offset_of!(BboMsg, price), offset_of!(Mbp1Msg, price));
    assert_eq!(offset_of!(BboMsg, size), offset_of!(Mbp1Msg, size));
    assert_eq!(offset_of!(BboMsg, side), offset_of!(Mbp1Msg, side));
    assert_eq!(offset_of!(BboMsg, flags), offset_of!(Mbp1Msg, flags));
    assert_eq!(offset_of!(BboMsg, ts_recv), offset_of!(Mbp1Msg, ts_recv));
    assert_eq!(offset_of!(BboMsg, sequence), offset_of!(Mbp1Msg, sequence));
    assert_eq!(offset_of!(BboMsg, levels), offset_of!(Mbp1Msg, levels));
}

#[test]
fn test_mbo_index_ts() {
    let rec = MboMsg {
        ts_recv: 1,
        ..Default::default()
    };
    assert_eq!(rec.raw_index_ts(), 1);
}

#[test]
fn test_def_index_ts() {
    let rec = InstrumentDefMsg {
        ts_recv: 1,
        ..Default::default()
    };
    assert_eq!(rec.raw_index_ts(), 1);
}

#[test]
fn test_db_ts_always_valid_time_offsetdatetime() {
    assert!(time::OffsetDateTime::from_unix_timestamp_nanos(0).is_ok());
    assert!(time::OffsetDateTime::from_unix_timestamp_nanos((u64::MAX - 1) as i128).is_ok());
    assert!(time::OffsetDateTime::from_unix_timestamp_nanos(UNDEF_TIMESTAMP as i128).is_ok());
}

#[test]
fn test_record_object_safe() {
    let _record: Box<dyn Record> = Box::new(ErrorMsg::new(1, None, "Boxed record", true));
}
