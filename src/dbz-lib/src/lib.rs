//! A crate for reading DBZ files and converting them to other [OutputEncoding]s.
#[deny(missing_docs)]
#[deny(rustdoc::broken_intra_doc_links)]
#[deny(clippy::missing_errors_doc)]
mod read;
mod write;

#[cfg(feature = "python")]
pub mod python;

pub use crate::read::{Dbz, DbzStreamIter, MappingInterval, Metadata, SymbolMapping};
pub use crate::write::{write_dbz, write_dbz_stream, OutputEncoding};
