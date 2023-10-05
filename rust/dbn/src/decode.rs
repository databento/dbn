//! Decoding DBN and Zstd-compressed DBN files and streams. Decoders implement the
//! [`DecodeDbn`] trait.
pub mod dbn;
// Having any tests in a deprecated module emits many warnings that can't be silenced, see
// https://github.com/rust-lang/rust/issues/47238
#[cfg_attr(
    not(test),
    deprecated(
        since = "0.3.0",
        note = "DBZ was renamed to DBN and the format was changed to no longer rely on Zstd."
    )
)]
pub mod dbz;
mod stream;
// used in databento_dbn
#[doc(hidden)]
pub mod zstd;

// Re-exports
pub use self::dbn::{
    Decoder as DbnDecoder, MetadataDecoder as DbnMetadataDecoder, RecordDecoder as DbnRecordDecoder,
};
pub use stream::StreamIterDecoder;

use std::{
    fs::File,
    io::{self, BufReader},
    mem,
    path::Path,
};

use crate::{
    enums::Compression,
    record::HasRType,
    record_ref::RecordRef,
    // record_ref::RecordRef,
    Metadata,
};

/// Trait for types that decode references to DBN records of a dynamic type.
pub trait DecodeRecordRef {
    /// Tries to decode a generic reference a record.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an error of a
    /// kind other than `io::ErrorKind::UnexpectedEof` upon reading.
    ///
    /// If the `length` property of the record is invalid, an
    /// [`Error::Decode`](crate::Error::Decode) will be returned.
    fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef>>;
}

/// Trait for types that decode DBN records of a particular type.
pub trait DecodeDbn: DecodeRecordRef + private::BufferSlice {
    /// Returns a reference to the decoded [`Metadata`].
    fn metadata(&self) -> &Metadata;

    /// Tries to decode a reference to a single record of type `T`. Returns `Ok(None)`
    /// if the input has been exhausted.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an error of a
    /// kind other than `io::ErrorKind::UnexpectedEof` upon reading.
    ///
    /// If the next record is of a different type than `T`, an
    /// [`Error::Conversion`](crate::Error::Conversion) will be returned.
    ///
    /// If the `length` property of the record is invalid, an
    /// [`Error::Decode`](crate::Error::Decode) will be returned.
    fn decode_record<T: HasRType>(&mut self) -> crate::Result<Option<&T>>;

    /// Converts the decoder into a streaming iterator of records of type `T`. This
    /// lazily decodes the data.
    fn decode_stream<T: HasRType>(self) -> StreamIterDecoder<Self, T>
    where
        Self: Sized;

    /// Tries to decode all records into a `Vec`. This eagerly decodes the data.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an error of a
    /// kind other than `io::ErrorKind::UnexpectedEof` upon reading.
    ///
    /// If any of the records is of a different type than `T`, an
    /// [`Error::Conversion`](crate::Error::Conversion) will be returned.
    ///
    /// If the `length` property of any of the records is invalid, a
    /// [`Error::Decode`](crate::Error::Decode) will be returned.
    fn decode_records<T: HasRType + Clone>(mut self) -> crate::Result<Vec<T>>
    where
        Self: Sized,
    {
        let mut res = Vec::new();
        while let Some(rec) = self.decode_record::<T>()? {
            res.push(rec.clone());
        }
        Ok(res)
    }
}

/// A decoder implementing [`DecodeDbn`] whose [`Encoding`](crate::enums::Encoding) and
/// [`Compression`] are determined at runtime by peeking at the first few bytes.
pub struct DynDecoder<'a, R>(DynDecoderImpl<'a, R>)
where
    R: io::BufRead;

enum DynDecoderImpl<'a, R>
where
    R: io::BufRead,
{
    Dbn(dbn::Decoder<R>),
    ZstdDbn(dbn::Decoder<::zstd::stream::Decoder<'a, R>>),
    #[allow(deprecated)]
    LegacyDbz(dbz::Decoder<R>),
}

impl<'a, R> DynDecoder<'a, BufReader<R>>
where
    R: io::Read,
{
    /// Creates a new [`DynDecoder`] from a reader, with the specified `compression`.
    ///
    /// # Errors
    /// This function will return an error if it fails to parse the metadata.
    pub fn new(reader: R, compression: Compression) -> crate::Result<Self> {
        Self::with_buffer(BufReader::new(reader), compression)
    }

    /// Creates a new [`DynDecoder`] from a reader, inferring the encoding and
    /// compression. If `reader` also implements [`io::BufRead`], it is better to use
    /// [`inferred_with_buffer()`](Self::inferred_with_buffer).
    ///
    /// # Errors
    /// This function will return an error if it is unable to determine
    /// the encoding of `reader` or it fails to parse the metadata.
    pub fn new_inferred(reader: R) -> crate::Result<Self> {
        Self::inferred_with_buffer(BufReader::new(reader))
    }
}

impl<'a, R> DynDecoder<'a, R>
where
    R: io::BufRead,
{
    /// Creates a new [`DynDecoder`] from a buffered reader with the specified
    /// `compression`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to determine
    /// the encoding of `reader` or it fails to parse the metadata.
    pub fn with_buffer(reader: R, compression: Compression) -> crate::Result<Self> {
        match compression {
            Compression::None => Ok(Self(DynDecoderImpl::Dbn(dbn::Decoder::new(reader)?))),
            Compression::ZStd => Ok(Self(DynDecoderImpl::ZstdDbn(
                dbn::Decoder::with_zstd_buffer(reader)?,
            ))),
        }
    }

    /// Creates a new [`DynDecoder`] from a buffered reader, inferring the encoding
    /// and compression.
    ///
    /// # Errors
    /// This function will return an error if it is unable to determine
    /// the encoding of `reader` or it fails to parse the metadata.
    pub fn inferred_with_buffer(mut reader: R) -> crate::Result<Self> {
        let first_bytes = reader
            .fill_buf()
            .map_err(|e| crate::Error::io(e, "creating buffer to infer encoding"))?;
        #[allow(deprecated)]
        if dbz::starts_with_prefix(first_bytes) {
            Ok(Self(DynDecoderImpl::LegacyDbz(dbz::Decoder::new(reader)?)))
        } else if dbn::starts_with_prefix(first_bytes) {
            Ok(Self(DynDecoderImpl::Dbn(dbn::Decoder::new(reader)?)))
        } else if zstd::starts_with_prefix(first_bytes) {
            Ok(Self(DynDecoderImpl::ZstdDbn(
                dbn::Decoder::with_zstd_buffer(reader)?,
            )))
        } else {
            Err(crate::Error::decode("Unable to determine encoding"))
        }
    }
}

impl<'a> DynDecoder<'a, BufReader<File>> {
    /// Creates a new [`DynDecoder`] from the file at `path`.
    ///
    /// # Errors
    /// This function will return an error if the file doesn't exist, it is unable to
    /// determine the encoding of the file or it fails to parse the metadata.
    pub fn from_file(path: impl AsRef<Path>) -> crate::Result<Self> {
        let file = File::open(path.as_ref()).map_err(|e| {
            crate::Error::io(
                e,
                format!(
                    "Error opening file to decode at path '{}'",
                    path.as_ref().display()
                ),
            )
        })?;
        DynDecoder::new_inferred(file)
    }
}

impl<'a, R> DecodeRecordRef for DynDecoder<'a, R>
where
    R: io::BufRead,
{
    fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef>> {
        match &mut self.0 {
            DynDecoderImpl::Dbn(decoder) => decoder.decode_record_ref(),
            DynDecoderImpl::ZstdDbn(decoder) => decoder.decode_record_ref(),
            DynDecoderImpl::LegacyDbz(decoder) => decoder.decode_record_ref(),
        }
    }
}

#[allow(deprecated)]
impl<'a, R> DecodeDbn for DynDecoder<'a, R>
where
    R: io::BufRead,
{
    fn metadata(&self) -> &Metadata {
        match &self.0 {
            DynDecoderImpl::Dbn(decoder) => decoder.metadata(),
            DynDecoderImpl::ZstdDbn(decoder) => decoder.metadata(),
            DynDecoderImpl::LegacyDbz(decoder) => decoder.metadata(),
        }
    }

    fn decode_record<T: HasRType>(&mut self) -> crate::Result<Option<&T>> {
        match &mut self.0 {
            DynDecoderImpl::Dbn(decoder) => decoder.decode_record(),
            DynDecoderImpl::ZstdDbn(decoder) => decoder.decode_record(),
            DynDecoderImpl::LegacyDbz(decoder) => decoder.decode_record(),
        }
    }

    fn decode_stream<T: HasRType>(self) -> StreamIterDecoder<Self, T>
    where
        Self: Sized,
    {
        StreamIterDecoder::new(self)
    }
}

/// Type for runtime polymorphism over whether decoding uncompressed or ZStd-compressed
/// DBN records. Implements [`std::io::Write`].
pub struct DynReader<'a, R>(DynReaderImpl<'a, R>)
where
    R: io::BufRead;

enum DynReaderImpl<'a, R>
where
    R: io::BufRead,
{
    Uncompressed(R),
    ZStd(::zstd::stream::Decoder<'a, R>),
}

impl<'a, R> DynReader<'a, BufReader<R>>
where
    R: io::Read,
{
    /// Creates a new [`DynReader`] from a reader, with the specified `compression`.
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

impl<'a, R> DynReader<'a, R>
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
            Compression::ZStd => Ok(Self(DynReaderImpl::ZStd(
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
            Ok(Self(DynReaderImpl::ZStd(
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
            DynReaderImpl::ZStd(reader) => reader.get_mut(),
        }
    }

    /// Returns a reference to the inner reader.
    pub fn get_ref(&self) -> &R {
        match &self.0 {
            DynReaderImpl::Uncompressed(reader) => reader,
            DynReaderImpl::ZStd(reader) => reader.get_ref(),
        }
    }
}

impl<'a> DynReader<'a, BufReader<File>> {
    /// Creates a new [`DynReader`] from the file at `path`.
    ///
    /// # Errors
    /// This function will return an error if the file doesn't exist, it is unable to
    /// determine the encoding of the file or it fails to parse the metadata.
    pub fn from_file(path: impl AsRef<Path>) -> crate::Result<Self> {
        let file = File::open(path.as_ref()).map_err(|e| {
            crate::Error::io(
                e,
                format!(
                    "Error opening file to decode at path '{}'",
                    path.as_ref().display()
                ),
            )
        })?;
        DynReader::new_inferred(file)
    }
}

impl<'a, R> io::Read for DynReader<'a, R>
where
    R: io::BufRead,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.0 {
            DynReaderImpl::Uncompressed(r) => r.read(buf),
            DynReaderImpl::ZStd(r) => r.read(buf),
        }
    }
}

impl<'a, R> private::BufferSlice for DynDecoder<'a, R>
where
    R: io::BufRead,
{
    fn buffer_slice(&self) -> &[u8] {
        match &self.0 {
            DynDecoderImpl::Dbn(decoder) => decoder.buffer_slice(),
            DynDecoderImpl::ZstdDbn(decoder) => decoder.buffer_slice(),
            DynDecoderImpl::LegacyDbz(decoder) => decoder.buffer_slice(),
        }
    }
}

mod private {
    /// An implementation detail for the interaction between [`StreamingIterator`] and
    /// implementors of [`DecodeDbn`].
    #[doc(hidden)]
    pub trait BufferSlice {
        fn buffer_slice(&self) -> &[u8];
    }
}

pub(crate) trait FromLittleEndianSlice {
    fn from_le_slice(slice: &[u8]) -> Self;
}

impl FromLittleEndianSlice for u64 {
    /// NOTE: assumes the length of `slice` is at least 8 bytes
    fn from_le_slice(slice: &[u8]) -> Self {
        let (bytes, _) = slice.split_at(mem::size_of::<Self>());
        Self::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl FromLittleEndianSlice for i32 {
    /// NOTE: assumes the length of `slice` is at least 4 bytes
    fn from_le_slice(slice: &[u8]) -> Self {
        let (bytes, _) = slice.split_at(mem::size_of::<Self>());
        Self::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl FromLittleEndianSlice for u32 {
    /// NOTE: assumes the length of `slice` is at least 4 bytes
    fn from_le_slice(slice: &[u8]) -> Self {
        let (bytes, _) = slice.split_at(mem::size_of::<Self>());
        Self::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl FromLittleEndianSlice for u16 {
    /// NOTE: assumes the length of `slice` is at least 2 bytes
    fn from_le_slice(slice: &[u8]) -> Self {
        let (bytes, _) = slice.split_at(mem::size_of::<Self>());
        Self::from_le_bytes(bytes.try_into().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    pub const TEST_DATA_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/data");

    #[test]
    fn test_dyn_reader() {
        for file in std::fs::read_dir(TEST_DATA_PATH).unwrap() {
            let file = file.unwrap();
            if matches!(file.path().extension(), Some(ext) if ext == "dbn") {
                let path = file.path();
                let mut uncompressed = DynReader::from_file(&path).unwrap();
                let mut compressed_path = path.clone().into_os_string();
                compressed_path.push(".zst");
                let mut compressed = DynReader::from_file(&compressed_path).unwrap();
                let mut uncompressed_res = Vec::new();
                uncompressed.read_to_end(&mut uncompressed_res).unwrap();
                let mut compressed_res = Vec::new();
                compressed.read_to_end(&mut compressed_res).unwrap();
                assert_eq!(
                    compressed_res,
                    uncompressed_res,
                    "failed at {}",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn test_detects_any_dbn_version_as_dbn() {
        let mut buf = Vec::new();
        let mut file = File::open(format!("{TEST_DATA_PATH}/test_data.mbo.dbn")).unwrap();
        file.read_to_end(&mut buf).unwrap();
        // change version
        buf[3] = crate::DBN_VERSION + 1;
        let res = DynDecoder::new_inferred(io::Cursor::new(buf));
        assert!(matches!(res, Err(e) if e
            .to_string()
            .contains("Can't decode newer version of DBN")));
    }
}

#[cfg(feature = "async")]
pub use self::{dbn::AsyncDecoder as AsyncDbnDecoder, r#async::DynReader as AsyncDynReader};

#[cfg(feature = "async")]
mod r#async {
    use std::pin::Pin;

    use async_compression::tokio::bufread::ZstdDecoder;
    use tokio::io::{self, BufReader};

    use crate::enums::Compression;

    /// A type for runtime polymorphism on compressed and uncompressed input.
    pub struct DynReader<R>(DynReaderImpl<R>)
    where
        R: io::AsyncReadExt + Unpin;

    enum DynReaderImpl<R>
    where
        R: io::AsyncReadExt + Unpin,
    {
        Uncompressed(R),
        ZStd(ZstdDecoder<BufReader<R>>),
    }

    impl<R> DynReader<R>
    where
        R: io::AsyncReadExt + Unpin,
    {
        /// Creates a new instance of [`DynReader`] with the specified `compression`.
        pub fn new(reader: R, compression: Compression) -> Self {
            Self(match compression {
                Compression::None => DynReaderImpl::Uncompressed(reader),
                Compression::ZStd => DynReaderImpl::ZStd(ZstdDecoder::new(BufReader::new(reader))),
            })
        }
    }

    impl<R> io::AsyncRead for DynReader<R>
    where
        R: io::AsyncRead + io::AsyncReadExt + Unpin,
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
                DynReaderImpl::ZStd(dec) => io::AsyncRead::poll_read(Pin::new(dec), cx, buf),
            }
        }
    }
}
