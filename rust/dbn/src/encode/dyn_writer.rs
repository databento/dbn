use std::io;

use super::zstd_encoder;
use crate::{Compression, Result};

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

    use crate::{encode::async_zstd_encoder, enums::Compression};

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
                Compression::ZStd => DynWriterImpl::ZStd(async_zstd_encoder(writer)),
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
