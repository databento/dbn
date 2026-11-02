//! Offsets of the fields DBN version 4 record structs don't hold, resolved once per
//! stream.

use crate::{
    layout::{DynFieldAccess, FieldOffset, LayoutIndex},
    pretty::px_to_f64,
    record::ts_to_dt,
    Error,
};

use super::{fields, rtype, *};

/// The offsets of the [`InstrumentDefMsg`] fields its compiled struct doesn't hold,
/// resolved from a stream's [`LayoutIndex`].
#[derive(Debug, Clone, Copy)]
pub struct InstrumentDefOffsets {
    raw_symbol: FieldOffset<crate::v4::types::CStr>,
    expiration: Option<FieldOffset<crate::v4::types::TimestampNs>>,
    activation: Option<FieldOffset<crate::v4::types::TimestampNs>>,
    high_limit_price: Option<FieldOffset<crate::v4::types::Decimal>>,
    low_limit_price: Option<FieldOffset<crate::v4::types::Decimal>>,
    max_price_variation: Option<FieldOffset<crate::v4::types::Decimal>>,
    unit_of_measure_qty: Option<FieldOffset<crate::v4::types::Decimal>>,
    min_price_increment_amount: Option<FieldOffset<crate::v4::types::Decimal>>,
    price_ratio: Option<FieldOffset<crate::v4::types::Decimal>>,
    strike_price: Option<FieldOffset<crate::v4::types::Decimal>>,
    leg_price: Option<FieldOffset<crate::v4::types::Decimal>>,
    leg_delta: Option<FieldOffset<crate::v4::types::Decimal>>,
    user_defined_instrument:
        Option<FieldOffset<crate::v4::types::OpenEnum<crate::UserDefinedInstrument>>>,
    leg_instrument_class: Option<FieldOffset<crate::v4::types::OpenEnum<crate::InstrumentClass>>>,
    leg_side: Option<FieldOffset<crate::v4::types::OpenEnum<crate::Side>>>,
}

impl InstrumentDefOffsets {
    /// Resolves the offsets from the stream's `index`.
    ///
    /// # Errors
    /// This function returns an error if the stream declares no layout for
    /// [`InstrumentDefMsg`] or its layout lacks a required field.
    pub fn resolve(index: &LayoutIndex) -> crate::Result<Self> {
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

impl InstrumentDefMsg {
    /// Parses the raw symbol into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if the record is too short to contain
    /// `raw_symbol` or it contains invalid UTF-8.
    pub fn raw_symbol(&self, offsets: &InstrumentDefOffsets) -> crate::Result<&str> {
        self.field_at(offsets.raw_symbol)
            .ok_or_else(|| Error::conversion::<&str>("`raw_symbol`"))
    }

    /// The last eligible trade time expressed as the number of nanoseconds since the
    /// UNIX epoch, or `None` if the stream doesn't declare `expiration`.
    pub fn raw_expiration(&self, offsets: &InstrumentDefOffsets) -> Option<u64> {
        offsets
            .expiration
            .and_then(|offset| self.field_at(offset))
            .map(|ts| ts.0)
    }

    /// Parses the last eligible trade time into a datetime. Returns `None` if the
    /// stream doesn't declare `expiration` or it contains the sentinel for a null
    /// timestamp.
    pub fn expiration(&self, offsets: &InstrumentDefOffsets) -> Option<time::OffsetDateTime> {
        self.raw_expiration(offsets).and_then(ts_to_dt)
    }

    /// The time of instrument activation expressed as the number of nanoseconds since the
    /// UNIX epoch, or `None` if the stream doesn't declare `activation`.
    pub fn raw_activation(&self, offsets: &InstrumentDefOffsets) -> Option<u64> {
        offsets
            .activation
            .and_then(|offset| self.field_at(offset))
            .map(|ts| ts.0)
    }

    /// Parses the time of instrument activation into a datetime. Returns `None` if the
    /// stream doesn't declare `activation` or it contains the sentinel for a null
    /// timestamp.
    pub fn activation(&self, offsets: &InstrumentDefOffsets) -> Option<time::OffsetDateTime> {
        self.raw_activation(offsets).and_then(ts_to_dt)
    }

    /// The high limit price where every 1 unit corresponds to 1e-9, or `None`
    /// if the stream doesn't declare `high_limit_price`.
    pub fn high_limit_price(&self, offsets: &InstrumentDefOffsets) -> Option<i64> {
        offsets
            .high_limit_price
            .and_then(|offset| self.field_at(offset))
            .map(|px| px.raw)
    }

    /// Converts the high limit price to a floating point, or `None` if the
    /// stream doesn't declare `high_limit_price`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn high_limit_price_f64(&self, offsets: &InstrumentDefOffsets) -> Option<f64> {
        self.high_limit_price(offsets).map(px_to_f64)
    }

    /// The low limit price where every 1 unit corresponds to 1e-9, or `None`
    /// if the stream doesn't declare `low_limit_price`.
    pub fn low_limit_price(&self, offsets: &InstrumentDefOffsets) -> Option<i64> {
        offsets
            .low_limit_price
            .and_then(|offset| self.field_at(offset))
            .map(|px| px.raw)
    }

    /// Converts the low limit price to a floating point, or `None` if the
    /// stream doesn't declare `low_limit_price`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn low_limit_price_f64(&self, offsets: &InstrumentDefOffsets) -> Option<f64> {
        self.low_limit_price(offsets).map(px_to_f64)
    }

    /// The differential value for price banding where every 1 unit corresponds to 1e-9, or `None`
    /// if the stream doesn't declare `max_price_variation`.
    pub fn max_price_variation(&self, offsets: &InstrumentDefOffsets) -> Option<i64> {
        offsets
            .max_price_variation
            .and_then(|offset| self.field_at(offset))
            .map(|px| px.raw)
    }

    /// Converts the differential value for price banding to a floating point, or `None` if the
    /// stream doesn't declare `max_price_variation`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn max_price_variation_f64(&self, offsets: &InstrumentDefOffsets) -> Option<f64> {
        self.max_price_variation(offsets).map(px_to_f64)
    }

    /// The contract size for each instrument where every 1 unit corresponds to 1e-9, or `None`
    /// if the stream doesn't declare `unit_of_measure_qty`.
    pub fn unit_of_measure_qty(&self, offsets: &InstrumentDefOffsets) -> Option<i64> {
        offsets
            .unit_of_measure_qty
            .and_then(|offset| self.field_at(offset))
            .map(|px| px.raw)
    }

    /// Converts the contract size for each instrument to a floating point, or `None` if the
    /// stream doesn't declare `unit_of_measure_qty`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn unit_of_measure_qty_f64(&self, offsets: &InstrumentDefOffsets) -> Option<f64> {
        self.unit_of_measure_qty(offsets).map(px_to_f64)
    }

    /// The min price increment amount where every 1 unit corresponds to 1e-9, or `None`
    /// if the stream doesn't declare `min_price_increment_amount`.
    pub fn min_price_increment_amount(&self, offsets: &InstrumentDefOffsets) -> Option<i64> {
        offsets
            .min_price_increment_amount
            .and_then(|offset| self.field_at(offset))
            .map(|px| px.raw)
    }

    /// Converts the min price increment amount to a floating point, or `None` if the
    /// stream doesn't declare `min_price_increment_amount`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn min_price_increment_amount_f64(&self, offsets: &InstrumentDefOffsets) -> Option<f64> {
        self.min_price_increment_amount(offsets).map(px_to_f64)
    }

    /// The price ratio where every 1 unit corresponds to 1e-9, or `None`
    /// if the stream doesn't declare `price_ratio`.
    pub fn price_ratio(&self, offsets: &InstrumentDefOffsets) -> Option<i64> {
        offsets
            .price_ratio
            .and_then(|offset| self.field_at(offset))
            .map(|px| px.raw)
    }

    /// Converts the price ratio to a floating point, or `None` if the
    /// stream doesn't declare `price_ratio`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn price_ratio_f64(&self, offsets: &InstrumentDefOffsets) -> Option<f64> {
        self.price_ratio(offsets).map(px_to_f64)
    }

    /// The strike price where every 1 unit corresponds to 1e-9, or `None`
    /// if the stream doesn't declare `strike_price`.
    pub fn strike_price(&self, offsets: &InstrumentDefOffsets) -> Option<i64> {
        offsets
            .strike_price
            .and_then(|offset| self.field_at(offset))
            .map(|px| px.raw)
    }

    /// Converts the strike price to a floating point, or `None` if the
    /// stream doesn't declare `strike_price`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn strike_price_f64(&self, offsets: &InstrumentDefOffsets) -> Option<f64> {
        self.strike_price(offsets).map(px_to_f64)
    }

    /// The leg price where every 1 unit corresponds to 1e-9, or `None`
    /// if the stream doesn't declare `leg_price`.
    pub fn leg_price(&self, offsets: &InstrumentDefOffsets) -> Option<i64> {
        offsets
            .leg_price
            .and_then(|offset| self.field_at(offset))
            .map(|px| px.raw)
    }

    /// Converts the leg price to a floating point, or `None` if the
    /// stream doesn't declare `leg_price`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn leg_price_f64(&self, offsets: &InstrumentDefOffsets) -> Option<f64> {
        self.leg_price(offsets).map(px_to_f64)
    }

    /// The leg delta where every 1 unit corresponds to 1e-9, or `None`
    /// if the stream doesn't declare `leg_delta`.
    pub fn leg_delta(&self, offsets: &InstrumentDefOffsets) -> Option<i64> {
        offsets
            .leg_delta
            .and_then(|offset| self.field_at(offset))
            .map(|px| px.raw)
    }

    /// Converts the leg delta to a floating point, or `None` if the
    /// stream doesn't declare `leg_delta`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn leg_delta_f64(&self, offsets: &InstrumentDefOffsets) -> Option<f64> {
        self.leg_delta(offsets).map(px_to_f64)
    }

    /// Parses the user-defined instrument flag into an enum, or `None` if the stream
    /// doesn't declare `user_defined_instrument`.
    ///
    /// # Errors
    /// This function returns an error if `user_defined_instrument` doesn't contain a valid
    /// [`UserDefinedInstrument`](crate::UserDefinedInstrument).
    pub fn user_defined_instrument(
        &self,
        offsets: &InstrumentDefOffsets,
    ) -> crate::Result<Option<crate::UserDefinedInstrument>> {
        offsets
            .user_defined_instrument
            .and_then(|offset| self.field_at(offset))
            .map(|value| value.get())
            .transpose()
    }

    /// Parses the leg instrument class into an enum, or `None` if the stream
    /// doesn't declare `leg_instrument_class`.
    ///
    /// # Errors
    /// This function returns an error if `leg_instrument_class` doesn't contain a valid
    /// [`InstrumentClass`](crate::InstrumentClass).
    pub fn leg_instrument_class(
        &self,
        offsets: &InstrumentDefOffsets,
    ) -> crate::Result<Option<crate::InstrumentClass>> {
        offsets
            .leg_instrument_class
            .and_then(|offset| self.field_at(offset))
            .map(|value| value.get())
            .transpose()
    }

    /// Parses the leg side into an enum, or `None` if the stream
    /// doesn't declare `leg_side`.
    ///
    /// # Errors
    /// This function returns an error if `leg_side` doesn't contain a valid
    /// [`Side`](crate::Side).
    pub fn leg_side(&self, offsets: &InstrumentDefOffsets) -> crate::Result<Option<crate::Side>> {
        offsets
            .leg_side
            .and_then(|offset| self.field_at(offset))
            .map(|value| value.get())
            .transpose()
    }
}

/// The offsets of the [`ImbalanceMsg`] fields its compiled struct doesn't hold,
/// resolved from a stream's [`LayoutIndex`].
#[derive(Debug, Clone, Copy)]
pub struct ImbalanceOffsets {
    auction_time: Option<FieldOffset<crate::v4::types::TimestampNs>>,
    ssr_filling_price: Option<FieldOffset<crate::v4::types::Decimal>>,
    ind_match_price: Option<FieldOffset<crate::v4::types::Decimal>>,
    upper_collar: Option<FieldOffset<crate::v4::types::Decimal>>,
    lower_collar: Option<FieldOffset<crate::v4::types::Decimal>>,
    unpaired_side: Option<FieldOffset<crate::v4::types::OpenEnum<crate::Side>>>,
}

impl ImbalanceOffsets {
    /// Resolves the offsets from the stream's `index`.
    ///
    /// # Errors
    /// This function returns an error if the stream declares no layout for
    /// [`ImbalanceMsg`].
    pub fn resolve(index: &LayoutIndex) -> crate::Result<Self> {
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

impl ImbalanceMsg {
    /// The auction time expressed as the number of nanoseconds since the
    /// UNIX epoch, or `None` if the stream doesn't declare `auction_time`.
    pub fn raw_auction_time(&self, offsets: &ImbalanceOffsets) -> Option<u64> {
        offsets
            .auction_time
            .and_then(|offset| self.field_at(offset))
            .map(|ts| ts.0)
    }

    /// Parses the auction time into a datetime. Returns `None` if the
    /// stream doesn't declare `auction_time` or it contains the sentinel for a null
    /// timestamp.
    pub fn auction_time(&self, offsets: &ImbalanceOffsets) -> Option<time::OffsetDateTime> {
        self.raw_auction_time(offsets).and_then(ts_to_dt)
    }

    /// The ssr filling price where every 1 unit corresponds to 1e-9, or `None`
    /// if the stream doesn't declare `ssr_filling_price`.
    pub fn ssr_filling_price(&self, offsets: &ImbalanceOffsets) -> Option<i64> {
        offsets
            .ssr_filling_price
            .and_then(|offset| self.field_at(offset))
            .map(|px| px.raw)
    }

    /// Converts the ssr filling price to a floating point, or `None` if the
    /// stream doesn't declare `ssr_filling_price`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn ssr_filling_price_f64(&self, offsets: &ImbalanceOffsets) -> Option<f64> {
        self.ssr_filling_price(offsets).map(px_to_f64)
    }

    /// The ind match price where every 1 unit corresponds to 1e-9, or `None`
    /// if the stream doesn't declare `ind_match_price`.
    pub fn ind_match_price(&self, offsets: &ImbalanceOffsets) -> Option<i64> {
        offsets
            .ind_match_price
            .and_then(|offset| self.field_at(offset))
            .map(|px| px.raw)
    }

    /// Converts the ind match price to a floating point, or `None` if the
    /// stream doesn't declare `ind_match_price`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn ind_match_price_f64(&self, offsets: &ImbalanceOffsets) -> Option<f64> {
        self.ind_match_price(offsets).map(px_to_f64)
    }

    /// The upper collar where every 1 unit corresponds to 1e-9, or `None`
    /// if the stream doesn't declare `upper_collar`.
    pub fn upper_collar(&self, offsets: &ImbalanceOffsets) -> Option<i64> {
        offsets
            .upper_collar
            .and_then(|offset| self.field_at(offset))
            .map(|px| px.raw)
    }

    /// Converts the upper collar to a floating point, or `None` if the
    /// stream doesn't declare `upper_collar`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn upper_collar_f64(&self, offsets: &ImbalanceOffsets) -> Option<f64> {
        self.upper_collar(offsets).map(px_to_f64)
    }

    /// The lower collar where every 1 unit corresponds to 1e-9, or `None`
    /// if the stream doesn't declare `lower_collar`.
    pub fn lower_collar(&self, offsets: &ImbalanceOffsets) -> Option<i64> {
        offsets
            .lower_collar
            .and_then(|offset| self.field_at(offset))
            .map(|px| px.raw)
    }

    /// Converts the lower collar to a floating point, or `None` if the
    /// stream doesn't declare `lower_collar`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn lower_collar_f64(&self, offsets: &ImbalanceOffsets) -> Option<f64> {
        self.lower_collar(offsets).map(px_to_f64)
    }

    /// Parses the unpaired side into an enum, or `None` if the stream
    /// doesn't declare `unpaired_side`.
    ///
    /// # Errors
    /// This function returns an error if `unpaired_side` doesn't contain a valid
    /// [`Side`](crate::Side).
    pub fn unpaired_side(&self, offsets: &ImbalanceOffsets) -> crate::Result<Option<crate::Side>> {
        offsets
            .unpaired_side
            .and_then(|offset| self.field_at(offset))
            .map(|value| value.get())
            .transpose()
    }
}

/// The offsets of the [`SymbolMappingMsg`] fields its compiled struct doesn't hold,
/// resolved from a stream's [`LayoutIndex`].
#[derive(Debug, Clone, Copy)]
pub struct SymbolMappingOffsets {
    stype_in_symbol: FieldOffset<crate::v4::types::CStr>,
    stype_out_symbol: FieldOffset<crate::v4::types::CStr>,
}

impl SymbolMappingOffsets {
    /// Resolves the offsets from the stream's `index`.
    ///
    /// # Errors
    /// This function returns an error if the stream declares no layout for
    /// [`SymbolMappingMsg`] or its layout lacks a required field.
    pub fn resolve(index: &LayoutIndex) -> crate::Result<Self> {
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

impl SymbolMappingMsg {
    /// Parses the input symbol into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if the record is too short to contain
    /// `stype_in_symbol` or it contains invalid UTF-8.
    pub fn stype_in_symbol(&self, offsets: &SymbolMappingOffsets) -> crate::Result<&str> {
        self.field_at(offsets.stype_in_symbol)
            .ok_or_else(|| Error::conversion::<&str>("`stype_in_symbol`"))
    }

    /// Parses the output symbol into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if the record is too short to contain
    /// `stype_out_symbol` or it contains invalid UTF-8.
    pub fn stype_out_symbol(&self, offsets: &SymbolMappingOffsets) -> crate::Result<&str> {
        self.field_at(offsets.stype_out_symbol)
            .ok_or_else(|| Error::conversion::<&str>("`stype_out_symbol`"))
    }
}
