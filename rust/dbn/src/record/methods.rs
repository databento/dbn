use std::fmt::Debug;

use num_enum::TryFromPrimitive;

use crate::{
    compat::{ErrorMsgV1, InstrumentDefMsgV1, SymbolMappingMsgV1, SystemMsgV1},
    enums::{StatusAction, StatusReason},
    SType, TradingEvent, TriState,
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
    /// Tries to convert the raw order side to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#04X}", self.side as u8)))
    }

    /// Tries to convert the raw event action to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`].
    pub fn action(&self) -> crate::Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#04X}", self.action as u8)))
    }

    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }

    /// Parses the raw `ts_in_delta`—the delta of `ts_recv - ts_exchange_send`—into a duration.
    pub fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.ts_in_delta)
    }
}

impl TradeMsg {
    /// Tries to convert the raw order side to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#04X}", self.side as u8)))
    }

    /// Tries to convert the raw event action to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`].
    pub fn action(&self) -> crate::Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#04X}", self.action as u8)))
    }

    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }

    /// Parses the raw `ts_in_delta`—the delta of `ts_recv - ts_exchange_send`—into a duration.
    pub fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.ts_in_delta)
    }
}

impl CbboMsg {
    /// Tries to convert the raw `side` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#04X}", self.side as u8)))
    }

    /// Tries to convert the raw event action to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`].
    pub fn action(&self) -> crate::Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#04X}", self.action as u8)))
    }

    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }

    /// Parses the raw `ts_in_delta`—the delta of `ts_recv - ts_exchange_send`—into a duration.
    pub fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.ts_in_delta)
    }
}

impl ConsolidatedBidAskPair {
    /// Tries to convert the raw `ask_pb` into an enum which is useful for
    /// exhaustive pattern matching.
    ///
    /// # Errors
    /// This function returns an error if the `ask_pb` does not correspond with
    /// any known [`Publisher`].
    pub fn ask_pb(&self) -> crate::Result<&str> {
        Publisher::try_from(self.ask_pb)
            .map(|pb| pb.as_str())
            .map_err(|_| crate::error::Error::conversion::<Publisher>(self.ask_pb))
    }

    /// Tries to convert the raw `bid_pb` into an enum which is useful for
    /// exhaustive pattern matching.
    ///
    /// # Errors
    /// This function returns an error if the `publisher_id` does not correspond with
    /// any known [`Publisher`].
    pub fn bid_pb(&self) -> crate::Result<&str> {
        Publisher::try_from(self.bid_pb)
            .map(|pb| pb.as_str())
            .map_err(|_| crate::error::Error::conversion::<Publisher>(self.bid_pb))
    }
}

impl Mbp1Msg {
    /// Tries to convert the raw `side` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#04X}", self.side as u8)))
    }

    /// Tries to convert the raw event action to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`].
    pub fn action(&self) -> crate::Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#04X}", self.action as u8)))
    }

    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }

    /// Parses the raw `ts_in_delta`—the delta of `ts_recv - ts_exchange_send`—into a duration.
    pub fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.ts_in_delta)
    }
}

impl Mbp10Msg {
    /// Tries to convert the raw `side` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#04X}", self.side as u8)))
    }

    /// Tries to convert the raw event action to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`].
    pub fn action(&self) -> Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#04X}", self.action as u8)))
    }

    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }

    /// Parses the raw `ts_in_delta`—the delta of `ts_recv - ts_exchange_send`—into a duration.
    pub fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.ts_in_delta)
    }
}

impl StatusMsg {
    /// Tries to convert the raw status action to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not contain a valid
    /// [`StatusAction`].
    pub fn action(&self) -> Result<StatusAction> {
        StatusAction::try_from(self.action)
            .map_err(|_| Error::conversion::<StatusAction>(format!("{:#06X}", self.action)))
    }

    /// Tries to convert the raw status reason to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `reason` field does not contain a valid
    /// [`StatusReason`].
    pub fn reason(&self) -> Result<StatusReason> {
        StatusReason::try_from(self.reason)
            .map_err(|_| Error::conversion::<StatusReason>(format!("{:#06X}", self.reason)))
    }

    /// Tries to convert the raw status trading event to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `trading_event` field does not contain a
    /// valid [`TradingEvent`].
    pub fn trading_event(&self) -> Result<TradingEvent> {
        TradingEvent::try_from(self.trading_event)
            .map_err(|_| Error::conversion::<TradingEvent>(format!("{:#06X}", self.trading_event)))
    }

    /// Converts the raw `is_trading` state to an `Option<bool>` where `None` indicates
    /// a value is not applicable or available.
    pub fn is_trading(&self) -> Option<bool> {
        TriState::try_from_primitive(self.is_trading as c_char as u8)
            .map(Option::<bool>::from)
            .unwrap_or_default()
    }

    /// Converts the raw `is_quoting` state to an `Option<bool>` where `None` indicates
    /// a value is not applicable or available.
    pub fn is_quoting(&self) -> Option<bool> {
        TriState::try_from_primitive(self.is_quoting as c_char as u8)
            .map(Option::<bool>::from)
            .unwrap_or_default()
    }

    /// Converts the raw `is_short_sell_restricted` state to an `Option<bool>` where
    /// `None` indicates a value is not applicable or available.
    pub fn is_short_sell_restricted(&self) -> Option<bool> {
        TriState::try_from_primitive(self.is_short_sell_restricted as c_char as u8)
            .map(Option::<bool>::from)
            .unwrap_or_default()
    }
}

impl InstrumentDefMsg {
    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
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
}

impl InstrumentDefMsgV1 {
    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
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
}

impl ImbalanceMsg {
    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }
}

impl StatMsg {
    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_recv)
    }

    /// Parses the raw reference timestamp of the statistic value into a datetime.
    /// Returns `None` if `ts_ref` contains the sentinel for a null timestamp.
    pub fn ts_ref(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.ts_ref)
    }

    /// Tries to convert the raw type of the statistic value to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `stat_type` field does not
    /// contain a valid [`StatType`].
    pub fn stat_type(&self) -> Result<StatType> {
        StatType::try_from(self.stat_type)
            .map_err(|_| Error::conversion::<StatType>(self.stat_type))
    }

    /// Tries to convert the raw `update_action` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `update_action` field does not
    /// contain a valid [`StatUpdateAction`].
    pub fn update_action(&self) -> Result<StatUpdateAction> {
        StatUpdateAction::try_from(self.update_action).map_err(|_| {
            Error::conversion::<StatUpdateAction>(format!("{:04X}", self.update_action))
        })
    }
}

impl ErrorMsgV1 {
    /// Creates a new `ErrorMsgV1`.
    ///
    /// # Errors
    /// This function returns an error if `msg` is too long.
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

    /// Returns `err` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `err` contains invalid UTF-8.
    pub fn err(&self) -> Result<&str> {
        c_chars_to_str(&self.err)
    }
}

impl ErrorMsg {
    /// Creates a new `ErrorMsg`.
    ///
    /// # Errors
    /// This function returns an error if `msg` is too long.
    pub fn new(ts_event: u64, msg: &str, is_last: bool) -> Self {
        let mut error = Self {
            hd: RecordHeader::new::<Self>(rtype::ERROR, 0, 0, ts_event),
            is_last: is_last as u8,
            ..Default::default()
        };
        // leave at least one null byte
        for (i, byte) in msg.as_bytes().iter().take(error.err.len() - 1).enumerate() {
            error.err[i] = *byte as c_char;
        }
        error
    }

    /// Returns `err` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `err` contains invalid UTF-8.
    pub fn err(&self) -> Result<&str> {
        c_chars_to_str(&self.err)
    }
}

impl SymbolMappingMsg {
    /// Creates a new `SymbolMappingMsgV2`.
    ///
    /// # Errors
    /// This function returns an error if `stype_in_symbol` or `stype_out_symbol`
    /// contain more than maximum number of characters of 70.
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

    /// Returns the input symbology type.
    ///
    /// # Errors
    /// This function returns an error if `stype_in` does not contain a valid [`SType`].
    pub fn stype_in(&self) -> Result<SType> {
        SType::try_from(self.stype_in).map_err(|_| Error::conversion::<SType>(self.stype_in))
    }

    /// Returns the input symbol as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `stype_in_symbol` contains invalid UTF-8.
    pub fn stype_in_symbol(&self) -> Result<&str> {
        c_chars_to_str(&self.stype_in_symbol)
    }

    /// Returns the output symbology type.
    ///
    /// # Errors
    /// This function returns an error if `stype_out` does not contain a valid [`SType`].
    pub fn stype_out(&self) -> Result<SType> {
        SType::try_from(self.stype_out).map_err(|_| Error::conversion::<SType>(self.stype_out))
    }

    /// Returns the output symbol as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `stype_out_symbol` contains invalid UTF-8.
    pub fn stype_out_symbol(&self) -> Result<&str> {
        c_chars_to_str(&self.stype_out_symbol)
    }

    /// Parses the raw start of the mapping interval into a datetime. Returns `None` if
    /// `start_ts` contains the sentinel for a null timestamp.
    pub fn start_ts(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.start_ts)
    }

    /// Parses the raw end of the mapping interval into a datetime. Returns `None` if
    /// `end_ts` contains the sentinel for a null timestamp.
    pub fn end_ts(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.end_ts)
    }
}

impl SymbolMappingMsgV1 {
    /// Creates a new `SymbolMappingMsg`.
    ///
    /// # Errors
    /// This function returns an error if `stype_in_symbol` or `stype_out_symbol`
    /// contain more than maximum number of characters of 21.
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

    /// Returns the input symbol as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `stype_in_symbol` contains invalid UTF-8.
    pub fn stype_in_symbol(&self) -> Result<&str> {
        c_chars_to_str(&self.stype_in_symbol)
    }

    /// Returns the output symbol as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `stype_out_symbol` contains invalid UTF-8.
    pub fn stype_out_symbol(&self) -> Result<&str> {
        c_chars_to_str(&self.stype_out_symbol)
    }

    /// Parses the raw start of the mapping interval into a datetime. Returns `None` if
    /// `start_ts` contains the sentinel for a null timestamp.
    pub fn start_ts(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.start_ts)
    }

    /// Parses the raw end of the mapping interval into a datetime. Returns `None` if
    /// `end_ts` contains the sentinel for a null timestamp.
    pub fn end_ts(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.end_ts)
    }
}

impl SystemMsg {
    const HEARTBEAT: &'static str = "Heartbeat";

    /// Creates a new `SystemMsg`.
    ///
    /// # Errors
    /// This function returns an error if `msg` is too long.
    pub fn new(ts_event: u64, msg: &str) -> Result<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::SYSTEM, 0, 0, ts_event),
            msg: str_to_c_chars(msg)?,
            ..Default::default()
        })
    }

    /// Creates a new heartbeat `SystemMsg`.
    pub fn heartbeat(ts_event: u64) -> Self {
        Self {
            hd: RecordHeader::new::<Self>(rtype::SYSTEM, 0, 0, ts_event),
            msg: str_to_c_chars(Self::HEARTBEAT).unwrap(),
            code: u8::MAX,
        }
    }

    /// Checks whether the message is a heartbeat from the gateway.
    pub fn is_heartbeat(&self) -> bool {
        self.msg()
            .map(|msg| msg == Self::HEARTBEAT)
            .unwrap_or_default()
    }

    /// Returns the message from the Databento Live Subscription Gateway (LSG) as
    /// a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `msg` contains invalid UTF-8.
    pub fn msg(&self) -> Result<&str> {
        c_chars_to_str(&self.msg)
    }
}

impl SystemMsgV1 {
    /// Creates a new `SystemMsgV1`.
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
            msg: str_to_c_chars(SystemMsg::HEARTBEAT).unwrap(),
        }
    }

    /// Checks whether the message is a heartbeat from the gateway.
    pub fn is_heartbeat(&self) -> bool {
        self.msg()
            .map(|msg| msg == SystemMsg::HEARTBEAT)
            .unwrap_or_default()
    }

    /// Returns the message from the Databento Live Subscription Gateway (LSG) as
    /// a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `msg` contains invalid UTF-8.
    pub fn msg(&self) -> Result<&str> {
        c_chars_to_str(&self.msg)
    }
}

impl<T: HasRType> Record for WithTsOut<T> {
    fn header(&self) -> &RecordHeader {
        self.rec.header()
    }

    fn raw_index_ts(&self) -> u64 {
        self.rec.raw_index_ts()
    }
}

impl<T: HasRType> RecordMut for WithTsOut<T> {
    fn header_mut(&mut self) -> &mut RecordHeader {
        self.rec.header_mut()
    }
}

impl<T: HasRType> HasRType for WithTsOut<T> {
    fn has_rtype(rtype: u8) -> bool {
        T::has_rtype(rtype)
    }
}

impl<T> AsRef<[u8]> for WithTsOut<T>
where
    T: HasRType + AsRef<[u8]>,
{
    fn as_ref(&self) -> &[u8] {
        unsafe { as_u8_slice(self) }
    }
}

impl<T: HasRType> WithTsOut<T> {
    /// Creates a new record with `ts_out`. Updates the `length` property in
    /// [`RecordHeader`] to ensure the additional field is accounted for.
    pub fn new(rec: T, ts_out: u64) -> Self {
        let mut res = Self { rec, ts_out };
        res.header_mut().length = (mem::size_of_val(&res) / RecordHeader::LENGTH_MULTIPLIER) as u8;
        res
    }

    /// Parses the raw live gateway send timestamp into a datetime.
    pub fn ts_out(&self) -> time::OffsetDateTime {
        // u64::MAX is within maximum allowable range
        time::OffsetDateTime::from_unix_timestamp_nanos(self.ts_out as i128).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::flags;

    use super::*;

    #[test]
    fn invalid_rtype_error() {
        let header = RecordHeader::new::<MboMsg>(0xE, 1, 2, 3);
        assert_eq!(
            header.rtype().unwrap_err().to_string(),
            "couldn't convert 0x0E to dbn::enums::rtype::RType"
        );
    }

    #[test]
    fn debug_mbo() {
        let rec = MboMsg {
            hd: RecordHeader::new::<MboMsg>(
                rtype::MBO,
                Publisher::OpraPillarXcbo as u16,
                678,
                1704468548242628731,
            ),
            flags: flags::LAST | flags::BAD_TS_RECV,
            price: 4_500_500_000_000,
            side: b'B' as c_char,
            action: b'A' as c_char,
            ..Default::default()
        };
        assert_eq!(
            format!("{rec:?}"),
            "MboMsg { hd: RecordHeader { length: 14, rtype: Mbo, publisher_id: OpraPillarXcbo, \
            instrument_id: 678, ts_event: 1704468548242628731 }, order_id: 0, \
            price: 4500.500000000, size: 4294967295, flags: 0b10001000, channel_id: 0, \
            action: 'A', side: 'B', ts_recv: 18446744073709551615, ts_in_delta: 0, sequence: 0 }"
        );
    }

    #[test]
    fn debug_stats() {
        let rec = StatMsg {
            stat_type: StatType::OpenInterest as u16,
            update_action: StatUpdateAction::New as u8,
            quantity: 5,
            stat_flags: 0b00000010,
            ..Default::default()
        };
        assert_eq!(
            format!("{rec:?}"),
            "StatMsg { hd: RecordHeader { length: 16, rtype: Statistics, publisher_id: 0, \
            instrument_id: 0, ts_event: 18446744073709551615 }, ts_recv: 18446744073709551615, \
            ts_ref: 18446744073709551615, price: UNDEF_PRICE, quantity: 5, sequence: 0, ts_in_delta: 0, \
            stat_type: OpenInterest, channel_id: 0, update_action: New, stat_flags: 0b00000010 }"
        );
    }

    #[test]
    fn debug_instrument_err() {
        let rec = ErrorMsg {
            err: str_to_c_chars("Missing stype_in").unwrap(),
            ..Default::default()
        };
        assert_eq!(
            format!("{rec:?}"),
            "ErrorMsg { hd: RecordHeader { length: 80, rtype: Error, publisher_id: 0, \
            instrument_id: 0, ts_event: 18446744073709551615 }, err: \"Missing stype_in\", code: 255, is_last: 255 }"
        );
    }

    #[test]
    fn debug_instrument_sys() {
        let rec = SystemMsg::heartbeat(123);
        assert_eq!(
            format!("{rec:?}"),
            "SystemMsg { hd: RecordHeader { length: 80, rtype: System, publisher_id: 0, \
            instrument_id: 0, ts_event: 123 }, msg: \"Heartbeat\", code: 255 }"
        );
    }

    #[test]
    fn debug_instrument_symbol_mapping() {
        let rec = SymbolMappingMsg {
            hd: RecordHeader::new::<SymbolMappingMsg>(
                rtype::SYMBOL_MAPPING,
                0,
                5602,
                1704466940331347283,
            ),
            stype_in: SType::RawSymbol as u8,
            stype_in_symbol: str_to_c_chars("ESM4").unwrap(),
            stype_out: SType::RawSymbol as u8,
            stype_out_symbol: str_to_c_chars("ESM4").unwrap(),
            ..Default::default()
        };
        assert_eq!(
            format!("{rec:?}"),
            "SymbolMappingMsg { hd: RecordHeader { length: 44, rtype: SymbolMapping, publisher_id: 0, instrument_id: 5602, ts_event: 1704466940331347283 }, stype_in: RawSymbol, stype_in_symbol: \"ESM4\", stype_out: RawSymbol, stype_out_symbol: \"ESM4\", start_ts: 18446744073709551615, end_ts: 18446744073709551615 }"
        );
    }
}
