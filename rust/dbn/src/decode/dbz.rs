//! Decoding of legacy DBZ files, a precursor to DBN.
use std::{
    fs::File,
    io::{self, BufReader, Read},
    mem,
    num::NonZeroU64,
    path::Path,
};

use anyhow::{anyhow, Context};

use crate::{
    decode::{dbn::decode_iso8601, FromLittleEndianSlice},
    enums::{Compression, SType, Schema},
    record::{transmute_record_bytes, HasRType},
    record_ref::RecordRef,
    MappingInterval, Metadata, SymbolMapping,
};

use super::{private::BufferSlice, zstd::ZSTD_SKIPPABLE_MAGIC_RANGE, DecodeDbn, StreamIterDecoder};

/// Object for reading, parsing, and serializing a legacy Databento Binary Encoding (DBZ) file.
pub struct Decoder<R: io::BufRead> {
    reader: zstd::Decoder<'static, R>,
    metadata: Metadata,
    buffer: Vec<u8>,
}

/// Returns `true` if `bytes` starts with valid DBZ.
pub fn starts_with_prefix(bytes: &[u8]) -> bool {
    if bytes.len() < 12 {
        return false;
    }
    let magic = u32::from_le_slice(&bytes[..4]);
    if !ZSTD_SKIPPABLE_MAGIC_RANGE.contains(&magic) {
        return false;
    }
    // frame length doesn't indicate anything; skip ahead to DBZ prefix
    &bytes[8..11] == MetadataDecoder::DBZ_PREFIX && bytes[11] == MetadataDecoder::SCHEMA_VERSION
}

impl Decoder<BufReader<File>> {
    /// Creates a new [`Decoder`] from the file at `path`. This function reads the metadata,
    /// but does not read the body of the file.
    ///
    /// # Errors
    /// This function will return an error if `path` doesn't exist. It will also return an error
    /// if it is unable to parse the metadata from the file.
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = File::open(path.as_ref()).with_context(|| {
            format!(
                "Error opening dbn file at path '{}'",
                path.as_ref().display()
            )
        })?;
        let reader = BufReader::new(file);
        Self::new(reader)
    }
}

// `BufRead` instead of `Read` because the [zstd::Decoder] works with `BufRead` so accepting
// a `Read` could result in redundant `BufReader`s being created.
impl<R: io::BufRead> Decoder<R> {
    /// Creates a new [`Decoder`] from `reader`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in `reader`.
    pub fn new(mut reader: R) -> anyhow::Result<Self> {
        let metadata = MetadataDecoder::read(&mut reader)?;
        let reader = zstd::Decoder::with_buffer(reader)?;
        Ok(Self {
            reader,
            metadata,
            buffer: vec![0],
        })
    }
}

impl<R: io::BufRead> DecodeDbn for Decoder<R> {
    /// Returns a reference to all metadata read from the DBZ data in a [`Metadata`] object.
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

    fn decode_record_ref(&mut self) -> Option<RecordRef> {
        if self.reader.read_exact(&mut self.buffer[..1]).is_err() {
            return None;
        }
        let length = self.buffer[0] as usize * 4;
        if length > self.buffer.len() {
            self.buffer.resize(length, 0);
        }
        if self.reader.read_exact(&mut self.buffer[1..length]).is_err() {
            return None;
        }
        // Safety: `buffer` is resized to contain at least `length` bytes.
        Some(unsafe { RecordRef::new(self.buffer.as_mut_slice()) })
    }

    /// Try to decode the DBZ file into a streaming iterator. This decodes the
    /// data lazily.
    ///
    /// # Errors
    /// This function will return an error if the zstd portion of the DBZ file
    /// was compressed in an unexpected manner.
    fn decode_stream<T: HasRType>(mut self) -> anyhow::Result<super::StreamIterDecoder<Self, T>>
    where
        Self: Sized,
    {
        self.buffer = vec![0; mem::size_of::<T>()];
        Ok(StreamIterDecoder::new(self))
    }
}

impl<R: io::BufRead> BufferSlice for Decoder<R> {
    fn buffer_slice(&self) -> &[u8] {
        self.buffer.as_slice()
    }
}

/// Object for decoding DBZ metadata from a Zstandard skippable frame.
pub struct MetadataDecoder {}

impl MetadataDecoder {
    const U32_SIZE: usize = mem::size_of::<u32>();
    const FIXED_METADATA_LEN: usize = 96;
    const SCHEMA_VERSION: u8 = 1;
    const VERSION_CSTR_LEN: usize = 4;
    const RESERVED_LEN: usize = 39;
    const DBZ_PREFIX: &[u8] = b"DBZ";

    pub(crate) fn read(reader: &mut impl io::Read) -> anyhow::Result<Metadata> {
        let mut prelude_buffer = [0u8; 2 * mem::size_of::<i32>()];
        reader
            .read_exact(&mut prelude_buffer)
            .with_context(|| "Failed to read metadata prelude")?;
        let magic = u32::from_le_slice(&prelude_buffer[..4]);
        if !ZSTD_SKIPPABLE_MAGIC_RANGE.contains(&magic) {
            return Err(anyhow!("Invalid metadata: no zstd magic number"));
        }
        let frame_size = u32::from_le_slice(&prelude_buffer[4..]);
        // debug!("magic={magic}, frame_size={frame_size}");
        if (frame_size as usize) < Self::FIXED_METADATA_LEN {
            return Err(anyhow!(
                "Frame length cannot be shorter than the fixed metadata size"
            ));
        }

        let mut metadata_buffer = vec![0u8; frame_size as usize];
        reader
            .read_exact(&mut metadata_buffer)
            .with_context(|| "Failed to read metadata")?;
        Self::decode(metadata_buffer)
    }

    fn decode(metadata_buffer: Vec<u8>) -> anyhow::Result<Metadata> {
        const U64_SIZE: usize = mem::size_of::<u64>();
        let mut pos = 0;
        if !matches!(&metadata_buffer[pos..pos + 3], MetadataDecoder::DBZ_PREFIX) {
            return Err(anyhow!("Invalid version string"));
        }
        // Interpret 4th character as an u8, not a char to allow for 254 versions (0 omitted)
        let version = metadata_buffer[pos + 3];
        // assume not forwards compatible
        if version > Self::SCHEMA_VERSION {
            return Err(anyhow!("Can't read newer version of DBZ"));
        }
        pos += Self::VERSION_CSTR_LEN;
        let dataset =
            std::str::from_utf8(&metadata_buffer[pos..pos + crate::METADATA_DATASET_CSTR_LEN])
                .with_context(|| "Failed to read dataset from metadata")?
                // remove null bytes
                .trim_end_matches('\0')
                .to_owned();
        pos += crate::METADATA_DATASET_CSTR_LEN;
        let schema = Schema::try_from(u16::from_le_slice(&metadata_buffer[pos..]))
            .with_context(|| format!("Failed to read schema: '{}'", metadata_buffer[pos]))?;
        pos += mem::size_of::<Schema>();
        let start = u64::from_le_slice(&metadata_buffer[pos..]);
        pos += U64_SIZE;
        let end = u64::from_le_slice(&metadata_buffer[pos..]);
        pos += U64_SIZE;
        let limit = NonZeroU64::new(u64::from_le_slice(&metadata_buffer[pos..]));
        pos += U64_SIZE;
        let record_count = u64::from_le_slice(&metadata_buffer[pos..]);
        pos += U64_SIZE;
        // Unused in new Metadata
        let _compression = Compression::try_from(metadata_buffer[pos])
            .with_context(|| format!("Failed to parse compression '{}'", metadata_buffer[pos]))?;
        pos += mem::size_of::<Compression>();
        let stype_in = SType::try_from(metadata_buffer[pos])
            .with_context(|| format!("Failed to read stype_in: '{}'", metadata_buffer[pos]))?;
        pos += mem::size_of::<SType>();
        let stype_out = SType::try_from(metadata_buffer[pos])
            .with_context(|| format!("Failed to read stype_out: '{}'", metadata_buffer[pos]))?;
        pos += mem::size_of::<SType>();
        // skip reserved
        pos += Self::RESERVED_LEN;
        // remaining metadata is compressed
        let mut zstd_decoder = zstd::Decoder::new(&metadata_buffer[pos..])
            .with_context(|| "Failed to read zstd-zipped variable-length metadata".to_owned())?;

        // decompressed variable-length metadata buffer
        let buffer_capacity = (metadata_buffer.len() - pos) * 3; // 3x is arbitrary
        let mut var_buffer = Vec::with_capacity(buffer_capacity);
        zstd_decoder.read_to_end(&mut var_buffer)?;
        pos = 0;
        let schema_definition_length = u32::from_le_slice(&var_buffer[pos..]);
        if schema_definition_length != 0 {
            return Err(anyhow!(
                "This version of dbn can't parse schema definitions"
            ));
        }
        pos += Self::U32_SIZE + (schema_definition_length as usize);
        let symbols = Self::decode_repeated_symbol_cstr(var_buffer.as_slice(), &mut pos)
            .with_context(|| "Failed to parse symbols")?;
        let partial = Self::decode_repeated_symbol_cstr(var_buffer.as_slice(), &mut pos)
            .with_context(|| "Failed to parse partial")?;
        let not_found = Self::decode_repeated_symbol_cstr(var_buffer.as_slice(), &mut pos)
            .with_context(|| "Failed to parse not_found")?;
        let mappings = Self::decode_symbol_mappings(var_buffer.as_slice(), &mut pos)?;

        Ok(Metadata {
            version: 0,
            dataset,
            schema,
            stype_in,
            stype_out,
            start,
            end: NonZeroU64::new(end),
            limit,
            // compression,
            record_count: Some(record_count),
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
        const U32_SIZE: usize = mem::size_of::<u32>();
        const MIN_SYMBOL_MAPPING_ENCODED_SIZE: usize = crate::SYMBOL_CSTR_LEN + U32_SIZE;
        const MAPPING_INTERVAL_ENCODED_SIZE: usize = U32_SIZE * 2 + crate::SYMBOL_CSTR_LEN;

        if *pos + MIN_SYMBOL_MAPPING_ENCODED_SIZE > buffer.len() {
            return Err(anyhow!(
                "Unexpected end of metadata buffer while parsing symbol mapping"
            ));
        }
        let native_symbol =
            Self::decode_symbol(buffer, pos).with_context(|| "Couldn't parse native symbol")?;
        let interval_count = u32::from_le_slice(&buffer[*pos..]) as usize;
        *pos += Self::U32_SIZE;
        let read_size = interval_count * MAPPING_INTERVAL_ENCODED_SIZE;
        if *pos + read_size > buffer.len() {
            return Err(anyhow!(
                "Symbol mapping interval_count ({interval_count}) doesn't match size of buffer \
                which only contains space for {} intervals",
                (buffer.len() - *pos) / MAPPING_INTERVAL_ENCODED_SIZE
            ));
        }
        let mut intervals = Vec::with_capacity(interval_count);
        for i in 0..interval_count {
            let raw_start_date = u32::from_le_slice(&buffer[*pos..]);
            *pos += U32_SIZE;
            let start_date = decode_iso8601(raw_start_date).with_context(|| {
                format!("Failed to parse start date of mapping interval at index {i}")
            })?;
            let raw_end_date = u32::from_le_slice(&buffer[*pos..]);
            *pos += U32_SIZE;
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

#[cfg(test)]
mod tests {
    use streaming_iterator::StreamingIterator;

    use super::*;
    use crate::decode::tests::TEST_DATA_PATH;
    use crate::record::{InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, TbboMsg, TradeMsg};

    /// there are crates like rstest that provide pytest-like parameterized tests, however
    /// they don't support passing types
    macro_rules! test_reading_dbz {
        // Rust doesn't allow concatenating identifiers in stable rust, so each test case needs
        // to be named explicitly
        ($test_name:ident, $record_type:ident, $schema:expr) => {
            #[test]
            fn $test_name() {
                let target = Decoder::from_file(format!(
                    "{TEST_DATA_PATH}/test_data.{}.dbz",
                    $schema.as_str()
                ))
                .unwrap();
                let exp_row_count = target.metadata().record_count;
                assert_eq!(target.metadata().schema, $schema);
                let actual_row_count = target.decode_stream::<$record_type>().unwrap().count();
                assert_eq!(exp_row_count.unwrap() as usize, actual_row_count);
            }
        };
    }

    test_reading_dbz!(test_reading_mbo, MboMsg, Schema::Mbo);
    test_reading_dbz!(test_reading_mbp1, Mbp1Msg, Schema::Mbp1);
    test_reading_dbz!(test_reading_mbp10, Mbp10Msg, Schema::Mbp10);
    test_reading_dbz!(test_reading_ohlcv1d, OhlcvMsg, Schema::Ohlcv1D);
    test_reading_dbz!(test_reading_ohlcv1h, OhlcvMsg, Schema::Ohlcv1H);
    test_reading_dbz!(test_reading_ohlcv1m, OhlcvMsg, Schema::Ohlcv1M);
    test_reading_dbz!(test_reading_ohlcv1s, OhlcvMsg, Schema::Ohlcv1S);
    test_reading_dbz!(test_reading_tbbo, TbboMsg, Schema::Tbbo);
    test_reading_dbz!(test_reading_trades, TradeMsg, Schema::Trades);
    test_reading_dbz!(
        test_reading_definition,
        InstrumentDefMsg,
        Schema::Definition
    );

    #[test]
    fn test_decode_symbol() {
        let bytes = b"SPX.1.2\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
        assert_eq!(bytes.len(), crate::SYMBOL_CSTR_LEN);
        let mut pos = 0;
        let res = MetadataDecoder::decode_symbol(bytes.as_slice(), &mut pos).unwrap();
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
        let res = MetadataDecoder::decode_symbol(BYTES.as_slice(), &mut pos);
        assert!(matches!(res, Err(e) if e.to_string().contains("Failed to decode bytes [")));
    }
}
