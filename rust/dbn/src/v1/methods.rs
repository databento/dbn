use std::os::raw::c_char;

use crate::{
    record::{c_chars_to_str, str_to_c_chars, ts_to_dt},
    rtype, Error, InstrumentClass, MatchAlgorithm, RecordHeader, Result,
};

use super::{ErrorMsg, InstrumentDefMsg, SymbolMappingMsg, SystemMsg};

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

    /// Returns the [Security type](https://databento.com/docs/schemas-and-data-formats/instrument-definitions#security-type)
    /// of the instrument, e.g. FUT for future or future spread as a `&str`.
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

impl ErrorMsg {
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
        ts_to_dt(self.start_ts)
    }

    /// Parses the raw end of the mapping interval into a datetime. Returns `None` if
    /// `end_ts` contains the sentinel for a null timestamp.
    pub fn end_ts(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.end_ts)
    }
}

impl SystemMsg {
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
            msg: str_to_c_chars(crate::SystemMsg::HEARTBEAT).unwrap(),
        }
    }

    /// Checks whether the message is a heartbeat from the gateway.
    pub fn is_heartbeat(&self) -> bool {
        self.msg()
            .map(|msg| msg == crate::SystemMsg::HEARTBEAT)
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
