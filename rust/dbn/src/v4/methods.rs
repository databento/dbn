use std::os::raw::c_char;

use num_enum::TryFromPrimitive;

use crate::{
    pretty::px_to_f64,
    record::{c_chars_to_str, ts_to_dt},
    Action, Error, ErrorCode, InstrumentClass, MatchAlgorithm, SType, Schema, SecurityUpdateAction,
    Side, StatType, StatUpdateAction, StatusAction, StatusReason, SystemCode, TradingEvent,
    TriState,
};

use super::*;

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
            .map_err(|_| Error::conversion::<Action>(format_args!("{:#04X}", self.action as u8)))
    }

    /// Parses the side that initiates the event into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format_args!("{:#04X}", self.side as u8)))
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
            .map_err(|_| Error::conversion::<Action>(format_args!("{:#04X}", self.action as u8)))
    }

    /// Parses the side into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format_args!("{:#04X}", self.side as u8)))
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
            .map_err(|_| Error::conversion::<Action>(format_args!("{:#04X}", self.action as u8)))
    }

    /// Parses the side that initiates the event into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format_args!("{:#04X}", self.side as u8)))
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
            .map_err(|_| Error::conversion::<Action>(format_args!("{:#04X}", self.action as u8)))
    }

    /// Parses the side that initiates the event into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format_args!("{:#04X}", self.side as u8)))
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
            .map_err(|_| Error::conversion::<Side>(format_args!("{:#04X}", self.side as u8)))
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
            .map_err(|_| Error::conversion::<Action>(format_args!("{:#04X}", self.action as u8)))
    }

    /// Parses the side that initiates the event into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format_args!("{:#04X}", self.side as u8)))
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
            .map_err(|_| Error::conversion::<Side>(format_args!("{:#04X}", self.side as u8)))
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
            .map_err(|_| Error::conversion::<StatusAction>(format_args!("{:#04X}", self.action)))
    }

    /// Parses the reason into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `reason` field does not
    /// contain a valid [`StatusReason`].
    pub fn reason(&self) -> crate::Result<StatusReason> {
        StatusReason::try_from(self.reason)
            .map_err(|_| Error::conversion::<StatusReason>(format_args!("{:#04X}", self.reason)))
    }

    /// Parses the trading event into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `trading_event` field does not
    /// contain a valid [`TradingEvent`].
    pub fn trading_event(&self) -> crate::Result<TradingEvent> {
        TradingEvent::try_from(self.trading_event).map_err(|_| {
            Error::conversion::<TradingEvent>(format_args!("{:#04X}", self.trading_event))
        })
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

    /// Parses the security update action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `security_update_action` field does not
    /// contain a valid [`SecurityUpdateAction`].
    pub fn security_update_action(&self) -> crate::Result<SecurityUpdateAction> {
        SecurityUpdateAction::try_from(self.security_update_action as u8).map_err(|_| {
            Error::conversion::<SecurityUpdateAction>(format_args!(
                "{:#04X}",
                self.security_update_action as u8
            ))
        })
    }

    /// Parses the instrument class into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `instrument_class` field does not
    /// contain a valid [`InstrumentClass`].
    pub fn instrument_class(&self) -> crate::Result<InstrumentClass> {
        InstrumentClass::try_from(self.instrument_class as u8).map_err(|_| {
            Error::conversion::<InstrumentClass>(format_args!(
                "{:#04X}",
                self.instrument_class as u8
            ))
        })
    }

    /// Parses the match algorithm into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `match_algorithm` field does not
    /// contain a valid [`MatchAlgorithm`].
    pub fn match_algorithm(&self) -> crate::Result<MatchAlgorithm> {
        MatchAlgorithm::try_from(self.match_algorithm as u8).map_err(|_| {
            Error::conversion::<MatchAlgorithm>(format_args!("{:#04X}", self.match_algorithm as u8))
        })
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

    /// Parses the side into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format_args!("{:#04X}", self.side as u8)))
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
            .map_err(|_| Error::conversion::<StatType>(format_args!("{:#04X}", self.stat_type)))
    }

    /// Parses the update action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `update_action` field does not
    /// contain a valid [`StatUpdateAction`].
    pub fn update_action(&self) -> crate::Result<StatUpdateAction> {
        StatUpdateAction::try_from(self.update_action).map_err(|_| {
            Error::conversion::<StatUpdateAction>(format_args!("{:#04X}", self.update_action))
        })
    }
}

impl ErrorMsg {
    /// Parses the schema into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `schema` field does not
    /// contain a valid [`Schema`].
    pub fn schema(&self) -> crate::Result<Schema> {
        Schema::try_from(self.schema)
            .map_err(|_| Error::conversion::<Schema>(format_args!("{:#04X}", self.schema)))
    }

    /// Parses the error code into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `code` field does not
    /// contain a valid [`ErrorCode`].
    pub fn code(&self) -> crate::Result<ErrorCode> {
        ErrorCode::try_from(self.code)
            .map_err(|_| Error::conversion::<ErrorCode>(format_args!("{:#04X}", self.code)))
    }

    /// Parses the error message into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `err` contains invalid UTF-8.
    pub fn err(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.err)
    }
}

impl SymbolMappingMsg {
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

    /// Parses the stype in into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `stype_in` field does not
    /// contain a valid [`SType`].
    pub fn stype_in(&self) -> crate::Result<SType> {
        SType::try_from(self.stype_in)
            .map_err(|_| Error::conversion::<SType>(format_args!("{:#04X}", self.stype_in)))
    }

    /// Parses the stype out into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `stype_out` field does not
    /// contain a valid [`SType`].
    pub fn stype_out(&self) -> crate::Result<SType> {
        SType::try_from(self.stype_out)
            .map_err(|_| Error::conversion::<SType>(format_args!("{:#04X}", self.stype_out)))
    }
}

impl SystemMsg {
    /// Parses the schema into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `schema` field does not
    /// contain a valid [`Schema`].
    pub fn schema(&self) -> crate::Result<Schema> {
        Schema::try_from(self.schema)
            .map_err(|_| Error::conversion::<Schema>(format_args!("{:#04X}", self.schema)))
    }

    /// Parses the type of system message into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `code` field does not
    /// contain a valid [`SystemCode`].
    pub fn code(&self) -> crate::Result<SystemCode> {
        SystemCode::try_from(self.code)
            .map_err(|_| Error::conversion::<SystemCode>(format_args!("{:#04X}", self.code)))
    }

    /// Parses the message from the Databento gateway into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `msg` contains invalid UTF-8.
    pub fn msg(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.msg)
    }
}
