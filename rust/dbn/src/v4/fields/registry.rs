//! The DBN v4 global field-ID registry.
//!
//! Generated from the `[fields]` table in `defs/dbn.toml`. Do not edit by hand;
//! change the TOML and rerun codegen. Field IDs are permanent and assigned once.

use super::{Field, FieldDefault};

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
/// Field ID for `conditions`.
pub const CONDITIONS: Field<crate::v4::types::Char> = Field::new(0x1A);
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
/// Field ID for `ts_out`.
pub const TS_OUT: Field<crate::v4::types::TimestampNs> = Field::new(0x4F);
/// Field ID for `hd`, the record header.
pub const HD: Field<crate::v4::RecordHeader> = Field::new(0x50);
/// Field ID for `levels` with a depth of 1, an array of [`BidAskPair`](crate::BidAskPair).
pub const LEVELS_1: Field<[crate::BidAskPair; 1]> = Field::new(0x51);
/// Field ID for `levels` with a depth of 10, an array of [`BidAskPair`](crate::BidAskPair).
pub const LEVELS_10: Field<[crate::BidAskPair; 10]> = Field::new(0x51);
/// Field ID for `consolidated_levels` with a depth of 1, an array of [`ConsolidatedBidAskPair`](crate::ConsolidatedBidAskPair).
pub const CONSOLIDATED_LEVELS_1: Field<[crate::ConsolidatedBidAskPair; 1]> = Field::new(0x52);
/// Field ID for `expiration`.
pub const EXPIRATION: Field<crate::v4::types::TimestampNs> = Field::new(0x53);
/// Field ID for `activation`.
pub const ACTIVATION: Field<crate::v4::types::TimestampNs> = Field::new(0x54);
/// Field ID for `high_limit_price`.
pub const HIGH_LIMIT_PRICE: Field<crate::v4::types::Decimal> = Field::new(0x55);
/// Field ID for `low_limit_price`.
pub const LOW_LIMIT_PRICE: Field<crate::v4::types::Decimal> = Field::new(0x56);
/// Field ID for `max_price_variation`.
pub const MAX_PRICE_VARIATION: Field<crate::v4::types::Decimal> = Field::new(0x57);
/// Field ID for `unit_of_measure_qty`.
pub const UNIT_OF_MEASURE_QTY: Field<crate::v4::types::Decimal> = Field::new(0x58);
/// Field ID for `min_price_increment_amount`.
pub const MIN_PRICE_INCREMENT_AMOUNT: Field<crate::v4::types::Decimal> = Field::new(0x59);
/// Field ID for `price_ratio`.
pub const PRICE_RATIO: Field<crate::v4::types::Decimal> = Field::new(0x5A);
/// Field ID for `strike_price`.
pub const STRIKE_PRICE: Field<crate::v4::types::Decimal> = Field::new(0x5B);
/// Field ID for `leg_price`.
pub const LEG_PRICE: Field<crate::v4::types::Decimal> = Field::new(0x5C);
/// Field ID for `leg_delta`.
pub const LEG_DELTA: Field<crate::v4::types::Decimal> = Field::new(0x5D);
/// Field ID for `inst_attrib_value`.
pub const INST_ATTRIB_VALUE: Field<i32> = Field::new(0x5E);
/// Field ID for `underlying_id`.
pub const UNDERLYING_ID: Field<u32> = Field::new(0x5F);
/// Field ID for `market_depth_implied`.
pub const MARKET_DEPTH_IMPLIED: Field<i32> = Field::new(0x60);
/// Field ID for `market_depth`.
pub const MARKET_DEPTH: Field<i32> = Field::new(0x61);
/// Field ID for `market_segment_id`.
pub const MARKET_SEGMENT_ID: Field<u32> = Field::new(0x62);
/// Field ID for `max_trade_vol`.
pub const MAX_TRADE_VOL: Field<u32> = Field::new(0x63);
/// Field ID for `min_lot_size`.
pub const MIN_LOT_SIZE: Field<i32> = Field::new(0x64);
/// Field ID for `min_lot_size_block`.
pub const MIN_LOT_SIZE_BLOCK: Field<i32> = Field::new(0x65);
/// Field ID for `min_trade_vol`.
pub const MIN_TRADE_VOL: Field<u32> = Field::new(0x66);
/// Field ID for `contract_multiplier`.
pub const CONTRACT_MULTIPLIER: Field<i32> = Field::new(0x67);
/// Field ID for `decay_quantity`.
pub const DECAY_QUANTITY: Field<i32> = Field::new(0x68);
/// Field ID for `original_contract_size`.
pub const ORIGINAL_CONTRACT_SIZE: Field<i32> = Field::new(0x69);
/// Field ID for `leg_instrument_id`.
pub const LEG_INSTRUMENT_ID: Field<u32> = Field::new(0x6A);
/// Field ID for `leg_ratio_price_numerator`.
pub const LEG_RATIO_PRICE_NUMERATOR: Field<i32> = Field::new(0x6B);
/// Field ID for `leg_ratio_price_denominator`.
pub const LEG_RATIO_PRICE_DENOMINATOR: Field<i32> = Field::new(0x6C);
/// Field ID for `leg_ratio_qty_numerator`.
pub const LEG_RATIO_QTY_NUMERATOR: Field<i32> = Field::new(0x6D);
/// Field ID for `leg_ratio_qty_denominator`.
pub const LEG_RATIO_QTY_DENOMINATOR: Field<i32> = Field::new(0x6E);
/// Field ID for `leg_underlying_id`.
pub const LEG_UNDERLYING_ID: Field<u32> = Field::new(0x6F);
/// Field ID for `appl_id`.
pub const APPL_ID: Field<i16> = Field::new(0x70);
/// Field ID for `maturity_year`.
pub const MATURITY_YEAR: Field<u16> = Field::new(0x71);
/// Field ID for `decay_start_date`.
pub const DECAY_START_DATE: Field<u16> = Field::new(0x72);
/// Field ID for `leg_count`.
pub const LEG_COUNT: Field<u16> = Field::new(0x73);
/// Field ID for `leg_index`.
pub const LEG_INDEX: Field<u16> = Field::new(0x74);
/// Field ID for `currency`.
pub const CURRENCY: Field<crate::v4::types::CStr> = Field::new(0x75);
/// Field ID for `settl_currency`.
pub const SETTL_CURRENCY: Field<crate::v4::types::CStr> = Field::new(0x76);
/// Field ID for `secsubtype`.
pub const SECSUBTYPE: Field<crate::v4::types::CStr> = Field::new(0x77);
/// Field ID for `group`.
pub const GROUP: Field<crate::v4::types::CStr> = Field::new(0x78);
/// Field ID for `exchange`.
pub const EXCHANGE: Field<crate::v4::types::CStr> = Field::new(0x79);
/// Field ID for `asset`.
pub const ASSET: Field<crate::v4::types::CStr> = Field::new(0x7A);
/// Field ID for `cfi`.
pub const CFI: Field<crate::v4::types::CStr> = Field::new(0x7B);
/// Field ID for `security_type`.
pub const SECURITY_TYPE: Field<crate::v4::types::CStr> = Field::new(0x7C);
/// Field ID for `unit_of_measure`.
pub const UNIT_OF_MEASURE: Field<crate::v4::types::CStr> = Field::new(0x7D);
/// Field ID for `underlying`.
pub const UNDERLYING: Field<crate::v4::types::CStr> = Field::new(0x7E);
/// Field ID for `strike_price_currency`.
pub const STRIKE_PRICE_CURRENCY: Field<crate::v4::types::CStr> = Field::new(0x7F);
/// Field ID for `leg_raw_symbol`.
pub const LEG_RAW_SYMBOL: Field<crate::v4::types::CStr> = Field::new(0x80);
/// Field ID for `main_fraction`.
pub const MAIN_FRACTION: Field<u8> = Field::new(0x81);
/// Field ID for `price_display_format`.
pub const PRICE_DISPLAY_FORMAT: Field<u8> = Field::new(0x82);
/// Field ID for `sub_fraction`.
pub const SUB_FRACTION: Field<u8> = Field::new(0x83);
/// Field ID for `underlying_product`.
pub const UNDERLYING_PRODUCT: Field<u8> = Field::new(0x84);
/// Field ID for `maturity_month`.
pub const MATURITY_MONTH: Field<u8> = Field::new(0x85);
/// Field ID for `maturity_day`.
pub const MATURITY_DAY: Field<u8> = Field::new(0x86);
/// Field ID for `maturity_week`.
pub const MATURITY_WEEK: Field<u8> = Field::new(0x87);
/// Field ID for `user_defined_instrument`.
pub const USER_DEFINED_INSTRUMENT: Field<crate::v4::types::OpenEnum<crate::UserDefinedInstrument>> =
    Field::new(0x88);
/// Field ID for `contract_multiplier_unit`.
pub const CONTRACT_MULTIPLIER_UNIT: Field<i8> = Field::new(0x89);
/// Field ID for `flow_schedule_type`.
pub const FLOW_SCHEDULE_TYPE: Field<i8> = Field::new(0x8A);
/// Field ID for `tick_rule`.
pub const TICK_RULE: Field<u8> = Field::new(0x8B);
/// Field ID for `leg_instrument_class`.
pub const LEG_INSTRUMENT_CLASS: Field<crate::v4::types::OpenEnum<crate::InstrumentClass>> =
    Field::new(0x8C);
/// Field ID for `leg_side`.
pub const LEG_SIDE: Field<crate::v4::types::OpenEnum<crate::Side>> = Field::new(0x8D);
/// Field ID for `trading_reference_price`.
pub const TRADING_REFERENCE_PRICE: Field<crate::v4::types::Decimal> = Field::new(0x8E);
/// Field ID for `trading_reference_date`.
pub const TRADING_REFERENCE_DATE: Field<u16> = Field::new(0x8F);
/// Field ID for `md_security_trading_status`.
pub const MD_SECURITY_TRADING_STATUS: Field<u8> = Field::new(0x90);
/// Field ID for `settl_price_type`.
pub const SETTL_PRICE_TYPE: Field<u8> = Field::new(0x91);
/// Field ID for `auction_time`.
pub const AUCTION_TIME: Field<crate::v4::types::TimestampNs> = Field::new(0x92);
/// Field ID for `ssr_filling_price`.
pub const SSR_FILLING_PRICE: Field<crate::v4::types::Decimal> = Field::new(0x93);
/// Field ID for `ind_match_price`.
pub const IND_MATCH_PRICE: Field<crate::v4::types::Decimal> = Field::new(0x94);
/// Field ID for `upper_collar`.
pub const UPPER_COLLAR: Field<crate::v4::types::Decimal> = Field::new(0x95);
/// Field ID for `lower_collar`.
pub const LOWER_COLLAR: Field<crate::v4::types::Decimal> = Field::new(0x96);
/// Field ID for `market_imbalance_qty`.
pub const MARKET_IMBALANCE_QTY: Field<u32> = Field::new(0x97);
/// Field ID for `unpaired_qty`.
pub const UNPAIRED_QTY: Field<u32> = Field::new(0x98);
/// Field ID for `auction_status`.
pub const AUCTION_STATUS: Field<u8> = Field::new(0x99);
/// Field ID for `freeze_status`.
pub const FREEZE_STATUS: Field<u8> = Field::new(0x9A);
/// Field ID for `num_extensions`.
pub const NUM_EXTENSIONS: Field<u8> = Field::new(0x9B);
/// Field ID for `unpaired_side`.
pub const UNPAIRED_SIDE: Field<crate::v4::types::OpenEnum<crate::Side>> = Field::new(0x9C);
/// Field ID for `significant_imbalance`.
pub const SIGNIFICANT_IMBALANCE: Field<crate::v4::types::Char> = Field::new(0x9D);
/// Field ID for `length_v1`.
pub const LENGTH_V1: Field<u8> = Field::new(0x01);
/// Field ID for `rtype_v1`.
pub const RTYPE_V1: Field<u8> = Field::new(0x02);
/// Field ID for `instrument_id_v1`.
pub const INSTRUMENT_ID_V1: Field<u32> = Field::new(0x04);
/// Field ID for `raw_instrument_id_v1`.
pub const RAW_INSTRUMENT_ID_V1: Field<u32> = Field::new(0x32);
/// Field ID for `channel_id_v1`.
pub const CHANNEL_ID_V1: Field<u16> = Field::new(0x1B);
/// Field ID for `quantity_v1`.
pub const QUANTITY_V1: Field<i32> = Field::new(0x3E);
/// Field ID for `raw_symbol_v1`.
pub const RAW_SYMBOL_V1: Field<crate::v4::types::CStr> = Field::new(0x2F);
/// Field ID for `stype_in_symbol_v1`.
pub const STYPE_IN_SYMBOL_V1: Field<crate::v4::types::CStr> = Field::new(0x4B);
/// Field ID for `stype_out_symbol_v1`.
pub const STYPE_OUT_SYMBOL_V1: Field<crate::v4::types::CStr> = Field::new(0x4C);

pub(crate) fn registry_default(field_id: u16) -> Option<FieldDefault> {
    Some(match field_id {
        id if [
            CHANNEL_ID.id(),
            MIN_LOT_SIZE_ROUND_LOT.id(),
            TOTAL_IMBALANCE_QTY.id(),
            PAIRED_QTY.id(),
            QUANTITY.id(),
            SUBSCRIPTION_ID.id(),
            SCHEMA.id(),
            IS_LAST.id(),
            STYPE_IN.id(),
            STYPE_OUT.id(),
            INST_ATTRIB_VALUE.id(),
            MARKET_DEPTH_IMPLIED.id(),
            MARKET_DEPTH.id(),
            MARKET_SEGMENT_ID.id(),
            MAX_TRADE_VOL.id(),
            MIN_LOT_SIZE.id(),
            MIN_LOT_SIZE_BLOCK.id(),
            MIN_TRADE_VOL.id(),
            CONTRACT_MULTIPLIER.id(),
            DECAY_QUANTITY.id(),
            ORIGINAL_CONTRACT_SIZE.id(),
            APPL_ID.id(),
            MATURITY_YEAR.id(),
            DECAY_START_DATE.id(),
            MAIN_FRACTION.id(),
            PRICE_DISPLAY_FORMAT.id(),
            SUB_FRACTION.id(),
            UNDERLYING_PRODUCT.id(),
            MATURITY_MONTH.id(),
            MATURITY_DAY.id(),
            MATURITY_WEEK.id(),
            CONTRACT_MULTIPLIER_UNIT.id(),
            FLOW_SCHEDULE_TYPE.id(),
            TICK_RULE.id(),
            TRADING_REFERENCE_DATE.id(),
            MD_SECURITY_TRADING_STATUS.id(),
            SETTL_PRICE_TYPE.id(),
            MARKET_IMBALANCE_QTY.id(),
            UNPAIRED_QTY.id(),
        ]
        .contains(&id) =>
        {
            FieldDefault::Max
        }
        id if [AUCTION_TYPE.id(), SIGNIFICANT_IMBALANCE.id()].contains(&id) => {
            FieldDefault::Value(b'~' as i64)
        }
        id if [ACTION.id()].contains(&id) => FieldDefault::Value(crate::Action::default() as i64),
        id if [ERROR_CODE.id()].contains(&id) => {
            FieldDefault::Value(crate::ErrorCode::default() as i64)
        }
        id if [MATCH_ALGORITHM.id()].contains(&id) => {
            FieldDefault::Value(crate::MatchAlgorithm::default() as i64)
        }
        id if [SECURITY_UPDATE_ACTION.id()].contains(&id) => {
            FieldDefault::Value(crate::SecurityUpdateAction::default() as i64)
        }
        id if [SIDE.id(), LEG_SIDE.id(), UNPAIRED_SIDE.id()].contains(&id) => {
            FieldDefault::Value(crate::Side::default() as i64)
        }
        id if [UPDATE_ACTION.id()].contains(&id) => {
            FieldDefault::Value(crate::StatUpdateAction::default() as i64)
        }
        id if [STATUS_ACTION.id()].contains(&id) => {
            FieldDefault::Value(crate::StatusAction::default() as i64)
        }
        id if [REASON.id()].contains(&id) => {
            FieldDefault::Value(crate::StatusReason::default() as i64)
        }
        id if [SYSTEM_CODE.id()].contains(&id) => {
            FieldDefault::Value(crate::SystemCode::default() as i64)
        }
        id if [TRADING_EVENT.id()].contains(&id) => {
            FieldDefault::Value(crate::TradingEvent::default() as i64)
        }
        id if [
            IS_TRADING.id(),
            IS_QUOTING.id(),
            IS_SHORT_SELL_RESTRICTED.id(),
        ]
        .contains(&id) =>
        {
            FieldDefault::Value(crate::TriState::default() as i64)
        }
        id if [USER_DEFINED_INSTRUMENT.id()].contains(&id) => {
            FieldDefault::Value(crate::UserDefinedInstrument::default() as i64)
        }
        _ => return None,
    })
}
