//! A crate for reading DBZ files and converting them to other [OutputEncoding]s.
#[deny(missing_docs)]
#[deny(rustdoc::broken_intra_doc_links)]
#[deny(clippy::missing_errors_doc)]
#[forbid(unsafe_code)]
mod read;
mod write;

#[cfg(feature = "python")]
pub mod python;

pub use crate::read::{Dbz, DbzIntoIter, MappingInterval, Metadata, SymbolMapping};
pub use crate::write::OutputEncoding;
