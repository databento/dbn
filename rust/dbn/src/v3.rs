//! Record data types for encoding different Databento [`Schema`](crate::enums::Schema)s
//! in the current DBN version 3.

pub use crate::compat::{
    HasRTypeV1 as HasRType, RecordBufV1 as RecordBuf, RecordRefMutV1 as RecordRefMut,
    RecordRefV1 as RecordRef, RecordV1 as Record, ASSET_CSTR_LEN_V3 as ASSET_CSTR_LEN,
    SYMBOL_CSTR_LEN_V3 as SYMBOL_CSTR_LEN, UNDEF_STAT_QUANTITY_V3 as UNDEF_STAT_QUANTITY,
};
pub use crate::record::{
    Bbo1MMsg, Bbo1SMsg, BboMsg, Cbbo1MMsg, Cbbo1SMsg, CbboMsg, Cmbp1Msg, ErrorMsg, ImbalanceMsg,
    InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, RecordHeader, StatMsg, StatusMsg,
    SymbolMappingMsg, SystemMsg, TbboMsg, TcbboMsg, TradeMsg, WithTsOut,
};

mod methods;

/// The DBN version of this module.
pub const DBN_VERSION: u8 = 3;

#[cfg(test)]
mod tests {
    use std::mem;

    use rstest::*;
    use type_layout::{Field, TypeLayout};

    use super::*;

    #[cfg(feature = "python")]
    #[test]
    fn test_consistent_field_order_and_leg_fields_last() {
        use std::ops::Not;

        use crate::{python::PyFieldDesc, v2};

        let v3_fields = InstrumentDefMsg::ordered_fields("");
        let mut v2_fields = v2::InstrumentDefMsg::ordered_fields("")
            .into_iter()
            .filter(|f| {
                matches!(
                    f.as_str(),
                    "trading_reference_date"
                        | "trading_reference_price"
                        | "settl_price_type"
                        | "md_security_trading_status"
                )
                .not()
            });
        let mut has_reached_leg_fields = false;
        for (i, field) in v3_fields.into_iter().enumerate() {
            if has_reached_leg_fields {
                assert!(field.starts_with("leg_"), "{i}");
            } else if field.starts_with("leg_") {
                has_reached_leg_fields = true;
                assert!(v2_fields.next().is_none(), "{i}");
            } else {
                assert_eq!(field, v2_fields.next().unwrap(), "{i}");
            }
        }
    }

    #[rstest]
    #[case::definition(InstrumentDefMsg::default(), 520)]
    #[case::definition(StatMsg::default(), 80)]
    fn test_sizes<R: Sized>(#[case] _rec: R, #[case] exp: usize) {
        assert_eq!(mem::size_of::<R>(), exp);
        assert!(mem::size_of::<R>() <= crate::MAX_RECORD_LEN);
    }

    #[rstest]
    #[case::definition(InstrumentDefMsg::default())]
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
