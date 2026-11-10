use std::os::raw::c_char;

use crate::{
    pretty::px_to_f64,
    record::{c_chars_to_str, ts_to_dt},
    BboRec, BidAskPair, CbboRec, Cmbp1Rec, ConsolidatedBidAskPair, Error, ErrorRec, FlagSet,
    InstrumentClass, MatchAlgorithm, MboRec, Mbp10Rec, Mbp1Rec, OhlcvRec, Result, SType,
    SecurityUpdateAction, Side, StatRec, StatusRec, SystemRec, TradeRec,
};

use super::*;

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
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
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
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
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
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
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
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    pub fn side(&self) -> crate::Result<Side> {
        Side::try_from(self.side as u8)
            .map_err(|_| Error::conversion::<Side>(format_args!("{:#04X}", self.side as u8)))
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
        self.channel_id
    }

    fn raw_update_action(&self) -> u8 {
        self.update_action
    }

    fn stat_flags(&self) -> u8 {
        self.stat_flags
    }
}

impl ErrorRec for ErrorMsg {
    fn subscription_id(&self) -> u32 {
        self.subscription_id
    }

    fn raw_schema(&self) -> u16 {
        self.schema
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

impl SystemRec for SystemMsg {
    fn subscription_id(&self) -> u32 {
        self.subscription_id
    }

    fn raw_schema(&self) -> u16 {
        self.schema
    }

    fn raw_code(&self) -> u8 {
        self.code
    }

    fn msg(&self) -> Result<&str> {
        c_chars_to_str(&self.msg)
    }
}
