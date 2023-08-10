//! Market data types for encoding different Databento [`Schema`](crate::enums::Schema)s
//! and conversion functions.

use std::{ffi::CStr, mem, os::raw::c_char, ptr::NonNull, slice};

// Dummy derive macro to get around `cfg_attr` incompatibility of several
// of pyo3's attribute macros. See https://github.com/PyO3/pyo3/issues/780
#[cfg(not(feature = "python"))]
use dbn_macros::MockPyo3;

use crate::{
    enums::{
        rtype::{self, RType},
        Action, InstrumentClass, MatchAlgorithm, SecurityUpdateAction, Side, StatType,
        StatUpdateAction, UserDefinedInstrument,
    },
    macros::{dbn_record, CsvSerialize, JsonSerialize},
    Error, Result,
};

/// Common data for all Databento records.
#[repr(C)]
#[derive(Clone, Debug, CsvSerialize, JsonSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
pub struct RecordHeader {
    /// The length of the record in 32-bit words.
    #[dbn(skip)]
    pub(crate) length: u8,
    /// The record type; with `0xe0..0x0F` specifying MBP levels size. Record types
    /// implement the trait [`HasRType`], and the [`has_rtype`][HasRType::has_rtype]
    /// function can be used to check if that type can be used to decode a message with
    /// a given rtype. The set of possible values is defined in [`rtype`].
    pub rtype: u8,
    /// The publisher ID assigned by Databento.
    pub publisher_id: u16,
    /// The numeric ID assigned to the instrument.
    pub instrument_id: u32,
    /// The matching-engine-received timestamp expressed as number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(encode_order(0), unix_nanos)]
    pub ts_event: u64,
}

/// A market-by-order (MBO) tick message. The record of the
/// [`Mbo`](crate::enums::Schema::Mbo) schema.
#[repr(C)]
#[derive(Clone, Debug, CsvSerialize, JsonSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(set_all, dict, module = "databento_dbn", name = "MBOMsg"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[dbn_record(rtype::MBO)]
pub struct MboMsg {
    /// The common header.
    #[pyo3(get)]
    pub hd: RecordHeader,
    /// The order ID assigned at the venue.
    #[pyo3(get)]
    pub order_id: u64,
    /// The order price expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(encode_order(4), fixed_price)]
    #[pyo3(get)]
    pub price: i64,
    /// The order quantity.
    #[dbn(encode_order(5))]
    #[pyo3(get)]
    pub size: u32,
    /// A combination of packet end with matching engine status. See
    /// [`enums::flags`](crate::enums::flags) for possible values.
    #[pyo3(get)]
    pub flags: u8,
    /// A channel ID within the venue.
    #[dbn(encode_order(6))]
    #[pyo3(get)]
    pub channel_id: u8,
    /// The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R**,
    /// **T**rade, or **F**ill.
    #[dbn(c_char, encode_order(2))]
    pub action: c_char,
    /// The order side. Can be **A**sk, **B**id or **N**one.
    #[dbn(c_char, encode_order(3))]
    pub side: c_char,
    /// The capture-server-received timestamp expressed as number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(encode_order(0), unix_nanos)]
    #[pyo3(get)]
    pub ts_recv: u64,
    /// The delta of `ts_recv - ts_exchange_send`, max 2 seconds.
    #[pyo3(get)]
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    #[pyo3(get)]
    pub sequence: u32,
}

/// A level.
#[repr(C)]
#[derive(Clone, Debug, JsonSerialize, PartialEq, Eq, Default)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
pub struct BidAskPair {
    /// The bid price.
    #[dbn(fixed_price)]
    pub bid_px: i64,
    /// The ask price.
    #[dbn(fixed_price)]
    pub ask_px: i64,
    /// The bid size.
    pub bid_sz: u32,
    /// The ask size.
    pub ask_sz: u32,
    /// The bid order count.
    pub bid_ct: u32,
    /// The ask order count.
    pub ask_ct: u32,
}

/// Market by price implementation with a book depth of 0. Equivalent to
/// MBP-0. The record of the [`Trades`](crate::enums::Schema::Trades) schema.
#[repr(C)]
#[derive(Clone, Debug, CsvSerialize, JsonSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(set_all, dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[dbn_record(rtype::MBP_0)]
pub struct TradeMsg {
    /// The common header.
    #[pyo3(get)]
    pub hd: RecordHeader,
    /// The order price expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get)]
    pub price: i64,
    /// The order quantity.
    #[pyo3(get)]
    pub size: u32,
    /// The event action. Always **T**rade in the trades schema.
    #[dbn(c_char, encode_order(2))]
    pub action: c_char,
    /// The aggressing order's side in the trade. Can be **A**sk, **B**id or **N**one.
    #[dbn(c_char, encode_order(3))]
    pub side: c_char,
    /// A combination of packet end with matching engine status. See
    /// [`enums::flags`](crate::enums::flags) for possible values.
    #[pyo3(get)]
    pub flags: u8,
    /// The depth of actual book change.
    #[dbn(encode_order(4))]
    #[pyo3(get)]
    pub depth: u8,
    /// The capture-server-received timestamp expressed as number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(encode_order(0), unix_nanos)]
    #[pyo3(get)]
    pub ts_recv: u64,
    /// The delta of `ts_recv - ts_exchange_send`, max 2 seconds.
    #[pyo3(get)]
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    #[pyo3(get)]
    pub sequence: u32,
}

/// Market by price implementation with a known book depth of 1. The record of the
/// [`Mbp1`](crate::enums::Schema::Mbp1) schema.
#[repr(C)]
#[derive(Clone, Debug, CsvSerialize, JsonSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(set_all, dict, module = "databento_dbn", name = "MBP1Msg"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[dbn_record(rtype::MBP_1)]
pub struct Mbp1Msg {
    /// The common header.
    #[pyo3(get)]
    pub hd: RecordHeader,
    /// The order price expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get)]
    pub price: i64,
    /// The order quantity.
    #[pyo3(get)]
    pub size: u32,
    /// The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R**, or
    /// **T**rade.
    #[dbn(c_char, encode_order(2))]
    pub action: c_char,
    /// The order side. Can be **A**sk, **B**id or **N**one.
    #[dbn(c_char, encode_order(3))]
    pub side: c_char,
    /// A combination of packet end with matching engine status. See
    /// [`enums::flags`](crate::enums::flags) for possible values.
    #[pyo3(get)]
    pub flags: u8,
    /// The depth of actual book change.
    #[dbn(encode_order(4))]
    #[pyo3(get)]
    pub depth: u8,
    /// The capture-server-received timestamp expressed as number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(encode_order(0), unix_nanos)]
    #[pyo3(get)]
    pub ts_recv: u64,
    /// The delta of `ts_recv - ts_exchange_send`, max 2 seconds.
    #[pyo3(get)]
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    #[pyo3(get)]
    pub sequence: u32,
    /// The top of the order book.
    #[pyo3(get)]
    pub levels: [BidAskPair; 1],
}

/// Market by price implementation with a known book depth of 10. The record of the
/// [`Mbp10`](crate::enums::Schema::Mbp10) schema.
#[repr(C)]
#[derive(Clone, Debug, CsvSerialize, JsonSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(set_all, dict, module = "databento_dbn", name = "MBP10Msg"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[dbn_record(rtype::MBP_10)]
pub struct Mbp10Msg {
    /// The common header.
    #[pyo3(get)]
    pub hd: RecordHeader,
    /// The order price expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get)]
    pub price: i64,
    /// The order quantity.
    #[pyo3(get)]
    pub size: u32,
    /// The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R**, or
    /// **T**rade.
    #[dbn(c_char, encode_order(2))]
    pub action: c_char,
    /// The order side. Can be **A**sk, **B**id or **N**one.
    #[dbn(c_char, encode_order(3))]
    pub side: c_char,
    /// A combination of packet end with matching engine status. See
    /// [`enums::flags`](crate::enums::flags) for possible values.
    #[pyo3(get)]
    pub flags: u8,
    /// The depth of actual book change.
    #[dbn(encode_order(4))]
    #[pyo3(get)]
    pub depth: u8,
    /// The capture-server-received timestamp expressed as number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(encode_order(0), unix_nanos)]
    #[pyo3(get)]
    pub ts_recv: u64,
    /// The delta of `ts_recv - ts_exchange_send`, max 2 seconds.
    #[pyo3(get)]
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    #[pyo3(get)]
    pub sequence: u32,
    /// The top 10 levels of the order book.
    #[pyo3(get)]
    pub levels: [BidAskPair; 10],
}

/// The record of the [`Tbbo`](crate::enums::Schema::Tbbo) schema.
pub type TbboMsg = Mbp1Msg;

/// Open, high, low, close, and volume. The record of the following schemas:
/// - [`Ohlcv1S`](crate::enums::Schema::Ohlcv1S)
/// - [`Ohlcv1M`](crate::enums::Schema::Ohlcv1M)
/// - [`Ohlcv1H`](crate::enums::Schema::Ohlcv1H)
/// - [`Ohlcv1D`](crate::enums::Schema::Ohlcv1D)
/// - [`OhlcvEod`](crate::enums::Schema::OhlcvEod)
#[repr(C)]
#[derive(Clone, Debug, CsvSerialize, JsonSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, dict, module = "databento_dbn", name = "OHLCVMsg"),
    derive(crate::macros::PyFieldDesc)
)]
#[dbn_record(
    rtype::OHLCV_1S,
    rtype::OHLCV_1M,
    rtype::OHLCV_1H,
    rtype::OHLCV_1D,
    rtype::OHLCV_EOD,
    rtype::OHLCV_DEPRECATED
)]
pub struct OhlcvMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The open price for the bar.
    #[dbn(fixed_price)]
    pub open: i64,
    /// The high price for the bar.
    #[dbn(fixed_price)]
    pub high: i64,
    /// The low price for the bar.
    #[dbn(fixed_price)]
    pub low: i64,
    /// The close price for the bar.
    #[dbn(fixed_price)]
    pub close: i64,
    /// The total volume traded during the aggregation period.
    pub volume: u64,
}

/// Trading status update message. The record of the
/// [`Status`](crate::enums::Schema::Status) schema.
#[doc(hidden)]
#[repr(C)]
#[derive(Clone, Debug, CsvSerialize, JsonSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[dbn_record(rtype::STATUS)]
pub struct StatusMsg {
    /// The common header.
    #[pyo3(get, set)]
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    pub group: [c_char; 21],
    #[pyo3(get, set)]
    pub trading_status: u8,
    #[pyo3(get, set)]
    pub halt_reason: u8,
    #[pyo3(get, set)]
    pub trading_event: u8,
}

/// Definition of an instrument. The record of the
/// [`Definition`](crate::enums::Schema::Definition) schema.
#[repr(C)]
#[derive(Clone, Debug, CsvSerialize, JsonSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[dbn_record(rtype::INSTRUMENT_DEF)]
pub struct InstrumentDefMsg {
    /// The common header.
    #[pyo3(get, set)]
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as number of nanoseconds since the
    /// UNIX epoch.
    #[dbn(encode_order(0), unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The minimum constant tick for the instrument in units of 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub min_price_increment: i64,
    /// The multiplier to convert the venue’s display price to the conventional price.
    #[pyo3(get, set)]
    pub display_factor: i64,
    /// The last eligible trade time expressed as a number of nanoseconds since the
    /// UNIX epoch. Will be [`crate::UNDEF_TIMESTAMP`] when null, such as for equities.
    #[dbn(unix_nanos)]
    #[pyo3(get, set)]
    pub expiration: u64,
    /// The time of instrument activation expressed as a number of nanoseconds since the
    /// UNIX epoch. Will be [`crate::UNDEF_TIMESTAMP`] when null, such as for equities.
    #[dbn(unix_nanos)]
    #[pyo3(get, set)]
    pub activation: u64,
    /// The allowable high limit price for the trading day in units of 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub high_limit_price: i64,
    /// The allowable low limit price for the trading day in units of 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub low_limit_price: i64,
    /// The differential value for price banding in units of 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub max_price_variation: i64,
    /// The trading session settlement price on `trading_reference_date`.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub trading_reference_price: i64,
    /// The contract size for each instrument, in combination with `unit_of_measure`.
    #[pyo3(get, set)]
    pub unit_of_measure_qty: i64,
    /// The value currently under development by the venue. Converted to units of 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub min_price_increment_amount: i64,
    /// The value used for price calculation in spread and leg pricing in units of 1e-9,
    /// i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub price_ratio: i64,
    /// A bitmap of instrument eligibility attributes.
    #[pyo3(get, set)]
    pub inst_attrib_value: i32,
    /// The `instrument_id` of the first underlying instrument.
    #[pyo3(get, set)]
    pub underlying_id: u32,
    /// The instrument ID assigned by the publisher. May be the same as `instrument_id`.
    #[pyo3(get, set)]
    pub raw_instrument_id: u32,
    /// The implied book depth on the price level data feed.
    #[pyo3(get, set)]
    pub market_depth_implied: i32,
    /// The (outright) book depth on the price level data feed.
    #[pyo3(get, set)]
    pub market_depth: i32,
    /// The market segment of the instrument.
    #[pyo3(get, set)]
    pub market_segment_id: u32,
    /// The maximum trading volume for the instrument.
    #[pyo3(get, set)]
    pub max_trade_vol: u32,
    /// The minimum order entry quantity for the instrument.
    #[pyo3(get, set)]
    pub min_lot_size: i32,
    /// The minimum quantity required for a block trade of the instrument.
    #[pyo3(get, set)]
    pub min_lot_size_block: i32,
    /// The minimum quantity required for a round lot of the instrument. Multiples of
    /// this quantity are also round lots.
    #[pyo3(get, set)]
    pub min_lot_size_round_lot: i32,
    /// The minimum trading volume for the instrument.
    #[pyo3(get, set)]
    pub min_trade_vol: u32,
    #[doc(hidden)]
    pub _reserved2: [u8; 4],
    /// The number of deliverables per instrument, i.e. peak days.
    #[pyo3(get, set)]
    pub contract_multiplier: i32,
    /// The quantity that a contract will decay daily, after `decay_start_date` has
    /// been reached.
    #[pyo3(get, set)]
    pub decay_quantity: i32,
    /// The fixed contract value assigned to each instrument.
    #[pyo3(get, set)]
    pub original_contract_size: i32,
    #[doc(hidden)]
    pub _reserved3: [u8; 4],
    /// The trading session date corresponding to the settlement price in
    /// `trading_reference_price`, in number of days since the UNIX epoch.
    #[pyo3(get, set)]
    pub trading_reference_date: u16,
    /// The channel ID assigned at the venue.
    #[pyo3(get, set)]
    pub appl_id: i16,
    /// The calendar year reflected in the instrument symbol.
    #[pyo3(get, set)]
    pub maturity_year: u16,
    /// The date at which a contract will begin to decay.
    #[pyo3(get, set)]
    pub decay_start_date: u16,
    /// The channel ID assigned by Databento as an incrementing integer starting at zero.
    #[pyo3(get, set)]
    pub channel_id: u16,
    /// The currency used for price fields.
    pub currency: [c_char; 4],
    /// The currency used for settlement, if different from `currency`.
    pub settl_currency: [c_char; 4],
    /// The strategy type of the spread.
    pub secsubtype: [c_char; 6],
    /// The instrument raw symbol assigned by the publisher.
    #[dbn(encode_order(2))]
    pub raw_symbol: [c_char; 22],
    /// The security group code of the instrument.
    pub group: [c_char; 21],
    /// The exchange used to identify the instrument.
    pub exchange: [c_char; 5],
    /// The underlying asset code (product code) of the instrument.
    pub asset: [c_char; 7],
    /// The ISO standard instrument categorization code.
    pub cfi: [c_char; 7],
    /// The type of the instrument, e.g. FUT for future or future spread.
    pub security_type: [c_char; 7],
    /// The unit of measure for the instrument’s original contract size, e.g. USD or LBS.
    pub unit_of_measure: [c_char; 31],
    /// The symbol of the first underlying instrument.
    pub underlying: [c_char; 21],
    /// The currency of [`strike_price`](Self::strike_price).
    pub strike_price_currency: [c_char; 4],
    /// The classification of the instrument.
    #[dbn(c_char, encode_order(4))]
    #[pyo3(set)]
    pub instrument_class: c_char,
    #[doc(hidden)]
    pub _reserved4: [u8; 2],
    /// The strike price of the option. Converted to units of 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub strike_price: i64,
    #[doc(hidden)]
    pub _reserved5: [u8; 6],
    /// The matching algorithm used for the instrument, typically **F**IFO.
    #[dbn(c_char)]
    #[pyo3(set)]
    pub match_algorithm: c_char,
    /// The current trading state of the instrument.
    #[pyo3(get, set)]
    pub md_security_trading_status: u8,
    /// The price denominator of the main fraction.
    #[pyo3(get, set)]
    pub main_fraction: u8,
    ///  The number of digits to the right of the tick mark, to display fractional prices.
    #[pyo3(get, set)]
    pub price_display_format: u8,
    /// The type indicators for the settlement price, as a bitmap.
    #[pyo3(get, set)]
    pub settl_price_type: u8,
    /// The price denominator of the sub fraction.
    #[pyo3(get, set)]
    pub sub_fraction: u8,
    /// The product complex of the instrument.
    #[pyo3(get, set)]
    pub underlying_product: u8,
    /// Indicates if the instrument definition has been added, modified, or deleted.
    #[dbn(encode_order(3))]
    #[pyo3(set)]
    pub security_update_action: SecurityUpdateAction,
    /// The calendar month reflected in the instrument symbol.
    #[pyo3(get, set)]
    pub maturity_month: u8,
    /// The calendar day reflected in the instrument symbol, or 0.
    #[pyo3(get, set)]
    pub maturity_day: u8,
    /// The calendar week reflected in the instrument symbol, or 0.
    #[pyo3(get, set)]
    pub maturity_week: u8,
    /// Indicates if the instrument is user defined: **Y**es or **N**o.
    #[pyo3(set)]
    pub user_defined_instrument: UserDefinedInstrument,
    /// The type of `contract_multiplier`. Either `1` for hours, or `2` for days.
    #[pyo3(get, set)]
    pub contract_multiplier_unit: i8,
    /// The schedule for delivering electricity.
    #[pyo3(get, set)]
    pub flow_schedule_type: i8,
    /// The tick rule of the spread.
    #[pyo3(get, set)]
    pub tick_rule: u8,
    // Filler for alignment.
    #[doc(hidden)]
    pub _dummy: [u8; 3],
}

/// An auction imbalance message.
#[repr(C)]
#[derive(Clone, Debug, CsvSerialize, JsonSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(set_all, dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[dbn_record(rtype::IMBALANCE)]
pub struct ImbalanceMsg {
    /// The common header.
    #[pyo3(get)]
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    #[dbn(encode_order(0), unix_nanos)]
    #[pyo3(get)]
    pub ts_recv: u64,
    /// The price at which the imbalance shares are calculated, where every 1 unit corresponds to
    /// 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get)]
    pub ref_price: i64,
    /// Reserved for future use.
    #[pyo3(get)]
    pub auction_time: u64,
    /// The hypothetical auction-clearing price for both cross and continuous orders.
    #[dbn(fixed_price)]
    #[pyo3(get)]
    pub cont_book_clr_price: i64,
    /// The hypothetical auction-clearing price for cross orders only.
    #[dbn(fixed_price)]
    #[pyo3(get)]
    pub auct_interest_clr_price: i64,
    /// Reserved for future use.
    #[dbn(fixed_price)]
    #[pyo3(get)]
    pub ssr_filling_price: i64,
    /// Reserved for future use.
    #[dbn(fixed_price)]
    #[pyo3(get)]
    pub ind_match_price: i64,
    /// Reserved for future use.
    #[dbn(fixed_price)]
    #[pyo3(get)]
    pub upper_collar: i64,
    /// Reserved for future use.
    #[dbn(fixed_price)]
    #[pyo3(get)]
    pub lower_collar: i64,
    /// The quantity of shares that are eligible to be matched at `ref_price`.
    #[pyo3(get)]
    pub paired_qty: u32,
    /// The quantity of shares that are not paired at `ref_price`.
    #[pyo3(get)]
    pub total_imbalance_qty: u32,
    /// Reserved for future use.
    #[pyo3(get)]
    pub market_imbalance_qty: u32,
    /// Reserved for future use.
    #[pyo3(get)]
    pub unpaired_qty: u32,
    /// Venue-specific character code indicating the auction type.
    #[dbn(c_char)]
    pub auction_type: c_char,
    /// The market side of the `total_imbalance_qty`. Can be **A**sk, **B**id, or **N**one.
    #[dbn(c_char)]
    pub side: c_char,
    /// Reserved for future use.
    #[pyo3(get)]
    pub auction_status: u8,
    /// Reserved for future use.
    #[pyo3(get)]
    pub freeze_status: u8,
    /// Reserved for future use.
    #[pyo3(get)]
    pub num_extensions: u8,
    /// Reserved for future use.
    #[dbn(c_char)]
    pub unpaired_side: c_char,
    /// Venue-specific character code. For Nasdaq, contains the raw Price Variation Indicator.
    #[dbn(c_char)]
    pub significant_imbalance: c_char,
    // Filler for alignment.
    #[doc(hidden)]
    pub _dummy: [u8; 1],
}

/// A statistics message. A catchall for various data disseminated by publishers.
/// The [`stat_type`](Self::stat_type) indicates the statistic contained in the message.
#[repr(C)]
#[derive(Clone, Debug, CsvSerialize, JsonSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[dbn_record(rtype::STATISTICS)]
pub struct StatMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    #[dbn(encode_order(0), unix_nanos)]
    pub ts_recv: u64,
    /// Reference timestamp expressed as the number of nanoseconds since the UNIX epoch.
    /// Will be [`crate::UNDEF_TIMESTAMP`] when unused.
    #[dbn(unix_nanos)]
    pub ts_ref: u64,
    /// The value for price statistics expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001. Will be
    /// [`crate::UNDEF_PRICE`] when unused.
    #[dbn(fixed_price)]
    pub price: i64,
    /// The value for non-price statistics. Will be [`crate::UNDEF_STAT_QUANTITY`] when
    /// unused.
    pub quantity: i32,
    /// The message sequence number assigned at the venue.
    pub sequence: u32,
    /// The delta of `ts_recv - ts_exchange_send`, max 2 seconds.
    pub ts_in_delta: i32,
    /// The type of statistic value contained in the message. Refer to the
    /// [`StatType`](crate::enums::StatType) for variants.
    pub stat_type: u16,
    /// A channel ID within the venue.
    pub channel_id: u16,
    /// Indicates if the statistic is newly added (1) or deleted (2). (Deleted is only used with
    /// some stat types)
    pub update_action: u8,
    /// Additional flags associate with certain stat types.
    pub stat_flags: u8,
    // Filler for alignment
    #[doc(hidden)]
    pub _dummy: [u8; 6],
}

/// An error message from the Databento Live Subscription Gateway (LSG).
#[repr(C)]
#[derive(Clone, Debug, CsvSerialize, JsonSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[dbn_record(rtype::ERROR)]
pub struct ErrorMsg {
    /// The common header.
    #[pyo3(get, set)]
    pub hd: RecordHeader,
    /// The error message.
    pub err: [c_char; 64],
}

/// A symbol mapping message which maps a symbol of one [`SType`](crate::enums::SType)
/// to another.
#[repr(C)]
#[derive(Clone, Debug, CsvSerialize, JsonSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[dbn_record(rtype::SYMBOL_MAPPING)]
pub struct SymbolMappingMsg {
    /// The common header.
    #[pyo3(get, set)]
    pub hd: RecordHeader,
    /// The input symbol.
    pub stype_in_symbol: [c_char; 22],
    /// The output symbol.
    pub stype_out_symbol: [c_char; 22],
    // Filler for alignment.
    #[doc(hidden)]
    pub _dummy: [u8; 4],
    /// The start of the mapping interval expressed as the number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(unix_nanos)]
    #[pyo3(get, set)]
    pub start_ts: u64,
    /// The end of the mapping interval expressed as the number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(unix_nanos)]
    #[pyo3(get, set)]
    pub end_ts: u64,
}

/// A non-error message from the Databento Live Subscription Gateway (LSG). Also used
/// for heartbeating.
#[repr(C)]
#[derive(Clone, Debug, CsvSerialize, JsonSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[dbn_record(rtype::SYSTEM)]
pub struct SystemMsg {
    /// The common header.
    #[pyo3(get, set)]
    pub hd: RecordHeader,
    /// The message from the Databento Live Subscription Gateway (LSG).
    pub msg: [c_char; 64],
}

/// A trait for objects with polymorphism based around [`RecordHeader::rtype`]. All implementing
/// types begin with a [`RecordHeader`].
pub trait HasRType {
    /// Returns `true` if `rtype` matches the value associated with the implementing type.
    fn has_rtype(rtype: u8) -> bool;

    /// Returns a reference to the `RecordHeader` that comes at the beginning of all
    /// record types.
    fn header(&self) -> &RecordHeader;

    /// Returns a mutable reference to the `RecordHeader` that comes at the beginning of
    /// all record types.
    fn header_mut(&mut self) -> &mut RecordHeader;

    /// Returns the size of the record in bytes.
    fn record_size(&self) -> usize {
        self.header().record_size()
    }

    /// Tries to convert the raw `rtype` into an enum which is useful for exhaustive
    /// pattern matching.
    ///
    /// # Errors
    /// This function returns an error if the `rtype` field does not
    /// contain a valid, known [`RType`](crate::enums::RType).
    fn rtype(&self) -> crate::error::Result<RType> {
        self.header().rtype()
    }
}

impl RecordHeader {
    /// The multiplier for converting the `length` field to the number of bytes.
    pub const LENGTH_MULTIPLIER: usize = 4;

    /// Creates a new `RecordHeader`. `R` and `rtype` should be compatible.
    pub const fn new<R: HasRType>(
        rtype: u8,
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
    ) -> Self {
        Self {
            length: (mem::size_of::<R>() / Self::LENGTH_MULTIPLIER) as u8,
            rtype,
            publisher_id,
            instrument_id,
            ts_event,
        }
    }

    /// Returns the size of the **entire** record in bytes. The size of a `RecordHeader`
    /// is constant.
    pub const fn record_size(&self) -> usize {
        self.length as usize * Self::LENGTH_MULTIPLIER
    }

    /// Tries to convert the raw `rtype` into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `rtype` field does not
    /// contain a valid, known [`RType`](crate::enums::RType).
    pub fn rtype(&self) -> crate::error::Result<RType> {
        RType::try_from(self.rtype)
            .map_err(|_| Error::conversion::<RType>(format!("{:#02X}", self.rtype)))
    }
}

impl MboMsg {
    /// Tries to convert the raw `side` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`](crate::enums::Side).
    pub fn side(&self) -> crate::error::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#02X}", self.side as u8)))
    }

    /// Tries to convert the raw `action` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`](crate::enums::Action).
    pub fn action(&self) -> crate::error::Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#02X}", self.action as u8)))
    }
}

impl TradeMsg {
    /// Tries to convert the raw `side` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`](crate::enums::Side).
    pub fn side(&self) -> crate::error::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#02X}", self.side as u8)))
    }

    /// Tries to convert the raw `action` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`](crate::enums::Action).
    pub fn action(&self) -> crate::error::Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#02X}", self.action as u8)))
    }
}

impl Mbp1Msg {
    /// Tries to convert the raw `side` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`](crate::enums::Side).
    pub fn side(&self) -> crate::error::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#02X}", self.side as u8)))
    }

    /// Tries to convert the raw `action` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`](crate::enums::Action).
    pub fn action(&self) -> crate::error::Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#02X}", self.action as u8)))
    }
}

impl Mbp10Msg {
    /// Tries to convert the raw `side` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`](crate::enums::Side).
    pub fn side(&self) -> Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#02X}", self.side as u8)))
    }

    /// Tries to convert the raw `action` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`](crate::enums::Action).
    pub fn action(&self) -> Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#02X}", self.action as u8)))
    }
}

impl StatusMsg {
    /// Returns `group` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `group` contains invalid UTF-8.
    pub fn group(&self) -> Result<&str> {
        c_chars_to_str(&self.group)
    }
}

impl InstrumentDefMsg {
    /// Returns `currency` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `currency` contains invalid UTF-8.
    pub fn currency(&self) -> Result<&str> {
        c_chars_to_str(&self.currency)
    }

    /// Returns `settl_currency` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `settl_currency` contains invalid UTF-8.
    pub fn settl_currency(&self) -> Result<&str> {
        c_chars_to_str(&self.settl_currency)
    }

    /// Returns `secsubtype` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `secsubtype` contains invalid UTF-8.
    pub fn secsubtype(&self) -> Result<&str> {
        c_chars_to_str(&self.secsubtype)
    }

    /// Returns `raw_symbol` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `raw_symbol` contains invalid UTF-8.
    pub fn raw_symbol(&self) -> Result<&str> {
        c_chars_to_str(&self.raw_symbol)
    }

    /// Returns `exchange` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `exchange` contains invalid UTF-8.
    pub fn exchange(&self) -> Result<&str> {
        c_chars_to_str(&self.exchange)
    }

    /// Returns `asset` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `asset` contains invalid UTF-8.
    pub fn asset(&self) -> Result<&str> {
        c_chars_to_str(&self.asset)
    }

    /// Returns `cfi` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `cfi` contains invalid UTF-8.
    pub fn cfi(&self) -> Result<&str> {
        c_chars_to_str(&self.cfi)
    }

    /// Returns `security_type` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `security_type` contains invalid UTF-8.
    pub fn security_type(&self) -> Result<&str> {
        c_chars_to_str(&self.security_type)
    }

    /// Returns `unit_of_measure` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `unit_of_measure` contains invalid UTF-8.
    pub fn unit_of_measure(&self) -> Result<&str> {
        c_chars_to_str(&self.unit_of_measure)
    }

    /// Returns `underlying` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `underlying` contains invalid UTF-8.
    pub fn underlying(&self) -> Result<&str> {
        c_chars_to_str(&self.underlying)
    }

    /// Returns `strike_price_currency` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `strike_price_currency` contains invalid UTF-8.
    pub fn strike_price_currency(&self) -> Result<&str> {
        c_chars_to_str(&self.strike_price_currency)
    }

    /// Returns `group` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `group` contains invalid UTF-8.
    pub fn group(&self) -> Result<&str> {
        c_chars_to_str(&self.group)
    }

    /// Tries to convert the raw `instrument_class` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `instrument_class` field does not
    /// contain a valid [`InstrumentClass`](crate::enums::InstrumentClass).
    pub fn instrument_class(&self) -> Result<InstrumentClass> {
        InstrumentClass::try_from(self.instrument_class as u8).map_err(|_| {
            Error::conversion::<InstrumentClass>(format!("{:#02X}", self.instrument_class as u8))
        })
    }

    /// Tries to convert the raw `match_algorithm` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `match_algorithm` field does not
    /// contain a valid [`MatchAlgorithm`](crate::enums::MatchAlgorithm).
    pub fn match_algorithm(&self) -> Result<MatchAlgorithm> {
        MatchAlgorithm::try_from(self.match_algorithm as u8).map_err(|_| {
            Error::conversion::<MatchAlgorithm>(format!("{:#02X}", self.match_algorithm as u8))
        })
    }
}

impl StatMsg {
    /// Tries to convert the raw `stat_type` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `stat_type` field does not
    /// contain a valid [`StatType`](crate::enums::StatType).
    pub fn stat_type(&self) -> Result<StatType> {
        StatType::try_from(self.stat_type)
            .map_err(|_| Error::conversion::<StatType>(format!("{:02X}", self.stat_type)))
    }

    /// Tries to convert the raw `update_action` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `update_action` field does not
    /// contain a valid [`StatUpdateAction`](crate::enums::StatUpdateAction).
    pub fn update_action(&self) -> Result<StatUpdateAction> {
        StatUpdateAction::try_from(self.update_action).map_err(|_| {
            Error::conversion::<StatUpdateAction>(format!("{:02X}", self.update_action))
        })
    }
}

impl ErrorMsg {
    /// Creates a new `ErrorMsg`.
    ///
    /// # Errors
    /// This function returns an error if `msg` is too long.
    pub fn new(ts_event: u64, msg: &str) -> Self {
        let mut error = Self {
            hd: RecordHeader::new::<Self>(rtype::ERROR, 0, 0, ts_event),
            err: [0; 64],
        };
        // leave at least one null byte
        for (i, byte) in msg.as_bytes().iter().take(error.err.len() - 1).enumerate() {
            error.err[i] = *byte as c_char;
        }
        error
    }

    /// Returns `err` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `err` contains invalid UTF-8.
    pub fn err(&self) -> Result<&str> {
        c_chars_to_str(&self.err)
    }
}

impl SymbolMappingMsg {
    /// Returns `stype_in_symbol` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `stype_in_symbol` contains invalid UTF-8.
    pub fn stype_in_symbol(&self) -> Result<&str> {
        c_chars_to_str(&self.stype_in_symbol)
    }

    /// Returns `stype_out_symbol` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `stype_out_symbol` contains invalid UTF-8.
    pub fn stype_out_symbol(&self) -> Result<&str> {
        c_chars_to_str(&self.stype_out_symbol)
    }
}

impl SystemMsg {
    const HEARTBEAT: &str = "Heartbeat";

    /// Creates a new `SystemMsg`.
    ///
    /// # Errors
    /// This function returns an error if `msg` is too long.
    pub fn new(ts_event: u64, msg: &str) -> Result<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::SYSTEM, 0, 0, ts_event),
            msg: str_to_c_chars(msg)?,
        })
    }

    /// Creates a new heartbeat `SystemMsg`.
    pub fn heartbeat(ts_event: u64) -> Self {
        Self {
            hd: RecordHeader::new::<Self>(rtype::SYSTEM, 0, 0, ts_event),
            msg: str_to_c_chars(Self::HEARTBEAT).unwrap(),
        }
    }

    /// Checks whether the message is a heartbeat from the gateway.
    pub fn is_heartbeat(&self) -> bool {
        self.msg()
            .map(|msg| msg == Self::HEARTBEAT)
            .unwrap_or_default()
    }

    /// Returns `msg` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `msg` contains invalid UTF-8.
    pub fn msg(&self) -> Result<&str> {
        c_chars_to_str(&self.msg)
    }
}

/// Wrapper object for records that include the live gateway send timestamp (`ts_out`).
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
pub struct WithTsOut<T: HasRType> {
    /// The inner record.
    pub rec: T,
    /// The live gateway send timestamp expressed as number of nanoseconds since the UNIX epoch.
    pub ts_out: u64,
}

impl<T: HasRType> HasRType for WithTsOut<T> {
    fn has_rtype(rtype: u8) -> bool {
        T::has_rtype(rtype)
    }

    fn header(&self) -> &RecordHeader {
        self.rec.header()
    }

    fn header_mut(&mut self) -> &mut RecordHeader {
        self.rec.header_mut()
    }
}

impl<T> AsRef<[u8]> for WithTsOut<T>
where
    T: HasRType + AsRef<[u8]>,
{
    fn as_ref(&self) -> &[u8] {
        unsafe { as_u8_slice(self) }
    }
}

impl<T: HasRType> WithTsOut<T> {
    /// Creates a new record with `ts_out`. Updates the `length` property in
    /// [`RecordHeader`] to ensure the additional field is accounted for.
    pub fn new(rec: T, ts_out: u64) -> Self {
        let mut res = Self { rec, ts_out };
        res.header_mut().length = (mem::size_of_val(&res) / RecordHeader::LENGTH_MULTIPLIER) as u8;
        res
    }
}

/// Provides a _relatively safe_ method for converting a reference to
/// [`RecordHeader`] to a struct beginning with the header. Because it accepts a
/// reference, the lifetime of the returned reference is tied to the input. This
/// function checks `rtype` before casting to ensure `bytes` contains a `T`.
///
/// # Safety
/// `raw` must contain at least `std::mem::size_of::<T>()` bytes and a valid
/// [`RecordHeader`] instance.
pub unsafe fn transmute_record_bytes<T: HasRType>(bytes: &[u8]) -> Option<&T> {
    assert!(
        bytes.len() >= mem::size_of::<T>(),
        "Passing a slice smaller than `{}` to `transmute_record_bytes` is invalid",
        std::any::type_name::<T>()
    );
    let non_null = NonNull::new_unchecked(bytes.as_ptr() as *mut u8);
    if T::has_rtype(non_null.cast::<RecordHeader>().as_ref().rtype) {
        Some(non_null.cast::<T>().as_ref())
    } else {
        None
    }
}

/// Provides a _relatively safe_ method for converting a view on bytes into a
/// a [`RecordHeader`].
/// Because it accepts a reference, the lifetime of the returned reference is
/// tied to the input.
///
/// # Safety
/// `bytes` must contain a complete record (not only the header). This is so that
/// the header can be subsequently passed to transmute_record
pub unsafe fn transmute_header_bytes(bytes: &[u8]) -> Option<&RecordHeader> {
    assert!(
        bytes.len() >= mem::size_of::<RecordHeader>(),
        concat!(
            "Passing a slice smaller than `",
            stringify!(RecordHeader),
            "` to `transmute_header_bytes` is invalid"
        )
    );
    let non_null = NonNull::new_unchecked(bytes.as_ptr() as *mut u8);
    let header = non_null.cast::<RecordHeader>().as_ref();
    if header.record_size() > bytes.len() {
        None
    } else {
        Some(header)
    }
}

/// Provides a _relatively safe_ method for converting a reference to a
/// [`RecordHeader`] to a struct beginning with the header. Because it accepts a reference,
/// the lifetime of the returned reference is tied to the input.
///
/// # Safety
/// Although this function accepts a reference to a [`RecordHeader`], it's assumed this is
/// part of a larger `T` struct.
pub unsafe fn transmute_record<T: HasRType>(header: &RecordHeader) -> Option<&T> {
    if T::has_rtype(header.rtype) {
        // Safety: because it comes from a reference, `header` must not be null. It's ok
        // to cast to `mut` because it's never mutated.
        let non_null = NonNull::from(header);
        Some(non_null.cast::<T>().as_ref())
    } else {
        None
    }
}

/// Aliases `data` as a slice of raw bytes.
///
/// # Safety
/// `data` must be sized and plain old data (POD), i.e. no pointers.
pub(crate) unsafe fn as_u8_slice<T: Sized>(data: &T) -> &[u8] {
    slice::from_raw_parts(data as *const T as *const u8, mem::size_of::<T>())
}

/// Provides a _relatively safe_ method for converting a mut reference to a
/// [`RecordHeader`] to a struct beginning with the header. Because it accepts a reference,
/// the lifetime of the returned reference is tied to the input.
///
/// # Safety
/// Although this function accepts a reference to a [`RecordHeader`], it's assumed this is
/// part of a larger `T` struct.
pub unsafe fn transmute_record_mut<T: HasRType>(header: &mut RecordHeader) -> Option<&mut T> {
    if T::has_rtype(header.rtype) {
        // Safety: because it comes from a reference, `header` must not be null.
        let non_null = NonNull::from(header);
        Some(non_null.cast::<T>().as_mut())
    } else {
        None
    }
}

/// Tries to convert a str slice to fixed-length null-terminated C char array.
///
/// # Errors
/// This function returns an error if `s` contains more than N - 1 characters. The last
/// character is reserved for the null byte.
pub fn str_to_c_chars<const N: usize>(s: &str) -> Result<[c_char; N]> {
    if s.len() > (N - 1) {
        return Err(Error::encode(format!(
            "String cannot be longer than {}; received str of length {}",
            N - 1,
            s.len(),
        )));
    }
    let mut res = [0; N];
    for (i, byte) in s.as_bytes().iter().enumerate() {
        res[i] = *byte as c_char;
    }
    Ok(res)
}

/// Tries to convert a slice of `c_char`s to a UTF-8 `str`.
///
/// # Safety
/// This should always be safe.
///
/// # Preconditions
/// None.
///
/// # Errors
/// This function returns an error if `chars` contains invalid UTF-8.
pub fn c_chars_to_str<const N: usize>(chars: &[c_char; N]) -> Result<&str> {
    let cstr = unsafe { CStr::from_ptr(chars.as_ptr()) };
    cstr.to_str()
        .map_err(|e| Error::utf8(e, format!("converting c_char array: {chars:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    const OHLCV_MSG: OhlcvMsg = OhlcvMsg {
        hd: RecordHeader {
            length: 56,
            rtype: rtype::OHLCV_1S,
            publisher_id: 1,
            instrument_id: 5482,
            ts_event: 1609160400000000000,
        },
        open: 372025000000000,
        high: 372050000000000,
        low: 372025000000000,
        close: 372050000000000,
        volume: 57,
    };

    #[test]
    fn test_transmute_record_bytes() {
        unsafe {
            let ohlcv_bytes = std::slice::from_raw_parts(
                &OHLCV_MSG as *const OhlcvMsg as *const u8,
                mem::size_of::<OhlcvMsg>(),
            )
            .to_vec();
            let ohlcv = transmute_record_bytes::<OhlcvMsg>(ohlcv_bytes.as_slice()).unwrap();
            assert_eq!(*ohlcv, OHLCV_MSG);
        };
    }

    #[test]
    #[should_panic]
    fn test_transmute_record_bytes_small_buffer() {
        let source = OHLCV_MSG;
        unsafe {
            let slice = std::slice::from_raw_parts(
                &source as *const OhlcvMsg as *const u8,
                mem::size_of::<OhlcvMsg>() - 5,
            );
            transmute_record_bytes::<OhlcvMsg>(slice);
        };
    }

    #[test]
    fn test_transmute_record() {
        let source = Box::new(OHLCV_MSG);
        let ohlcv_ref: &OhlcvMsg = unsafe { transmute_record(&source.hd) }.unwrap();
        assert_eq!(*ohlcv_ref, OHLCV_MSG);
    }

    #[test]
    fn test_transmute_record_mut() {
        let mut source = Box::new(OHLCV_MSG);
        let ohlcv_ref: &OhlcvMsg = unsafe { transmute_record_mut(&mut source.hd) }.unwrap();
        assert_eq!(*ohlcv_ref, OHLCV_MSG);
    }

    #[test]
    fn test_sizes() {
        assert_eq!(mem::size_of::<RecordHeader>(), 16);
        assert_eq!(mem::size_of::<MboMsg>(), 56);
        assert_eq!(mem::size_of::<Mbp1Msg>(), 80);
        assert_eq!(mem::size_of::<Mbp10Msg>(), 368);
        assert_eq!(mem::size_of::<OhlcvMsg>(), 56);
        assert_eq!(mem::size_of::<StatusMsg>(), 48);
        assert_eq!(mem::size_of::<InstrumentDefMsg>(), 360);
        assert_eq!(mem::size_of::<StatMsg>(), 64);
        assert_eq!(mem::size_of::<ErrorMsg>(), 80);
        assert_eq!(mem::size_of::<SymbolMappingMsg>(), 80);
        assert_eq!(mem::size_of::<SystemMsg>(), 80);
    }
}
