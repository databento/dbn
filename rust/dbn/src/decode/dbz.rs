//! Decoding of legacy DBZ files, a precursor to DBN.
use std::{
    fs::File,
    io::{self, BufReader, Read},
    mem,
    num::NonZeroU64,
    path::Path,
    str::Utf8Error,
};

use super::{
    private::BufferSlice, zstd::ZSTD_SKIPPABLE_MAGIC_RANGE, DbnMetadata, DecodeRecord,
    DecodeRecordRef, DecodeStream, StreamIterDecoder, VersionUpgradePolicy,
};
use crate::{
    compat,
    decode::{dbn::decode_iso8601, FromLittleEndianSlice},
    error::silence_eof_error,
    Compression, HasRType, MappingInterval, Metadata, Record, RecordHeader, RecordRef, SType,
    Schema, SymbolMapping,
};

/// Object for reading, parsing, and serializing a legacy Databento Binary Encoding (DBZ) file.
pub struct Decoder<R: io::BufRead> {
    upgrade_policy: VersionUpgradePolicy,
    reader: zstd::Decoder<'static, R>,
    metadata: Metadata,
    read_buffer: Vec<u8>,
    compat_buffer: [u8; crate::MAX_RECORD_LEN],
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
    pub fn from_file(path: impl AsRef<Path>) -> crate::Result<Self> {
        let file = File::open(path.as_ref()).map_err(|e| {
            crate::Error::io(
                e,
                format!("opening dbn file at path '{}'", path.as_ref().display()),
            )
        })?;
        let reader = BufReader::new(file);
        Self::new(reader)
    }
}

// `BufRead` instead of `Read` because the [zstd::Decoder] works with `BufRead` so accepting
// a `Read` could result in redundant `BufReader`s being created.
impl<R: io::BufRead> Decoder<R> {
    /// Creates a new DBZ [`Decoder`] from `reader`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in `reader`.
    pub fn new(reader: R) -> crate::Result<Self> {
        Self::with_upgrade_policy(reader, VersionUpgradePolicy::AsIs)
    }

    /// Creates a new DBZ [`Decoder`] from `reader`. It will decode records from
    /// according to `upgrade_policy`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in
    /// `reader.
    pub fn with_upgrade_policy(
        mut reader: R,
        upgrade_policy: VersionUpgradePolicy,
    ) -> crate::Result<Self> {
        let mut metadata = MetadataDecoder::read(&mut reader)?;
        metadata.upgrade(upgrade_policy);
        let reader = zstd::Decoder::with_buffer(reader)
            .map_err(|e| crate::Error::io(e, "creating zstd decoder"))?;
        Ok(Self {
            upgrade_policy,
            reader,
            metadata,
            read_buffer: vec![0],
            compat_buffer: [0; crate::MAX_RECORD_LEN],
        })
    }
}

impl<R: io::BufRead> DecodeRecordRef for Decoder<R> {
    fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef>> {
        let io_err = |e| crate::Error::io(e, "decoding record reference");
        if let Err(err) = self.reader.read_exact(&mut self.read_buffer[..1]) {
            return silence_eof_error(err).map_err(io_err);
        }
        let length = self.read_buffer[0] as usize * RecordHeader::LENGTH_MULTIPLIER;
        if length < mem::size_of::<RecordHeader>() {
            return Err(crate::Error::decode(format!(
                "invalid record with length {length} shorter than header"
            )));
        }
        if length > self.read_buffer.len() {
            self.read_buffer.resize(length, 0);
        }
        if let Err(err) = self.reader.read_exact(&mut self.read_buffer[1..length]) {
            return silence_eof_error(err).map_err(io_err);
        }
        // Safety: `buffer` is resized to contain at least `length` bytes.
        Ok(Some(unsafe {
            // DBZ records are the same as DBN version 1
            compat::decode_record_ref(
                1,
                self.upgrade_policy,
                &mut self.compat_buffer,
                &self.read_buffer,
            )
        }))
    }
}

impl<R: io::BufRead> DbnMetadata for Decoder<R> {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }
}

impl<R: io::BufRead> DecodeRecord for Decoder<R> {
    fn decode_record<T: HasRType>(&mut self) -> crate::Result<Option<&T>> {
        let rec_ref = self.decode_record_ref()?;
        if let Some(rec_ref) = rec_ref {
            rec_ref
                .get::<T>()
                .ok_or_else(|| {
                    crate::Error::conversion::<T>(format!(
                        "record with rtype {:#04X}",
                        rec_ref.header().rtype
                    ))
                })
                .map(Some)
        } else {
            Ok(None)
        }
    }
}

impl<R: io::BufRead> DecodeStream for Decoder<R> {
    /// Try to decode the DBZ file into a streaming iterator. This decodes the
    /// data lazily.
    ///
    /// # Errors
    /// This function will return an error if the zstd portion of the DBZ file
    /// was compressed in an unexpected manner.
    fn decode_stream<T: HasRType>(mut self) -> super::StreamIterDecoder<Self, T>
    where
        Self: Sized,
    {
        self.read_buffer = vec![0; mem::size_of::<T>()];
        StreamIterDecoder::new(self)
    }
}

impl<R: io::BufRead> BufferSlice for Decoder<R> {
    fn buffer_slice(&self) -> &[u8] {
        self.read_buffer.as_slice()
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
    const DBZ_PREFIX: &'static [u8] = b"DBZ";

    pub(crate) fn read(reader: &mut impl io::Read) -> crate::Result<Metadata> {
        let mut prelude_buffer = [0u8; 2 * mem::size_of::<i32>()];
        reader
            .read_exact(&mut prelude_buffer)
            .map_err(|e| crate::Error::io(e, "reading metadata prelude"))?;
        let magic = u32::from_le_slice(&prelude_buffer[..4]);
        if !ZSTD_SKIPPABLE_MAGIC_RANGE.contains(&magic) {
            return Err(crate::Error::decode(
                "invalid metadata: no zstd magic number",
            ));
        }
        let frame_size = u32::from_le_slice(&prelude_buffer[4..]);
        // debug!("magic={magic}, frame_size={frame_size}");
        if (frame_size as usize) < Self::FIXED_METADATA_LEN {
            return Err(crate::Error::Decode(
                "frame length cannot be shorter than the fixed metadata size".to_owned(),
            ));
        }

        let mut metadata_buffer = vec![0u8; frame_size as usize];
        reader
            .read_exact(&mut metadata_buffer)
            .map_err(|e| crate::Error::io(e, "reading metadata"))?;
        Self::decode(metadata_buffer)
    }

    fn decode(metadata_buffer: Vec<u8>) -> crate::Result<Metadata> {
        const U64_SIZE: usize = mem::size_of::<u64>();
        let mut pos = 0;
        if &metadata_buffer[pos..pos + 3] != MetadataDecoder::DBZ_PREFIX {
            return Err(crate::Error::Decode("Invalid version string".to_owned()));
        }
        // Interpret 4th character as an u8, not a char to allow for 254 versions (0 omitted)
        let version = metadata_buffer[pos + 3];
        // assume not forwards compatible
        if version > Self::SCHEMA_VERSION {
            return Err(crate::Error::Decode(
                "Can't read newer version of DBZ".to_owned(),
            ));
        }
        pos += Self::VERSION_CSTR_LEN;
        let dataset =
            std::str::from_utf8(&metadata_buffer[pos..pos + crate::METADATA_DATASET_CSTR_LEN])
                .map_err(|e| crate::Error::utf8(e, "reading dataset from metadata"))?
                // remove null bytes
                .trim_end_matches('\0')
                .to_owned();
        pos += crate::METADATA_DATASET_CSTR_LEN;
        let schema =
            Schema::try_from(u16::from_le_slice(&metadata_buffer[pos..])).map_err(|_| {
                crate::Error::conversion::<Schema>(format!("{:?}", &metadata_buffer[pos..pos + 2]))
            })?;
        pos += mem::size_of::<Schema>();
        let start = u64::from_le_slice(&metadata_buffer[pos..]);
        pos += U64_SIZE;
        let end = u64::from_le_slice(&metadata_buffer[pos..]);
        pos += U64_SIZE;
        let limit = NonZeroU64::new(u64::from_le_slice(&metadata_buffer[pos..]));
        pos += U64_SIZE;
        // skip over deprecated record_count
        pos += U64_SIZE;
        // Unused in new Metadata
        let _compression = Compression::try_from(metadata_buffer[pos])
            .map_err(|_| crate::Error::conversion::<Compression>(metadata_buffer[pos]))?;
        pos += mem::size_of::<Compression>();
        let stype_in = SType::try_from(metadata_buffer[pos])
            .map_err(|_| crate::Error::conversion::<SType>(metadata_buffer[pos]))?;
        pos += mem::size_of::<SType>();
        let stype_out = SType::try_from(metadata_buffer[pos])
            .map_err(|_| crate::Error::conversion::<SType>(metadata_buffer[pos]))?;
        pos += mem::size_of::<SType>();
        // skip reserved
        pos += Self::RESERVED_LEN;
        // remaining metadata is compressed
        let mut zstd_decoder = zstd::Decoder::new(&metadata_buffer[pos..])
            .map_err(|e| crate::Error::io(e, "reading zstd-compressed variable-length metadata"))?;

        // decompressed variable-length metadata buffer
        let buffer_capacity = (metadata_buffer.len() - pos) * 3; // 3x is arbitrary
        let mut var_buffer = Vec::with_capacity(buffer_capacity);
        zstd_decoder
            .read_to_end(&mut var_buffer)
            .map_err(|e| crate::Error::io(e, "reading variable-length metadata"))?;
        pos = 0;
        let schema_definition_length = u32::from_le_slice(&var_buffer[pos..]);
        if schema_definition_length != 0 {
            return Err(crate::Error::decode(
                "DBZ doesn't support schema definitions",
            ));
        }
        pos += Self::U32_SIZE + (schema_definition_length as usize);
        let symbols = Self::decode_repeated_symbol_cstr(var_buffer.as_slice(), &mut pos)?;
        let partial = Self::decode_repeated_symbol_cstr(var_buffer.as_slice(), &mut pos)?;
        let not_found = Self::decode_repeated_symbol_cstr(var_buffer.as_slice(), &mut pos)?;
        let mappings = Self::decode_symbol_mappings(var_buffer.as_slice(), &mut pos)?;

        Ok(Metadata {
            version: 0,
            dataset,
            schema: Some(schema),
            stype_in: Some(stype_in),
            stype_out,
            start,
            end: NonZeroU64::new(end),
            limit,
            ts_out: false,
            symbols,
            partial,
            not_found,
            mappings,
            symbol_cstr_len: crate::compat::SYMBOL_CSTR_LEN_V1,
        })
    }

    fn decode_repeated_symbol_cstr(buffer: &[u8], pos: &mut usize) -> crate::Result<Vec<String>> {
        if *pos + Self::U32_SIZE > buffer.len() {
            return Err(crate::Error::decode("unexpected end of metadata buffer"));
        }
        let count = u32::from_le_slice(&buffer[*pos..]) as usize;
        *pos += Self::U32_SIZE;
        let read_size = count * crate::compat::SYMBOL_CSTR_LEN_V1;
        if *pos + read_size > buffer.len() {
            return Err(crate::Error::decode("unexpected end of metadata buffer"));
        }
        let mut res = Vec::with_capacity(count);
        for i in 0..count {
            res.push(
                Self::decode_symbol(buffer, pos)
                    .map_err(|e| crate::Error::utf8(e, format!("decoding symbol at index {i}")))?,
            );
        }
        Ok(res)
    }

    fn decode_symbol_mappings(buffer: &[u8], pos: &mut usize) -> crate::Result<Vec<SymbolMapping>> {
        if *pos + Self::U32_SIZE > buffer.len() {
            return Err(crate::Error::decode(
                "unexpected end of metadata buffer while decoding symbol mappings",
            ));
        }
        let count = u32::from_le_slice(&buffer[*pos..]) as usize;
        *pos += Self::U32_SIZE;
        let mut res = Vec::with_capacity(count);
        // Because each `SymbolMapping` itself is of a variable length, decoding it requires frequent bounds checks
        for _ in 0..count {
            res.push(Self::decode_symbol_mapping(buffer, pos)?);
        }
        Ok(res)
    }

    fn decode_symbol_mapping(buffer: &[u8], pos: &mut usize) -> crate::Result<SymbolMapping> {
        const U32_SIZE: usize = mem::size_of::<u32>();
        const MIN_SYMBOL_MAPPING_ENCODED_SIZE: usize = crate::compat::SYMBOL_CSTR_LEN_V1 + U32_SIZE;
        const MAPPING_INTERVAL_ENCODED_SIZE: usize =
            U32_SIZE * 2 + crate::compat::SYMBOL_CSTR_LEN_V1;

        if *pos + MIN_SYMBOL_MAPPING_ENCODED_SIZE > buffer.len() {
            return Err(crate::Error::decode(
                "unexpected end of metadata buffer while parsing symbol mapping",
            ));
        }
        let raw_symbol = Self::decode_symbol(buffer, pos)
            .map_err(|e| crate::Error::utf8(e, "parsing raw symbol"))?;
        let interval_count = u32::from_le_slice(&buffer[*pos..]) as usize;
        *pos += Self::U32_SIZE;
        let read_size = interval_count * MAPPING_INTERVAL_ENCODED_SIZE;
        if *pos + read_size > buffer.len() {
            return Err(crate::Error::decode(format!(
                "symbol mapping interval_count ({interval_count}) doesn't match size of buffer \
                which only contains space for {} intervals",
                (buffer.len() - *pos) / MAPPING_INTERVAL_ENCODED_SIZE
            )));
        }
        let mut intervals = Vec::with_capacity(interval_count);
        for i in 0..interval_count {
            let raw_start_date = u32::from_le_slice(&buffer[*pos..]);
            *pos += U32_SIZE;
            let start_date = decode_iso8601(raw_start_date).map_err(|e| {
                crate::Error::decode(format!(
                    "{e} while parsing start date of mapping interval at index {i}"
                ))
            })?;
            let raw_end_date = u32::from_le_slice(&buffer[*pos..]);
            *pos += U32_SIZE;
            let end_date = decode_iso8601(raw_end_date).map_err(|e| {
                crate::Error::decode(format!(
                    "{e} while parsing start date of mapping interval at index {i}"
                ))
            })?;
            let symbol = Self::decode_symbol(buffer, pos).map_err(|e| {
                crate::Error::utf8(e, format!("parsing symbol mapping interval at index {i}"))
            })?;
            intervals.push(MappingInterval {
                start_date,
                end_date,
                symbol,
            });
        }
        Ok(SymbolMapping {
            raw_symbol,
            intervals,
        })
    }

    fn decode_symbol(buffer: &[u8], pos: &mut usize) -> Result<String, Utf8Error> {
        let symbol_slice = &buffer[*pos..*pos + crate::compat::SYMBOL_CSTR_LEN_V1];
        let symbol = std::str::from_utf8(symbol_slice)?
            // remove null bytes
            .trim_end_matches('\0')
            .to_owned();
        *pos += crate::compat::SYMBOL_CSTR_LEN_V1;
        Ok(symbol)
    }
}

#[cfg(test)]
mod tests {
    use streaming_iterator::StreamingIterator;

    use super::*;
    use crate::compat::InstrumentDefMsgV1;
    use crate::decode::tests::TEST_DATA_PATH;
    use crate::record::{MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, TbboMsg, TradeMsg};

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
                let exp_rec_count = if $schema == Schema::Ohlcv1D { 0 } else { 2 };
                let actual_rec_count = target.decode_stream::<$record_type>().count();
                assert_eq!(exp_rec_count, actual_rec_count);
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
        InstrumentDefMsgV1,
        Schema::Definition
    );

    #[test]
    fn test_decode_symbol() {
        let bytes = b"SPX.1.2\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
        assert_eq!(bytes.len(), crate::compat::SYMBOL_CSTR_LEN_V1);
        let mut pos = 0;
        let res = MetadataDecoder::decode_symbol(bytes.as_slice(), &mut pos).unwrap();
        assert_eq!(pos, crate::compat::SYMBOL_CSTR_LEN_V1);
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
        assert!(res.is_err());
    }
}
