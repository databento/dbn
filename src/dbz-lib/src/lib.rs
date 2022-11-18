//! A crate for reading DBZ files and converting them to other [OutputEncoding]s.
#[deny(missing_docs)]
#[deny(rustdoc::broken_intra_doc_links)]
#[deny(clippy::missing_errors_doc)]
mod read;
mod write;

#[cfg(any(feature = "python", feature = "python-test"))]
pub mod python;

pub use crate::read::{Dbz, DbzStreamIter, MappingInterval, Metadata, SymbolMapping};
pub use crate::write::{
    dbz::{write_dbz, write_dbz_stream},
    OutputEncoding,
};
