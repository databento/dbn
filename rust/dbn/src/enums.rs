//! Enums used in Databento APIs.
use std::fmt::{self, Display, Formatter};

use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Serialize;

use crate::error::ConversionError;

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
    None,
}

impl From<Side> for char {
    fn from(side: Side) -> Self {
        u8::from(side) as char
    }
}

impl Serialize for Side {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_char(char::from(*self))
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

impl serde::Serialize for Action {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_char(char::from(*self))
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

impl serde::Serialize for InstrumentClass {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_char(char::from(*self))
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

impl serde::Serialize for MatchAlgorithm {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_char(char::from(*self))
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

impl serde::Serialize for UserDefinedInstrument {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_char(char::from(*self))
    }
}

/// A symbology type. Refer to the [symbology documentation](https://docs.databento.com/api-reference-historical/basics/symbology)
/// for more information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TryFromPrimitive)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum SType {
    /// Symbology using a unique numeric ID.
    ProductId = 0,
    /// Symbology using the original symbols provided by the publisher.
    Native = 1,
    /// A set of Databento-specific symbologies for referring to groups of symbols.
    Smart = 2,
}

impl std::str::FromStr for SType {
    type Err = ConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "product_id" => Ok(SType::ProductId),
            "native" => Ok(SType::Native),
            "smart" => Ok(SType::Smart),
            _ => Err(ConversionError::TypeConversion(
                "Value doesn't match a valid symbol type",
            )),
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
            SType::Native => "native",
            SType::Smart => "smart",
            SType::ProductId => "product_id",
        }
    }
}

impl Display for SType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Record types, possible values for [`RecordHeader::rtype`][crate::record::RecordHeader::rtype]
pub mod rtype {
    use super::Schema;

    /// Market by price with a book depth of 0 (used for trades).
    pub const MBP_0: u8 = 0x00;
    /// Market by price with a book depth of 1 (also used for TBBO).
    pub const MBP_1: u8 = 0x01;
    /// Market by price with a book depth of 10.
    pub const MBP_10: u8 = 0x0A;
    /// Open, high, low, close, and volume at an unspecified cadence.
    #[deprecated(
        since = "0.3.3",
        note = "Separated into separate rtypes for each OHLCV schema."
    )]
    pub const OHLCV_DEPRECATED: u8 = 0x11;
    /// Open, high, low, close, and volume at a 1-second cadence.
    pub const OHLCV_1S: u8 = 0x20;
    /// Open, high, low, close, and volume at a 1-minute cadence.
    pub const OHLCV_1M: u8 = 0x21;
    /// Open, high, low, close, and volume at an hourly cadence.
    pub const OHLCV_1H: u8 = 0x22;
    /// Open, high, low, close, and volume at a daily cadence.
    pub const OHLCV_1D: u8 = 0x23;
    /// Exchange status.
    pub const STATUS: u8 = 0x12;
    /// Instrument definition.
    pub const INSTRUMENT_DEF: u8 = 0x13;
    /// Order imbalance.
    pub const IMBALANCE: u8 = 0x14;
    /// Error from gateway.
    pub const ERROR: u8 = 0x15;
    /// Symbol mapping.
    pub const SYMBOL_MAPPING: u8 = 0x16;
    /// A non-error message. Also used for heartbeats.
    pub const SYSTEM: u8 = 0x17;
    /// Market by order.
    pub const MBO: u8 = 0xA0;

    /// Get the corresponding `rtype` for the given `schema`.
    pub fn from(schema: Schema) -> u8 {
        match schema {
            Schema::Mbo => MBO,
            Schema::Mbp1 => MBP_1,
            Schema::Mbp10 => MBP_10,
            Schema::Tbbo => MBP_1,
            Schema::Trades => MBP_0,
            Schema::Ohlcv1S => OHLCV_1S,
            Schema::Ohlcv1M => OHLCV_1M,
            Schema::Ohlcv1H => OHLCV_1H,
            Schema::Ohlcv1D => OHLCV_1D,
            Schema::Definition => INSTRUMENT_DEF,
            Schema::Statistics => unimplemented!("Statistics is not yet supported"),
            Schema::Status => STATUS,
            Schema::Imbalance => IMBALANCE,
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
            STATUS => Some(Schema::Status),
            INSTRUMENT_DEF => Some(Schema::Definition),
            IMBALANCE => Some(Schema::Imbalance),
            MBO => Some(Schema::Mbo),
            _ => None,
        }
    }
}

/// A data record schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u16)]
pub enum Schema {
    /// Market by order.
    Mbo = 0,
    /// Market by price with a book depth of 1.
    Mbp1 = 1,
    /// Market by price with a book depth of 10.
    Mbp10 = 2,
    /// All trade events with the best bid and offer (BBO) immediately **before** the
    /// effect of the trade.
    Tbbo = 3,
    /// All trade events.
    Trades = 4,
    /// Open, high, low, close, and volume at a one-second interval.
    Ohlcv1S = 5,
    /// Open, high, low, close, and volume at a one-minute interval.
    Ohlcv1M = 6,
    /// Open, high, low, close, and volume at an hourly interval.
    Ohlcv1H = 7,
    /// Open, high, low, close, and volume at a daily interval.
    Ohlcv1D = 8,
    /// Instrument definitions.
    Definition = 9,
    #[doc(hidden)]
    Statistics = 10,
    /// Exchange status.
    #[doc(hidden)]
    Status = 11,
    /// Auction imbalance events.
    Imbalance = 12,
}

/// The number of [`Schema`]s.
pub const SCHEMA_COUNT: usize = 12;

impl std::str::FromStr for Schema {
    type Err = ConversionError;

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
            "definition" => Ok(Schema::Definition),
            "statistics" => Ok(Schema::Statistics),
            "status" => Ok(Schema::Status),
            "imbalance" => Ok(Schema::Imbalance),
            _ => Err(ConversionError::TypeConversion(
                "Value doesn't match a valid schema",
            )),
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
            Schema::Definition => "definition",
            Schema::Statistics => "statistics",
            Schema::Status => "status",
            Schema::Imbalance => "imbalance",
        }
    }
}

impl Serialize for Schema {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl Display for Schema {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data encoding format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TryFromPrimitive)]
#[serde(rename_all = "lowercase")]
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
    type Err = ConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dbn" | "dbz" => Ok(Encoding::Dbn),
            "csv" => Ok(Encoding::Csv),
            "json" => Ok(Encoding::Json),
            _ => Err(ConversionError::TypeConversion(
                "Value doesn't match a valid encoding",
            )),
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TryFromPrimitive)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum Compression {
    /// Uncompressed.
    None = 0,
    /// Zstandard compressed.
    ZStd = 1,
}

impl std::str::FromStr for Compression {
    type Err = ConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Compression::None),
            "zstd" => Ok(Compression::ZStd),
            _ => Err(ConversionError::TypeConversion(
                "Value doesn't match a valid compression",
            )),
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
    /// `product_id`.
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

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[doc(hidden)]
pub enum SecurityUpdateAction {
    Add = b'A',
    Modify = b'M',
    Delete = b'D',
    #[deprecated = "Still present in legacy files."]
    Invalid = b'~',
}

impl Serialize for SecurityUpdateAction {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_char(char::from(*self as u8))
    }
}
