//! Encoding of DBN records into [JSON lines](https://jsonlines.org).

pub(crate) mod serialize;
mod sync;
pub mod v4;
pub use sync::{Encoder, EncoderBuilder};
#[cfg(feature = "async")]
mod r#async;
#[cfg(feature = "async")]
pub use r#async::Encoder as AsyncEncoder;
