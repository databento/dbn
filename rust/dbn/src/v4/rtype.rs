//! Record type sentinels for DBN version 4.
#![allow(deprecated)]

/// See [`crate::rtype::MBP_0`].
pub const MBP_0: u16 = crate::rtype::MBP_0 as u16;
/// See [`crate::rtype::MBP_1`].
pub const MBP_1: u16 = crate::rtype::MBP_1 as u16;
/// See [`crate::rtype::MBP_10`].
pub const MBP_10: u16 = crate::rtype::MBP_10 as u16;
/// See [`crate::rtype::OHLCV_DEPRECATED`].
#[deprecated(
    since = "0.3.3",
    note = "Separated into separate rtypes for each OHLCV schema."
)]
pub const OHLCV_DEPRECATED: u16 = crate::rtype::OHLCV_DEPRECATED as u16;
/// See [`crate::rtype::OHLCV_1S`].
pub const OHLCV_1S: u16 = crate::rtype::OHLCV_1S as u16;
/// See [`crate::rtype::OHLCV_1M`].
pub const OHLCV_1M: u16 = crate::rtype::OHLCV_1M as u16;
/// See [`crate::rtype::OHLCV_1H`].
pub const OHLCV_1H: u16 = crate::rtype::OHLCV_1H as u16;
/// See [`crate::rtype::OHLCV_1D`].
pub const OHLCV_1D: u16 = crate::rtype::OHLCV_1D as u16;
/// See [`crate::rtype::OHLCV_EOD`].
pub const OHLCV_EOD: u16 = crate::rtype::OHLCV_EOD as u16;
/// See [`crate::rtype::STATUS`].
pub const STATUS: u16 = crate::rtype::STATUS as u16;
/// See [`crate::rtype::INSTRUMENT_DEF`].
pub const INSTRUMENT_DEF: u16 = crate::rtype::INSTRUMENT_DEF as u16;
/// See [`crate::rtype::IMBALANCE`].
pub const IMBALANCE: u16 = crate::rtype::IMBALANCE as u16;
/// See [`crate::rtype::ERROR`].
pub const ERROR: u16 = crate::rtype::ERROR as u16;
/// See [`crate::rtype::SYMBOL_MAPPING`].
pub const SYMBOL_MAPPING: u16 = crate::rtype::SYMBOL_MAPPING as u16;
/// See [`crate::rtype::SYSTEM`].
pub const SYSTEM: u16 = crate::rtype::SYSTEM as u16;
/// See [`crate::rtype::STATISTICS`].
pub const STATISTICS: u16 = crate::rtype::STATISTICS as u16;
/// See [`crate::rtype::MBO`].
pub const MBO: u16 = crate::rtype::MBO as u16;
/// See [`crate::rtype::CMBP_1`].
pub const CMBP_1: u16 = crate::rtype::CMBP_1 as u16;
/// See [`crate::rtype::CBBO_1S`].
pub const CBBO_1S: u16 = crate::rtype::CBBO_1S as u16;
/// See [`crate::rtype::CBBO_1M`].
pub const CBBO_1M: u16 = crate::rtype::CBBO_1M as u16;
/// See [`crate::rtype::TCBBO`].
pub const TCBBO: u16 = crate::rtype::TCBBO as u16;
/// See [`crate::rtype::BBO_1S`].
pub const BBO_1S: u16 = crate::rtype::BBO_1S as u16;
/// See [`crate::rtype::BBO_1M`].
pub const BBO_1M: u16 = crate::rtype::BBO_1M as u16;
/// See [`crate::rtype::TBBO`].
pub const TBBO: u16 = crate::rtype::TBBO as u16;
