//! Layouts of the compiled DBN version 4 record structs.

use std::sync::LazyLock;

use crate::layout::{RecordLayoutBuilder, RecordLayoutDesc, StreamLayout, StreamLayoutBuilder};

use super::{fields::Field, rtype, types::CStr};

/// The layout of the compiled v4 record structs, for
/// [`LayoutIndex::new(layout, compiled)`](crate::layout::LayoutIndex::new).
pub fn default_layout() -> &'static StreamLayout {
    static LAYOUT: LazyLock<StreamLayout> = LazyLock::new(|| compiled_layout(|_| true, |_, _| {}));
    &LAYOUT
}

pub(crate) fn compiled_layout(
    include: impl Fn(u16) -> bool,
    mut extend: impl FnMut(u16, &mut RecordLayoutBuilder<'_>),
) -> StreamLayout {
    fn add<R: RecordLayoutDesc>(
        sb: &mut StreamLayoutBuilder,
        rtype: u16,
        include: &impl Fn(u16) -> bool,
        extend: &mut impl FnMut(u16, &mut RecordLayoutBuilder<'_>),
    ) {
        if !include(rtype) {
            return;
        }
        let mut r = sb.layout_of::<R>(rtype);
        extend(rtype, &mut r);
        r.finish();
    }

    let mut sb = StreamLayoutBuilder::new(1, crate::compat::SYMBOL_CSTR_LEN_V3 as u16);
    add::<super::MboMsg>(&mut sb, rtype::MBO, &include, &mut extend);
    add::<super::TradeMsg>(&mut sb, rtype::MBP_0, &include, &mut extend);
    add::<super::Mbp1Msg>(&mut sb, rtype::MBP_1, &include, &mut extend);
    add::<super::Mbp1Msg>(&mut sb, rtype::TBBO, &include, &mut extend);
    add::<super::Mbp10Msg>(&mut sb, rtype::MBP_10, &include, &mut extend);
    add::<super::BboMsg>(&mut sb, rtype::BBO_1S, &include, &mut extend);
    add::<super::BboMsg>(&mut sb, rtype::BBO_1M, &include, &mut extend);
    add::<super::Cmbp1Msg>(&mut sb, rtype::CMBP_1, &include, &mut extend);
    add::<super::Cmbp1Msg>(&mut sb, rtype::TCBBO, &include, &mut extend);
    add::<super::CbboMsg>(&mut sb, rtype::CBBO_1S, &include, &mut extend);
    add::<super::CbboMsg>(&mut sb, rtype::CBBO_1M, &include, &mut extend);
    add::<super::OhlcvMsg>(&mut sb, rtype::OHLCV_1S, &include, &mut extend);
    add::<super::OhlcvMsg>(&mut sb, rtype::OHLCV_1M, &include, &mut extend);
    add::<super::OhlcvMsg>(&mut sb, rtype::OHLCV_1H, &include, &mut extend);
    add::<super::OhlcvMsg>(&mut sb, rtype::OHLCV_1D, &include, &mut extend);
    add::<super::OhlcvMsg>(&mut sb, rtype::OHLCV_EOD, &include, &mut extend);
    add::<super::StatusMsg>(&mut sb, rtype::STATUS, &include, &mut extend);
    add::<super::InstrumentDefMsg>(&mut sb, rtype::INSTRUMENT_DEF, &include, &mut extend);
    add::<super::ImbalanceMsg>(&mut sb, rtype::IMBALANCE, &include, &mut extend);
    add::<super::StatMsg>(&mut sb, rtype::STATISTICS, &include, &mut extend);
    add::<super::ErrorMsg>(&mut sb, rtype::ERROR, &include, &mut extend);
    add::<super::SymbolMappingMsg>(&mut sb, rtype::SYMBOL_MAPPING, &include, &mut extend);
    add::<super::SystemMsg>(&mut sb, rtype::SYSTEM, &include, &mut extend);
    sb.build().expect("v4 record layouts are valid")
}

pub(crate) fn required_fields(rtype: u16) -> &'static [(Field<CStr>, &'static str)] {
    match rtype {
        rtype::INSTRUMENT_DEF => super::InstrumentDefMsg::REQUIRED_FIELDS,
        rtype::SYMBOL_MAPPING => super::SymbolMappingMsg::REQUIRED_FIELDS,
        _ => &[],
    }
}
