//! The official crate for working with [**D**atabento](https://databento.com)
//! **B**inary E**n**coding (DBN), an extremely fast message encoding and storage format
//! for normalized market data. The DBN specification includes a simple, self-describing
//! metadata header and a fixed set of struct definitions, which enforce a standardized
//! way to normalize market data.
//!
//! All official Databento client libraries use DBN under the hood, both as a data
//! interchange format and for in-memory representation of data. DBN is also the default
//! encoding for all Databento APIs, including live data streaming, historical data
//! streaming, and batch flat files. For more information about the encoding, read our
//! [introduction to DBN](https://databento.com/docs/standards-and-conventions/databento-binary-encoding).
//!
//! The crate supports reading and writing DBN files and streams, as well as converting
//! them to other [`Encoding`]s. It can also be used to update legacy
//! DBZ files to DBN.
//!
//! This crate provides:
//! - [Decoders](crate::decode) for DBN and DBZ (the precursor to DBN), both
//!   sync and async, with the `async` feature flag
//! - [Encoders](crate::encode) for CSV, DBN, and JSON, both sync and async,
//!   with the `async` feature flag
//! - [Normalized market data struct definitions](crate::record) corresponding to the
//!   different market data schemas offered by Databento
//! - A [wrapper type](crate::RecordRef) for holding a reference to a record struct of
//!   a dynamic type
//! - Helper functions and [macros] for common tasks
//!
//! # Feature flags
//! - `async`: enables async decoding and encoding
//! - `python`: enables `pyo3` bindings
//! - `serde`: enables deriving `serde` traits for types
//! - `trivial_copy`: enables deriving the `Copy` trait for records

// Experimental feature to allow docs.rs to display features
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::missing_errors_doc)]

pub mod compat;
pub mod decode;
pub mod encode;
pub mod enums;
pub mod error;
pub mod flags;
mod json_writer;
pub mod macros;
pub mod metadata;
pub mod pretty;
pub mod publishers;
#[cfg(feature = "python")]
pub mod python;
pub mod record;
mod record_enum;
pub mod record_ref;
pub mod symbol_map;
#[cfg(test)]
mod test_utils;
pub mod v1;
pub mod v2;
pub mod v3;

pub use crate::{
    enums::{
        rtype, Action, Compression, Encoding, ErrorCode, InstrumentClass, MatchAlgorithm, RType,
        SType, Schema, SecurityUpdateAction, Side, StatType, StatUpdateAction, StatusAction,
        StatusReason, SystemCode, TradingEvent, TriState, UserDefinedInstrument,
        VersionUpgradePolicy,
    },
    error::{Error, Result},
    flags::FlagSet,
    metadata::{MappingInterval, Metadata, MetadataBuilder, SymbolMapping},
    publishers::{Dataset, Publisher, Venue},
    record::{
        Bbo1MMsg, Bbo1SMsg, BboMsg, BidAskPair, Cbbo1MMsg, Cbbo1SMsg, CbboMsg, Cmbp1Msg,
        ConsolidatedBidAskPair, ErrorMsg, HasRType, ImbalanceMsg, InstrumentDefMsg, MboMsg,
        Mbp10Msg, Mbp1Msg, OhlcvMsg, Record, RecordHeader, RecordMut, StatMsg, StatusMsg,
        SymbolMappingMsg, SystemMsg, TbboMsg, TcbboMsg, TradeMsg, WithTsOut,
    },
    record_enum::{RecordEnum, RecordRefEnum},
    record_ref::RecordRef,
    symbol_map::{PitSymbolMap, SymbolIndex, TsSymbolMap},
};

/// The current version of the DBN encoding, which is different from the crate version.
pub const DBN_VERSION: u8 = 3;

/// The length of fixed-length symbol strings.
pub const SYMBOL_CSTR_LEN: usize = v3::SYMBOL_CSTR_LEN;
/// The length of the fixed-length asset string.
pub const ASSET_CSTR_LEN: usize = v3::ASSET_CSTR_LEN;

const METADATA_DATASET_CSTR_LEN: usize = 16;
const METADATA_RESERVED_LEN: usize = 53;
/// Excludes magic string, version, and length.
const METADATA_FIXED_LEN: usize = 100;
const NULL_LIMIT: u64 = 0;
const NULL_RECORD_COUNT: u64 = u64::MAX;
const NULL_SCHEMA: u16 = u16::MAX;
const NULL_STYPE: u8 = u8::MAX;

/// The denominator of fixed prices in DBN.
pub const FIXED_PRICE_SCALE: i64 = 1_000_000_000;
/// The sentinel value for an unset or null price.
pub const UNDEF_PRICE: i64 = i64::MAX;
/// The sentinel value for an unset or null order quantity.
pub const UNDEF_ORDER_SIZE: u32 = u32::MAX;
/// The sentinel value for an unset or null stat quantity.
pub const UNDEF_STAT_QUANTITY: i64 = v3::UNDEF_STAT_QUANTITY;
/// The sentinel value for an unset or null timestamp.
pub const UNDEF_TIMESTAMP: u64 = u64::MAX;
/// The length in bytes of the largest record type.
pub const MAX_RECORD_LEN: usize = std::mem::size_of::<WithTsOut<v3::InstrumentDefMsg>>();

/// New type for validating DBN versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct DbnVersion(u8);

impl TryFrom<u8> for DbnVersion {
    type Error = crate::Error;

    fn try_from(version: u8) -> crate::Result<Self> {
        if (1..=DBN_VERSION).contains(&version) {
            Ok(Self(version))
        } else {
            Err(Error::BadArgument {
                param_name: "version".to_owned(),
                desc: format!("invalid, must be between 1 and {DBN_VERSION}, inclusive"),
            })
        }
    }
}

impl DbnVersion {
    /// Returns the version value.
    pub fn get(self) -> u8 {
        self.0
    }
}

impl PartialEq<u8> for DbnVersion {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<u8> for DbnVersion {
    fn partial_cmp(&self, other: &u8) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}
