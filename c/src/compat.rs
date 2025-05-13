use dbn::{
    compat::{
        ErrorMsgV1, InstrumentDefMsgV1, InstrumentDefMsgV3, StatMsgV3, SymbolMappingMsgV1,
        SystemMsgV1,
    },
    ErrorMsg, InstrumentDefMsg, StatMsg, SymbolMappingMsg, SystemMsg,
};

/// Converts an V1 ErrorMsg to V2.
#[no_mangle]
pub extern "C" fn from_error_v1_to_v2(def_v1: &ErrorMsgV1) -> ErrorMsg {
    ErrorMsg::from(def_v1)
}

/// Converts an V1 InstrumentDefMsg to V2.
#[no_mangle]
pub extern "C" fn from_instrument_def_v1_to_v2(def_v1: &InstrumentDefMsgV1) -> InstrumentDefMsg {
    InstrumentDefMsg::from(def_v1)
}

/// Converts a V1 InstrumentDefMsg to V3.
#[no_mangle]
pub extern "C" fn from_instrument_def_v1_to_v3(def_v1: &InstrumentDefMsgV1) -> InstrumentDefMsgV3 {
    InstrumentDefMsgV3::from(def_v1)
}

/// Converts a V2 InstrumentDefMsg to V3.
#[no_mangle]
pub extern "C" fn from_instrument_def_v2_to_v3(def_v2: &InstrumentDefMsg) -> InstrumentDefMsgV3 {
    InstrumentDefMsgV3::from(def_v2)
}

/// Converts a V1 StatMsg to V3.
#[no_mangle]
pub extern "C" fn from_stat_v1_to_v3(stat_v1: &StatMsg) -> StatMsgV3 {
    StatMsgV3::from(stat_v1)
}

/// Converts an V1 SymbolMappingMsg to V2.
#[no_mangle]
pub extern "C" fn from_symbol_mapping_v1_to_v2(def_v1: &SymbolMappingMsgV1) -> SymbolMappingMsg {
    SymbolMappingMsg::from(def_v1)
}

/// Converts an V1 SystemMsg to V2.
#[no_mangle]
pub extern "C" fn from_system_v1_to_v2(def_v1: &SystemMsgV1) -> SystemMsg {
    SystemMsg::from(def_v1)
}
