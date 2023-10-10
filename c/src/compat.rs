use dbn::{
    compat::{InstrumentDefMsgV2, SymbolMappingMsgV2},
    InstrumentDefMsg, SymbolMappingMsg,
};

/// Converts an V1 InstrumentDefMsg to V2.
#[no_mangle]
pub extern "C" fn from_instrument_def_v1_to_v2(def_v1: &InstrumentDefMsg) -> InstrumentDefMsgV2 {
    InstrumentDefMsgV2::from(def_v1)
}

/// Converts an V1 SymbolMappingMsg to V2.
#[no_mangle]
pub extern "C" fn from_symbol_mapping_v1_to_v2(def_v1: &SymbolMappingMsg) -> SymbolMappingMsgV2 {
    SymbolMappingMsgV2::from(def_v1)
}
