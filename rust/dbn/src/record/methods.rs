use std::fmt::Debug;

use crate::{
    enums::{ErrorCode, InstrumentClass, SystemCode},
    pretty::px_to_f64,
    SType,
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
            .map_err(|_| Error::conversion::<RType>(format_args!("{:#04X}", self.rtype)))
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

impl MboRec for MboMsg {
    fn order_id(&self) -> u64 {
        self.order_id
    }

    fn price(&self) -> i64 {
        self.price
    }

    fn size(&self) -> u32 {
        self.size
    }

    fn flags(&self) -> FlagSet {
        self.flags
    }

    fn channel_id(&self) -> u8 {
        self.channel_id
    }

    fn raw_action(&self) -> c_char {
        self.action
    }

    fn raw_side(&self) -> c_char {
        self.side
    }

    fn raw_ts_recv(&self) -> u64 {
        self.ts_recv
    }

    fn raw_ts_in_delta(&self) -> i32 {
        self.ts_in_delta
    }

    fn sequence(&self) -> u32 {
        self.sequence
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
            .map_err(|_| Error::conversion::<Publisher>(format_args!("{:#04X}", self.bid_pb)))
    }

    /// Parses the ask publisher into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `ask_pb` field does not
    /// contain a valid [`Publisher`].
    pub fn ask_pb(&self) -> crate::Result<Publisher> {
        Publisher::try_from(self.ask_pb)
            .map_err(|_| Error::conversion::<Publisher>(format_args!("{:#04X}", self.ask_pb)))
    }
}

impl TradeRec for TradeMsg {
    fn price(&self) -> i64 {
        self.price
    }

    fn size(&self) -> u32 {
        self.size
    }

    fn raw_action(&self) -> c_char {
        self.action
    }

    fn raw_side(&self) -> c_char {
        self.side
    }

    fn flags(&self) -> FlagSet {
        self.flags
    }

    fn raw_ts_recv(&self) -> u64 {
        self.ts_recv
    }

    fn raw_ts_in_delta(&self) -> i32 {
        self.ts_in_delta
    }

    fn sequence(&self) -> u32 {
        self.sequence
    }
}

impl Mbp1Rec for Mbp1Msg {
    fn price(&self) -> i64 {
        self.price
    }

    fn size(&self) -> u32 {
        self.size
    }

    fn raw_action(&self) -> c_char {
        self.action
    }

    fn raw_side(&self) -> c_char {
        self.side
    }

    fn flags(&self) -> FlagSet {
        self.flags
    }

    fn raw_ts_recv(&self) -> u64 {
        self.ts_recv
    }

    fn raw_ts_in_delta(&self) -> i32 {
        self.ts_in_delta
    }

    fn sequence(&self) -> u32 {
        self.sequence
    }

    fn levels(&self) -> &[BidAskPair; 1] {
        &self.levels
    }
}

impl Mbp10Rec for Mbp10Msg {
    fn price(&self) -> i64 {
        self.price
    }

    fn size(&self) -> u32 {
        self.size
    }

    fn raw_action(&self) -> c_char {
        self.action
    }

    fn raw_side(&self) -> c_char {
        self.side
    }

    fn flags(&self) -> FlagSet {
        self.flags
    }

    fn depth(&self) -> u8 {
        self.depth
    }

    fn raw_ts_recv(&self) -> u64 {
        self.ts_recv
    }

    fn raw_ts_in_delta(&self) -> i32 {
        self.ts_in_delta
    }

    fn sequence(&self) -> u32 {
        self.sequence
    }

    fn levels(&self) -> &[BidAskPair; 10] {
        &self.levels
    }
}

impl BboRec for BboMsg {
    fn price(&self) -> i64 {
        self.price
    }

    fn size(&self) -> u32 {
        self.size
    }

    fn raw_side(&self) -> c_char {
        self.side
    }

    fn flags(&self) -> FlagSet {
        self.flags
    }

    fn raw_ts_recv(&self) -> u64 {
        self.ts_recv
    }

    fn sequence(&self) -> u32 {
        self.sequence
    }

    fn levels(&self) -> &[BidAskPair; 1] {
        &self.levels
    }
}

impl Cmbp1Rec for Cmbp1Msg {
    fn price(&self) -> i64 {
        self.price
    }

    fn size(&self) -> u32 {
        self.size
    }

    fn raw_action(&self) -> c_char {
        self.action
    }

    fn raw_side(&self) -> c_char {
        self.side
    }

    fn flags(&self) -> FlagSet {
        self.flags
    }

    fn raw_ts_recv(&self) -> u64 {
        self.ts_recv
    }

    fn raw_ts_in_delta(&self) -> i32 {
        self.ts_in_delta
    }

    fn levels(&self) -> &[ConsolidatedBidAskPair; 1] {
        &self.levels
    }
}

impl CbboRec for CbboMsg {
    fn price(&self) -> i64 {
        self.price
    }

    fn size(&self) -> u32 {
        self.size
    }

    fn raw_side(&self) -> c_char {
        self.side
    }

    fn flags(&self) -> FlagSet {
        self.flags
    }

    fn raw_ts_recv(&self) -> u64 {
        self.ts_recv
    }

    fn levels(&self) -> &[ConsolidatedBidAskPair; 1] {
        &self.levels
    }
}

impl OhlcvRec for OhlcvMsg {
    fn open(&self) -> i64 {
        self.open
    }

    fn high(&self) -> i64 {
        self.high
    }

    fn low(&self) -> i64 {
        self.low
    }

    fn close(&self) -> i64 {
        self.close
    }

    fn volume(&self) -> u64 {
        self.volume
    }
}

impl StatusRec for StatusMsg {
    fn raw_ts_recv(&self) -> u64 {
        self.ts_recv
    }

    fn raw_action(&self) -> u16 {
        self.action
    }

    fn raw_reason(&self) -> u16 {
        self.reason
    }

    fn raw_trading_event(&self) -> u16 {
        self.trading_event
    }

    fn raw_is_trading(&self) -> c_char {
        self.is_trading
    }

    fn raw_is_quoting(&self) -> c_char {
        self.is_quoting
    }

    fn raw_is_short_sell_restricted(&self) -> c_char {
        self.is_short_sell_restricted
    }
}

impl InstrumentDefMsg {
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

    /// Parses the leg raw symbol into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `leg_raw_symbol` contains invalid UTF-8.
    pub fn leg_raw_symbol(&self) -> crate::Result<&str> {
        c_chars_to_str(&self.leg_raw_symbol)
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
        self.raw_instrument_id
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
        Some(self.leg_price)
    }

    fn leg_delta(&self) -> Option<i64> {
        Some(self.leg_delta)
    }

    fn user_defined_instrument(&self) -> Result<Option<UserDefinedInstrument>> {
        UserDefinedInstrument::try_from(self.user_defined_instrument as u8)
            .map(Some)
            .map_err(|_| {
                Error::conversion::<UserDefinedInstrument>(format_args!(
                    "{:#04X}",
                    self.user_defined_instrument as u8
                ))
            })
    }

    fn leg_instrument_class(&self) -> Result<Option<InstrumentClass>> {
        InstrumentClass::try_from(self.leg_instrument_class as u8)
            .map(Some)
            .map_err(|_| {
                Error::conversion::<InstrumentClass>(format_args!(
                    "{:#04X}",
                    self.leg_instrument_class as u8
                ))
            })
    }

    fn leg_side(&self) -> Result<Option<Side>> {
        Side::try_from(self.leg_side as u8)
            .map(Some)
            .map_err(|_| Error::conversion::<Side>(format_args!("{:#04X}", self.leg_side as u8)))
    }
}

impl ImbalanceRec for ImbalanceMsg {
    fn raw_ts_recv(&self) -> u64 {
        self.ts_recv
    }

    fn ref_price(&self) -> i64 {
        self.ref_price
    }

    fn cont_book_clr_price(&self) -> i64 {
        self.cont_book_clr_price
    }

    fn auct_interest_clr_price(&self) -> i64 {
        self.auct_interest_clr_price
    }

    fn paired_qty(&self) -> u32 {
        self.paired_qty
    }

    fn total_imbalance_qty(&self) -> u32 {
        self.total_imbalance_qty
    }

    fn auction_type(&self) -> c_char {
        self.auction_type
    }

    fn raw_side(&self) -> c_char {
        self.side
    }

    fn raw_auction_time(&self) -> Option<u64> {
        Some(self.auction_time)
    }

    fn ssr_filling_price(&self) -> Option<i64> {
        Some(self.ssr_filling_price)
    }

    fn ind_match_price(&self) -> Option<i64> {
        Some(self.ind_match_price)
    }

    fn upper_collar(&self) -> Option<i64> {
        Some(self.upper_collar)
    }

    fn lower_collar(&self) -> Option<i64> {
        Some(self.lower_collar)
    }

    fn unpaired_side(&self) -> Result<Option<Side>> {
        Side::try_from(self.unpaired_side as u8)
            .map(Some)
            .map_err(|_| {
                Error::conversion::<Side>(format_args!("{:#04X}", self.unpaired_side as u8))
            })
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
        self.quantity
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
}

impl ErrorRec for ErrorMsg {
    fn subscription_id(&self) -> u32 {
        u32::MAX
    }

    fn raw_schema(&self) -> u16 {
        u16::MAX
    }

    fn raw_code(&self) -> u8 {
        self.code
    }

    fn is_last(&self) -> u8 {
        self.is_last
    }

    fn err(&self) -> Result<&str> {
        c_chars_to_str(&self.err)
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
}

impl SymbolMappingRec for SymbolMappingMsg {
    fn raw_start_ts(&self) -> u64 {
        self.start_ts
    }

    fn raw_end_ts(&self) -> u64 {
        self.end_ts
    }

    fn raw_stype_in(&self) -> u8 {
        self.stype_in
    }

    fn raw_stype_out(&self) -> u8 {
        self.stype_out
    }

    fn stype_in_symbol(&self) -> Result<&str> {
        c_chars_to_str(&self.stype_in_symbol)
    }

    fn stype_out_symbol(&self) -> Result<&str> {
        c_chars_to_str(&self.stype_out_symbol)
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
}

impl SystemRec for SystemMsg {
    fn subscription_id(&self) -> u32 {
        u32::MAX
    }

    fn raw_schema(&self) -> u16 {
        u16::MAX
    }

    fn raw_code(&self) -> u8 {
        self.code
    }

    fn msg(&self) -> Result<&str> {
        c_chars_to_str(&self.msg)
    }
}
