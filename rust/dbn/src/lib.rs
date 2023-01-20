//! A crate for reading DBN files and converting them to other [OutputEncoding]s.
#[deny(missing_docs)]
#[deny(rustdoc::broken_intra_doc_links)]
#[deny(clippy::missing_errors_doc)]
mod read;
mod write;

#[cfg(any(feature = "python", feature = "python-test"))]
pub mod python;

pub use crate::read::{Dbn, DbnStreamIter, MappingInterval, Metadata, SymbolMapping};
pub use crate::write::{
    dbn::{write_dbn, write_dbn_stream},
    OutputEncoding,
};
