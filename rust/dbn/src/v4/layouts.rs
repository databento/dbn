//! Layouts of the compiled DBN version 4 record structs.

use std::sync::LazyLock;

use crate::layout::{RecordLayoutDesc, StreamLayout, StreamLayoutBuilder};

use super::{fields::Field, rtype, types::Symbol};

/// The layout of the compiled v4 record structs, for
/// [`LayoutIndex::new(layout, compiled)`](crate::layout::LayoutIndex::new).
#[allow(deprecated)]
pub fn default_layout() -> &'static StreamLayout {
    static LAYOUT: LazyLock<StreamLayout> = LazyLock::new(|| {
        let mut sb = StreamLayoutBuilder::new(1, crate::compat::SYMBOL_CSTR_LEN_V3 as u16);
        sb.record_of::<super::MboMsg>(rtype::MBO)
            .record_of::<super::TradeMsg>(rtype::MBP_0)
            .record_of::<super::Mbp1Msg>(rtype::MBP_1)
            .record_of::<super::Mbp1Msg>(rtype::TBBO)
            .record_of::<super::Mbp10Msg>(rtype::MBP_10)
            .record_of::<super::BboMsg>(rtype::BBO_1S)
            .record_of::<super::BboMsg>(rtype::BBO_1M)
            .record_of::<super::Cmbp1Msg>(rtype::CMBP_1)
            .record_of::<super::Cmbp1Msg>(rtype::TCBBO)
            .record_of::<super::CbboMsg>(rtype::CBBO_1S)
            .record_of::<super::CbboMsg>(rtype::CBBO_1M)
            .record_of::<super::OhlcvMsg>(rtype::OHLCV_1S)
            .record_of::<super::OhlcvMsg>(rtype::OHLCV_1M)
            .record_of::<super::OhlcvMsg>(rtype::OHLCV_1H)
            .record_of::<super::OhlcvMsg>(rtype::OHLCV_1D)
            .record_of::<super::OhlcvMsg>(rtype::OHLCV_EOD)
            .record_of::<super::OhlcvMsg>(rtype::OHLCV_DEPRECATED)
            .record_of::<super::StatusMsg>(rtype::STATUS)
            .record_of::<super::InstrumentDefMsg>(rtype::INSTRUMENT_DEF)
            .record_of::<super::ImbalanceMsg>(rtype::IMBALANCE)
            .record_of::<super::StatMsg>(rtype::STATISTICS)
            .record_of::<super::ErrorMsg>(rtype::ERROR)
            .record_of::<super::SymbolMappingMsg>(rtype::SYMBOL_MAPPING)
            .record_of::<super::SystemMsg>(rtype::SYSTEM);
        sb.build().expect("compiled v4 record layouts are valid")
    });
    &LAYOUT
}

pub(crate) fn required_fields(rtype: u16) -> &'static [(Field<Symbol>, &'static str)] {
    match rtype {
        rtype::INSTRUMENT_DEF => super::InstrumentDefMsg::REQUIRED_FIELDS,
        rtype::SYMBOL_MAPPING => super::SymbolMappingMsg::REQUIRED_FIELDS,
        _ => &[],
    }
}
