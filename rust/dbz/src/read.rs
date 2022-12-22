use std::{
    fs::File,
    io::{self, BufReader, Read},
    marker::PhantomData,
    mem,
    path::Path,
};

use anyhow::{anyhow, Context};
use log::{debug, warn};
use serde::Serialize;
use streaming_iterator::StreamingIterator;
use zstd::Decoder;

use databento_defs::{
    enums::{Compression, SType, Schema},
    record::{transmute_record_bytes, ConstTypeId},
};

use crate::write::dbz::SCHEMA_VERSION;

/// Object for reading, parsing, and serializing a Databento Binary Encoding (DBZ) file.
#[derive(Debug)]
pub struct Dbz<R: io::BufRead> {
    reader: R,
    metadata: Metadata,
}

/// Information about the data contained in a DBZ file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Metadata {
    /// The DBZ schema version number.
    pub version: u8,
    /// The dataset name.
    pub dataset: String,
    /// The data record schema. Specifies which record type is stored in the DBZ file.
    pub schema: Schema,
    /// The UNIX nanosecond timestamp of the query start, or the first record if the file was split.
    pub start: u64,
    /// The UNIX nanosecond timestamp of the query end, or the last record if the file was split.
    pub end: u64,
    /// The maximum number of records for the query.
    pub limit: u64,
    /// The total number of data records.
    pub record_count: u64,
    /// The data compression format (if any).
    pub compression: Compression,
    /// The input symbology type to map from.
    pub stype_in: SType,
    /// The output symbology type to map to.
    pub stype_out: SType,
    /// The original query input symbols from the request.
    pub symbols: Vec<String>,
    /// Symbols that did not resolve for _at least one day_ in the query time range.
    pub partial: Vec<String>,
    /// Symbols that did not resolve for _any_ day in the query time range.
    pub not_found: Vec<String>,
    /// Symbol mappings containing a native symbol and its mapping intervals.
    pub mappings: Vec<SymbolMapping>,
}

/// A native symbol and its symbol mappings for different time ranges within the query range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(
    any(feature = "python", feature = "python-test"),
    derive(pyo3::FromPyObject)
)]
pub struct SymbolMapping {
    /// The native symbol.
    pub native: String,
    /// The mappings of `native` for different date ranges.
    pub intervals: Vec<MappingInterval>,
}

/// The resolved symbol for a date range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MappingInterval {
    /// UTC start date of interval.
    #[serde(serialize_with = "serialize_date")]
    pub start_date: time::Date,
    /// UTC end date of interval.
    #[serde(serialize_with = "serialize_date")]
    pub end_date: time::Date,
    /// The resolved symbol for this interval.
    pub symbol: String,
}

// Override `time::Date`'s serialization format to be ISO 8601.
fn serialize_date<S: serde::Serializer>(
    date: &time::Date,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&date.to_string()) // ISO 8601
}

impl Dbz<BufReader<File>> {
    /// Creates a new [`Dbz`] from the file at `path`. This function reads the metadata,
    /// but does not read the body of the file.
    ///
    /// # Errors
    /// This function will return an error if `path` doesn't exist. It will also return an error
    /// if it is unable to parse the metadata from the file.
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = File::open(path.as_ref()).with_context(|| {
            format!(
                "Error opening dbz file at path '{}'",
                path.as_ref().display()
            )
        })?;
        let reader = BufReader::new(file);
        Self::new(reader)
    }
}

// `BufRead` instead of `Read` because the [zstd::Decoder] works with `BufRead` so accepting
// a `Read` could result in redundant `BufReader`s being created.
impl<R: io::BufRead> Dbz<R> {
    /// Creates a new [`Dbz`] from `reader`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in `reader`.
    pub fn new(mut reader: R) -> anyhow::Result<Self> {
        let metadata = Metadata::read(&mut reader)?;
        Ok(Self { reader, metadata })
    }

    /// Returns the [`Schema`] of the DBZ data. The schema also indicates the record type `T` for
    /// [`Self::try_into_iter`].
    pub fn schema(&self) -> Schema {
        self.metadata.schema
    }

    /// Returns a reference to all metadata read from the Dbz data in a [`Metadata`] object.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Try to decode the DBZ file into a streaming iterator. This decodes the
    /// data lazily.
    ///
    /// # Errors
    /// This function will return an error if the zstd portion of the DBZ file
    /// was compressed in an unexpected manner.
    pub fn try_into_iter<T: ConstTypeId>(self) -> anyhow::Result<DbzStreamIter<R, T>> {
        DbzStreamIter::new(self.reader, self.metadata)
    }
}

/// A consuming iterator over a [`Dbz`]. Lazily decompresses and translates the contents of the file
/// or other buffer. This struct is created by the [`Dbz::try_into_iter`] method.
pub struct DbzStreamIter<R: io::BufRead, T> {
    /// [`Metadata`] about the file being iterated
    metadata: Metadata,
    /// Reference to the underlying [`Dbz`] object.
    /// Buffered zstd decoder of the DBZ file, so each call to [`DbzStreamIter::next()`] doesn't result in a
    /// separate system call.
    decoder: Decoder<'static, R>,
    /// Number of elements that have been decoded. Used for [`Iterator::size_hint`].
    i: usize,
    /// Reusable buffer for reading into.
    buffer: Vec<u8>,
    /// Required to associate [`DbzStreamIter`] with a `T`.
    _item: PhantomData<T>,
}

impl<R: io::BufRead, T> DbzStreamIter<R, T> {
    pub(crate) fn new(reader: R, metadata: Metadata) -> anyhow::Result<Self> {
        let decoder = Decoder::with_buffer(reader)?;
        Ok(DbzStreamIter {
            metadata,
            decoder,
            i: 0,
            buffer: vec![0; mem::size_of::<T>()],
            _item: PhantomData {},
        })
    }
}

impl<R: io::BufRead, T: ConstTypeId> StreamingIterator for DbzStreamIter<R, T> {
    type Item = T;

    fn advance(&mut self) {
        if let Err(e) = self.decoder.read_exact(&mut self.buffer) {
            warn!("Failed to read from DBZ decoder: {e:?}");
            self.i = self.metadata.record_count as usize + 1;
        }
        self.i += 1;
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.i > self.metadata.record_count as usize {
            return None;
        }
        // Safety: `buffer` is specifically sized to `T`
        unsafe { transmute_record_bytes(self.buffer.as_slice()) }
    }

    /// Returns the lower bound and upper bounds of remaining length of iterator.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.metadata.record_count as usize - self.i;
        // assumes `record_count` is accurate. If it is not, the program won't crash but
        // performance will be suboptimal
        (remaining, Some(remaining))
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

impl Metadata {
    const U32_SIZE: usize = mem::size_of::<u32>();

    pub(crate) fn read(reader: &mut impl io::Read) -> anyhow::Result<Self> {
        let mut prelude_buffer = [0u8; 2 * mem::size_of::<i32>()];
        reader
            .read_exact(&mut prelude_buffer)
            .with_context(|| "Failed to read metadata prelude")?;
        let magic = u32::from_le_slice(&prelude_buffer[..4]);
        if !Self::ZSTD_MAGIC_RANGE.contains(&magic) {
            return Err(anyhow!("Invalid metadata: no zstd magic number"));
        }
        let frame_size = u32::from_le_slice(&prelude_buffer[4..]);
        debug!("magic={magic}, frame_size={frame_size}");
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

    fn decode(metadata_buffer: Vec<u8>) -> anyhow::Result<Self> {
        const U64_SIZE: usize = mem::size_of::<u64>();
        let mut pos = 0;
        if &metadata_buffer[pos..pos + 3] != b"DBZ" {
            return Err(anyhow!("Invalid version string"));
        }
        // Interpret 4th character as an u8, not a char to allow for 254 versions (0 omitted)
        let version = metadata_buffer[pos + 3];
        // assume not forwards compatible
        if version > SCHEMA_VERSION {
            return Err(anyhow!("Can't read newer version of DBZ"));
        }
        pos += Self::VERSION_CSTR_LEN;
        let dataset = std::str::from_utf8(&metadata_buffer[pos..pos + Self::DATASET_CSTR_LEN])
            .with_context(|| "Failed to read dataset from metadata")?
            // remove null bytes
            .trim_end_matches('\0')
            .to_owned();
        pos += Self::DATASET_CSTR_LEN;
        let schema = Schema::try_from(u16::from_le_slice(&metadata_buffer[pos..]))
            .with_context(|| format!("Failed to read schema: '{}'", metadata_buffer[pos]))?;
        pos += mem::size_of::<Schema>();
        let start = u64::from_le_slice(&metadata_buffer[pos..]);
        pos += U64_SIZE;
        let end = u64::from_le_slice(&metadata_buffer[pos..]);
        pos += U64_SIZE;
        let limit = u64::from_le_slice(&metadata_buffer[pos..]);
        pos += U64_SIZE;
        let record_count = u64::from_le_slice(&metadata_buffer[pos..]);
        pos += U64_SIZE;
        let compression = Compression::try_from(metadata_buffer[pos])
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
        let mut zstd_decoder = Decoder::new(&metadata_buffer[pos..])
            .with_context(|| "Failed to read zstd-zipped variable-length metadata".to_owned())?;

        // decompressed variable-length metadata buffer
        let buffer_capacity = (metadata_buffer.len() - pos) * 3; // 3x is arbitrary
        let mut var_buffer = Vec::with_capacity(buffer_capacity);
        zstd_decoder.read_to_end(&mut var_buffer)?;
        pos = 0;
        let schema_definition_length = u32::from_le_slice(&var_buffer[pos..]);
        if schema_definition_length != 0 {
            return Err(anyhow!(
                "This version of dbz can't parse schema definitions"
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

        Ok(Self {
            version,
            dataset,
            schema,
            stype_in,
            stype_out,
            start,
            end,
            limit,
            compression,
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
        let read_size = count * Self::SYMBOL_CSTR_LEN;
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
        const MIN_SYMBOL_MAPPING_ENCODED_SIZE: usize =
            Metadata::SYMBOL_CSTR_LEN + Metadata::U32_SIZE;
        const MAPPING_INTERVAL_ENCODED_SIZE: usize =
            Metadata::U32_SIZE * 2 + Metadata::SYMBOL_CSTR_LEN;

        if *pos + MIN_SYMBOL_MAPPING_ENCODED_SIZE > buffer.len() {
            return Err(anyhow!(
                "Unexpected end of metadata buffer while parsing symbol mapping"
            ));
        }
        let native =
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
            *pos += Metadata::U32_SIZE;
            let start_date = Self::decode_iso8601(raw_start_date).with_context(|| {
                format!("Failed to parse start date of mapping interval at index {i}")
            })?;
            let raw_end_date = u32::from_le_slice(&buffer[*pos..]);
            *pos += Metadata::U32_SIZE;
            let end_date = Self::decode_iso8601(raw_end_date).with_context(|| {
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
        Ok(SymbolMapping { native, intervals })
    }

    fn decode_symbol(buffer: &[u8], pos: &mut usize) -> anyhow::Result<String> {
        let symbol_slice = &buffer[*pos..*pos + Self::SYMBOL_CSTR_LEN];
        let symbol = std::str::from_utf8(symbol_slice)
            .with_context(|| format!("Failed to decode bytes {symbol_slice:?}"))?
            // remove null bytes
            .trim_end_matches('\0')
            .to_owned();
        *pos += Self::SYMBOL_CSTR_LEN;
        Ok(symbol)
    }

    fn decode_iso8601(raw: u32) -> anyhow::Result<time::Date> {
        let year = raw / 10_000;
        let remaining = raw % 10_000;
        let raw_month = remaining / 100;
        let month = u8::try_from(raw_month)
            .map_err(|e| anyhow!(e))
            .and_then(|m| time::Month::try_from(m).map_err(|e| anyhow!(e)))
            .with_context(|| {
                format!("Invalid month {raw_month} while parsing {raw} into a date")
            })?;
        let day = remaining % 100;
        time::Date::from_calendar_date(year as i32, month, day as u8)
            .with_context(|| format!("Couldn't convert {raw} to a valid date"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use databento_defs::record::{Mbp10Msg, Mbp1Msg, OhlcvMsg, TbboMsg, TickMsg, TradeMsg};

    const DBZ_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/data");

    /// there are crates like rstest that provide pytest-like parameterized tests, however
    /// they don't support passing types
    macro_rules! test_reading_dbz {
        // Rust doesn't allow concatenating identifiers in stable rust, so each test case needs
        // to be named explicitly
        ($test_name:ident, $record_type:ident, $schema:expr) => {
            #[test]
            fn $test_name() {
                let target =
                    Dbz::from_file(format!("{DBZ_PATH}/test_data.{}.dbz", $schema.as_str()))
                        .unwrap();
                let exp_row_count = target.metadata().record_count;
                assert_eq!(target.schema(), $schema);
                let actual_row_count = target.try_into_iter::<$record_type>().unwrap().count();
                assert_eq!(exp_row_count as usize, actual_row_count);
            }
        };
    }

    test_reading_dbz!(test_reading_mbo, TickMsg, Schema::Mbo);
    test_reading_dbz!(test_reading_mbp1, Mbp1Msg, Schema::Mbp1);
    test_reading_dbz!(test_reading_mbp10, Mbp10Msg, Schema::Mbp10);
    test_reading_dbz!(test_reading_ohlcv1d, OhlcvMsg, Schema::Ohlcv1D);
    test_reading_dbz!(test_reading_ohlcv1h, OhlcvMsg, Schema::Ohlcv1H);
    test_reading_dbz!(test_reading_ohlcv1m, OhlcvMsg, Schema::Ohlcv1M);
    test_reading_dbz!(test_reading_ohlcv1s, OhlcvMsg, Schema::Ohlcv1S);
    test_reading_dbz!(test_reading_tbbo, TbboMsg, Schema::Tbbo);
    test_reading_dbz!(test_reading_trades, TradeMsg, Schema::Trades);

    #[test]
    fn test_decode_symbol() {
        let bytes = b"SPX.1.2\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
        assert_eq!(bytes.len(), Metadata::SYMBOL_CSTR_LEN);
        let mut pos = 0;
        let res = Metadata::decode_symbol(bytes.as_slice(), &mut pos).unwrap();
        assert_eq!(pos, Metadata::SYMBOL_CSTR_LEN);
        assert_eq!(&res, "SPX.1.2");
    }

    #[test]
    fn test_decode_symbol_invalid_utf8() {
        const BYTES: [u8; 22] = [
            // continuation byte
            0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let mut pos = 0;
        let res = Metadata::decode_symbol(BYTES.as_slice(), &mut pos);
        assert!(matches!(res, Err(e) if e.to_string().contains("Failed to decode bytes [")));
    }

    #[test]
    fn test_decode_iso8601_valid() {
        let res = Metadata::decode_iso8601(20151031).unwrap();
        let exp: time::Date =
            time::Date::from_calendar_date(2015, time::Month::October, 31).unwrap();
        assert_eq!(res, exp);
    }

    #[test]
    fn test_decode_iso8601_invalid_month() {
        let res = Metadata::decode_iso8601(20101305);
        assert!(matches!(res, Err(e) if e.to_string().contains("Invalid month")));
    }

    #[test]
    fn test_decode_iso8601_invalid_day() {
        let res = Metadata::decode_iso8601(20100600);
        assert!(matches!(res, Err(e) if e.to_string().contains("a valid date")));
    }
}
