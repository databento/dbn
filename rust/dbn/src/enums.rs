#![allow(deprecated)] // TODO: remove with SType::Smart

//! Enums used in Databento APIs.

use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

// Dummy derive macro to get around `cfg_attr` incompatibility of several
// of pyo3's attribute macros. See https://github.com/PyO3/pyo3/issues/780
#[cfg(not(feature = "python"))]
use dbn_macros::MockPyo3;
use num_enum::{IntoPrimitive, TryFromPrimitive};

/// A [side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types)
/// of the market. The side of the market for resting orders, or the side of the
/// aggressor for trades.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, TryFromPrimitive, IntoPrimitive)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter, strum::AsRefStr),
    pyo3::pyclass(module = "databento_dbn", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum Side {
    /// A sell order or sell aggressor in a trade.
    Ask = b'A',
    /// A buy order or a buy aggressor in a trade.
    Bid = b'B',
    /// No side specified by the original source.
    #[default]
    None = b'N',
}

impl From<Side> for char {
    fn from(side: Side) -> Self {
        u8::from(side) as char
    }
}

/// An [order event or order book operation](https://databento.com/docs/api-reference-historical/basics/schemas-and-conventions).
///
/// For example usage see:
/// - [Order actions](https://databento.com/docs/examples/order-book/order-actions)
/// - [Order tracking](https://databento.com/docs/examples/order-book/order-tracking)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, TryFromPrimitive, IntoPrimitive)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter, strum::AsRefStr),
    pyo3::pyclass(module = "databento_dbn", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum Action {
    /// An existing order was modified: price and/or size.
    Modify = b'M',
    /// An aggressing order traded. Does not affect the book.
    Trade = b'T',
    /// An existing order was filled. Does not affect the book.
    Fill = b'F',
    /// An order was fully or partially cancelled.
    Cancel = b'C',
    /// A new order was added to the book.
    Add = b'A',
    /// Reset the book; clear all orders for an instrument.
    Clear = b'R',
    /// Has no effect on the book, but may carry `flags` or other information.
    #[default]
    None = b'N',
}

impl From<Action> for char {
    fn from(action: Action) -> Self {
        u8::from(action) as char
    }
}

/// The class of instrument.
///
/// For example usage see
/// [Getting options with their underlying](https://databento.com/docs/examples/options/options-and-futures).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum InstrumentClass {
    /// A bond.
    Bond = b'B',
    /// A call option.
    Call = b'C',
    /// A future.
    Future = b'F',
    /// A stock.
    Stock = b'K',
    /// A spread composed of multiple instrument classes.
    MixedSpread = b'M',
    /// A put option.
    Put = b'P',
    /// A spread composed of futures.
    FutureSpread = b'S',
    /// A spread composed of options.
    OptionSpread = b'T',
    /// A foreign exchange spot.
    FxSpot = b'X',
}

impl From<InstrumentClass> for char {
    fn from(class: InstrumentClass) -> Self {
        u8::from(class) as char
    }
}

impl InstrumentClass {
    /// Returns `true` if the instrument class is a type of option.
    ///
    /// Note: excludes [`Self::MixedSpread`], which *may* include options.
    pub fn is_option(&self) -> bool {
        matches!(self, Self::Call | Self::Put | Self::OptionSpread)
    }

    /// Returns `true` if the instrument class is a type of future.
    ///
    /// Note: excludes [`Self::MixedSpread`], which *may* include futures.
    pub fn is_future(&self) -> bool {
        matches!(self, Self::Future | Self::FutureSpread)
    }

    /// Returns `true` if the instrument class is a type of spread, i.e. composed of two
    /// or more instrument legs.
    pub fn is_spread(&self) -> bool {
        matches!(
            self,
            Self::FutureSpread | Self::OptionSpread | Self::MixedSpread
        )
    }
}

/// The type of matching algorithm used for the instrument at the exchange.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum MatchAlgorithm {
    /// No matching algorithm was specified.
    #[default]
    Undefined = b' ',
    /// First-in-first-out matching.
    Fifo = b'F',
    /// A configurable match algorithm.
    Configurable = b'K',
    /// Trade quantity is allocated to resting orders based on a pro-rata percentage:
    /// resting order quantity divided by total quantity.
    ProRata = b'C',
    /// Like [`Self::Fifo`] but with LMM allocations prior to FIFO allocations.
    FifoLmm = b'T',
    /// Like [`Self::ProRata`] but includes a configurable allocation to the first order that
    /// improves the market.
    ThresholdProRata = b'O',
    /// Like [`Self::FifoLmm`] but includes a configurable allocation to the first order that
    /// improves the market.
    FifoTopLmm = b'S',
    /// Like [`Self::ThresholdProRata`] but includes a special priority to LMMs.
    ThresholdProRataLmm = b'Q',
    /// Special variant used only for Eurodollar futures on CME. See
    /// [CME documentation](https://www.cmegroup.com/confluence/display/EPICSANDBOX/Supported+Matching+Algorithms#SupportedMatchingAlgorithms-Pro-RataAllocationforEurodollarFutures).
    EurodollarFutures = b'Y',
    /// Trade quantity is shared between all orders at the best price. Orders with the
    /// highest time priority receive a higher matched quantity.
    TimeProRata = b'P',
}

impl From<MatchAlgorithm> for char {
    fn from(algo: MatchAlgorithm) -> Self {
        u8::from(algo) as char
    }
}

/// Whether the instrument is user-defined.
///
/// For example usage see
/// [Getting options with their underlying](https://databento.com/docs/examples/options/options-and-futures).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive, Default)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter, strum::AsRefStr),
    pyo3::pyclass(module = "databento_dbn", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum UserDefinedInstrument {
    /// The instrument is not user-defined.
    #[default]
    No = b'N',
    /// The instrument is user-defined.
    Yes = b'Y',
}

impl From<UserDefinedInstrument> for char {
    fn from(user_defined_instrument: UserDefinedInstrument) -> Self {
        u8::from(user_defined_instrument) as char
    }
}

/// A symbology type. Refer to the
/// [symbology documentation](https://databento.com/docs/api-reference-historical/basics/symbology)
/// for more information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, TryFromPrimitive)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum SType {
    /// Symbology using a unique numeric ID.
    InstrumentId = 0,
    /// Symbology using the original symbols provided by the publisher.
    RawSymbol = 1,
    /// A set of Databento-specific symbologies for referring to groups of symbols.
    #[deprecated(since = "0.5.0", note = "Smart was split into Continuous and Parent.")]
    Smart = 2,
    /// A Databento-specific symbology where one symbol may point to different
    /// instruments at different points of time, e.g. to always refer to the front month
    /// future.
    Continuous = 3,
    /// A Databento-specific symbology for referring to a group of symbols by one
    /// "parent" symbol, e.g. ES.FUT to refer to all ES futures.
    Parent = 4,
    /// Symbology for US equities using NASDAQ Integrated suffix conventions.
    NasdaqSymbol = 5,
    /// Symbology for US equities using CMS suffix conventions.
    CmsSymbol = 6,
    /// Symbology using International Security Identification Numbers (ISIN) - ISO 6166.
    Isin = 7,
    /// Symbology using US domestic Committee on Uniform Securities Identification Procedure (CUSIP) codes.
    UsCode = 8,
    /// Symbology using Bloomberg composite global IDs.
    BbgCompId = 9,
    /// Symbology using Bloomberg composite tickers.
    BbgCompTicker = 10,
    /// Symbology using Bloomberg FIGI exchange level IDs.
    Figi = 11,
    /// Symbology using Bloomberg exchange level tickers.
    FigiTicker = 12,
}

impl std::str::FromStr for SType {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "instrument_id" | "product_id" => Ok(SType::InstrumentId),
            "raw_symbol" | "native" => Ok(SType::RawSymbol),
            "smart" => Ok(SType::Smart),
            "continuous" => Ok(SType::Continuous),
            "parent" => Ok(SType::Parent),
            "nasdaq_symbol" | "nasdaq" => Ok(SType::NasdaqSymbol),
            "cms_symbol" | "cms" => Ok(SType::CmsSymbol),
            "isin" => Ok(SType::Isin),
            "us_code" => Ok(SType::UsCode),
            "bbg_comp_id" => Ok(SType::BbgCompId),
            "bbg_comp_ticker" => Ok(SType::BbgCompTicker),
            "figi" => Ok(SType::Figi),
            "figi_ticker" => Ok(SType::FigiTicker),
            _ => Err(crate::Error::conversion::<Self>(s.to_owned())),
        }
    }
}

impl AsRef<str> for SType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl SType {
    /// Convert the symbology type to its `str` representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            SType::InstrumentId => "instrument_id",
            SType::RawSymbol => "raw_symbol",
            #[allow(deprecated)]
            SType::Smart => "smart",
            SType::Continuous => "continuous",
            SType::Parent => "parent",
            SType::NasdaqSymbol => "nasdaq_symbol",
            SType::CmsSymbol => "cms_symbol",
            SType::Isin => "isin",
            SType::UsCode => "us_code",
            SType::BbgCompId => "bbg_comp_id",
            SType::BbgCompTicker => "bbg_comp_ticker",
            SType::Figi => "figi",
            SType::FigiTicker => "figi_ticker",
        }
    }
}

impl Display for SType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub use rtype::RType;

/// Record types, possible values for [`RecordHeader::rtype`][crate::RecordHeader::rtype]
#[allow(deprecated)]
pub mod rtype {
    use num_enum::TryFromPrimitive;

    use super::Schema;

    /// A [record type](https://databento.com/docs/standards-and-conventions/common-fields-enums-types),
    /// i.e. a sentinel for different types implementing [`HasRType`](crate::record::HasRType).
    ///
    /// Use in [`RecordHeader`](crate::RecordHeader) to indicate the type of record,
    /// which is useful when working with DBN streams containing multiple record types
    /// or an unknown record type.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, TryFromPrimitive)]
    #[cfg_attr(
        feature = "python",
        derive(strum::EnumIter),
        pyo3::pyclass(module = "databento_dbn", rename_all = "SCREAMING_SNAKE_CASE")
    )]
    #[repr(u8)]
    pub enum RType {
        /// Denotes a market-by-price record with a book depth of 0 (used for the
        /// [`Trades`](super::Schema::Trades) schema).
        Mbp0 = 0,
        /// Denotes a market-by-price record with a book depth of 1 (also used for the
        /// [`Tbbo`](super::Schema::Tbbo) schema).
        Mbp1 = 0x01,
        /// Denotes a market-by-price record with a book depth of 10.
        Mbp10 = 0x0A,
        /// Denotes an open, high, low, close, and volume record at an unspecified cadence.
        #[deprecated(
            since = "0.3.3",
            note = "Separated into separate rtypes for each OHLCV schema."
        )]
        OhlcvDeprecated = 0x11,
        /// Denotes an open, high, low, close, and volume record at a 1-second cadence.
        Ohlcv1S = 0x20,
        /// Denotes an open, high, low, close, and volume record at a 1-minute cadence.
        Ohlcv1M = 0x21,
        /// Denotes an open, high, low, close, and volume record at an hourly cadence.
        Ohlcv1H = 0x22,
        /// Denotes an open, high, low, close, and volume record at a daily cadence
        /// based on the UTC date.
        Ohlcv1D = 0x23,
        /// Denotes an open, high, low, close, and volume record at a daily cadence
        /// based on the end of the trading session.
        OhlcvEod = 0x24,
        /// Denotes an exchange status record.
        Status = 0x12,
        /// Denotes an instrument definition record.
        InstrumentDef = 0x13,
        /// Denotes an order imbalance record.
        Imbalance = 0x14,
        /// Denotes an error from gateway.
        Error = 0x15,
        /// Denotes a symbol mapping record.
        SymbolMapping = 0x16,
        /// Denotes a non-error message from the gateway. Also used for heartbeats.
        System = 0x17,
        /// Denotes a statistics record from the publisher (not calculated by Databento).
        Statistics = 0x18,
        /// Denotes a market by order record.
        Mbo = 0xA0,
        /// Denotes a consolidated best bid and offer record.
        Cmbp1 = 0xB1,
        /// Denotes a consolidated best bid and offer record subsampled on a one-second
        /// interval.
        Cbbo1S = 0xC0,
        /// Denotes a consolidated best bid and offer record subsampled on a one-minute
        /// interval.
        Cbbo1M = 0xC1,
        /// Denotes a consolidated best bid and offer trade record containing the
        /// consolidated BBO before the trade.
        Tcbbo = 0xC2,
        /// Denotes a best bid and offer record subsampled on a one-second interval.
        Bbo1S = 0xC3,
        /// Denotes a best bid and offer record subsampled on a one-minute interval.
        Bbo1M = 0xC4,
    }

    /// Denotes a market-by-price record with a book depth of 0 (used for the
    /// [`Trades`](super::Schema::Trades) schema).
    pub const MBP_0: u8 = RType::Mbp0 as u8;
    /// Denotes a market-by-price record with a book depth of 1 (also used for the
    /// [`Tbbo`](super::Schema::Tbbo) schema).
    pub const MBP_1: u8 = RType::Mbp1 as u8;
    /// Denotes a market-by-price record with a book depth of 10.
    pub const MBP_10: u8 = RType::Mbp10 as u8;
    /// Denotes an open, high, low, close, and volume record at an unspecified cadence.
    #[deprecated(
        since = "0.3.3",
        note = "Separated into separate rtypes for each OHLCV schema."
    )]
    pub const OHLCV_DEPRECATED: u8 = RType::OhlcvDeprecated as u8;
    /// Denotes an open, high, low, close, and volume record at a 1-second cadence.
    pub const OHLCV_1S: u8 = RType::Ohlcv1S as u8;
    /// Denotes an open, high, low, close, and volume record at a 1-minute cadence.
    pub const OHLCV_1M: u8 = RType::Ohlcv1M as u8;
    /// Denotes an open, high, low, close, and volume record at an hourly cadence.
    pub const OHLCV_1H: u8 = RType::Ohlcv1H as u8;
    /// Denotes an open, high, low, close, and volume record at a daily cadence based
    /// on the UTC date.
    pub const OHLCV_1D: u8 = RType::Ohlcv1D as u8;
    /// Denotes an open, high, low, close, and volume record at a daily cadence
    /// based on the end of the trading session.
    pub const OHLCV_EOD: u8 = RType::OhlcvEod as u8;
    /// Denotes an exchange status record.
    pub const STATUS: u8 = RType::Status as u8;
    /// Denotes an instrument definition record.
    pub const INSTRUMENT_DEF: u8 = RType::InstrumentDef as u8;
    /// Denotes an order imbalance record.
    pub const IMBALANCE: u8 = RType::Imbalance as u8;
    /// Denotes an error from gateway.
    pub const ERROR: u8 = RType::Error as u8;
    /// Denotes a symbol mapping record.
    pub const SYMBOL_MAPPING: u8 = RType::SymbolMapping as u8;
    /// Denotes a non-error message from the gateway. Also used for heartbeats.
    pub const SYSTEM: u8 = RType::System as u8;
    /// Denotes a statistics record from the publisher (not calculated by Databento).
    pub const STATISTICS: u8 = RType::Statistics as u8;
    /// Denotes a market-by-order record.
    pub const MBO: u8 = RType::Mbo as u8;
    /// Denotes a consolidated best bid and offer record.
    pub const CMBP1: u8 = RType::Cmbp1 as u8;
    /// Denotes a consolidated best bid and offer record subsampled on a one-second interval.
    pub const CBBO_1S: u8 = RType::Cbbo1S as u8;
    /// Denotes a consolidated best bid and offer record subsampled on a one-minute interval.
    pub const CBBO_1M: u8 = RType::Cbbo1M as u8;
    /// Denotes a consolidated best bid and offer trade record containing the consolidated BBO before the trade.
    pub const TCBBO: u8 = RType::Tcbbo as u8;
    /// Denotes a best bid and offer record subsampled on a one-second interval.
    pub const BBO_1S: u8 = RType::Bbo1S as u8;
    /// Denotes a best bid and offer record subsampled on a one-minute interval.
    pub const BBO_1M: u8 = RType::Bbo1M as u8;

    /// Get the corresponding `rtype` for the given `schema`.
    impl From<Schema> for RType {
        fn from(schema: Schema) -> Self {
            match schema {
                Schema::Mbo => RType::Mbo,
                Schema::Mbp1 | Schema::Tbbo => RType::Mbp1,
                Schema::Mbp10 => RType::Mbp10,
                Schema::Trades => RType::Mbp0,
                Schema::Ohlcv1S => RType::Ohlcv1S,
                Schema::Ohlcv1M => RType::Ohlcv1M,
                Schema::Ohlcv1H => RType::Ohlcv1H,
                Schema::Ohlcv1D => RType::Ohlcv1D,
                Schema::OhlcvEod => RType::OhlcvEod,
                Schema::Definition => RType::InstrumentDef,
                Schema::Statistics => RType::Statistics,
                Schema::Status => RType::Status,
                Schema::Imbalance => RType::Imbalance,
                Schema::Cmbp1 => RType::Cmbp1,
                Schema::Cbbo1S => RType::Cbbo1S,
                Schema::Cbbo1M => RType::Cbbo1M,
                Schema::Tcbbo => RType::Tcbbo,
                Schema::Bbo1S => RType::Bbo1S,
                Schema::Bbo1M => RType::Bbo1M,
            }
        }
    }

    /// Tries to convert the given rtype to a [`Schema`].
    ///
    /// Returns `None` if there's no corresponding `Schema` for the given rtype or
    /// in the case of [`OHLCV_DEPRECATED`], it doesn't map to a single `Schema`.
    pub fn try_into_schema(rtype: u8) -> Option<Schema> {
        match rtype {
            MBP_0 => Some(Schema::Trades),
            MBP_1 => Some(Schema::Mbp1),
            MBP_10 => Some(Schema::Mbp10),
            OHLCV_1S => Some(Schema::Ohlcv1S),
            OHLCV_1M => Some(Schema::Ohlcv1M),
            OHLCV_1H => Some(Schema::Ohlcv1H),
            OHLCV_1D => Some(Schema::Ohlcv1D),
            OHLCV_EOD => Some(Schema::OhlcvEod),
            STATUS => Some(Schema::Status),
            INSTRUMENT_DEF => Some(Schema::Definition),
            IMBALANCE => Some(Schema::Imbalance),
            STATISTICS => Some(Schema::Statistics),
            MBO => Some(Schema::Mbo),
            CMBP1 => Some(Schema::Cmbp1),
            CBBO_1S => Some(Schema::Cbbo1S),
            CBBO_1M => Some(Schema::Cbbo1M),
            TCBBO => Some(Schema::Tcbbo),
            BBO_1S => Some(Schema::Bbo1S),
            BBO_1M => Some(Schema::Bbo1M),
            _ => None,
        }
    }

    impl std::str::FromStr for RType {
        type Err = crate::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "mbp-0" => Ok(RType::Mbp0),
                "mbp-1" => Ok(RType::Mbp1),
                "mbp-10" => Ok(RType::Mbp10),
                "ohlcv-deprecated" => Ok(RType::OhlcvDeprecated),
                "ohlcv-1s" => Ok(RType::Ohlcv1S),
                "ohlcv-1m" => Ok(RType::Ohlcv1M),
                "ohlcv-1h" => Ok(RType::Ohlcv1H),
                "ohlcv-1d" => Ok(RType::Ohlcv1D),
                "ohlcv-eod" => Ok(RType::OhlcvEod),
                "status" => Ok(RType::Status),
                "instrument-def" => Ok(RType::InstrumentDef),
                "imbalance" => Ok(RType::Imbalance),
                "error" => Ok(RType::Error),
                "symbol-mapping" => Ok(RType::SymbolMapping),
                "system" => Ok(RType::System),
                "statistics" => Ok(RType::Statistics),
                "mbo" => Ok(RType::Mbo),
                "cmbp-1" => Ok(RType::Cmbp1),
                "cbbo-1s" => Ok(RType::Cbbo1S),
                "cbbo-1m" => Ok(RType::Cbbo1M),
                "tcbbo" => Ok(RType::Tcbbo),
                "bbo-1s" => Ok(RType::Bbo1S),
                "bbo-1m" => Ok(RType::Bbo1M),
                _ => Err(crate::Error::conversion::<Self>(s.to_owned())),
            }
        }
    }

    impl RType {
        /// Convert the RType type to its `str` representation.
        pub const fn as_str(&self) -> &'static str {
            match self {
                RType::Mbp0 => "mbp-0",
                RType::Mbp1 => "mbp-1",
                RType::Mbp10 => "mbp-10",
                RType::OhlcvDeprecated => "ohlcv-deprecated",
                RType::Ohlcv1S => "ohlcv-1s",
                RType::Ohlcv1M => "ohlcv-1m",
                RType::Ohlcv1H => "ohlcv-1h",
                RType::Ohlcv1D => "ohlcv-1d",
                RType::OhlcvEod => "ohlcv-eod",
                RType::Status => "status",
                RType::InstrumentDef => "instrument-def",
                RType::Imbalance => "imbalance",
                RType::Error => "error",
                RType::SymbolMapping => "symbol-mapping",
                RType::System => "system",
                RType::Statistics => "statistics",
                RType::Mbo => "mbo",
                RType::Cmbp1 => "cmbp-1",
                RType::Cbbo1S => "cbbo-1s",
                RType::Cbbo1M => "cbbo-1m",
                RType::Tcbbo => "tcbbo",
                RType::Bbo1S => "bbo-1s",
                RType::Bbo1M => "bbo-1m",
            }
        }
    }
}

/// A data record schema.
///
/// Each schema has a particular [record](crate::record) type associated with it.
///
/// See [List of supported market data schemas](https://databento.com/docs/schemas-and-data-formats/whats-a-schema)
/// for an overview of the differences and use cases of each schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, TryFromPrimitive)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[cfg_attr(test, derive(strum::EnumCount))]
#[repr(u16)]
pub enum Schema {
    /// Market by order.
    #[pyo3(name = "MBO")]
    Mbo = 0,
    /// Market by price with a book depth of 1.
    #[pyo3(name = "MBP_1")]
    Mbp1 = 1,
    /// Market by price with a book depth of 10.
    #[pyo3(name = "MBP_10")]
    Mbp10 = 2,
    /// All trade events with the best bid and offer (BBO) immediately **before** the
    /// effect of the trade.
    #[pyo3(name = "TBBO")]
    Tbbo = 3,
    /// All trade events.
    #[pyo3(name = "TRADES")]
    Trades = 4,
    /// Open, high, low, close, and volume at a one-second interval.
    #[pyo3(name = "OHLCV_1S")]
    Ohlcv1S = 5,
    /// Open, high, low, close, and volume at a one-minute interval.
    #[pyo3(name = "OHLCV_1M")]
    Ohlcv1M = 6,
    /// Open, high, low, close, and volume at an hourly interval.
    #[pyo3(name = "OHLCV_1H")]
    Ohlcv1H = 7,
    /// Open, high, low, close, and volume at a daily interval based on the UTC date.
    #[pyo3(name = "OHLCV_1D")]
    Ohlcv1D = 8,
    /// Instrument definitions.
    #[pyo3(name = "DEFINITION")]
    Definition = 9,
    /// Additional data disseminated by publishers.
    #[pyo3(name = "STATISTICS")]
    Statistics = 10,
    /// Trading status events.
    #[pyo3(name = "STATUS")]
    Status = 11,
    /// Auction imbalance events.
    #[pyo3(name = "IMBALANCE")]
    Imbalance = 12,
    /// Open, high, low, close, and volume at a daily cadence based on the end of the
    /// trading session.
    #[pyo3(name = "OHLCV_EOD")]
    OhlcvEod = 13,
    /// Consolidated best bid and offer.
    #[pyo3(name = "CMBP_1")]
    Cmbp1 = 14,
    /// Consolidated best bid and offer subsampled at one-second intervals, in addition
    /// to trades.
    #[pyo3(name = "CBBO_1S")]
    Cbbo1S = 15,
    /// Consolidated best bid and offer subsampled at one-minute intervals, in addition
    /// to trades.
    #[pyo3(name = "CBBO_1M")]
    Cbbo1M = 16,
    /// All trade events with the consolidated best bid and offer (CBBO) immediately
    /// **before** the effect of the trade.
    #[pyo3(name = "TCBBO")]
    Tcbbo = 17,
    /// Best bid and offer subsampled at one-second intervals, in addition to trades.
    #[pyo3(name = "BBO_1S")]
    Bbo1S = 18,
    /// Best bid and offer subsampled at one-minute intervals, in addition to trades.
    #[pyo3(name = "BBO_1M")]
    Bbo1M = 19,
}

/// The number of [`Schema`]s.
pub const SCHEMA_COUNT: usize = 20;

impl std::str::FromStr for Schema {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mbo" => Ok(Schema::Mbo),
            "mbp-1" => Ok(Schema::Mbp1),
            "mbp-10" => Ok(Schema::Mbp10),
            "tbbo" => Ok(Schema::Tbbo),
            "trades" => Ok(Schema::Trades),
            "ohlcv-1s" => Ok(Schema::Ohlcv1S),
            "ohlcv-1m" => Ok(Schema::Ohlcv1M),
            "ohlcv-1h" => Ok(Schema::Ohlcv1H),
            "ohlcv-1d" => Ok(Schema::Ohlcv1D),
            "ohlcv-eod" => Ok(Schema::OhlcvEod),
            "definition" => Ok(Schema::Definition),
            "statistics" => Ok(Schema::Statistics),
            "status" => Ok(Schema::Status),
            "imbalance" => Ok(Schema::Imbalance),
            "cmbp-1" => Ok(Schema::Cmbp1),
            "cbbo-1s" => Ok(Schema::Cbbo1S),
            "cbbo-1m" => Ok(Schema::Cbbo1M),
            "tcbbo" => Ok(Schema::Tcbbo),
            "bbo-1s" => Ok(Schema::Bbo1S),
            "bbo-1m" => Ok(Schema::Bbo1M),
            _ => Err(crate::Error::conversion::<Self>(s.to_owned())),
        }
    }
}

impl AsRef<str> for Schema {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Schema {
    /// Converts the given schema to a `&'static str`.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Schema::Mbo => "mbo",
            Schema::Mbp1 => "mbp-1",
            Schema::Mbp10 => "mbp-10",
            Schema::Tbbo => "tbbo",
            Schema::Trades => "trades",
            Schema::Ohlcv1S => "ohlcv-1s",
            Schema::Ohlcv1M => "ohlcv-1m",
            Schema::Ohlcv1H => "ohlcv-1h",
            Schema::Ohlcv1D => "ohlcv-1d",
            Schema::OhlcvEod => "ohlcv-eod",
            Schema::Definition => "definition",
            Schema::Statistics => "statistics",
            Schema::Status => "status",
            Schema::Imbalance => "imbalance",
            Schema::Cmbp1 => "cmbp-1",
            Schema::Cbbo1S => "cbbo-1s",
            Schema::Cbbo1M => "cbbo-1m",
            Schema::Tcbbo => "tcbbo",
            Schema::Bbo1S => "bbo-1s",
            Schema::Bbo1M => "bbo-1m",
        }
    }
}

impl Display for Schema {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data encoding format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, TryFromPrimitive)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum Encoding {
    /// Databento Binary Encoding.
    Dbn = 0,
    /// Comma-separated values.
    Csv = 1,
    /// JavaScript object notation.
    Json = 2,
}

impl std::str::FromStr for Encoding {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dbn" | "dbz" => Ok(Encoding::Dbn),
            "csv" => Ok(Encoding::Csv),
            "json" => Ok(Encoding::Json),
            _ => Err(crate::Error::conversion::<Self>(s.to_owned())),
        }
    }
}

impl AsRef<str> for Encoding {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Encoding {
    /// Converts the given encoding to a `&'static str`.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Encoding::Dbn => "dbn",
            Encoding::Csv => "csv",
            Encoding::Json => "json",
        }
    }
}

impl Display for Encoding {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A compression format or none if uncompressed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, TryFromPrimitive)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[repr(u8)]
pub enum Compression {
    /// Uncompressed.
    #[pyo3(name = "NONE")]
    None = 0,
    /// Zstandard compressed.
    #[pyo3(name = "ZSTD")]
    ZStd = 1,
}

impl std::str::FromStr for Compression {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Compression::None),
            "zstd" => Ok(Compression::ZStd),
            _ => Err(crate::Error::conversion::<Self>(s.to_owned())),
        }
    }
}

impl AsRef<str> for Compression {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Compression {
    /// Converts the given compression to a `&'static str`.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Compression::None => "none",
            Compression::ZStd => "zstd",
        }
    }
}

impl Display for Compression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Constants for the bit flag record fields.
pub mod flags {
    /// Indicates it's the last message in the packet from the venue for a given
    /// `instrument_id`.
    pub const LAST: u8 = 1 << 7;
    /// Indicates a top-of-book message, not an individual order.
    pub const TOB: u8 = 1 << 6;
    /// Indicates the message was sourced from a replay, such as a snapshot server.
    pub const SNAPSHOT: u8 = 1 << 5;
    /// Indicates an aggregated price level message, not an individual order.
    pub const MBP: u8 = 1 << 4;
    /// Indicates the `ts_recv` value is inaccurate due to clock issues or packet
    /// reordering.
    pub const BAD_TS_RECV: u8 = 1 << 3;
    /// Indicates an unrecoverable gap was detected in the channel.
    pub const MAYBE_BAD_BOOK: u8 = 1 << 2;
}

/// The type of [`InstrumentDefMsg`](crate::record::InstrumentDefMsg) update.
#[allow(clippy::manual_non_exhaustive)] // false positive
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter, strum::AsRefStr),
    pyo3::pyclass(module = "databento_dbn", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum SecurityUpdateAction {
    /// A new instrument definition.
    Add = b'A',
    /// A modified instrument definition of an existing one.
    Modify = b'M',
    /// Removal of an instrument definition.
    Delete = b'D',
    #[doc(hidden)]
    #[deprecated = "Still present in legacy files."]
    Invalid = b'~',
}

/// The type of statistic contained in a [`StatMsg`](crate::record::StatMsg).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[non_exhaustive]
#[repr(u16)]
pub enum StatType {
    /// The price of the first trade of an instrument. `price` will be set.
    /// `quantity` will be set when provided by the venue.
    OpeningPrice = 1,
    /// The probable price of the first trade of an instrument published during pre-
    /// open. Both `price` and `quantity` will be set.
    IndicativeOpeningPrice = 2,
    /// The settlement price of an instrument. `price` will be set and `flags` indicate
    /// whether the price is final or preliminary and actual or theoretical. `ts_ref`
    /// will indicate the trading date of the settlement price.
    SettlementPrice = 3,
    /// The lowest trade price of an instrument during the trading session. `price` will
    /// be set.
    TradingSessionLowPrice = 4,
    /// The highest trade price of an instrument during the trading session. `price` will
    /// be set.
    TradingSessionHighPrice = 5,
    /// The number of contracts cleared for an instrument on the previous trading date.
    /// `quantity` will be set. `ts_ref` will indicate the trading date of the volume.
    ClearedVolume = 6,
    /// The lowest offer price for an instrument during the trading session. `price`
    /// will be set.
    LowestOffer = 7,
    /// The highest bid price for an instrument during the trading session. `price`
    /// will be set.
    HighestBid = 8,
    /// The current number of outstanding contracts of an instrument. `quantity` will
    /// be set. `ts_ref` will indicate the trading date for which the open interest was
    /// calculated.
    OpenInterest = 9,
    /// The volume-weighted average price (VWAP) for a fixing period. `price` will be
    /// set.
    FixingPrice = 10,
    /// The last trade price during a trading session. `price` will be set.
    /// `quantity` will be set when provided by the venue.
    ClosePrice = 11,
    /// The change in price from the close price of the previous trading session to the
    /// most recent trading session. `price` will be set.
    NetChange = 12,
    /// The volume-weighted average price (VWAP) during the trading session.
    /// `price` will be set to the VWAP while `quantity` will be the traded
    /// volume.
    Vwap = 13,
    /// The implied volatility associated with the settlement price. `price` will be set
    /// with the standard precision.
    Volatility = 14,
    /// The option delta associated with the settlement price. `price` will be set with
    /// the standard precision.
    Delta = 15,
    /// The auction uncrossing price. This is used for auctions that are neither the
    /// official opening auction nor the official closing auction. `price` will be set.
    /// `quantity` will be set when provided by the venue.
    UncrossingPrice = 16,
}

/// The type of [`StatMsg`](crate::record::StatMsg) update.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum StatUpdateAction {
    /// A new statistic.
    New = 1,
    /// A removal of a statistic.
    Delete = 2,
}

/// The primary enum for the type of [`StatusMsg`](crate::record::StatusMsg) update.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Default)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[non_exhaustive]
#[repr(u16)]
pub enum StatusAction {
    /// No change.
    #[default]
    None = 0,
    /// The instrument is in a pre-open period.
    PreOpen = 1,
    /// The instrument is in a pre-cross period.
    PreCross = 2,
    /// The instrument is quoting but not trading.
    Quoting = 3,
    /// The instrument is in a cross/auction.
    Cross = 4,
    /// The instrument is being opened through a trading rotation.
    Rotation = 5,
    /// A new price indication is available for the instrument.
    NewPriceIndication = 6,
    /// The instrument is trading.
    Trading = 7,
    /// Trading in the instrument has been halted.
    Halt = 8,
    /// Trading in the instrument has been paused.
    Pause = 9,
    /// Trading in the instrument has been suspended.
    Suspend = 10,
    /// The instrument is in a pre-close period.
    PreClose = 11,
    /// Trading in the instrument has closed.
    Close = 12,
    /// The instrument is in a post-close period.
    PostClose = 13,
    /// A change in short-selling restrictions.
    SsrChange = 14,
    /// The instrument is not available for trading, either trading has closed or been
    /// halted.
    NotAvailableForTrading = 15,
}

/// The secondary enum for a [`StatusMsg`](crate::record::StatusMsg) update, explains
/// the cause of a halt or other change in `action`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Default)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[non_exhaustive]
#[repr(u16)]
pub enum StatusReason {
    /// No reason is given.
    #[default]
    None = 0,
    /// The change in status occurred as scheduled.
    Scheduled = 1,
    /// The instrument stopped due to a market surveillance intervention.
    SurveillanceIntervention = 2,
    /// The status changed due to activity in the market.
    MarketEvent = 3,
    /// The derivative instrument began trading.
    InstrumentActivation = 4,
    /// The derivative instrument expired.
    InstrumentExpiration = 5,
    /// Recovery in progress.
    RecoveryInProcess = 6,
    /// The status change was caused by a regulatory action.
    Regulatory = 10,
    /// The status change was caused by an administrative action.
    Administrative = 11,
    /// The status change was caused by the issuer not being compliance with regulatory
    /// requirements.
    NonCompliance = 12,
    /// Trading halted because the issuer's filings are not current.
    FilingsNotCurrent = 13,
    /// Trading halted due to an SEC trading suspension.
    SecTradingSuspension = 14,
    /// The status changed because a new issue is available.
    NewIssue = 15,
    /// The status changed because an issue is available.
    IssueAvailable = 16,
    /// The status changed because the issue(s) were reviewed.
    IssuesReviewed = 17,
    /// The status changed because the filing requirements were satisfied.
    FilingReqsSatisfied = 18,
    /// Relevant news is pending.
    NewsPending = 30,
    /// Relevant news was released.
    NewsReleased = 31,
    /// The news has been fully disseminated and times are available for the resumption
    /// in quoting and trading.
    NewsAndResumptionTimes = 32,
    /// The relevants news was not forthcoming.
    NewsNotForthcoming = 33,
    /// Halted for order imbalance.
    OrderImbalance = 40,
    /// The instrument hit limit up or limit down.
    LuldPause = 50,
    /// An operational issue occurred with the venue.
    Operational = 60,
    /// The status changed until the exchange receives additional information.
    AdditionalInformationRequested = 70,
    /// Trading halted due to merger becoming effective.
    MergerEffective = 80,
    /// Trading is halted in an ETF due to conditions with the component securities.
    Etf = 90,
    /// Trading is halted for a corporate action.
    CorporateAction = 100,
    /// Trading is halted because the instrument is a new offering.
    NewSecurityOffering = 110,
    /// Halted due to the market-wide circuit breaker level 1.
    MarketWideHaltLevel1 = 120,
    /// Halted due to the market-wide circuit breaker level 2.
    MarketWideHaltLevel2 = 121,
    /// Halted due to the market-wide circuit breaker level 3.
    MarketWideHaltLevel3 = 122,
    /// Halted due to the carryover of a market-wide circuit breaker from the previous
    /// trading day.
    MarketWideHaltCarryover = 123,
    /// Resumption due to the end of the a market-wide circuit breaker halt.
    MarketWideHaltResumption = 124,
    /// Halted because quotation is not available.
    QuotationNotAvailable = 130,
}

/// Further information about a status update.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Default)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[non_exhaustive]
#[repr(u16)]
pub enum TradingEvent {
    /// No additional information given.
    #[default]
    None = 0,
    /// Order entry and modification are not allowed.
    NoCancel = 1,
    /// A change of trading session occurred. Daily statistics are reset.
    ChangeTradingSession = 2,
    /// Implied matching is available.
    ImpliedMatchingOn = 3,
    /// Implied matching is not available.
    ImpliedMatchingOff = 4,
}

/// An enum for representing unknown, true, or false values. Equivalent to
/// `Option<bool>` but with a human-readable repr.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Default)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum TriState {
    /// The value is not applicable or not known.
    #[default]
    NotAvailable = b'~',
    /// False
    No = b'N',
    /// True
    Yes = b'Y',
}

impl From<TriState> for Option<bool> {
    fn from(value: TriState) -> Self {
        match value {
            TriState::NotAvailable => None,
            TriState::No => Some(false),
            TriState::Yes => Some(true),
        }
    }
}

impl From<Option<bool>> for TriState {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(true) => Self::Yes,
            Some(false) => Self::No,
            None => Self::NotAvailable,
        }
    }
}

/// How to handle decoding DBN data from a prior version.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[non_exhaustive]
pub enum VersionUpgradePolicy {
    /// Decode data from previous versions as-is.
    AsIs,
    /// Decode data from previous versions converting it to the latest version. This
    /// breaks zero-copy decoding for structs that need updating, but makes usage
    /// simpler.
    #[default]
    Upgrade,
}

impl FromStr for VersionUpgradePolicy {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AsIs" => Ok(Self::AsIs),
            "Upgrade" => Ok(Self::Upgrade),
            _ => Err(crate::Error::conversion::<VersionUpgradePolicy>(s)),
        }
    }
}

#[cfg(feature = "serde")]
mod deserialize {
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer, Serialize};

    use super::*;

    impl<'de> Deserialize<'de> for Compression {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let str = String::deserialize(deserializer)?;
            FromStr::from_str(&str).map_err(de::Error::custom)
        }
    }

    impl Serialize for Compression {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.as_str().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for SType {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let str = String::deserialize(deserializer)?;
            FromStr::from_str(&str).map_err(de::Error::custom)
        }
    }

    impl Serialize for SType {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.as_str().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Schema {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let str = String::deserialize(deserializer)?;
            FromStr::from_str(&str).map_err(de::Error::custom)
        }
    }

    impl Serialize for Schema {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.as_str().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Encoding {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let str = String::deserialize(deserializer)?;
            FromStr::from_str(&str).map_err(de::Error::custom)
        }
    }

    impl Serialize for Encoding {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.as_str().serialize(serializer)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_schema_count() {
        use strum::EnumCount;

        assert_eq!(Schema::COUNT, SCHEMA_COUNT);
    }
}
