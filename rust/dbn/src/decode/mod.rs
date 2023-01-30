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
mod zstd;

use std::{
    fs::File,
    io::{self, BufReader},
    mem,
    path::Path,
};

use anyhow::{anyhow, Context};

use crate::{
    record::HasRType,
    // record_ref::RecordRef,
    Metadata,
};
pub use stream::StreamIterDecoder;

/// Trait for types that decode DBN records.
pub trait DecodeDbn: private::BufferSlice {
    /// Returns a reference to the decoded [`Metadata`].
    fn metadata(&self) -> &Metadata;

    /// Try to decode a reference to a single record of type `T`. Returns `None` if
    /// the input has been exhausted or the next record is not of type `T`.
    fn decode_record<T: HasRType>(&mut self) -> Option<&T>;

    /// Try to decode a generic reference a record.
    // fn decode_record_ref<'a>(&'a mut self) -> Option<RecordRef<'a>>;

    /// Try to convert the decoder into a streaming iterator. This lazily decodes the
    /// data.
    ///
    /// # Errors
    /// This function returns an error if schema of the data being decoded doesn't
    /// correspond with `T`.
    fn decode_stream<T: HasRType>(self) -> anyhow::Result<StreamIterDecoder<Self, T>>
    where
        Self: Sized;

    /// Try to decode all records into a `Vec`. This eagerly decodes the data.
    ///
    /// # Errors
    /// This function returns an error if schema of the data being decoded doesn't
    /// correspond with `T`.
    fn decode_records<T: HasRType + Clone>(mut self) -> anyhow::Result<Vec<T>>
    where
        Self: Sized,
    {
        let mut res = if let Some(record_count) = self.metadata().record_count {
            Vec::with_capacity(record_count as usize)
        } else {
            Vec::new()
        };
        while let Some(rec) = self.decode_record::<T>() {
            res.push(rec.clone());
        }
        Ok(res)
    }
}

/// A decoder implementing [`DecodeDbn`] whose [`Encoding`](crate::enums::Encoding) and
/// [`Compression`](crate::enums::Compression) can be determined at runtime by peeking
/// at the first few bytes.
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
    /// Creates a new [`DynDecoder`] from a reader. If `reader` also implements
    /// [`io::BufRead`](std::io::BufRead), it is better to use [`with_buffer()`](Self::with_buffer).
    ///
    /// # Errors
    /// This function will return an error if it is unable to determine
    /// the encoding of `reader` or it fails to parse the metadata.
    pub fn new(reader: R) -> anyhow::Result<Self> {
        Self::with_buffer(BufReader::new(reader))
    }
}

impl<'a, R> DynDecoder<'a, R>
where
    R: io::BufRead,
{
    /// Creates a new [`DynDecoder`] from a buffered reader.
    ///
    /// # Errors
    /// This function will return an error if it is unable to determine
    /// the encoding of `reader` or it fails to parse the metadata.
    pub fn with_buffer(mut reader: R) -> anyhow::Result<Self> {
        let first_bytes = reader
            .fill_buf()
            .context("Failed to read bytes to determine encoding")?;
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
            Err(anyhow!("Unable to determine encoding"))
        }
    }
}

impl<'a> DynDecoder<'a, BufReader<File>> {
    /// Creates a new [`DynDecoder`] from the file at `path`.
    ///
    /// # Errors
    /// This function will return an error if the file doesn't exist, it is unable to
    /// determine the encoding of the file or it fails to parse the metadata.
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = File::open(path.as_ref()).with_context(|| {
            format!(
                "Error opening file to decode at path '{}'",
                path.as_ref().display()
            )
        })?;
        DynDecoder::new(file)
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

    fn decode_record<T: HasRType>(&mut self) -> Option<&T> {
        match &mut self.0 {
            DynDecoderImpl::Dbn(decoder) => decoder.decode_record(),
            DynDecoderImpl::ZstdDbn(decoder) => decoder.decode_record(),
            DynDecoderImpl::LegacyDbz(decoder) => decoder.decode_record(),
        }
    }

    fn decode_stream<T: HasRType>(self) -> anyhow::Result<StreamIterDecoder<Self, T>>
    where
        Self: Sized,
    {
        Ok(StreamIterDecoder::new(self))
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
    pub const TEST_DATA_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/data");
}
