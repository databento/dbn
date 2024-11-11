//! Market data types for encoding different Databento [`Schema`](crate::enums::Schema)s
//! and conversion functions.

pub(crate) mod conv;
mod impl_default;
mod methods;

use std::{ffi::CStr, mem, os::raw::c_char, ptr::NonNull, slice};

// Dummy derive macro to get around `cfg_attr` incompatibility of several
// of pyo3's attribute macros. See https://github.com/PyO3/pyo3/issues/780
#[cfg(not(feature = "python"))]
use dbn_macros::MockPyo3;

use crate::{
    enums::rtype,
    macros::{dbn_record, CsvSerialize, JsonSerialize, RecordDebug},
    Action, Error, FlagSet, InstrumentClass, MatchAlgorithm, Publisher, RType, Result,
    SecurityUpdateAction, Side, StatType, StatUpdateAction, UserDefinedInstrument, SYMBOL_CSTR_LEN,
};
pub(crate) use conv::as_u8_slice;
#[cfg(feature = "serde")]
pub(crate) use conv::cstr_serde;
pub use conv::{
    c_chars_to_str, str_to_c_chars, transmute_header_bytes, transmute_record,
    transmute_record_bytes, transmute_record_mut, ts_to_dt,
};

/// Common data for all Databento records. Always found at the beginning of a record
/// struct.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
pub struct RecordHeader {
    /// The length of the record in 32-bit words.
    #[dbn(skip)]
    pub(crate) length: u8,
    /// The record type; with `0xe0..0x0F` specifying MBP levels size. Record types
    /// implement the trait [`HasRType`], and the [`has_rtype`][HasRType::has_rtype]
    /// function can be used to check if that type can be used to decode a message with
    /// a given rtype. The set of possible values is defined in [`rtype`].
    pub rtype: u8,
    /// The publisher ID assigned by Databento, which denotes the dataset and venue.
    #[pyo3(set)]
    pub publisher_id: u16,
    /// The numeric ID assigned to the instrument.
    #[pyo3(set)]
    pub instrument_id: u32,
    /// The matching-engine-received timestamp expressed as number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(encode_order(0), unix_nanos)]
    #[pyo3(set)]
    pub ts_event: u64,
}

/// A market-by-order (MBO) tick message. The record of the
/// [`Mbo`](crate::enums::Schema::Mbo) schema.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn", name = "MBOMsg"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::MBO)]
pub struct MboMsg {
    /// The common header.
    #[pyo3(get)]
    pub hd: RecordHeader,
    /// The order ID assigned at the venue.
    #[pyo3(get, set)]
    pub order_id: u64,
    /// The order price expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(encode_order(4), fixed_price)]
    #[pyo3(get, set)]
    pub price: i64,
    /// The order quantity.
    #[dbn(encode_order(5))]
    #[pyo3(get, set)]
    pub size: u32,
    /// A bit field indicating event end, message characteristics, and data quality. See
    /// [`enums::flags`](crate::enums::flags) for possible values.
    #[pyo3(get, set)]
    pub flags: FlagSet,
    /// A channel ID within the venue.
    #[dbn(encode_order(6))]
    #[pyo3(get, set)]
    pub channel_id: u8,
    /// The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R**,
    /// **T**rade, **F**ill, or **N**one.
    #[dbn(c_char, encode_order(2))]
    pub action: c_char,
    /// The side that initiates the event. Can be **A**sk for a sell order (or sell
    /// aggressor in a trade), **B**id for a buy order (or buy aggressor in a trade), or
    /// **N**one where no side is specified by the original source.
    #[dbn(c_char, encode_order(3))]
    pub side: c_char,
    /// The capture-server-received timestamp expressed as number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The delta of `ts_recv - ts_exchange_send`, max 2 seconds.
    #[pyo3(get, set)]
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    #[pyo3(get, set)]
    pub sequence: u32,
}

/// A level.
#[repr(C)]
#[derive(Clone, JsonSerialize, RecordDebug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(test, derive(type_layout::TypeLayout))]
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

/// A price level consolidated from multiple venues.
#[repr(C)]
#[derive(Clone, JsonSerialize, RecordDebug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(test, derive(type_layout::TypeLayout))]
pub struct ConsolidatedBidAskPair {
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
    /// The bid publisher ID assigned by Databento, which denotes the dataset and venue.
    #[dbn(fmt_method)]
    pub bid_pb: u16,
    // Reserved for later usage.
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved1: [c_char; 2],
    /// The ask publisher ID assigned by Databento, which denotes the dataset and venue.
    #[dbn(fmt_method)]
    pub ask_pb: u16,
    // Reserved for later usage.
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved2: [c_char; 2],
}

/// Market by price implementation with a book depth of 0. Equivalent to
/// MBP-0. The record of the [`Trades`](crate::enums::Schema::Trades) schema.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::MBP_0)]
pub struct TradeMsg {
    /// The common header.
    #[pyo3(get)]
    pub hd: RecordHeader,
    /// The order price expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub price: i64,
    /// The order quantity.
    #[pyo3(get, set)]
    pub size: u32,
    /// The event action. Always **T**rade in the trades schema.
    #[dbn(c_char, encode_order(2))]
    pub action: c_char,
    /// The side that initiates the trade. Can be **A**sk for a sell aggressor in a
    /// trade, **B**id for a buy aggressor in a trade, or **N**one where no side is
    /// specified by the original source.
    #[dbn(c_char, encode_order(3))]
    pub side: c_char,
    /// A bit field indicating event end, message characteristics, and data quality. See
    /// [`enums::flags`](crate::enums::flags) for possible values.
    #[pyo3(get, set)]
    pub flags: FlagSet,
    /// The depth of actual book change.
    #[dbn(encode_order(4))]
    #[pyo3(get, set)]
    pub depth: u8,
    /// The capture-server-received timestamp expressed as number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The delta of `ts_recv - ts_exchange_send`, max 2 seconds.
    #[pyo3(get, set)]
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    #[pyo3(get, set)]
    pub sequence: u32,
}

/// Market by price implementation with a known book depth of 1. The record of the
/// [`Mbp1`](crate::enums::Schema::Mbp1) schema.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn", name = "MBP1Msg"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::MBP_1)]
pub struct Mbp1Msg {
    /// The common header.
    #[pyo3(get)]
    pub hd: RecordHeader,
    /// The order price expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub price: i64,
    /// The order quantity.
    #[pyo3(get, set)]
    pub size: u32,
    /// The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R**, or
    /// **T**rade.
    #[dbn(c_char, encode_order(2))]
    pub action: c_char,
    /// The side that initiates the event. Can be **A**sk for a sell order (or sell
    /// aggressor in a trade), **B**id for a buy order (or buy aggressor in a trade), or
    /// **N**one where no side is specified by the original source.
    #[dbn(c_char, encode_order(3))]
    pub side: c_char,
    /// A bit field indicating event end, message characteristics, and data quality. See
    /// [`enums::flags`](crate::enums::flags) for possible values.
    #[pyo3(get, set)]
    pub flags: FlagSet,
    /// The depth of actual book change.
    #[dbn(encode_order(4))]
    #[pyo3(get, set)]
    pub depth: u8,
    /// The capture-server-received timestamp expressed as number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The delta of `ts_recv - ts_exchange_send`, max 2 seconds.
    #[pyo3(get, set)]
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    #[pyo3(get, set)]
    pub sequence: u32,
    /// The top of the order book.
    #[pyo3(get, set)]
    pub levels: [BidAskPair; 1],
}

/// Market by price implementation with a known book depth of 10. The record of the
/// [`Mbp10`](crate::enums::Schema::Mbp10) schema.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn", name = "MBP10Msg"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::MBP_10)]
pub struct Mbp10Msg {
    /// The common header.
    #[pyo3(get)]
    pub hd: RecordHeader,
    /// The order price expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub price: i64,
    /// The order quantity.
    #[pyo3(get, set)]
    pub size: u32,
    /// The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R**, or
    /// **T**rade.
    #[dbn(c_char, encode_order(2))]
    pub action: c_char,
    /// The side that initiates the event. Can be **A**sk for a sell order (or sell
    /// aggressor in a trade), **B**id for a buy order (or buy aggressor in a trade), or
    /// **N**one where no side is specified by the original source.
    #[dbn(c_char, encode_order(3))]
    pub side: c_char,
    /// A bit field indicating event end, message characteristics, and data quality. See
    /// [`enums::flags`](crate::enums::flags) for possible values.
    #[pyo3(get, set)]
    pub flags: FlagSet,
    /// The depth of actual book change.
    #[dbn(encode_order(4))]
    #[pyo3(get, set)]
    pub depth: u8,
    /// The capture-server-received timestamp expressed as number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The delta of `ts_recv - ts_exchange_send`, max 2 seconds.
    #[pyo3(get, set)]
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    #[pyo3(get, set)]
    pub sequence: u32,
    /// The top 10 levels of the order book.
    #[pyo3(get, set)]
    pub levels: [BidAskPair; 10],
}

/// Subsampled market by price with a known book depth of 1. The record of the
/// [`Bbo1S`](crate::Schema::Bbo1S) and [`Bbo1M`](crate::Schema::Bbo1M) schemas.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn", name = "BBOMsg"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::BBO_1S, rtype::BBO_1M)]
pub struct BboMsg {
    /// The common header.
    #[pyo3(get)]
    pub hd: RecordHeader,
    /// The price of the last trade expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub price: i64,
    /// The quantity of the last trade.
    #[pyo3(get, set)]
    pub size: u32,
    // Reserved for later usage.
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved1: u8,
    /// The side that initiated the last trade. Can be **A**sk for a sell order (or sell
    /// aggressor in a trade), **B**id for a buy order (or buy aggressor in a trade), or
    /// **N**one where no side is specified by the original source.
    #[dbn(c_char, encode_order(2))]
    pub side: c_char,
    /// A bit field indicating event end, message characteristics, and data quality. See
    /// [`enums::flags`](crate::enums::flags) for possible values.
    #[pyo3(get, set)]
    pub flags: FlagSet,
    // Reserved for later usage.
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved2: u8,
    /// The interval timestamp expressed as number of nanoseconds since the UNIX epoch.
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    // Reserved for later usage.
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved3: [u8; 4],
    /// The sequence number assigned at the venue of the last update.
    #[pyo3(get, set)]
    pub sequence: u32,
    /// The top of the order book.
    #[pyo3(get, set)]
    pub levels: [BidAskPair; 1],
}

/// Consolidated market by price implementation with a known book depth of 1. The record of the
/// [`Cmbp1`](crate::Schema::Cmbp1) schema.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn", name = "CMBP1Msg"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::CMBP1, rtype::TCBBO)]
pub struct Cmbp1Msg {
    /// The common header.
    #[pyo3(get)]
    pub hd: RecordHeader,
    /// The order price expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub price: i64,
    /// The order quantity.
    #[pyo3(get, set)]
    pub size: u32,
    /// The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R**, or
    /// **T**rade.
    #[dbn(c_char, encode_order(2))]
    pub action: c_char,
    /// The side that initiates the event. Can be **A**sk for a sell order (or sell
    /// aggressor in a trade), **B**id for a buy order (or buy aggressor in a trade), or
    /// **N**one where no side is specified by the original source.
    #[dbn(c_char, encode_order(3))]
    pub side: c_char,
    /// A bit field indicating event end, message characteristics, and data quality. See
    /// [`enums::flags`](crate::enums::flags) for possible values.
    #[pyo3(get, set)]
    pub flags: FlagSet,
    // Reserved for future usage.
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved1: [c_char; 1],
    /// The capture-server-received timestamp expressed as number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The delta of `ts_recv - ts_exchange_send`, max 2 seconds.
    #[pyo3(get, set)]
    pub ts_in_delta: i32,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved2: [c_char; 4],
    /// The top of the order book.
    #[pyo3(get, set)]
    pub levels: [ConsolidatedBidAskPair; 1],
}

/// Subsampled market by price with a known book depth of 1. The record of the
/// [`Bbo1S`](crate::Schema::Bbo1S) and [`Bbo1M`](crate::Schema::Bbo1M) schemas.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn", name = "CBBOMsg"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::CBBO_1S, rtype::CBBO_1M)]
pub struct CbboMsg {
    /// The common header.
    #[pyo3(get)]
    pub hd: RecordHeader,
    /// The price of the last trade expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub price: i64,
    /// The quantity of the last trade.
    #[pyo3(get, set)]
    pub size: u32,
    // Reserved for later usage.
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved1: u8,
    /// The side that initiated the last trade. Can be **A**sk for a sell order (or sell
    /// aggressor in a trade), **B**id for a buy order (or buy aggressor in a trade), or
    /// **N**one where no side is specified by the original source.
    #[dbn(c_char, encode_order(2))]
    pub side: c_char,
    /// A bit field indicating event end, message characteristics, and data quality. See
    /// [`enums::flags`](crate::enums::flags) for possible values.
    #[pyo3(get, set)]
    pub flags: FlagSet,
    // Reserved for later usage.
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved2: u8,
    /// The interval timestamp expressed as number of nanoseconds since the UNIX epoch.
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    // Reserved for later usage.
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved3: [u8; 8],
    /// The top of the order book.
    #[pyo3(get, set)]
    pub levels: [ConsolidatedBidAskPair; 1],
}

/// The record of the [`Tbbo`](crate::enums::Schema::Tbbo) schema.
pub type TbboMsg = Mbp1Msg;
/// The record of the [`Bbo1S`](crate::enums::Schema::Bbo1S) schema.
pub type Bbo1SMsg = BboMsg;
/// The record of the [`Bbo1M`](crate::enums::Schema::Bbo1M) schema.
pub type Bbo1MMsg = BboMsg;

/// The record of the [`Tcbbo`](crate::enums::Schema::Tcbbo) schema.
pub type TcbboMsg = Cmbp1Msg;
/// The record of the [`Cbbo1S`](crate::enums::Schema::Cbbo1S) schema.
pub type Cbbo1SMsg = CbboMsg;
/// The record of the [`Cbbo1M`](crate::enums::Schema::Cbbo1M) schema.
pub type Cbbo1MMsg = CbboMsg;

/// Open, high, low, close, and volume. The record of the following schemas:
/// - [`Ohlcv1S`](crate::enums::Schema::Ohlcv1S)
/// - [`Ohlcv1M`](crate::enums::Schema::Ohlcv1M)
/// - [`Ohlcv1H`](crate::enums::Schema::Ohlcv1H)
/// - [`Ohlcv1D`](crate::enums::Schema::Ohlcv1D)
/// - [`OhlcvEod`](crate::enums::Schema::OhlcvEod)
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, dict, module = "databento_dbn", name = "OHLCVMsg"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
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
    #[pyo3(set)]
    pub open: i64,
    /// The high price for the bar.
    #[dbn(fixed_price)]
    #[pyo3(set)]
    pub high: i64,
    /// The low price for the bar.
    #[dbn(fixed_price)]
    #[pyo3(set)]
    pub low: i64,
    /// The close price for the bar.
    #[dbn(fixed_price)]
    #[pyo3(set)]
    pub close: i64,
    /// The total volume traded during the aggregation period.
    #[pyo3(set)]
    pub volume: u64,
}

/// A trading status update message. The record of the
/// [`Status`](crate::enums::Schema::Status) schema.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::STATUS)]
pub struct StatusMsg {
    /// The common header.
    #[pyo3(get)]
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The type of status change.
    #[dbn(fmt_method)]
    #[pyo3(get, set)]
    pub action: u16,
    /// Additional details about the cause of the status change.
    #[dbn(fmt_method)]
    #[pyo3(get, set)]
    pub reason: u16,
    /// Further information about the status change and its effect on trading.
    #[dbn(fmt_method)]
    #[pyo3(get, set)]
    pub trading_event: u16,
    /// The state of trading in the instrument.
    #[dbn(c_char)]
    #[pyo3(get, set)]
    pub is_trading: c_char,
    /// The state of quoting in the instrument.
    #[dbn(c_char)]
    #[pyo3(get, set)]
    pub is_quoting: c_char,
    /// The state of short sell restrictions for the instrument.
    #[dbn(c_char)]
    #[pyo3(get, set)]
    pub is_short_sell_restricted: c_char,
    // Filler for alignment.
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved: [u8; 7],
}

/// Definition of an instrument. The record of the
/// [`Definition`](crate::enums::Schema::Definition) schema.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::INSTRUMENT_DEF)]
pub struct InstrumentDefMsg {
    /// The common header.
    #[pyo3(get, set)]
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as number of nanoseconds since the
    /// UNIX epoch.
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The minimum constant tick for the instrument in units of 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub min_price_increment: i64,
    /// The multiplier to convert the venue’s display price to the conventional price,
    /// in units of 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
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
    /// The contract size for each instrument, in combination with `unit_of_measure`, in units
    /// of 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
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
    /// The strike price of the option. Converted to units of 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001.
    #[dbn(fixed_price, encode_order(46))]
    #[pyo3(get, set)]
    pub strike_price: i64,
    /// A bitmap of instrument eligibility attributes.
    #[dbn(fmt_binary)]
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
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "conv::cstr_serde"))]
    pub currency: [c_char; 4],
    /// The currency used for settlement, if different from `currency`.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "conv::cstr_serde"))]
    pub settl_currency: [c_char; 4],
    /// The strategy type of the spread.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "conv::cstr_serde"))]
    pub secsubtype: [c_char; 6],
    /// The instrument raw symbol assigned by the publisher.
    #[dbn(encode_order(2), fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "conv::cstr_serde"))]
    pub raw_symbol: [c_char; SYMBOL_CSTR_LEN],
    /// The security group code of the instrument.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "conv::cstr_serde"))]
    pub group: [c_char; 21],
    /// The exchange used to identify the instrument.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "conv::cstr_serde"))]
    pub exchange: [c_char; 5],
    /// The underlying asset code (product code) of the instrument.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "conv::cstr_serde"))]
    pub asset: [c_char; 7],
    /// The ISO standard instrument categorization code.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "conv::cstr_serde"))]
    pub cfi: [c_char; 7],
    /// The type of the instrument, e.g. FUT for future or future spread.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "conv::cstr_serde"))]
    pub security_type: [c_char; 7],
    /// The unit of measure for the instrument’s original contract size, e.g. USD or LBS.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "conv::cstr_serde"))]
    pub unit_of_measure: [c_char; 31],
    /// The symbol of the first underlying instrument.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "conv::cstr_serde"))]
    pub underlying: [c_char; 21],
    /// The currency of [`strike_price`](Self::strike_price).
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "conv::cstr_serde"))]
    pub strike_price_currency: [c_char; 4],
    /// The classification of the instrument.
    #[dbn(c_char, encode_order(4))]
    #[pyo3(set)]
    pub instrument_class: c_char,
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
    #[dbn(c_char, encode_order(3))]
    #[pyo3(set)]
    pub security_update_action: c_char,
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
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved: [u8; 10],
}

/// An auction imbalance message.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::IMBALANCE)]
pub struct ImbalanceMsg {
    /// The common header.
    #[pyo3(get)]
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The price at which the imbalance shares are calculated, where every 1 unit corresponds to
    /// 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub ref_price: i64,
    /// Reserved for future use.
    #[pyo3(get, set)]
    pub auction_time: u64,
    /// The hypothetical auction-clearing price for both cross and continuous orders.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub cont_book_clr_price: i64,
    /// The hypothetical auction-clearing price for cross orders only.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub auct_interest_clr_price: i64,
    /// Reserved for future use.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub ssr_filling_price: i64,
    /// Reserved for future use.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub ind_match_price: i64,
    /// Reserved for future use.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub upper_collar: i64,
    /// Reserved for future use.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub lower_collar: i64,
    /// The quantity of shares that are eligible to be matched at `ref_price`.
    #[pyo3(get, set)]
    pub paired_qty: u32,
    /// The quantity of shares that are not paired at `ref_price`.
    #[pyo3(get, set)]
    pub total_imbalance_qty: u32,
    /// Reserved for future use.
    #[pyo3(get, set)]
    pub market_imbalance_qty: u32,
    /// Reserved for future use.
    #[pyo3(get, set)]
    pub unpaired_qty: u32,
    /// Venue-specific character code indicating the auction type.
    #[dbn(c_char)]
    pub auction_type: c_char,
    /// The market side of the `total_imbalance_qty`. Can be **A**sk, **B**id, or **N**one.
    #[dbn(c_char)]
    pub side: c_char,
    /// Reserved for future use.
    #[pyo3(get, set)]
    pub auction_status: u8,
    /// Reserved for future use.
    #[pyo3(get, set)]
    pub freeze_status: u8,
    /// Reserved for future use.
    #[pyo3(get, set)]
    pub num_extensions: u8,
    /// Reserved for future use.
    #[dbn(c_char)]
    pub unpaired_side: c_char,
    /// Venue-specific character code. For Nasdaq, contains the raw Price Variation Indicator.
    #[dbn(c_char)]
    pub significant_imbalance: c_char,
    // Filler for alignment.
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved: [u8; 1],
}

/// A statistics message. A catchall for various data disseminated by publishers.
/// The [`stat_type`](Self::stat_type) indicates the statistic contained in the message.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::STATISTICS)]
pub struct StatMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(set)]
    pub ts_recv: u64,
    /// The reference timestamp of the statistic value expressed as the number of
    /// nanoseconds since the UNIX epoch. Will be [`crate::UNDEF_TIMESTAMP`] when
    /// unused.
    #[dbn(unix_nanos)]
    #[pyo3(set)]
    pub ts_ref: u64,
    /// The value for price statistics expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001. Will be
    /// [`crate::UNDEF_PRICE`] when unused.
    #[dbn(fixed_price)]
    #[pyo3(set)]
    pub price: i64,
    /// The value for non-price statistics. Will be [`crate::UNDEF_STAT_QUANTITY`] when
    /// unused.
    #[pyo3(set)]
    pub quantity: i32,
    /// The message sequence number assigned at the venue.
    #[pyo3(set)]
    pub sequence: u32,
    /// The delta of `ts_recv - ts_exchange_send`, max 2 seconds.
    #[pyo3(set)]
    pub ts_in_delta: i32,
    /// The type of statistic value contained in the message. Refer to the
    /// [`StatType`](crate::enums::StatType) for variants.
    #[dbn(fmt_method)]
    #[pyo3(set)]
    pub stat_type: u16,
    /// A channel ID within the venue.
    #[pyo3(set)]
    pub channel_id: u16,
    /// Indicates if the statistic is newly added (1) or deleted (2). (Deleted is only used with
    /// some stat types)
    #[dbn(fmt_method)]
    #[pyo3(set)]
    pub update_action: u8,
    /// Additional flags associate with certain stat types.
    #[dbn(fmt_binary)]
    #[pyo3(set)]
    pub stat_flags: u8,
    // Filler for alignment
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved: [u8; 6],
}

/// An error message from the Databento Live Subscription Gateway (LSG).
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::ERROR)]
pub struct ErrorMsg {
    /// The common header.
    #[pyo3(get)]
    pub hd: RecordHeader,
    /// The error message.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "conv::cstr_serde"))]
    pub err: [c_char; 302],
    /// The error code. Currently unused.
    #[pyo3(get, set)]
    pub code: u8,
    /// Sometimes multiple errors are sent together. This field will be non-zero for the
    /// last error.
    #[pyo3(get, set)]
    pub is_last: u8,
}

/// A symbol mapping message which maps a symbol of one [`SType`](crate::enums::SType)
/// to another.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::SYMBOL_MAPPING)]
pub struct SymbolMappingMsg {
    /// The common header.
    #[pyo3(get, set)]
    pub hd: RecordHeader,
    // TODO(carter): special serialization to string?
    /// The input symbology type of `stype_in_symbol`.
    #[dbn(fmt_method)]
    #[pyo3(get, set)]
    pub stype_in: u8,
    /// The input symbol.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "conv::cstr_serde"))]
    pub stype_in_symbol: [c_char; SYMBOL_CSTR_LEN],
    /// The output symbology type of `stype_out_symbol`.
    #[dbn(fmt_method)]
    #[pyo3(get, set)]
    pub stype_out: u8,
    /// The output symbol.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "conv::cstr_serde"))]
    pub stype_out_symbol: [c_char; SYMBOL_CSTR_LEN],
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
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::SYSTEM)]
pub struct SystemMsg {
    /// The common header.
    #[pyo3(get, set)]
    pub hd: RecordHeader,
    /// The message from the Databento Live Subscription Gateway (LSG).
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "conv::cstr_serde"))]
    pub msg: [c_char; 303],
    /// Type of system message, currently unused.
    #[pyo3(get, set)]
    pub code: u8,
}

/// Used for polymorphism around types all beginning with a [`RecordHeader`] where
/// `rtype` is the discriminant used for indicating the type of record.
pub trait Record {
    /// Returns a reference to the `RecordHeader` that comes at the beginning of all
    /// record types.
    fn header(&self) -> &RecordHeader;

    /// Returns the size of the record in bytes.
    fn record_size(&self) -> usize {
        self.header().record_size()
    }

    /// Tries to convert the raw record type into an enum which is useful for exhaustive
    /// pattern matching.
    ///
    /// # Errors
    /// This function returns an error if the `rtype` field does not
    /// contain a valid, known [`RType`].
    fn rtype(&self) -> crate::Result<RType> {
        self.header().rtype()
    }

    /// Tries to convert the raw `publisher_id` into an enum which is useful for
    /// exhaustive pattern matching.
    ///
    /// # Errors
    /// This function returns an error if the `publisher_id` does not correspond with
    /// any known [`Publisher`].
    fn publisher(&self) -> crate::Result<Publisher> {
        self.header().publisher()
    }

    /// Returns the raw primary timestamp for the record.
    ///
    /// This timestamp should be used for sorting records as well as indexing into any
    /// symbology data structure.
    fn raw_index_ts(&self) -> u64 {
        self.header().ts_event
    }

    /// Returns the primary timestamp for the record. Returns `None` if the primary
    /// timestamp contains the sentinel value for a null timestamp.
    ///
    /// This timestamp should be used for sorting records as well as indexing into any
    /// symbology data structure.
    fn index_ts(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.raw_index_ts())
    }

    /// Returns the primary date for the record; the date component of the primary
    /// timestamp (`index_ts()`). Returns `None` if the primary timestamp contains the
    /// sentinel value for a null timestamp.
    fn index_date(&self) -> Option<time::Date> {
        self.index_ts().map(|dt| dt.date())
    }
}

/// Used for polymorphism around mutable types beginning with a [`RecordHeader`].
pub trait RecordMut {
    /// Returns a mutable reference to the `RecordHeader` that comes at the beginning of
    /// all record types.
    fn header_mut(&mut self) -> &mut RecordHeader;
}

/// An extension of the [`Record`] trait for types with a static [`RType`]. Used for
/// determining if a rtype matches a type.
pub trait HasRType: Record + RecordMut {
    /// Returns `true` if `rtype` matches the value associated with the implementing type.
    fn has_rtype(rtype: u8) -> bool;
}

/// Wrapper object for records that include the live gateway send timestamp (`ts_out`).
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WithTsOut<T: HasRType> {
    /// The inner record.
    pub rec: T,
    /// The live gateway send timestamp expressed as number of nanoseconds since the UNIX epoch.
    pub ts_out: u64,
}

#[cfg(test)]
mod tests {
    use mem::offset_of;
    use rstest::rstest;
    use type_layout::{Field, TypeLayout};

    use crate::Schema;
    use crate::UNDEF_TIMESTAMP;

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

    #[rstest]
    #[case::header(RecordHeader::default::<MboMsg>(rtype::MBO), 16)]
    #[case::mbo(MboMsg::default(), 56)]
    #[case::ba_pair(BidAskPair::default(), 32)]
    #[case::cba_pair(ConsolidatedBidAskPair::default(), mem::size_of::<BidAskPair>())]
    #[case::trade(TradeMsg::default(), 48)]
    #[case::mbp1(Mbp1Msg::default(), mem::size_of::<TradeMsg>() + mem::size_of::<BidAskPair>())]
    #[case::mbp10(Mbp10Msg::default(), mem::size_of::<TradeMsg>() + mem::size_of::<BidAskPair>() * 10)]
    #[case::bbo(BboMsg::default_for_schema(Schema::Bbo1S), mem::size_of::<Mbp1Msg>())]
    #[case::cmbp1(Cmbp1Msg::default_for_schema(Schema::Cmbp1), mem::size_of::<Mbp1Msg>())]
    #[case::cbbo(CbboMsg::default_for_schema(Schema::Cbbo1S), mem::size_of::<Mbp1Msg>())]
    #[case::ohlcv(OhlcvMsg::default_for_schema(Schema::Ohlcv1S), 56)]
    #[case::status(StatusMsg::default(), 40)]
    #[case::definition(InstrumentDefMsg::default(), 400)]
    #[case::imbalance(ImbalanceMsg::default(), 112)]
    #[case::stat(StatMsg::default(), 64)]
    #[case::error(ErrorMsg::default(), 320)]
    #[case::symbol_mapping(SymbolMappingMsg::default(), 176)]
    #[case::system(SystemMsg::default(), 320)]
    #[case::with_ts_out(WithTsOut::new(SystemMsg::default(), 0), mem::size_of::<SystemMsg>() + 8)]
    fn test_sizes<R: Sized>(#[case] _rec: R, #[case] exp: usize) {
        assert_eq!(mem::size_of::<R>(), exp);
        assert!(mem::size_of::<R>() <= crate::MAX_RECORD_LEN);
    }

    #[rstest]
    #[case::header(RecordHeader::default::<MboMsg>(rtype::MBO))]
    #[case::mbo(MboMsg::default())]
    #[case::ba_pair(BidAskPair::default())]
    #[case::cba_pair(ConsolidatedBidAskPair::default())]
    #[case::trade(TradeMsg::default())]
    #[case::mbp1(Mbp1Msg::default())]
    #[case::mbp10(Mbp10Msg::default())]
    #[case::bbo(BboMsg::default_for_schema(crate::Schema::Bbo1S))]
    #[case::cmbp1(Cmbp1Msg::default_for_schema(crate::Schema::Cmbp1))]
    #[case::cbbo(CbboMsg::default_for_schema(crate::Schema::Cbbo1S))]
    #[case::ohlcv(OhlcvMsg::default_for_schema(Schema::Ohlcv1S))]
    #[case::status(StatusMsg::default())]
    #[case::definition(InstrumentDefMsg::default())]
    #[case::imbalance(ImbalanceMsg::default())]
    #[case::stat(StatMsg::default())]
    #[case::error(ErrorMsg::default())]
    #[case::symbol_mapping(SymbolMappingMsg::default())]
    #[case::system(SystemMsg::default())]
    fn test_alignment_and_no_padding<R: TypeLayout>(#[case] _rec: R) {
        let layout = R::type_layout();
        assert_eq!(layout.alignment, 8, "Unexpected alignment: {layout}");
        for field in layout.fields.iter() {
            assert!(
                matches!(field, Field::Field { .. }),
                "Detected padding: {layout}"
            );
        }
    }

    #[test]
    fn test_bbo_alignment_matches_mbp1() {
        assert_eq!(offset_of!(BboMsg, hd), offset_of!(Mbp1Msg, hd));
        assert_eq!(offset_of!(BboMsg, price), offset_of!(Mbp1Msg, price));
        assert_eq!(offset_of!(BboMsg, size), offset_of!(Mbp1Msg, size));
        assert_eq!(offset_of!(BboMsg, side), offset_of!(Mbp1Msg, side));
        assert_eq!(offset_of!(BboMsg, flags), offset_of!(Mbp1Msg, flags));
        assert_eq!(offset_of!(BboMsg, ts_recv), offset_of!(Mbp1Msg, ts_recv));
        assert_eq!(offset_of!(BboMsg, sequence), offset_of!(Mbp1Msg, sequence));
        assert_eq!(offset_of!(BboMsg, levels), offset_of!(Mbp1Msg, levels));
    }

    #[test]
    fn test_mbo_index_ts() {
        let rec = MboMsg {
            ts_recv: 1,
            ..Default::default()
        };
        assert_eq!(rec.raw_index_ts(), 1);
    }

    #[test]
    fn test_def_index_ts() {
        let rec = InstrumentDefMsg {
            ts_recv: 1,
            ..Default::default()
        };
        assert_eq!(rec.raw_index_ts(), 1);
    }

    #[test]
    fn test_db_ts_always_valid_time_offsetdatetime() {
        assert!(time::OffsetDateTime::from_unix_timestamp_nanos(0).is_ok());
        assert!(time::OffsetDateTime::from_unix_timestamp_nanos((u64::MAX - 1) as i128).is_ok());
        assert!(time::OffsetDateTime::from_unix_timestamp_nanos(UNDEF_TIMESTAMP as i128).is_ok());
    }

    #[test]
    fn test_record_object_safe() {
        let _record: Box<dyn Record> = Box::new(ErrorMsg::new(1, "Boxed record", true));
    }
}
