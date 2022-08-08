use std::{
    fs::File,
    io::{self, BufReader, Read},
    marker::PhantomData,
    mem,
    path::Path,
    str::FromStr,
};

use anyhow::{anyhow, Context};
use log::{debug, warn};
use zstd::Decoder;

use db_def::{
    enums::{Compression, Dataset, Encoding, SType, Schema},
    tick::{CommonHeader, Tick},
};

/// Object for reading, parsing, and serializing a Databento Binary Encoding (DBZ) file.
#[derive(Debug)]
pub struct Dbz<R: io::Read> {
    reader: R,
    metadata: Metadata,
}

/// Information about the data contained in a DBZ file.
#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    /// The DBZ schema version number.
    pub version: u8,
    /// The dataset ID.
    pub dataset: Dataset,
    /// The data record schema. Specifies which tick type is stored in the DBZ file.
    pub schema: Schema,
    /// The input symbol type to map from.
    pub stype_in: SType,
    /// The target output symbol type to map to.
    pub stype_out: SType,
    /// The UNIX nanosecond timestamp of the query start, or the first record if the file was split.
    pub start: u64,
    /// The UNIX nanosecond timestamp of the query end, or the last record if the file was split.
    pub end: u64,
    /// The maximum number of records for the query.
    pub limit: u64,
    /// The data output encoding. Should always be [Encoding::Dbz].
    pub encoding: Encoding,
    /// The data output compression mode.
    pub compression: Compression,
    /// The number of data records for the metadata shape.
    pub nrows: u64,
    /// The number of data columns for the metadata shape.
    pub ncols: u16,
    /// Additional metadata, including symbology.
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Dbz<BufReader<File>> {
    /// Creates a new [Dbz] from the file at `path`. This function reads the metadata,
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
    /// Creates a new [Dbz] from `reader`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in `reader`.
    pub fn new(mut reader: R) -> anyhow::Result<Self> {
        let metadata = Metadata::read(&mut reader)?;
        Ok(Self { reader, metadata })
    }

    /// Returns the [Schema] of the DBZ data. The schema also indicates the tick type `T` for
    /// [Self::try_into_iter].
    pub fn schema(&self) -> Schema {
        self.metadata.schema
    }

    /// Returns a reference to all metadata read from the Dbz data in a [Metadata] object.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Try to decode the DBZ file into an iterator. This decodes the data
    /// lazily.
    ///
    /// # Errors
    /// This function will return an error if the zstd portion of the DBZ file was compressed in
    /// an unexpected manner.
    pub fn try_into_iter<T: TryFrom<Tick>>(self) -> anyhow::Result<DbzIntoIter<R, T>> {
        let decoder = Decoder::with_buffer(self.reader)?;
        Ok(DbzIntoIter {
            metadata: self.metadata,
            decoder,
            i: 0,
            buffer: vec![0; mem::size_of::<T>()],
            _item: PhantomData {},
        })
    }
}

/// A consuming iterator over a [Dbz]. Lazily decompresses and translates the contents of the file
/// or other buffer. This struct is created by the [Dbz::try_into_iter] method.
pub struct DbzIntoIter<R: io::BufRead, T> {
    /// [Metadata] about the file being iterated
    metadata: Metadata,
    /// Reference to the underlying [Dbz] object.
    /// Buffered zstd decoder of the DBZ file, so each call to [DbzIntoIter::next()] doesn't result in a
    /// separate system call.
    decoder: Decoder<'static, R>,
    /// Number of elements that have been decoded. Used for [Iterator::size_hint].
    i: usize,
    /// Reusable buffer for reading into.
    buffer: Vec<u8>,
    /// Required to associate [DbzIntoIter] with a `T`.
    _item: PhantomData<T>,
}

impl<R: io::BufRead, T: TryFrom<Tick>> Iterator for DbzIntoIter<R, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.decoder.read_exact(&mut self.buffer).is_err() {
            return None;
        }
        let tick = match Tick::new(self.buffer.as_ptr() as *const CommonHeader) {
            Ok(tick) => tick,
            Err(e) => {
                warn!("Unexpected tick value: {e}. Raw buffer: {:?}", self.buffer);
                return None;
            }
        };
        self.i += 1;
        T::try_from(tick).ok()
    }

    /// Returns the lower bound and upper bounds of remaining length of iterator.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.metadata.nrows as usize - self.i;
        // assumes `nrows` is always accurate. If it is not, the program won't crash but
        // performance will be suboptimal
        (remaining, Some(remaining))
    }
}

trait FromLittleEndianSlice {
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

impl FromLittleEndianSlice for u16 {
    /// NOTE: assumes the length of `slice` is at least 2 bytes
    fn from_le_slice(slice: &[u8]) -> Self {
        let (bytes, _) = slice.split_at(mem::size_of::<Self>());
        Self::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl Metadata {
    pub(crate) const ZSTD_FIRST_MAGIC: i32 = 0x184D2A50;
    pub(crate) const BENTO_SCHEMA_VERSION: u8 = 1;
    pub(crate) const BENTO_MAGIC: i32 = Self::ZSTD_FIRST_MAGIC + Self::BENTO_SCHEMA_VERSION as i32;
    pub(crate) const DATASET_CSTR_LEN: usize = 16;
    pub(crate) const FIXED_METADATA_LEN: usize = 96;

    pub(crate) fn read(reader: &mut impl io::Read) -> anyhow::Result<Self> {
        let mut prelude_buffer = [0u8; 2 * mem::size_of::<i32>()];
        reader
            .read_exact(&mut prelude_buffer)
            .with_context(|| "Failed to read metadata prelude")?;
        let magic = i32::from_le_slice(&prelude_buffer[..4]);
        if magic != Self::BENTO_MAGIC {
            return Err(anyhow!("Invalid metadata"));
        }
        let frame_size = i32::from_le_slice(&prelude_buffer[4..]);
        debug!("magic={magic}, frame_size={frame_size}");

        let mut metadata_buffer = vec![0u8; frame_size as usize];
        reader
            .read_exact(&mut metadata_buffer)
            .with_context(|| "Failed to read metadata")?;
        Self::decode(metadata_buffer)
    }

    fn decode(metadata_buffer: Vec<u8>) -> anyhow::Result<Self> {
        const U64_SIZE: usize = mem::size_of::<u64>();
        let mut pos = 0;
        let version = metadata_buffer[pos];
        pos += mem::size_of::<u8>();
        let dataset_str = std::str::from_utf8(&metadata_buffer[pos..pos + Self::DATASET_CSTR_LEN])
            .with_context(|| "Failed to read dataset from metadata")?
            // remove null bytes
            .trim_end_matches('\0');
        let dataset = Dataset::from_str(dataset_str)
            .with_context(|| format!("Unknown dataset '{dataset_str}'"))?;
        pos += Self::DATASET_CSTR_LEN;
        let schema = Schema::try_from(metadata_buffer[pos])
            .with_context(|| format!("Failed to read schema: '{}'", metadata_buffer[pos]))?;
        pos += mem::size_of::<Schema>();
        let stype_in = SType::try_from(metadata_buffer[pos])
            .with_context(|| format!("Failed to read stype_in: '{}'", metadata_buffer[pos]))?;
        pos += mem::size_of::<SType>();
        let stype_out = SType::try_from(metadata_buffer[pos])
            .with_context(|| format!("Failed to read stype_out: '{}'", metadata_buffer[pos]))?;
        pos += mem::size_of::<SType>();
        let start = u64::from_le_slice(&metadata_buffer[pos..]);
        pos += U64_SIZE;
        let end = u64::from_le_slice(&metadata_buffer[pos..]);
        pos += U64_SIZE;
        let limit = u64::from_le_slice(&metadata_buffer[pos..]);
        pos += U64_SIZE;
        let encoding = Encoding::try_from(metadata_buffer[pos])
            .with_context(|| format!("Failed to parse encoding '{}'", metadata_buffer[pos]))?;
        pos += mem::size_of::<Encoding>();
        let compression = Compression::try_from(metadata_buffer[pos])
            .with_context(|| format!("Failed to parse compression '{}'", metadata_buffer[pos]))?;
        pos += mem::size_of::<Compression>();
        let nrows = u64::from_le_slice(&metadata_buffer[pos..]);
        pos += U64_SIZE;
        let ncols = u16::from_le_slice(&metadata_buffer[pos..]);
        let (_, var_buffer) = metadata_buffer.split_at(Self::FIXED_METADATA_LEN);
        let mut decoder = Decoder::new(var_buffer).with_context(|| {
            "Failed to create zstd decoder for variable-length portion of metadata"
        })?;
        // capacity should be informed by expected compression ratio
        let mut var_decompressed = Vec::with_capacity(var_buffer.len() * 3);
        decoder
            .read_to_end(&mut var_decompressed)
            .with_context(|| "Failed to decompress variable-length portion of metadata")?;
        let extra = serde_json::from_slice(&var_decompressed[..])
            .with_context(|| "Failed to parse variable-length JSON in metadata")?;

        Ok(Self {
            version,
            dataset,
            schema,
            stype_in,
            stype_out,
            start,
            end,
            limit,
            encoding,
            compression,
            nrows,
            ncols,
            extra,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db_def::tick::{Mbp10Msg, Mbp1Msg, OhlcvMsg, TbboMsg, TickMsg, TradeMsg};

    const DBZ_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../public/databento-python/tests/data"
    );

    /// there are crates like rstest that provide pytest-like parameterized tests, however
    /// they don't support passing types
    macro_rules! test_reading_dbz {
        // Rust doesn't allow concatenating identifiers in stable rust, so each test case needs
        // to be named explicitly
        ($test_name:ident, $tick_type:ident, $schema:expr, $file_name:expr) => {
            #[test]
            fn $test_name() {
                let target = Dbz::from_file(format!("{DBZ_PATH}/{}", $file_name)).unwrap();
                let exp_row_count = target.metadata().nrows;
                assert_eq!(target.schema(), $schema);
                let actual_row_count = target.try_into_iter::<$tick_type>().unwrap().count();
                assert_eq!(exp_row_count as usize, actual_row_count);
            }
        };
    }

    test_reading_dbz!(test_reading_mbo, TickMsg, Schema::Mbo, "test_data.mbo.dbz");
    test_reading_dbz!(
        test_reading_mbp1,
        Mbp1Msg,
        Schema::Mbp1,
        "test_data.mbp-1.dbz"
    );
    test_reading_dbz!(
        test_reading_mbp10,
        Mbp10Msg,
        Schema::Mbp10,
        "test_data.mbp-10.dbz"
    );
    test_reading_dbz!(
        test_reading_ohlcv1d,
        OhlcvMsg,
        Schema::Ohlcv1d,
        "test_data.ohlcv-1d.dbz"
    );
    test_reading_dbz!(
        test_reading_ohlcv1h,
        OhlcvMsg,
        Schema::Ohlcv1h,
        "test_data.ohlcv-1h.dbz"
    );
    test_reading_dbz!(
        test_reading_ohlcv1m,
        OhlcvMsg,
        Schema::Ohlcv1m,
        "test_data.ohlcv-1m.dbz"
    );
    test_reading_dbz!(
        test_reading_ohlcv1s,
        OhlcvMsg,
        Schema::Ohlcv1s,
        "test_data.ohlcv-1s.dbz"
    );
    test_reading_dbz!(
        test_reading_tbbo,
        TbboMsg,
        Schema::Tbbo,
        "test_data.tbbo.dbz"
    );
    test_reading_dbz!(
        test_reading_trades,
        TradeMsg,
        Schema::Trades,
        "test_data.trades.dbz"
    );
}
