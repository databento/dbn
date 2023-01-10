//! Enums used in Databento APIs.
use std::fmt::{self, Display, Formatter};
use std::os::raw::c_char;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Serialize;

use crate::error::ConversionError;

/// A side of the market. The side of the market for resting orders, or the side
/// of the aggressor for trades.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    /// A sell order.
    Ask,
    /// A buy order.
    Bid,
    /// None or unknown.
    None,
}

impl From<Side> for char {
    fn from(side: Side) -> Self {
        match side {
            Side::Ask => 'A',
            Side::Bid => 'B',
            Side::None => 'N',
        }
    }
}

impl From<Side> for c_char {
    fn from(side: Side) -> Self {
        char::from(side) as c_char
    }
}

impl Serialize for Side {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_char(char::from(*self))
    }
}

/// A tick action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// An existing order was modified.
    Modify,
    /// A trade executed.
    Trade,
    /// An order was cancelled.
    Cancel,
    /// A new order was added.
    Add,
    /// Reset the book; clear all orders for an instrument.
    Clear,
}

impl From<Action> for char {
    fn from(action: Action) -> Self {
        match action {
            Action::Modify => 'M',
            Action::Trade => 'T',
            Action::Cancel => 'C',
            Action::Add => 'A',
            Action::Clear => 'R',
        }
    }
}

impl From<Action> for c_char {
    fn from(action: Action) -> Self {
        char::from(action) as c_char
    }
}

impl serde::Serialize for Action {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_char(char::from(*self))
    }
}

/// A symbology type. Refer to the [symbology documentation](https://docs.databento.com/reference-historical/basics/symbology)
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

impl SType {
    /// Convert the symbology type to its `str` representation.
    pub fn as_str(&self) -> &'static str {
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
    /// Combination of [Self::Trades] and [Self::Mbp1].
    Tbbo = 3,
    /// All trade events.
    Trades = 4,
    /// Open, high, low, close, and volume at a 1-second cadence.
    Ohlcv1S = 5,
    /// Open, high, low, close, and volume at a 1-minute cadence.
    Ohlcv1M = 6,
    /// Open, high, low, close, and volume at an hourly cadence.
    Ohlcv1H = 7,
    /// Open, high, low, close, and volume at a daily cadence.
    Ohlcv1D = 8,
    /// Symbol definitions.
    Definition = 9,
    ///
    Statistics = 10,
    /// Exchange status.
    Status = 11,
}
/// Update SCHEMA_COUNT value, if add more Schema items
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
            _ => Err(ConversionError::TypeConversion(
                "Value doesn't match a valid schema",
            )),
        }
    }
}

impl Schema {
    /// Converts the given schema to a `&'static str`.
    pub fn as_str(&self) -> &'static str {
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
    /// Databento Binary Encoding + Zstandard compression.
    Dbz = 0,
    /// Comma-separated values.
    Csv = 1,
    /// JavaScript object notation.
    Json = 2,
}

impl std::str::FromStr for Encoding {
    type Err = ConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dbz" => Ok(Encoding::Dbz),
            "csv" => Ok(Encoding::Csv),
            "json" => Ok(Encoding::Json),
            _ => Err(ConversionError::TypeConversion(
                "Value doesn't match a valid encoding",
            )),
        }
    }
}

impl Encoding {
    /// Converts the given encoding to a `&'static str`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Encoding::Dbz => "dbz",
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

impl Compression {
    /// Converts the given compression to a `&'static str`.
    pub fn as_str(&self) -> &'static str {
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

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoPrimitive)]
#[doc(hidden)]
pub enum SecurityUpdateAction {
    Add = b'A',
    Modify = b'M',
    Delete = b'D',
    // Deprecated, but still present in legacy files
    Invalid = b'~',
}

impl Serialize for SecurityUpdateAction {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_char(char::from(*self as u8))
    }
}
