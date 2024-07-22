//! Decoding DBN and Zstd-compressed DBN files and streams. Sync decoders implement
//the ! [`DecodeDbn`] trait.
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
    enums::{Compression, VersionUpgradePolicy},
    record::HasRType,
    record_ref::RecordRef,
    Metadata,
};

/// Trait for types that decode references to DBN records of a dynamic type.
pub trait DecodeRecordRef {
    /// Tries to decode a generic reference a record. Returns `Ok(None)` if input
    /// has been exhausted.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an error of a
    /// kind other than `io::ErrorKind::UnexpectedEof` upon reading.
    ///
    /// If the `length` property of the record is invalid, an
    /// [`Error::Decode`](crate::Error::Decode) will be returned.
    fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef>>;
}

/// Trait for decoders with metadata about what's being decoded.
pub trait DbnMetadata {
    /// Returns an immutable reference to the decoded [`Metadata`].
    fn metadata(&self) -> &Metadata;

    /// Returns a mutable reference to the decoded [`Metadata`].
    fn metadata_mut(&mut self) -> &mut Metadata;
}

/// Trait for types that decode DBN records of a particular type.
pub trait DecodeRecord {
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

/// A trait alias for DBN decoders with metadata.
pub trait DecodeDbn: DecodeRecord + DecodeRecordRef + DbnMetadata {}

/// A trait for decoders that can be converted to streaming iterators.
pub trait DecodeStream: DecodeRecord + private::BufferSlice {
    /// Converts the decoder into a streaming iterator of records of type `T`. This
    /// lazily decodes the data.
    fn decode_stream<T: HasRType>(self) -> StreamIterDecoder<Self, T>
    where
        Self: Sized;
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
    /// Creates a new [`DynDecoder`] from a reader, with the specified `compression`. It
    /// will decode records from previous DBN versions according to `upgrade_policy`.
    ///
    /// # Errors
    /// This function will return an error if it fails to parse the metadata.
    pub fn new(
        reader: R,
        compression: Compression,
        upgrade_policy: VersionUpgradePolicy,
    ) -> crate::Result<Self> {
        Self::with_buffer(BufReader::new(reader), compression, upgrade_policy)
    }

    /// Creates a new [`DynDecoder`] from a reader, inferring the encoding and
    /// compression. If `reader` also implements [`io::BufRead`], it is better to use
    /// [`inferred_with_buffer()`](Self::inferred_with_buffer). It will decode records
    /// from previous DBN versions according to `upgrade_policy`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to determine
    /// the encoding of `reader` or it fails to parse the metadata.
    pub fn new_inferred(reader: R, upgrade_policy: VersionUpgradePolicy) -> crate::Result<Self> {
        Self::inferred_with_buffer(BufReader::new(reader), upgrade_policy)
    }
}

impl<'a, R> DynDecoder<'a, R>
where
    R: io::BufRead,
{
    /// Creates a new [`DynDecoder`] from a buffered reader with the specified
    /// `compression`.It will decode records from previous DBN versions according to
    /// `upgrade_policy`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to determine
    /// the encoding of `reader` or it fails to parse the metadata.
    pub fn with_buffer(
        reader: R,
        compression: Compression,
        upgrade_policy: VersionUpgradePolicy,
    ) -> crate::Result<Self> {
        match compression {
            Compression::None => Ok(Self(DynDecoderImpl::Dbn(
                dbn::Decoder::with_upgrade_policy(reader, upgrade_policy)?,
            ))),
            Compression::ZStd => Ok(Self(DynDecoderImpl::ZstdDbn(
                dbn::Decoder::with_upgrade_policy(
                    ::zstd::stream::Decoder::with_buffer(reader)
                        .map_err(|e| crate::Error::io(e, "creating zstd decoder"))?,
                    upgrade_policy,
                )?,
            ))),
        }
    }

    /// Creates a new [`DynDecoder`] from a buffered reader, inferring the encoding
    /// and compression.It will decode records from previous DBN versions according
    /// to `upgrade_policy`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to determine
    /// the encoding of `reader` or it fails to parse the metadata.
    pub fn inferred_with_buffer(
        mut reader: R,
        upgrade_policy: VersionUpgradePolicy,
    ) -> crate::Result<Self> {
        let first_bytes = reader
            .fill_buf()
            .map_err(|e| crate::Error::io(e, "creating buffer to infer encoding"))?;
        #[allow(deprecated)]
        if dbz::starts_with_prefix(first_bytes) {
            Ok(Self(DynDecoderImpl::LegacyDbz(
                dbz::Decoder::with_upgrade_policy(reader, upgrade_policy)?,
            )))
        } else if dbn::starts_with_prefix(first_bytes) {
            Ok(Self(DynDecoderImpl::Dbn(
                dbn::Decoder::with_upgrade_policy(reader, upgrade_policy)?,
            )))
        } else if zstd::starts_with_prefix(first_bytes) {
            Ok(Self(DynDecoderImpl::ZstdDbn(
                dbn::Decoder::with_upgrade_policy(
                    ::zstd::stream::Decoder::with_buffer(reader)
                        .map_err(|e| crate::Error::io(e, "creating zstd decoder"))?,
                    upgrade_policy,
                )?,
            )))
        } else {
            Err(crate::Error::decode("unable to determine encoding"))
        }
    }
}

impl<'a> DynDecoder<'a, BufReader<File>> {
    /// Creates a new [`DynDecoder`] from the file at `path`. It will decode records
    /// from previous DBN versions according to `upgrade_policy`.
    ///
    /// # Errors
    /// This function will return an error if the file doesn't exist, it is unable to
    /// determine the encoding of the file or it fails to parse the metadata.
    pub fn from_file(
        path: impl AsRef<Path>,
        upgrade_policy: VersionUpgradePolicy,
    ) -> crate::Result<Self> {
        let file = File::open(path.as_ref()).map_err(|e| {
            crate::Error::io(
                e,
                format!(
                    "opening file to decode at path '{}'",
                    path.as_ref().display()
                ),
            )
        })?;
        DynDecoder::new_inferred(file, upgrade_policy)
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
impl<'a, R> DbnMetadata for DynDecoder<'a, R>
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

    fn metadata_mut(&mut self) -> &mut Metadata {
        match &mut self.0 {
            DynDecoderImpl::Dbn(decoder) => decoder.metadata_mut(),
            DynDecoderImpl::ZstdDbn(decoder) => decoder.metadata_mut(),
            DynDecoderImpl::LegacyDbz(decoder) => decoder.metadata_mut(),
        }
    }
}

#[allow(deprecated)]
impl<'a, R> DecodeRecord for DynDecoder<'a, R>
where
    R: io::BufRead,
{
    fn decode_record<T: HasRType>(&mut self) -> crate::Result<Option<&T>> {
        match &mut self.0 {
            DynDecoderImpl::Dbn(decoder) => decoder.decode_record(),
            DynDecoderImpl::ZstdDbn(decoder) => decoder.decode_record(),
            DynDecoderImpl::LegacyDbz(decoder) => decoder.decode_record(),
        }
    }
}

impl<'a, R> DecodeStream for DynDecoder<'a, R>
where
    R: io::BufRead,
{
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

#[doc(hidden)]
pub mod private {
    /// An implementation detail for the interaction between [`StreamingIterator`] and
    /// implementors of [`DecodeRecord`].
    #[doc(hidden)]
    pub trait BufferSlice {
        /// Returns an immutable slice of the decoder's buffer.
        fn buffer_slice(&self) -> &[u8];
    }
}

pub(crate) trait FromLittleEndianSlice {
    fn from_le_slice(slice: &[u8]) -> Self;
}

impl FromLittleEndianSlice for u64 {
    /// # Panics
    /// Panics if the length of `slice` is less than 8 bytes.
    fn from_le_slice(slice: &[u8]) -> Self {
        let (bytes, _) = slice.split_at(mem::size_of::<Self>());
        Self::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl FromLittleEndianSlice for i32 {
    /// # Panics
    /// Panics if the length of `slice` is less than 4 bytes.
    fn from_le_slice(slice: &[u8]) -> Self {
        let (bytes, _) = slice.split_at(mem::size_of::<Self>());
        Self::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl FromLittleEndianSlice for u32 {
    /// # Panics
    /// Panics if the length of `slice` is less than 4 bytes.
    fn from_le_slice(slice: &[u8]) -> Self {
        let (bytes, _) = slice.split_at(mem::size_of::<Self>());
        Self::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl FromLittleEndianSlice for u16 {
    /// # Panics
    /// Panics if the length of `slice` is less than 2 bytes.
    fn from_le_slice(slice: &[u8]) -> Self {
        let (bytes, _) = slice.split_at(mem::size_of::<Self>());
        Self::from_le_bytes(bytes.try_into().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use crate::enums::VersionUpgradePolicy;

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
        let res = DynDecoder::new_inferred(io::Cursor::new(buf), VersionUpgradePolicy::default());
        assert!(matches!(res, Err(e) if e
            .to_string()
            .contains("can't decode newer version of DBN")));
    }
}

#[cfg(feature = "async")]
pub use self::{
    dbn::{
        AsyncDecoder as AsyncDbnDecoder, AsyncMetadataDecoder as AsyncDbnMetadataDecoder,
        AsyncRecordDecoder as AsyncDbnRecordDecoder,
    },
    r#async::DynReader as AsyncDynReader,
};

#[cfg(feature = "async")]
mod r#async {
    use std::{path::Path, pin::Pin};

    use async_compression::tokio::bufread::ZstdDecoder;
    use tokio::{
        fs::File,
        io::{self, BufReader},
    };

    pub(crate) const ZSTD_FILE_BUFFER_CAPACITY: usize = 1 << 20;

    use crate::enums::Compression;

    use super::zstd::zstd_decoder;

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
        ZStd(ZstdDecoder<R>),
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
                Compression::ZStd => Self(DynReaderImpl::ZStd(ZstdDecoder::new(reader))),
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
                Self(DynReaderImpl::ZStd(zstd_decoder(reader)))
            } else {
                Self(DynReaderImpl::Uncompressed(reader))
            })
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
                DynReaderImpl::ZStd(dec) => io::AsyncRead::poll_read(Pin::new(dec), cx, buf),
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
