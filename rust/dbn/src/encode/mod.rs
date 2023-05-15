//! Encoding DBN and Zstd-compressed DBN files and streams. Encoders implement the
//! [`EncodeDbn`] trait.
pub mod csv;
pub mod dbn;
pub mod json;

use std::{fmt, io, num::NonZeroU64};

use streaming_iterator::StreamingIterator;
use time::format_description::FormatItem;

use crate::{
    decode::DecodeDbn,
    enums::{Compression, Encoding},
    record::HasRType,
    record_ref::RecordRef,
    rtype_ts_out_dispatch, Metadata, FIXED_PRICE_SCALE,
};

use self::{csv::serialize::CsvSerialize, json::serialize::JsonSerialize};

/// Trait alias for [`HasRType`], `AsRef<[u8]>`, `CsvSerialize`, [`fmt::Debug`], and `JsonSerialize`.
pub trait DbnEncodable: HasRType + AsRef<[u8]> + CsvSerialize + fmt::Debug + JsonSerialize {}
impl<T> DbnEncodable for T where
    T: HasRType + AsRef<[u8]> + CsvSerialize + fmt::Debug + JsonSerialize
{
}

/// Trait for types that encode DBN records with mixed schemas.
pub trait EncodeDbn {
    /// Encodes a single DBN record of type `R`.
    ///
    /// Returns `true`if the pipe was closed.
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying writer
    /// or there's a serialization error.
    fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> anyhow::Result<bool>;

    /// Encodes a slice of DBN records.
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying writer
    /// or there's a serialization error.
    fn encode_records<R: DbnEncodable>(&mut self, records: &[R]) -> anyhow::Result<()> {
        for record in records {
            if self.encode_record(record)? {
                break;
            }
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
    ) -> anyhow::Result<()> {
        while let Some(record) = stream.next() {
            if self.encode_record(record)? {
                break;
            }
        }
        self.flush()?;
        Ok(())
    }

    /// Flushes any buffered content to the true output.
    ///
    /// # Errors
    /// This function returns an error if it's unable to flush the underlying writer.
    fn flush(&mut self) -> anyhow::Result<()>;

    /// Encodes a single DBN record.
    ///
    /// Returns `true`if the pipe was closed.
    ///
    /// # Safety
    /// `ts_out` must be `false` if `record` does not have an appended `ts_out
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying writer
    /// or there's a serialization error.
    unsafe fn encode_record_ref(
        &mut self,
        record: RecordRef,
        ts_out: bool,
    ) -> anyhow::Result<bool> {
        rtype_ts_out_dispatch!(record, ts_out, |rec| self.encode_record(rec))?
    }

    /// Encodes DBN records directly from a DBN decoder.
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying writer
    /// or there's a serialization error.
    fn encode_decoded<D: DecodeDbn>(&mut self, mut decoder: D) -> anyhow::Result<()> {
        let ts_out = decoder.metadata().ts_out;
        while let Some(record) = decoder.decode_record_ref()? {
            // Safety: It's safe to cast to `WithTsOut` because we're passing in the `ts_out`
            // from the metadata header.
            if unsafe { self.encode_record_ref(record, ts_out)? } {
                break;
            }
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
    ) -> anyhow::Result<()> {
        let ts_out = decoder.metadata().ts_out;
        let mut i = 0;
        while let Some(record) = decoder.decode_record_ref()? {
            // Safety: It's safe to cast to `WithTsOut` because we're passing in the `ts_out`
            // from the metadata header.
            if unsafe { self.encode_record_ref(record, ts_out)? } {
                break;
            }
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
pub enum DynWriter<'a, W>
where
    W: io::Write,
{
    /// For writing uncompressed records.
    Uncompressed(W),
    /// For writing Zstandard-compressed records.
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
    pub fn new(writer: W, compression: Compression) -> anyhow::Result<Self> {
        match compression {
            Compression::None => Ok(Self::Uncompressed(writer)),
            Compression::ZStd => zstd_encoder(writer).map(Self::ZStd),
        }
    }

    /// Returns a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        match self {
            DynWriter::Uncompressed(w) => w,
            DynWriter::ZStd(enc) => enc.get_mut(),
        }
    }
}

fn zstd_encoder<'a, W: io::Write>(
    writer: W,
) -> anyhow::Result<zstd::stream::AutoFinishEncoder<'a, W>> {
    let mut zstd_encoder = zstd::Encoder::new(writer, ZSTD_COMPRESSION_LEVEL)?;
    zstd_encoder.include_checksum(true)?;
    Ok(zstd_encoder.auto_finish())
}

impl<'a, W> io::Write for DynWriter<'a, W>
where
    W: io::Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            DynWriter::Uncompressed(writer) => writer.write(buf),
            DynWriter::ZStd(writer) => writer.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            DynWriter::Uncompressed(writer) => writer.flush(),
            DynWriter::ZStd(writer) => writer.flush(),
        }
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        match self {
            DynWriter::Uncompressed(writer) => writer.write_vectored(bufs),
            DynWriter::ZStd(writer) => writer.write_vectored(bufs),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match self {
            DynWriter::Uncompressed(writer) => writer.write_all(buf),
            DynWriter::ZStd(writer) => writer.write_all(buf),
        }
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()> {
        match self {
            DynWriter::Uncompressed(writer) => writer.write_fmt(fmt),
            DynWriter::ZStd(writer) => writer.write_fmt(fmt),
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
    ) -> anyhow::Result<Self> {
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

impl<'a, W> EncodeDbn for DynEncoder<'a, W>
where
    W: io::Write,
{
    fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> anyhow::Result<bool> {
        self.0.encode_record(record)
    }

    fn encode_records<R: DbnEncodable>(&mut self, records: &[R]) -> anyhow::Result<()> {
        self.0.encode_records(records)
    }

    fn encode_stream<R: DbnEncodable>(
        &mut self,
        stream: impl StreamingIterator<Item = R>,
    ) -> anyhow::Result<()> {
        self.0.encode_stream(stream)
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        self.0.flush()
    }

    unsafe fn encode_record_ref(
        &mut self,
        record: RecordRef,
        ts_out: bool,
    ) -> anyhow::Result<bool> {
        self.0.encode_record_ref(record, ts_out)
    }

    fn encode_decoded<D: DecodeDbn>(&mut self, decoder: D) -> anyhow::Result<()> {
        self.0.encode_decoded(decoder)
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
        fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> anyhow::Result<bool> {
            match self {
                $(Self::$variant(v) => v.encode_record(record),)*
            }
        }

        fn encode_records<R: DbnEncodable>(&mut self, records: &[R]) -> anyhow::Result<()> {
            match self {
                $(Self::$variant(v) => v.encode_records(records),)*
            }
        }

        fn encode_stream<R: DbnEncodable>(
            &mut self,
            stream: impl StreamingIterator<Item = R>,
        ) -> anyhow::Result<()> {
            match self {
                $(Self::$variant(v) => v.encode_stream(stream),)*
            }
        }

        fn flush(&mut self) -> anyhow::Result<()> {
            match self {
                $(Self::$variant(v) => v.flush(),)*
            }
        }

        unsafe fn encode_record_ref(
            &mut self,
            record: RecordRef,
            ts_out: bool,
        ) -> anyhow::Result<bool> {
            match self {
                $(Self::$variant(v) => v.encode_record_ref(record, ts_out),)*
            }
        }

        fn encode_decoded<D: DecodeDbn>(
            &mut self,
            decoder: D,
        ) -> anyhow::Result<()> {
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

fn format_px(px: i64) -> String {
    if px == crate::UNDEF_PRICE {
        "UNDEF_PRICE".to_owned()
    } else {
        let (sign, px_abs) = if px < 0 { ("-", -px) } else { ("", px) };
        let px_integer = px_abs / FIXED_PRICE_SCALE;
        let px_fraction = px_abs % FIXED_PRICE_SCALE;
        format!("{sign}{px_integer}.{px_fraction:09}")
    }
}

fn format_ts(ts: u64) -> String {
    const TS_FORMAT: &[FormatItem<'static>] = time::macros::format_description!(
        "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:9]"
    );
    if ts == 0 {
        String::new()
    } else {
        time::OffsetDateTime::from_unix_timestamp_nanos(ts as i128)
            .map_err(|_| ())
            .and_then(|dt| dt.format(TS_FORMAT).map_err(|_| ()))
            .unwrap_or_else(|_| ts.to_string())
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

#[cfg(test)]
mod tests {
    use crate::UNDEF_PRICE;

    use super::*;

    #[test]
    fn test_format_px_negative() {
        assert_eq!(format_px(-100_000), "-0.000100000");
    }

    #[test]
    fn test_format_px_positive() {
        assert_eq!(format_px(32_500_000_000), "32.500000000");
    }

    #[test]
    fn test_format_px_zero() {
        assert_eq!(format_px(0), "0.000000000");
    }

    #[test]
    fn test_format_px_undef() {
        assert_eq!(format_px(UNDEF_PRICE), "UNDEF_PRICE");
    }

    #[test]
    fn format_ts_0() {
        assert!(format_ts(0).is_empty());
    }

    #[test]
    fn format_ts_1() {
        assert_eq!(format_ts(1), "1970-01-01T00:00:00.000000001");
    }

    #[test]
    fn format_ts_future() {
        assert_eq!(
            format_ts(1622838300000000000),
            "2021-06-04T20:25:00.000000000"
        );
    }
}
