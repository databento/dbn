//! Record data types for encoding different Databento [`Schema`](crate::enums::Schema)s
//! in DBN version 1.

pub(crate) use crate::compat::METADATA_RESERVED_LEN_V1 as METADATA_RESERVED_LEN;
pub use crate::compat::{
    ErrorMsgV1 as ErrorMsg, InstrumentDefMsgV1 as InstrumentDefMsg, StatMsgV1 as StatMsg,
    SymbolMappingMsgV1 as SymbolMappingMsg, SystemMsgV1 as SystemMsg,
    ASSET_CSTR_LEN_V1 as ASSET_CSTR_LEN, SYMBOL_CSTR_LEN_V1 as SYMBOL_CSTR_LEN,
    UNDEF_STAT_QUANTITY_V1 as UNDEF_STAT_QUANTITY,
};
pub use crate::record::{
    Bbo1MMsg, Bbo1SMsg, BboMsg, Cbbo1MMsg, Cbbo1SMsg, CbboMsg, Cmbp1Msg, ImbalanceMsg, MboMsg,
    Mbp10Msg, Mbp1Msg, OhlcvMsg, StatusMsg, TbboMsg, TcbboMsg, TradeMsg, WithTsOut,
};

mod impl_default;
mod methods;

use crate::compat::{InstrumentDefRec, StatRec, SymbolMappingRec};

/// The DBN version of this module.
pub const DBN_VERSION: u8 = 1;

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

impl InstrumentDefRec for InstrumentDefMsg {
    fn raw_symbol(&self) -> crate::Result<&str> {
        Self::raw_symbol(self)
    }

    fn asset(&self) -> crate::Result<&str> {
        Self::asset(self)
    }

    fn security_type(&self) -> crate::Result<&str> {
        Self::security_type(self)
    }

    fn security_update_action(&self) -> crate::Result<crate::SecurityUpdateAction> {
        Ok(self.security_update_action)
    }

    fn channel_id(&self) -> u16 {
        self.channel_id
    }
}

impl StatRec for StatMsg {
    const UNDEF_STAT_QUANTITY: i64 = UNDEF_STAT_QUANTITY as i64;

    fn stat_type(&self) -> crate::Result<crate::StatType> {
        Self::stat_type(self)
    }

    fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        Self::ts_recv(self)
    }

    fn ts_ref(&self) -> Option<time::OffsetDateTime> {
        Self::ts_ref(self)
    }

    fn update_action(&self) -> crate::Result<crate::StatUpdateAction> {
        Self::update_action(self)
    }

    fn price(&self) -> i64 {
        self.price
    }

    fn quantity(&self) -> i64 {
        self.quantity as i64
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use rstest::*;
    use type_layout::{Field, TypeLayout};

    use crate::{v2, v3};

    use super::*;

    #[test]
    fn test_default_equivalency() {
        assert_eq!(
            v2::InstrumentDefMsg::from(&InstrumentDefMsg::default()),
            v2::InstrumentDefMsg::default()
        );
        assert_eq!(
            v3::InstrumentDefMsg::from(&InstrumentDefMsg::default()),
            v3::InstrumentDefMsg::default()
        );
        assert_eq!(
            v3::StatMsg::from(&StatMsg::default()),
            v3::StatMsg::default()
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
        assert_eq!(StatMsg::ordered_fields(""), v3::StatMsg::ordered_fields(""));
    }

    #[rstest]
    #[case::definition(InstrumentDefMsg::default(), 360)]
    #[case::stat(StatMsg::default(), 64)]
    #[case::error(ErrorMsg::default(), 80)]
    #[case::symbol_mapping(SymbolMappingMsg::default(), 80)]
    #[case::system(SystemMsg::default(), 80)]
    fn test_sizes<R: Sized>(#[case] _rec: R, #[case] exp: usize) {
        assert_eq!(mem::size_of::<R>(), exp);
        assert!(mem::size_of::<R>() <= crate::MAX_RECORD_LEN);
    }

    #[rstest]
    #[case::definition(InstrumentDefMsg::default())]
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
}
