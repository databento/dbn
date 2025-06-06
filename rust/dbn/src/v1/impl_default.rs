use std::os::raw::c_char;

use crate::{
    rtype, MatchAlgorithm, RecordHeader, SecurityUpdateAction, StatUpdateAction,
    UserDefinedInstrument, UNDEF_PRICE, UNDEF_TIMESTAMP,
};

use super::{
    ErrorMsg, InstrumentDefMsg, StatMsg, SymbolMappingMsg, SystemMsg, SYMBOL_CSTR_LEN,
    UNDEF_STAT_QUANTITY,
};

impl Default for InstrumentDefMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::INSTRUMENT_DEF),
            ts_recv: UNDEF_TIMESTAMP,
            min_price_increment: UNDEF_PRICE,
            display_factor: UNDEF_PRICE,
            expiration: UNDEF_TIMESTAMP,
            activation: UNDEF_TIMESTAMP,
            high_limit_price: UNDEF_PRICE,
            low_limit_price: UNDEF_PRICE,
            max_price_variation: UNDEF_PRICE,
            trading_reference_price: UNDEF_PRICE,
            unit_of_measure_qty: UNDEF_PRICE,
            min_price_increment_amount: UNDEF_PRICE,
            price_ratio: UNDEF_PRICE,
            inst_attrib_value: i32::MAX,
            underlying_id: 0,
            raw_instrument_id: 0,
            market_depth_implied: i32::MAX,
            market_depth: i32::MAX,
            market_segment_id: u32::MAX,
            max_trade_vol: u32::MAX,
            min_lot_size: i32::MAX,
            min_lot_size_block: i32::MAX,
            min_lot_size_round_lot: i32::MAX,
            min_trade_vol: u32::MAX,
            contract_multiplier: i32::MAX,
            decay_quantity: i32::MAX,
            original_contract_size: i32::MAX,
            trading_reference_date: u16::MAX,
            appl_id: i16::MAX,
            maturity_year: u16::MAX,
            decay_start_date: u16::MAX,
            channel_id: u16::MAX,
            currency: Default::default(),
            settl_currency: Default::default(),
            secsubtype: Default::default(),
            raw_symbol: Default::default(),
            group: Default::default(),
            exchange: Default::default(),
            asset: Default::default(),
            cfi: Default::default(),
            security_type: Default::default(),
            unit_of_measure: Default::default(),
            underlying: Default::default(),
            strike_price_currency: Default::default(),
            instrument_class: 0,
            strike_price: UNDEF_PRICE,
            match_algorithm: MatchAlgorithm::Undefined as c_char,
            md_security_trading_status: u8::MAX,
            main_fraction: u8::MAX,
            price_display_format: u8::MAX,
            settl_price_type: u8::MAX,
            sub_fraction: u8::MAX,
            underlying_product: u8::MAX,
            security_update_action: SecurityUpdateAction::Add,
            maturity_month: u8::MAX,
            maturity_day: u8::MAX,
            maturity_week: u8::MAX,
            user_defined_instrument: UserDefinedInstrument::No,
            contract_multiplier_unit: i8::MAX,
            flow_schedule_type: i8::MAX,
            tick_rule: u8::MAX,
            _reserved2: Default::default(),
            _reserved3: Default::default(),
            _reserved4: Default::default(),
            _reserved5: Default::default(),
            _dummy: Default::default(),
        }
    }
}

impl Default for StatMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::STATISTICS),
            ts_recv: UNDEF_TIMESTAMP,
            ts_ref: UNDEF_TIMESTAMP,
            price: UNDEF_PRICE,
            quantity: UNDEF_STAT_QUANTITY,
            sequence: 0,
            ts_in_delta: 0,
            stat_type: 0,
            channel_id: 0,
            update_action: StatUpdateAction::New as u8,
            stat_flags: 0,
            _reserved: Default::default(),
        }
    }
}

impl Default for ErrorMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::ERROR),
            err: [0; 64],
        }
    }
}
impl Default for SymbolMappingMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::SYMBOL_MAPPING),
            stype_in_symbol: [0; SYMBOL_CSTR_LEN],
            stype_out_symbol: [0; SYMBOL_CSTR_LEN],
            _dummy: Default::default(),
            start_ts: UNDEF_TIMESTAMP,
            end_ts: UNDEF_TIMESTAMP,
        }
    }
}

impl Default for SystemMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::SYSTEM),
            msg: [0; 64],
        }
    }
}
