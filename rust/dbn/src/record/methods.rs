use std::fmt::Debug;

use num_enum::TryFromPrimitive;

use crate::{
    enums::{ErrorCode, StatusAction, StatusReason, SystemCode},
    pretty::px_to_f64,
    SType, StatType, TradingEvent, TriState,
};

use super::*;

impl RecordHeader {
    /// The multiplier for converting the `length` field to the number of bytes.
    pub const LENGTH_MULTIPLIER: usize = 4;

    /// Creates a new `RecordHeader`. `R` and `rtype` should be compatible.
    pub const fn new<R: HasRType>(
        rtype: u8,
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
    ) -> Self {
        Self {
            length: (mem::size_of::<R>() / Self::LENGTH_MULTIPLIER) as u8,
            rtype,
            publisher_id,
            instrument_id,
            ts_event,
        }
    }

    /// Returns the size of the **entire** record in bytes. The size of a `RecordHeader`
    /// is constant.
    pub const fn record_size(&self) -> usize {
        self.length as usize * Self::LENGTH_MULTIPLIER
    }

    /// Tries to convert the raw record type into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `rtype` field does not
    /// contain a valid, known [`RType`].
    pub fn rtype(&self) -> crate::Result<RType> {
        RType::try_from(self.rtype)
            .map_err(|_| Error::conversion::<RType>(format!("{:#04X}", self.rtype)))
    }

    /// Tries to convert the raw `publisher_id` into an enum which is useful for
    /// exhaustive pattern matching.
    ///
    /// # Errors
    /// This function returns an error if the `publisher_id` does not correspond with
    /// any known [`Publisher`].
    pub fn publisher(&self) -> crate::Result<Publisher> {
        Publisher::try_from(self.publisher_id)
            .map_err(|_| Error::conversion::<Publisher>(self.publisher_id))
    }

    /// Parses the raw matching-engine-received timestamp into a datetime. Returns
    /// `None` if `ts_event` contains the sentinel for a null timestamp.
    pub fn ts_event(&self) -> Option<time::OffsetDateTime> {
        if self.ts_event == crate::UNDEF_TIMESTAMP {
            None
        } else {
            // u64::MAX is within maximum allowable range
            Some(time::OffsetDateTime::from_unix_timestamp_nanos(self.ts_event as i128).unwrap())
        }
    }
}

impl Debug for RecordHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("RecordHeader");
        debug_struct.field("length", &self.length);
        match self.rtype() {
            Ok(rtype) => debug_struct.field("rtype", &format_args!("{rtype:?}")),
            Err(_) => debug_struct.field("rtype", &format_args!("{:#04X}", &self.rtype)),
        };
        match self.publisher() {
            Ok(p) => debug_struct.field("publisher_id", &format_args!("{p:?}")),
            Err(_) => debug_struct.field("publisher_id", &self.publisher_id),
        };
        debug_struct
            .field("instrument_id", &self.instrument_id)
            .field("ts_event", &self.ts_event)
            .finish()
    }
}

impl MboMsg {
    /// Converts the order price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn price_f64(&self) -> f64 {
        px_to_f64(self.price)
    }

    /// Parses the action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`].
    pub fn action(&self) -> crate::Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#04X}", self.action as u8)))
    }

    /// Parses the side that initiates the event into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#04X}", self.side as u8)))
    }

    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }

    /// Parses the difference between `ts_recv` and the matching-engine-sending timestamp into a duration.
    pub fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.ts_in_delta)
    }
}

impl BidAskPair {
    /// Converts the bid price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn bid_px_f64(&self) -> f64 {
        px_to_f64(self.bid_px)
    }

    /// Converts the ask price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn ask_px_f64(&self) -> f64 {
        px_to_f64(self.ask_px)
    }
}

impl ConsolidatedBidAskPair {
    /// Converts the bid price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn bid_px_f64(&self) -> f64 {
        px_to_f64(self.bid_px)
    }

    /// Converts the ask price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn ask_px_f64(&self) -> f64 {
        px_to_f64(self.ask_px)
    }

    /// Parses the bid publisher into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `bid_pb` field does not
    /// contain a valid [`Publisher`].
    pub fn bid_pb(&self) -> crate::Result<Publisher> {
        Publisher::try_from(self.bid_pb)
            .map_err(|_| Error::conversion::<Publisher>(format!("{:#04X}", self.bid_pb)))
    }

    /// Parses the ask publisher into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `ask_pb` field does not
    /// contain a valid [`Publisher`].
    pub fn ask_pb(&self) -> crate::Result<Publisher> {
        Publisher::try_from(self.ask_pb)
            .map_err(|_| Error::conversion::<Publisher>(format!("{:#04X}", self.ask_pb)))
    }
}

impl TradeMsg {
    /// Converts the price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn price_f64(&self) -> f64 {
        px_to_f64(self.price)
    }

    /// Parses the action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`].
    pub fn action(&self) -> crate::Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#04X}", self.action as u8)))
    }

    /// Parses the side into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#04X}", self.side as u8)))
    }

    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }

    /// Parses the difference between `ts_recv` and the matching-engine-sending timestamp into a duration.
    pub fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.ts_in_delta)
    }
}

impl Mbp1Msg {
    /// Converts the order price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn price_f64(&self) -> f64 {
        px_to_f64(self.price)
    }

    /// Parses the action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`].
    pub fn action(&self) -> crate::Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#04X}", self.action as u8)))
    }

    /// Parses the side that initiates the event into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#04X}", self.side as u8)))
    }

    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }

    /// Parses the difference between `ts_recv` and the matching-engine-sending timestamp into a duration.
    pub fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.ts_in_delta)
    }
}

impl Mbp10Msg {
    /// Converts the order price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn price_f64(&self) -> f64 {
        px_to_f64(self.price)
    }

    /// Parses the action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`].
    pub fn action(&self) -> crate::Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#04X}", self.action as u8)))
    }

    /// Parses the side that initiates the event into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#04X}", self.side as u8)))
    }

    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }

    /// Parses the difference between `ts_recv` and the matching-engine-sending timestamp into a duration.
    pub fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.ts_in_delta)
    }
}

impl BboMsg {
    /// Converts the last trade price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn price_f64(&self) -> f64 {
        px_to_f64(self.price)
    }

    /// Parses the side that initiated the last trade into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#04X}", self.side as u8)))
    }

    /// Parses the end timestamp of the interval capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }
}

impl Cmbp1Msg {
    /// Converts the order price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn price_f64(&self) -> f64 {
        px_to_f64(self.price)
    }

    /// Parses the action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`].
    pub fn action(&self) -> crate::Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#04X}", self.action as u8)))
    }

    /// Parses the side that initiates the event into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#04X}", self.side as u8)))
    }

    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }

    /// Parses the difference between `ts_recv` and the matching-engine-sending timestamp into a duration.
    pub fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.ts_in_delta)
    }
}

impl CbboMsg {
    /// Converts the last trade price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn price_f64(&self) -> f64 {
        px_to_f64(self.price)
    }

    /// Parses the side that initiated the last trade into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#04X}", self.side as u8)))
    }

    /// Parses the end timestamp of the interval capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }
}

impl OhlcvMsg {
    /// Converts the open price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn open_f64(&self) -> f64 {
        px_to_f64(self.open)
    }

    /// Converts the high price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn high_f64(&self) -> f64 {
        px_to_f64(self.high)
    }

    /// Converts the low price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn low_f64(&self) -> f64 {
        px_to_f64(self.low)
    }

    /// Converts the close price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn close_f64(&self) -> f64 {
        px_to_f64(self.close)
    }
}

impl StatusMsg {
    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }

    /// Parses the action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`StatusAction`].
    pub fn action(&self) -> crate::Result<StatusAction> {
        StatusAction::try_from(self.action)
            .map_err(|_| Error::conversion::<StatusAction>(format!("{:#04X}", self.action)))
    }

    /// Parses the reason into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `reason` field does not
    /// contain a valid [`StatusReason`].
    pub fn reason(&self) -> crate::Result<StatusReason> {
        StatusReason::try_from(self.reason)
            .map_err(|_| Error::conversion::<StatusReason>(format!("{:#04X}", self.reason)))
    }

    /// Parses the trading event into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `trading_event` field does not
    /// contain a valid [`TradingEvent`].
    pub fn trading_event(&self) -> crate::Result<TradingEvent> {
        TradingEvent::try_from(self.trading_event)
            .map_err(|_| Error::conversion::<TradingEvent>(format!("{:#04X}", self.trading_event)))
    }

    /// Parses the trading state into an `Option<bool>` where `None` indicates
    /// a value is not applicable or available.
    pub fn is_trading(&self) -> Option<bool> {
        TriState::try_from_primitive(self.is_trading as c_char as u8)
            .map(Option::<bool>::from)
            .unwrap_or_default()
    }

    /// Parses the quoting state into an `Option<bool>` where `None` indicates
    /// a value is not applicable or available.
    pub fn is_quoting(&self) -> Option<bool> {
        TriState::try_from_primitive(self.is_quoting as c_char as u8)
            .map(Option::<bool>::from)
            .unwrap_or_default()
    }

    /// Parses the short selling state into an `Option<bool>` where `None` indicates
    /// a value is not applicable or available.
    pub fn is_short_sell_restricted(&self) -> Option<bool> {
        TriState::try_from_primitive(self.is_short_sell_restricted as c_char as u8)
            .map(Option::<bool>::from)
            .unwrap_or_default()
    }
}

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

    /// Converts the leg price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn leg_price_f64(&self) -> f64 {
        px_to_f64(self.leg_price)
    }

    /// Converts the leg delta to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn leg_delta_f64(&self) -> f64 {
        px_to_f64(self.leg_delta)
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

    /// Parses the leg raw symbol into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `leg_raw_symbol` contains invalid UTF-8.
    pub fn leg_raw_symbol(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.leg_raw_symbol)
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

    /// Parses the user-defined instrument flag into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `user_defined_instrument` field does not
    /// contain a valid [`UserDefinedInstrument`].
    pub fn user_defined_instrument(&self) -> crate::Result<UserDefinedInstrument> {
        UserDefinedInstrument::try_from(self.user_defined_instrument as u8).map_err(|_| {
            Error::conversion::<UserDefinedInstrument>(format!(
                "{:#04X}",
                self.user_defined_instrument as u8
            ))
        })
    }

    /// Parses the leg instrument class into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `leg_instrument_class` field does not
    /// contain a valid [`InstrumentClass`].
    pub fn leg_instrument_class(&self) -> crate::Result<InstrumentClass> {
        InstrumentClass::try_from(self.leg_instrument_class as u8).map_err(|_| {
            Error::conversion::<InstrumentClass>(format!(
                "{:#04X}",
                self.leg_instrument_class as u8
            ))
        })
    }

    /// Parses the leg side into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `leg_side` field does not
    /// contain a valid [`Side`].
    pub fn leg_side(&self) -> crate::Result<Side> {
        Side::try_from(self.leg_side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#04X}", self.leg_side as u8)))
    }
}

impl ImbalanceMsg {
    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }

    /// Converts the ref price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn ref_price_f64(&self) -> f64 {
        px_to_f64(self.ref_price)
    }

    /// Parses the auction time into a datetime.
    /// Returns `None` if `auction_time` contains the sentinel for a null timestamp.
    pub fn auction_time(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.auction_time)
    }

    /// Converts the cont book clr price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn cont_book_clr_price_f64(&self) -> f64 {
        px_to_f64(self.cont_book_clr_price)
    }

    /// Converts the auct interest clr price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn auct_interest_clr_price_f64(&self) -> f64 {
        px_to_f64(self.auct_interest_clr_price)
    }

    /// Converts the ssr filling price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn ssr_filling_price_f64(&self) -> f64 {
        px_to_f64(self.ssr_filling_price)
    }

    /// Converts the ind match price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn ind_match_price_f64(&self) -> f64 {
        px_to_f64(self.ind_match_price)
    }

    /// Converts the upper collar to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn upper_collar_f64(&self) -> f64 {
        px_to_f64(self.upper_collar)
    }

    /// Converts the lower collar to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn lower_collar_f64(&self) -> f64 {
        px_to_f64(self.lower_collar)
    }

    /// Parses the side into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#04X}", self.side as u8)))
    }

    /// Parses the unpaired side into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `unpaired_side` field does not
    /// contain a valid [`Side`].
    pub fn unpaired_side(&self) -> crate::Result<Side> {
        Side::try_from(self.unpaired_side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#04X}", self.unpaired_side as u8)))
    }
}

impl StatMsg {
    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }

    /// Parses the reference timestamp of the statistic value into a datetime.
    /// Returns `None` if `ts_ref` contains the sentinel for a null timestamp.
    pub fn ts_ref(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_ref)
    }

    /// Converts the value for price statistics to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    pub fn price_f64(&self) -> f64 {
        px_to_f64(self.price)
    }

    /// Parses the difference between `ts_recv` and the matching-engine-sending timestamp into a duration.
    pub fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.ts_in_delta)
    }

    /// Parses the type of statistic value into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `stat_type` field does not
    /// contain a valid [`StatType`].
    pub fn stat_type(&self) -> crate::Result<StatType> {
        StatType::try_from(self.stat_type)
            .map_err(|_| Error::conversion::<StatType>(format!("{:#04X}", self.stat_type)))
    }

    /// Parses the update action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `update_action` field does not
    /// contain a valid [`StatUpdateAction`].
    pub fn update_action(&self) -> crate::Result<StatUpdateAction> {
        StatUpdateAction::try_from(self.update_action).map_err(|_| {
            Error::conversion::<StatUpdateAction>(format!("{:#04X}", self.update_action))
        })
    }
}

impl ErrorMsg {
    /// Creates a new `ErrorMsg`. `msg` will be truncated if it's too long.
    pub fn new(ts_event: u64, code: Option<ErrorCode>, msg: &str, is_last: bool) -> Self {
        let mut error = Self {
            hd: RecordHeader::new::<Self>(rtype::ERROR, 0, 0, ts_event),
            is_last: is_last as u8,
            ..Default::default()
        };
        if let Some(code) = code {
            error.code = code as u8;
        }
        // leave at least one null byte
        for (i, byte) in msg.as_bytes().iter().take(error.err.len() - 1).enumerate() {
            error.err[i] = *byte as c_char;
        }
        error
    }

    /// Parses the error message into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `err` contains invalid UTF-8.
    pub fn err(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.err)
    }

    /// Parses the error code into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `code` field does not
    /// contain a valid [`ErrorCode`].
    pub fn code(&self) -> crate::Result<ErrorCode> {
        ErrorCode::try_from(self.code)
            .map_err(|_| Error::conversion::<ErrorCode>(format!("{:#04X}", self.code)))
    }
}

impl SymbolMappingMsg {
    /// Creates a new `SymbolMappingMsg`.
    ///
    /// # Errors
    /// This function returns an error if `stype_in_symbol` or `stype_out_symbol`
    /// contain more than maximum number of 70 characters.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instrument_id: u32,
        ts_event: u64,
        stype_in: SType,
        stype_in_symbol: &str,
        stype_out: SType,
        stype_out_symbol: &str,
        start_ts: u64,
        end_ts: u64,
    ) -> crate::Result<Self> {
        Ok(Self {
            // symbol mappings aren't publisher-specific
            hd: RecordHeader::new::<Self>(rtype::SYMBOL_MAPPING, 0, instrument_id, ts_event),
            stype_in: stype_in as u8,
            stype_in_symbol: str_to_c_chars(stype_in_symbol)?,
            stype_out: stype_out as u8,
            stype_out_symbol: str_to_c_chars(stype_out_symbol)?,
            start_ts,
            end_ts,
        })
    }

    /// Parses the stype in into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `stype_in` field does not
    /// contain a valid [`SType`].
    pub fn stype_in(&self) -> crate::Result<SType> {
        SType::try_from(self.stype_in)
            .map_err(|_| Error::conversion::<SType>(format!("{:#04X}", self.stype_in)))
    }

    /// Parses the input symbol into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `stype_in_symbol` contains invalid UTF-8.
    pub fn stype_in_symbol(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.stype_in_symbol)
    }

    /// Parses the stype out into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `stype_out` field does not
    /// contain a valid [`SType`].
    pub fn stype_out(&self) -> crate::Result<SType> {
        SType::try_from(self.stype_out)
            .map_err(|_| Error::conversion::<SType>(format!("{:#04X}", self.stype_out)))
    }

    /// Parses the output symbol into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `stype_out_symbol` contains invalid UTF-8.
    pub fn stype_out_symbol(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.stype_out_symbol)
    }

    /// Parses the start of the mapping interval into a datetime.
    /// Returns `None` if `start_ts` contains the sentinel for a null timestamp.
    pub fn start_ts(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.start_ts)
    }

    /// Parses the end of the mapping interval into a datetime.
    /// Returns `None` if `end_ts` contains the sentinel for a null timestamp.
    pub fn end_ts(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.end_ts)
    }
}

impl SystemMsg {
    pub(crate) const HEARTBEAT: &'static str = "Heartbeat";

    /// Creates a new `SystemMsg`.
    ///
    /// # Errors
    /// This function returns an error if `msg` is too long.
    pub fn new(ts_event: u64, code: Option<SystemCode>, msg: &str) -> Result<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::SYSTEM, 0, 0, ts_event),
            msg: str_to_c_chars(msg)?,
            code: code.map(u8::from).unwrap_or(u8::MAX),
        })
    }

    /// Creates a new heartbeat `SystemMsg`.
    pub fn heartbeat(ts_event: u64) -> Self {
        Self {
            hd: RecordHeader::new::<Self>(rtype::SYSTEM, 0, 0, ts_event),
            msg: str_to_c_chars(Self::HEARTBEAT).unwrap(),
            code: SystemCode::Heartbeat as u8,
        }
    }

    /// Checks whether the message is a heartbeat from the gateway.
    pub fn is_heartbeat(&self) -> bool {
        if let Ok(code) = self.code() {
            code == SystemCode::Heartbeat
        } else {
            self.msg()
                .map(|msg| msg == Self::HEARTBEAT)
                .unwrap_or_default()
        }
    }

    /// Parses the message from the Databento gateway into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `msg` contains invalid UTF-8.
    pub fn msg(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.msg)
    }

    /// Parses the type of system message into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `code` field does not
    /// contain a valid [`SystemCode`].
    pub fn code(&self) -> crate::Result<SystemCode> {
        SystemCode::try_from(self.code)
            .map_err(|_| Error::conversion::<SystemCode>(format!("{:#04X}", self.code)))
    }
}
