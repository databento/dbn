use std::os::raw::c_char;

use crate::{
    rtype, v1, MatchAlgorithm, RecordHeader, StatUpdateAction, UNDEF_PRICE, UNDEF_TIMESTAMP,
};

use super::{
    ErrorMsg, InstrumentDefMsg, StatMsg, SymbolMappingMsg, SystemMsg, UNDEF_STAT_QUANTITY,
};

impl Default for ErrorMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::ERROR),
            err: [0; 64],
        }
    }
}

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
            _reserved2: Default::default(),
            contract_multiplier: i32::MAX,
            decay_quantity: i32::MAX,
            original_contract_size: i32::MAX,
            _reserved3: Default::default(),
            trading_reference_date: u16::MAX,
            appl_id: i16::MAX,
            maturity_year: u16::MAX,
            decay_start_date: u16::MAX,
            channel_id: u16::MAX,
            currency: [0; 4],
            settl_currency: [0; 4],
            secsubtype: [0; 6],
            raw_symbol: [0; v1::SYMBOL_CSTR_LEN],
            group: [0; 21],
            exchange: [0; 5],
            asset: [0; v1::ASSET_CSTR_LEN],
            cfi: [0; 7],
            security_type: [0; 7],
            unit_of_measure: [0; 31],
            underlying: [0; 21],
            strike_price_currency: [0; 4],
            instrument_class: 0,
            _reserved4: Default::default(),
            strike_price: UNDEF_PRICE,
            _reserved5: Default::default(),
            match_algorithm: MatchAlgorithm::default() as c_char,
            md_security_trading_status: u8::MAX,
            main_fraction: u8::MAX,
            price_display_format: u8::MAX,
            settl_price_type: u8::MAX,
            sub_fraction: u8::MAX,
            underlying_product: u8::MAX,
            security_update_action: Default::default(),
            maturity_month: u8::MAX,
            maturity_day: u8::MAX,
            maturity_week: u8::MAX,
            user_defined_instrument: Default::default(),
            contract_multiplier_unit: i8::MAX,
            flow_schedule_type: i8::MAX,
            tick_rule: u8::MAX,
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
            channel_id: u16::MAX,
            update_action: StatUpdateAction::default() as u8,
            stat_flags: 0,
            _reserved: Default::default(),
        }
    }
}

impl Default for SymbolMappingMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::SYMBOL_MAPPING),
            stype_in_symbol: [0; v1::SYMBOL_CSTR_LEN],
            stype_out_symbol: [0; v1::SYMBOL_CSTR_LEN],
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
