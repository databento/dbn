//! Accessor traits for reading records the same way across DBN versions.

use std::os::raw::c_char;

use num_enum::TryFromPrimitive;

use crate::{
    pretty::px_to_f64, record::ts_to_dt, Action, BidAskPair, ConsolidatedBidAskPair, Error,
    ErrorCode, FlagSet, InstrumentClass, MatchAlgorithm, Record, Result, SType, Schema,
    SecurityUpdateAction, Side, StatType, StatUpdateAction, StatusAction, StatusReason, SystemCode,
    SystemMsg, TradingEvent, TriState, UserDefinedInstrument,
};

/// Accessors for the fields of [`MboMsg`](crate::MboMsg) records from any DBN
/// version.
pub trait MboRec: Record {
    /// The order ID assigned at the venue.
    fn order_id(&self) -> u64;

    /// The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    fn price(&self) -> i64;

    /// The order quantity.
    fn size(&self) -> u32;

    /// A bit field indicating event end, message characteristics, and data quality.
    /// See [`flags`](crate::flags) for possible values.
    fn flags(&self) -> FlagSet;

    /// The channel ID assigned by Databento as an incrementing integer starting at zero.
    fn channel_id(&self) -> u8;

    /// The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R** book, **T**rade, **F**ill, or **N**one.
    ///
    /// See [Action](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#action).
    fn raw_action(&self) -> c_char;

    /// The side that initiates the event. Can be **A**sk for a sell order (or sell aggressor in
    /// a trade), **B**id for a buy order (or buy aggressor in a trade), or **N**one where no side is specified.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    fn raw_side(&self) -> c_char;

    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    fn raw_ts_recv(&self) -> u64;

    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    fn raw_ts_in_delta(&self) -> i32;

    /// The message sequence number assigned at the venue.
    fn sequence(&self) -> u32;

    /// Converts the order price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn price_f64(&self) -> f64 {
        px_to_f64(self.price())
    }

    /// Parses the action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`](crate::Action).
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    fn action(&self) -> Result<Action> {
        let raw = self.raw_action() as u8;
        Action::try_from(raw).map_err(|_| Error::conversion::<Action>(format_args!("{raw:#04X}")))
    }

    /// Parses the side that initiates the event into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`](crate::Side).
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    fn side(&self) -> Result<Side> {
        let raw = self.raw_side() as u8;
        Side::try_from(raw).map_err(|_| Error::conversion::<Side>(format_args!("{raw:#04X}")))
    }

    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.raw_ts_recv())
    }

    /// Parses the difference between `ts_recv` and the matching-engine-sending timestamp into a duration.
    fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.raw_ts_in_delta())
    }
}

/// Accessors for the fields of [`TradeMsg`](crate::TradeMsg) records from any DBN
/// version.
pub trait TradeRec: Record {
    /// The trade price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    fn price(&self) -> i64;

    /// The order quantity.
    fn size(&self) -> u32;

    /// The event action. Always **T**rade in the trades schema.
    ///
    /// See [Action](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#action).
    fn raw_action(&self) -> c_char;

    /// The side that initiates the trade. Can be **A**sk for a sell aggressor in a trade, **B**id for a buy aggressor in a trade, or **N**one where no side is specified.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    fn raw_side(&self) -> c_char;

    /// A bit field indicating event end, message characteristics, and data quality.
    /// See [`flags`](crate::flags) for possible values.
    fn flags(&self) -> FlagSet;

    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    fn raw_ts_recv(&self) -> u64;

    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    fn raw_ts_in_delta(&self) -> i32;

    /// The message sequence number assigned at the venue.
    fn sequence(&self) -> u32;

    /// Converts the price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn price_f64(&self) -> f64 {
        px_to_f64(self.price())
    }

    /// Parses the action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`](crate::Action).
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    fn action(&self) -> Result<Action> {
        let raw = self.raw_action() as u8;
        Action::try_from(raw).map_err(|_| Error::conversion::<Action>(format_args!("{raw:#04X}")))
    }

    /// Parses the side into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`](crate::Side).
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    fn side(&self) -> Result<Side> {
        let raw = self.raw_side() as u8;
        Side::try_from(raw).map_err(|_| Error::conversion::<Side>(format_args!("{raw:#04X}")))
    }

    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.raw_ts_recv())
    }

    /// Parses the difference between `ts_recv` and the matching-engine-sending timestamp into a duration.
    fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.raw_ts_in_delta())
    }
}

/// Accessors for the fields of [`Mbp1Msg`](crate::Mbp1Msg) records from any DBN
/// version.
pub trait Mbp1Rec: Record {
    /// The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    fn price(&self) -> i64;

    /// The order quantity.
    fn size(&self) -> u32;

    /// The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R** book, or **T**rade.
    ///
    /// See [Action](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#action).
    fn raw_action(&self) -> c_char;

    /// The side that initiates the event. Can be **A**sk for a sell order (or sell aggressor in
    /// a trade), **B**id for a buy order (or buy aggressor in a trade), or **N**one where no side is specified.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    fn raw_side(&self) -> c_char;

    /// A bit field indicating event end, message characteristics, and data quality.
    /// See [`flags`](crate::flags) for possible values.
    fn flags(&self) -> FlagSet;

    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    fn raw_ts_recv(&self) -> u64;

    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    fn raw_ts_in_delta(&self) -> i32;

    /// The message sequence number assigned at the venue.
    fn sequence(&self) -> u32;

    /// The top of the order book.
    fn levels(&self) -> &[BidAskPair; 1];

    /// Converts the order price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn price_f64(&self) -> f64 {
        px_to_f64(self.price())
    }

    /// Parses the action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`](crate::Action).
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    fn action(&self) -> Result<Action> {
        let raw = self.raw_action() as u8;
        Action::try_from(raw).map_err(|_| Error::conversion::<Action>(format_args!("{raw:#04X}")))
    }

    /// Parses the side that initiates the event into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`](crate::Side).
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    fn side(&self) -> Result<Side> {
        let raw = self.raw_side() as u8;
        Side::try_from(raw).map_err(|_| Error::conversion::<Side>(format_args!("{raw:#04X}")))
    }

    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.raw_ts_recv())
    }

    /// Parses the difference between `ts_recv` and the matching-engine-sending timestamp into a duration.
    fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.raw_ts_in_delta())
    }
}

/// Accessors for the fields of [`Mbp10Msg`](crate::Mbp10Msg) records from any DBN
/// version.
pub trait Mbp10Rec: Record {
    /// The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    fn price(&self) -> i64;

    /// The order quantity.
    fn size(&self) -> u32;

    /// The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R** book, or **T**rade.
    ///
    /// See [Action](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#action).
    fn raw_action(&self) -> c_char;

    /// The side that initiates the event. Can be **A**sk for a sell order (or sell aggressor in
    /// a trade), **B**id for a buy order (or buy aggressor in a trade), or **N**one where no side is specified.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    fn raw_side(&self) -> c_char;

    /// A bit field indicating event end, message characteristics, and data quality.
    /// See [`flags`](crate::flags) for possible values.
    fn flags(&self) -> FlagSet;

    /// The book level where the update event occurred.
    fn depth(&self) -> u8;

    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    fn raw_ts_recv(&self) -> u64;

    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    fn raw_ts_in_delta(&self) -> i32;

    /// The message sequence number assigned at the venue.
    fn sequence(&self) -> u32;

    /// The top 10 levels of the order book.
    fn levels(&self) -> &[BidAskPair; 10];

    /// Converts the order price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn price_f64(&self) -> f64 {
        px_to_f64(self.price())
    }

    /// Parses the action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`](crate::Action).
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    fn action(&self) -> Result<Action> {
        let raw = self.raw_action() as u8;
        Action::try_from(raw).map_err(|_| Error::conversion::<Action>(format_args!("{raw:#04X}")))
    }

    /// Parses the side that initiates the event into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`](crate::Side).
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    fn side(&self) -> Result<Side> {
        let raw = self.raw_side() as u8;
        Side::try_from(raw).map_err(|_| Error::conversion::<Side>(format_args!("{raw:#04X}")))
    }

    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.raw_ts_recv())
    }

    /// Parses the difference between `ts_recv` and the matching-engine-sending timestamp into a duration.
    fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.raw_ts_in_delta())
    }
}

/// Accessors for the fields of [`BboMsg`](crate::BboMsg) records from any DBN
/// version.
pub trait BboRec: Record {
    /// The last trade price price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001. Will be [`UNDEF_PRICE`](crate::UNDEF_PRICE) if there was no last trade
    /// in the session.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    fn price(&self) -> i64;

    /// The quantity of the last trade.
    fn size(&self) -> u32;

    /// The side that initiated the last trade. Can be **A**sk for a sell order (or sell
    /// aggressor in a trade), **B**id for a buy order (or buy aggressor in a trade), or
    /// **N**one where no side is specified.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    fn raw_side(&self) -> c_char;

    /// A bit field indicating event end, message characteristics, and data quality.
    /// See [`flags`](crate::flags) for possible values.
    fn flags(&self) -> FlagSet;

    /// The end timestamp of the interval capture-server-received timestamp expressed as the
    /// number of nanoseconds since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    fn raw_ts_recv(&self) -> u64;

    /// The message sequence number assigned at the venue of the last update.
    fn sequence(&self) -> u32;

    /// The top of the order book.
    fn levels(&self) -> &[BidAskPair; 1];

    /// Converts the last trade price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn price_f64(&self) -> f64 {
        px_to_f64(self.price())
    }

    /// Parses the side that initiated the last trade into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`](crate::Side).
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    fn side(&self) -> Result<Side> {
        let raw = self.raw_side() as u8;
        Side::try_from(raw).map_err(|_| Error::conversion::<Side>(format_args!("{raw:#04X}")))
    }

    /// Parses the end timestamp of the interval capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.raw_ts_recv())
    }
}

/// Accessors for the fields of [`Cmbp1Msg`](crate::Cmbp1Msg) records from any DBN
/// version.
pub trait Cmbp1Rec: Record {
    /// The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    fn price(&self) -> i64;

    /// The order quantity.
    fn size(&self) -> u32;

    /// The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R** book, or **T**rade.
    ///
    /// See [Action](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#action).
    fn raw_action(&self) -> c_char;

    /// The side that initiates the event. Can be **A**sk for a sell order (or sell aggressor in
    /// a trade), **B**id for a buy order (or buy aggressor in a trade), or **N**one where no side is specified.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    fn raw_side(&self) -> c_char;

    /// A bit field indicating event end, message characteristics, and data quality.
    /// See [`flags`](crate::flags) for possible values.
    fn flags(&self) -> FlagSet;

    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    fn raw_ts_recv(&self) -> u64;

    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    fn raw_ts_in_delta(&self) -> i32;

    /// The top of the order book.
    fn levels(&self) -> &[ConsolidatedBidAskPair; 1];

    /// Converts the order price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn price_f64(&self) -> f64 {
        px_to_f64(self.price())
    }

    /// Parses the action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`Action`](crate::Action).
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    fn action(&self) -> Result<Action> {
        let raw = self.raw_action() as u8;
        Action::try_from(raw).map_err(|_| Error::conversion::<Action>(format_args!("{raw:#04X}")))
    }

    /// Parses the side that initiates the event into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`](crate::Side).
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    fn side(&self) -> Result<Side> {
        let raw = self.raw_side() as u8;
        Side::try_from(raw).map_err(|_| Error::conversion::<Side>(format_args!("{raw:#04X}")))
    }

    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.raw_ts_recv())
    }

    /// Parses the difference between `ts_recv` and the matching-engine-sending timestamp into a duration.
    fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.raw_ts_in_delta())
    }
}

/// Accessors for the fields of [`CbboMsg`](crate::CbboMsg) records from any DBN
/// version.
pub trait CbboRec: Record {
    /// The last trade price price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001. Will be [`UNDEF_PRICE`](crate::UNDEF_PRICE) if there was no last trade
    /// in the session.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    fn price(&self) -> i64;

    /// The quantity of the last trade.
    fn size(&self) -> u32;

    /// The side that initiated the last trade. Can be **A**sk for a sell order (or sell
    /// aggressor in a trade), **B**id for a buy order (or buy aggressor in a trade), or
    /// **N**one where no side is specified.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    fn raw_side(&self) -> c_char;

    /// A bit field indicating event end, message characteristics, and data quality.
    /// See [`flags`](crate::flags) for possible values.
    fn flags(&self) -> FlagSet;

    /// The end timestamp of the interval capture-server-received timestamp expressed as the
    /// number of nanoseconds since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    fn raw_ts_recv(&self) -> u64;

    /// The top of the order book.
    fn levels(&self) -> &[ConsolidatedBidAskPair; 1];

    /// Converts the last trade price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn price_f64(&self) -> f64 {
        px_to_f64(self.price())
    }

    /// Parses the side that initiated the last trade into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`](crate::Side).
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    fn side(&self) -> Result<Side> {
        let raw = self.raw_side() as u8;
        Side::try_from(raw).map_err(|_| Error::conversion::<Side>(format_args!("{raw:#04X}")))
    }

    /// Parses the end timestamp of the interval capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.raw_ts_recv())
    }
}

/// Accessors for the fields of [`OhlcvMsg`](crate::OhlcvMsg) records from any DBN
/// version.
pub trait OhlcvRec: Record {
    /// The open price for the bar where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    fn open(&self) -> i64;

    /// The high price for the bar where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    fn high(&self) -> i64;

    /// The low price for the bar where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    fn low(&self) -> i64;

    /// The close price for the bar where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000
    /// or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    fn close(&self) -> i64;

    /// The total volume traded during the aggregation period.
    fn volume(&self) -> u64;

    /// Converts the open price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn open_f64(&self) -> f64 {
        px_to_f64(self.open())
    }

    /// Converts the high price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn high_f64(&self) -> f64 {
        px_to_f64(self.high())
    }

    /// Converts the low price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn low_f64(&self) -> f64 {
        px_to_f64(self.low())
    }

    /// Converts the close price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn close_f64(&self) -> f64 {
        px_to_f64(self.close())
    }
}

/// Accessors for the fields of [`StatusMsg`](crate::StatusMsg) records from any DBN
/// version.
pub trait StatusRec: Record {
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    fn raw_ts_recv(&self) -> u64;

    /// The type of status change.
    fn raw_action(&self) -> u16;

    /// Additional details about the cause of the status change.
    fn raw_reason(&self) -> u16;

    /// Further information about the status change and its effect on trading.
    fn raw_trading_event(&self) -> u16;

    /// The best-efforts state of trading in the instrument, either `Y`, `N` or `~`.
    fn raw_is_trading(&self) -> c_char;

    /// The best-efforts state of quoting in the instrument, either `Y`, `N` or `~`.
    fn raw_is_quoting(&self) -> c_char;

    /// The best-efforts state of short sell restrictions for the instrument (if applicable), either `Y`, `N` or `~`.
    fn raw_is_short_sell_restricted(&self) -> c_char;

    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.raw_ts_recv())
    }

    /// Parses the action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `action` field does not
    /// contain a valid [`StatusAction`](crate::StatusAction).
    fn action(&self) -> Result<StatusAction> {
        let raw = self.raw_action();
        StatusAction::try_from(raw)
            .map_err(|_| Error::conversion::<StatusAction>(format_args!("{raw:#04X}")))
    }

    /// Parses the reason into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `reason` field does not
    /// contain a valid [`StatusReason`](crate::StatusReason).
    fn reason(&self) -> Result<StatusReason> {
        let raw = self.raw_reason();
        StatusReason::try_from(raw)
            .map_err(|_| Error::conversion::<StatusReason>(format_args!("{raw:#04X}")))
    }

    /// Parses the trading event into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `trading_event` field does not
    /// contain a valid [`TradingEvent`](crate::TradingEvent).
    fn trading_event(&self) -> Result<TradingEvent> {
        let raw = self.raw_trading_event();
        TradingEvent::try_from(raw)
            .map_err(|_| Error::conversion::<TradingEvent>(format_args!("{raw:#04X}")))
    }

    /// Parses the trading state into an `Option<bool>` where `None` indicates
    /// a value is not applicable or available.
    fn is_trading(&self) -> Option<bool> {
        TriState::try_from_primitive(self.raw_is_trading() as c_char as u8)
            .map(Option::<bool>::from)
            .unwrap_or_default()
    }

    /// Parses the quoting state into an `Option<bool>` where `None` indicates
    /// a value is not applicable or available.
    fn is_quoting(&self) -> Option<bool> {
        TriState::try_from_primitive(self.raw_is_quoting() as c_char as u8)
            .map(Option::<bool>::from)
            .unwrap_or_default()
    }

    /// Parses the short selling state into an `Option<bool>` where `None` indicates
    /// a value is not applicable or available.
    fn is_short_sell_restricted(&self) -> Option<bool> {
        TriState::try_from_primitive(self.raw_is_short_sell_restricted() as c_char as u8)
            .map(Option::<bool>::from)
            .unwrap_or_default()
    }
}

/// Accessors for the fields of [`InstrumentDefMsg`](crate::InstrumentDefMsg) records from any DBN
/// version.
///
/// The version 4 struct holds only some of its fields, so in version 4 a
/// [`ResolvedInstrumentDefRef`](crate::v4::ResolvedInstrumentDefRef) implements this trait.
pub trait InstrumentDefRec: Record {
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    fn raw_ts_recv(&self) -> u64;

    /// The minimum constant tick for the instrument where every 1 unit corresponds to 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    fn min_price_increment(&self) -> i64;

    /// The multiplier to convert the venue's display price to the conventional price where every
    /// 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    fn display_factor(&self) -> i64;

    /// The instrument ID assigned by the publisher. May be the same as `instrument_id`.
    ///
    /// See [Instrument identifiers](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers)
    fn raw_instrument_id(&self) -> u64;

    /// The minimum quantity required for a round lot of the instrument. Multiples of this
    /// quantity are also round lots.
    fn min_lot_size_round_lot(&self) -> i32;

    /// Indicates if the instrument definition has been added, modified, or deleted.
    fn raw_security_update_action(&self) -> c_char;

    /// The classification of the instrument.
    ///
    /// See [Instrument class](https://databento.com/docs/schemas-and-data-formats/instrument-definitions#instrument-class).
    fn raw_instrument_class(&self) -> c_char;

    /// The matching algorithm used for the instrument, typically **F**IFO.
    ///
    /// See [Matching algorithm](https://databento.com/docs/schemas-and-data-formats/instrument-definitions#matching-algorithm).
    fn raw_match_algorithm(&self) -> c_char;

    /// The channel ID assigned by Databento as an incrementing integer starting at zero.
    fn channel_id(&self) -> u8;

    /// Parses the raw symbol into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if the record is too short to contain
    /// `raw_symbol` or it contains invalid UTF-8.
    fn raw_symbol(&self) -> Result<&str>;

    /// The last eligible trade time expressed as the number of nanoseconds since the
    /// UNIX epoch, or `None` if the record doesn't hold `expiration`.
    fn raw_expiration(&self) -> Option<u64>;

    /// Parses the last eligible trade time into a datetime. Returns `None` if the
    /// record doesn't hold `expiration` or it contains the sentinel for a null
    /// timestamp.
    fn expiration(&self) -> Option<time::OffsetDateTime> {
        self.raw_expiration().and_then(ts_to_dt)
    }

    /// The time of instrument activation expressed as the number of nanoseconds since the
    /// UNIX epoch, or `None` if the record doesn't hold `activation`.
    fn raw_activation(&self) -> Option<u64>;

    /// Parses the time of instrument activation into a datetime. Returns `None` if the
    /// record doesn't hold `activation` or it contains the sentinel for a null
    /// timestamp.
    fn activation(&self) -> Option<time::OffsetDateTime> {
        self.raw_activation().and_then(ts_to_dt)
    }

    /// The high limit price where every 1 unit corresponds to 1e-9, or `None`
    /// if the record doesn't hold `high_limit_price`.
    fn high_limit_price(&self) -> Option<i64>;

    /// Converts the high limit price to a floating point, or `None` if the
    /// record doesn't hold `high_limit_price`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn high_limit_price_f64(&self) -> Option<f64> {
        self.high_limit_price().map(px_to_f64)
    }

    /// The low limit price where every 1 unit corresponds to 1e-9, or `None`
    /// if the record doesn't hold `low_limit_price`.
    fn low_limit_price(&self) -> Option<i64>;

    /// Converts the low limit price to a floating point, or `None` if the
    /// record doesn't hold `low_limit_price`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn low_limit_price_f64(&self) -> Option<f64> {
        self.low_limit_price().map(px_to_f64)
    }

    /// The differential value for price banding where every 1 unit corresponds to 1e-9, or `None`
    /// if the record doesn't hold `max_price_variation`.
    fn max_price_variation(&self) -> Option<i64>;

    /// Converts the differential value for price banding to a floating point, or `None` if the
    /// record doesn't hold `max_price_variation`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn max_price_variation_f64(&self) -> Option<f64> {
        self.max_price_variation().map(px_to_f64)
    }

    /// The contract size for each instrument where every 1 unit corresponds to 1e-9, or `None`
    /// if the record doesn't hold `unit_of_measure_qty`.
    fn unit_of_measure_qty(&self) -> Option<i64>;

    /// Converts the contract size for each instrument to a floating point, or `None` if the
    /// record doesn't hold `unit_of_measure_qty`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn unit_of_measure_qty_f64(&self) -> Option<f64> {
        self.unit_of_measure_qty().map(px_to_f64)
    }

    /// The min price increment amount where every 1 unit corresponds to 1e-9, or `None`
    /// if the record doesn't hold `min_price_increment_amount`.
    fn min_price_increment_amount(&self) -> Option<i64>;

    /// Converts the min price increment amount to a floating point, or `None` if the
    /// record doesn't hold `min_price_increment_amount`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn min_price_increment_amount_f64(&self) -> Option<f64> {
        self.min_price_increment_amount().map(px_to_f64)
    }

    /// The price ratio where every 1 unit corresponds to 1e-9, or `None`
    /// if the record doesn't hold `price_ratio`.
    fn price_ratio(&self) -> Option<i64>;

    /// Converts the price ratio to a floating point, or `None` if the
    /// record doesn't hold `price_ratio`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn price_ratio_f64(&self) -> Option<f64> {
        self.price_ratio().map(px_to_f64)
    }

    /// The strike price where every 1 unit corresponds to 1e-9, or `None`
    /// if the record doesn't hold `strike_price`.
    fn strike_price(&self) -> Option<i64>;

    /// Converts the strike price to a floating point, or `None` if the
    /// record doesn't hold `strike_price`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn strike_price_f64(&self) -> Option<f64> {
        self.strike_price().map(px_to_f64)
    }

    /// The leg price where every 1 unit corresponds to 1e-9, or `None`
    /// if the record doesn't hold `leg_price`.
    fn leg_price(&self) -> Option<i64>;

    /// Converts the leg price to a floating point, or `None` if the
    /// record doesn't hold `leg_price`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn leg_price_f64(&self) -> Option<f64> {
        self.leg_price().map(px_to_f64)
    }

    /// The leg delta where every 1 unit corresponds to 1e-9, or `None`
    /// if the record doesn't hold `leg_delta`.
    fn leg_delta(&self) -> Option<i64>;

    /// Converts the leg delta to a floating point, or `None` if the
    /// record doesn't hold `leg_delta`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn leg_delta_f64(&self) -> Option<f64> {
        self.leg_delta().map(px_to_f64)
    }

    /// Parses the user-defined instrument flag into an enum, or `None` if the record
    /// doesn't hold `user_defined_instrument`.
    ///
    /// # Errors
    /// This function returns an error if `user_defined_instrument` doesn't contain a valid
    /// [`UserDefinedInstrument`](crate::UserDefinedInstrument).
    fn user_defined_instrument(&self) -> Result<Option<UserDefinedInstrument>>;

    /// Parses the leg instrument class into an enum, or `None` if the record
    /// doesn't hold `leg_instrument_class`.
    ///
    /// # Errors
    /// This function returns an error if `leg_instrument_class` doesn't contain a valid
    /// [`InstrumentClass`](crate::InstrumentClass).
    fn leg_instrument_class(&self) -> Result<Option<InstrumentClass>>;

    /// Parses the leg side into an enum, or `None` if the record
    /// doesn't hold `leg_side`.
    ///
    /// # Errors
    /// This function returns an error if `leg_side` doesn't contain a valid
    /// [`Side`](crate::Side).
    fn leg_side(&self) -> Result<Option<Side>>;

    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.raw_ts_recv())
    }

    /// Converts the minimum constant tick to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn min_price_increment_f64(&self) -> f64 {
        px_to_f64(self.min_price_increment())
    }

    /// Converts the display factor to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn display_factor_f64(&self) -> f64 {
        px_to_f64(self.display_factor())
    }

    /// Parses the security update action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `security_update_action` field does not
    /// contain a valid [`SecurityUpdateAction`](crate::SecurityUpdateAction).
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    fn security_update_action(&self) -> Result<SecurityUpdateAction> {
        let raw = self.raw_security_update_action() as u8;
        SecurityUpdateAction::try_from(raw)
            .map_err(|_| Error::conversion::<SecurityUpdateAction>(format_args!("{raw:#04X}")))
    }

    /// Parses the instrument class into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `instrument_class` field does not
    /// contain a valid [`InstrumentClass`](crate::InstrumentClass).
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    fn instrument_class(&self) -> Result<InstrumentClass> {
        let raw = self.raw_instrument_class() as u8;
        InstrumentClass::try_from(raw)
            .map_err(|_| Error::conversion::<InstrumentClass>(format_args!("{raw:#04X}")))
    }

    /// Parses the match algorithm into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `match_algorithm` field does not
    /// contain a valid [`MatchAlgorithm`](crate::MatchAlgorithm).
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    fn match_algorithm(&self) -> Result<MatchAlgorithm> {
        let raw = self.raw_match_algorithm() as u8;
        MatchAlgorithm::try_from(raw)
            .map_err(|_| Error::conversion::<MatchAlgorithm>(format_args!("{raw:#04X}")))
    }
}

/// Accessors for the fields of [`ImbalanceMsg`](crate::ImbalanceMsg) records from any DBN
/// version.
///
/// The version 4 struct holds only some of its fields, so in version 4 a
/// [`ResolvedImbalanceRef`](crate::v4::ResolvedImbalanceRef) implements this trait.
pub trait ImbalanceRec: Record {
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    fn raw_ts_recv(&self) -> u64;

    /// The price at which the imbalance shares are calculated, where every 1 unit corresponds
    /// to 1e-9, i.e. 1/1,000,000,000 or 0.000000001. Will be [`UNDEF_PRICE`](crate::UNDEF_PRICE)
    /// when unused.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    fn ref_price(&self) -> i64;

    /// The hypothetical auction-clearing price for both cross and continuous orders where every
    /// 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or 0.000000001.
    /// Will be [`UNDEF_PRICE`](crate::UNDEF_PRICE) when unused.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    fn cont_book_clr_price(&self) -> i64;

    /// The hypothetical auction-clearing price for cross orders only where every 1 unit corresponds
    /// to 1e-9, i.e. 1/1,000,000,000 or 0.000000001. Will be [`UNDEF_PRICE`](crate::UNDEF_PRICE) when unused.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    fn auct_interest_clr_price(&self) -> i64;

    /// The quantity of shares that are eligible to be matched at [`ref_price`](Self::ref_price).
    /// Will be [`UNDEF_ORDER_SIZE`](crate::UNDEF_ORDER_SIZE) when unused.
    fn paired_qty(&self) -> u32;

    /// The quantity of shares that are not paired at [`ref_price`](Self::ref_price).
    /// Will be [`UNDEF_ORDER_SIZE`](crate::UNDEF_ORDER_SIZE) when not used.
    fn total_imbalance_qty(&self) -> u32;

    /// Venue-specific character code indicating the auction type. Will be `~` when unused.
    ///
    /// Refer to the [venue-specific documentation](https://databento.com/docs/venues-and-datasets).
    fn auction_type(&self) -> c_char;

    /// The market side of the [`total_imbalance_qty`](Self::total_imbalance_qty).
    /// Can be **A**sk, **B**id, or **N**one.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    fn raw_side(&self) -> c_char;

    /// The auction time expressed as the number of nanoseconds since the
    /// UNIX epoch, or `None` if the record doesn't hold `auction_time`.
    fn raw_auction_time(&self) -> Option<u64>;

    /// Parses the auction time into a datetime. Returns `None` if the
    /// record doesn't hold `auction_time` or it contains the sentinel for a null
    /// timestamp.
    fn auction_time(&self) -> Option<time::OffsetDateTime> {
        self.raw_auction_time().and_then(ts_to_dt)
    }

    /// The ssr filling price where every 1 unit corresponds to 1e-9, or `None`
    /// if the record doesn't hold `ssr_filling_price`.
    fn ssr_filling_price(&self) -> Option<i64>;

    /// Converts the ssr filling price to a floating point, or `None` if the
    /// record doesn't hold `ssr_filling_price`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn ssr_filling_price_f64(&self) -> Option<f64> {
        self.ssr_filling_price().map(px_to_f64)
    }

    /// The ind match price where every 1 unit corresponds to 1e-9, or `None`
    /// if the record doesn't hold `ind_match_price`.
    fn ind_match_price(&self) -> Option<i64>;

    /// Converts the ind match price to a floating point, or `None` if the
    /// record doesn't hold `ind_match_price`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn ind_match_price_f64(&self) -> Option<f64> {
        self.ind_match_price().map(px_to_f64)
    }

    /// The upper collar where every 1 unit corresponds to 1e-9, or `None`
    /// if the record doesn't hold `upper_collar`.
    fn upper_collar(&self) -> Option<i64>;

    /// Converts the upper collar to a floating point, or `None` if the
    /// record doesn't hold `upper_collar`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn upper_collar_f64(&self) -> Option<f64> {
        self.upper_collar().map(px_to_f64)
    }

    /// The lower collar where every 1 unit corresponds to 1e-9, or `None`
    /// if the record doesn't hold `lower_collar`.
    fn lower_collar(&self) -> Option<i64>;

    /// Converts the lower collar to a floating point, or `None` if the
    /// record doesn't hold `lower_collar`.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn lower_collar_f64(&self) -> Option<f64> {
        self.lower_collar().map(px_to_f64)
    }

    /// Parses the unpaired side into an enum, or `None` if the record
    /// doesn't hold `unpaired_side`.
    ///
    /// # Errors
    /// This function returns an error if `unpaired_side` doesn't contain a valid
    /// [`Side`](crate::Side).
    fn unpaired_side(&self) -> Result<Option<Side>>;

    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.raw_ts_recv())
    }

    /// Converts the ref price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn ref_price_f64(&self) -> f64 {
        px_to_f64(self.ref_price())
    }

    /// Converts the cont book clr price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn cont_book_clr_price_f64(&self) -> f64 {
        px_to_f64(self.cont_book_clr_price())
    }

    /// Converts the auct interest clr price to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn auct_interest_clr_price_f64(&self) -> f64 {
        px_to_f64(self.auct_interest_clr_price())
    }

    /// Parses the side into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `side` field does not
    /// contain a valid [`Side`](crate::Side).
    #[allow(
        clippy::unnecessary_cast,
        reason = "c_char is u8 on some targets like aarch64 Linux"
    )]
    fn side(&self) -> Result<Side> {
        let raw = self.raw_side() as u8;
        Side::try_from(raw).map_err(|_| Error::conversion::<Side>(format_args!("{raw:#04X}")))
    }
}

/// Accessors for the fields of [`StatMsg`](crate::StatMsg) records from any DBN
/// version.
pub trait StatRec: Record {
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    fn raw_ts_recv(&self) -> u64;

    /// The reference timestamp of the statistic value expressed as the number of
    /// nanoseconds since the UNIX epoch. Will be [`UNDEF_TIMESTAMP`](crate::UNDEF_TIMESTAMP) when
    /// unused.
    fn raw_ts_ref(&self) -> u64;

    /// The value for price statistics where every 1 unit corresponds to 1e-9, i.e.
    /// 1/1,000,000,000 or 0.000000001. Will be [`UNDEF_PRICE`](crate::UNDEF_PRICE)
    /// when unused.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    fn price(&self) -> i64;

    /// The value for non-price statistics. Will be [`UNDEF_STAT_QUANTITY`](crate::UNDEF_STAT_QUANTITY)
    /// when unused.
    fn quantity(&self) -> i64;

    /// The message sequence number assigned at the venue.
    fn sequence(&self) -> u32;

    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    fn raw_ts_in_delta(&self) -> i32;

    /// The type of statistic value contained in the message. Refer to the
    /// [`StatType`](crate::enums::StatType) enum for possible variants.
    fn raw_stat_type(&self) -> u16;

    /// The channel ID assigned by Databento as an incrementing integer starting at zero.
    fn channel_id(&self) -> u8;

    /// Indicates if the statistic is newly added (1) or deleted (2). (Deleted is only
    /// used with some stat types).
    fn raw_update_action(&self) -> u8;

    /// Additional flags associate with certain stat types.
    fn stat_flags(&self) -> u8;

    /// Parses the capture-server-received timestamp into a datetime.
    /// Returns `None` if `ts_recv` contains the sentinel for a null timestamp.
    fn ts_recv(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.raw_ts_recv())
    }

    /// Parses the reference timestamp of the statistic value into a datetime.
    /// Returns `None` if `ts_ref` contains the sentinel for a null timestamp.
    fn ts_ref(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.raw_ts_ref())
    }

    /// Converts the value for price statistics to a floating point.
    ///
    /// `UNDEF_PRICE` will be converted to NaN.
    ///
    /// <div class="warning">
    /// This may introduce floating-point error.
    /// </div>
    fn price_f64(&self) -> f64 {
        px_to_f64(self.price())
    }

    /// Parses the difference between `ts_recv` and the matching-engine-sending timestamp into a duration.
    fn ts_in_delta(&self) -> time::Duration {
        time::Duration::new(0, self.raw_ts_in_delta())
    }

    /// Parses the type of statistic value into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `stat_type` field does not
    /// contain a valid [`StatType`](crate::StatType).
    fn stat_type(&self) -> Result<StatType> {
        let raw = self.raw_stat_type();
        StatType::try_from(raw)
            .map_err(|_| Error::conversion::<StatType>(format_args!("{raw:#04X}")))
    }

    /// Parses the update action into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `update_action` field does not
    /// contain a valid [`StatUpdateAction`](crate::StatUpdateAction).
    fn update_action(&self) -> Result<StatUpdateAction> {
        let raw = self.raw_update_action();
        StatUpdateAction::try_from(raw)
            .map_err(|_| Error::conversion::<StatUpdateAction>(format_args!("{raw:#04X}")))
    }
}

/// Accessors for the fields of [`ErrorMsg`](crate::ErrorMsg) records from any DBN
/// version.
pub trait ErrorRec: Record {
    /// The ID of the subscription that triggered the error, if applicable.
    fn subscription_id(&self) -> u32;

    /// The schema associated with the error, if applicable.
    fn raw_schema(&self) -> u16;

    /// The error code. See the [`ErrorCode`](crate::enums::ErrorCode) enum
    /// for possible values.
    fn raw_code(&self) -> u8;

    /// Sometimes multiple errors are sent together. This field will be non-zero for the
    /// last error.
    fn is_last(&self) -> u8;

    /// Parses the error message into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `err` contains invalid UTF-8.
    fn err(&self) -> Result<&str>;

    /// Parses the schema into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `schema` field does not
    /// contain a valid [`Schema`](crate::Schema).
    fn schema(&self) -> Result<Schema> {
        let raw = self.raw_schema();
        Schema::try_from(raw).map_err(|_| Error::conversion::<Schema>(format_args!("{raw:#04X}")))
    }

    /// Parses the error code into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `code` field does not
    /// contain a valid [`ErrorCode`](crate::ErrorCode).
    fn code(&self) -> Result<ErrorCode> {
        let raw = self.raw_code();
        ErrorCode::try_from(raw)
            .map_err(|_| Error::conversion::<ErrorCode>(format_args!("{raw:#04X}")))
    }
}

/// Accessors for the fields of [`SymbolMappingMsg`](crate::SymbolMappingMsg) records from any DBN
/// version.
///
/// The version 4 struct holds only some of its fields, so in version 4 a
/// [`ResolvedSymbolMappingRef`](crate::v4::ResolvedSymbolMappingRef) implements this trait.
pub trait SymbolMappingRec: Record {
    /// The start of the mapping interval expressed as the number of nanoseconds since
    /// the UNIX epoch.
    fn raw_start_ts(&self) -> u64;

    /// The end of the mapping interval expressed as the number of nanoseconds since
    /// the UNIX epoch.
    fn raw_end_ts(&self) -> u64;

    /// The input symbology type of `stype_in_symbol`.
    fn raw_stype_in(&self) -> u8;

    /// The output symbology type of `stype_out_symbol`. Will always be [`RawSymbol`](crate::SType::RawSymbol).
    fn raw_stype_out(&self) -> u8;

    /// Parses the input symbol into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if the record is too short to contain
    /// `stype_in_symbol` or it contains invalid UTF-8.
    fn stype_in_symbol(&self) -> Result<&str>;

    /// Parses the output symbol into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if the record is too short to contain
    /// `stype_out_symbol` or it contains invalid UTF-8.
    fn stype_out_symbol(&self) -> Result<&str>;

    /// Parses the start of the mapping interval into a datetime.
    /// Returns `None` if `start_ts` contains the sentinel for a null timestamp.
    fn start_ts(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.raw_start_ts())
    }

    /// Parses the end of the mapping interval into a datetime.
    /// Returns `None` if `end_ts` contains the sentinel for a null timestamp.
    fn end_ts(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.raw_end_ts())
    }

    /// Parses the stype in into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `stype_in` field does not
    /// contain a valid [`SType`](crate::SType).
    fn stype_in(&self) -> Result<SType> {
        let raw = self.raw_stype_in();
        SType::try_from(raw).map_err(|_| Error::conversion::<SType>(format_args!("{raw:#04X}")))
    }

    /// Parses the stype out into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `stype_out` field does not
    /// contain a valid [`SType`](crate::SType).
    fn stype_out(&self) -> Result<SType> {
        let raw = self.raw_stype_out();
        SType::try_from(raw).map_err(|_| Error::conversion::<SType>(format_args!("{raw:#04X}")))
    }
}

/// Accessors for the fields of [`SystemMsg`](crate::SystemMsg) records from any DBN
/// version.
pub trait SystemRec: Record {
    /// The ID of the subscription that triggered the message, if applicable.
    fn subscription_id(&self) -> u32;

    /// The schema associated with the message, if applicable.
    fn raw_schema(&self) -> u16;

    /// Type of system message. See the [`SystemCode`](crate::enums::SystemCode) enum
    /// for possible values.
    fn raw_code(&self) -> u8;

    /// Parses the message from the Databento gateway into a `&str`.
    ///
    /// # Errors
    /// This function returns an error if `msg` contains invalid UTF-8.
    fn msg(&self) -> Result<&str>;

    /// Parses the schema into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `schema` field does not
    /// contain a valid [`Schema`](crate::Schema).
    fn schema(&self) -> Result<Schema> {
        let raw = self.raw_schema();
        Schema::try_from(raw).map_err(|_| Error::conversion::<Schema>(format_args!("{raw:#04X}")))
    }

    /// Parses the type of system message into an enum.
    ///
    /// # Errors
    /// This function returns an error if the `code` field does not
    /// contain a valid [`SystemCode`](crate::SystemCode).
    fn code(&self) -> Result<SystemCode> {
        let raw = self.raw_code();
        SystemCode::try_from(raw)
            .map_err(|_| Error::conversion::<SystemCode>(format_args!("{raw:#04X}")))
    }

    /// Checks whether the message is a heartbeat from the gateway.
    fn is_heartbeat(&self) -> bool {
        match self.code() {
            Ok(SystemCode::Unset) | Err(_) => self
                .msg()
                .map(|msg| msg == SystemMsg::HEARTBEAT)
                .unwrap_or_default(),
            Ok(code) => code == SystemCode::Heartbeat,
        }
    }
}
