//! Market data types for encoding different Databento [`Schema`](crate::enums::Schema)s and conversion functions.
use crate::enums::rtype;
use std::{ffi::CStr, mem, os::raw::c_char, ptr::NonNull, str::Utf8Error};

use serde::Serialize;

use crate::enums::SecurityUpdateAction;

/// Common data for all Databento records.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, module = "databento_dbn")
)]
pub struct RecordHeader {
    /// The length of the message in 32-bit words.
    #[serde(skip)]
    pub(crate) length: u8,
    /// The record type; with `0x00..0x0F` specifying MBP booklevel size.
    /// Record types implement the trait [`HasRType`], and the [`has_rtype`][HasRType::has_rtype]
    /// function can be used to check if that type can be used to decode a message with a given rtype.
    /// The set of possible values is defined in [`rtype`].
    pub rtype: u8,
    /// The publisher ID assigned by Databento.
    pub publisher_id: u16,
    /// The product ID assigned by the venue.
    pub product_id: u32,
    /// The matching-engine-received timestamp expressed as number of nanoseconds since UNIX epoch.
    #[serde(serialize_with = "serialize_large_u64")]
    pub ts_event: u64,
}

/// A market-by-order (MBO) tick message. The record of the
/// [`Mbo`](crate::enums::Schema::Mbo) schema.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, module = "databento_dbn")
)]
pub struct MboMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The order ID assigned at the venue.
    pub order_id: u64,
    /// The order price expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    pub price: i64,
    /// The order quantity.
    pub size: u32,
    /// A combination of packet end with matching engine status.
    pub flags: u8,
    /// A channel ID within the venue.
    pub channel_id: u8,
    /// The event action. Can be M\[odify\], T\[rade\], C\[ancel\], A\[dd\]
    /// or special: \[S\]tatus, \[U\]pdate.
    pub action: c_char,
    /// The order side. Can be A\[sk\], B\[id\] or N\[one\].
    pub side: c_char,
    /// The capture-server-received timestamp expressed as number of nanoseconds since UNIX epoch.
    #[serde(serialize_with = "serialize_large_u64")]
    pub ts_recv: u64,
    /// The delta of `ts_recv - ts_exchange_send`, max 2 seconds.
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    pub sequence: u32,
}

/// A book level.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, module = "databento_dbn")
)]
pub struct BidAskPair {
    /// The bid price.
    pub bid_px: i64,
    /// The ask price.
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, module = "databento_dbn")
)]
pub struct TradeMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The order price expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    pub price: i64,
    /// The order quantity.
    pub size: u32,
    /// The event action. Can be M\[odify\], T\[rade\], C\[ancel\], A\[dd\]
    /// or special: \[S\]tatus, \[U\]pdate.
    pub action: c_char,
    /// The order side. Can be A\[sk\], B\[id\] or N\[one\].
    pub side: c_char,
    /// A combination of packet end with matching engine status.
    pub flags: u8,
    /// The depth of actual book change.
    pub depth: u8,
    /// The capture-server-received timestamp expressed as number of nanoseconds since UNIX epoch.
    #[serde(serialize_with = "serialize_large_u64")]
    pub ts_recv: u64,
    /// The delta of `ts_recv - ts_exchange_send`, max 2 seconds.
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    pub sequence: u32,
    #[serde(skip)]
    pub booklevel: [BidAskPair; 0],
}

/// Market by price implementation with a known book depth of 1. The record of the
/// [`Mbp1`](crate::enums::Schema::Mbp1) schema.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, module = "databento_dbn")
)]
pub struct Mbp1Msg {
    /// The common header.
    pub hd: RecordHeader,
    /// The order price expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    pub price: i64,
    /// The order quantity.
    pub size: u32,
    /// The event action. Can be M\[odify\], T\[rade\], C\[ancel\], A\[dd\]
    /// or special: \[S\]tatus, \[U\]pdate.
    pub action: c_char,
    /// The order side. Can be A\[sk\], B\[id\] or N\[one\].
    pub side: c_char,
    /// A combination of packet end with matching engine status.
    pub flags: u8,
    /// The depth of actual book change.
    pub depth: u8,
    /// The capture-server-received timestamp expressed as number of nanoseconds since UNIX epoch.
    #[serde(serialize_with = "serialize_large_u64")]
    pub ts_recv: u64,
    /// The delta of `ts_recv - ts_exchange_send`, max 2 seconds.
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    pub sequence: u32,
    pub booklevel: [BidAskPair; 1],
}

/// Market by price implementation with a known book depth of 10. The record of the
/// [`Mbp10`](crate::enums::Schema::Mbp10) schema.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, module = "databento_dbn")
)]
pub struct Mbp10Msg {
    /// The common header.
    pub hd: RecordHeader,
    /// The order price expressed as a signed integer where every 1 unit
    /// corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    pub price: i64,
    /// The order quantity.
    pub size: u32,
    /// The event action. Can be M\[odify\], T\[rade\], C\[ancel\], A\[dd\]
    /// or special: \[S\]tatus, \[U\]pdate.
    pub action: c_char,
    /// The order side. Can be A\[sk\], B\[id\] or N\[one\].
    pub side: c_char,
    /// A combination of packet end with matching engine status.
    pub flags: u8,
    /// The depth of actual book change.
    pub depth: u8,
    /// The capture-server-received timestamp expressed as number of nanoseconds since UNIX epoch.
    #[serde(serialize_with = "serialize_large_u64")]
    pub ts_recv: u64,
    /// The delta of `ts_recv - ts_exchange_send`, max 2 seconds.
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    pub sequence: u32,
    pub booklevel: [BidAskPair; 10],
}

/// The record of the [`Tbbo`](crate::enums::Schema::Tbbo) schema.
pub type TbboMsg = Mbp1Msg;

/// Open, high, low, close, and volume. The record of the following schemas:
/// - [`Ohlcv1S`](crate::enums::Schema::Ohlcv1S)
/// - [`Ohlcv1M`](crate::enums::Schema::Ohlcv1M)
/// - [`Ohlcv1H`](crate::enums::Schema::Ohlcv1H)
/// - [`Ohlcv1D`](crate::enums::Schema::Ohlcv1D)
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, module = "databento_dbn")
)]
pub struct OhlcvMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The open price for the bar.
    pub open: i64,
    /// The high price for the bar.
    pub high: i64,
    /// The low price for the bar.
    pub low: i64,
    /// The close price for the bar.
    pub close: i64,
    /// The total volume traded during the aggregation period.
    pub volume: u64,
}

/// Trading status update message. The record of the
/// [`Status`](crate::enums::Schema::Status) schema.
#[doc(hidden)]
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
pub struct StatusMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as number of nanoseconds since UNIX epoch.
    #[serde(serialize_with = "serialize_large_u64")]
    pub ts_recv: u64,
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub group: [c_char; 21],
    pub trading_status: u8,
    pub halt_reason: u8,
    pub trading_event: u8,
}

/// Definition of an instrument. The record of the
/// [`Definition`](crate::enums::Schema::Definition) schema.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, module = "databento_dbn")
)]
pub struct InstrumentDefMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The capture-server-received timestamp expressed as number of nanoseconds since
    /// UNIX epoch.
    #[serde(serialize_with = "serialize_large_u64")]
    pub ts_recv: u64,
    /// The minimum constant tick for the instrument in units of 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001.
    pub min_price_increment: i64,
    /// The multiplier to convert the venue’s display price to the conventional price.
    pub display_factor: i64,
    /// The time of instrument activation expressed as a number of nanoseconds since UNIX epoch.
    #[serde(serialize_with = "serialize_large_u64")]
    pub expiration: u64,
    /// The last eligible trade time expressed as a number of nanoseconds since UNIX epoch.
    #[serde(serialize_with = "serialize_large_u64")]
    pub activation: u64,
    /// The allowable high limit price for the trading day in units of 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001.
    pub high_limit_price: i64,
    /// The allowable low limit price for the trading day in units of 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001.
    pub low_limit_price: i64,
    /// The differential value for price banding in units of 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001.
    pub max_price_variation: i64,
    /// The trading session date corresponding to the settlement price in
    /// `trading_reference_price,` in number of days since the UNIX epoch.
    pub trading_reference_price: i64,
    /// The contract size for each instrument, in combination with `unit_of_measure`.
    pub unit_of_measure_qty: i64,
    /// The value currently under development by the venue. Converted to units of 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001.
    pub min_price_increment_amount: i64,
    /// The value used for price calculation in spread and leg pricing in units of 1e-9,
    /// i.e. 1/1,000,000,000 or 0.000000001.
    pub price_ratio: i64,
    /// A bitmap of instrument eligibility attributes.
    pub inst_attrib_value: i32,
    /// The `product_id` of the first underlying instrument.
    pub underlying_id: u32,
    /// The total cleared volume of the instrument traded during the prior trading session.
    pub cleared_volume: i32,
    /// The implied book depth on the price level data feed.
    pub market_depth_implied: i32,
    /// The (outright) book depth on the price level data feed.
    pub market_depth: i32,
    /// The market segment of the instrument.
    pub market_segment_id: u32,
    /// The maximum trading volume for the instrument.
    pub max_trade_vol: u32,
    /// The minimum order entry quantity for the instrument.
    pub min_lot_size: i32,
    /// The minimum quantity required for a block trade of the instrument.
    pub min_lot_size_block: i32,
    /// The minimum quantity required for a round lot of the instrument. Multiples of this quantity
    /// are also round lots.
    pub min_lot_size_round_lot: i32,
    /// The minimum trading volume for the instrument.
    pub min_trade_vol: u32,
    /// The total open interest for the market at the close of the prior trading session.
    pub open_interest_qty: i32,
    /// The number of deliverables per instrument, i.e. peak days.
    pub contract_multiplier: i32,
    /// The quantity that a contract will decay daily, after `decay_start_date` has been reached.
    pub decay_quantity: i32,
    /// The fixed contract value assigned to each instrument.
    pub original_contract_size: i32,
    #[doc(hidden)]
    pub related_security_id: u32,
    /// The trading session date corresponding to the settlement price in
    /// `trading_reference_price`, in number of days since the UNIX epoch.
    pub trading_reference_date: u16,
    /// The channel ID assigned at the venue.
    pub appl_id: i16,
    /// The calendar year reflected in the instrument symbol.
    pub maturity_year: u16,
    /// The date at which a contract will begin to decay.
    pub decay_start_date: u16,
    /// The channel ID assigned by Databento as an incrementing integer starting at zero.
    pub channel_id: u16,
    /// The currency used for price fields.
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub currency: [c_char; 4],
    /// The currency used for settlement, if different from `currency`.
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub settl_currency: [c_char; 4],
    /// The strategy type of the spread.
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub secsubtype: [c_char; 6],
    /// The instrument name (symbol).
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub symbol: [c_char; 22],
    /// The security group code of the instrument.
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub group: [c_char; 21],
    /// The exchange used to identify the instrument.
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub exchange: [c_char; 5],
    /// The underlying asset code (product code) of the instrument.
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub asset: [c_char; 7],
    /// The ISO standard instrument categorization code.
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub cfi: [c_char; 7],
    /// The type of the instrument, e.g. FUT for future or future spread.
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub security_type: [c_char; 7],
    /// The unit of measure for the instrument’s original contract size, e.g. USD or LBS.
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub unit_of_measure: [c_char; 31],
    /// The symbol of the first underlying instrument.
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub underlying: [c_char; 21],
    #[doc(hidden)]
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub related: [c_char; 21],
    /// The matching algorithm used for the instrument, typically **F**IFO.
    pub match_algorithm: c_char,
    /// The current trading state of the instrument.
    pub md_security_trading_status: u8,
    /// The price denominator of the main fraction.
    pub main_fraction: u8,
    ///  The number of digits to the right of the tick mark, to display fractional prices.
    pub price_display_format: u8,
    /// The type indicators for the settlement price, as a bitmap.
    pub settl_price_type: u8,
    /// The price denominator of the sub fraction.
    pub sub_fraction: u8,
    /// The product complex of the instrument.
    pub underlying_product: u8,
    /// Indicates if the instrument definition has been added, modified, or deleted.
    pub security_update_action: SecurityUpdateAction,
    /// The calendar month reflected in the instrument symbol.
    pub maturity_month: u8,
    /// The calendar day reflected in the instrument symbol, or 0.
    pub maturity_day: u8,
    /// The calendar week reflected in the instrument symbol, or 0.
    pub maturity_week: u8,
    /// Indicates if the instrument is user defined: **Y**es or **N**o.
    pub user_defined_instrument: c_char,
    /// The type of `contract_multiplier`. Either `1` for hours, or `2` for days.
    pub contract_multiplier_unit: i8,
    /// The schedule for delivering electricity.
    pub flow_schedule_type: i8,
    /// The tick rule of the spread.
    pub tick_rule: u8,
    /// Adjust filler for alignment.
    #[serde(skip)]
    pub _dummy: [c_char; 3],
}

/// Order imbalance message.
#[doc(hidden)]
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
pub struct ImbalanceMsg {
    pub hd: RecordHeader,
    #[serde(serialize_with = "serialize_large_u64")]
    pub ts_recv: u64,
    pub ref_price: i64,
    #[serde(serialize_with = "serialize_large_u64")]
    pub auction_time: u64,
    /// Continuous book clearing price.
    pub cont_book_clr_price: i64,
    /// Auction interest clearing price.
    pub auct_interest_clr_price: i64,
    // Short-selling restriction filling price.
    pub ssr_filling_price: i64,
    /// Indicative match price.
    pub ind_match_price: i64,
    pub upper_collar: i64,
    pub lower_collar: i64,
    pub paired_qty: u32,
    pub total_imbalance_qty: u32,
    pub market_imbalance_qty: u32,
    pub auction_type: c_char,
    pub side: c_char,
    pub auction_status: u8,
    pub freeze_status: u8,
    pub num_extensions: u8,
    pub unpaired_qty: u8,
    pub unpaired_side: c_char,
    pub significant_imbalance: c_char,
    #[serde(skip)]
    pub _dummy: [c_char; 4],
}

/// An error message from the Databento Live Subscription Gateway (LSG).
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, module = "databento_dbn")
)]
pub struct ErrorMsg {
    pub hd: RecordHeader,
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub err: [c_char; 64],
}

/// A symbol mapping message.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, module = "databento_dbn")
)]
pub struct SymbolMappingMsg {
    pub hd: RecordHeader,
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub stype_in_symbol: [c_char; 22],
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub stype_out_symbol: [c_char; 22],
    #[serde(skip)]
    pub _dummy: [c_char; 4],
    pub start_ts: u64,
    pub end_ts: u64,
}

/// A non-error message from the Databento Live Subscription Gateway (LSG). Also used
/// for heartbeating.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(get_all, set_all, module = "databento_dbn")
)]
pub struct SystemMsg {
    pub hd: RecordHeader,
    #[serde(serialize_with = "serialize_c_char_arr")]
    pub msg: [c_char; 64],
}

fn serialize_c_char_arr<S: serde::Serializer, const N: usize>(
    arr: &[c_char; N],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let cstr = unsafe { std::ffi::CStr::from_ptr(&arr[0]) };
    let str = cstr.to_str().unwrap_or("<invalid UTF-8>");
    serializer.serialize_str(str)
}

/// Serialize as a string to avoid any loss of precision with JSON serializers and parsers.
fn serialize_large_u64<S: serde::Serializer>(num: &u64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&num.to_string())
}

/// A trait for objects with polymorphism based around [`RecordHeader::rtype`]. All implementing
/// types begin with a [`RecordHeader`].
pub trait HasRType {
    /// Returns `true` if `rtype` matches the value associated with the implementing type.
    fn has_rtype(rtype: u8) -> bool;

    /// Returns the `RecordHeader` that comes at the beginning of all record types.
    fn header(&self) -> &RecordHeader;

    /// Returns the size of the record in bytes.
    fn size(&self) -> usize {
        self.header().record_size()
    }
}

impl RecordHeader {
    pub const LENGTH_MULTIPLIER: usize = 4;

    /// Creates a new `RecordHeader`. `R` and `rtype` should be compatible.
    pub const fn new<R: HasRType>(
        rtype: u8,
        publisher_id: u16,
        product_id: u32,
        ts_event: u64,
    ) -> Self {
        Self {
            length: (mem::size_of::<R>() / Self::LENGTH_MULTIPLIER) as u8,
            rtype,
            publisher_id,
            product_id,
            ts_event,
        }
    }

    /// Returns the size of the **entire** record in bytes. The size of a `RecordHeader`
    /// is constant.
    pub const fn record_size(&self) -> usize {
        self.length as usize * Self::LENGTH_MULTIPLIER
    }
}

impl ErrorMsg {
    /// Creates a new `ErrorMsg`.
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

    /// Returns `err` as a `str`.
    ///
    /// # Errors
    /// This function returns an error if `err` contains invalid UTF-8.
    pub fn err(&self) -> Result<&str, Utf8Error> {
        // Safety: a pointer to `self.err` will always be valid
        unsafe { CStr::from_ptr(&self.err as *const i8).to_str() }
    }
}

impl SystemMsg {
    const HEARTBEAT: &str = "Heartbeat";

    /// Creates a new `SystemMsg`.
    pub fn new(ts_event: u64, msg: &str) -> Self {
        let mut rec = Self {
            hd: RecordHeader::new::<Self>(rtype::SYSTEM, 0, 0, ts_event),
            msg: [0; 64],
        };
        // leave at least one null byte
        for (i, byte) in msg.as_bytes().iter().take(rec.msg.len() - 1).enumerate() {
            rec.msg[i] = *byte as c_char;
        }
        rec
    }

    /// Creates a new heartbeat `SystemMsg`.
    pub fn heartbeat(ts_event: u64) -> Self {
        let mut rec = Self {
            hd: RecordHeader::new::<Self>(rtype::SYSTEM, 0, 0, ts_event),
            msg: [0; 64],
        };
        for (i, byte) in Self::HEARTBEAT.as_bytes().iter().enumerate() {
            rec.msg[i] = *byte as c_char;
        }
        rec
    }

    /// Checks whether the message is a heartbeat from the gateway.
    pub fn is_heartbeat(&self) -> bool {
        self.msg()
            .map(|msg| msg == Self::HEARTBEAT)
            .unwrap_or_default()
    }

    /// Returns `msg` as a `str`.
    ///
    /// # Errors
    /// This function returns an error if `msg` contains invalid UTF-8.
    pub fn msg(&self) -> Result<&str, Utf8Error> {
        // Safety: a pointer to `self.err` will always be valid
        unsafe { CStr::from_ptr(&self.msg as *const i8).to_str() }
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
        concat!(
            "Passing a slice smaller than `",
            stringify!(T),
            "` to `transmute_record_bytes` is invalid"
        )
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
    if header.length as usize * 4 > bytes.len() {
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
        // Safety: because it comes from a reference, `header` must not be null. It's ok to cast to `mut`
        // because it's never mutated.
        let non_null = NonNull::from(header);
        Some(non_null.cast::<T>().as_ref())
    } else {
        None
    }
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
        // Safety: because it comes from a reference, `header` must not be null. It's ok to cast to `mut`
        // because it's never mutated.
        let non_null = NonNull::from(header);
        Some(non_null.cast::<T>().as_mut())
    } else {
        None
    }
}

impl HasRType for MboMsg {
    fn has_rtype(rtype: u8) -> bool {
        rtype == rtype::MBO
    }

    fn header(&self) -> &RecordHeader {
        &self.hd
    }
}

/// [TradeMsg]'s type ID is the size of its `booklevel` array (0) and is
/// equivalent to MBP-0.
impl HasRType for TradeMsg {
    fn has_rtype(rtype: u8) -> bool {
        rtype == rtype::MBP_0
    }

    fn header(&self) -> &RecordHeader {
        &self.hd
    }
}

/// [Mbp1Msg]'s type ID is the size of its `booklevel` array.
impl HasRType for Mbp1Msg {
    fn has_rtype(rtype: u8) -> bool {
        rtype == rtype::MBP_1
    }

    fn header(&self) -> &RecordHeader {
        &self.hd
    }
}

/// [Mbp10Msg]'s type ID is the size of its `booklevel` array.
impl HasRType for Mbp10Msg {
    fn has_rtype(rtype: u8) -> bool {
        rtype == rtype::MBP_10
    }

    fn header(&self) -> &RecordHeader {
        &self.hd
    }
}

impl HasRType for OhlcvMsg {
    #[allow(deprecated)]
    fn has_rtype(rtype: u8) -> bool {
        matches!(
            rtype,
            rtype::OHLCV_DEPRECATED
                | rtype::OHLCV_1S
                | rtype::OHLCV_1M
                | rtype::OHLCV_1H
                | rtype::OHLCV_1D
        )
    }

    fn header(&self) -> &RecordHeader {
        &self.hd
    }
}

impl HasRType for StatusMsg {
    fn has_rtype(rtype: u8) -> bool {
        rtype == rtype::STATUS
    }

    fn header(&self) -> &RecordHeader {
        &self.hd
    }
}

impl HasRType for InstrumentDefMsg {
    fn has_rtype(rtype: u8) -> bool {
        rtype == rtype::INSTRUMENT_DEF
    }

    fn header(&self) -> &RecordHeader {
        &self.hd
    }
}

impl HasRType for ImbalanceMsg {
    fn has_rtype(rtype: u8) -> bool {
        rtype == rtype::IMBALANCE
    }

    fn header(&self) -> &RecordHeader {
        &self.hd
    }
}

impl HasRType for ErrorMsg {
    fn has_rtype(rtype: u8) -> bool {
        rtype == rtype::ERROR
    }

    fn header(&self) -> &RecordHeader {
        &self.hd
    }
}

impl HasRType for SymbolMappingMsg {
    fn has_rtype(rtype: u8) -> bool {
        rtype == rtype::SYMBOL_MAPPING
    }

    fn header(&self) -> &RecordHeader {
        &self.hd
    }
}

impl HasRType for SystemMsg {
    fn has_rtype(rtype: u8) -> bool {
        rtype == rtype::SYSTEM
    }

    fn header(&self) -> &RecordHeader {
        &self.hd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OHLCV_MSG: OhlcvMsg = OhlcvMsg {
        hd: RecordHeader {
            length: 56,
            rtype: rtype::OHLCV_1S,
            publisher_id: 1,
            product_id: 5482,
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
    fn test_symbol_mapping_size() {
        assert_eq!(mem::size_of::<SymbolMappingMsg>(), 80);
    }

    #[test]
    fn test_serialize_quoted_str_to_json() {
        let error = ErrorMsg::new(0, "\"A test");
        let json = serde_json::to_string(&error).unwrap();
        assert_eq!(
            json,
            r#"{"hd":{"rtype":21,"publisher_id":0,"product_id":0,"ts_event":"0"},"err":"\"A test"}"#
        );
    }
}
