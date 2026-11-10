use crate::{
    pretty::px_to_f64, record::c_chars_to_str, InstrumentClass, InstrumentDefRec, Result, Side,
    UserDefinedInstrument,
};

use super::*;

impl InstrumentDefMsg {
    /// Converts the trading session settlement price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn trading_reference_price_f64(&self) -> f64 {
        px_to_f64(self.trading_reference_price)
    }

    /// Parses the currency into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `currency` contains invalid UTF-8.
    pub fn currency(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.currency)
    }

    /// Parses the currency used for settlement into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `settl_currency` contains invalid UTF-8.
    pub fn settl_currency(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.settl_currency)
    }

    /// Parses the strategy type of the spread into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `secsubtype` contains invalid UTF-8.
    pub fn secsubtype(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.secsubtype)
    }

    /// Parses the security group code into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `group` contains invalid UTF-8.
    pub fn group(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.group)
    }

    /// Parses the exchange into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `exchange` contains invalid UTF-8.
    pub fn exchange(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.exchange)
    }

    /// Parses the asset into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `asset` contains invalid UTF-8.
    pub fn asset(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.asset)
    }

    /// Parses the CFI code into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `cfi` contains invalid UTF-8.
    pub fn cfi(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.cfi)
    }

    /// Parses the security type into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `security_type` contains invalid UTF-8.
    pub fn security_type(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.security_type)
    }

    /// Parses the unit of measure into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `unit_of_measure` contains invalid UTF-8.
    pub fn unit_of_measure(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.unit_of_measure)
    }

    /// Parses the underlying into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `underlying` contains invalid UTF-8.
    pub fn underlying(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.underlying)
    }

    /// Parses the strike price currency into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `strike_price_currency` contains invalid UTF-8.
    pub fn strike_price_currency(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.strike_price_currency)
    }
}

impl InstrumentDefRec for InstrumentDefMsg {
    fn raw_ts_recv(&self) -> u64 {
        self.ts_recv
    }

    fn min_price_increment(&self) -> i64 {
        self.min_price_increment
    }

    fn display_factor(&self) -> i64 {
        self.display_factor
    }

    fn raw_instrument_id(&self) -> u64 {
        u64::from(self.raw_instrument_id)
    }

    fn min_lot_size_round_lot(&self) -> i32 {
        self.min_lot_size_round_lot
    }

    fn raw_security_update_action(&self) -> c_char {
        self.security_update_action
    }

    fn raw_instrument_class(&self) -> c_char {
        self.instrument_class
    }

    fn raw_match_algorithm(&self) -> c_char {
        self.match_algorithm
    }

    fn channel_id(&self) -> u8 {
        u8::try_from(self.channel_id).unwrap_or(u8::MAX)
    }

    fn raw_symbol(&self) -> Result<&str> {
        c_chars_to_str(&self.raw_symbol)
    }

    fn raw_expiration(&self) -> Option<u64> {
        Some(self.expiration)
    }

    fn raw_activation(&self) -> Option<u64> {
        Some(self.activation)
    }

    fn high_limit_price(&self) -> Option<i64> {
        Some(self.high_limit_price)
    }

    fn low_limit_price(&self) -> Option<i64> {
        Some(self.low_limit_price)
    }

    fn max_price_variation(&self) -> Option<i64> {
        Some(self.max_price_variation)
    }

    fn unit_of_measure_qty(&self) -> Option<i64> {
        Some(self.unit_of_measure_qty)
    }

    fn min_price_increment_amount(&self) -> Option<i64> {
        Some(self.min_price_increment_amount)
    }

    fn price_ratio(&self) -> Option<i64> {
        Some(self.price_ratio)
    }

    fn strike_price(&self) -> Option<i64> {
        Some(self.strike_price)
    }

    fn leg_price(&self) -> Option<i64> {
        None
    }

    fn leg_delta(&self) -> Option<i64> {
        None
    }

    fn user_defined_instrument(&self) -> Result<Option<UserDefinedInstrument>> {
        Ok(Some(self.user_defined_instrument))
    }

    fn leg_instrument_class(&self) -> Result<Option<InstrumentClass>> {
        Ok(None)
    }

    fn leg_side(&self) -> Result<Option<Side>> {
        Ok(None)
    }
}
