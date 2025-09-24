//! Market data types for encoding different Databento [`Schema`](crate::enums::Schema)s
//! in the most recent DBN version, as well as conversion functions.

pub(crate) mod conv;
mod impl_default;
mod layout_tests;
mod methods;
mod record_methods_tests;
mod traits;
mod with_ts_out_methods;

use std::{ffi::CStr, mem, os::raw::c_char, ptr::NonNull, slice};

// Dummy derive macro to get around `cfg_attr` incompatibility of several
// of pyo3's attribute macros. See https://github.com/PyO3/pyo3/issues/780
#[cfg(not(feature = "python"))]
use dbn_macros::MockPyo3;

use crate::{
    enums::rtype,
    macros::{dbn_record, CsvSerialize, JsonSerialize, RecordDebug},
    Action, Error, FlagSet, InstrumentClass, MatchAlgorithm, Publisher, RType, Result,
    SecurityUpdateAction, Side, StatUpdateAction, UserDefinedInstrument, ASSET_CSTR_LEN,
    SYMBOL_CSTR_LEN,
};
pub(crate) use conv::as_u8_slice;
#[cfg(feature = "serde")]
pub(crate) use conv::cstr_serde;
pub use conv::{
    c_chars_to_str, str_to_c_chars, transmute_header_bytes, transmute_record,
    transmute_record_bytes, transmute_record_mut, ts_to_dt,
};
pub use traits::{HasRType, Record, RecordMut};

/// Common data for all Databento records. Always found at the beginning of a record
/// struct.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "python", derive(crate::macros::PyFieldDesc))]
#[cfg_attr(test, derive(type_layout::TypeLayout))]
pub struct RecordHeader {
    /// The length of the record in 32-bit words.
    #[dbn(skip)]
    pub(crate) length: u8,
    /// The record type; with `0x00..0x0F` specifying MBP levels size. Record types
    /// implement the trait [`HasRType`], and the [`has_rtype`][HasRType::has_rtype]
    /// function can be used to check if that type can be used to decode a message with
    /// a given rtype. The set of possible values is defined in [`rtype`].
    pub rtype: u8,
    /// The publisher ID assigned by Databento, which denotes the dataset and venue.
    ///
    /// See [Publishers](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#publishers-datasets-and-venues).
    pub publisher_id: u16,
    /// The numeric instrument ID. See [Instrument identifiers](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers).
    pub instrument_id: u32,
    /// The matching-engine-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_event](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-event).
    #[dbn(encode_order(0), unix_nanos)]
    pub ts_event: u64,
}

/// A market-by-order (MBO) tick message. The record of the [`Mbo`](crate::Schema::Mbo)
/// schema.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, eq, module = "databento_dbn", name = "MBOMsg"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::MBO)]
pub struct MboMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The order ID assigned at the venue.
    #[pyo3(get, set)]
    pub order_id: u64,
    /// The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(encode_order(4), fixed_price)]
    #[pyo3(get, set)]
    pub price: i64,
    /// The order quantity.
    #[dbn(encode_order(5))]
    #[pyo3(get, set)]
    pub size: u32,
    /// A bit field indicating event end, message characteristics, and data quality.
    /// See [`flags`](crate::flags) for possible values.
    #[pyo3(get, set)]
    pub flags: FlagSet,
    /// The channel ID assigned by Databento as an incrementing integer starting at zero.
    #[dbn(encode_order(6))]
    #[pyo3(get, set)]
    pub channel_id: u8,
    /// The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R** book, **T**rade, **F**ill, or **N**one.
    ///
    /// See [Action](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#action).
    #[dbn(c_char, encode_order(2))]
    pub action: c_char,
    /// The side that initiates the event. Can be **A**sk for a sell order (or sell aggressor in
    /// a trade), **B**id for a buy order (or buy aggressor in a trade), or **N**one where no side is specified.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    #[dbn(c_char, encode_order(3))]
    pub side: c_char,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    #[pyo3(get, set)]
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    #[pyo3(get, set)]
    pub sequence: u32,
}

/// A price level.
#[repr(C)]
#[derive(Clone, JsonSerialize, RecordDebug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, eq, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
pub struct BidAskPair {
    /// The bid price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub bid_px: i64,
    /// The ask price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub ask_px: i64,
    /// The bid size.
    #[pyo3(get, set)]
    pub bid_sz: u32,
    /// The ask size.
    #[pyo3(get, set)]
    pub ask_sz: u32,
    /// The bid order count.
    #[pyo3(get, set)]
    pub bid_ct: u32,
    /// The ask order count.
    #[pyo3(get, set)]
    pub ask_ct: u32,
}

/// A price level consolidated from multiple venues.
#[repr(C)]
#[derive(Clone, JsonSerialize, RecordDebug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, eq, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
pub struct ConsolidatedBidAskPair {
    /// The bid price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub bid_px: i64,
    /// The ask price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub ask_px: i64,
    /// The bid size.
    #[pyo3(get, set)]
    pub bid_sz: u32,
    /// The ask size.
    #[pyo3(get, set)]
    pub ask_sz: u32,
    /// The publisher ID indicating the venue containing the best bid.
    ///
    /// See [Publishers](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#publishers-datasets-and-venues).
    #[dbn(fmt_method)]
    #[pyo3(get, set)]
    pub bid_pb: u16,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved1: [u8; 2],
    /// The publisher ID indicating the venue containing the best ask.
    ///
    /// See [Publishers](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#publishers-datasets-and-venues).
    #[dbn(fmt_method)]
    #[pyo3(get, set)]
    pub ask_pb: u16,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved2: [u8; 2],
}

/// Market-by-price implementation with a book depth of 0. Equivalent to MBP-0. The record of the [`Trades`](crate::Schema::Trades) schema.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, eq, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::MBP_0)]
pub struct TradeMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The trade price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub price: i64,
    /// The order quantity.
    #[pyo3(get, set)]
    pub size: u32,
    /// The event action. Always **T**rade in the trades schema.
    ///
    /// See [Action](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#action).
    #[dbn(c_char, encode_order(2))]
    pub action: c_char,
    /// The side that initiates the trade. Can be **A**sk for a sell aggressor in a trade, **B**id for a buy aggressor in a trade, or **N**one where no side is specified.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    #[dbn(c_char, encode_order(3))]
    pub side: c_char,
    /// A bit field indicating event end, message characteristics, and data quality.
    /// See [`flags`](crate::flags) for possible values.
    #[pyo3(get, set)]
    pub flags: FlagSet,
    /// The book level where the update event occurred.
    #[dbn(encode_order(4))]
    #[pyo3(get, set)]
    pub depth: u8,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    #[pyo3(get, set)]
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    #[pyo3(get, set)]
    pub sequence: u32,
}

/// Market-by-price implementation with a known book depth of 1. The record of the
/// [`Mbp1`](crate::Schema::Mbp1) schema.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, eq, module = "databento_dbn", name = "MBP1Msg"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::MBP_1)]
pub struct Mbp1Msg {
    /// The common header.
    pub hd: RecordHeader,
    /// The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub price: i64,
    /// The order quantity.
    #[pyo3(get, set)]
    pub size: u32,
    /// The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R** book, or **T**rade.
    ///
    /// See [Action](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#action).
    #[dbn(c_char, encode_order(2))]
    pub action: c_char,
    /// The side that initiates the event. Can be **A**sk for a sell order (or sell aggressor in
    /// a trade), **B**id for a buy order (or buy aggressor in a trade), or **N**one where no side is specified.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    #[dbn(c_char, encode_order(3))]
    pub side: c_char,
    /// A bit field indicating event end, message characteristics, and data quality.
    /// See [`flags`](crate::flags) for possible values.
    #[pyo3(get, set)]
    pub flags: FlagSet,
    /// The book level where the update event occurred.
    #[dbn(encode_order(4))]
    #[pyo3(get, set)]
    pub depth: u8,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    #[pyo3(get, set)]
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    #[pyo3(get, set)]
    pub sequence: u32,
    /// The top of the order book.
    #[pyo3(get, set)]
    pub levels: [BidAskPair; 1],
}

/// Market-by-price implementation with a known book depth of 10. The record of the
/// [`Mbp10`](crate::Schema::Mbp10) schema.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, eq, module = "databento_dbn", name = "MBP10Msg"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::MBP_10)]
pub struct Mbp10Msg {
    /// The common header.
    pub hd: RecordHeader,
    /// The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub price: i64,
    /// The order quantity.
    #[pyo3(get, set)]
    pub size: u32,
    /// The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R** book, or **T**rade.
    ///
    /// See [Action](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#action).
    #[dbn(c_char, encode_order(2))]
    pub action: c_char,
    /// The side that initiates the event. Can be **A**sk for a sell order (or sell aggressor in
    /// a trade), **B**id for a buy order (or buy aggressor in a trade), or **N**one where no side is specified.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    #[dbn(c_char, encode_order(3))]
    pub side: c_char,
    /// A bit field indicating event end, message characteristics, and data quality.
    /// See [`flags`](crate::flags) for possible values.
    #[pyo3(get, set)]
    pub flags: FlagSet,
    /// The book level where the update event occurred.
    #[dbn(encode_order(4))]
    #[pyo3(get, set)]
    pub depth: u8,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
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
    pyo3::pyclass(dict, eq, module = "databento_dbn", name = "BBOMsg"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::BBO_1S, rtype::BBO_1M)]
pub struct BboMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The last trade price price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001. Will be [`UNDEF_PRICE`](crate::UNDEF_PRICE) if there was no last trade
    /// in the session.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub price: i64,
    /// The quantity of the last trade.
    #[pyo3(get, set)]
    pub size: u32,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved1: u8,
    /// The side that initiated the last trade. Can be **A**sk for a sell order (or sell
    /// aggressor in a trade), **B**id for a buy order (or buy aggressor in a trade), or
    /// **N**one where no side is specified.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    #[dbn(c_char, encode_order(2))]
    pub side: c_char,
    /// A bit field indicating event end, message characteristics, and data quality.
    /// See [`flags`](crate::flags) for possible values.
    #[pyo3(get, set)]
    pub flags: FlagSet,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved2: u8,
    /// The end timestamp of the interval capture-server-received timestamp expressed as the
    /// number of nanoseconds since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved3: [u8; 4],
    /// The message sequence number assigned at the venue of the last update.
    #[pyo3(get, set)]
    pub sequence: u32,
    /// The top of the order book.
    #[pyo3(get, set)]
    pub levels: [BidAskPair; 1],
}

/// Consolidated market-by-price implementation with a known book depth of 1. The record of
/// the [`Cmbp1`](crate::Schema::Cmbp1) schema.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, eq, module = "databento_dbn", name = "CMBP1Msg"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::CMBP_1, rtype::TCBBO)]
pub struct Cmbp1Msg {
    /// The common header.
    pub hd: RecordHeader,
    /// The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub price: i64,
    /// The order quantity.
    #[pyo3(get, set)]
    pub size: u32,
    /// The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R** book, or **T**rade.
    ///
    /// See [Action](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#action).
    #[dbn(c_char, encode_order(2))]
    pub action: c_char,
    /// The side that initiates the event. Can be **A**sk for a sell order (or sell aggressor in
    /// a trade), **B**id for a buy order (or buy aggressor in a trade), or **N**one where no side is specified.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    #[dbn(c_char, encode_order(3))]
    pub side: c_char,
    /// A bit field indicating event end, message characteristics, and data quality.
    /// See [`flags`](crate::flags) for possible values.
    #[pyo3(get, set)]
    pub flags: FlagSet,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved1: [u8; 1],
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    #[pyo3(get, set)]
    pub ts_in_delta: i32,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved2: [u8; 4],
    /// The top of the order book.
    #[pyo3(get, set)]
    pub levels: [ConsolidatedBidAskPair; 1],
}

/// Subsampled consolidated market by price with a known book depth of 1. The record of the [`Cbbo1S`](crate::Schema::Cbbo1S) and [`Cbbo1M`](crate::Schema::Cbbo1M) schemas.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, eq, module = "databento_dbn", name = "CBBOMsg"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::CBBO_1S, rtype::CBBO_1M)]
pub struct CbboMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The last trade price price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001. Will be [`UNDEF_PRICE`](crate::UNDEF_PRICE) if there was no last trade
    /// in the session.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub price: i64,
    /// The quantity of the last trade.
    #[pyo3(get, set)]
    pub size: u32,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved1: u8,
    /// The side that initiated the last trade. Can be **A**sk for a sell order (or sell
    /// aggressor in a trade), **B**id for a buy order (or buy aggressor in a trade), or
    /// **N**one where no side is specified.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    #[dbn(c_char, encode_order(2))]
    pub side: c_char,
    /// A bit field indicating event end, message characteristics, and data quality.
    /// See [`flags`](crate::flags) for possible values.
    #[pyo3(get, set)]
    pub flags: FlagSet,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved2: u8,
    /// The end timestamp of the interval capture-server-received timestamp expressed as the
    /// number of nanoseconds since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved3: [u8; 8],
    /// The top of the order book.
    #[pyo3(get, set)]
    pub levels: [ConsolidatedBidAskPair; 1],
}

/// The record of the [`Tbbo`](crate::Schema::Tbbo) schema.
pub type TbboMsg = Mbp1Msg;

/// The record of the [`Bbo1S`](crate::Schema::Bbo1S) schema.
pub type Bbo1SMsg = BboMsg;

/// The record of the [`Bbo1M`](crate::Schema::Bbo1M) schema.
pub type Bbo1MMsg = BboMsg;

/// The record of the [`Tcbbo`](crate::Schema::Tcbbo) schema.
pub type TcbboMsg = Cmbp1Msg;

/// The record of the [`Cbbo1S`](crate::Schema::Cbbo1S) schema.
pub type Cbbo1SMsg = CbboMsg;

/// The record of the [`Cbbo1M`](crate::Schema::Cbbo1M) schema.
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
    pyo3::pyclass(dict, eq, module = "databento_dbn", name = "OHLCVMsg"),
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
    /// The open price for the bar where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub open: i64,
    /// The high price for the bar where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub high: i64,
    /// The low price for the bar where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub low: i64,
    /// The close price for the bar where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub close: i64,
    /// The total volume traded during the aggregation period.
    #[pyo3(get, set)]
    pub volume: u64,
}

/// A trading status update message. The record of the [`Status`](crate::Schema::Status) schema.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, eq, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::STATUS)]
pub struct StatusMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
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
    /// The best-efforts state of trading in the instrument, either `Y`, `N` or `~`.
    #[dbn(c_char)]
    pub is_trading: c_char,
    /// The best-efforts state of quoting in the instrument, either `Y`, `N` or `~`.
    #[dbn(c_char)]
    pub is_quoting: c_char,
    /// The best-efforts state of short sell restrictions for the instrument (if applicable), either `Y`, `N` or `~`.
    #[dbn(c_char)]
    pub is_short_sell_restricted: c_char,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved: [u8; 7],
}

/// A definition of an instrument. The record of the
/// [`Definition`](crate::Schema::Definition) schema.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, eq, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::INSTRUMENT_DEF)]
pub struct InstrumentDefMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The minimum constant tick for the instrument where every 1 unit corresponds to 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub min_price_increment: i64,
    /// The multiplier to convert the venue's display price to the conventional price where every
    /// 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub display_factor: i64,
    /// The last eligible trade time expressed as the number of nanoseconds since the
    /// UNIX epoch.
    ///
    /// Will be [`crate::UNDEF_TIMESTAMP`] when null, such as for equities. Some publishers
    /// only provide date-level granularity.
    #[dbn(unix_nanos)]
    #[pyo3(get, set)]
    pub expiration: u64,
    /// The time of instrument activation expressed as the number of nanoseconds since the
    /// UNIX epoch.
    ///
    /// Will be [`crate::UNDEF_TIMESTAMP`] when null, such as for equities. Some publishers
    /// only provide date-level granularity.
    #[dbn(unix_nanos)]
    #[pyo3(get, set)]
    pub activation: u64,
    /// The allowable high limit price for the trading day where every 1 unit corresponds to
    /// 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub high_limit_price: i64,
    /// The allowable low limit price for the trading day where every 1 unit corresponds to
    /// 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub low_limit_price: i64,
    /// The differential value for price banding where every 1 unit corresponds to 1e-9,
    /// i.e. 1/1,000,000,000 or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub max_price_variation: i64,
    /// The contract size for each instrument, in combination with `unit_of_measure`, where every
    /// 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub unit_of_measure_qty: i64,
    /// The value currently under development by the venue where every 1 unit corresponds to 1e-9,
    /// i.e. 1/1,000,000,000 or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub min_price_increment_amount: i64,
    /// The value used for price calculation in spread and leg pricing where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub price_ratio: i64,
    /// The strike price of the option where every 1 unit corresponds to 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(encode_order(54), fixed_price)]
    #[pyo3(get, set)]
    pub strike_price: i64,
    /// The instrument ID assigned by the publisher. May be the same as `instrument_id`.
    ///
    /// See [Instrument identifiers](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers)
    #[dbn(encode_order(20))]
    #[pyo3(get, set)]
    pub raw_instrument_id: u64,
    /// The tied price (if any) of the leg.
    #[dbn(encode_order(165), fixed_price)]
    #[pyo3(get, set)]
    pub leg_price: i64,
    /// The associated delta (if any) of the leg.
    #[dbn(encode_order(166), fixed_price)]
    #[pyo3(get, set)]
    pub leg_delta: i64,
    /// A bitmap of instrument eligibility attributes.
    #[dbn(fmt_binary)]
    #[pyo3(get, set)]
    pub inst_attrib_value: i32,
    /// The `instrument_id` of the first underlying instrument.
    ///
    /// See [Instrument identifiers](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers).
    #[pyo3(get, set)]
    pub underlying_id: u32,
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
    /// The minimum quantity required for a round lot of the instrument. Multiples of this
    /// quantity are also round lots.
    #[pyo3(get, set)]
    pub min_lot_size_round_lot: i32,
    /// The minimum trading volume for the instrument.
    #[pyo3(get, set)]
    pub min_trade_vol: u32,
    /// The number of deliverables per instrument, i.e. peak days.
    #[pyo3(get, set)]
    pub contract_multiplier: i32,
    /// The quantity that a contract will decay daily, after `decay_start_date` has been reached.
    #[pyo3(get, set)]
    pub decay_quantity: i32,
    /// The fixed contract value assigned to each instrument.
    #[pyo3(get, set)]
    pub original_contract_size: i32,
    /// The numeric ID assigned to the leg instrument.
    ///
    /// See [Instrument identifiers](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers).
    #[dbn(encode_order(160))]
    #[pyo3(get, set)]
    pub leg_instrument_id: u32,
    /// The numerator of the price ratio of the leg within the spread.
    #[dbn(encode_order(167))]
    #[pyo3(get, set)]
    pub leg_ratio_price_numerator: i32,
    /// The denominator of the price ratio of the leg within the spread.
    #[dbn(encode_order(168))]
    #[pyo3(get, set)]
    pub leg_ratio_price_denominator: i32,
    /// The numerator of the quantity ratio of the leg within the spread.
    #[dbn(encode_order(169))]
    #[pyo3(get, set)]
    pub leg_ratio_qty_numerator: i32,
    /// The denominator of the quantity ratio of the leg within the spread.
    #[dbn(encode_order(170))]
    #[pyo3(get, set)]
    pub leg_ratio_qty_denominator: i32,
    /// The numeric ID of the leg instrument's underlying instrument.
    ///
    /// See [Instrument identifiers](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers).
    #[dbn(encode_order(171))]
    #[pyo3(get, set)]
    pub leg_underlying_id: u32,
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
    /// The number of legs in the strategy or spread. Will be 0 for outrights.
    #[dbn(encode_order(158))]
    #[pyo3(get, set)]
    pub leg_count: u16,
    /// The 0-based index of the leg.
    #[dbn(encode_order(159))]
    #[pyo3(get, set)]
    pub leg_index: u16,
    /// The currency used for price fields.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub currency: [c_char; 4],
    /// The currency used for settlement, if different from `currency`.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub settl_currency: [c_char; 4],
    /// The strategy type of the spread.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub secsubtype: [c_char; 6],
    /// The instrument raw symbol assigned by the publisher.
    #[dbn(encode_order(2), fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub raw_symbol: [c_char; SYMBOL_CSTR_LEN],
    /// The security group code of the instrument.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub group: [c_char; 21],
    /// The exchange used to identify the instrument.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub exchange: [c_char; 5],
    /// The underlying asset code (product code) of the instrument.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub asset: [c_char; ASSET_CSTR_LEN],
    /// The ISO standard instrument categorization code.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub cfi: [c_char; 7],
    /// The security type of the instrument, e.g. FUT for future or future spread.
    ///
    /// See [Security type](https://databento.com/docs/schemas-and-data-formats/instrument-definitions#security-type).
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub security_type: [c_char; 7],
    /// The unit of measure for the instrument’s original contract size, e.g. USD or LBS.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub unit_of_measure: [c_char; 31],
    /// The symbol of the first underlying instrument.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub underlying: [c_char; 21],
    /// The currency of [`strike_price`](Self::strike_price).
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub strike_price_currency: [c_char; 4],
    /// The leg instrument's raw symbol assigned by the publisher.
    #[dbn(encode_order(161), fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub leg_raw_symbol: [c_char; SYMBOL_CSTR_LEN],
    /// The classification of the instrument.
    ///
    /// See [Instrument class](https://databento.com/docs/schemas-and-data-formats/instrument-definitions#instrument-class).
    #[dbn(c_char, encode_order(4))]
    pub instrument_class: c_char,
    /// The matching algorithm used for the instrument, typically **F**IFO.
    ///
    /// See [Matching algorithm](https://databento.com/docs/schemas-and-data-formats/instrument-definitions#matching-algorithm).
    #[dbn(c_char)]
    pub match_algorithm: c_char,
    /// The price denominator of the main fraction.
    #[pyo3(get, set)]
    pub main_fraction: u8,
    /// The number of digits to the right of the tick mark, to display fractional prices.
    #[pyo3(get, set)]
    pub price_display_format: u8,
    /// The price denominator of the sub fraction.
    #[pyo3(get, set)]
    pub sub_fraction: u8,
    /// The product complex of the instrument.
    #[pyo3(get, set)]
    pub underlying_product: u8,
    /// Indicates if the instrument definition has been added, modified, or deleted.
    #[dbn(c_char, encode_order(3))]
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
    #[dbn(c_char)]
    pub user_defined_instrument: c_char,
    /// The type of `contract_multiplier`. Either `1` for hours, or `2` for days.
    #[pyo3(get, set)]
    pub contract_multiplier_unit: i8,
    /// The schedule for delivering electricity.
    #[pyo3(get, set)]
    pub flow_schedule_type: i8,
    /// The tick rule of the spread.
    #[pyo3(get, set)]
    pub tick_rule: u8,
    /// The classification of the leg instrument.
    #[dbn(c_char, encode_order(163))]
    pub leg_instrument_class: c_char,
    /// The side taken for the leg when purchasing the spread.
    #[dbn(c_char, encode_order(164))]
    pub leg_side: c_char,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved: [u8; 17],
}

/// An auction imbalance message.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, eq, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::IMBALANCE)]
pub struct ImbalanceMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The price at which the imbalance shares are calculated, where every 1 unit corresponds
    /// to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub ref_price: i64,
    /// Reserved for future use.
    #[dbn(unix_nanos)]
    #[pyo3(get, set)]
    pub auction_time: u64,
    /// The hypothetical auction-clearing price for both cross and continuous orders where every
    /// 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub cont_book_clr_price: i64,
    /// The hypothetical auction-clearing price for cross orders only where every 1 unit corresponds
    /// to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
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
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
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
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved: [u8; 1],
}

/// A statistics message. A catchall for various data disseminated by publishers. The
/// [`stat_type`](Self::stat_type) indicates the statistic contained in the message.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, eq, module = "databento_dbn"),
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
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    #[pyo3(get, set)]
    pub ts_recv: u64,
    /// The reference timestamp of the statistic value expressed as the number of
    /// nanoseconds since the UNIX epoch. Will be [`crate::UNDEF_TIMESTAMP`] when
    /// unused.
    #[dbn(unix_nanos)]
    #[pyo3(get, set)]
    pub ts_ref: u64,
    /// The value for price statistics where every 1 unit corresponds to 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001. Will be [`UNDEF_PRICE`](crate::UNDEF_PRICE)
    /// when unused.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub price: i64,
    /// The value for non-price statistics. Will be [`crate::UNDEF_STAT_QUANTITY`] when
    /// unused.
    #[pyo3(get, set)]
    pub quantity: i64,
    /// The message sequence number assigned at the venue.
    #[pyo3(get, set)]
    pub sequence: u32,
    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    #[pyo3(get, set)]
    pub ts_in_delta: i32,
    /// The type of statistic value contained in the message. Refer to the
    /// [`StatType`](crate::enums::StatType) enum for possible variants.
    #[dbn(fmt_method)]
    #[pyo3(get, set)]
    pub stat_type: u16,
    /// The channel ID assigned by Databento as an incrementing integer starting at zero.
    #[pyo3(get, set)]
    pub channel_id: u16,
    /// Indicates if the statistic is newly added (1) or deleted (2). (Deleted is only
    /// used with some stat types).
    #[dbn(fmt_method)]
    #[pyo3(get, set)]
    pub update_action: u8,
    /// Additional flags associate with certain stat types.
    #[dbn(fmt_binary)]
    #[pyo3(get, set)]
    pub stat_flags: u8,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved: [u8; 18],
}

/// An error message from the Databento Live Subscription Gateway (LSG).
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, eq, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::ERROR)]
pub struct ErrorMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The error message.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub err: [c_char; 302],
    /// The error code. See the [`ErrorCode`](crate::enums::ErrorCode) enum
    /// for possible values.
    #[dbn(fmt_method)]
    #[pyo3(get, set)]
    pub code: u8,
    /// Sometimes multiple errors are sent together. This field will be non-zero for the
    /// last error.
    #[pyo3(get, set)]
    pub is_last: u8,
}

/// A symbol mapping message from the live API which maps a symbol from one
/// [`SType`](crate::enums::SType) to another.
#[repr(C)]
#[derive(Clone, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, eq, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::SYMBOL_MAPPING)]
pub struct SymbolMappingMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The input symbology type of `stype_in_symbol`.
    #[dbn(fmt_method)]
    #[pyo3(get, set)]
    pub stype_in: u8,
    /// The input symbol.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub stype_in_symbol: [c_char; SYMBOL_CSTR_LEN],
    /// The output symbology type of `stype_out_symbol`.
    #[dbn(fmt_method)]
    #[pyo3(get, set)]
    pub stype_out: u8,
    /// The output symbol.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
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
    pyo3::pyclass(dict, eq, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::SYSTEM)]
pub struct SystemMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The message from the Databento gateway.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub msg: [c_char; 303],
    /// Type of system message. See the [`SystemCode`](crate::enums::SystemCode) enum
    /// for possible values.
    #[dbn(fmt_method)]
    #[pyo3(get, set)]
    pub code: u8,
}

/// Wrapper object for records that include the live gateway send timestamp (`ts_out`).
///
/// # Examples
/// ```
/// use dbn::{MboMsg, WithTsOut};
/// use std::time::SystemTime;
///
/// let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos() as u64;
/// let rec = WithTsOut::new(MboMsg::default(), now);
/// assert_eq!(rec.ts_out, now);
/// ```
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WithTsOut<T: HasRType> {
    /// The inner record.
    pub rec: T,
    /// The live gateway send timestamp expressed as the number of nanoseconds since the
    /// UNIX epoch.
    ///
    /// See [ts_out](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-out).
    pub ts_out: u64,
}
