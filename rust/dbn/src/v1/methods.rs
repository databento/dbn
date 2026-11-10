use std::os::raw::c_char;

use crate::{
    pretty::px_to_f64,
    record::{c_chars_to_str, str_to_c_chars},
    rtype, ErrorCode, ErrorRec, InstrumentClass, InstrumentDefRec, RecordHeader, Result, Side,
    StatRec, SymbolMappingRec, SystemCode, SystemRec, UserDefinedInstrument,
};

use super::*;

impl ErrorMsg {
    /// Creates a new `ErrorMsg`. `msg` will be truncated if it's too long.
    pub fn new(ts_event: u64, msg: &str) -> Self {
        let mut error = Self {
            hd: RecordHeader::new::<Self>(rtype::ERROR, 0, 0, ts_event),
            ..Default::default()
        };
        // leave at least one null byte
        for (i, byte) in msg.as_bytes().iter().take(error.err.len() - 1).enumerate() {
            error.err[i] = *byte as c_char;
        }
        error
    }
}

impl ErrorRec for ErrorMsg {
    fn subscription_id(&self) -> u32 {
        u32::MAX
    }

    fn raw_schema(&self) -> u16 {
        u16::MAX
    }

    fn raw_code(&self) -> u8 {
        ErrorCode::default() as u8
    }

    fn is_last(&self) -> u8 {
        u8::MAX
    }

    fn err(&self) -> Result<&str> {
        c_chars_to_str(&self.err)
    }
}

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
        self.security_update_action as c_char
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

impl StatRec for StatMsg {
    fn raw_ts_recv(&self) -> u64 {
        self.ts_recv
    }

    fn raw_ts_ref(&self) -> u64 {
        self.ts_ref
    }

    fn price(&self) -> i64 {
        self.price
    }

    fn quantity(&self) -> i64 {
        if self.quantity == i32::MAX {
            i64::MAX
        } else {
            i64::from(self.quantity)
        }
    }

    fn sequence(&self) -> u32 {
        self.sequence
    }

    fn raw_ts_in_delta(&self) -> i32 {
        self.ts_in_delta
    }

    fn raw_stat_type(&self) -> u16 {
        self.stat_type
    }

    fn channel_id(&self) -> u8 {
        u8::try_from(self.channel_id).unwrap_or(u8::MAX)
    }

    fn raw_update_action(&self) -> u8 {
        self.update_action
    }

    fn stat_flags(&self) -> u8 {
        self.stat_flags
    }
}

impl SymbolMappingMsg {
    /// Creates a new `SymbolMappingMsg`.
    ///
    /// # Errors
    /// This function returns an error if `stype_in_symbol` or `stype_out_symbol`
    /// contain more than maximum number of 21 characters.
    pub fn new(
        instrument_id: u32,
        ts_event: u64,

        stype_in_symbol: &str,

        stype_out_symbol: &str,
        start_ts: u64,
        end_ts: u64,
    ) -> crate::Result<Self> {
        Ok(Self {
            // symbol mappings aren't publisher-specific
            hd: RecordHeader::new::<Self>(rtype::SYMBOL_MAPPING, 0, instrument_id, ts_event),
            stype_in_symbol: str_to_c_chars(stype_in_symbol)?,
            stype_out_symbol: str_to_c_chars(stype_out_symbol)?,
            _dummy: Default::default(),
            start_ts,
            end_ts,
        })
    }
}

impl SymbolMappingRec for SymbolMappingMsg {
    fn raw_start_ts(&self) -> u64 {
        self.start_ts
    }

    fn raw_end_ts(&self) -> u64 {
        self.end_ts
    }

    fn raw_stype_in(&self) -> u8 {
        u8::MAX
    }

    fn raw_stype_out(&self) -> u8 {
        u8::MAX
    }

    fn stype_in_symbol(&self) -> Result<&str> {
        c_chars_to_str(&self.stype_in_symbol)
    }

    fn stype_out_symbol(&self) -> Result<&str> {
        c_chars_to_str(&self.stype_out_symbol)
    }
}

impl SystemMsg {
    /// Creates a new `SystemMsg`.
    ///
    /// # Errors
    /// This function returns an error if `msg` is too long.
    pub fn new(ts_event: u64, msg: &str) -> Result<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::SYSTEM, 0, 0, ts_event),
            msg: str_to_c_chars(msg)?,
        })
    }

    /// Creates a new heartbeat `SystemMsg`.
    pub fn heartbeat(ts_event: u64) -> Self {
        Self {
            hd: RecordHeader::new::<Self>(rtype::SYSTEM, 0, 0, ts_event),
            msg: str_to_c_chars(crate::SystemMsg::HEARTBEAT).unwrap(),
        }
    }
}

impl SystemRec for SystemMsg {
    fn subscription_id(&self) -> u32 {
        u32::MAX
    }

    fn raw_schema(&self) -> u16 {
        u16::MAX
    }

    fn raw_code(&self) -> u8 {
        SystemCode::default() as u8
    }

    fn msg(&self) -> Result<&str> {
        c_chars_to_str(&self.msg)
    }
}
