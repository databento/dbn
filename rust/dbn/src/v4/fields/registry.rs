//! The DBN v4 global field-ID registry.
//!
//! Generated from the `[fields]` table in `defs/dbn.toml`. Do not edit by hand;
//! change the TOML and rerun codegen. Field IDs are permanent and assigned once.

use super::Field;

/// Field ID for `length`.
pub const LENGTH: Field<u16> = Field::new(0x01);
/// Field ID for `rtype`.
pub const RTYPE: Field<u16> = Field::new(0x02);
/// Field ID for `publisher_id`.
pub const PUBLISHER_ID: Field<u16> = Field::new(0x03);
/// Field ID for `instrument_id`.
pub const INSTRUMENT_ID: Field<u64> = Field::new(0x04);
/// Field ID for `ts_event`.
pub const TS_EVENT: Field<crate::v4::types::TimestampNs> = Field::new(0x05);
/// Field ID for `price`.
pub const PRICE: Field<crate::v4::types::Decimal> = Field::new(0x10);
/// Field ID for `size`.
pub const SIZE: Field<u32> = Field::new(0x11);
/// Field ID for `action`.
pub const ACTION: Field<crate::v4::types::OpenEnum<crate::Action>> = Field::new(0x12);
/// Field ID for `side`.
pub const SIDE: Field<crate::v4::types::OpenEnum<crate::Side>> = Field::new(0x13);
/// Field ID for `flags`.
pub const FLAGS: Field<crate::FlagSet> = Field::new(0x14);
/// Field ID for `depth`.
pub const DEPTH: Field<u8> = Field::new(0x15);
/// Field ID for `ts_recv`.
pub const TS_RECV: Field<crate::v4::types::TimestampNs> = Field::new(0x16);
/// Field ID for `ts_in_delta`.
pub const TS_IN_DELTA: Field<i32> = Field::new(0x17);
/// Field ID for `sequence`.
pub const SEQUENCE: Field<u32> = Field::new(0x18);
/// Field ID for `order_id`.
pub const ORDER_ID: Field<u64> = Field::new(0x19);
/// Field ID for `channel_id`.
pub const CHANNEL_ID: Field<u8> = Field::new(0x1B);
/// Field ID for `bid_px`.
pub const BID_PX: Field<crate::v4::types::Decimal> = Field::new(0x1C);
/// Field ID for `ask_px`.
pub const ASK_PX: Field<crate::v4::types::Decimal> = Field::new(0x1D);
/// Field ID for `bid_sz`.
pub const BID_SZ: Field<u32> = Field::new(0x1E);
/// Field ID for `ask_sz`.
pub const ASK_SZ: Field<u32> = Field::new(0x1F);
/// Field ID for `bid_ct`.
pub const BID_CT: Field<u32> = Field::new(0x20);
/// Field ID for `ask_ct`.
pub const ASK_CT: Field<u32> = Field::new(0x21);
/// Field ID for `bid_pb`.
pub const BID_PB: Field<u16> = Field::new(0x22);
/// Field ID for `ask_pb`.
pub const ASK_PB: Field<u16> = Field::new(0x23);
/// Field ID for `open`.
pub const OPEN: Field<crate::v4::types::Decimal> = Field::new(0x24);
/// Field ID for `high`.
pub const HIGH: Field<crate::v4::types::Decimal> = Field::new(0x25);
/// Field ID for `low`.
pub const LOW: Field<crate::v4::types::Decimal> = Field::new(0x26);
/// Field ID for `close`.
pub const CLOSE: Field<crate::v4::types::Decimal> = Field::new(0x27);
/// Field ID for `volume`.
pub const VOLUME: Field<u64> = Field::new(0x28);
/// Field ID for `status_action`.
pub const STATUS_ACTION: Field<crate::v4::types::OpenEnum<crate::StatusAction>> = Field::new(0x29);
/// Field ID for `reason`.
pub const REASON: Field<crate::v4::types::OpenEnum<crate::StatusReason>> = Field::new(0x2A);
/// Field ID for `trading_event`.
pub const TRADING_EVENT: Field<crate::v4::types::OpenEnum<crate::TradingEvent>> = Field::new(0x2B);
/// Field ID for `is_trading`.
pub const IS_TRADING: Field<crate::v4::types::OpenEnum<crate::TriState>> = Field::new(0x2C);
/// Field ID for `is_quoting`.
pub const IS_QUOTING: Field<crate::v4::types::OpenEnum<crate::TriState>> = Field::new(0x2D);
/// Field ID for `is_short_sell_restricted`.
pub const IS_SHORT_SELL_RESTRICTED: Field<crate::v4::types::OpenEnum<crate::TriState>> =
    Field::new(0x2E);
/// Field ID for `raw_symbol`.
pub const RAW_SYMBOL: Field<crate::v4::types::Symbol> = Field::new(0x2F);
/// Field ID for `min_price_increment`.
pub const MIN_PRICE_INCREMENT: Field<crate::v4::types::Decimal> = Field::new(0x30);
/// Field ID for `display_factor`.
pub const DISPLAY_FACTOR: Field<crate::v4::types::Decimal> = Field::new(0x31);
/// Field ID for `raw_instrument_id`.
pub const RAW_INSTRUMENT_ID: Field<u64> = Field::new(0x32);
/// Field ID for `min_lot_size_round_lot`.
pub const MIN_LOT_SIZE_ROUND_LOT: Field<i32> = Field::new(0x33);
/// Field ID for `security_update_action`.
pub const SECURITY_UPDATE_ACTION: Field<crate::v4::types::OpenEnum<crate::SecurityUpdateAction>> =
    Field::new(0x34);
/// Field ID for `instrument_class`.
pub const INSTRUMENT_CLASS: Field<crate::v4::types::OpenEnum<crate::InstrumentClass>> =
    Field::new(0x35);
/// Field ID for `match_algorithm`.
pub const MATCH_ALGORITHM: Field<crate::v4::types::OpenEnum<crate::MatchAlgorithm>> =
    Field::new(0x36);
/// Field ID for `ref_price`.
pub const REF_PRICE: Field<crate::v4::types::Decimal> = Field::new(0x37);
/// Field ID for `auct_interest_clr_price`.
pub const AUCT_INTEREST_CLR_PRICE: Field<crate::v4::types::Decimal> = Field::new(0x38);
/// Field ID for `cont_book_clr_price`.
pub const CONT_BOOK_CLR_PRICE: Field<crate::v4::types::Decimal> = Field::new(0x39);
/// Field ID for `total_imbalance_qty`.
pub const TOTAL_IMBALANCE_QTY: Field<u32> = Field::new(0x3A);
/// Field ID for `paired_qty`.
pub const PAIRED_QTY: Field<u32> = Field::new(0x3B);
/// Field ID for `auction_type`.
pub const AUCTION_TYPE: Field<crate::v4::types::Char> = Field::new(0x3C);
/// Field ID for `ts_ref`.
pub const TS_REF: Field<crate::v4::types::TimestampNs> = Field::new(0x3D);
/// Field ID for `quantity`.
pub const QUANTITY: Field<i64> = Field::new(0x3E);
/// Field ID for `stat_type`.
pub const STAT_TYPE: Field<crate::v4::types::OpenEnum<crate::StatType>> = Field::new(0x3F);
/// Field ID for `update_action`.
pub const UPDATE_ACTION: Field<crate::v4::types::OpenEnum<crate::StatUpdateAction>> =
    Field::new(0x40);
/// Field ID for `stat_flags`.
pub const STAT_FLAGS: Field<u8> = Field::new(0x41);
/// Field ID for `subscription_id`.
pub const SUBSCRIPTION_ID: Field<u32> = Field::new(0x42);
/// Field ID for `schema`.
pub const SCHEMA: Field<crate::v4::types::OpenEnum<crate::Schema>> = Field::new(0x43);
/// Field ID for `err`.
pub const ERR: Field<crate::v4::types::CStr> = Field::new(0x44);
/// Field ID for `error_code`.
pub const ERROR_CODE: Field<crate::v4::types::OpenEnum<crate::ErrorCode>> = Field::new(0x45);
/// Field ID for `is_last`.
pub const IS_LAST: Field<u8> = Field::new(0x46);
/// Field ID for `msg`.
pub const MSG: Field<crate::v4::types::CStr> = Field::new(0x47);
/// Field ID for `system_code`.
pub const SYSTEM_CODE: Field<crate::v4::types::OpenEnum<crate::SystemCode>> = Field::new(0x48);
/// Field ID for `stype_in`.
pub const STYPE_IN: Field<crate::v4::types::OpenEnum<crate::SType>> = Field::new(0x49);
/// Field ID for `stype_out`.
pub const STYPE_OUT: Field<crate::v4::types::OpenEnum<crate::SType>> = Field::new(0x4A);
/// Field ID for `stype_in_symbol`.
pub const STYPE_IN_SYMBOL: Field<crate::v4::types::Symbol> = Field::new(0x4B);
/// Field ID for `stype_out_symbol`.
pub const STYPE_OUT_SYMBOL: Field<crate::v4::types::Symbol> = Field::new(0x4C);
/// Field ID for `start_ts`.
pub const START_TS: Field<crate::v4::types::TimestampNs> = Field::new(0x4D);
/// Field ID for `end_ts`.
pub const END_TS: Field<crate::v4::types::TimestampNs> = Field::new(0x4E);
/// Field ID for `hd`, the record header.
pub const HD: Field<crate::v4::RecordHeader> = Field::new(0x1000);
/// Field ID for `levels` with a depth of 1, an array of [`BidAskPair`](crate::BidAskPair).
pub const LEVELS_1: Field<[crate::BidAskPair; 1]> = Field::new(0x1001);
/// Field ID for `levels` with a depth of 10, an array of [`BidAskPair`](crate::BidAskPair).
pub const LEVELS_10: Field<[crate::BidAskPair; 10]> = Field::new(0x1001);
/// Field ID for `consolidated_levels` with a depth of 1, an array of [`ConsolidatedBidAskPair`](crate::ConsolidatedBidAskPair).
pub const CONSOLIDATED_LEVELS_1: Field<[crate::ConsolidatedBidAskPair; 1]> = Field::new(0x1002);
