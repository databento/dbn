//! Enums used in Databento APIs.
#![allow(deprecated)] // TODO: remove with SType::Smart
#![allow(clippy::manual_non_exhaustive)] // false positive

mod methods;

use std::fmt::{self, Display, Formatter};

// Dummy derive macro to get around `cfg_attr` incompatibility of several
// of pyo3's attribute macros. See https://github.com/PyO3/pyo3/issues/780
#[cfg(not(feature = "python"))]
use dbn_macros::MockPyo3;
use num_enum::{IntoPrimitive, TryFromPrimitive};

/// A [record type](https://databento.com/docs/standards-and-conventions/common-fields-enums-types),
/// i.e. a sentinel for different types implementing [`HasRType`](crate::record::HasRType).
///
/// Use in [`RecordHeader`](crate::RecordHeader) to indicate the type of record,
/// which is useful when working with DBN streams containing multiple record types.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive, IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[repr(u8)]
pub enum RType {
    /// Denotes a market-by-price record with a book depth of 0 (used for the
    /// [`Trades`](super::Schema::Trades) schema).
    #[pyo3(name = "MBP_0")]
    Mbp0 = 0x00,
    /// Denotes a market-by-price record with a book depth of 1 (also used for the
    /// [`Tbbo`](super::Schema::Tbbo) schema).
    #[pyo3(name = "MBP_1")]
    Mbp1 = 0x01,
    /// Denotes a market-by-price record with a book depth of 10.
    #[pyo3(name = "MBP_10")]
    Mbp10 = 0x0A,
    /// Denotes an open, high, low, close, and volume record at an unspecified cadence.
    #[deprecated(
        since = "0.3.3",
        note = "Separated into separate rtypes for each OHLCV schema."
    )]
    #[pyo3(name = "OHLCV_DEPRECATED")]
    OhlcvDeprecated = 0x11,
    /// Denotes an open, high, low, close, and volume record at a 1-second cadence.
    #[pyo3(name = "OHLCV_1S")]
    Ohlcv1S = 0x20,
    /// Denotes an open, high, low, close, and volume record at a 1-minute cadence.
    #[pyo3(name = "OHLCV_1M")]
    Ohlcv1M = 0x21,
    /// Denotes an open, high, low, close, and volume record at an hourly cadence.
    #[pyo3(name = "OHLCV_1H")]
    Ohlcv1H = 0x22,
    /// Denotes an open, high, low, close, and volume record at a daily cadence
    /// based on the UTC date.
    #[pyo3(name = "OHLCV_1D")]
    Ohlcv1D = 0x23,
    /// Denotes an open, high, low, close, and volume record at a daily cadence
    /// based on the end of the trading session.
    #[pyo3(name = "OHLCV_EOD")]
    OhlcvEod = 0x24,
    /// Denotes an exchange status record.
    #[pyo3(name = "STATUS")]
    Status = 0x12,
    /// Denotes an instrument definition record.
    #[pyo3(name = "INSTRUMENT_DEF")]
    InstrumentDef = 0x13,
    /// Denotes an order imbalance record.
    #[pyo3(name = "IMBALANCE")]
    Imbalance = 0x14,
    /// Denotes an error from gateway.
    #[pyo3(name = "ERROR")]
    Error = 0x15,
    /// Denotes a symbol mapping record.
    #[pyo3(name = "SYMBOL_MAPPING")]
    SymbolMapping = 0x16,
    /// Denotes a non-error message from the gateway. Also used for heartbeats.
    #[pyo3(name = "SYSTEM")]
    System = 0x17,
    /// Denotes a statistics record from the publisher (not calculated by Databento).
    #[pyo3(name = "STATISTICS")]
    Statistics = 0x18,
    /// Denotes a market-by-order record.
    #[pyo3(name = "MBO")]
    Mbo = 0xA0,
    /// Denotes a consolidated best bid and offer record.
    #[pyo3(name = "CMBP_1")]
    Cmbp1 = 0xB1,
    /// Denotes a consolidated best bid and offer record subsampled on a one-second
    /// interval.
    #[pyo3(name = "CBBO_1S")]
    Cbbo1S = 0xC0,
    /// Denotes a consolidated best bid and offer record subsampled on a one-minute
    /// interval.
    #[pyo3(name = "CBBO_1M")]
    Cbbo1M = 0xC1,
    /// Denotes a consolidated best bid and offer trade record containing the
    /// consolidated BBO before the trade
    #[pyo3(name = "TCBBO")]
    Tcbbo = 0xC2,
    /// Denotes a best bid and offer record subsampled on a one-second interval.
    #[pyo3(name = "BBO_1S")]
    Bbo1S = 0xC3,
    /// Denotes a best bid and offer record subsampled on a one-minute interval.
    #[pyo3(name = "BBO_1M")]
    Bbo1M = 0xC4,
}

/// Record types, possible values for [`RecordHeader::rtype`][crate::RecordHeader::rtype].
pub mod rtype {
    use super::*;
    /// Denotes a market-by-price record with a book depth of 0 (used for the
    /// [`Trades`](super::Schema::Trades) schema).
    pub const MBP_0: u8 = RType::Mbp0 as u8;
    /// Denotes a market-by-price record with a book depth of 1 (also used for the
    /// [`Tbbo`](super::Schema::Tbbo) schema).
    pub const MBP_1: u8 = RType::Mbp1 as u8;
    /// Denotes a market-by-price record with a book depth of 10.
    pub const MBP_10: u8 = RType::Mbp10 as u8;
    /// Denotes an open, high, low, close, and volume record at an unspecified cadence.#[deprecated(since = "0.3.3", note = "Separated into separate rtypes for each OHLCV schema.")]
    pub const OHLCV_DEPRECATED: u8 = RType::OhlcvDeprecated as u8;
    /// Denotes an open, high, low, close, and volume record at a 1-second cadence.
    pub const OHLCV_1S: u8 = RType::Ohlcv1S as u8;
    /// Denotes an open, high, low, close, and volume record at a 1-minute cadence.
    pub const OHLCV_1M: u8 = RType::Ohlcv1M as u8;
    /// Denotes an open, high, low, close, and volume record at an hourly cadence.
    pub const OHLCV_1H: u8 = RType::Ohlcv1H as u8;
    /// Denotes an open, high, low, close, and volume record at a daily cadence
    /// based on the UTC date.
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
    pub const CMBP_1: u8 = RType::Cmbp1 as u8;
    /// Denotes a consolidated best bid and offer record subsampled on a one-second
    /// interval.
    pub const CBBO_1S: u8 = RType::Cbbo1S as u8;
    /// Denotes a consolidated best bid and offer record subsampled on a one-minute
    /// interval.
    pub const CBBO_1M: u8 = RType::Cbbo1M as u8;
    /// Denotes a consolidated best bid and offer trade record containing the
    /// consolidated BBO before the trade
    pub const TCBBO: u8 = RType::Tcbbo as u8;
    /// Denotes a best bid and offer record subsampled on a one-second interval.
    pub const BBO_1S: u8 = RType::Bbo1S as u8;
    /// Denotes a best bid and offer record subsampled on a one-minute interval.
    pub const BBO_1M: u8 = RType::Bbo1M as u8;
}

impl std::str::FromStr for RType {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mbp-0" => Ok(Self::Mbp0),
            "mbp-1" => Ok(Self::Mbp1),
            "mbp-10" => Ok(Self::Mbp10),
            #[allow(deprecated)]
            "ohlcv-deprecated" => Ok(Self::OhlcvDeprecated),
            "ohlcv-1s" => Ok(Self::Ohlcv1S),
            "ohlcv-1m" => Ok(Self::Ohlcv1M),
            "ohlcv-1h" => Ok(Self::Ohlcv1H),
            "ohlcv-1d" => Ok(Self::Ohlcv1D),
            "ohlcv-eod" => Ok(Self::OhlcvEod),
            "status" => Ok(Self::Status),
            "instrument-def" => Ok(Self::InstrumentDef),
            "imbalance" => Ok(Self::Imbalance),
            "error" => Ok(Self::Error),
            "symbol-mapping" => Ok(Self::SymbolMapping),
            "system" => Ok(Self::System),
            "statistics" => Ok(Self::Statistics),
            "mbo" => Ok(Self::Mbo),
            "cmbp-1" => Ok(Self::Cmbp1),
            "cbbo-1s" => Ok(Self::Cbbo1S),
            "cbbo-1m" => Ok(Self::Cbbo1M),
            "tcbbo" => Ok(Self::Tcbbo),
            "bbo-1s" => Ok(Self::Bbo1S),
            "bbo-1m" => Ok(Self::Bbo1M),
            _ => Err(crate::Error::conversion::<Self>(s.to_owned())),
        }
    }
}

impl AsRef<str> for RType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl RType {
    /// Converts the `RType` to its `str` representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Mbp0 => "mbp-0",
            Self::Mbp1 => "mbp-1",
            Self::Mbp10 => "mbp-10",
            #[allow(deprecated)]
            Self::OhlcvDeprecated => "ohlcv-deprecated",
            Self::Ohlcv1S => "ohlcv-1s",
            Self::Ohlcv1M => "ohlcv-1m",
            Self::Ohlcv1H => "ohlcv-1h",
            Self::Ohlcv1D => "ohlcv-1d",
            Self::OhlcvEod => "ohlcv-eod",
            Self::Status => "status",
            Self::InstrumentDef => "instrument-def",
            Self::Imbalance => "imbalance",
            Self::Error => "error",
            Self::SymbolMapping => "symbol-mapping",
            Self::System => "system",
            Self::Statistics => "statistics",
            Self::Mbo => "mbo",
            Self::Cmbp1 => "cmbp-1",
            Self::Cbbo1S => "cbbo-1s",
            Self::Cbbo1M => "cbbo-1m",
            Self::Tcbbo => "tcbbo",
            Self::Bbo1S => "bbo-1s",
            Self::Bbo1M => "bbo-1m",
        }
    }
}

impl Display for RType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A [side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types)
/// of the market. The side of the market for resting orders, or the side of the
/// aggressor for trades.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum Side {
    /// A sell order or sell aggressor in a trade.
    #[pyo3(name = "ASK")]
    Ask = b'A',
    /// A buy order or a buy aggressor in a trade.
    #[pyo3(name = "BID")]
    Bid = b'B',
    /// No side specified by the original source.
    #[default]
    #[pyo3(name = "NONE")]
    None = b'N',
}

impl From<Side> for char {
    fn from(value: Side) -> Self {
        u8::from(value) as char
    }
}

/// An [order event or order book operation](https://databento.com/docs/api-reference-historical/basics/schemas-and-conventions).
///
/// For example usage see:
/// - [Order actions](https://databento.com/docs/examples/order-book/order-actions)
/// - [Order tracking](https://databento.com/docs/examples/order-book/order-tracking)
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum Action {
    /// An existing order was modified: price and/or size.
    #[pyo3(name = "MODIFY")]
    Modify = b'M',
    /// An aggressing order traded. Does not affect the book.
    #[pyo3(name = "TRADE")]
    Trade = b'T',
    /// An existing order was filled. Does not affect the book.
    #[pyo3(name = "FILL")]
    Fill = b'F',
    /// An order was fully or partially cancelled.
    #[pyo3(name = "CANCEL")]
    Cancel = b'C',
    /// A new order was added to the book.
    #[pyo3(name = "ADD")]
    Add = b'A',
    /// Reset the book; clear all orders for an instrument.
    #[pyo3(name = "CLEAR")]
    Clear = b'R',
    /// Has no effect on the book, but may carry `flags` or other information.
    #[default]
    #[pyo3(name = "NONE")]
    None = b'N',
}

impl From<Action> for char {
    fn from(value: Action) -> Self {
        u8::from(value) as char
    }
}

/// The class of instrument.
///
/// For example usage see
/// [Getting options with their underlying](https://databento.com/docs/examples/options/options-and-futures).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive, IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
#[repr(u8)]
pub enum InstrumentClass {
    /// A bond.
    #[pyo3(name = "BOND")]
    Bond = b'B',
    /// A call option.
    #[pyo3(name = "CALL")]
    Call = b'C',
    /// A future.
    #[pyo3(name = "FUTURE")]
    Future = b'F',
    /// A stock.
    #[pyo3(name = "STOCK")]
    Stock = b'K',
    /// A spread composed of multiple instrument classes.
    #[pyo3(name = "MIXED_SPREAD")]
    MixedSpread = b'M',
    /// A put option.
    #[pyo3(name = "PUT")]
    Put = b'P',
    /// A spread composed of futures.
    #[pyo3(name = "FUTURE_SPREAD")]
    FutureSpread = b'S',
    /// A spread composed of options.
    #[pyo3(name = "OPTION_SPREAD")]
    OptionSpread = b'T',
    /// A foreign exchange spot.
    #[pyo3(name = "FX_SPOT")]
    FxSpot = b'X',
    /// A commodity being traded for immediate delivery.
    #[pyo3(name = "COMMODITY_SPOT")]
    CommoditySpot = b'Y',
}

impl From<InstrumentClass> for char {
    fn from(value: InstrumentClass) -> Self {
        u8::from(value) as char
    }
}

/// The type of matching algorithm used for the instrument at the exchange.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum MatchAlgorithm {
    /// No matching algorithm was specified.
    #[default]
    #[pyo3(name = "UNDEFINED")]
    Undefined = b' ',
    /// First-in-first-out matching.
    #[pyo3(name = "FIFO")]
    Fifo = b'F',
    /// A configurable match algorithm.
    #[pyo3(name = "CONFIGURABLE")]
    Configurable = b'K',
    /// Trade quantity is allocated to resting orders based on a pro-rata percentage:
    /// resting order quantity divided by total quantity.
    #[pyo3(name = "PRO_RATA")]
    ProRata = b'C',
    /// Like [`Self::Fifo`] but with LMM allocations prior to FIFO allocations.
    #[pyo3(name = "FIFO_LMM")]
    FifoLmm = b'T',
    /// Like [`Self::ProRata`] but includes a configurable allocation to the first order that
    /// improves the market.
    #[pyo3(name = "THRESHOLD_PRO_RATA")]
    ThresholdProRata = b'O',
    /// Like [`Self::FifoLmm`] but includes a configurable allocation to the first order that
    /// improves the market.
    #[pyo3(name = "FIFO_TOP_LMM")]
    FifoTopLmm = b'S',
    /// Like [`Self::ThresholdProRata`] but includes a special priority to LMMs.
    #[pyo3(name = "THRESHOLD_PRO_RATA_LMM")]
    ThresholdProRataLmm = b'Q',
    /// Special variant used only for Eurodollar futures on CME.
    #[pyo3(name = "EURODOLLAR_FUTURES")]
    EurodollarFutures = b'Y',
    /// Trade quantity is shared between all orders at the best price. Orders with the
    /// highest time priority receive a higher matched quantity.
    #[pyo3(name = "TIME_PRO_RATA")]
    TimeProRata = b'P',
    /// A two-pass FIFO algorithm. The first pass fills the Institutional Group the aggressing
    /// order is associated with. The second pass matches orders without an Institutional Group
    /// association. See [CME documentation](https://cmegroupclientsite.atlassian.net/wiki/spaces/EPICSANDBOX/pages/457217267#InstitutionalPrioritizationMatchAlgorithm).
    #[pyo3(name = "INSTITUTIONAL_PRIORITIZATION")]
    InstitutionalPrioritization = b'V',
}

impl From<MatchAlgorithm> for char {
    fn from(value: MatchAlgorithm) -> Self {
        u8::from(value) as char
    }
}

/// Whether the instrument is user-defined.
///
/// For example usage see
/// [Getting options with their underlying](https://databento.com/docs/examples/options/options-and-futures).
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum UserDefinedInstrument {
    /// The instrument is not user-defined.
    #[default]
    #[pyo3(name = "NO")]
    No = b'N',
    /// The instrument is user-defined.
    #[pyo3(name = "YES")]
    Yes = b'Y',
}

impl From<UserDefinedInstrument> for char {
    fn from(value: UserDefinedInstrument) -> Self {
        u8::from(value) as char
    }
}

/// The type of [`InstrumentDefMsg`](crate::record::InstrumentDefMsg) update.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum SecurityUpdateAction {
    /// A new instrument definition.
    #[default]
    #[pyo3(name = "ADD")]
    Add = b'A',
    /// A modified instrument definition of an existing one.
    #[pyo3(name = "MODIFY")]
    Modify = b'M',
    /// Removal of an instrument definition.
    #[pyo3(name = "DELETE")]
    Delete = b'D',
    #[doc(hidden)]
    #[deprecated(since = "0.3.0", note = "Still present in legacy files.")]
    #[pyo3(name = "INVALID")]
    Invalid = b'~',
}

impl From<SecurityUpdateAction> for char {
    fn from(value: SecurityUpdateAction) -> Self {
        u8::from(value) as char
    }
}

/// A symbology type. Refer to the
/// [symbology documentation](https://databento.com/docs/api-reference-historical/basics/symbology)
/// for more information.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive, IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[repr(u8)]
pub enum SType {
    /// Symbology using a unique numeric ID.
    #[pyo3(name = "INSTRUMENT_ID")]
    InstrumentId = 0,
    /// Symbology using the original symbols provided by the publisher.
    #[pyo3(name = "RAW_SYMBOL")]
    RawSymbol = 1,
    /// A set of Databento-specific symbologies for referring to groups of symbols.
    #[deprecated(since = "0.5.0", note = "Smart was split into continuous and parent.")]
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
    #[pyo3(name = "NASDAQ_SYMBOL")]
    NasdaqSymbol = 5,
    /// Symbology for US equities using CMS suffix conventions.
    #[pyo3(name = "CMS_SYMBOL")]
    CmsSymbol = 6,
    /// Symbology using International Security Identification Numbers (ISIN) - ISO 6166.
    #[pyo3(name = "ISIN")]
    Isin = 7,
    /// Symbology using US domestic Committee on Uniform Securities Identification Procedure (CUSIP) codes.
    #[pyo3(name = "US_CODE")]
    UsCode = 8,
    /// Symbology using Bloomberg composite global IDs.
    #[pyo3(name = "BBG_COMP_ID")]
    BbgCompId = 9,
    /// Symbology using Bloomberg composite tickers.
    #[pyo3(name = "BBG_COMP_TICKER")]
    BbgCompTicker = 10,
    /// Symbology using Bloomberg FIGI exchange level IDs.
    #[pyo3(name = "FIGI")]
    Figi = 11,
    /// Symbology using Bloomberg exchange level tickers.
    #[pyo3(name = "FIGI_TICKER")]
    FigiTicker = 12,
}

impl std::str::FromStr for SType {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "instrument_id" | "product_id" => Ok(Self::InstrumentId),
            "raw_symbol" | "native" => Ok(Self::RawSymbol),
            #[allow(deprecated)]
            "smart" => Ok(Self::Smart),
            "continuous" => Ok(Self::Continuous),
            "parent" => Ok(Self::Parent),
            "nasdaq_symbol" | "nasdaq" => Ok(Self::NasdaqSymbol),
            "cms_symbol" | "cms" => Ok(Self::CmsSymbol),
            "isin" => Ok(Self::Isin),
            "us_code" => Ok(Self::UsCode),
            "bbg_comp_id" => Ok(Self::BbgCompId),
            "bbg_comp_ticker" => Ok(Self::BbgCompTicker),
            "figi" => Ok(Self::Figi),
            "figi_ticker" => Ok(Self::FigiTicker),
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
    /// Converts the `SType` to its `str` representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::InstrumentId => "instrument_id",
            Self::RawSymbol => "raw_symbol",
            #[allow(deprecated)]
            Self::Smart => "smart",
            Self::Continuous => "continuous",
            Self::Parent => "parent",
            Self::NasdaqSymbol => "nasdaq_symbol",
            Self::CmsSymbol => "cms_symbol",
            Self::Isin => "isin",
            Self::UsCode => "us_code",
            Self::BbgCompId => "bbg_comp_id",
            Self::BbgCompTicker => "bbg_comp_ticker",
            Self::Figi => "figi",
            Self::FigiTicker => "figi_ticker",
        }
    }
}

impl Display for SType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data record schema.
///
/// Each schema has a particular [record](crate::record) type associated with it.
///
/// See [List of supported market data schemas](https://databento.com/docs/schemas-and-data-formats/whats-a-schema)
/// for an overview of the differences and use cases of each schema.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive, IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
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
    /// All trade events with the best bid and offer (BBO) immediately **before** the effect of
    /// the trade.
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
    /// Open, high, low, close, and volume at a daily cadence based on the end of the trading
    /// session.
    #[pyo3(name = "OHLCV_EOD")]
    OhlcvEod = 13,
    /// Consolidated best bid and offer.
    #[pyo3(name = "CMBP_1")]
    Cmbp1 = 14,
    /// Consolidated best bid and offer subsampled at one-second intervals, in addition to
    /// trades.
    #[pyo3(name = "CBBO_1S")]
    Cbbo1S = 15,
    /// Consolidated best bid and offer subsampled at one-minute intervals, in addition to
    /// trades.
    #[pyo3(name = "CBBO_1M")]
    Cbbo1M = 16,
    /// All trade events with the consolidated best bid and offer (CBBO) immediately **before**
    /// the effect of the trade.
    #[pyo3(name = "TCBBO")]
    Tcbbo = 17,
    /// Best bid and offer subsampled at one-second intervals, in addition to trades.
    #[pyo3(name = "BBO_1S")]
    Bbo1S = 18,
    /// Best bid and offer subsampled at one-minute intervals, in addition to trades.
    #[pyo3(name = "BBO_1M")]
    Bbo1M = 19,
}

impl std::str::FromStr for Schema {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mbo" => Ok(Self::Mbo),
            "mbp-1" => Ok(Self::Mbp1),
            "mbp-10" => Ok(Self::Mbp10),
            "tbbo" => Ok(Self::Tbbo),
            "trades" => Ok(Self::Trades),
            "ohlcv-1s" => Ok(Self::Ohlcv1S),
            "ohlcv-1m" => Ok(Self::Ohlcv1M),
            "ohlcv-1h" => Ok(Self::Ohlcv1H),
            "ohlcv-1d" => Ok(Self::Ohlcv1D),
            "definition" => Ok(Self::Definition),
            "statistics" => Ok(Self::Statistics),
            "status" => Ok(Self::Status),
            "imbalance" => Ok(Self::Imbalance),
            "ohlcv-eod" => Ok(Self::OhlcvEod),
            "cmbp-1" => Ok(Self::Cmbp1),
            "cbbo-1s" => Ok(Self::Cbbo1S),
            "cbbo-1m" => Ok(Self::Cbbo1M),
            "tcbbo" => Ok(Self::Tcbbo),
            "bbo-1s" => Ok(Self::Bbo1S),
            "bbo-1m" => Ok(Self::Bbo1M),
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
    /// Converts the `Schema` to its `str` representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Mbo => "mbo",
            Self::Mbp1 => "mbp-1",
            Self::Mbp10 => "mbp-10",
            Self::Tbbo => "tbbo",
            Self::Trades => "trades",
            Self::Ohlcv1S => "ohlcv-1s",
            Self::Ohlcv1M => "ohlcv-1m",
            Self::Ohlcv1H => "ohlcv-1h",
            Self::Ohlcv1D => "ohlcv-1d",
            Self::Definition => "definition",
            Self::Statistics => "statistics",
            Self::Status => "status",
            Self::Imbalance => "imbalance",
            Self::OhlcvEod => "ohlcv-eod",
            Self::Cmbp1 => "cmbp-1",
            Self::Cbbo1S => "cbbo-1s",
            Self::Cbbo1M => "cbbo-1m",
            Self::Tcbbo => "tcbbo",
            Self::Bbo1S => "bbo-1s",
            Self::Bbo1M => "bbo-1m",
        }
    }

    /// The number of Schema variants.
    pub const COUNT: usize = 20;
}

impl Display for Schema {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A data encoding format.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive, IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[repr(u8)]
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
            "dbn" | "dbz" => Ok(Self::Dbn),
            "csv" => Ok(Self::Csv),
            "json" => Ok(Self::Json),
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
    /// Converts the `Encoding` to its `str` representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Dbn => "dbn",
            Self::Csv => "csv",
            Self::Json => "json",
        }
    }
}

impl Display for Encoding {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A compression format or none if uncompressed.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive, IntoPrimitive,
)]
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
    Zstd = 1,
}

impl std::str::FromStr for Compression {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "zstd" => Ok(Self::Zstd),
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
    /// Converts the `Compression` to its `str` representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Zstd => "zstd",
        }
    }
}

impl Display for Compression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The type of statistic contained in a [`StatMsg`](crate::record::StatMsg).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive, IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[non_exhaustive]
#[repr(u16)]
pub enum StatType {
    /// The price of the first trade of an instrument. `price` will be set.
    /// `quantity` will be set when provided by the venue.
    #[pyo3(name = "OPENING_PRICE")]
    OpeningPrice = 1,
    /// The probable price of the first trade of an instrument published during pre-
    /// open. Both `price` and `quantity` will be set.
    #[pyo3(name = "INDICATIVE_OPENING_PRICE")]
    IndicativeOpeningPrice = 2,
    /// The settlement price of an instrument. `price` will be set and `flags` indicate
    /// whether the price is final or preliminary and actual or theoretical. `ts_ref`
    /// will indicate the trading date of the settlement price.
    #[pyo3(name = "SETTLEMENT_PRICE")]
    SettlementPrice = 3,
    /// The lowest trade price of an instrument during the trading session. `price` will
    /// be set.
    #[pyo3(name = "TRADING_SESSION_LOW_PRICE")]
    TradingSessionLowPrice = 4,
    /// The highest trade price of an instrument during the trading session. `price` will
    /// be set.
    #[pyo3(name = "TRADING_SESSION_HIGH_PRICE")]
    TradingSessionHighPrice = 5,
    /// The number of contracts cleared for an instrument on the previous trading date.
    /// `quantity` will be set. `ts_ref` will indicate the trading date of the volume.
    #[pyo3(name = "CLEARED_VOLUME")]
    ClearedVolume = 6,
    /// The lowest offer price for an instrument during the trading session. `price`
    /// will be set.
    #[pyo3(name = "LOWEST_OFFER")]
    LowestOffer = 7,
    /// The highest bid price for an instrument during the trading session. `price`
    /// will be set.
    #[pyo3(name = "HIGHEST_BID")]
    HighestBid = 8,
    /// The current number of outstanding contracts of an instrument. `quantity` will
    /// be set. `ts_ref` will indicate the trading date for which the open interest was
    /// calculated.
    #[pyo3(name = "OPEN_INTEREST")]
    OpenInterest = 9,
    /// The volume-weighted average price (VWAP) for a fixing period. `price` will be
    /// set.
    #[pyo3(name = "FIXING_PRICE")]
    FixingPrice = 10,
    /// The last trade price during a trading session. `price` will be set.
    /// `quantity` will be set when provided by the venue.
    #[pyo3(name = "CLOSE_PRICE")]
    ClosePrice = 11,
    /// The change in price from the close price of the previous trading session to the
    /// most recent trading session. `price` will be set.
    #[pyo3(name = "NET_CHANGE")]
    NetChange = 12,
    /// The volume-weighted average price (VWAP) during the trading session.
    /// `price` will be set to the VWAP while `quantity` will be the traded
    /// volume.
    #[pyo3(name = "VWAP")]
    Vwap = 13,
    /// The implied volatility associated with the settlement price. `price` will be set
    /// with the standard precision.
    #[pyo3(name = "VOLATILITY")]
    Volatility = 14,
    /// The option delta associated with the settlement price. `price` will be set with
    /// the standard precision.
    #[pyo3(name = "DELTA")]
    Delta = 15,
    /// The auction uncrossing price. This is used for auctions that are neither the
    /// official opening auction nor the official closing auction. `price` will be set.
    /// `quantity` will be set when provided by the venue.
    #[pyo3(name = "UNCROSSING_PRICE")]
    UncrossingPrice = 16,
}

/// The type of [`StatMsg`](crate::record::StatMsg) update.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[non_exhaustive]
#[repr(u8)]
pub enum StatUpdateAction {
    /// A new statistic.
    #[default]
    #[pyo3(name = "NEW")]
    New = 1,
    /// A removal of a statistic.
    #[pyo3(name = "DELETE")]
    Delete = 2,
}

/// The primary enum for the type of [`StatusMsg`](crate::record::StatusMsg) update.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[non_exhaustive]
#[repr(u16)]
pub enum StatusAction {
    /// No change.
    #[default]
    #[pyo3(name = "NONE")]
    None = 0,
    /// The instrument is in a pre-open period.
    #[pyo3(name = "PRE_OPEN")]
    PreOpen = 1,
    /// The instrument is in a pre-cross period.
    #[pyo3(name = "PRE_CROSS")]
    PreCross = 2,
    /// The instrument is quoting but not trading.
    #[pyo3(name = "QUOTING")]
    Quoting = 3,
    /// The instrument is in a cross/auction.
    #[pyo3(name = "CROSS")]
    Cross = 4,
    /// The instrument is being opened through a trading rotation.
    #[pyo3(name = "ROTATION")]
    Rotation = 5,
    /// A new price indication is available for the instrument.
    #[pyo3(name = "NEW_PRICE_INDICATION")]
    NewPriceIndication = 6,
    /// The instrument is trading.
    #[pyo3(name = "TRADING")]
    Trading = 7,
    /// Trading in the instrument has been halted.
    #[pyo3(name = "HALT")]
    Halt = 8,
    /// Trading in the instrument has been paused.
    #[pyo3(name = "PAUSE")]
    Pause = 9,
    /// Trading in the instrument has been suspended.
    #[pyo3(name = "SUSPEND")]
    Suspend = 10,
    /// The instrument is in a pre-close period.
    #[pyo3(name = "PRE_CLOSE")]
    PreClose = 11,
    /// Trading in the instrument has closed.
    #[pyo3(name = "CLOSE")]
    Close = 12,
    /// The instrument is in a post-close period.
    #[pyo3(name = "POST_CLOSE")]
    PostClose = 13,
    /// A change in short-selling restrictions.
    #[pyo3(name = "SSR_CHANGE")]
    SsrChange = 14,
    /// The instrument is not available for trading, either trading has closed or been halted.
    #[pyo3(name = "NOT_AVAILABLE_FOR_TRADING")]
    NotAvailableForTrading = 15,
}

/// The secondary enum for a [`StatusMsg`](crate::record::StatusMsg) update, explains
/// the cause of a halt or other change in `action`.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[non_exhaustive]
#[repr(u16)]
pub enum StatusReason {
    /// No reason is given.
    #[default]
    #[pyo3(name = "NONE")]
    None = 0,
    /// The change in status occurred as scheduled.
    #[pyo3(name = "SCHEDULED")]
    Scheduled = 1,
    /// The instrument stopped due to a market surveillance intervention.
    #[pyo3(name = "SURVEILLANCE_INTERVENTION")]
    SurveillanceIntervention = 2,
    /// The status changed due to activity in the market.
    #[pyo3(name = "MARKET_EVENT")]
    MarketEvent = 3,
    /// The derivative instrument began trading.
    #[pyo3(name = "INSTRUMENT_ACTIVATION")]
    InstrumentActivation = 4,
    /// The derivative instrument expired.
    #[pyo3(name = "INSTRUMENT_EXPIRATION")]
    InstrumentExpiration = 5,
    /// Recovery in progress.
    #[pyo3(name = "RECOVERY_IN_PROCESS")]
    RecoveryInProcess = 6,
    /// The status change was caused by a regulatory action.
    #[pyo3(name = "REGULATORY")]
    Regulatory = 10,
    /// The status change was caused by an administrative action.
    #[pyo3(name = "ADMINISTRATIVE")]
    Administrative = 11,
    /// The status change was caused by the issuer not being compliance with regulatory
    /// requirements.
    #[pyo3(name = "NON_COMPLIANCE")]
    NonCompliance = 12,
    /// Trading halted because the issuer's filings are not current.
    #[pyo3(name = "FILINGS_NOT_CURRENT")]
    FilingsNotCurrent = 13,
    /// Trading halted due to an SEC trading suspension.
    #[pyo3(name = "SEC_TRADING_SUSPENSION")]
    SecTradingSuspension = 14,
    /// The status changed because a new issue is available.
    #[pyo3(name = "NEW_ISSUE")]
    NewIssue = 15,
    /// The status changed because an issue is available.
    #[pyo3(name = "ISSUE_AVAILABLE")]
    IssueAvailable = 16,
    /// The status changed because the issue(s) were reviewed.
    #[pyo3(name = "ISSUES_REVIEWED")]
    IssuesReviewed = 17,
    /// The status changed because the filing requirements were satisfied.
    #[pyo3(name = "FILING_REQS_SATISFIED")]
    FilingReqsSatisfied = 18,
    /// Relevant news is pending.
    #[pyo3(name = "NEWS_PENDING")]
    NewsPending = 30,
    /// Relevant news was released.
    #[pyo3(name = "NEWS_RELEASED")]
    NewsReleased = 31,
    /// The news has been fully disseminated and times are available for the resumption
    /// in quoting and trading.
    #[pyo3(name = "NEWS_AND_RESUMPTION_TIMES")]
    NewsAndResumptionTimes = 32,
    /// The relevant news was not forthcoming.
    #[pyo3(name = "NEWS_NOT_FORTHCOMING")]
    NewsNotForthcoming = 33,
    /// Halted for order imbalance.
    #[pyo3(name = "ORDER_IMBALANCE")]
    OrderImbalance = 40,
    /// The instrument hit limit up or limit down.
    #[pyo3(name = "LULD_PAUSE")]
    LuldPause = 50,
    /// An operational issue occurred with the venue.
    #[pyo3(name = "OPERATIONAL")]
    Operational = 60,
    /// The status changed until the exchange receives additional information.
    #[pyo3(name = "ADDITIONAL_INFORMATION_REQUESTED")]
    AdditionalInformationRequested = 70,
    /// Trading halted due to merger becoming effective.
    #[pyo3(name = "MERGER_EFFECTIVE")]
    MergerEffective = 80,
    /// Trading is halted in an ETF due to conditions with the component securities.
    #[pyo3(name = "ETF")]
    Etf = 90,
    /// Trading is halted for a corporate action.
    #[pyo3(name = "CORPORATE_ACTION")]
    CorporateAction = 100,
    /// Trading is halted because the instrument is a new offering.
    #[pyo3(name = "NEW_SECURITY_OFFERING")]
    NewSecurityOffering = 110,
    /// Halted due to the market-wide circuit breaker level 1.
    #[pyo3(name = "MARKET_WIDE_HALT_LEVEL1")]
    MarketWideHaltLevel1 = 120,
    /// Halted due to the market-wide circuit breaker level 2.
    #[pyo3(name = "MARKET_WIDE_HALT_LEVEL2")]
    MarketWideHaltLevel2 = 121,
    /// Halted due to the market-wide circuit breaker level 3.
    #[pyo3(name = "MARKET_WIDE_HALT_LEVEL3")]
    MarketWideHaltLevel3 = 122,
    /// Halted due to the carryover of a market-wide circuit breaker from the previous
    /// trading day.
    #[pyo3(name = "MARKET_WIDE_HALT_CARRYOVER")]
    MarketWideHaltCarryover = 123,
    /// Resumption due to the end of a market-wide circuit breaker halt.
    #[pyo3(name = "MARKET_WIDE_HALT_RESUMPTION")]
    MarketWideHaltResumption = 124,
    /// Halted because quotation is not available.
    #[pyo3(name = "QUOTATION_NOT_AVAILABLE")]
    QuotationNotAvailable = 130,
}

/// Further information about a status update.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[non_exhaustive]
#[repr(u16)]
pub enum TradingEvent {
    /// No additional information given.
    #[default]
    #[pyo3(name = "NONE")]
    None = 0,
    /// Order entry and modification are not allowed.
    #[pyo3(name = "NO_CANCEL")]
    NoCancel = 1,
    /// A change of trading session occurred. Daily statistics are reset.
    #[pyo3(name = "CHANGE_TRADING_SESSION")]
    ChangeTradingSession = 2,
    /// Implied matching is available.
    #[pyo3(name = "IMPLIED_MATCHING_ON")]
    ImpliedMatchingOn = 3,
    /// Implied matching is not available.
    #[pyo3(name = "IMPLIED_MATCHING_OFF")]
    ImpliedMatchingOff = 4,
}

/// An enum for representing unknown, true, or false values. Equivalent to
/// `Option<bool>` but with a human-readable repr.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum TriState {
    /// The value is not applicable or not known.
    #[default]
    #[pyo3(name = "NOT_AVAILABLE")]
    NotAvailable = b'~',
    /// False.
    #[pyo3(name = "NO")]
    No = b'N',
    /// True.
    #[pyo3(name = "YES")]
    Yes = b'Y',
}

impl From<TriState> for char {
    fn from(value: TriState) -> Self {
        u8::from(value) as char
    }
}

/// How to handle decoding DBN data from other versions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
pub enum VersionUpgradePolicy {
    /// Decode data from all supported versions (less than or equal to
    /// [`DBN_VERSION`](crate::DBN_VERSION)) as-is.
    #[pyo3(name = "AS_IS")]
    AsIs,
    /// Decode and convert data from DBN versions prior to version 2 to that version.
    /// Attempting to decode data from newer versions will fail.
    #[pyo3(name = "UPGRADE_TO_V2")]
    UpgradeToV2,
    /// Decode and convert data from DBN versions prior to version 3 to that version.
    /// Attempting to decode data from newer versions (when they're introduced) will
    /// fail.
    #[default]
    #[pyo3(name = "UPGRADE_TO_V3")]
    UpgradeToV3,
}

/// An error code from the live subscription gateway.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[non_exhaustive]
#[repr(u8)]
pub enum ErrorCode {
    /// The authentication step failed.
    #[pyo3(name = "AUTH_FAILED")]
    AuthFailed = 1,
    /// The user account or API key were deactivated.
    #[pyo3(name = "API_KEY_DEACTIVATED")]
    ApiKeyDeactivated = 2,
    /// The user has exceeded their open connection limit.
    #[pyo3(name = "CONNECTION_LIMIT_EXCEEDED")]
    ConnectionLimitExceeded = 3,
    /// One or more symbols failed to resolve.
    #[pyo3(name = "SYMBOL_RESOLUTION_FAILED")]
    SymbolResolutionFailed = 4,
    /// There was an issue with a subscription request (other than symbol resolution).
    #[pyo3(name = "INVALID_SUBSCRIPTION")]
    InvalidSubscription = 5,
    /// An error occurred in the gateway.
    #[pyo3(name = "INTERNAL_ERROR")]
    InternalError = 6,
    /// No error code was specified or this record was upgraded from a version 1 struct where the code field didn't exist.
    #[default]
    #[pyo3(name = "UNSET")]
    Unset = 255,
}

impl std::str::FromStr for ErrorCode {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auth_failed" => Ok(Self::AuthFailed),
            "api_key_deactivated" => Ok(Self::ApiKeyDeactivated),
            "connection_limit_exceeded" => Ok(Self::ConnectionLimitExceeded),
            "symbol_resolution_failed" => Ok(Self::SymbolResolutionFailed),
            "invalid_subscription" => Ok(Self::InvalidSubscription),
            "internal_error" => Ok(Self::InternalError),
            "unset" => Ok(Self::Unset),
            _ => Err(crate::Error::conversion::<Self>(s.to_owned())),
        }
    }
}

impl AsRef<str> for ErrorCode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl ErrorCode {
    /// Converts the `ErrorCode` to its `str` representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::AuthFailed => "auth_failed",
            Self::ApiKeyDeactivated => "api_key_deactivated",
            Self::ConnectionLimitExceeded => "connection_limit_exceeded",
            Self::SymbolResolutionFailed => "symbol_resolution_failed",
            Self::InvalidSubscription => "invalid_subscription",
            Self::InternalError => "internal_error",
            Self::Unset => "unset",
        }
    }
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A [`SystemMsg`](crate::SystemMsg) code indicating the type of message from the live
/// subscription gateway.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[cfg_attr(
    feature = "python",
    derive(strum::EnumIter),
    pyo3::pyclass(module = "databento_dbn")
)]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))]
#[non_exhaustive]
#[repr(u8)]
pub enum SystemCode {
    /// A message sent in the absence of other records to indicate the connection
    /// remains open.
    #[pyo3(name = "HEARTBEAT")]
    Heartbeat = 0,
    /// An acknowledgement of a subscription request.
    #[pyo3(name = "SUBSCRIPTION_ACK")]
    SubscriptionAck = 1,
    /// The gateway has detected this session is falling behind real-time.
    #[pyo3(name = "SLOW_READER_WARNING")]
    SlowReaderWarning = 2,
    /// Indicates a replay subscription has caught up with real-time data.
    #[pyo3(name = "REPLAY_COMPLETED")]
    ReplayCompleted = 3,
    /// Signals that all records for interval-based schemas have been published for the given timestamp.
    #[pyo3(name = "END_OF_INTERVAL")]
    EndOfInterval = 4,
    /// No system code was specified or this record was upgraded from a version 1 struct where
    /// the code field didn't exist.
    #[default]
    #[pyo3(name = "UNSET")]
    Unset = 255,
}

impl std::str::FromStr for SystemCode {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "heartbeat" => Ok(Self::Heartbeat),
            "subscription_ack" => Ok(Self::SubscriptionAck),
            "slow_reader_warning" => Ok(Self::SlowReaderWarning),
            "replay_completed" => Ok(Self::ReplayCompleted),
            "end_of_interval" => Ok(Self::EndOfInterval),
            "unset" => Ok(Self::Unset),
            _ => Err(crate::Error::conversion::<Self>(s.to_owned())),
        }
    }
}

impl AsRef<str> for SystemCode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl SystemCode {
    /// Converts the `SystemCode` to its `str` representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Heartbeat => "heartbeat",
            Self::SubscriptionAck => "subscription_ack",
            Self::SlowReaderWarning => "slow_reader_warning",
            Self::ReplayCompleted => "replay_completed",
            Self::EndOfInterval => "end_of_interval",
            Self::Unset => "unset",
        }
    }
}

impl Display for SystemCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
mod deserialize {
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer, Serialize};

    use super::*;

    impl<'de> Deserialize<'de> for RType {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let str = String::deserialize(deserializer)?;
            FromStr::from_str(&str).map_err(de::Error::custom)
        }
    }

    impl Serialize for RType {
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

    impl<'de> Deserialize<'de> for ErrorCode {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let str = String::deserialize(deserializer)?;
            FromStr::from_str(&str).map_err(de::Error::custom)
        }
    }

    impl Serialize for ErrorCode {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.as_str().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for SystemCode {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let str = String::deserialize(deserializer)?;
            FromStr::from_str(&str).map_err(de::Error::custom)
        }
    }

    impl Serialize for SystemCode {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.as_str().serialize(serializer)
        }
    }
}
