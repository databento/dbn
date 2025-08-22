use std::ffi::c_char;

use crate::{
    Schema, StatusAction, StatusReason, TradingEvent, TriState, UNDEF_ORDER_SIZE, UNDEF_PRICE,
    UNDEF_STAT_QUANTITY, UNDEF_TIMESTAMP,
};

use super::*;

impl RecordHeader {
    /// Creates a new `RecordHeader` with `rtype` and `length` set
    /// for `R` while the other fields are set to their defaults.
    pub const fn default<R: HasRType>(rtype: u8) -> Self {
        Self::new::<R>(rtype, 0, 0, UNDEF_TIMESTAMP)
    }
}

impl Default for MboMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::MBO),
            order_id: 0,
            price: UNDEF_PRICE,
            size: UNDEF_ORDER_SIZE,
            flags: Default::default(),
            channel_id: u8::MAX,
            action: Action::default() as c_char,
            side: Side::default() as c_char,
            ts_recv: UNDEF_TIMESTAMP,
            ts_in_delta: 0,
            sequence: 0,
        }
    }
}

impl Default for BidAskPair {
    fn default() -> Self {
        Self {
            bid_px: UNDEF_PRICE,
            ask_px: UNDEF_PRICE,
            bid_sz: 0,
            ask_sz: 0,
            bid_ct: 0,
            ask_ct: 0,
        }
    }
}

impl Default for ConsolidatedBidAskPair {
    fn default() -> Self {
        Self {
            bid_px: UNDEF_PRICE,
            ask_px: UNDEF_PRICE,
            bid_sz: 0,
            ask_sz: 0,
            bid_pb: 0,
            _reserved1: Default::default(),
            ask_pb: 0,
            _reserved2: Default::default(),
        }
    }
}

impl Default for TradeMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::MBP_0),
            price: UNDEF_PRICE,
            size: UNDEF_ORDER_SIZE,
            action: Action::default() as c_char,
            side: Side::default() as c_char,
            flags: Default::default(),
            depth: 0,
            ts_recv: UNDEF_TIMESTAMP,
            ts_in_delta: 0,
            sequence: 0,
        }
    }
}

impl Default for Mbp1Msg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::MBP_1),
            price: UNDEF_PRICE,
            size: UNDEF_ORDER_SIZE,
            action: Action::default() as c_char,
            side: Side::default() as c_char,
            flags: Default::default(),
            depth: 0,
            ts_recv: UNDEF_TIMESTAMP,
            ts_in_delta: 0,
            sequence: 0,
            levels: Default::default(),
        }
    }
}

impl Default for Mbp10Msg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::MBP_10),
            price: UNDEF_PRICE,
            size: UNDEF_ORDER_SIZE,
            action: Action::default() as c_char,
            side: Side::default() as c_char,
            flags: Default::default(),
            depth: 0,
            ts_recv: UNDEF_TIMESTAMP,
            ts_in_delta: 0,
            sequence: 0,
            levels: Default::default(),
        }
    }
}

impl BboMsg {
    /// Creates a new default `BboMsg` for the given `schema`.
    pub fn default_for_schema(schema: Schema) -> Self {
        Self {
            hd: RecordHeader::default::<Self>(RType::from(schema) as u8),
            price: UNDEF_PRICE,
            size: UNDEF_ORDER_SIZE,
            _reserved1: 0,
            side: Side::default() as c_char,
            flags: Default::default(),
            _reserved2: 0,
            ts_recv: UNDEF_TIMESTAMP,
            _reserved3: Default::default(),
            sequence: 0,
            levels: Default::default(),
        }
    }
}

impl Cmbp1Msg {
    /// Creates a new default `Cmbp1Msg` for the given `schema`.
    pub fn default_for_schema(schema: Schema) -> Self {
        Self {
            hd: RecordHeader::default::<Self>(RType::from(schema) as u8),
            price: UNDEF_PRICE,
            size: UNDEF_ORDER_SIZE,
            action: Action::default() as c_char,
            side: Side::default() as c_char,
            flags: Default::default(),
            _reserved1: Default::default(),
            ts_recv: UNDEF_TIMESTAMP,
            ts_in_delta: 0,
            _reserved2: Default::default(),
            levels: Default::default(),
        }
    }
}

impl CbboMsg {
    /// Creates a new default `CbboMsg` for the given `schema`.
    pub fn default_for_schema(schema: Schema) -> Self {
        Self {
            hd: RecordHeader::default::<Self>(RType::from(schema) as u8),
            price: UNDEF_PRICE,
            size: UNDEF_ORDER_SIZE,
            _reserved1: 0,
            side: Side::default() as c_char,
            flags: Default::default(),
            _reserved2: 0,
            ts_recv: UNDEF_TIMESTAMP,
            _reserved3: Default::default(),
            levels: Default::default(),
        }
    }
}

impl OhlcvMsg {
    /// Creates a new default `OhlcvMsg` for the given `schema`.
    pub fn default_for_schema(schema: Schema) -> Self {
        Self {
            hd: RecordHeader::default::<Self>(RType::from(schema) as u8),
            open: UNDEF_PRICE,
            high: UNDEF_PRICE,
            low: UNDEF_PRICE,
            close: UNDEF_PRICE,
            volume: 0,
        }
    }
}

impl Default for StatusMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::STATUS),
            ts_recv: UNDEF_TIMESTAMP,
            action: StatusAction::default() as u16,
            reason: StatusReason::default() as u16,
            trading_event: TradingEvent::default() as u16,
            is_trading: TriState::default() as c_char,
            is_quoting: TriState::default() as c_char,
            is_short_sell_restricted: TriState::default() as c_char,
            _reserved: Default::default(),
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
            unit_of_measure_qty: UNDEF_PRICE,
            min_price_increment_amount: UNDEF_PRICE,
            price_ratio: UNDEF_PRICE,
            strike_price: UNDEF_PRICE,
            raw_instrument_id: 0,
            leg_price: UNDEF_PRICE,
            leg_delta: UNDEF_PRICE,
            inst_attrib_value: i32::MAX,
            underlying_id: 0,
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
            leg_instrument_id: 0,
            leg_ratio_price_numerator: 0,
            leg_ratio_price_denominator: 0,
            leg_ratio_qty_numerator: 0,
            leg_ratio_qty_denominator: 0,
            leg_underlying_id: 0,
            appl_id: i16::MAX,
            maturity_year: u16::MAX,
            decay_start_date: u16::MAX,
            channel_id: u16::MAX,
            leg_count: 0,
            leg_index: 0,
            currency: [0; 4],
            settl_currency: [0; 4],
            secsubtype: [0; 6],
            raw_symbol: [0; SYMBOL_CSTR_LEN],
            group: [0; 21],
            exchange: [0; 5],
            asset: [0; ASSET_CSTR_LEN],
            cfi: [0; 7],
            security_type: [0; 7],
            unit_of_measure: [0; 31],
            underlying: [0; 21],
            strike_price_currency: [0; 4],
            leg_raw_symbol: [0; SYMBOL_CSTR_LEN],
            instrument_class: 0,
            match_algorithm: MatchAlgorithm::default() as c_char,
            main_fraction: u8::MAX,
            price_display_format: u8::MAX,
            sub_fraction: u8::MAX,
            underlying_product: u8::MAX,
            security_update_action: SecurityUpdateAction::default() as c_char,
            maturity_month: u8::MAX,
            maturity_day: u8::MAX,
            maturity_week: u8::MAX,
            user_defined_instrument: UserDefinedInstrument::default() as c_char,
            contract_multiplier_unit: i8::MAX,
            flow_schedule_type: i8::MAX,
            tick_rule: u8::MAX,
            leg_instrument_class: 0,
            leg_side: Side::default() as c_char,
            _reserved: Default::default(),
        }
    }
}

impl Default for ImbalanceMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::IMBALANCE),
            ts_recv: UNDEF_TIMESTAMP,
            ref_price: UNDEF_PRICE,
            auction_time: UNDEF_TIMESTAMP,
            cont_book_clr_price: UNDEF_PRICE,
            auct_interest_clr_price: UNDEF_PRICE,
            ssr_filling_price: UNDEF_PRICE,
            ind_match_price: UNDEF_PRICE,
            upper_collar: UNDEF_PRICE,
            lower_collar: UNDEF_PRICE,
            paired_qty: UNDEF_ORDER_SIZE,
            total_imbalance_qty: UNDEF_ORDER_SIZE,
            market_imbalance_qty: UNDEF_ORDER_SIZE,
            unpaired_qty: UNDEF_ORDER_SIZE,
            auction_type: b'~' as c_char,
            side: Side::default() as c_char,
            auction_status: 0,
            freeze_status: 0,
            num_extensions: 0,
            unpaired_side: Side::default() as c_char,
            significant_imbalance: b'~' as c_char,
            _reserved: Default::default(),
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

impl Default for ErrorMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::ERROR),
            err: [0; 302],
            code: u8::MAX,
            is_last: u8::MAX,
        }
    }
}

impl Default for SymbolMappingMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::SYMBOL_MAPPING),
            stype_in: u8::MAX,
            stype_in_symbol: [0; SYMBOL_CSTR_LEN],
            stype_out: u8::MAX,
            stype_out_symbol: [0; SYMBOL_CSTR_LEN],
            start_ts: UNDEF_TIMESTAMP,
            end_ts: UNDEF_TIMESTAMP,
        }
    }
}

impl Default for SystemMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::SYSTEM),
            msg: [0; 303],
            code: u8::MAX,
        }
    }
}
