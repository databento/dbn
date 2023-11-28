//! Compatibility shims for different DBN versions.
use std::os::raw::c_char;

use crate::{
    macros::{dbn_record, CsvSerialize, JsonSerialize},
    record::{transmute_header_bytes, transmute_record_bytes},
    rtype, HasRType, InstrumentDefMsg, RecordHeader, RecordRef, SecurityUpdateAction,
    SymbolMappingMsg, UserDefinedInstrument, VersionUpgradePolicy,
};

// Dummy derive macro to get around `cfg_attr` incompatibility of several
// of pyo3's attribute macros. See https://github.com/PyO3/pyo3/issues/780
#[cfg(not(feature = "python"))]
use dbn_macros::MockPyo3;

/// The length of symbol fields in DBN version 1 (current version).
pub const SYMBOL_CSTR_LEN_V1: usize = 22;
/// The length of symbol fields in DBN version 2 (future version).
pub const SYMBOL_CSTR_LEN_V2: usize = 71;
pub(crate) const METADATA_RESERVED_LEN_V1: usize = 47;

/// Returns the length of symbol fields in the given DBN version
pub const fn version_symbol_cstr_len(version: u8) -> usize {
    if version < 2 {
        SYMBOL_CSTR_LEN_V1
    } else {
        SYMBOL_CSTR_LEN_V2
    }
}
pub use crate::record::InstrumentDefMsg as InstrumentDefMsgV2;
pub use crate::record::SymbolMappingMsg as SymbolMappingMsgV2;

/// Decodes bytes into a [`RecordRef`], optionally applying conversion from structs
/// of a prior DBN version to the current DBN version, according to the `version` and
/// `upgrade_policy`.
///
/// # Panics
/// This function will panic if it's passed only a single partial record in `input`.
///
/// # Safety
/// Assumes `input` contains a full record.
pub unsafe fn decode_record_ref<'a>(
    version: u8,
    upgrade_policy: VersionUpgradePolicy,
    compat_buffer: &'a mut [u8; crate::MAX_RECORD_LEN],
    input: &'a [u8],
) -> RecordRef<'a> {
    if version == 1 && upgrade_policy == VersionUpgradePolicy::Upgrade {
        let header = transmute_header_bytes(input).unwrap();
        match header.rtype {
            rtype::INSTRUMENT_DEF => {
                let definition = InstrumentDefMsg::from(
                    transmute_record_bytes::<InstrumentDefMsgV1>(input).unwrap(),
                );
                std::ptr::copy_nonoverlapping(&definition, compat_buffer.as_mut_ptr().cast(), 1);
                return RecordRef::new(compat_buffer);
            }
            rtype::SYMBOL_MAPPING => {
                let definition = SymbolMappingMsg::from(
                    transmute_record_bytes::<SymbolMappingMsgV1>(input).unwrap(),
                );
                std::ptr::copy_nonoverlapping(&definition, compat_buffer.as_mut_ptr().cast(), 1);
                return RecordRef::new(compat_buffer);
            }
            _ => (),
        }
    }
    RecordRef::new(input)
}

/// Definition of an instrument in DBN version 1. The record of the
/// [`Definition`](crate::enums::Schema::Definition) schema.
///
/// Note: This will be renamed to `InstrumentDefMsg` in DBN version 2.
#[repr(C)]
#[derive(Clone, Debug, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::INSTRUMENT_DEF)]
pub struct InstrumentDefMsgV1 {
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
    pub raw_symbol: [c_char; SYMBOL_CSTR_LEN_V1],
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
/// A symbol mapping message in DBN version 1 which maps a symbol of one
/// [`SType`](crate::SType) to another.
///
/// Note: This will be renamed to `SymbolMappingMsg` in DBN version 2.
#[repr(C)]
#[derive(Clone, Debug, CsvSerialize, JsonSerialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(dict, module = "databento_dbn"),
    derive(crate::macros::PyFieldDesc)
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
#[cfg_attr(test, derive(type_layout::TypeLayout))]
#[dbn_record(rtype::SYMBOL_MAPPING)]
pub struct SymbolMappingMsgV1 {
    /// The common header.
    #[pyo3(get, set)]
    pub hd: RecordHeader,
    /// The input symbol.
    pub stype_in_symbol: [c_char; SYMBOL_CSTR_LEN_V1],
    /// The output symbol.
    pub stype_out_symbol: [c_char; SYMBOL_CSTR_LEN_V1],
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

impl From<&InstrumentDefMsgV1> for InstrumentDefMsg {
    fn from(old: &InstrumentDefMsgV1) -> Self {
        let mut res = Self {
            // recalculate length
            hd: RecordHeader::new::<Self>(
                rtype::INSTRUMENT_DEF,
                old.hd.publisher_id,
                old.hd.instrument_id,
                old.hd.ts_event,
            ),
            ts_recv: old.ts_recv,
            min_price_increment: old.min_price_increment,
            display_factor: old.display_factor,
            expiration: old.expiration,
            activation: old.activation,
            high_limit_price: old.high_limit_price,
            low_limit_price: old.low_limit_price,
            max_price_variation: old.max_price_variation,
            trading_reference_price: old.trading_reference_price,
            unit_of_measure_qty: old.unit_of_measure_qty,
            min_price_increment_amount: old.min_price_increment_amount,
            price_ratio: old.price_ratio,
            inst_attrib_value: old.inst_attrib_value,
            underlying_id: old.underlying_id,
            raw_instrument_id: old.raw_instrument_id,
            market_depth_implied: old.market_depth_implied,
            market_depth: old.market_depth,
            market_segment_id: old.market_segment_id,
            max_trade_vol: old.max_trade_vol,
            min_lot_size: old.min_lot_size,
            min_lot_size_block: old.min_lot_size_block,
            min_lot_size_round_lot: old.min_lot_size_round_lot,
            min_trade_vol: old.min_trade_vol,
            contract_multiplier: old.contract_multiplier,
            decay_quantity: old.decay_quantity,
            original_contract_size: old.original_contract_size,
            trading_reference_date: old.trading_reference_date,
            appl_id: old.appl_id,
            maturity_year: old.maturity_year,
            decay_start_date: old.decay_start_date,
            channel_id: old.channel_id,
            currency: old.currency,
            settl_currency: old.settl_currency,
            secsubtype: old.secsubtype,
            raw_symbol: [0; SYMBOL_CSTR_LEN_V2],
            group: old.group,
            exchange: old.exchange,
            asset: old.asset,
            cfi: old.cfi,
            security_type: old.security_type,
            unit_of_measure: old.unit_of_measure,
            underlying: old.underlying,
            strike_price_currency: old.strike_price_currency,
            instrument_class: old.instrument_class,
            strike_price: old.strike_price,
            match_algorithm: old.match_algorithm,
            md_security_trading_status: old.md_security_trading_status,
            main_fraction: old.main_fraction,
            price_display_format: old.price_display_format,
            settl_price_type: old.settl_price_type,
            sub_fraction: old.sub_fraction,
            underlying_product: old.underlying_product,
            security_update_action: old.security_update_action as c_char,
            maturity_month: old.maturity_month,
            maturity_day: old.maturity_day,
            maturity_week: old.maturity_week,
            user_defined_instrument: old.user_defined_instrument,
            contract_multiplier_unit: old.contract_multiplier_unit,
            flow_schedule_type: old.flow_schedule_type,
            tick_rule: old.tick_rule,
            _reserved: Default::default(),
        };
        // Safety: SYMBOL_CSTR_LEN_V1 is less than SYMBOL_CSTR_LEN
        unsafe {
            std::ptr::copy_nonoverlapping(
                old.raw_symbol.as_ptr(),
                res.raw_symbol.as_mut_ptr(),
                SYMBOL_CSTR_LEN_V1,
            );
        }
        res
    }
}

impl From<&SymbolMappingMsgV1> for SymbolMappingMsg {
    fn from(old: &SymbolMappingMsgV1) -> Self {
        let mut res = Self {
            hd: RecordHeader::new::<Self>(
                rtype::SYMBOL_MAPPING,
                old.hd.publisher_id,
                old.hd.instrument_id,
                old.hd.ts_event,
            ),
            stype_in_symbol: [0; SYMBOL_CSTR_LEN_V2],
            stype_out_symbol: [0; SYMBOL_CSTR_LEN_V2],
            start_ts: old.start_ts,
            end_ts: old.end_ts,
            // Invalid
            stype_in: u8::MAX,
            stype_out: u8::MAX,
        };
        // Safety: SYMBOL_CSTR_LEN_V1 is less than SYMBOL_CSTR_LEN
        unsafe {
            std::ptr::copy_nonoverlapping(
                old.stype_in_symbol.as_ptr(),
                res.stype_in_symbol.as_mut_ptr(),
                SYMBOL_CSTR_LEN_V1,
            );
            std::ptr::copy_nonoverlapping(
                old.stype_out_symbol.as_ptr(),
                res.stype_out_symbol.as_mut_ptr(),
                SYMBOL_CSTR_LEN_V1,
            );
        }
        res
    }
}

/// A trait for symbol mapping records.
pub trait SymbolMappingRec: HasRType {
    /// Returns the input symbol as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `stype_in_symbol` contains invalid UTF-8.
    fn stype_in_symbol(&self) -> crate::Result<&str>;

    /// Returns the output symbol as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `stype_out_symbol` contains invalid UTF-8.
    fn stype_out_symbol(&self) -> crate::Result<&str>;

    /// Parses the raw start of the mapping interval into a datetime. Returns `None` if
    /// `start_ts` contains the sentinel for a null timestamp.
    fn start_ts(&self) -> Option<time::OffsetDateTime>;

    /// Parses the raw end of the mapping interval into a datetime. Returns `None` if
    /// `end_ts` contains the sentinel for a null timestamp.
    fn end_ts(&self) -> Option<time::OffsetDateTime>;
}

impl SymbolMappingRec for SymbolMappingMsgV1 {
    fn stype_in_symbol(&self) -> crate::Result<&str> {
        Self::stype_in_symbol(self)
    }

    fn stype_out_symbol(&self) -> crate::Result<&str> {
        Self::stype_out_symbol(self)
    }

    fn start_ts(&self) -> Option<time::OffsetDateTime> {
        Self::start_ts(self)
    }

    fn end_ts(&self) -> Option<time::OffsetDateTime> {
        Self::end_ts(self)
    }
}

impl SymbolMappingRec for SymbolMappingMsgV2 {
    fn stype_in_symbol(&self) -> crate::Result<&str> {
        Self::stype_in_symbol(self)
    }

    fn stype_out_symbol(&self) -> crate::Result<&str> {
        Self::stype_out_symbol(self)
    }

    fn start_ts(&self) -> Option<time::OffsetDateTime> {
        Self::start_ts(self)
    }

    fn end_ts(&self) -> Option<time::OffsetDateTime> {
        Self::end_ts(self)
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use type_layout::{Field, TypeLayout};

    use super::*;

    #[cfg(feature = "python")]
    #[test]
    fn test_strike_price_order_didnt_change() {
        use crate::python::PyFieldDesc;

        assert_eq!(
            InstrumentDefMsg::ordered_fields(""),
            InstrumentDefMsgV2::ordered_fields("")
        );
    }

    #[test]
    fn test_default_equivalency() {
        assert_eq!(
            InstrumentDefMsgV2::from(&InstrumentDefMsgV1::default()),
            InstrumentDefMsgV2::default()
        );
    }

    #[test]
    fn test_definition_size_alignment_and_padding() {
        assert_eq!(mem::size_of::<InstrumentDefMsgV1>(), 360);
        let layout = InstrumentDefMsgV1::type_layout();
        assert_eq!(layout.alignment, 8);
        for field in layout.fields.iter() {
            assert!(
                matches!(field, Field::Field { .. }),
                "Detected padding: {layout}"
            );
        }
    }

    #[test]
    fn test_symbol_mapping_size_alignment_and_padding() {
        assert_eq!(mem::size_of::<SymbolMappingMsgV1>(), 80);
        let layout = SymbolMappingMsgV1::type_layout();
        assert_eq!(layout.alignment, 8);
        for field in layout.fields.iter() {
            assert!(
                matches!(field, Field::Field { .. }),
                "Detected padding: {layout}"
            );
        }
    }
}
