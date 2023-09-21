//! Encoding of DBN records into comma-separated values (CSV).

pub(crate) mod serialize;
mod sync;

pub use sync::Encoder;
