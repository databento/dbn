//! Encoding DBN records into DBN, Zstandard-compressed or not.
mod sync;
pub use sync::{Encoder, MetadataEncoder, RecordEncoder};

#[cfg(feature = "async")]
mod r#async;
#[cfg(feature = "async")]
pub use r#async::{MetadataEncoder as AsyncMetadataEncoder, RecordEncoder as AsyncRecordEncoder};
