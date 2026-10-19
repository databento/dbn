use std::os::raw::c_char;

use crate::{
    Action, ErrorCode, MatchAlgorithm, Schema, SecurityUpdateAction, Side, StatUpdateAction,
    StatusAction, StatusReason, SystemCode, TradingEvent, TriState, UNDEF_PRICE, UNDEF_TIMESTAMP,
};

use super::*;

impl RecordHeader {
    pub(crate) const fn default<R: HasRType<Header = Self>>(rtype: u16) -> Self {
        Self::new::<R>(rtype, 0, 0, UNDEF_TIMESTAMP)
    }
}

impl Default for MboMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::MBO),
            order_id: 0,
            price: UNDEF_PRICE,
            size: 0,
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

impl Default for TradeMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::MBP_0),
            price: UNDEF_PRICE,
            size: 0,
            action: Action::default() as c_char,
            side: Side::default() as c_char,
            flags: Default::default(),
            _reserved: 0,
            ts_recv: UNDEF_TIMESTAMP,
            ts_in_delta: 0,
            sequence: 0,
        }
    }
}

impl Mbp1Msg {
    /// Creates a new default `Mbp1Msg` for the given `schema`.
    pub fn default_for_schema(schema: Schema) -> Self {
        Self {
            hd: RecordHeader::default::<Self>(RType::from(schema) as _),
            price: UNDEF_PRICE,
            size: 0,
            action: Action::default() as c_char,
            side: Side::default() as c_char,
            flags: Default::default(),
            _reserved: 0,
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
            size: 0,
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
            hd: RecordHeader::default::<Self>(RType::from(schema) as _),
            price: UNDEF_PRICE,
            size: 0,
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
            hd: RecordHeader::default::<Self>(RType::from(schema) as _),
            price: UNDEF_PRICE,
            size: 0,
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
            hd: RecordHeader::default::<Self>(RType::from(schema) as _),
            price: UNDEF_PRICE,
            size: 0,
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
            hd: RecordHeader::default::<Self>(RType::from(schema) as _),
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
            raw_instrument_id: 0,
            min_lot_size_round_lot: i32::MAX,
            security_update_action: SecurityUpdateAction::default() as c_char,
            instrument_class: 0,
            match_algorithm: MatchAlgorithm::default() as c_char,
            channel_id: u8::MAX,
        }
    }
}

impl Default for ImbalanceMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::IMBALANCE),
            ts_recv: UNDEF_TIMESTAMP,
            ref_price: UNDEF_PRICE,
            cont_book_clr_price: UNDEF_PRICE,
            auct_interest_clr_price: UNDEF_PRICE,
            paired_qty: u32::MAX,
            total_imbalance_qty: u32::MAX,
            auction_type: b'~' as c_char,
            side: Side::default() as c_char,
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
            quantity: i64::MAX,
            sequence: 0,
            ts_in_delta: 0,
            stat_type: 0,
            channel_id: u8::MAX,
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
            subscription_id: u32::MAX,
            schema: u16::MAX,
            code: ErrorCode::default() as u8,
            is_last: u8::MAX,
            _reserved: Default::default(),
            err: [0; 302],
        }
    }
}

impl Default for SymbolMappingMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::SYMBOL_MAPPING),
            start_ts: UNDEF_TIMESTAMP,
            end_ts: UNDEF_TIMESTAMP,
            stype_in: u8::MAX,
            stype_out: u8::MAX,
            _reserved: Default::default(),
        }
    }
}

impl Default for SystemMsg {
    fn default() -> Self {
        Self {
            hd: RecordHeader::default::<Self>(rtype::SYSTEM),
            subscription_id: u32::MAX,
            schema: u16::MAX,
            code: SystemCode::default() as u8,
            _reserved: Default::default(),
            msg: [0; 303],
        }
    }
}
