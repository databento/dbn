use std::os::raw::c_char;

use crate::{
    pretty::px_to_f64,
    record::{c_chars_to_str, ts_to_dt},
    rtype, v1, v2, Error, InstrumentClass, MatchAlgorithm, RecordHeader, Result,
    SecurityUpdateAction, UserDefinedInstrument,
};

use super::InstrumentDefMsg;

impl From<&v1::InstrumentDefMsg> for InstrumentDefMsg {
    fn from(old: &v1::InstrumentDefMsg) -> Self {
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
            unit_of_measure_qty: old.unit_of_measure_qty,
            min_price_increment_amount: old.min_price_increment_amount,
            price_ratio: old.price_ratio,
            inst_attrib_value: old.inst_attrib_value,
            underlying_id: old.underlying_id,
            raw_instrument_id: u64::from(old.raw_instrument_id),
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
            appl_id: old.appl_id,
            maturity_year: old.maturity_year,
            decay_start_date: old.decay_start_date,
            channel_id: old.channel_id,
            currency: old.currency,
            settl_currency: old.settl_currency,
            secsubtype: old.secsubtype,
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
            main_fraction: old.main_fraction,
            price_display_format: old.price_display_format,
            sub_fraction: old.sub_fraction,
            underlying_product: old.underlying_product,
            security_update_action: old.security_update_action as c_char,
            maturity_month: old.maturity_month,
            maturity_day: old.maturity_day,
            maturity_week: old.maturity_week,
            user_defined_instrument: old.user_defined_instrument as c_char,
            contract_multiplier_unit: old.contract_multiplier_unit,
            flow_schedule_type: old.flow_schedule_type,
            tick_rule: old.tick_rule,
            ..Default::default()
        };
        res.raw_symbol[..v1::SYMBOL_CSTR_LEN].copy_from_slice(old.raw_symbol.as_slice());
        res
    }
}

impl From<&v2::InstrumentDefMsg> for InstrumentDefMsg {
    fn from(old: &v2::InstrumentDefMsg) -> Self {
        Self {
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
            unit_of_measure_qty: old.unit_of_measure_qty,
            min_price_increment_amount: old.min_price_increment_amount,
            price_ratio: old.price_ratio,
            inst_attrib_value: old.inst_attrib_value,
            underlying_id: old.underlying_id,
            raw_instrument_id: u64::from(old.raw_instrument_id),
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
            appl_id: old.appl_id,
            maturity_year: old.maturity_year,
            decay_start_date: old.decay_start_date,
            channel_id: old.channel_id,
            currency: old.currency,
            settl_currency: old.settl_currency,
            secsubtype: old.secsubtype,
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
            main_fraction: old.main_fraction,
            price_display_format: old.price_display_format,
            sub_fraction: old.sub_fraction,
            underlying_product: old.underlying_product,
            security_update_action: old.security_update_action as c_char,
            maturity_month: old.maturity_month,
            maturity_day: old.maturity_day,
            maturity_week: old.maturity_week,
            user_defined_instrument: old.user_defined_instrument as c_char,
            contract_multiplier_unit: old.contract_multiplier_unit,
            flow_schedule_type: old.flow_schedule_type,
            tick_rule: old.tick_rule,
            raw_symbol: old.raw_symbol,
            ..Default::default()
        }
    }
}

impl InstrumentDefMsg {
    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }

    /// Returns the unit of measure quantity as a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    pub fn unit_of_measure_qty_f64(&self) -> f64 {
        px_to_f64(self.unit_of_measure_qty)
    }

    /// Returns the strike price as a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    pub fn strike_price_f64(&self) -> f64 {
        px_to_f64(self.strike_price)
    }

    /// Parses the raw last eligible trade time into a datetime. Returns `None` if
    /// `expiration` contains the sentinel for a null timestamp.
    pub fn expiration(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.expiration)
    }

    /// Parses the raw time of instrument action into a datetime. Returns `None` if
    /// `activation` contains the sentinel for a null timestamp.
    pub fn activation(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.activation)
    }

    /// Returns currency used for price fields as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `currency` contains invalid UTF-8.
    pub fn currency(&self) -> Result<&str> {
        c_chars_to_str(&self.currency)
    }

    /// Returns currency used for settlement as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `settl_currency` contains invalid UTF-8.
    pub fn settl_currency(&self) -> Result<&str> {
        c_chars_to_str(&self.settl_currency)
    }

    /// Returns the strategy type of the spread as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `secsubtype` contains invalid UTF-8.
    pub fn secsubtype(&self) -> Result<&str> {
        c_chars_to_str(&self.secsubtype)
    }

    /// Returns the instrument raw symbol assigned by the publisher as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `raw_symbol` contains invalid UTF-8.
    pub fn raw_symbol(&self) -> Result<&str> {
        c_chars_to_str(&self.raw_symbol)
    }

    /// Returns exchange used to identify the instrument as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `exchange` contains invalid UTF-8.
    pub fn exchange(&self) -> Result<&str> {
        c_chars_to_str(&self.exchange)
    }

    /// Returns the underlying asset code (product code) of the instrument as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `asset` contains invalid UTF-8.
    pub fn asset(&self) -> Result<&str> {
        c_chars_to_str(&self.asset)
    }

    /// Returns the ISO standard instrument categorization code as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `cfi` contains invalid UTF-8.
    pub fn cfi(&self) -> Result<&str> {
        c_chars_to_str(&self.cfi)
    }

    /// Returns the type of the strument, e.g. FUT for future or future spread as
    /// a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `security_type` contains invalid UTF-8.
    pub fn security_type(&self) -> Result<&str> {
        c_chars_to_str(&self.security_type)
    }

    /// Returns the unit of measure for the instrument's original contract size, e.g.
    /// USD or LBS, as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `unit_of_measure` contains invalid UTF-8.
    pub fn unit_of_measure(&self) -> Result<&str> {
        c_chars_to_str(&self.unit_of_measure)
    }

    /// Returns the symbol of the first underlying instrument as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `underlying` contains invalid UTF-8.
    pub fn underlying(&self) -> Result<&str> {
        c_chars_to_str(&self.underlying)
    }

    /// Returns the currency of [`strike_price`](Self::strike_price) as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `strike_price_currency` contains invalid UTF-8.
    pub fn strike_price_currency(&self) -> Result<&str> {
        c_chars_to_str(&self.strike_price_currency)
    }

    /// Returns the security group code of the instrumnet as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `group` contains invalid UTF-8.
    pub fn group(&self) -> Result<&str> {
        c_chars_to_str(&self.group)
    }

    /// Tries to convert the raw classification of the instrument to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `instrument_class` field does not
    /// contain a valid [`InstrumentClass`].
    pub fn instrument_class(&self) -> Result<InstrumentClass> {
        InstrumentClass::try_from(self.instrument_class as u8).map_err(|_| {
            Error::conversion::<InstrumentClass>(format!("{:#04X}", self.instrument_class as u8))
        })
    }

    /// Tries to convert the raw matching algorithm used for the instrument to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `match_algorithm` field does not
    /// contain a valid [`MatchAlgorithm`].
    pub fn match_algorithm(&self) -> Result<MatchAlgorithm> {
        MatchAlgorithm::try_from(self.match_algorithm as u8).map_err(|_| {
            Error::conversion::<MatchAlgorithm>(format!("{:#04X}", self.match_algorithm as u8))
        })
    }

    /// Returns the action indicating whether the instrument definition has been added,
    /// modified, or deleted.
    ///
    /// # Errors
    /// This function returns an error if the `security_update_action` field does not
    /// contain a valid [`SecurityUpdateAction`].
    pub fn security_update_action(&self) -> Result<SecurityUpdateAction> {
        SecurityUpdateAction::try_from(self.security_update_action as u8).map_err(|_| {
            Error::conversion::<SecurityUpdateAction>(format!(
                "{:#04X}",
                self.security_update_action as u8
            ))
        })
    }

    /// Returns the enum whether the instrument definition is user-defined or not.
    ///
    /// # Errors
    /// This function returns an error if the `security_update_action` field does not
    /// contain a valid [`UserDefinedInstrument`].
    pub fn user_defined_instrument(&self) -> Result<UserDefinedInstrument> {
        UserDefinedInstrument::try_from(self.user_defined_instrument as u8).map_err(|_| {
            Error::conversion::<UserDefinedInstrument>(format!(
                "{:#04X}",
                self.user_defined_instrument as u8
            ))
        })
    }

    /// Returns the leg's raw symbol assigned by the publisher as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `raw_symbol` contains invalid UTF-8.
    pub fn leg_raw_symbol(&self) -> Result<&str> {
        c_chars_to_str(&self.leg_raw_symbol)
    }

    /// Tries to convert the raw classification of the leg instrument to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `instrument_class` field does not
    /// contain a valid [`InstrumentClass`].
    pub fn leg_instrument_class(&self) -> Result<InstrumentClass> {
        InstrumentClass::try_from(self.leg_instrument_class as u8).map_err(|_| {
            Error::conversion::<InstrumentClass>(format!(
                "{:#04X}",
                self.leg_instrument_class as u8
            ))
        })
    }

    /// Returns the leg price as a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    pub fn leg_price_f64(&self) -> f64 {
        px_to_f64(self.leg_price)
    }

    /// Returns the leg delta as a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    pub fn leg_delta_f64(&self) -> f64 {
        px_to_f64(self.leg_delta)
    }
}
