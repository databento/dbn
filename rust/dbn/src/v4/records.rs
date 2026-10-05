//! Record types for DBN version 4.

use std::os::raw::c_char;

use crate::{
    macros::{dbn_record, DbnAttr},
    BidAskPair, ConsolidatedBidAskPair, FlagSet,
};

use super::{rtype, RecordHeader};

/// A market-by-order (MBO) tick message. The record of the [`Mbo`](crate::Schema::Mbo)
/// schema.
#[repr(C)]
#[derive(Clone, DbnAttr, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[dbn_record(rtype::MBO)]
pub struct MboMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The order ID assigned at the venue.
    pub order_id: u64,
    /// The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(encode_order(4), fixed_price)]
    pub price: i64,
    /// The order quantity.
    #[dbn(encode_order(5))]
    pub size: u32,
    /// A bit field indicating event end, message characteristics, and data quality.
    /// See [`flags`](crate::flags) for possible values.
    pub flags: FlagSet,
    /// The channel ID assigned by Databento as an incrementing integer starting at zero.
    #[dbn(encode_order(6))]
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
    pub ts_recv: u64,
    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    pub sequence: u32,
}

const _: () = assert!(std::mem::size_of::<MboMsg>() == 64);

/// Market-by-price implementation with a book depth of 0. Equivalent to MBP-0. The record of the [`Trades`](crate::Schema::Trades) schema.
#[repr(C)]
#[derive(Clone, DbnAttr, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[dbn_record(rtype::MBP_0)]
pub struct TradeMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The trade price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    pub price: i64,
    /// The order quantity.
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
    pub flags: FlagSet,
    #[doc(hidden)]
    #[dbn(encode_order(4))]
    pub _reserved: u8,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    pub ts_recv: u64,
    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    pub sequence: u32,
}

const _: () = assert!(std::mem::size_of::<TradeMsg>() == 56);

/// Market-by-price implementation with a known book depth of 1. The record of the
/// [`Mbp1`](crate::Schema::Mbp1) and [`Tbbo`](crate::Schema::Tbbo) schemas.
#[repr(C)]
#[derive(Clone, DbnAttr, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[dbn_record(rtype::MBP_1, rtype::TBBO)]
pub struct Mbp1Msg {
    /// The common header.
    pub hd: RecordHeader,
    /// The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    pub price: i64,
    /// The order quantity.
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
    pub flags: FlagSet,
    #[doc(hidden)]
    #[dbn(encode_order(4))]
    pub _reserved: u8,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    pub ts_recv: u64,
    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    pub sequence: u32,
    /// The top of the order book.
    pub levels: [BidAskPair; 1],
}

const _: () = assert!(std::mem::size_of::<Mbp1Msg>() == 88);

/// Market-by-price implementation with a known book depth of 10. The record of the
/// [`Mbp10`](crate::Schema::Mbp10) schema.
#[repr(C)]
#[derive(Clone, DbnAttr, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[dbn_record(rtype::MBP_10)]
pub struct Mbp10Msg {
    /// The common header.
    pub hd: RecordHeader,
    /// The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    pub price: i64,
    /// The order quantity.
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
    pub flags: FlagSet,
    /// The book level where the update event occurred.
    #[dbn(encode_order(4))]
    pub depth: u8,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    pub ts_recv: u64,
    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    pub sequence: u32,
    /// The top 10 levels of the order book.
    pub levels: [BidAskPair; 10],
}

const _: () = assert!(std::mem::size_of::<Mbp10Msg>() == 376);

/// Subsampled market by price with a known book depth of 1. The record of the
/// [`Bbo1S`](crate::Schema::Bbo1S) and [`Bbo1M`](crate::Schema::Bbo1M) schemas.
#[repr(C)]
#[derive(Clone, DbnAttr, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
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
    pub price: i64,
    /// The quantity of the last trade.
    pub size: u32,
    #[doc(hidden)]
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
    pub flags: FlagSet,
    #[doc(hidden)]
    pub _reserved2: u8,
    /// The end timestamp of the interval capture-server-received timestamp expressed as the
    /// number of nanoseconds since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    pub ts_recv: u64,
    #[doc(hidden)]
    pub _reserved3: [u8; 4],
    /// The message sequence number assigned at the venue of the last update.
    pub sequence: u32,
    /// The top of the order book.
    pub levels: [BidAskPair; 1],
}

const _: () = assert!(std::mem::size_of::<BboMsg>() == 88);

/// Consolidated market-by-price implementation with a known book depth of 1. The record of
/// the [`Cmbp1`](crate::Schema::Cmbp1) schema.
#[repr(C)]
#[derive(Clone, DbnAttr, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[dbn_record(rtype::CMBP_1, rtype::TCBBO)]
pub struct Cmbp1Msg {
    /// The common header.
    pub hd: RecordHeader,
    /// The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    pub price: i64,
    /// The order quantity.
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
    pub flags: FlagSet,
    #[doc(hidden)]
    pub _reserved1: [u8; 1],
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    pub ts_recv: u64,
    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    pub ts_in_delta: i32,
    #[doc(hidden)]
    pub _reserved2: [u8; 4],
    /// The top of the order book.
    pub levels: [ConsolidatedBidAskPair; 1],
}

const _: () = assert!(std::mem::size_of::<Cmbp1Msg>() == 88);

/// Subsampled consolidated market by price with a known book depth of 1. The record of the [`Cbbo1S`](crate::Schema::Cbbo1S) and [`Cbbo1M`](crate::Schema::Cbbo1M) schemas.
#[repr(C)]
#[derive(Clone, DbnAttr, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
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
    pub price: i64,
    /// The quantity of the last trade.
    pub size: u32,
    #[doc(hidden)]
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
    pub flags: FlagSet,
    #[doc(hidden)]
    pub _reserved2: u8,
    /// The end timestamp of the interval capture-server-received timestamp expressed as the
    /// number of nanoseconds since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    pub ts_recv: u64,
    #[doc(hidden)]
    pub _reserved3: [u8; 8],
    /// The top of the order book.
    pub levels: [ConsolidatedBidAskPair; 1],
}

const _: () = assert!(std::mem::size_of::<CbboMsg>() == 88);

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
#[derive(Clone, DbnAttr, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
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
    pub open: i64,
    /// The high price for the bar where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    pub high: i64,
    /// The low price for the bar where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    pub low: i64,
    /// The close price for the bar where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    pub close: i64,
    /// The total volume traded during the aggregation period.
    pub volume: u64,
}

const _: () = assert!(std::mem::size_of::<OhlcvMsg>() == 64);

/// A trading status update message. The record of the [`Status`](crate::Schema::Status) schema.
#[repr(C)]
#[derive(Clone, DbnAttr, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[dbn_record(rtype::STATUS)]
pub struct StatusMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    pub ts_recv: u64,
    /// The type of status change.
    #[dbn(fmt_method)]
    pub action: u16,
    /// Additional details about the cause of the status change.
    #[dbn(fmt_method)]
    pub reason: u16,
    /// Further information about the status change and its effect on trading.
    #[dbn(fmt_method)]
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
    pub _reserved: [u8; 7],
}

const _: () = assert!(std::mem::size_of::<StatusMsg>() == 48);

/// A definition of an instrument. The record of the
/// [`Definition`](crate::Schema::Definition) schema.
#[repr(C)]
#[derive(Clone, DbnAttr, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[dbn_record(rtype::INSTRUMENT_DEF)]
pub struct InstrumentDefMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    pub ts_recv: u64,
    /// The minimum constant tick for the instrument where every 1 unit corresponds to 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(encode_order(4), fixed_price)]
    pub min_price_increment: i64,
    /// The multiplier to convert the venue's display price to the conventional price where every
    /// 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    #[dbn(encode_order(5), fixed_price)]
    pub display_factor: i64,
    /// The instrument ID assigned by the publisher. May be the same as `instrument_id`.
    ///
    /// See [Instrument identifiers](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers)
    #[dbn(encode_order(8))]
    pub raw_instrument_id: u64,
    /// The minimum quantity required for a round lot of the instrument. Multiples of this
    /// quantity are also round lots.
    #[dbn(encode_order(6))]
    pub min_lot_size_round_lot: i32,
    /// Indicates if the instrument definition has been added, modified, or deleted.
    #[dbn(c_char, encode_order(2))]
    pub security_update_action: c_char,
    /// The classification of the instrument.
    ///
    /// See [Instrument class](https://databento.com/docs/schemas-and-data-formats/instrument-definitions#instrument-class).
    #[dbn(c_char, encode_order(3))]
    pub instrument_class: c_char,
    /// The matching algorithm used for the instrument, typically **F**IFO.
    ///
    /// See [Matching algorithm](https://databento.com/docs/schemas-and-data-formats/instrument-definitions#matching-algorithm).
    #[dbn(c_char, encode_order(7))]
    pub match_algorithm: c_char,
    /// The channel ID assigned by Databento as an incrementing integer starting at zero.
    #[dbn(encode_order(9))]
    pub channel_id: u8,
}

const _: () = assert!(std::mem::size_of::<InstrumentDefMsg>() == 64);

/// An auction imbalance message.
#[repr(C)]
#[derive(Clone, DbnAttr, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[dbn_record(rtype::IMBALANCE)]
pub struct ImbalanceMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    pub ts_recv: u64,
    /// The price at which the imbalance shares are calculated, where every 1 unit corresponds
    /// to 1e-9, i.e. 1/1,000,000,000 or 0.000000001. Will be [`UNDEF_PRICE`](crate::UNDEF_PRICE)
    /// when unused.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    pub ref_price: i64,
    /// The hypothetical auction-clearing price for both cross and continuous orders where every
    /// 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    /// Will be [`UNDEF_PRICE`](crate::UNDEF_PRICE) when unused.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    pub cont_book_clr_price: i64,
    /// The hypothetical auction-clearing price for cross orders only where every 1 unit corresponds
    /// to 1e-9, i.e. 1/1,000,000,000 or 0.000000001. Will be [`UNDEF_PRICE`](crate::UNDEF_PRICE) when unused.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    pub auct_interest_clr_price: i64,
    /// The quantity of shares that are eligible to be matched at [`ref_price`](Self::ref_price).
    /// Will be [`UNDEF_ORDER_SIZE`](crate::UNDEF_ORDER_SIZE) when unused.
    pub paired_qty: u32,
    /// The quantity of shares that are not paired at [`ref_price`](Self::ref_price).
    /// Will be [`UNDEF_ORDER_SIZE`](crate::UNDEF_ORDER_SIZE) when not used.
    pub total_imbalance_qty: u32,
    /// Venue-specific character code indicating the auction type. Will be `~` when unused.
    ///
    /// Refer to the [venue-specific documentation](https://databento.com/docs/venues-and-datasets).
    #[dbn(c_char)]
    pub auction_type: c_char,
    /// The market side of the [`total_imbalance_qty`](Self::total_imbalance_qty).
    /// Can be **A**sk, **B**id, or **N**one.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    #[dbn(c_char)]
    pub side: c_char,
    #[doc(hidden)]
    pub _reserved: [u8; 6],
}

const _: () = assert!(std::mem::size_of::<ImbalanceMsg>() == 72);

/// A statistics message. A catchall for various data disseminated by publishers. The
/// [`stat_type`](Self::stat_type) indicates the statistic contained in the message.
#[repr(C)]
#[derive(Clone, DbnAttr, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[dbn_record(rtype::STATISTICS)]
pub struct StatMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    #[dbn(encode_order(0), index_ts, unix_nanos)]
    pub ts_recv: u64,
    /// The reference timestamp of the statistic value expressed as the number of
    /// nanoseconds since the UNIX epoch. Will be [`UNDEF_TIMESTAMP`](crate::UNDEF_TIMESTAMP) when
    /// unused.
    #[dbn(unix_nanos)]
    pub ts_ref: u64,
    /// The value for price statistics where every 1 unit corresponds to 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001. Will be [`UNDEF_PRICE`](crate::UNDEF_PRICE)
    /// when unused.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    pub price: i64,
    /// The value for non-price statistics. Will be [`UNDEF_STAT_QUANTITY`](crate::UNDEF_STAT_QUANTITY)
    /// when unused.
    pub quantity: i64,
    /// The message sequence number assigned at the venue.
    pub sequence: u32,
    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    pub ts_in_delta: i32,
    /// The type of statistic value contained in the message. Refer to the
    /// [`StatType`](crate::enums::StatType) enum for possible variants.
    #[dbn(fmt_method)]
    pub stat_type: u16,
    /// The channel ID assigned by Databento as an incrementing integer starting at zero.
    pub channel_id: u8,
    /// Indicates if the statistic is newly added (1) or deleted (2). (Deleted is only
    /// used with some stat types).
    #[dbn(fmt_method)]
    pub update_action: u8,
    /// Additional flags associate with certain stat types.
    #[dbn(fmt_binary)]
    pub stat_flags: u8,
    #[doc(hidden)]
    pub _reserved: [u8; 3],
}

const _: () = assert!(std::mem::size_of::<StatMsg>() == 72);

/// An error message from the Databento Live Subscription Gateway (LSG).
#[repr(C)]
#[derive(Clone, DbnAttr, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[dbn_record(rtype::ERROR)]
pub struct ErrorMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The ID of the subscription that triggered the error, if applicable.
    pub subscription_id: u32,
    /// The schema associated with the error, if applicable.
    #[dbn(fmt_method)]
    pub schema: u16,
    /// The error code. See the [`ErrorCode`](crate::enums::ErrorCode) enum
    /// for possible values.
    #[dbn(fmt_method)]
    pub code: u8,
    /// Sometimes multiple errors are sent together. This field will be non-zero for the
    /// last error.
    pub is_last: u8,
    #[doc(hidden)]
    pub _reserved: [u8; 2],
    /// The error message.
    #[dbn(fmt_method)]
    pub err: [c_char; 302],
}

const _: () = assert!(std::mem::size_of::<ErrorMsg>() == 336);

/// A symbol mapping message from the live API which maps a symbol from one
/// [`SType`](crate::enums::SType) to another.
#[repr(C)]
#[derive(Clone, DbnAttr, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[dbn_record(rtype::SYMBOL_MAPPING)]
pub struct SymbolMappingMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The start of the mapping interval expressed as the number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(unix_nanos)]
    pub start_ts: u64,
    /// The end of the mapping interval expressed as the number of nanoseconds since
    /// the UNIX epoch.
    #[dbn(unix_nanos)]
    pub end_ts: u64,
    /// The input symbology type of `stype_in_symbol`.
    #[dbn(fmt_method)]
    pub stype_in: u8,
    /// The output symbology type of `stype_out_symbol`. Will always be [`RawSymbol`](crate::SType::RawSymbol).
    #[dbn(fmt_method)]
    pub stype_out: u8,
    #[doc(hidden)]
    pub _reserved: [u8; 6],
}

const _: () = assert!(std::mem::size_of::<SymbolMappingMsg>() == 48);

/// A non-error message from the Databento Live Subscription Gateway (LSG). Also used
/// for heartbeating.
#[repr(C)]
#[derive(Clone, DbnAttr, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[dbn_record(rtype::SYSTEM)]
pub struct SystemMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The ID of the subscription that triggered the message, if applicable.
    pub subscription_id: u32,
    /// The schema associated with the message, if applicable.
    #[dbn(fmt_method)]
    pub schema: u16,
    /// Type of system message. See the [`SystemCode`](crate::enums::SystemCode) enum
    /// for possible values.
    #[dbn(fmt_method)]
    pub code: u8,
    #[doc(hidden)]
    pub _reserved: [u8; 2],
    /// The message from the Databento gateway.
    #[dbn(fmt_method)]
    pub msg: [c_char; 303],
}

const _: () = assert!(std::mem::size_of::<SystemMsg>() == 336);
