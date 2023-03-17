//! Encoding DBN and Zstd-compressed DBN files and streams. Encoders implement the
//! [`EncodeDbn`] trait.
pub mod csv;
pub mod dbn;
pub mod json;

use std::{fmt, io};

use anyhow::anyhow;
use serde::Serialize;
use streaming_iterator::StreamingIterator;

use crate::{
    decode::DecodeDbn,
    enums::{Compression, Encoding, Schema},
    record::{
        HasRType, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, TbboMsg, TradeMsg,
        WithTsOut,
    },
    Metadata,
};

use self::csv::serialize::CsvSerialize;

/// Trait alias for [`HasRType`], `csv::serialize::CsvSerialize`, [`fmt::Debug`], and [`serde::Serialize`].
pub trait DbnEncodable: HasRType + AsRef<[u8]> + CsvSerialize + fmt::Debug + Serialize {}
impl<T> DbnEncodable for T where T: HasRType + AsRef<[u8]> + CsvSerialize + fmt::Debug + Serialize {}

/// Trait for types that encode DBN records.
pub trait EncodeDbn {
    /// Encode a single DBN record of type `R`.
    ///
    /// Returns `true`if the pipe was closed.
    fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> anyhow::Result<bool>;
    /// Encode a slice of DBN records.
    fn encode_records<R: DbnEncodable>(&mut self, records: &[R]) -> anyhow::Result<()>;
    /// Encode a stream of DBN records.
    fn encode_stream<R: DbnEncodable>(
        &mut self,
        stream: impl StreamingIterator<Item = R>,
    ) -> anyhow::Result<()>;

    /// Encode DBN records directly from a DBN decoder.
    fn encode_decoded<D: DecodeDbn>(&mut self, decoder: D) -> anyhow::Result<()> {
        match (decoder.metadata().schema, decoder.metadata().ts_out) {
            (Schema::Mbo, true) => {
                self.encode_stream(decoder.decode_stream::<WithTsOut<MboMsg>>()?)
            }
            (Schema::Mbo, false) => self.encode_stream(decoder.decode_stream::<MboMsg>()?),
            (Schema::Mbp1, true) => {
                self.encode_stream(decoder.decode_stream::<WithTsOut<Mbp1Msg>>()?)
            }
            (Schema::Mbp1, false) => self.encode_stream(decoder.decode_stream::<Mbp1Msg>()?),
            (Schema::Mbp10, true) => {
                self.encode_stream(decoder.decode_stream::<WithTsOut<Mbp10Msg>>()?)
            }
            (Schema::Mbp10, false) => self.encode_stream(decoder.decode_stream::<Mbp10Msg>()?),
            (Schema::Tbbo, true) => {
                self.encode_stream(decoder.decode_stream::<WithTsOut<TbboMsg>>()?)
            }
            (Schema::Tbbo, false) => self.encode_stream(decoder.decode_stream::<TbboMsg>()?),
            (Schema::Trades, true) => {
                self.encode_stream(decoder.decode_stream::<WithTsOut<TradeMsg>>()?)
            }
            (Schema::Trades, false) => self.encode_stream(decoder.decode_stream::<TradeMsg>()?),
            (Schema::Ohlcv1S | Schema::Ohlcv1M | Schema::Ohlcv1H | Schema::Ohlcv1D, true) => {
                self.encode_stream(decoder.decode_stream::<WithTsOut<OhlcvMsg>>()?)
            }
            (Schema::Ohlcv1S | Schema::Ohlcv1M | Schema::Ohlcv1H | Schema::Ohlcv1D, false) => {
                self.encode_stream(decoder.decode_stream::<OhlcvMsg>()?)
            }
            (Schema::Definition, true) => {
                self.encode_stream(decoder.decode_stream::<WithTsOut<InstrumentDefMsg>>()?)
            }
            (Schema::Definition, false) => {
                self.encode_stream(decoder.decode_stream::<InstrumentDefMsg>()?)
            }
            (Schema::Statistics | Schema::Status, _) => Err(anyhow!("Not implemented")),
        }
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
    Uncompressed(W),
    ZStd(zstd::stream::AutoFinishEncoder<'a, W>),
}

impl<'a, W> DynWriter<'a, W>
where
    W: io::Write,
{
    /// Create a new instance of [`DynWriter`] which will wrap `writer` with `compression`.
    pub fn new(writer: W, compression: Compression) -> anyhow::Result<Self> {
        match compression {
            Compression::None => Ok(Self::Uncompressed(writer)),
            Compression::ZStd => zstd_encoder(writer).map(Self::ZStd),
        }
    }

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
    /// Note: `should_pretty_print` is ignored unless `encoding` is [`Encoding::Json`].
    pub fn new(
        writer: W,
        encoding: Encoding,
        compression: Compression,
        metadata: &Metadata,
        should_pretty_print: bool,
    ) -> anyhow::Result<Self> {
        let writer = DynWriter::new(writer, compression)?;
        match encoding {
            Encoding::Dbn => {
                dbn::Encoder::new(writer, metadata).map(|e| Self(DynEncoderImpl::Dbn(e)))
            }
            Encoding::Csv => Ok(Self(DynEncoderImpl::Csv(csv::Encoder::new(writer)))),
            Encoding::Json => Ok(Self(DynEncoderImpl::Json(json::Encoder::new(
                writer,
                should_pretty_print,
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
        product_id: 323,
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
