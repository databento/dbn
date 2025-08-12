use crate::{
    pretty::px_to_f64,
    record::{c_chars_to_str, ts_to_dt},
    Error, InstrumentClass, MatchAlgorithm, SecurityUpdateAction,
};

use super::*;

impl InstrumentDefMsg {
    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }

    /// Converts the minimum constant tick to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn min_price_increment_f64(&self) -> f64 {
        px_to_f64(self.min_price_increment)
    }

    /// Converts the display factor to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn display_factor_f64(&self) -> f64 {
        px_to_f64(self.display_factor)
    }

    /// Parses the last eligible trade time into a datetime.
    /// Returns `None` if `expiration` contains the sentinel for a null timestamp.
    pub fn expiration(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.expiration)
    }

    /// Parses the time of instrument activation into a datetime.
    /// Returns `None` if `activation` contains the sentinel for a null timestamp.
    pub fn activation(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.activation)
    }

    /// Converts the high limit price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn high_limit_price_f64(&self) -> f64 {
        px_to_f64(self.high_limit_price)
    }

    /// Converts the low limit price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn low_limit_price_f64(&self) -> f64 {
        px_to_f64(self.low_limit_price)
    }

    /// Converts the differential value for price banding to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn max_price_variation_f64(&self) -> f64 {
        px_to_f64(self.max_price_variation)
    }

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

    /// Converts the contract size for each instrument to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn unit_of_measure_qty_f64(&self) -> f64 {
        px_to_f64(self.unit_of_measure_qty)
    }

    /// Converts the min price increment amount to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn min_price_increment_amount_f64(&self) -> f64 {
        px_to_f64(self.min_price_increment_amount)
    }

    /// Converts the price ratio to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn price_ratio_f64(&self) -> f64 {
        px_to_f64(self.price_ratio)
    }

    /// Converts the strike price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn strike_price_f64(&self) -> f64 {
        px_to_f64(self.strike_price)
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

    /// Parses the raw symbol into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `raw_symbol` contains invalid UTF-8.
    pub fn raw_symbol(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.raw_symbol)
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

    /// Parses the instrument class into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `instrument_class` field does not
    /// contain a valid [`InstrumentClass`].
    pub fn instrument_class(&self) -> crate::Result<InstrumentClass> {
        InstrumentClass::try_from(self.instrument_class as u8).map_err(|_| {
            Error::conversion::<InstrumentClass>(format!("{:#04X}", self.instrument_class as u8))
        })
    }

    /// Parses the match algorithm into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `match_algorithm` field does not
    /// contain a valid [`MatchAlgorithm`].
    pub fn match_algorithm(&self) -> crate::Result<MatchAlgorithm> {
        MatchAlgorithm::try_from(self.match_algorithm as u8).map_err(|_| {
            Error::conversion::<MatchAlgorithm>(format!("{:#04X}", self.match_algorithm as u8))
        })
    }

    /// Parses the security update action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `security_update_action` field does not
    /// contain a valid [`SecurityUpdateAction`].
    pub fn security_update_action(&self) -> crate::Result<SecurityUpdateAction> {
        SecurityUpdateAction::try_from(self.security_update_action as u8).map_err(|_| {
            Error::conversion::<SecurityUpdateAction>(format!(
                "{:#04X}",
                self.security_update_action as u8
            ))
        })
    }
}
