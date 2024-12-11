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
