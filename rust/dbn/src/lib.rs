//! A crate for reading DBN and legacy DBZ files and converting them to other
//! [`Encoding`](enums::Encoding)s.

#[deny(missing_docs)]
#[deny(rustdoc::broken_intra_doc_links)]
#[deny(clippy::missing_errors_doc)]
pub mod decode;
pub mod encode;
pub mod enums;
pub mod error;
pub mod metadata;
pub mod record;
pub mod record_ref;

#[cfg(feature = "python")]
pub mod python;

pub use crate::metadata::{MappingInterval, Metadata, MetadataBuilder, SymbolMapping};

/// The current version of the DBN encoding, which is different from the crate version.
pub const DBN_VERSION: u8 = 1;
const METADATA_DATASET_CSTR_LEN: usize = 16;
const METADATA_RESERVED_LEN: usize = 47;
/// Excludes magic string, version, and length.
const METADATA_FIXED_LEN: usize = 100;
const SYMBOL_CSTR_LEN: usize = 22;
const NULL_END: u64 = u64::MAX;
const NULL_LIMIT: u64 = 0;
const NULL_RECORD_COUNT: u64 = u64::MAX;
