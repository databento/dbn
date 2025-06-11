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
mod dyn_decoder;
mod dyn_reader;
mod merge;
mod stream;
// used in databento_dbn
#[doc(hidden)]
pub mod zstd;

// Re-exports
pub use self::dbn::{
    Decoder as DbnDecoder, MetadataDecoder as DbnMetadataDecoder, RecordDecoder as DbnRecordDecoder,
};
pub use dyn_decoder::DynDecoder;
pub use dyn_reader::*;
pub use merge::{Decoder as MergeDecoder, RecordDecoder as MergeRecordDecoder};
pub use stream::StreamIterDecoder;

use std::{io::Seek, mem};

use crate::{HasRType, Metadata, RecordRef, VersionUpgradePolicy};

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
pub trait DecodeStream: DecodeRecord + private::LastRecord {
    /// Converts the decoder into a streaming iterator of records of type `T`. This
    /// lazily decodes the data.
    fn decode_stream<T: HasRType>(self) -> StreamIterDecoder<Self, T>
    where
        Self: Sized;
}
/// Like [`Seek`], but only allows seeking forward from the current
/// position.
pub trait SkipBytes {
    /// Skips `n_bytes` ahead.
    ///
    /// # Errors
    /// This function returns an error if the I/O operations fail.
    fn skip_bytes(&mut self, n_bytes: usize) -> crate::Result<()>;
}

impl<T> SkipBytes for T
where
    T: Seek,
{
    fn skip_bytes(&mut self, n_bytes: usize) -> crate::Result<()> {
        self.seek(std::io::SeekFrom::Current(n_bytes as i64))
            .map(drop)
            .map_err(|err| crate::Error::io(err, format!("seeking ahead {n_bytes} bytes")))
    }
}

/// Async trait for types that decode references to DBN records of a dynamic type.
#[cfg(feature = "async")]
#[allow(async_fn_in_trait)] // the futures can't be Send because self is borrowed mutably
pub trait AsyncDecodeRecordRef {
    /// Tries to decode a generic reference a record. Returns `Ok(None)` if input
    /// has been exhausted.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an error of a
    /// kind other than `io::ErrorKind::UnexpectedEof` upon reading.
    ///
    /// If the `length` property of the record is invalid, an
    /// [`Error::Decode`](crate::Error::Decode) will be returned.
    ///
    /// # Cancel safety
    /// This method is cancel safe. It can be used within a `tokio::select!` statement
    /// without the potential for corrupting the input stream.
    async fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef>>;
}

/// Async trait for types that decode DBN records of a particular type.
#[cfg(feature = "async")]
#[allow(async_fn_in_trait)] // the futures can't be Send because self is borrowed mutably
pub trait AsyncDecodeRecord {
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
    ///
    /// # Cancel safety
    /// This method is cancel safe. It can be used within a `tokio::select!` statement
    /// without the potential for corrupting the input stream.
    async fn decode_record<'a, T: HasRType + 'a>(&'a mut self) -> crate::Result<Option<&'a T>>;

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
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If used within a `tokio::select!` statement
    /// partially decoded records will be lost and the stream may be corrupted.
    async fn decode_records<T: HasRType + Clone>(&mut self) -> crate::Result<Vec<T>>
    where
        Self: Sized,
    {
        let mut res = Vec::new();
        while let Some(rec) = self.decode_record::<T>().await? {
            res.push(rec.clone());
        }
        Ok(res)
    }
}

/// Like [`AsyncSeek`](tokio::io::AsyncSeek), but only allows seeking forward from the current position.
#[cfg(feature = "async")]
#[allow(async_fn_in_trait)] // the futures can't be Send because self is borrowed mutably
pub trait AsyncSkipBytes {
    /// Skips ahead `n_bytes` bytes.
    ///
    /// # Errors
    /// This function returns an error if the I/O operations fail.
    async fn skip_bytes(&mut self, n_bytes: usize) -> crate::Result<()>;
}

#[cfg(feature = "async")]
const ZSTD_FILE_BUFFER_CAPACITY: usize = 1 << 20;

#[doc(hidden)]
pub mod private {
    use crate::RecordRef;

    /// An implementation detail for the interaction between [`StreamingIterator`] and
    /// implementors of [`DecodeRecord`].
    #[doc(hidden)]
    pub trait LastRecord {
        fn last_record(&self) -> Option<RecordRef>;
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

#[cfg(feature = "async")]
pub use self::dbn::{
    AsyncDecoder as AsyncDbnDecoder, AsyncMetadataDecoder as AsyncDbnMetadataDecoder,
    AsyncRecordDecoder as AsyncDbnRecordDecoder,
};

#[cfg(test)]
mod tests {
    pub const TEST_DATA_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/data");
}
