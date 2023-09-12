//! Encoding DBN and Zstd-compressed DBN files and streams. Encoders implement the
//! [`EncodeDbn`] trait.
pub mod csv;
pub mod dbn;
pub mod json;

use std::{fmt, io, num::NonZeroU64};

use streaming_iterator::StreamingIterator;

// Re-exports
#[cfg(feature = "async")]
pub use self::dbn::{
    AsyncMetadataEncoder as AsyncDbnMetadataEncoder, AsyncRecordEncoder as AsyncDbnRecordEncoder,
};
#[cfg(feature = "async")]
pub use self::json::AsyncEncoder as AsyncJsonEncoder;
pub use self::{
    csv::Encoder as CsvEncoder,
    dbn::{
        Encoder as DbnEncoder, MetadataEncoder as DbnMetadataEncoder,
        RecordEncoder as DbnRecordEncoder,
    },
    json::Encoder as JsonEncoder,
};

use crate::Error;
use crate::{
    decode::DecodeDbn,
    enums::{Compression, Encoding},
    record::HasRType,
    record_ref::RecordRef,
    Metadata, Result,
};

use self::{csv::serialize::CsvSerialize, json::serialize::JsonSerialize};

/// Trait alias for [`HasRType`], `AsRef<[u8]>`, `CsvSerialize`, [`fmt::Debug`], and `JsonSerialize`.
pub trait DbnEncodable: HasRType + AsRef<[u8]> + CsvSerialize + fmt::Debug + JsonSerialize {}
impl<T> DbnEncodable for T where
    T: HasRType + AsRef<[u8]> + CsvSerialize + fmt::Debug + JsonSerialize
{
}

/// Trait for types that encode a DBN record of a specific type.
pub trait EncodeRecord {
    /// Encodes a single DBN record of type `R`.
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying writer
    /// or there's a serialization error.
    fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> Result<()>;

    /// Flushes any buffered content to the true output.
    ///
    /// # Errors
    /// This function returns an error if it's unable to flush the underlying writer.
    fn flush(&mut self) -> Result<()>;
}

/// Trait for types that encode DBN records with mixed schemas.
pub trait EncodeRecordRef {
    /// Encodes a single DBN [`RecordRef`].
    ///
    /// # Safety
    /// `ts_out` must be `false` if `record` does not have an appended `ts_out`.
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying writer
    /// or there's a serialization error.
    unsafe fn encode_record_ref(&mut self, record: RecordRef, ts_out: bool) -> Result<()>;
}

/// Trait for types that encode DBN records with a specific record type.
pub trait EncodeDbn: EncodeRecord + EncodeRecordRef {
    /// Encodes a slice of DBN records.
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying writer
    /// or there's a serialization error.
    fn encode_records<R: DbnEncodable>(&mut self, records: &[R]) -> Result<()> {
        for record in records {
            self.encode_record(record)?;
        }
        self.flush()?;
        Ok(())
    }

    /// Encodes a stream of DBN records.
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying writer
    /// or there's a serialization error.
    fn encode_stream<R: DbnEncodable>(
        &mut self,
        mut stream: impl StreamingIterator<Item = R>,
    ) -> Result<()> {
        while let Some(record) = stream.next() {
            self.encode_record(record)?;
        }
        self.flush()?;
        Ok(())
    }

    /// Encodes DBN records directly from a DBN decoder.
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying writer
    /// or there's a serialization error.
    fn encode_decoded<D: DecodeDbn>(&mut self, mut decoder: D) -> Result<()> {
        let ts_out = decoder.metadata().ts_out;
        while let Some(record) = decoder.decode_record_ref()? {
            // Safety: It's safe to cast to `WithTsOut` because we're passing in the `ts_out`
            // from the metadata header.
            unsafe { self.encode_record_ref(record, ts_out) }?;
        }
        self.flush()?;
        Ok(())
    }

    /// Encodes DBN records directly from a DBN decoder, outputting no more than
    /// `limit` records.
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying writer
    /// or there's a serialization error.
    fn encode_decoded_with_limit<D: DecodeDbn>(
        &mut self,
        mut decoder: D,
        limit: NonZeroU64,
    ) -> Result<()> {
        let ts_out = decoder.metadata().ts_out;
        let mut i = 0;
        while let Some(record) = decoder.decode_record_ref()? {
            // Safety: It's safe to cast to `WithTsOut` because we're passing in the `ts_out`
            // from the metadata header.
            unsafe { self.encode_record_ref(record, ts_out) }?;
            i += 1;
            if i == limit.get() {
                break;
            }
        }
        self.flush()?;
        Ok(())
    }
}

/// The default Zstandard compression level.
const ZSTD_COMPRESSION_LEVEL: i32 = 0;

/// Type for runtime polymorphism over whether encoding uncompressed or ZStd-compressed
/// DBN records. Implements [`std::io::Write`].
pub struct DynWriter<'a, W>(DynWriterImpl<'a, W>)
where
    W: io::Write;

enum DynWriterImpl<'a, W>
where
    W: io::Write,
{
    Uncompressed(W),
    ZStd(zstd::stream::AutoFinishEncoder<'a, W>),
}

impl<'a, W> DynWriter<'a, W>
where
    W: io::Write,
{
    /// Create a new instance of [`DynWriter`] which will wrap `writer` with `compression`.
    ///
    /// # Errors
    /// This function returns an error if it fails to initialize the Zstd compression.
    pub fn new(writer: W, compression: Compression) -> Result<Self> {
        match compression {
            Compression::None => Ok(Self(DynWriterImpl::Uncompressed(writer))),
            Compression::ZStd => zstd_encoder(writer).map(|enc| Self(DynWriterImpl::ZStd(enc))),
        }
    }

    /// Returns a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        match &mut self.0 {
            DynWriterImpl::Uncompressed(w) => w,
            DynWriterImpl::ZStd(enc) => enc.get_mut(),
        }
    }
}

fn zstd_encoder<'a, W: io::Write>(writer: W) -> Result<zstd::stream::AutoFinishEncoder<'a, W>> {
    let mut zstd_encoder = zstd::Encoder::new(writer, ZSTD_COMPRESSION_LEVEL)
        .map_err(|e| Error::io(e, "creating zstd encoder"))?;
    zstd_encoder
        .include_checksum(true)
        .map_err(|e| Error::io(e, "setting zstd checksum"))?;
    Ok(zstd_encoder.auto_finish())
}

impl<'a, W> io::Write for DynWriter<'a, W>
where
    W: io::Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut self.0 {
            DynWriterImpl::Uncompressed(writer) => writer.write(buf),
            DynWriterImpl::ZStd(writer) => writer.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.0 {
            DynWriterImpl::Uncompressed(writer) => writer.flush(),
            DynWriterImpl::ZStd(writer) => writer.flush(),
        }
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        match &mut self.0 {
            DynWriterImpl::Uncompressed(writer) => writer.write_vectored(bufs),
            DynWriterImpl::ZStd(writer) => writer.write_vectored(bufs),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match &mut self.0 {
            DynWriterImpl::Uncompressed(writer) => writer.write_all(buf),
            DynWriterImpl::ZStd(writer) => writer.write_all(buf),
        }
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()> {
        match &mut self.0 {
            DynWriterImpl::Uncompressed(writer) => writer.write_fmt(fmt),
            DynWriterImpl::ZStd(writer) => writer.write_fmt(fmt),
        }
    }
}

/// An encoder implementing [`EncodeDbn`] whose [`Encoding`] and [`Compression`] can be
/// set at runtime.
pub struct DynEncoder<'a, W>(DynEncoderImpl<'a, W>)
where
    W: io::Write;

// [`DynEncoder`] isn't cloned so this isn't a concern.
#[allow(clippy::large_enum_variant)]
enum DynEncoderImpl<'a, W>
where
    W: io::Write,
{
    Dbn(dbn::Encoder<DynWriter<'a, W>>),
    Csv(csv::Encoder<DynWriter<'a, W>>),
    Json(json::Encoder<DynWriter<'a, W>>),
}

impl<'a, W> DynEncoder<'a, W>
where
    W: io::Write,
{
    /// Constructs a new instance of [`DynEncoder`].
    ///
    /// Note: `should_pretty_print`, `user_pretty_px`, and `use_pretty_ts` are ignored
    /// if `encoding` is `Dbn`.
    ///
    /// # Errors
    /// This function returns an error if it fails to encode the DBN metadata or
    /// it fails to initialize the Zstd compression.
    pub fn new(
        writer: W,
        encoding: Encoding,
        compression: Compression,
        metadata: &Metadata,
        should_pretty_print: bool,
        use_pretty_px: bool,
        use_pretty_ts: bool,
    ) -> Result<Self> {
        let writer = DynWriter::new(writer, compression)?;
        match encoding {
            Encoding::Dbn => {
                dbn::Encoder::new(writer, metadata).map(|e| Self(DynEncoderImpl::Dbn(e)))
            }
            Encoding::Csv => Ok(Self(DynEncoderImpl::Csv(csv::Encoder::new(
                writer,
                use_pretty_px,
                use_pretty_ts,
            )))),
            Encoding::Json => Ok(Self(DynEncoderImpl::Json(json::Encoder::new(
                writer,
                should_pretty_print,
                use_pretty_px,
                use_pretty_ts,
            )))),
        }
    }
}

impl<'a, W> EncodeRecord for DynEncoder<'a, W>
where
    W: io::Write,
{
    fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> Result<()> {
        self.0.encode_record(record)
    }

    fn flush(&mut self) -> Result<()> {
        self.0.flush()
    }
}

impl<'a, W> EncodeRecordRef for DynEncoder<'a, W>
where
    W: io::Write,
{
    unsafe fn encode_record_ref(&mut self, record: RecordRef, ts_out: bool) -> Result<()> {
        self.0.encode_record_ref(record, ts_out)
    }
}

impl<'a, W> EncodeDbn for DynEncoder<'a, W>
where
    W: io::Write,
{
    fn encode_records<R: DbnEncodable>(&mut self, records: &[R]) -> Result<()> {
        self.0.encode_records(records)
    }

    fn encode_stream<R: DbnEncodable>(
        &mut self,
        stream: impl StreamingIterator<Item = R>,
    ) -> Result<()> {
        self.0.encode_stream(stream)
    }

    fn encode_decoded<D: DecodeDbn>(&mut self, decoder: D) -> Result<()> {
        self.0.encode_decoded(decoder)
    }
}

impl<'a, W> EncodeRecord for DynEncoderImpl<'a, W>
where
    W: io::Write,
{
    fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> Result<()> {
        match self {
            DynEncoderImpl::Dbn(enc) => enc.encode_record(record),
            DynEncoderImpl::Csv(enc) => enc.encode_record(record),
            DynEncoderImpl::Json(enc) => enc.encode_record(record),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            DynEncoderImpl::Dbn(enc) => enc.flush(),
            DynEncoderImpl::Csv(enc) => enc.flush(),
            DynEncoderImpl::Json(enc) => enc.flush(),
        }
    }
}

impl<'a, W> EncodeRecordRef for DynEncoderImpl<'a, W>
where
    W: io::Write,
{
    unsafe fn encode_record_ref(&mut self, record: RecordRef, ts_out: bool) -> Result<()> {
        match self {
            DynEncoderImpl::Dbn(enc) => enc.encode_record_ref(record, ts_out),
            DynEncoderImpl::Csv(enc) => enc.encode_record_ref(record, ts_out),
            DynEncoderImpl::Json(enc) => enc.encode_record_ref(record, ts_out),
        }
    }
}

impl<'a, W> EncodeDbn for DynEncoderImpl<'a, W>
where
    W: io::Write,
{
    encoder_enum_dispatch! {Dbn, Csv, Json}
}

/// An aid the with boilerplate code of calling the same method on each enum variant's
/// inner value.
macro_rules! encoder_enum_dispatch {
    ($($variant:ident),*) => {
        fn encode_records<R: DbnEncodable>(&mut self, records: &[R]) -> Result<()> {
            match self {
                $(Self::$variant(v) => v.encode_records(records),)*
            }
        }

        fn encode_stream<R: DbnEncodable>(
            &mut self,
            stream: impl StreamingIterator<Item = R>,
        ) -> Result<()> {
            match self {
                $(Self::$variant(v) => v.encode_stream(stream),)*
            }
        }

        fn encode_decoded<D: DecodeDbn>(
            &mut self,
            decoder: D,
        ) -> Result<()> {
            match self {
                $(Self::$variant(v) => v.encode_decoded(decoder),)*
            }
        }
    };
}

pub(crate) use encoder_enum_dispatch;

#[cfg(test)]
mod test_data {
    use streaming_iterator::StreamingIterator;

    use crate::record::{BidAskPair, RecordHeader};

    // Common data used in multiple tests
    pub const RECORD_HEADER: RecordHeader = RecordHeader {
        length: 30,
        rtype: 4,
        publisher_id: 1,
        instrument_id: 323,
        ts_event: 1658441851000000000,
    };

    pub const BID_ASK: BidAskPair = BidAskPair {
        bid_px: 372000000000000,
        ask_px: 372500000000000,
        bid_sz: 10,
        ask_sz: 5,
        bid_ct: 5,
        ask_ct: 2,
    };

    /// A testing shim to get a streaming iterator from a [`Vec`].
    pub struct VecStream<T> {
        vec: Vec<T>,
        idx: isize,
    }

    impl<T> VecStream<T> {
        pub fn new(vec: Vec<T>) -> Self {
            // initialize at -1 because `advance()` is always called before
            // `get()`.
            Self { vec, idx: -1 }
        }
    }

    impl<T> StreamingIterator for VecStream<T> {
        type Item = T;

        fn advance(&mut self) {
            self.idx += 1;
        }

        fn get(&self) -> Option<&Self::Item> {
            self.vec.get(self.idx as usize)
        }
    }
}

#[cfg(feature = "async")]
pub use r#async::DynWriter as DynAsyncWriter;

#[cfg(feature = "async")]
mod r#async {
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };

    use async_compression::tokio::write::ZstdEncoder;
    use tokio::io;

    use crate::enums::Compression;

    /// An object that allows for abstracting over compressed and uncompressed output.
    pub struct DynWriter<W>(DynWriterImpl<W>)
    where
        W: io::AsyncWriteExt + Unpin;

    enum DynWriterImpl<W>
    where
        W: io::AsyncWriteExt + Unpin,
    {
        Uncompressed(W),
        ZStd(ZstdEncoder<W>),
    }

    impl<W> DynWriter<W>
    where
        W: io::AsyncWriteExt + Unpin,
    {
        /// Creates a new instance of [`DynWriter`] which will wrap `writer` with
        /// `compression`.
        pub fn new(writer: W, compression: Compression) -> Self {
            Self(match compression {
                Compression::None => DynWriterImpl::Uncompressed(writer),
                Compression::ZStd => DynWriterImpl::ZStd(ZstdEncoder::new(writer)),
            })
        }

        /// Returns a mutable reference to the underlying writer.
        pub fn get_mut(&mut self) -> &mut W {
            match &mut self.0 {
                DynWriterImpl::Uncompressed(w) => w,
                DynWriterImpl::ZStd(enc) => enc.get_mut(),
            }
        }
    }

    impl<W> io::AsyncWrite for DynWriter<W>
    where
        W: io::AsyncWrite + io::AsyncWriteExt + Unpin,
    {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            match &mut self.0 {
                DynWriterImpl::Uncompressed(w) => io::AsyncWrite::poll_write(Pin::new(w), cx, buf),
                DynWriterImpl::ZStd(enc) => io::AsyncWrite::poll_write(Pin::new(enc), cx, buf),
            }
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            match &mut self.0 {
                DynWriterImpl::Uncompressed(w) => io::AsyncWrite::poll_flush(Pin::new(w), cx),
                DynWriterImpl::ZStd(enc) => io::AsyncWrite::poll_flush(Pin::new(enc), cx),
            }
        }

        fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            match &mut self.0 {
                DynWriterImpl::Uncompressed(w) => io::AsyncWrite::poll_shutdown(Pin::new(w), cx),
                DynWriterImpl::ZStd(enc) => io::AsyncWrite::poll_shutdown(Pin::new(enc), cx),
            }
        }
    }
}
