use std::os::raw::c_char;

use crate::{
    rtype, MatchAlgorithm, RecordHeader, SecurityUpdateAction, Side, UserDefinedInstrument,
    UNDEF_PRICE, UNDEF_TIMESTAMP,
};

use super::{InstrumentDefMsg, SYMBOL_CSTR_LEN};

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
            appl_id: i16::MAX,
            maturity_year: u16::MAX,
            decay_start_date: u16::MAX,
            channel_id: u16::MAX,
            currency: Default::default(),
            settl_currency: Default::default(),
            secsubtype: Default::default(),
            raw_symbol: [0; SYMBOL_CSTR_LEN],
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
            main_fraction: u8::MAX,
            price_display_format: u8::MAX,
            sub_fraction: u8::MAX,
            underlying_product: u8::MAX,
            security_update_action: SecurityUpdateAction::Add as c_char,
            maturity_month: u8::MAX,
            maturity_day: u8::MAX,
            maturity_week: u8::MAX,
            user_defined_instrument: UserDefinedInstrument::No as c_char,
            contract_multiplier_unit: i8::MAX,
            flow_schedule_type: i8::MAX,
            tick_rule: u8::MAX,
            leg_count: 0,
            leg_index: 0,
            leg_price: UNDEF_PRICE,
            leg_delta: UNDEF_PRICE,
            leg_instrument_id: 0,
            leg_ratio_price_numerator: 0,
            leg_ratio_price_denominator: 0,
            leg_ratio_qty_numerator: 0,
            leg_ratio_qty_denominator: 0,
            leg_underlying_id: 0,
            leg_raw_symbol: [0; SYMBOL_CSTR_LEN],
            leg_instrument_class: 0,
            leg_side: Side::None as c_char,
            _reserved: Default::default(),
        }
    }
}