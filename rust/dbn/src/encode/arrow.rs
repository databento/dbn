//! Arrow columnar encoding for DBN records.
//!
//! This module provides the [`ArrowSerialize`] trait (derived via the
//! `ArrowSerialize` macro) with [`arrow_schema()`](ArrowSerialize::arrow_schema),
//! the in-memory [`to_record_batch`] helper, and the streaming Arrow IPC [`Encoder`].
//!
//! Use `R = WithTsOut<T>` to get a `ts_out` column populated from each
//! record's wrapper field; bare `R = T` produces a schema without `ts_out`.

pub(crate) mod serialize;
pub(crate) mod sync;
#[cfg(test)]
mod tests;

use std::io::Write;

use ::arrow::datatypes::SchemaRef;
use ::arrow::ipc::writer::{FileWriter, IpcWriteOptions};
use ::arrow::ipc::CompressionType;
use ::arrow::record_batch::RecordBatch;

pub use serialize::ArrowSerialize;

use sync::ColumnarWriter;

use crate::{Compression, Error, Result};

/// The Arrow schema and Parquet key-value metadata key holding the DBN [`Metadata`](crate::Metadata)
/// as JSON, in the same format as [`JsonEncoder::encode_metadata`](crate::encode::JsonEncoder::encode_metadata).
pub(crate) const METADATA_KEY: &str = "databento:metadata";

/// Synchronous Arrow IPC encoder for DBN records (Feather v2).
pub type Encoder<W> = sync::Encoder<IpcFile<W>>;
/// Builder for the Arrow IPC encoder.
pub type EncoderBuilder<W> = sync::EncoderBuilder<IpcFile<W>>;

/// An Arrow IPC file (Feather v2) being written by [`Encoder`].
pub struct IpcFile<W: Write>(FileWriter<W>);

/// Converts a slice of DBN records into an Arrow [`RecordBatch`].
///
/// When `pretty_px` is `true`, price fields are scaled to `f64`.
///
/// # Errors
/// Returns an error if the Arrow batch construction fails.
pub fn to_record_batch<R: ArrowSerialize>(records: &[R], pretty_px: bool) -> Result<RecordBatch> {
    let mut columns = Vec::new();
    R::arrow_columns(records, |r| r, pretty_px, &mut columns);
    RecordBatch::try_new(R::arrow_schema(pretty_px), columns)
        .map_err(|e| Error::arrow(e, "creating RecordBatch"))
}

impl<W: Write + Send> ColumnarWriter for IpcFile<W> {
    type Inner = W;

    fn open(writer: W, schema: SchemaRef, compression: Compression) -> Result<Self> {
        let options = match compression {
            Compression::Zstd => IpcWriteOptions::default()
                .try_with_compression(Some(CompressionType::ZSTD))
                .map_err(|e| Error::arrow(e, "setting Arrow IPC compression"))?,
            Compression::None => IpcWriteOptions::default(),
        };
        FileWriter::try_new_with_options(writer, &schema, options)
            .map(Self)
            .map_err(|e| Error::arrow(e, "creating Arrow IPC writer"))
    }

    fn write(&mut self, batch: &RecordBatch) -> Result<()> {
        self.0
            .write(batch)
            .map_err(|e| Error::arrow(e, "writing Arrow IPC batch"))
    }

    fn flush(&mut self) -> Result<()> {
        self.0
            .get_mut()
            .flush()
            .map_err(|e| Error::io(e, "flushing Arrow IPC writer"))
    }

    fn close(&mut self) -> Result<()> {
        self.0
            .finish()
            .map_err(|e| Error::arrow(e, "finishing Arrow IPC file"))
    }

    fn get_ref(&self) -> &W {
        self.0.get_ref()
    }

    fn get_mut(&mut self) -> &mut W {
        self.0.get_mut()
    }
}
