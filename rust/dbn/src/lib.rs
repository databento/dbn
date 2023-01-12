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

#[cfg(any(feature = "python", feature = "python-test"))]
pub mod python;

pub use crate::metadata::{MappingInterval, Metadata, MetadataBuilder, SymbolMapping};

/// The current version of the DBN encoding.
pub const DBN_VERSION: u8 = 1;
pub(crate) const METADATA_DATASET_CSTR_LEN: usize = 16;
pub(crate) const METADATA_RESERVED_LEN: usize = 48;
/// Excludes magic string, version, and length.
pub(crate) const METADATA_FIXED_LEN: usize = 100;
pub(crate) const SYMBOL_CSTR_LEN: usize = 22;
