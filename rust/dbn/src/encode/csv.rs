//! Encoding of DBN records into comma-separated values (CSV).

pub(crate) mod serialize;
mod sync;
pub mod v4;

pub use sync::{Encoder, EncoderBuilder};
