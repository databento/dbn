//! Synchronous columnar encoder for DBN records.

use std::any::Any;
use std::collections::HashMap;
use std::io::Write;
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::sync::Arc;

use arrow::array::{ArrayRef, RecordBatch, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema as ArrowSchema, SchemaRef};

use super::serialize::ArrowSerialize;
use super::METADATA_KEY;
use crate::encode::{
    json::serialize::to_json_in_buf, DbnEncodable, EncodeDbn, EncodeRecord, EncodeRecordRef,
    EncodeRecordTextExt,
};
use crate::{
    rtype_dispatch, schema_dispatch, v3, Compression, Error, Metadata, RecordRef, Result, Schema,
    DBN_VERSION,
};

/// Columnar encoder for DBN records. Use
/// [`ArrowEncoder`](crate::encode::ArrowEncoder) (Arrow IPC) or
/// [`ParquetEncoder`](crate::encode::ParquetEncoder).
///
/// The record type and thus the schema are fixed when the encoder is built from
/// the DBN [`Schema`], `ts_out` flag, and version. Encoding a record of any other
/// type returns an error.
pub struct Encoder<B: ColumnarWriter> {
    // `None` once closed
    writer: Option<B>,
    schema: SchemaRef,
    rows: Box<dyn Rows>,
    symbol: Option<StringBuilder>,
    pretty_px: bool,
    batch_size: usize,
}

/// Builder for the columnar encoders. See [`ArrowEncoderBuilder`](crate::encode::arrow::EncoderBuilder)
/// and [`ParquetEncoderBuilder`](crate::encode::parquet::EncoderBuilder).
pub struct EncoderBuilder<B: ColumnarWriter> {
    writer: B::Inner,
    batch_size: NonZeroUsize,
    pretty_px: bool,
    with_symbol: bool,
    compression: Compression,
    metadata: HashMap<String, String>,
    schema: Option<Schema>,
    ts_out: bool,
    version: u8,
    _format: PhantomData<B>,
}

impl<B: ColumnarWriter> EncoderBuilder<B> {
    /// Creates a new builder.
    pub fn new(writer: B::Inner) -> Self {
        Self {
            writer,
            batch_size: DEFAULT_BATCH_SIZE,
            pretty_px: false,
            with_symbol: false,
            compression: Compression::None,
            metadata: HashMap::new(),
            schema: None,
            ts_out: false,
            version: DBN_VERSION,
            _format: PhantomData,
        }
    }

    /// Sets the number of records per Arrow record batch or Parquet row group.
    /// Defaults to 65,536.
    pub fn batch_size(mut self, size: NonZeroUsize) -> Self {
        self.batch_size = size;
        self
    }

    /// Sets whether price fields are written as `Float64` (scaled). Defaults to
    /// `false` (`Int64` with fixed-point metadata).
    pub fn pretty_px(mut self, pretty_px: bool) -> Self {
        self.pretty_px = pretty_px;
        self
    }

    /// Sets whether the schema gets a trailing nullable `symbol` column,
    /// populated by [`encode_record_with_sym`](EncodeRecordTextExt::encode_record_with_sym).
    ///
    /// Defaults to `false`. When `true`, plain
    /// [`encode_record`](EncodeRecord::encode_record) calls return an error;
    /// when `false`, `encode_record_with_sym` returns an error.
    pub fn with_symbol(mut self, with_symbol: bool) -> Self {
        self.with_symbol = with_symbol;
        self
    }

    /// Sets the compression codec. Defaults to no compression.
    ///
    /// `Compression::Zstd` maps to Zstd level 3 for Parquet and to ZSTD body
    /// compression for Arrow IPC.
    pub fn compression(mut self, compression: Compression) -> Self {
        self.compression = compression;
        self
    }

    /// Sets the schema, `ts_out`, and version from a DBN [`Metadata`] and embeds the
    /// metadata as JSON under [`METADATA_KEY`] in the output file's schema. Equivalent
    /// to calling [`schema()`](Self::schema), [`ts_out()`](Self::ts_out), and
    /// [`version()`](Self::version) with the respective fields.
    pub fn metadata(mut self, metadata: &Metadata) -> Self {
        let mut json = String::new();
        to_json_in_buf(&mut json, metadata, false, false, false);
        // Drop the trailing newline that separates JSON lines
        json.pop();
        self.metadata = HashMap::from([(METADATA_KEY.to_owned(), json)]);
        self.schema = metadata.schema;
        self.ts_out = metadata.ts_out;
        self.version = metadata.version;
        self
    }

    /// Sets the DBN schema, which determines the record type and columns. Required.
    pub fn schema(mut self, schema: Schema) -> Self {
        self.schema = Some(schema);
        self
    }

    /// Sets whether records have an appended `ts_out`, adding a `ts_out` column.
    /// Defaults to `false`.
    pub fn ts_out(mut self, ts_out: bool) -> Self {
        self.ts_out = ts_out;
        self
    }

    /// Sets the DBN version, which determines the columns for schemas whose record
    /// changed between versions. Defaults to the current version.
    pub fn version(mut self, version: u8) -> Self {
        self.version = version;
        self
    }

    /// Builds the encoder and writes the file header.
    ///
    /// # Errors
    /// This function returns an error if no schema was set or if it fails to write
    /// to the underlying writer.
    pub fn build(self) -> Result<Encoder<B>> {
        let Some(schema) = self.schema else {
            return Err(Error::encode(
                "can't encode columnar format with mixed schemas; a single schema is required",
            ));
        };
        let batch_size = self.batch_size.get();
        let (mut fields, rows) = schema_dispatch!(
            schema,
            version: self.version,
            ts_out: self.ts_out,
            rows_for(self.pretty_px, batch_size)
        );
        let symbol = self.with_symbol.then(|| {
            fields.push(Field::new("symbol", DataType::Utf8, true));
            StringBuilder::with_capacity(batch_size, 0)
        });
        let schema = Arc::new(ArrowSchema::new_with_metadata(fields, self.metadata));
        let writer = B::open(self.writer, schema.clone(), self.compression)?;
        Ok(Encoder {
            writer: Some(writer),
            schema,
            rows,
            symbol,
            pretty_px: self.pretty_px,
            batch_size,
        })
    }
}

impl<B: ColumnarWriter> Encoder<B> {
    /// Creates a builder for configuring an `Encoder` object.
    pub fn builder(writer: B::Inner) -> EncoderBuilder<B> {
        EncoderBuilder::new(writer)
    }

    /// Returns `true` if the encoder has been closed.
    pub fn is_closed(&self) -> bool {
        self.writer.is_none()
    }

    /// Flushes any buffered records and finalizes the file. Calling this a second
    /// time has no effect.
    ///
    /// # Errors
    /// Returns an error if flushing or closing the writer fails.
    pub fn close(&mut self) -> Result<()> {
        let Some(mut writer) = self.writer.take() else {
            return Ok(());
        };
        if self.rows.len() > 0 {
            writer.write(&self.finish_batch()?)?;
        }
        writer.close()
    }

    fn writer(&mut self) -> Result<&mut B> {
        self.writer
            .as_mut()
            .ok_or_else(|| Error::encode("columnar encoder has already been closed"))
    }

    fn append(
        &mut self,
        record: &dyn Any,
        type_name: &str,
        symbol: Option<Option<&str>>,
    ) -> Result<()> {
        self.writer()?;
        self.rows.push(record, type_name)?;
        if let Some(symbol) = symbol {
            self.symbol
                .as_mut()
                .expect("symbol column checked by caller")
                .append_option(symbol);
        }
        if self.rows.len() >= self.batch_size {
            let batch = self.finish_batch()?;
            self.writer()?.write(&batch)?;
        }
        Ok(())
    }

    fn finish_batch(&mut self) -> Result<RecordBatch> {
        let mut columns = Vec::with_capacity(self.schema.fields().len());
        self.rows.take_columns(self.pretty_px, &mut columns);
        if let Some(symbol) = &mut self.symbol {
            columns.push(Arc::new(symbol.finish()));
        }
        RecordBatch::try_new(self.schema.clone(), columns)
            .map_err(|e| Error::arrow(e, "finishing Arrow batch"))
    }

    fn encode_with_ts_out<R: DbnEncodable + v3::HasRType + crate::RecordMut + Clone>(
        &mut self,
        rec: &R,
        ts_out: u64,
    ) -> Result<()> {
        self.encode_record(&crate::WithTsOut::new(rec.clone(), ts_out))
    }
}

impl<B: ColumnarWriter> EncodeRecord for Encoder<B> {
    fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> Result<()> {
        if self.symbol.is_some() {
            return Err(Error::encode(
                "Encoder configured with with_symbol(true); use encode_record_with_sym",
            ));
        }
        self.append(record, std::any::type_name::<R>(), None)
    }

    fn flush(&mut self) -> Result<()> {
        if self.rows.len() > 0 {
            let batch = self.finish_batch()?;
            self.writer()?.write(&batch)?;
        }
        self.writer()?.flush()
    }
}

impl<B: ColumnarWriter> EncodeRecordRef for Encoder<B> {
    fn encode_record_ref(&mut self, record: RecordRef) -> Result<()> {
        if self.symbol.is_some() {
            return Err(Error::encode(
                "Encoder configured with with_symbol(true); use encode_ref_with_sym",
            ));
        }
        rtype_dispatch!(record, self.encode_record())?
    }

    unsafe fn encode_record_ref_ts_out(&mut self, record: RecordRef, ts_out: bool) -> Result<()> {
        rtype_dispatch!(record, ts_out: ts_out, self.encode_record())?
    }

    fn encode_record_ref_with_ts_out(&mut self, record: RecordRef, ts_out: u64) -> Result<()> {
        rtype_dispatch!(record, self.encode_with_ts_out(ts_out))?
    }
}

impl<B: ColumnarWriter> EncodeDbn for Encoder<B> {}

impl<B: ColumnarWriter> EncodeRecordTextExt for Encoder<B> {
    fn encode_record_with_sym<R: DbnEncodable>(
        &mut self,
        record: &R,
        symbol: Option<&str>,
    ) -> Result<()> {
        if self.symbol.is_none() {
            return Err(Error::encode(
                "Encoder not configured with with_symbol(true); use encode_record",
            ));
        }
        self.append(record, std::any::type_name::<R>(), Some(symbol))
    }
}

impl<B: ColumnarWriter> Drop for Encoder<B> {
    fn drop(&mut self) {
        // Best effort: call `close()` to surface errors
        let _ = self.close();
    }
}

/// Default number of records buffered before a batch or row group is flushed.
pub(crate) const DEFAULT_BATCH_SIZE: NonZeroUsize = NonZeroUsize::new(65_536).unwrap();

/// A columnar file format that consumes Arrow [`RecordBatch`]es. Implemented by
/// [`IpcFile`](super::IpcFile) and [`ParquetFile`](crate::encode::parquet::ParquetFile).
pub trait ColumnarWriter: Sized {
    /// The underlying byte sink.
    type Inner: Write + Send;

    /// Opens a new file writing to `writer`. Any key-value metadata on `schema` is
    /// embedded in the file.
    ///
    /// # Errors
    /// This function returns an error if `compression` isn't supported by the format
    /// or if it fails to write the file header to `writer`.
    fn open(writer: Self::Inner, schema: SchemaRef, compression: Compression) -> Result<Self>;
    /// Writes a batch of records.
    ///
    /// # Errors
    /// This function returns an error if it fails to write `batch` to the underlying
    /// writer.
    fn write(&mut self, batch: &RecordBatch) -> Result<()>;
    /// Flushes any buffered output to the underlying writer without finalizing
    /// the file.
    ///
    /// # Errors
    /// This function returns an error if it fails to flush the underlying writer.
    fn flush(&mut self) -> Result<()>;
    /// Finalizes the file (writes footer, magic bytes, etc.) and consumes the
    /// writer.
    ///
    /// # Errors
    /// This function returns an error if it fails to write the file footer to the
    /// underlying writer.
    fn close(self) -> Result<()>;
}

/// The records buffered between batches, typed on the record type fixed at build time
/// so a batch's columns are built in one typed pass.
trait Rows: Send {
    fn push(&mut self, record: &dyn Any, type_name: &str) -> Result<()>;
    fn len(&self) -> usize;
    /// Builds the columns from the buffered records and clears the buffer.
    fn take_columns(&mut self, pretty_px: bool, out: &mut Vec<ArrayRef>);
}

struct TypedRows<R>(Vec<R>);

impl<R> Rows for TypedRows<R>
where
    R: ArrowSerialize + v3::HasRType + Clone + Send + 'static,
{
    fn push(&mut self, record: &dyn Any, type_name: &str) -> Result<()> {
        let Some(record) = record.downcast_ref::<R>() else {
            return Err(Error::encode(format!(
                "expected {} records, but found {type_name}. Mixed schemas cannot be encoded \
                 in Parquet or Arrow",
                std::any::type_name::<R>(),
            )));
        };
        self.0.push(record.clone());
        Ok(())
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn take_columns(&mut self, pretty_px: bool, out: &mut Vec<ArrayRef>) {
        R::arrow_columns(&self.0, |r| r, pretty_px, out);
        self.0.clear();
    }
}

fn rows_for<R>(pretty_px: bool, capacity: usize) -> (Vec<Field>, Box<dyn Rows>)
where
    R: ArrowSerialize + v3::HasRType + Clone + Send + 'static,
{
    let mut fields = Vec::new();
    R::arrow_fields(pretty_px, &mut fields);
    (
        fields,
        Box::new(TypedRows::<R>(Vec::with_capacity(capacity))),
    )
}
