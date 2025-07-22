use std::{
    fs::File,
    io::{self, BufReader, ErrorKind, Read},
    path::Path,
};

use crate::Compression;

use super::{zstd, SkipBytes};

/// Type for runtime polymorphism over whether decoding uncompressed or Zstd-compressed
/// DBN records. Implements [`std::io::Write`].
pub struct DynReader<'a, R>(DynReaderImpl<'a, R>)
where
    R: io::BufRead;

enum DynReaderImpl<'a, R>
where
    R: io::BufRead,
{
    Uncompressed(R),
    Zstd(::zstd::stream::Decoder<'a, R>),
}

impl<R> DynReader<'_, BufReader<R>>
where
    R: io::Read,
{
    /// Creates a new [`DynReader`] from a reader, with the specified `compression`.
    /// If `reader` also implements [`BufRead`](io::BufRead), it's better to use
    /// [`with_buffer()`](Self::with_buffer).
    ///
    /// # Errors
    /// This function will return an error if it fails to create the zstd decoder.
    pub fn new(reader: R, compression: Compression) -> crate::Result<Self> {
        Self::with_buffer(BufReader::new(reader), compression)
    }

    /// Creates a new [`DynReader`] from a reader, inferring the compression.
    /// If `reader` also implements [`io::BufRead`], it is better to use
    /// [`inferred_with_buffer()`](Self::inferred_with_buffer).
    ///
    /// # Errors
    /// This function will return an error if it is unable to read from `reader`
    /// or it fails to create the zstd decoder.
    pub fn new_inferred(reader: R) -> crate::Result<Self> {
        Self::inferred_with_buffer(BufReader::new(reader))
    }
}

impl<R> DynReader<'_, R>
where
    R: io::BufRead,
{
    /// Creates a new [`DynReader`] from a buffered reader with the specified
    /// `compression`.
    ///
    /// # Errors
    /// This function will return an error if it fails to create the zstd decoder.
    pub fn with_buffer(reader: R, compression: Compression) -> crate::Result<Self> {
        match compression {
            Compression::None => Ok(Self(DynReaderImpl::Uncompressed(reader))),
            Compression::Zstd => Ok(Self(DynReaderImpl::Zstd(
                ::zstd::stream::Decoder::with_buffer(reader)
                    .map_err(|e| crate::Error::io(e, "creating zstd decoder"))?,
            ))),
        }
    }

    /// Creates a new [`DynReader`] from a buffered reader, inferring the compression.
    ///
    /// # Errors
    /// This function will return an error if it fails to read from `reader` or creating
    /// the zstd decoder fails.
    pub fn inferred_with_buffer(mut reader: R) -> crate::Result<Self> {
        let first_bytes = reader
            .fill_buf()
            .map_err(|e| crate::Error::io(e, "creating buffer to infer encoding"))?;
        if zstd::starts_with_prefix(first_bytes) {
            Ok(Self(DynReaderImpl::Zstd(
                ::zstd::stream::Decoder::with_buffer(reader)
                    .map_err(|e| crate::Error::io(e, "creating zstd decoder"))?,
            )))
        } else {
            Ok(Self(DynReaderImpl::Uncompressed(reader)))
        }
    }

    /// Returns a mutable reference to the inner reader.
    pub fn get_mut(&mut self) -> &mut R {
        match &mut self.0 {
            DynReaderImpl::Uncompressed(reader) => reader,
            DynReaderImpl::Zstd(reader) => reader.get_mut(),
        }
    }

    /// Returns a reference to the inner reader.
    pub fn get_ref(&self) -> &R {
        match &self.0 {
            DynReaderImpl::Uncompressed(reader) => reader,
            DynReaderImpl::Zstd(reader) => reader.get_ref(),
        }
    }
}

impl DynReader<'_, BufReader<File>> {
    /// Creates a new [`DynReader`] from the file at `path`.
    ///
    /// # Errors
    /// This function will return an error if the file doesn't exist, it is unable to
    /// determine the encoding of the file, or it fails to create the zstd decoder.
    pub fn from_file(path: impl AsRef<Path>) -> crate::Result<Self> {
        let file = File::open(path.as_ref()).map_err(|e| {
            crate::Error::io(
                e,
                format!(
                    "opening file to decode at path '{}'",
                    path.as_ref().display()
                ),
            )
        })?;
        DynReader::new_inferred(file)
    }
}

impl<R> io::Read for DynReader<'_, R>
where
    R: io::BufRead,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.0 {
            DynReaderImpl::Uncompressed(r) => r.read(buf),
            DynReaderImpl::Zstd(r) => r.read(buf),
        }
    }
}

impl<R> SkipBytes for DynReader<'_, R>
where
    R: io::BufRead + io::Read + io::Seek,
{
    fn skip_bytes(&mut self, n_bytes: usize) -> crate::Result<()> {
        let handle_err = |err| crate::Error::io(err, format!("seeking ahead {n_bytes} bytes"));
        match &mut self.0 {
            DynReaderImpl::Uncompressed(reader) => {
                reader.seek_relative(n_bytes as i64).map_err(handle_err)
            }
            DynReaderImpl::Zstd(reader) => {
                let mut buf = [0; 1024];
                let mut remaining = n_bytes;
                while remaining > 0 {
                    let max_read_size = remaining.min(buf.len());
                    let read_size = reader.read(&mut buf[..max_read_size]).map_err(handle_err)?;
                    if read_size == 0 {
                        return Err(crate::Error::io(
                            std::io::Error::from(ErrorKind::UnexpectedEof),
                            format!(
                                "seeking ahead {n_bytes} bytes. Only able to seek {} bytes",
                                n_bytes - remaining
                            ),
                        ));
                    }
                    remaining -= read_size;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::decode::tests::TEST_DATA_PATH;

    #[test]
    fn test_dyn_reader() {
        let mut uncompressed =
            DynReader::from_file(format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn")).unwrap();
        let mut compressed =
            DynReader::from_file(format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst")).unwrap();
        let mut uncompressed_res = Vec::new();
        uncompressed.read_to_end(&mut uncompressed_res).unwrap();
        let mut compressed_res = Vec::new();
        compressed.read_to_end(&mut compressed_res).unwrap();
        assert_eq!(compressed_res, uncompressed_res);
    }
}

#[cfg(feature = "async")]
pub use self::r#async::DynReader as AsyncDynReader;

#[cfg(feature = "async")]
mod r#async {
    use std::{io::ErrorKind, path::Path, pin::Pin};

    use async_compression::tokio::bufread::ZstdDecoder;
    use tokio::{
        fs::File,
        io::{self, AsyncReadExt, AsyncSeekExt, BufReader},
    };

    use crate::{
        decode::{AsyncSkipBytes, ZSTD_FILE_BUFFER_CAPACITY},
        enums::Compression,
    };

    use super::zstd::zstd_decoder;

    impl<T> AsyncSkipBytes for T
    where
        T: AsyncSeekExt + Unpin,
    {
        async fn skip_bytes(&mut self, n_bytes: usize) -> crate::Result<()> {
            self.seek(std::io::SeekFrom::Current(n_bytes as i64))
                .await
                .map(drop)
                .map_err(|err| crate::Error::io(err, format!("seeking ahead {n_bytes} bytes")))
        }
    }

    /// A type for runtime polymorphism on compressed and uncompressed input.
    /// The async version of [`DynReader`](super::DynReader).
    pub struct DynReader<R>(DynReaderImpl<R>)
    where
        R: io::AsyncBufReadExt + Unpin;

    enum DynReaderImpl<R>
    where
        R: io::AsyncBufReadExt + Unpin,
    {
        Uncompressed(R),
        Zstd(ZstdDecoder<R>),
    }

    impl<R> DynReader<BufReader<R>>
    where
        R: io::AsyncReadExt + Unpin,
    {
        /// Creates a new instance of [`DynReader`] with the specified `compression`. If
        /// `reader` also implements [`AsyncBufRead`](tokio::io::AsyncBufRead), it's
        /// better to use [`with_buffer()`](Self::with_buffer).
        pub fn new(reader: R, compression: Compression) -> Self {
            Self::with_buffer(BufReader::new(reader), compression)
        }

        /// Creates a new [`DynReader`] from a reader, inferring the compression.
        /// If `reader` also implements [`AsyncBufRead`](tokio::io::AsyncBufRead), it is
        /// better to use [`inferred_with_buffer()`](Self::inferred_with_buffer).
        ///
        /// # Errors
        /// This function will return an error if it is unable to read from `reader`.
        pub async fn new_inferred(reader: R) -> crate::Result<Self> {
            Self::inferred_with_buffer(BufReader::new(reader)).await
        }
    }

    impl<R> DynReader<R>
    where
        R: io::AsyncBufReadExt + Unpin,
    {
        /// Creates a new [`DynReader`] from a buffered reader with the specified
        /// `compression`.
        pub fn with_buffer(reader: R, compression: Compression) -> Self {
            match compression {
                Compression::None => Self(DynReaderImpl::Uncompressed(reader)),
                Compression::Zstd => Self(DynReaderImpl::Zstd(ZstdDecoder::new(reader))),
            }
        }

        /// Creates a new [`DynReader`] from a buffered reader, inferring the compression.
        ///
        /// # Errors
        /// This function will return an error if it fails to read from `reader`.
        pub async fn inferred_with_buffer(mut reader: R) -> crate::Result<Self> {
            let first_bytes = reader
                .fill_buf()
                .await
                .map_err(|e| crate::Error::io(e, "creating buffer to infer encoding"))?;
            Ok(if super::zstd::starts_with_prefix(first_bytes) {
                Self(DynReaderImpl::Zstd(zstd_decoder(reader)))
            } else {
                Self(DynReaderImpl::Uncompressed(reader))
            })
        }

        /// Returns a mutable reference to the inner reader.
        pub fn get_mut(&mut self) -> &mut R {
            match &mut self.0 {
                DynReaderImpl::Uncompressed(reader) => reader,
                DynReaderImpl::Zstd(reader) => reader.get_mut(),
            }
        }

        /// Returns a reference to the inner reader.
        pub fn get_ref(&self) -> &R {
            match &self.0 {
                DynReaderImpl::Uncompressed(reader) => reader,
                DynReaderImpl::Zstd(reader) => reader.get_ref(),
            }
        }
    }

    impl DynReader<BufReader<File>> {
        /// Creates a new [`DynReader`] from the file at `path`.
        ///
        /// # Errors
        /// This function will return an error if the file doesn't exist, it is unable
        /// to read from it.
        pub async fn from_file(path: impl AsRef<Path>) -> crate::Result<Self> {
            let file = File::open(path.as_ref()).await.map_err(|e| {
                crate::Error::io(
                    e,
                    format!(
                        "opening file to decode at path '{}'",
                        path.as_ref().display()
                    ),
                )
            })?;
            DynReader::inferred_with_buffer(BufReader::with_capacity(
                ZSTD_FILE_BUFFER_CAPACITY,
                file,
            ))
            .await
        }
    }

    impl<R> io::AsyncRead for DynReader<R>
    where
        R: io::AsyncRead + io::AsyncReadExt + io::AsyncBufReadExt + Unpin,
    {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match &mut self.0 {
                DynReaderImpl::Uncompressed(reader) => {
                    io::AsyncRead::poll_read(Pin::new(reader), cx, buf)
                }
                DynReaderImpl::Zstd(reader) => io::AsyncRead::poll_read(Pin::new(reader), cx, buf),
            }
        }
    }

    impl<R> AsyncSkipBytes for DynReader<R>
    where
        R: io::AsyncSeekExt + io::AsyncBufReadExt + Unpin,
    {
        /// Like AsyncSeek, but only allows seeking forward from the current position.
        ///
        /// # Cancellation safety
        /// This method is not cancel safe.
        async fn skip_bytes(&mut self, n_bytes: usize) -> crate::Result<()> {
            let handle_err = |err| crate::Error::io(err, format!("seeking ahead {n_bytes} bytes"));
            match &mut self.0 {
                DynReaderImpl::Uncompressed(reader) => reader
                    .seek(std::io::SeekFrom::Current(n_bytes as i64))
                    .await
                    .map(drop)
                    .map_err(handle_err),
                DynReaderImpl::Zstd(reader) => {
                    let mut buf = [0; 1024];
                    let mut remaining = n_bytes;
                    while remaining > 0 {
                        let max_read_size = remaining.min(buf.len());
                        let read_size = reader
                            .read(&mut buf[..max_read_size])
                            .await
                            .map_err(handle_err)?;
                        if read_size == 0 {
                            return Err(crate::Error::io(
                                std::io::Error::from(ErrorKind::UnexpectedEof),
                                format!(
                                    "seeking ahead {n_bytes} bytes. Only able to seek {} bytes",
                                    n_bytes - remaining
                                ),
                            ));
                        }
                        remaining -= read_size;
                    }
                    Ok(())
                }
            }
        }
    }
    #[cfg(test)]
    mod tests {
        use crate::{
            compat::InstrumentDefMsgV1,
            decode::{tests::TEST_DATA_PATH, AsyncDbnRecordDecoder},
            VersionUpgradePolicy,
        };

        use super::*;

        #[tokio::test]
        async fn test_decode_multiframe_zst() {
            let mut decoder = AsyncDbnRecordDecoder::with_version(
                DynReader::from_file(&format!(
                    "{TEST_DATA_PATH}/multi-frame.definition.v1.dbn.frag.zst"
                ))
                .await
                .unwrap(),
                1,
                VersionUpgradePolicy::AsIs,
                false,
            )
            .unwrap();
            let mut count = 0;
            while let Some(_rec) = decoder.decode::<InstrumentDefMsgV1>().await.unwrap() {
                count += 1;
            }
            assert_eq!(count, 8);
        }
    }
}
