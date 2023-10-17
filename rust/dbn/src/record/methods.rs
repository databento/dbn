use crate::{compat::SymbolMappingMsgV2, SType};

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
            .map_err(|_| Error::conversion::<RType>(format!("{:#02X}", self.rtype)))
    }

    /// Tries to convert the raw `publisher_id` into an enum which is useful for
    /// exhaustive pattern matching.
    ///
    /// # Errors
    /// This function returns an error if the `publisher_id` does not correspond with
    /// any known [`Publisher`].
    pub fn publisher(&self) -> crate::Result<Publisher> {
        Publisher::try_from(self.publisher_id)
            .map_err(|_| Error::conversion::<Publisher>(format!("{}", self.publisher_id)))
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

impl MboMsg {
    /// Tries to convert the raw order side to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`].
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format!("{:#02X}", self.side as u8)))
    }

    /// Tries to convert the raw event action to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`].
    pub fn action(&self) -> crate::Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#02X}", self.action as u8)))
    }

    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        if self.ts_recv == crate::UNDEF_TIMESTAMP {
            None
        } else {
            // u64::MAX is within maximum allowable range
            Some(time::OffsetDateTime::from_unix_timestamp_nanos(self.ts_recv as i128).unwrap())
        }
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
            .map_err(|_| Error::conversion::<Side>(format!("{:#02X}", self.side as u8)))
    }

    /// Tries to convert the raw event action to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`].
    pub fn action(&self) -> crate::Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#02X}", self.action as u8)))
    }

    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        if self.ts_recv == crate::UNDEF_TIMESTAMP {
            None
        } else {
            // u64::MAX is within maximum allowable range
            Some(time::OffsetDateTime::from_unix_timestamp_nanos(self.ts_recv as i128).unwrap())
        }
    }

    /// Parses the raw `ts_in_delta`—the delta of `ts_recv - ts_exchange_send`—into a duration.
    pub fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.ts_in_delta)
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
            .map_err(|_| Error::conversion::<Side>(format!("{:#02X}", self.side as u8)))
    }

    /// Tries to convert the raw event action to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`].
    pub fn action(&self) -> crate::Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#02X}", self.action as u8)))
    }

    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        if self.ts_recv == crate::UNDEF_TIMESTAMP {
            None
        } else {
            // u64::MAX is within maximum allowable range
            Some(time::OffsetDateTime::from_unix_timestamp_nanos(self.ts_recv as i128).unwrap())
        }
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
            .map_err(|_| Error::conversion::<Side>(format!("{:#02X}", self.side as u8)))
    }

    /// Tries to convert the raw event action to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`].
    pub fn action(&self) -> Result<Action> {
        Action::try_from(self.action as u8)
            .map_err(|_| Error::conversion::<Action>(format!("{:#02X}", self.action as u8)))
    }

    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        if self.ts_recv == crate::UNDEF_TIMESTAMP {
            None
        } else {
            // u64::MAX is within maximum allowable range
            Some(time::OffsetDateTime::from_unix_timestamp_nanos(self.ts_recv as i128).unwrap())
        }
    }

    /// Parses the raw `ts_in_delta`—the delta of `ts_recv - ts_exchange_send`—into a duration.
    pub fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.ts_in_delta)
    }
}

impl StatusMsg {
    /// Returns `group` as a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `group` contains invalid UTF-8.
    pub fn group(&self) -> Result<&str> {
        c_chars_to_str(&self.group)
    }
}

impl InstrumentDefMsg {
    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        if self.ts_recv == crate::UNDEF_TIMESTAMP {
            None
        } else {
            // u64::MAX is within maximum allowable range
            Some(time::OffsetDateTime::from_unix_timestamp_nanos(self.ts_recv as i128).unwrap())
        }
    }

    /// Parses the raw last eligible trade time into a datetime. Returns `None` if
    /// `expiration` contains the sentinel for a null timestamp.
    pub fn expiration(&self) -> Option<time::OffsetDateTime> {
        if self.expiration == crate::UNDEF_TIMESTAMP {
            None
        } else {
            // u64::MAX is within maximum allowable range
            Some(time::OffsetDateTime::from_unix_timestamp_nanos(self.expiration as i128).unwrap())
        }
    }

    /// Parses the raw time of instrument action into a datetime. Returns `None` if
    /// `activation` contains the sentinel for a null timestamp.
    pub fn activation(&self) -> Option<time::OffsetDateTime> {
        if self.activation == crate::UNDEF_TIMESTAMP {
            None
        } else {
            // u64::MAX is within maximum allowable range
            Some(time::OffsetDateTime::from_unix_timestamp_nanos(self.activation as i128).unwrap())
        }
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
            Error::conversion::<InstrumentClass>(format!("{:#02X}", self.instrument_class as u8))
        })
    }

    /// Tries to convert the raw matching algorithm used for the instrument to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `match_algorithm` field does not
    /// contain a valid [`MatchAlgorithm`].
    pub fn match_algorithm(&self) -> Result<MatchAlgorithm> {
        MatchAlgorithm::try_from(self.match_algorithm as u8).map_err(|_| {
            Error::conversion::<MatchAlgorithm>(format!("{:#02X}", self.match_algorithm as u8))
        })
    }
}

impl ImbalanceMsg {
    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        if self.ts_recv == crate::UNDEF_TIMESTAMP {
            None
        } else {
            // u64::MAX is within maximum allowable range
            Some(time::OffsetDateTime::from_unix_timestamp_nanos(self.ts_recv as i128).unwrap())
        }
    }
}

impl StatMsg {
    /// Parses the raw capture-server-received timestamp into a datetime. Returns `None`
    /// if `ts_recv` contains the sentinel for a null timestamp.
    pub fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        if self.ts_recv == crate::UNDEF_TIMESTAMP {
            None
        } else {
            // u64::MAX is within maximum allowable range
            Some(time::OffsetDateTime::from_unix_timestamp_nanos(self.ts_recv as i128).unwrap())
        }
    }

    /// Parses the raw reference timestamp of the statistic value into a datetime.
    /// Returns `None` if `ts_ref` contains the sentinel for a null timestamp.
    pub fn ts_ref(&self) -> Option<time::OffsetDateTime> {
        if self.ts_ref == crate::UNDEF_TIMESTAMP {
            None
        } else {
            // u64::MAX is within maximum allowable range
            Some(time::OffsetDateTime::from_unix_timestamp_nanos(self.ts_ref as i128).unwrap())
        }
    }

    /// Tries to convert the raw type of the statistic value to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `stat_type` field does not
    /// contain a valid [`StatType`].
    pub fn stat_type(&self) -> Result<StatType> {
        StatType::try_from(self.stat_type)
            .map_err(|_| Error::conversion::<StatType>(format!("{:02X}", self.stat_type)))
    }

    /// Tries to convert the raw `update_action` to an enum.
    ///
    /// # Errors
    /// This function returns an error if the `update_action` field does not
    /// contain a valid [`StatUpdateAction`].
    pub fn update_action(&self) -> Result<StatUpdateAction> {
        StatUpdateAction::try_from(self.update_action).map_err(|_| {
            Error::conversion::<StatUpdateAction>(format!("{:02X}", self.update_action))
        })
    }
}

impl ErrorMsg {
    /// Creates a new `ErrorMsg`.
    ///
    /// # Errors
    /// This function returns an error if `msg` is too long.
    pub fn new(ts_event: u64, msg: &str) -> Self {
        let mut error = Self {
            hd: RecordHeader::new::<Self>(rtype::ERROR, 0, 0, ts_event),
            err: [0; 64],
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
        if self.start_ts == crate::UNDEF_TIMESTAMP {
            None
        } else {
            // u64::MAX is within maximum allowable range
            Some(time::OffsetDateTime::from_unix_timestamp_nanos(self.start_ts as i128).unwrap())
        }
    }

    /// Parses the raw end of the mapping interval into a datetime. Returns `None` if
    /// `end_ts` contains the sentinel for a null timestamp.
    pub fn end_ts(&self) -> Option<time::OffsetDateTime> {
        if self.end_ts == crate::UNDEF_TIMESTAMP {
            None
        } else {
            // u64::MAX is within maximum allowable range
            Some(time::OffsetDateTime::from_unix_timestamp_nanos(self.end_ts as i128).unwrap())
        }
    }
}

impl SymbolMappingMsgV2 {
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
}

impl SystemMsg {
    const HEARTBEAT: &str = "Heartbeat";

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
            msg: str_to_c_chars(Self::HEARTBEAT).unwrap(),
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

impl<T: HasRType> HasRType for WithTsOut<T> {
    fn has_rtype(rtype: u8) -> bool {
        T::has_rtype(rtype)
    }

    fn header(&self) -> &RecordHeader {
        self.rec.header()
    }

    fn header_mut(&mut self) -> &mut RecordHeader {
        self.rec.header_mut()
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
