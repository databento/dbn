//! Offsets of the fields DBN version 4 record structs don't hold, resolved once per
//! stream.

use std::os::raw::c_char;

use crate::{
    layout::{DynFieldAccess, FieldOffset, LayoutIndex},
    Error, ImbalanceRec, InstrumentClass, InstrumentDefRec, Record, Result, Side, SymbolMappingRec,
    UserDefinedInstrument,
};

use super::{
    fields, rtype,
    types::{CStr, Decimal, OpenEnum, TimestampNs},
    *,
};

/// The offsets of the [`InstrumentDefMsg`] fields its compiled struct doesn't hold,
/// resolved from a stream's [`LayoutIndex`].
#[derive(Debug, Clone, Copy)]
pub struct InstrumentDefOffsets {
    raw_symbol: FieldOffset<CStr>,
    expiration: Option<FieldOffset<TimestampNs>>,
    activation: Option<FieldOffset<TimestampNs>>,
    high_limit_price: Option<FieldOffset<Decimal>>,
    low_limit_price: Option<FieldOffset<Decimal>>,
    max_price_variation: Option<FieldOffset<Decimal>>,
    unit_of_measure_qty: Option<FieldOffset<Decimal>>,
    min_price_increment_amount: Option<FieldOffset<Decimal>>,
    price_ratio: Option<FieldOffset<Decimal>>,
    strike_price: Option<FieldOffset<Decimal>>,
    leg_price: Option<FieldOffset<Decimal>>,
    leg_delta: Option<FieldOffset<Decimal>>,
    user_defined_instrument: Option<FieldOffset<OpenEnum<UserDefinedInstrument>>>,
    leg_instrument_class: Option<FieldOffset<OpenEnum<InstrumentClass>>>,
    leg_side: Option<FieldOffset<OpenEnum<Side>>>,
}

impl InstrumentDefOffsets {
    /// Resolves the offsets from the stream's `index`.
    ///
    /// # Errors
    /// This function returns an error if the stream declares no layout for
    /// [`InstrumentDefMsg`] or its layout lacks a required field.
    pub fn resolve(index: &LayoutIndex) -> Result<Self> {
        if index.verdict(rtype::INSTRUMENT_DEF).is_none() {
            return Err(Error::layout(
                "the stream declares no `InstrumentDefMsg` layout",
            ));
        }
        Ok(Self {
            raw_symbol: index
                .offset_of(rtype::INSTRUMENT_DEF, fields::RAW_SYMBOL)
                .ok_or_else(|| Error::layout("`InstrumentDefMsg` layout lacks `raw_symbol`"))?,
            expiration: index.offset_of(rtype::INSTRUMENT_DEF, fields::EXPIRATION),
            activation: index.offset_of(rtype::INSTRUMENT_DEF, fields::ACTIVATION),
            high_limit_price: index.offset_of(rtype::INSTRUMENT_DEF, fields::HIGH_LIMIT_PRICE),
            low_limit_price: index.offset_of(rtype::INSTRUMENT_DEF, fields::LOW_LIMIT_PRICE),
            max_price_variation: index
                .offset_of(rtype::INSTRUMENT_DEF, fields::MAX_PRICE_VARIATION),
            unit_of_measure_qty: index
                .offset_of(rtype::INSTRUMENT_DEF, fields::UNIT_OF_MEASURE_QTY),
            min_price_increment_amount: index
                .offset_of(rtype::INSTRUMENT_DEF, fields::MIN_PRICE_INCREMENT_AMOUNT),
            price_ratio: index.offset_of(rtype::INSTRUMENT_DEF, fields::PRICE_RATIO),
            strike_price: index.offset_of(rtype::INSTRUMENT_DEF, fields::STRIKE_PRICE),
            leg_price: index.offset_of(rtype::INSTRUMENT_DEF, fields::LEG_PRICE),
            leg_delta: index.offset_of(rtype::INSTRUMENT_DEF, fields::LEG_DELTA),
            user_defined_instrument: index
                .offset_of(rtype::INSTRUMENT_DEF, fields::USER_DEFINED_INSTRUMENT),
            leg_instrument_class: index
                .offset_of(rtype::INSTRUMENT_DEF, fields::LEG_INSTRUMENT_CLASS),
            leg_side: index.offset_of(rtype::INSTRUMENT_DEF, fields::LEG_SIDE),
        })
    }
}

/// A [`InstrumentDefMsg`] bound to the [`InstrumentDefOffsets`] of its stream.
#[derive(Debug, Clone, Copy)]
pub struct ResolvedInstrumentDefRef<'a> {
    rec: &'a InstrumentDefMsg,
    offsets: &'a InstrumentDefOffsets,
}

impl<'a> ResolvedInstrumentDefRef<'a> {
    /// Binds `rec` to `offsets` for reading its fields through
    /// [`InstrumentDefRec`](crate::InstrumentDefRec).
    pub fn resolve(rec: &'a InstrumentDefMsg, offsets: &'a InstrumentDefOffsets) -> Self {
        Self { rec, offsets }
    }

    /// Returns the underlying record.
    pub fn record(&self) -> &'a InstrumentDefMsg {
        self.rec
    }
}

impl AsRef<[u8]> for ResolvedInstrumentDefRef<'_> {
    fn as_ref(&self) -> &[u8] {
        self.rec.as_ref()
    }
}

impl Record for ResolvedInstrumentDefRef<'_> {
    type Header = RecordHeader;

    fn record_size(&self) -> usize {
        self.rec.record_size()
    }

    fn rtype(&self) -> Result<RType> {
        self.rec.rtype()
    }

    fn raw_rtype(&self) -> u16 {
        self.rec.raw_rtype()
    }

    fn publisher_id(&self) -> u16 {
        self.rec.publisher_id()
    }

    fn publisher(&self) -> Result<Publisher> {
        self.rec.publisher()
    }

    fn instrument_id(&self) -> u64 {
        self.rec.instrument_id()
    }

    fn raw_ts_event(&self) -> u64 {
        self.rec.raw_ts_event()
    }

    fn raw_index_ts(&self) -> u64 {
        self.rec.raw_index_ts()
    }
}

impl InstrumentDefRec for ResolvedInstrumentDefRef<'_> {
    fn raw_ts_recv(&self) -> u64 {
        self.rec.ts_recv
    }

    fn min_price_increment(&self) -> i64 {
        self.rec.min_price_increment
    }

    fn display_factor(&self) -> i64 {
        self.rec.display_factor
    }

    fn raw_instrument_id(&self) -> u64 {
        self.rec.raw_instrument_id
    }

    fn min_lot_size_round_lot(&self) -> i32 {
        self.rec.min_lot_size_round_lot
    }

    fn raw_security_update_action(&self) -> c_char {
        self.rec.security_update_action
    }

    fn raw_instrument_class(&self) -> c_char {
        self.rec.instrument_class
    }

    fn raw_match_algorithm(&self) -> c_char {
        self.rec.match_algorithm
    }

    fn channel_id(&self) -> u8 {
        self.rec.channel_id
    }

    fn raw_symbol(&self) -> Result<&str> {
        self.rec
            .field_at(self.offsets.raw_symbol)
            .ok_or_else(|| Error::conversion::<&str>("`raw_symbol`"))
    }

    fn raw_expiration(&self) -> Option<u64> {
        self.offsets
            .expiration
            .and_then(|offset| self.rec.field_at(offset))
            .map(|ts| ts.0)
    }

    fn raw_activation(&self) -> Option<u64> {
        self.offsets
            .activation
            .and_then(|offset| self.rec.field_at(offset))
            .map(|ts| ts.0)
    }

    fn high_limit_price(&self) -> Option<i64> {
        self.offsets
            .high_limit_price
            .and_then(|offset| self.rec.field_at(offset))
            .map(|px| px.raw)
    }

    fn low_limit_price(&self) -> Option<i64> {
        self.offsets
            .low_limit_price
            .and_then(|offset| self.rec.field_at(offset))
            .map(|px| px.raw)
    }

    fn max_price_variation(&self) -> Option<i64> {
        self.offsets
            .max_price_variation
            .and_then(|offset| self.rec.field_at(offset))
            .map(|px| px.raw)
    }

    fn unit_of_measure_qty(&self) -> Option<i64> {
        self.offsets
            .unit_of_measure_qty
            .and_then(|offset| self.rec.field_at(offset))
            .map(|px| px.raw)
    }

    fn min_price_increment_amount(&self) -> Option<i64> {
        self.offsets
            .min_price_increment_amount
            .and_then(|offset| self.rec.field_at(offset))
            .map(|px| px.raw)
    }

    fn price_ratio(&self) -> Option<i64> {
        self.offsets
            .price_ratio
            .and_then(|offset| self.rec.field_at(offset))
            .map(|px| px.raw)
    }

    fn strike_price(&self) -> Option<i64> {
        self.offsets
            .strike_price
            .and_then(|offset| self.rec.field_at(offset))
            .map(|px| px.raw)
    }

    fn leg_price(&self) -> Option<i64> {
        self.offsets
            .leg_price
            .and_then(|offset| self.rec.field_at(offset))
            .map(|px| px.raw)
    }

    fn leg_delta(&self) -> Option<i64> {
        self.offsets
            .leg_delta
            .and_then(|offset| self.rec.field_at(offset))
            .map(|px| px.raw)
    }

    fn user_defined_instrument(&self) -> Result<Option<UserDefinedInstrument>> {
        self.offsets
            .user_defined_instrument
            .and_then(|offset| self.rec.field_at(offset))
            .map(|value| value.get())
            .transpose()
    }

    fn leg_instrument_class(&self) -> Result<Option<InstrumentClass>> {
        self.offsets
            .leg_instrument_class
            .and_then(|offset| self.rec.field_at(offset))
            .map(|value| value.get())
            .transpose()
    }

    fn leg_side(&self) -> Result<Option<Side>> {
        self.offsets
            .leg_side
            .and_then(|offset| self.rec.field_at(offset))
            .map(|value| value.get())
            .transpose()
    }
}

/// The offsets of the [`ImbalanceMsg`] fields its compiled struct doesn't hold,
/// resolved from a stream's [`LayoutIndex`].
#[derive(Debug, Clone, Copy)]
pub struct ImbalanceOffsets {
    auction_time: Option<FieldOffset<TimestampNs>>,
    ssr_filling_price: Option<FieldOffset<Decimal>>,
    ind_match_price: Option<FieldOffset<Decimal>>,
    upper_collar: Option<FieldOffset<Decimal>>,
    lower_collar: Option<FieldOffset<Decimal>>,
    unpaired_side: Option<FieldOffset<OpenEnum<Side>>>,
}

impl ImbalanceOffsets {
    /// Resolves the offsets from the stream's `index`.
    ///
    /// # Errors
    /// This function returns an error if the stream declares no layout for
    /// [`ImbalanceMsg`].
    pub fn resolve(index: &LayoutIndex) -> Result<Self> {
        if index.verdict(rtype::IMBALANCE).is_none() {
            return Err(Error::layout(
                "the stream declares no `ImbalanceMsg` layout",
            ));
        }
        Ok(Self {
            auction_time: index.offset_of(rtype::IMBALANCE, fields::AUCTION_TIME),
            ssr_filling_price: index.offset_of(rtype::IMBALANCE, fields::SSR_FILLING_PRICE),
            ind_match_price: index.offset_of(rtype::IMBALANCE, fields::IND_MATCH_PRICE),
            upper_collar: index.offset_of(rtype::IMBALANCE, fields::UPPER_COLLAR),
            lower_collar: index.offset_of(rtype::IMBALANCE, fields::LOWER_COLLAR),
            unpaired_side: index.offset_of(rtype::IMBALANCE, fields::UNPAIRED_SIDE),
        })
    }
}

/// A [`ImbalanceMsg`] bound to the [`ImbalanceOffsets`] of its stream.
#[derive(Debug, Clone, Copy)]
pub struct ResolvedImbalanceRef<'a> {
    rec: &'a ImbalanceMsg,
    offsets: &'a ImbalanceOffsets,
}

impl<'a> ResolvedImbalanceRef<'a> {
    /// Binds `rec` to `offsets` for reading its fields through
    /// [`ImbalanceRec`](crate::ImbalanceRec).
    pub fn resolve(rec: &'a ImbalanceMsg, offsets: &'a ImbalanceOffsets) -> Self {
        Self { rec, offsets }
    }

    /// Returns the underlying record.
    pub fn record(&self) -> &'a ImbalanceMsg {
        self.rec
    }
}

impl AsRef<[u8]> for ResolvedImbalanceRef<'_> {
    fn as_ref(&self) -> &[u8] {
        self.rec.as_ref()
    }
}

impl Record for ResolvedImbalanceRef<'_> {
    type Header = RecordHeader;

    fn record_size(&self) -> usize {
        self.rec.record_size()
    }

    fn rtype(&self) -> Result<RType> {
        self.rec.rtype()
    }

    fn raw_rtype(&self) -> u16 {
        self.rec.raw_rtype()
    }

    fn publisher_id(&self) -> u16 {
        self.rec.publisher_id()
    }

    fn publisher(&self) -> Result<Publisher> {
        self.rec.publisher()
    }

    fn instrument_id(&self) -> u64 {
        self.rec.instrument_id()
    }

    fn raw_ts_event(&self) -> u64 {
        self.rec.raw_ts_event()
    }

    fn raw_index_ts(&self) -> u64 {
        self.rec.raw_index_ts()
    }
}

impl ImbalanceRec for ResolvedImbalanceRef<'_> {
    fn raw_ts_recv(&self) -> u64 {
        self.rec.ts_recv
    }

    fn ref_price(&self) -> i64 {
        self.rec.ref_price
    }

    fn cont_book_clr_price(&self) -> i64 {
        self.rec.cont_book_clr_price
    }

    fn auct_interest_clr_price(&self) -> i64 {
        self.rec.auct_interest_clr_price
    }

    fn paired_qty(&self) -> u32 {
        self.rec.paired_qty
    }

    fn total_imbalance_qty(&self) -> u32 {
        self.rec.total_imbalance_qty
    }

    fn auction_type(&self) -> c_char {
        self.rec.auction_type
    }

    fn raw_side(&self) -> c_char {
        self.rec.side
    }

    fn raw_auction_time(&self) -> Option<u64> {
        self.offsets
            .auction_time
            .and_then(|offset| self.rec.field_at(offset))
            .map(|ts| ts.0)
    }

    fn ssr_filling_price(&self) -> Option<i64> {
        self.offsets
            .ssr_filling_price
            .and_then(|offset| self.rec.field_at(offset))
            .map(|px| px.raw)
    }

    fn ind_match_price(&self) -> Option<i64> {
        self.offsets
            .ind_match_price
            .and_then(|offset| self.rec.field_at(offset))
            .map(|px| px.raw)
    }

    fn upper_collar(&self) -> Option<i64> {
        self.offsets
            .upper_collar
            .and_then(|offset| self.rec.field_at(offset))
            .map(|px| px.raw)
    }

    fn lower_collar(&self) -> Option<i64> {
        self.offsets
            .lower_collar
            .and_then(|offset| self.rec.field_at(offset))
            .map(|px| px.raw)
    }

    fn unpaired_side(&self) -> Result<Option<Side>> {
        self.offsets
            .unpaired_side
            .and_then(|offset| self.rec.field_at(offset))
            .map(|value| value.get())
            .transpose()
    }
}

/// The offsets of the [`SymbolMappingMsg`] fields its compiled struct doesn't hold,
/// resolved from a stream's [`LayoutIndex`].
#[derive(Debug, Clone, Copy)]
pub struct SymbolMappingOffsets {
    stype_in_symbol: FieldOffset<CStr>,
    stype_out_symbol: FieldOffset<CStr>,
}

impl SymbolMappingOffsets {
    /// Resolves the offsets from the stream's `index`.
    ///
    /// # Errors
    /// This function returns an error if the stream declares no layout for
    /// [`SymbolMappingMsg`] or its layout lacks a required field.
    pub fn resolve(index: &LayoutIndex) -> Result<Self> {
        if index.verdict(rtype::SYMBOL_MAPPING).is_none() {
            return Err(Error::layout(
                "the stream declares no `SymbolMappingMsg` layout",
            ));
        }
        Ok(Self {
            stype_in_symbol: index
                .offset_of(rtype::SYMBOL_MAPPING, fields::STYPE_IN_SYMBOL)
                .ok_or_else(|| {
                    Error::layout("`SymbolMappingMsg` layout lacks `stype_in_symbol`")
                })?,
            stype_out_symbol: index
                .offset_of(rtype::SYMBOL_MAPPING, fields::STYPE_OUT_SYMBOL)
                .ok_or_else(|| {
                    Error::layout("`SymbolMappingMsg` layout lacks `stype_out_symbol`")
                })?,
        })
    }
}

/// A [`SymbolMappingMsg`] bound to the [`SymbolMappingOffsets`] of its stream.
#[derive(Debug, Clone, Copy)]
pub struct ResolvedSymbolMappingRef<'a> {
    rec: &'a SymbolMappingMsg,
    offsets: &'a SymbolMappingOffsets,
}

impl<'a> ResolvedSymbolMappingRef<'a> {
    /// Binds `rec` to `offsets` for reading its fields through
    /// [`SymbolMappingRec`](crate::SymbolMappingRec).
    pub fn resolve(rec: &'a SymbolMappingMsg, offsets: &'a SymbolMappingOffsets) -> Self {
        Self { rec, offsets }
    }

    /// Returns the underlying record.
    pub fn record(&self) -> &'a SymbolMappingMsg {
        self.rec
    }
}

impl AsRef<[u8]> for ResolvedSymbolMappingRef<'_> {
    fn as_ref(&self) -> &[u8] {
        self.rec.as_ref()
    }
}

impl Record for ResolvedSymbolMappingRef<'_> {
    type Header = RecordHeader;

    fn record_size(&self) -> usize {
        self.rec.record_size()
    }

    fn rtype(&self) -> Result<RType> {
        self.rec.rtype()
    }

    fn raw_rtype(&self) -> u16 {
        self.rec.raw_rtype()
    }

    fn publisher_id(&self) -> u16 {
        self.rec.publisher_id()
    }

    fn publisher(&self) -> Result<Publisher> {
        self.rec.publisher()
    }

    fn instrument_id(&self) -> u64 {
        self.rec.instrument_id()
    }

    fn raw_ts_event(&self) -> u64 {
        self.rec.raw_ts_event()
    }

    fn raw_index_ts(&self) -> u64 {
        self.rec.raw_index_ts()
    }
}

impl SymbolMappingRec for ResolvedSymbolMappingRef<'_> {
    fn raw_start_ts(&self) -> u64 {
        self.rec.start_ts
    }

    fn raw_end_ts(&self) -> u64 {
        self.rec.end_ts
    }

    fn raw_stype_in(&self) -> u8 {
        self.rec.stype_in
    }

    fn raw_stype_out(&self) -> u8 {
        self.rec.stype_out
    }

    fn stype_in_symbol(&self) -> Result<&str> {
        self.rec
            .field_at(self.offsets.stype_in_symbol)
            .ok_or_else(|| Error::conversion::<&str>("`stype_in_symbol`"))
    }

    fn stype_out_symbol(&self) -> Result<&str> {
        self.rec
            .field_at(self.offsets.stype_out_symbol)
            .ok_or_else(|| Error::conversion::<&str>("`stype_out_symbol`"))
    }
}
