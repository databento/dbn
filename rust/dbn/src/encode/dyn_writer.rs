use std::io;

use super::{zstd_encoder, zstd_encoder_with_clevel};
use crate::{Compression, Result};

/// Type for runtime polymorphism over whether encoding uncompressed or Zstd-compressed
/// DBN records. Implements [`std::io::Write`].
pub struct DynWriter<'a, W>(DynWriterImpl<'a, W>)
where
    W: io::Write;

enum DynWriterImpl<'a, W>
where
    W: io::Write,
{
    Uncompressed(W),
    Zstd(zstd::stream::AutoFinishEncoder<'a, W>),
}

impl<W> DynWriter<'_, W>
where
    W: io::Write,
{
    /// Creates a new instance of [`DynWriter`] which will wrap `writer` with `compression`.
    ///
    /// # Errors
    /// This function returns an error if it fails to initialize the Zstd encoder.
    pub fn new(writer: W, compression: Compression) -> Result<Self> {
        match compression {
            Compression::None => Ok(Self(DynWriterImpl::Uncompressed(writer))),
            Compression::Zstd => zstd_encoder(writer).map(|enc| Self(DynWriterImpl::Zstd(enc))),
        }
    }

    /// Creates a new instance with zstd compression of the specified level.
    ///
    /// # Errors
    /// This function returns an error if it fails to initialize the Zstd encoder.
    pub fn with_compression_level(writer: W, level: i32) -> Result<Self> {
        Ok(Self(DynWriterImpl::Zstd(zstd_encoder_with_clevel(
            writer, level,
        )?)))
    }

    /// Returns a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        match &mut self.0 {
            DynWriterImpl::Uncompressed(w) => w,
            DynWriterImpl::Zstd(enc) => enc.get_mut(),
        }
    }
}

impl<W> io::Write for DynWriter<'_, W>
where
    W: io::Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut self.0 {
            DynWriterImpl::Uncompressed(writer) => writer.write(buf),
            DynWriterImpl::Zstd(writer) => writer.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.0 {
            DynWriterImpl::Uncompressed(writer) => writer.flush(),
            DynWriterImpl::Zstd(writer) => writer.flush(),
        }
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        match &mut self.0 {
            DynWriterImpl::Uncompressed(writer) => writer.write_vectored(bufs),
            DynWriterImpl::Zstd(writer) => writer.write_vectored(bufs),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match &mut self.0 {
            DynWriterImpl::Uncompressed(writer) => writer.write_all(buf),
            DynWriterImpl::Zstd(writer) => writer.write_all(buf),
        }
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()> {
        match &mut self.0 {
            DynWriterImpl::Uncompressed(writer) => writer.write_fmt(fmt),
            DynWriterImpl::Zstd(writer) => writer.write_fmt(fmt),
        }
    }
}

#[cfg(feature = "async")]
pub use r#async::DynBufWriter as DynAsyncBufWriter;
#[cfg(feature = "async")]
pub use r#async::DynWriter as DynAsyncWriter;

#[cfg(feature = "async")]
mod r#async {
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };

    use async_compression::tokio::write::ZstdEncoder;
    use tokio::io::{self, BufWriter};

    use crate::{
        encode::{async_zstd_encoder, async_zstd_encoder_with_clevel},
        enums::Compression,
    };

    /// An object that allows for abstracting over compressed and uncompressed output
    /// with buffering.
    pub struct DynBufWriter<W, B = W>(DynBufWriterImpl<W, B>);

    enum DynBufWriterImpl<W, B> {
        Uncompressed(B),
        Zstd(ZstdEncoder<W>),
    }

    impl<W> DynBufWriter<W, BufWriter<W>>
    where
        W: io::AsyncWriteExt + Unpin,
    {
        /// Creates a new instance which will wrap `writer` in a `BufWriter` and
        /// `compression`.
        pub fn new(writer: W, compression: Compression) -> Self {
            Self(match compression {
                Compression::None => DynBufWriterImpl::Uncompressed(BufWriter::new(writer)),
                // async zstd always wraps the writer in a BufWriter
                Compression::Zstd => DynBufWriterImpl::Zstd(async_zstd_encoder(writer)),
            })
        }

        /// Creates a new instance, wrapping `writer` in a `BufWriter` and compressing
        /// the output according to `level`.
        pub fn with_compression_level(writer: W, level: i32) -> Self {
            Self(DynBufWriterImpl::Zstd(async_zstd_encoder_with_clevel(
                writer, level,
            )))
        }
    }

    impl<W> io::AsyncWrite for DynBufWriter<W>
    where
        W: io::AsyncWrite + io::AsyncWriteExt + Unpin,
    {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            match &mut self.0 {
                DynBufWriterImpl::Uncompressed(w) => {
                    io::AsyncWrite::poll_write(Pin::new(w), cx, buf)
                }
                DynBufWriterImpl::Zstd(enc) => io::AsyncWrite::poll_write(Pin::new(enc), cx, buf),
            }
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            match &mut self.0 {
                DynBufWriterImpl::Uncompressed(w) => io::AsyncWrite::poll_flush(Pin::new(w), cx),
                DynBufWriterImpl::Zstd(enc) => io::AsyncWrite::poll_flush(Pin::new(enc), cx),
            }
        }

        fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            match &mut self.0 {
                DynBufWriterImpl::Uncompressed(w) => io::AsyncWrite::poll_shutdown(Pin::new(w), cx),
                DynBufWriterImpl::Zstd(enc) => io::AsyncWrite::poll_shutdown(Pin::new(enc), cx),
            }
        }

        fn poll_write_vectored(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            bufs: &[std::io::IoSlice<'_>],
        ) -> Poll<io::Result<usize>> {
            match &mut self.0 {
                DynBufWriterImpl::Uncompressed(w) => {
                    io::AsyncWrite::poll_write_vectored(Pin::new(w), cx, bufs)
                }
                DynBufWriterImpl::Zstd(enc) => {
                    io::AsyncWrite::poll_write_vectored(Pin::new(enc), cx, bufs)
                }
            }
        }

        fn is_write_vectored(&self) -> bool {
            match &self.0 {
                DynBufWriterImpl::Uncompressed(w) => w.is_write_vectored(),
                DynBufWriterImpl::Zstd(enc) => enc.is_write_vectored(),
            }
        }
    }

    /// An object that allows for abstracting over compressed and uncompressed output.
    ///
    /// Compared with [`DynBufWriter`], only the compressed output is buffered, as it is
    /// required by the async Zstd implementation.
    pub struct DynWriter<W>(DynWriterImpl<W>)
    where
        W: io::AsyncWriteExt + Unpin;

    enum DynWriterImpl<W>
    where
        W: io::AsyncWriteExt + Unpin,
    {
        Uncompressed(W),
        Zstd(ZstdEncoder<W>),
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
                Compression::Zstd => DynWriterImpl::Zstd(async_zstd_encoder(writer)),
            })
        }

        /// Creates a new instance, compressing the output according to `level`.
        pub fn with_compression_level(writer: W, level: i32) -> Self {
            Self(DynWriterImpl::Zstd(async_zstd_encoder_with_clevel(
                writer, level,
            )))
        }

        /// Returns a mutable reference to the underlying writer.
        pub fn get_mut(&mut self) -> &mut W {
            match &mut self.0 {
                DynWriterImpl::Uncompressed(w) => w,
                DynWriterImpl::Zstd(enc) => enc.get_mut(),
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
                DynWriterImpl::Zstd(enc) => io::AsyncWrite::poll_write(Pin::new(enc), cx, buf),
            }
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            match &mut self.0 {
                DynWriterImpl::Uncompressed(w) => io::AsyncWrite::poll_flush(Pin::new(w), cx),
                DynWriterImpl::Zstd(enc) => io::AsyncWrite::poll_flush(Pin::new(enc), cx),
            }
        }

        fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            match &mut self.0 {
                DynWriterImpl::Uncompressed(w) => io::AsyncWrite::poll_shutdown(Pin::new(w), cx),
                DynWriterImpl::Zstd(enc) => io::AsyncWrite::poll_shutdown(Pin::new(enc), cx),
            }
        }

        fn poll_write_vectored(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            bufs: &[std::io::IoSlice<'_>],
        ) -> Poll<io::Result<usize>> {
            match &mut self.0 {
                DynWriterImpl::Uncompressed(w) => {
                    io::AsyncWrite::poll_write_vectored(Pin::new(w), cx, bufs)
                }
                DynWriterImpl::Zstd(enc) => {
                    io::AsyncWrite::poll_write_vectored(Pin::new(enc), cx, bufs)
                }
            }
        }

        fn is_write_vectored(&self) -> bool {
            match &self.0 {
                DynWriterImpl::Uncompressed(w) => w.is_write_vectored(),
                DynWriterImpl::Zstd(enc) => enc.is_write_vectored(),
            }
        }
    }
}
