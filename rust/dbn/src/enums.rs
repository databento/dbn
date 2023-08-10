#![allow(deprecated)] // TODO: remove with SType::Smart

//! Enums used in Databento APIs.
use std::fmt::{self, Display, Formatter};

// Dummy derive macro to get around `cfg_attr` incompatibility of several
// of pyo3's attribute macros. See https://github.com/PyO3/pyo3/issues/780
#[cfg(not(feature = "python"))]
use dbn_macros::MockPyo3;
use num_enum::{IntoPrimitive, TryFromPrimitive};

/// A side of the market. The side of the market for resting orders, or the side
/// of the aggressor for trades.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Side {
    /// A sell order.
    Ask = b'A',
    /// A buy order.
    Bid = b'B',
    /// None or unknown.
    None = b'N',
}

impl From<Side> for char {
    fn from(side: Side) -> Self {
        u8::from(side) as char
    }
}

/// A tick action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Action {
    /// An existing order was modified.
    Modify = b'M',
    /// A trade executed.
    Trade = b'T',
    /// An existing order was filled.
    Fill = b'F',
    /// An order was cancelled.
    Cancel = b'C',
    /// A new order was added.
    Add = b'A',
    /// Reset the book; clear all orders for an instrument.
    Clear = b'R',
}

impl From<Action> for char {
    fn from(action: Action) -> Self {
        u8::from(action) as char
    }
}

/// The class of instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
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

/// The type of matching algorithm used for the instrument at the exchange.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum MatchAlgorithm {
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
}

impl From<MatchAlgorithm> for char {
    fn from(algo: MatchAlgorithm) -> Self {
        u8::from(algo) as char
    }
}

/// Whether the instrument is user-defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive, Default)]
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

/// A symbology type. Refer to the [symbology documentation](https://docs.databento.com/api-reference-historical/basics/symbology)
/// for more information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
#[cfg_attr(feature = "python", pyo3::pyclass(module = "databento_dbn"))]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[cfg_attr(feature = "python", derive(strum::EnumIter))]
pub enum SType {
    /// Symbology using a unique numeric ID.
    #[pyo3(name = "INSTRUMENT_ID")]
    InstrumentId = 0,
    /// Symbology using the original symbols provided by the publisher.
    #[pyo3(name = "RAW_SYMBOL")]
    RawSymbol = 1,
    /// A set of Databento-specific symbologies for referring to groups of symbols.
    #[deprecated(since = "0.5.0", note = "Smart was split into Continuous and Parent.")]
    #[pyo3(name = "SMART")]
    Smart = 2,
    /// A Databento-specific symbology where one symbol may point to different
    /// instruments at different points of time, e.g. to always refer to the front month
    /// future.
    #[pyo3(name = "CONTINUOUS")]
    Continuous = 3,
    /// A Databento-specific symbology for referring to a group of symbols by one
    /// "parent" symbol, e.g. ES.FUT to refer to all ES futures.
    #[pyo3(name = "PARENT")]
    Parent = 4,
    /// Symbology for US equities using NASDAQ Integrated suffix conventions.
    #[pyo3(name = "NASDAQ")]
    Nasdaq = 5,
    /// Symbology for US equities using CMS suffix conventions.
    #[pyo3(name = "CMS")]
    Cms = 6,
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
            "nasdaq" => Ok(SType::Nasdaq),
            "cms" => Ok(SType::Cms),
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
            SType::Nasdaq => "nasdaq",
            SType::Cms => "cms",
        }
    }
}

impl Display for SType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub use rtype::RType;

/// Record types, possible values for [`RecordHeader::rtype`][crate::record::RecordHeader::rtype]
#[allow(deprecated)]
pub mod rtype {
    use num_enum::TryFromPrimitive;

    use super::Schema;

    /// A type of record, i.e. a struct implementing [`HasRType`](crate::record::HasRType).
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, TryFromPrimitive)]
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
            _ => None,
        }
    }
}

/// A data record schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u16)]
#[cfg_attr(feature = "python", pyo3::pyclass(module = "databento_dbn"))]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[cfg_attr(feature = "python", derive(strum::EnumIter))]
#[cfg_attr(test, derive(strum::EnumCount))]
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
    /// Exchange status.
    #[doc(hidden)]
    #[pyo3(name = "STATUS")]
    Status = 11,
    /// Auction imbalance events.
    #[pyo3(name = "IMBALANCE")]
    Imbalance = 12,
    /// Open, high, low, close, and volume at a daily cadence based on the end of the
    /// trading session.
    #[pyo3(name = "OHLCV_EOD")]
    OhlcvEod = 13,
}

/// The number of [`Schema`]s.
pub const SCHEMA_COUNT: usize = 14;

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
#[repr(u8)]
#[cfg_attr(feature = "python", pyo3::pyclass(module = "databento_dbn"))]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[cfg_attr(feature = "python", derive(strum::EnumIter))]
pub enum Encoding {
    /// Databento Binary Encoding.
    #[pyo3(name = "DBN")]
    Dbn = 0,
    /// Comma-separated values.
    #[pyo3(name = "CSV")]
    Csv = 1,
    /// JavaScript object notation.
    #[pyo3(name = "JSON")]
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
#[repr(u8)]
#[cfg_attr(feature = "python", pyo3::pyclass(module = "databento_dbn"))]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[cfg_attr(feature = "python", derive(strum::EnumIter))]
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
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum StatType {
    /// The price of the first trade of an instrument. `price` will be set.
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
}

/// The type of [`StatMsg`](crate::record::StatMsg) update.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum StatUpdateAction {
    /// A new statistic.
    New = 1,
    /// A removal of a statistic.
    Delete = 2,
}

#[cfg(feature = "serde")]
mod deserialize {
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer};

    use super::*;

    impl<'de> Deserialize<'de> for Compression {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let str = String::deserialize(deserializer)?;
            FromStr::from_str(&str).map_err(de::Error::custom)
        }
    }

    impl<'de> Deserialize<'de> for SType {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let str = String::deserialize(deserializer)?;
            FromStr::from_str(&str).map_err(de::Error::custom)
        }
    }

    impl<'de> Deserialize<'de> for Schema {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let str = String::deserialize(deserializer)?;
            FromStr::from_str(&str).map_err(de::Error::custom)
        }
    }

    impl<'de> Deserialize<'de> for Encoding {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let str = String::deserialize(deserializer)?;
            FromStr::from_str(&str).map_err(de::Error::custom)
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
