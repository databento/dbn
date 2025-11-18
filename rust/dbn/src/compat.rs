//! Compatibility shims for different DBN versions.

mod traits;

/// The length of symbol fields in DBN version 1 (prior version being phased out).
pub const SYMBOL_CSTR_LEN_V1: usize = 22;
/// The length of symbol fields in DBN version 2 (prior version being phased out).
pub const SYMBOL_CSTR_LEN_V2: usize = 71;
/// The length of symbol fields in DBN version 3 (current version).
pub const SYMBOL_CSTR_LEN_V3: usize = SYMBOL_CSTR_LEN_V2;

/// The length of the fixed-length asset field in instrument definitions in DBN version
/// 1.
pub const ASSET_CSTR_LEN_V1: usize = 7;
/// The length of the fixed-length asset field in instrument definitions in DBN version
/// 2.
pub const ASSET_CSTR_LEN_V2: usize = ASSET_CSTR_LEN_V1;
/// The length of the fixed-length asset field in instrument definitions in DBN version
/// 3 (current version).
pub const ASSET_CSTR_LEN_V3: usize = 11;

/// The sentinel value for an unset or null stat quantity in DBN version 1.
pub const UNDEF_STAT_QUANTITY_V1: i32 = i32::MAX;
/// The sentinel value for an unset or null stat quantity in DBN version 2.
pub const UNDEF_STAT_QUANTITY_V2: i32 = UNDEF_STAT_QUANTITY_V1;
/// The sentinel value for an unset or null stat quantity in DBN version 3 (current
/// version).
pub const UNDEF_STAT_QUANTITY_V3: i64 = i64::MAX;

pub(crate) const METADATA_RESERVED_LEN_V1: usize = 47;

/// Returns the length of symbol fields in the given DBN version
pub const fn version_symbol_cstr_len(version: u8) -> usize {
    if version < 2 {
        SYMBOL_CSTR_LEN_V1
    } else {
        SYMBOL_CSTR_LEN_V2
    }
}
pub use crate::record::ErrorMsg as ErrorMsgV2;
pub use crate::record::InstrumentDefMsg as InstrumentDefMsgV3;
pub use crate::record::StatMsg as StatMsgV3;
pub use crate::record::SymbolMappingMsg as SymbolMappingMsgV2;
pub use crate::record::SystemMsg as SystemMsgV2;
pub use traits::{InstrumentDefRec, StatRec, SymbolMappingRec};

use std::os::raw::c_char;

// Dummy derive macro to get around `cfg_attr` incompatibility of several
// of pyo3's attribute macros. See https://github.com/PyO3/pyo3/issues/780
#[cfg(not(feature = "python"))]
use dbn_macros::MockPyo3;

use crate::{
    macros::{dbn_record, CsvSerialize, JsonSerialize},
    rtype, RecordHeader, SecurityUpdateAction, UserDefinedInstrument,
};

// NOTE: Versioned records need to be defined in this file to work with cbindgen.

/// An error message from the Databento Live Subscription Gateway (LSG) in DBN version 1.
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
pub struct ErrorMsgV1 {
    /// The common header.
    pub hd: RecordHeader,
    /// The error message.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub err: [c_char; 64],
}

/// A definition of an instrument in DBN version 1. The record of the
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
pub struct InstrumentDefMsgV1 {
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
    /// Will be [`UNDEF_TIMESTAMP`](crate::UNDEF_TIMESTAMP) when null, such as for equities. Some publishers
    /// only provide date-level granularity.
    #[dbn(unix_nanos)]
    #[pyo3(get, set)]
    pub expiration: u64,
    /// The time of instrument activation expressed as the number of nanoseconds since the
    /// UNIX epoch.
    ///
    /// Will be [`UNDEF_TIMESTAMP`](crate::UNDEF_TIMESTAMP) when null, such as for equities. Some publishers
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
    /// The trading session settlement price on `trading_reference_date`.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub trading_reference_price: i64,
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
    /// A bitmap of instrument eligibility attributes.
    #[dbn(fmt_binary)]
    #[pyo3(get, set)]
    pub inst_attrib_value: i32,
    /// The `instrument_id` of the first underlying instrument.
    ///
    /// See [Instrument identifiers](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers).
    #[pyo3(get, set)]
    pub underlying_id: u32,
    /// The instrument ID assigned by the publisher. May be the same as `instrument_id`.
    ///
    /// See [Instrument identifiers](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers)
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
    /// The minimum quantity required for a round lot of the instrument. Multiples of this
    /// quantity are also round lots.
    #[pyo3(get, set)]
    pub min_lot_size_round_lot: i32,
    /// The minimum trading volume for the instrument.
    #[pyo3(get, set)]
    pub min_trade_vol: u32,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved2: [u8; 4],
    /// The number of deliverables per instrument, i.e. peak days.
    #[pyo3(get, set)]
    pub contract_multiplier: i32,
    /// The quantity that a contract will decay daily, after `decay_start_date` has been reached.
    #[pyo3(get, set)]
    pub decay_quantity: i32,
    /// The fixed contract value assigned to each instrument.
    #[pyo3(get, set)]
    pub original_contract_size: i32,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
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
    pub raw_symbol: [c_char; SYMBOL_CSTR_LEN_V1],
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
    pub asset: [c_char; ASSET_CSTR_LEN_V1],
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
    /// The classification of the instrument.
    ///
    /// See [Instrument class](https://databento.com/docs/schemas-and-data-formats/instrument-definitions#instrument-class).
    #[dbn(c_char, encode_order(4))]
    pub instrument_class: c_char,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved4: [u8; 2],
    /// The strike price of the option where every 1 unit corresponds to 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub strike_price: i64,
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved5: [u8; 6],
    /// The matching algorithm used for the instrument, typically **F**IFO.
    ///
    /// See [Matching algorithm](https://databento.com/docs/schemas-and-data-formats/instrument-definitions#matching-algorithm).
    #[dbn(c_char)]
    pub match_algorithm: c_char,
    /// The current trading state of the instrument.
    #[pyo3(get, set)]
    pub md_security_trading_status: u8,
    /// The price denominator of the main fraction.
    #[pyo3(get, set)]
    pub main_fraction: u8,
    /// The number of digits to the right of the tick mark, to display fractional prices.
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
    #[pyo3(get, set)]
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
    #[pyo3(get, set)]
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
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _dummy: [u8; 3],
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
pub struct StatMsgV1 {
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
    /// nanoseconds since the UNIX epoch. Will be [`UNDEF_TIMESTAMP`](crate::UNDEF_TIMESTAMP) when
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
    /// The value for non-price statistics. Will be [`UNDEF_STAT_QUANTITY`](crate::UNDEF_STAT_QUANTITY)
    /// when unused.
    #[pyo3(get, set)]
    pub quantity: i32,
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
    pub _reserved: [u8; 6],
}

/// A symbol mapping message from the live API in DBN version 1.
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
pub struct SymbolMappingMsgV1 {
    /// The common header.
    pub hd: RecordHeader,
    /// The input symbol.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub stype_in_symbol: [c_char; SYMBOL_CSTR_LEN_V1],
    /// The output symbol.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub stype_out_symbol: [c_char; SYMBOL_CSTR_LEN_V1],
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
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

/// A non-error message from the Databento Live Subscription Gateway (LSG) in DBN version 1.
/// Also used for heartbeating.
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
pub struct SystemMsgV1 {
    /// The common header.
    pub hd: RecordHeader,
    /// The message from the Databento gateway.
    #[dbn(fmt_method)]
    #[cfg_attr(feature = "serde", serde(with = "crate::record::cstr_serde"))]
    pub msg: [c_char; 64],
}

/// A definition of an instrument in DBN version 2. The record of the
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
pub struct InstrumentDefMsgV2 {
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
    /// Will be [`UNDEF_TIMESTAMP`](crate::UNDEF_TIMESTAMP) when null, such as for equities. Some publishers
    /// only provide date-level granularity.
    #[dbn(unix_nanos)]
    #[pyo3(get, set)]
    pub expiration: u64,
    /// The time of instrument activation expressed as the number of nanoseconds since the
    /// UNIX epoch.
    ///
    /// Will be [`UNDEF_TIMESTAMP`](crate::UNDEF_TIMESTAMP) when null, such as for equities. Some publishers
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
    /// The trading session settlement price on `trading_reference_date`.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    #[dbn(fixed_price)]
    #[pyo3(get, set)]
    pub trading_reference_price: i64,
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
    #[dbn(encode_order(46), fixed_price)]
    #[pyo3(get, set)]
    pub strike_price: i64,
    /// A bitmap of instrument eligibility attributes.
    #[dbn(fmt_binary)]
    #[pyo3(get, set)]
    pub inst_attrib_value: i32,
    /// The `instrument_id` of the first underlying instrument.
    ///
    /// See [Instrument identifiers](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers).
    #[pyo3(get, set)]
    pub underlying_id: u32,
    /// The instrument ID assigned by the publisher. May be the same as `instrument_id`.
    ///
    /// See [Instrument identifiers](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers)
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
    pub raw_symbol: [c_char; SYMBOL_CSTR_LEN_V2],
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
    pub asset: [c_char; ASSET_CSTR_LEN_V2],
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
    /// The current trading state of the instrument.
    #[pyo3(get, set)]
    pub md_security_trading_status: u8,
    /// The price denominator of the main fraction.
    #[pyo3(get, set)]
    pub main_fraction: u8,
    /// The number of digits to the right of the tick mark, to display fractional prices.
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
    #[pyo3(get, set)]
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
    #[doc(hidden)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub _reserved: [u8; 10],
}

#[cfg(all(test, feature = "python"))]
mod tests {
    use super::*;

    #[test]
    fn test_strike_price_order_didnt_change() {
        use crate::python::PyFieldDesc;

        assert_eq!(
            InstrumentDefMsgV1::ordered_fields(""),
            InstrumentDefMsgV2::ordered_fields("")
        );
    }
}
