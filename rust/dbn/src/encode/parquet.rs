//! Encoding DBN records into Apache Parquet columnar format.

use std::io::Write;

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::{Compression as Codec, ZstdLevel};
use parquet::file::metadata::KeyValue;
use parquet::file::properties::WriterProperties;

use super::arrow::sync::{self, ColumnarWriter};
use crate::{Compression, Error, Result};

/// Synchronous Parquet encoder for DBN records.
pub type Encoder<W> = sync::Encoder<ParquetFile<W>>;
/// Builder for the Parquet encoder.
pub type EncoderBuilder<W> = sync::EncoderBuilder<ParquetFile<W>>;

/// A Parquet file being written by [`Encoder`].
pub struct ParquetFile<W: Write + Send>(ArrowWriter<W>);

impl<W: Write + Send> ColumnarWriter for ParquetFile<W> {
    type Inner = W;

    fn open(writer: W, schema: SchemaRef, compression: Compression) -> Result<Self> {
        // arrow-rs only embeds the schema metadata inside its own `ARROW:schema`
        // entry, so copy it to plain key-value pairs for non-Arrow readers
        let key_values = schema
            .metadata()
            .iter()
            .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
            .collect();
        let props = WriterProperties::builder()
            .set_compression(codec(compression))
            .set_key_value_metadata(Some(key_values))
            .build();
        ArrowWriter::try_new(writer, schema, Some(props))
            .map(Self)
            .map_err(|e| Error::parquet(e, "creating Parquet writer"))
    }

    fn write(&mut self, batch: &RecordBatch) -> Result<()> {
        self.0
            .write(batch)
            .map_err(|e| Error::parquet(e, "writing row group"))
    }

    fn flush(&mut self) -> Result<()> {
        self.0
            .flush()
            .map_err(|e| Error::parquet(e, "flushing Parquet writer"))?;
        self.0
            .sync()
            .map_err(|e| Error::io(e, "flushing Parquet writer"))
    }

    fn close(&mut self) -> Result<()> {
        self.0
            .finish()
            .map(|_| ())
            .map_err(|e| Error::parquet(e, "closing Parquet writer"))
    }

    fn get_ref(&self) -> &W {
        self.0.inner()
    }

    fn get_mut(&mut self) -> &mut W {
        self.0.inner_mut()
    }
}

fn codec(compression: Compression) -> Codec {
    match compression {
        Compression::None => Codec::UNCOMPRESSED,
        Compression::Zstd => Codec::ZSTD(
            ZstdLevel::try_new(zstd::DEFAULT_COMPRESSION_LEVEL).expect("valid default level"),
        ),
    }
}
