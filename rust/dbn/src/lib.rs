//! A crate for reading DBN and legacy DBZ files and converting them to other
//! [`Encoding`](enums::Encoding)s.

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::missing_errors_doc)]

pub mod decode;
pub mod encode;
pub mod enums;
pub mod error;
#[doc(hidden)]
pub mod json_writer;
pub mod macros;
pub mod metadata;
#[cfg(feature = "python")]
pub mod python;
pub mod record;
pub mod record_ref;

pub use crate::metadata::{MappingInterval, Metadata, MetadataBuilder, SymbolMapping};

/// The current version of the DBN encoding, which is different from the crate version.
pub const DBN_VERSION: u8 = 1;
const METADATA_DATASET_CSTR_LEN: usize = 16;
const METADATA_RESERVED_LEN: usize = 47;
/// Excludes magic string, version, and length.
const METADATA_FIXED_LEN: usize = 100;
const SYMBOL_CSTR_LEN: usize = 22;
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
pub const UNDEF_STAT_QUANTITY: i32 = i32::MAX;
/// The sentinel value for an unset or null timestamp.
pub const UNDEF_TIMESTAMP: u64 = u64::MAX;

/// Contains dataset code constants.
pub mod datasets {
    /// The dataset code for CME Globex MDP 3.0.
    pub const GLBX_MDP3: &str = "GLBX.MDP3";
    /// The dataset code for Nasdaq TotalView ITCH.
    pub const XNAS_ITCH: &str = "XNAS.ITCH";
}
