//! Decoding of legacy DBZ files, a precursor to DBN.
use std::{
    fs::File,
    io::{self, BufReader, Read},
    mem,
    num::NonZeroU64,
    path::Path,
    str::Utf8Error,
};

use crate::{
    decode::{
        dbn::{
            decode_iso8601,
            fsm::{DbnFsm, ProcessResult},
        },
        private::LastRecord,
        zstd::ZSTD_SKIPPABLE_MAGIC_RANGE,
        DbnMetadata, DecodeRecord, DecodeRecordRef, DecodeStream, FromLittleEndianSlice,
        StreamIterDecoder, VersionUpgradePolicy,
    },
    Compression, HasRType, MappingInterval, Metadata, RecordRef, SType, Schema, SymbolMapping,
};

/// Object for reading, parsing, and serializing a legacy Databento Binary Encoding (DBZ) file.
pub struct Decoder<R: io::BufRead> {
    reader: zstd::Decoder<'static, R>,
    metadata: Metadata,
    fsm: DbnFsm,
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
    /// Creates a new DBZ [`Decoder`] from `reader`. Will upgrade records from previous
    /// versions to the current version.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in `reader`.
    pub fn new(reader: R) -> crate::Result<Self> {
        Self::with_upgrade_policy(reader, VersionUpgradePolicy::default())
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
        let fsm = DbnFsm::builder()
            // DBN version 1 records are the same as DBZ
            .input_dbn_version(Some(1))
            .unwrap() // 1 is a valid DBN version
            .ts_out(metadata.ts_out)
            .upgrade_policy(upgrade_policy)
            // decoded metadata separately
            .skip_metadata(true)
            .build()?;
        metadata.upgrade(upgrade_policy);
        let reader = zstd::Decoder::with_buffer(reader)
            .map_err(|e| crate::Error::io(e, "creating zstd decoder"))?;
        Ok(Self {
            reader,
            metadata,
            fsm,
        })
    }
}

impl<R: io::BufRead> DecodeRecordRef for Decoder<R> {
    fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef<'_>>> {
        loop {
            match self.fsm.process() {
                ProcessResult::ReadMore(_) => match self.reader.read(self.fsm.space()) {
                    Ok(0) => return Ok(None),
                    Ok(nbytes) => {
                        self.fsm.fill(nbytes);
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                        return Ok(None);
                    }
                    Err(err) => {
                        return Err(crate::Error::io(err, "decoding record reference"));
                    }
                },
                ProcessResult::Record(_) => return Ok(self.fsm.last_record()),
                ProcessResult::Err(error) => return Err(error),
                ProcessResult::Metadata(_) => unreachable!("skipped metadata"),
            }
        }
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
        self.decode_record_ref().and_then(|rec| {
            if let Some(rec) = rec {
                rec.try_get().map(Some)
            } else {
                Ok(None)
            }
        })
    }
}

impl<R: io::BufRead> DecodeStream for Decoder<R> {
    /// Try to decode the DBZ file into a streaming iterator. This decodes the
    /// data lazily.
    ///
    /// # Errors
    /// This function will return an error if the zstd portion of the DBZ file
    /// was compressed in an unexpected manner.
    fn decode_stream<T: HasRType>(self) -> super::StreamIterDecoder<Self, T>
    where
        Self: Sized,
    {
        StreamIterDecoder::new(self)
    }
}

impl<R: io::BufRead> LastRecord for Decoder<R> {
    fn last_record(&self) -> Option<RecordRef<'_>> {
        self.fsm.last_record()
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
    use fallible_streaming_iterator::FallibleStreamingIterator;
    use rstest::*;

    use super::*;
    use crate::compat::InstrumentDefMsgV1;
    use crate::decode::tests::TEST_DATA_PATH;
    use crate::record::{MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, TbboMsg, TradeMsg};

    #[rstest]
    #[case::mbo(MboMsg::default(), Schema::Mbo, 2)]
    #[case::mbp1(Mbp1Msg::default(), Schema::Mbp1, 2)]
    #[case::mbp10(Mbp10Msg::default(), Schema::Mbp10, 2)]
    #[case::ohlcv_1d(OhlcvMsg::default_for_schema(Schema::Ohlcv1D), Schema::Ohlcv1D, 0)]
    #[case::ohlcv_1h(OhlcvMsg::default_for_schema(Schema::Ohlcv1H), Schema::Ohlcv1H, 2)]
    #[case::ohlcv_1m(OhlcvMsg::default_for_schema(Schema::Ohlcv1M), Schema::Ohlcv1M, 2)]
    #[case::ohlcv_1s(OhlcvMsg::default_for_schema(Schema::Ohlcv1S), Schema::Ohlcv1S, 2)]
    #[case::tbbo(TbboMsg::default(), Schema::Tbbo, 2)]
    #[case::trades(TradeMsg::default(), Schema::Trades, 2)]
    #[case::definition(InstrumentDefMsgV1::default(), Schema::Definition, 2)]
    fn test_decode_stream<R: HasRType>(
        #[case] _rec: R,
        #[case] schema: Schema,
        #[case] exp_rec_count: usize,
    ) {
        let target =
            Decoder::from_file(format!("{TEST_DATA_PATH}/test_data.{schema}.dbz")).unwrap();
        let actual_rec_count = target.decode_stream::<R>().count().unwrap();
        assert_eq!(exp_rec_count, actual_rec_count);
    }

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
