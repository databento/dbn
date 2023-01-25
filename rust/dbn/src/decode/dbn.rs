//! Decoding of DBN files.
use std::{
    fs::File,
    io::{self, BufReader},
    mem,
    num::NonZeroU64,
    path::Path,
};

use anyhow::{anyhow, Context};

use super::{private::BufferSlice, DecodeDbn, FromLittleEndianSlice, StreamIterDecoder};
use crate::{
    enums::{SType, Schema},
    record::{transmute_record_bytes, HasRType},
    // record_ref::RecordRef,
    MappingInterval,
    Metadata,
    SymbolMapping,
    DBN_VERSION,
    METADATA_FIXED_LEN,
};

/// Returns `true` if `bytes` starts with valid uncompressed DBN.
pub fn starts_with_prefix(bytes: &[u8]) -> bool {
    bytes.len() >= 4
        && &bytes[..3] == MetadataDecoder::<File>::DBN_PREFIX
        && bytes[3] <= crate::DBN_VERSION
}

/// Type for decoding files and streams in Databento Binary Encoding (DBN).
pub struct Decoder<R>
where
    R: io::Read,
{
    reader: R,
    metadata: Metadata,
    buffer: Vec<u8>,
}

impl<R> Decoder<R>
where
    R: io::Read,
{
    /// Crates a new DBN [`Decoder`] from `reader`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in `reader`.
    pub fn new(mut reader: R) -> anyhow::Result<Self> {
        let metadata = MetadataDecoder::new(&mut reader).decode()?;
        Ok(Self {
            reader,
            metadata,
            // buffer should capacity for reading `length`
            buffer: vec![0],
        })
    }
}

impl<'a, R> Decoder<zstd::stream::Decoder<'a, BufReader<R>>>
where
    R: io::Read,
{
    /// Crates a new DBN [`Decoder`] from Zstandard-compressed `reader`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in `reader`.
    pub fn with_zstd(reader: R) -> anyhow::Result<Self> {
        Decoder::new(zstd::stream::Decoder::new(reader)?)
    }
}

impl<'a, R> Decoder<zstd::stream::Decoder<'a, R>>
where
    R: io::BufRead,
{
    /// Crates a new DBN [`Decoder`] from Zstandard-compressed buffered `reader`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in `reader`.
    pub fn with_zstd_buffer(reader: R) -> anyhow::Result<Self> {
        Decoder::new(zstd::stream::Decoder::with_buffer(reader)?)
    }
}

impl Decoder<BufReader<File>> {
    /// Creates a DBN [`Decoder`] from the file at `path`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to read the file at `path` or
    /// if it is unable to parse the metadata in the file.
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = File::open(path.as_ref()).with_context(|| {
            format!(
                "Error opening DBN file at path '{}'",
                path.as_ref().display()
            )
        })?;
        Self::new(BufReader::new(file))
    }
}

impl<'a> Decoder<zstd::stream::Decoder<'a, BufReader<File>>> {
    /// Creates a DBN [`Decoder`] from the Zstandard-compressed file at `path`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to read the file at `path` or
    /// if it is unable to parse the metadata in the file.
    pub fn from_zstd_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = File::open(path.as_ref()).with_context(|| {
            format!(
                "Error opening Zstandard-compressed DBN file at path '{}'",
                path.as_ref().display()
            )
        })?;
        Self::with_zstd(file)
    }
}

impl<R> DecodeDbn for Decoder<R>
where
    R: io::Read,
{
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn decode_record<T: HasRType>(&mut self) -> Option<&T> {
        self.buffer.resize(mem::size_of::<T>(), 0);
        if self.reader.read_exact(&mut self.buffer).is_ok() {
            // Safety: `buffer` if specifically sized for `T` and
            // `transmute_record_bytes` verifies the `rtype` is correct.
            unsafe { transmute_record_bytes(self.buffer.as_slice()) }
        } else {
            None
        }
    }

    // fn decode_record_ref<'a>(&'a mut self) -> Option<RecordRef<'a>> {
    //     if self.reader.read_exact(&mut self.buffer[..1]).is_err() {
    //         return None;
    //     }
    //     let length = self.buffer[0] as usize * 4;
    //     if length > self.buffer.len() {
    //         self.buffer.resize(length, 0);
    //     }
    //     if self
    //         .reader
    //         .read_exact(&mut self.buffer[1..length - 1])
    //         .is_err()
    //     {
    //         return None;
    //     }
    //     // Safety: `buffer` is resized to contain at least `length` bytes.
    //     Some(unsafe { RecordRef::new(self.buffer.as_mut_slice()) })
    // }

    fn decode_stream<T: HasRType>(self) -> anyhow::Result<super::StreamIterDecoder<Self, T>> {
        // FIXME: verify `T` matches schema
        Ok(StreamIterDecoder::new(self))
    }
}

impl<R> BufferSlice for Decoder<R>
where
    R: io::Read,
{
    fn buffer_slice(&self) -> &[u8] {
        self.buffer.as_slice()
    }
}

/// Type for decoding [`Metadata`](crate::Metadata) from Databento Binary Encoding (DBN).
pub struct MetadataDecoder<R>
where
    R: io::Read,
{
    reader: R,
}

impl<R> MetadataDecoder<R>
where
    R: io::Read,
{
    const DBN_PREFIX: &[u8] = b"DBN";
    const U32_SIZE: usize = mem::size_of::<u32>();

    /// Crates a new DBN [`MetadataDecoder`] from `reader`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in `reader`.
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Decodes and returns a DBN [`Metadata`].
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata.
    pub fn decode(&mut self) -> anyhow::Result<Metadata> {
        let mut prelude_buffer = [0u8; 8];
        self.reader
            .read_exact(&mut prelude_buffer)
            .with_context(|| "Failed to read DBN metadata prelude")?;
        if &prelude_buffer[..3] != Self::DBN_PREFIX {
            return Err(anyhow!("Invalid DBN header"));
        }
        let version = prelude_buffer[3];
        if version > DBN_VERSION {
            return Err(anyhow!("Can't decode newer version of DBN. Decododer version is {DBN_VERSION}, input version is {version}"));
        }
        let length = u32::from_le_slice(&prelude_buffer[4..]);
        if (length as usize) < METADATA_FIXED_LEN {
            return Err(anyhow!(
                "Invalid DBN metadata. Metadata length shorter than fixed length."
            ));
        }
        let mut metadata_buffer = vec![0u8; length as usize];
        self.reader
            .read_exact(&mut metadata_buffer)
            .with_context(|| "Failed to read metadata")?;
        Self::decode_metadata_fields(version, metadata_buffer)
    }

    fn decode_metadata_fields(version: u8, buffer: Vec<u8>) -> anyhow::Result<Metadata> {
        const U64_SIZE: usize = mem::size_of::<u64>();
        let mut pos = 0;
        let dataset = std::str::from_utf8(&buffer[pos..pos + crate::METADATA_DATASET_CSTR_LEN])
            .with_context(|| "Failed to read dataset from metadata")?
            // remove null bytes
            .trim_end_matches('\0')
            .to_owned();
        pos += crate::METADATA_DATASET_CSTR_LEN;
        let schema = Schema::try_from(u16::from_le_slice(&buffer[pos..]))
            .with_context(|| format!("Failed to read schema: '{}'", buffer[pos]))?;
        pos += mem::size_of::<Schema>();
        let start = u64::from_le_slice(&buffer[pos..]);
        pos += U64_SIZE;
        let end = u64::from_le_slice(&buffer[pos..]);
        pos += U64_SIZE;
        let limit = NonZeroU64::new(u64::from_le_slice(&buffer[pos..]));
        pos += U64_SIZE;
        let record_count = u64::from_le_slice(&buffer[pos..]);
        pos += U64_SIZE;
        let stype_in = SType::try_from(buffer[pos])
            .with_context(|| format!("Failed to read stype_in: '{}'", buffer[pos]))?;
        pos += mem::size_of::<SType>();
        let stype_out = SType::try_from(buffer[pos])
            .with_context(|| format!("Failed to read stype_out: '{}'", buffer[pos]))?;
        pos += mem::size_of::<SType>();
        // skip reserved
        pos += crate::METADATA_RESERVED_LEN;
        let schema_definition_length = u32::from_le_slice(&buffer[pos..]);
        if schema_definition_length != 0 {
            return Err(anyhow!(
                "This version of DBN can't parse schema definitions"
            ));
        }
        pos += Self::U32_SIZE + (schema_definition_length as usize);
        let symbols = Self::decode_repeated_symbol_cstr(buffer.as_slice(), &mut pos)
            .with_context(|| "Failed to parse symbols")?;
        let partial = Self::decode_repeated_symbol_cstr(buffer.as_slice(), &mut pos)
            .with_context(|| "Failed to parse partial")?;
        let not_found = Self::decode_repeated_symbol_cstr(buffer.as_slice(), &mut pos)
            .with_context(|| "Failed to parse not_found")?;
        let mappings = Self::decode_symbol_mappings(buffer.as_slice(), &mut pos)?;

        Ok(Metadata {
            version,
            dataset,
            schema,
            stype_in,
            stype_out,
            start,
            end,
            limit,
            record_count,
            symbols,
            partial,
            not_found,
            mappings,
        })
    }
    fn decode_repeated_symbol_cstr(buffer: &[u8], pos: &mut usize) -> anyhow::Result<Vec<String>> {
        if *pos + Self::U32_SIZE > buffer.len() {
            return Err(anyhow!("Unexpected end of metadata buffer"));
        }
        let count = u32::from_le_slice(&buffer[*pos..]) as usize;
        *pos += Self::U32_SIZE;
        let read_size = count * crate::SYMBOL_CSTR_LEN;
        if *pos + read_size > buffer.len() {
            return Err(anyhow!("Unexpected end of metadata buffer"));
        }
        let mut res = Vec::with_capacity(count);
        for i in 0..count {
            res.push(
                Self::decode_symbol(buffer, pos)
                    .with_context(|| format!("Failed to decode symbol at index {i}"))?,
            );
        }
        Ok(res)
    }

    fn decode_symbol_mappings(
        buffer: &[u8],
        pos: &mut usize,
    ) -> anyhow::Result<Vec<SymbolMapping>> {
        if *pos + Self::U32_SIZE > buffer.len() {
            return Err(anyhow!("Unexpected end of metadata buffer"));
        }
        let count = u32::from_le_slice(&buffer[*pos..]) as usize;
        *pos += Self::U32_SIZE;
        let mut res = Vec::with_capacity(count);
        // Because each `SymbolMapping` itself is of a variable length, decoding it requires frequent bounds checks
        for i in 0..count {
            res.push(
                Self::decode_symbol_mapping(buffer, pos)
                    .with_context(|| format!("Failed to parse symbol mapping at index {i}"))?,
            );
        }
        Ok(res)
    }

    fn decode_symbol_mapping(buffer: &[u8], pos: &mut usize) -> anyhow::Result<SymbolMapping> {
        const MIN_SYMBOL_MAPPING_ENCODED_LEN: usize =
            crate::SYMBOL_CSTR_LEN + mem::size_of::<u32>();
        const MAPPING_INTERVAL_ENCODED_LEN: usize =
            mem::size_of::<u32>() * 2 + crate::SYMBOL_CSTR_LEN;

        if *pos + MIN_SYMBOL_MAPPING_ENCODED_LEN > buffer.len() {
            return Err(anyhow!(
                "Unexpected end of metadata buffer while parsing symbol mapping"
            ));
        }
        let native_symbol =
            Self::decode_symbol(buffer, pos).with_context(|| "Couldn't parse native symbol")?;
        let interval_count = u32::from_le_slice(&buffer[*pos..]) as usize;
        *pos += Self::U32_SIZE;
        let read_size = interval_count * MAPPING_INTERVAL_ENCODED_LEN;
        if *pos + read_size > buffer.len() {
            return Err(anyhow!(
                "Symbol mapping interval_count ({interval_count}) doesn't match size of buffer \
                which only contains space for {} intervals",
                (buffer.len() - *pos) / MAPPING_INTERVAL_ENCODED_LEN
            ));
        }
        let mut intervals = Vec::with_capacity(interval_count);
        for i in 0..interval_count {
            let raw_start_date = u32::from_le_slice(&buffer[*pos..]);
            *pos += Self::U32_SIZE;
            let start_date = decode_iso8601(raw_start_date).with_context(|| {
                format!("Failed to parse start date of mapping interval at index {i}")
            })?;
            let raw_end_date = u32::from_le_slice(&buffer[*pos..]);
            *pos += Self::U32_SIZE;
            let end_date = decode_iso8601(raw_end_date).with_context(|| {
                format!("Failed to parse end date of mapping interval at index {i}")
            })?;
            let symbol = Self::decode_symbol(buffer, pos).with_context(|| {
                format!("Failed to parse symbol for mapping interval at index {i}")
            })?;
            intervals.push(MappingInterval {
                start_date,
                end_date,
                symbol,
            });
        }
        Ok(SymbolMapping {
            native_symbol,
            intervals,
        })
    }

    fn decode_symbol(buffer: &[u8], pos: &mut usize) -> anyhow::Result<String> {
        let symbol_slice = &buffer[*pos..*pos + crate::SYMBOL_CSTR_LEN];
        let symbol = std::str::from_utf8(symbol_slice)
            .with_context(|| format!("Failed to decode bytes {symbol_slice:?}"))?
            // remove null bytes
            .trim_end_matches('\0')
            .to_owned();
        *pos += crate::SYMBOL_CSTR_LEN;
        Ok(symbol)
    }
}

pub(crate) fn decode_iso8601(raw: u32) -> anyhow::Result<time::Date> {
    let year = raw / 10_000;
    let remaining = raw % 10_000;
    let raw_month = remaining / 100;
    let month = u8::try_from(raw_month)
        .map_err(|e| anyhow!(e))
        .and_then(|m| time::Month::try_from(m).map_err(|e| anyhow!(e)))
        .with_context(|| format!("Invalid month {raw_month} while parsing {raw} into a date"))?;
    let day = remaining % 100;
    time::Date::from_calendar_date(year as i32, month, day as u8)
        .with_context(|| format!("Couldn't convert {raw} to a valid date"))
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    #[test]
    fn test_decode_symbol() {
        let bytes = b"SPX.1.2\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
        assert_eq!(bytes.len(), crate::SYMBOL_CSTR_LEN);
        let mut pos = 0;
        let res = MetadataDecoder::<File>::decode_symbol(bytes.as_slice(), &mut pos).unwrap();
        assert_eq!(pos, crate::SYMBOL_CSTR_LEN);
        assert_eq!(&res, "SPX.1.2");
    }

    #[test]
    fn test_decode_symbol_invalid_utf8() {
        const BYTES: [u8; 22] = [
            // continuation byte
            0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let mut pos = 0;
        let res = MetadataDecoder::<File>::decode_symbol(BYTES.as_slice(), &mut pos);
        assert!(matches!(res, Err(e) if e.to_string().contains("Failed to decode bytes [")));
    }

    #[test]
    fn test_decode_iso8601_valid() {
        let res = decode_iso8601(20151031).unwrap();
        let exp: time::Date =
            time::Date::from_calendar_date(2015, time::Month::October, 31).unwrap();
        assert_eq!(res, exp);
    }

    #[test]
    fn test_decode_iso8601_invalid_month() {
        let res = decode_iso8601(20101305);
        assert!(matches!(res, Err(e) if e.to_string().contains("Invalid month")));
    }

    #[test]
    fn test_decode_iso8601_invalid_day() {
        let res = decode_iso8601(20100600);
        assert!(matches!(res, Err(e) if e.to_string().contains("a valid date")));
    }
}
