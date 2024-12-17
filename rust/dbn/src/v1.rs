//! Record data types for encoding different Databento [`Schema`](crate::enums::Schema)s
//! in DBN version 1.

pub use crate::compat::ErrorMsgV1 as ErrorMsg;
pub use crate::compat::InstrumentDefMsgV1 as InstrumentDefMsg;
pub use crate::compat::SymbolMappingMsgV1 as SymbolMappingMsg;
pub use crate::compat::SystemMsgV1 as SystemMsg;
pub use crate::compat::SYMBOL_CSTR_LEN_V1 as SYMBOL_CSTR_LEN;
pub use crate::record::{
    Bbo1MMsg, Bbo1SMsg, BboMsg, Cbbo1MMsg, Cbbo1SMsg, CbboMsg, Cmbp1Msg, ImbalanceMsg, MboMsg,
    OhlcvMsg, StatMsg, StatusMsg, TbboMsg, TcbboMsg, TradeMsg, WithTsOut,
};

mod impl_default;
mod methods;

use crate::compat::SymbolMappingRec;

impl SymbolMappingRec for SymbolMappingMsg {
    fn stype_in_symbol(&self) -> crate::Result<&str> {
        Self::stype_in_symbol(self)
    }

    fn stype_out_symbol(&self) -> crate::Result<&str> {
        Self::stype_out_symbol(self)
    }

    fn start_ts(&self) -> Option<time::OffsetDateTime> {
        Self::start_ts(self)
    }

    fn end_ts(&self) -> Option<time::OffsetDateTime> {
        Self::end_ts(self)
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use rstest::*;
    use type_layout::{Field, TypeLayout};

    use crate::v2;

    use super::*;

    #[test]
    fn test_default_equivalency() {
        assert_eq!(
            v2::InstrumentDefMsg::from(&InstrumentDefMsg::default()),
            v2::InstrumentDefMsg::default()
        );
    }

    #[cfg(feature = "python")]
    #[test]
    fn test_strike_price_order_didnt_change() {
        use crate::python::PyFieldDesc;

        assert_eq!(
            InstrumentDefMsg::ordered_fields(""),
            v2::InstrumentDefMsg::ordered_fields("")
        );
    }

    #[rstest]
    #[case::definition(InstrumentDefMsg::default(), 360)]
    #[case::error(ErrorMsg::default(), 80)]
    #[case::symbol_mapping(SymbolMappingMsg::default(), 80)]
    #[case::system(SystemMsg::default(), 80)]
    fn test_sizes<R: Sized>(#[case] _rec: R, #[case] exp: usize) {
        assert_eq!(mem::size_of::<R>(), exp);
        assert!(mem::size_of::<R>() <= crate::MAX_RECORD_LEN);
    }

    #[rstest]
    #[case::definition(InstrumentDefMsg::default())]
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
}
