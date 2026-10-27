//! Layouts of the compiled DBN version 4 record structs.

use std::sync::LazyLock;

use crate::layout::{RecordLayoutBuilder, RecordLayoutDesc, StreamLayout, StreamLayoutBuilder};

use super::{fields::Field, rtype, types::Symbol};

/// The layout of the compiled v4 record structs, for
/// [`LayoutIndex::new(layout, compiled)`](crate::layout::LayoutIndex::new).
pub fn default_layout() -> &'static StreamLayout {
    static LAYOUT: LazyLock<StreamLayout> = LazyLock::new(|| compiled_layout(|_, _| {}));
    &LAYOUT
}

pub(crate) fn compiled_layout(
    mut extend: impl FnMut(u16, &mut RecordLayoutBuilder<'_>),
) -> StreamLayout {
    fn add<R: RecordLayoutDesc>(
        sb: &mut StreamLayoutBuilder,
        rtype: u16,
        extend: &mut impl FnMut(u16, &mut RecordLayoutBuilder<'_>),
    ) {
        let mut r = sb.layout_of::<R>(rtype);
        extend(rtype, &mut r);
        r.finish();
    }

    let mut sb = StreamLayoutBuilder::new(1, crate::compat::SYMBOL_CSTR_LEN_V3 as u16);
    add::<super::MboMsg>(&mut sb, rtype::MBO, &mut extend);
    add::<super::TradeMsg>(&mut sb, rtype::MBP_0, &mut extend);
    add::<super::Mbp1Msg>(&mut sb, rtype::MBP_1, &mut extend);
    add::<super::Mbp1Msg>(&mut sb, rtype::TBBO, &mut extend);
    add::<super::Mbp10Msg>(&mut sb, rtype::MBP_10, &mut extend);
    add::<super::BboMsg>(&mut sb, rtype::BBO_1S, &mut extend);
    add::<super::BboMsg>(&mut sb, rtype::BBO_1M, &mut extend);
    add::<super::Cmbp1Msg>(&mut sb, rtype::CMBP_1, &mut extend);
    add::<super::Cmbp1Msg>(&mut sb, rtype::TCBBO, &mut extend);
    add::<super::CbboMsg>(&mut sb, rtype::CBBO_1S, &mut extend);
    add::<super::CbboMsg>(&mut sb, rtype::CBBO_1M, &mut extend);
    add::<super::OhlcvMsg>(&mut sb, rtype::OHLCV_1S, &mut extend);
    add::<super::OhlcvMsg>(&mut sb, rtype::OHLCV_1M, &mut extend);
    add::<super::OhlcvMsg>(&mut sb, rtype::OHLCV_1H, &mut extend);
    add::<super::OhlcvMsg>(&mut sb, rtype::OHLCV_1D, &mut extend);
    add::<super::OhlcvMsg>(&mut sb, rtype::OHLCV_EOD, &mut extend);
    add::<super::StatusMsg>(&mut sb, rtype::STATUS, &mut extend);
    add::<super::InstrumentDefMsg>(&mut sb, rtype::INSTRUMENT_DEF, &mut extend);
    add::<super::ImbalanceMsg>(&mut sb, rtype::IMBALANCE, &mut extend);
    add::<super::StatMsg>(&mut sb, rtype::STATISTICS, &mut extend);
    add::<super::ErrorMsg>(&mut sb, rtype::ERROR, &mut extend);
    add::<super::SymbolMappingMsg>(&mut sb, rtype::SYMBOL_MAPPING, &mut extend);
    add::<super::SystemMsg>(&mut sb, rtype::SYSTEM, &mut extend);
    sb.build().expect("v4 record layouts are valid")
}

pub(crate) fn required_fields(rtype: u16) -> &'static [(Field<Symbol>, &'static str)] {
    match rtype {
        rtype::INSTRUMENT_DEF => super::InstrumentDefMsg::REQUIRED_FIELDS,
        rtype::SYMBOL_MAPPING => super::SymbolMappingMsg::REQUIRED_FIELDS,
        _ => &[],
    }
}
