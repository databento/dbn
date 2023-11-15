use dbn::{
    compat::{InstrumentDefMsgV1, SymbolMappingMsgV1},
    InstrumentDefMsg, SymbolMappingMsg,
};

/// Converts an V1 InstrumentDefMsg to V2.
#[no_mangle]
pub extern "C" fn from_instrument_def_v1_to_v2(def_v1: &InstrumentDefMsgV1) -> InstrumentDefMsg {
    InstrumentDefMsg::from(def_v1)
}

/// Converts an V1 SymbolMappingMsg to V2.
#[no_mangle]
pub extern "C" fn from_symbol_mapping_v1_to_v2(def_v1: &SymbolMappingMsgV1) -> SymbolMappingMsg {
    SymbolMappingMsg::from(def_v1)
}
