use std::{
    io::{self, SeekFrom},
    mem,
    num::NonZeroU64,
};

use crate::{
    encode::{zstd_encoder, DbnEncodable, EncodeDbn, EncodeRecord, EncodeRecordRef},
    Error, Metadata, RecordRef, Result, Schema, SymbolMapping, DBN_VERSION, NULL_LIMIT,
    NULL_RECORD_COUNT, NULL_SCHEMA, NULL_STYPE, UNDEF_TIMESTAMP,
};

/// Type for encoding files and streams in Databento Binary Encoding (DBN).
pub struct Encoder<W>
where
    W: io::Write,
{
    record_encoder: RecordEncoder<W>,
}

impl<W> Encoder<W>
where
    W: io::Write,
{
    /// Creates a new DBN [`Encoder`] that will write to `writer`.
    ///
    /// # Errors
    /// This function will return an error if it fails to encode `metadata` to
    /// `writer`.
    pub fn new(mut writer: W, metadata: &Metadata) -> Result<Self> {
        MetadataEncoder::new(&mut writer).encode(metadata)?;
        let record_encoder = RecordEncoder::new(writer);
        Ok(Self { record_encoder })
    }

    /// Returns a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        self.record_encoder.get_ref()
    }

    /// Returns a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        self.record_encoder.get_mut()
    }
}

impl<W> Encoder<zstd::stream::AutoFinishEncoder<'_, W>>
where
    W: io::Write,
{
    /// Creates a new DBN [`Encoder`] that will write Zstd-compressed output to
    /// `writer`.
    ///
    /// # Errors
    /// This function will return an error if it fails to encode `metadata` to
    /// `writer`.
    pub fn with_zstd(writer: W, metadata: &Metadata) -> Result<Self> {
        Encoder::new(zstd_encoder(writer)?, metadata)
    }
}

impl<W> EncodeRecord for Encoder<W>
where
    W: io::Write,
{
    fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> Result<()> {
        self.record_encoder.encode_record(record)
    }

    fn flush(&mut self) -> Result<()> {
        self.record_encoder.flush()
    }
}

impl<W> EncodeRecordRef for Encoder<W>
where
    W: io::Write,
{
    fn encode_record_ref(&mut self, record: RecordRef) -> Result<()> {
        self.record_encoder.encode_record_ref(record)
    }

    /// Encodes a single DBN record.
    ///
    /// # Safety
    /// The DBN encoding a [`RecordRef`] is safe because no dispatching based on type
    /// is required.
    ///
    /// # Errors
    /// This function will return an error if it fails to encode `record` to
    /// `writer`.
    unsafe fn encode_record_ref_ts_out(&mut self, record: RecordRef, ts_out: bool) -> Result<()> {
        self.record_encoder.encode_record_ref_ts_out(record, ts_out)
    }
}

impl<W> EncodeDbn for Encoder<W> where W: io::Write {}

/// Type for encoding [`Metadata`] into Databento Binary Encoding (DBN).
pub struct MetadataEncoder<W>
where
    W: io::Write,
{
    writer: W,
}

impl<W> MetadataEncoder<W>
where
    W: io::Write,
{
    /// The minimum size in bytes of encoded metadata.
    pub const MIN_ENCODED_SIZE: usize = 128;
    /// The offset of `start` in encoded metadata.
    pub const START_OFFSET: usize =
        (8 + crate::METADATA_DATASET_CSTR_LEN + mem::size_of::<Schema>());

    /// Creates a new [`MetadataEncoder`] that will write to `writer`.
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Encodes `metadata` into DBN.
    ///
    /// # Errors
    /// This function returns an error if it fails to write to the underlying writer.
    pub fn encode(&mut self, metadata: &Metadata) -> Result<()> {
        let metadata_err = |e| Error::io(e, "writing DBN metadata");
        self.writer.write_all(b"DBN").map_err(metadata_err)?;
        self.writer
            // Never write version=0, which denotes legacy DBZ files or a version
            // greater than this version of the crate supports
            .write_all(&[metadata.version.clamp(1, DBN_VERSION)])
            .map_err(metadata_err)?;
        let (length, end_padding) = Self::calc_length(metadata);
        self.writer
            .write_all(length.to_le_bytes().as_slice())
            .map_err(metadata_err)?;
        self.encode_fixed_len_cstr(crate::METADATA_DATASET_CSTR_LEN, &metadata.dataset)?;
        self.writer
            .write_all(
                (metadata.schema.map(|s| s as u16).unwrap_or(NULL_SCHEMA))
                    .to_le_bytes()
                    .as_slice(),
            )
            .map_err(metadata_err)?;
        self.encode_range_and_counts(
            metadata.version,
            metadata.start,
            metadata.end,
            metadata.limit,
        )?;
        self.writer
            .write_all(&[
                metadata.stype_in.map(|s| s as u8).unwrap_or(NULL_STYPE),
                metadata.stype_out as u8,
                metadata.ts_out as u8,
            ])
            .map_err(metadata_err)?;
        if metadata.version > 1 {
            self.writer
                .write_all(&(metadata.symbol_cstr_len as u16).to_le_bytes())
                .map_err(metadata_err)?;
        }
        // padding
        self.writer
            .write_all(if metadata.version == 1 {
                &[0; crate::compat::METADATA_RESERVED_LEN_V1]
            } else {
                &[0; crate::METADATA_RESERVED_LEN]
            })
            .map_err(metadata_err)?;
        // schema_definition_length
        self.writer
            .write_all(0u32.to_le_bytes().as_slice())
            .map_err(metadata_err)?;

        self.encode_repeated_symbol_cstr(metadata.symbol_cstr_len, metadata.symbols.as_slice())?;
        self.encode_repeated_symbol_cstr(metadata.symbol_cstr_len, metadata.partial.as_slice())?;
        self.encode_repeated_symbol_cstr(metadata.symbol_cstr_len, metadata.not_found.as_slice())?;
        self.encode_symbol_mappings(metadata.symbol_cstr_len, metadata.mappings.as_slice())?;
        if end_padding > 0 {
            let padding = [0; 7];
            self.writer
                .write_all(&padding[..end_padding as usize])
                .map_err(metadata_err)?;
        }

        Ok(())
    }

    pub(super) fn calc_length(metadata: &Metadata) -> (u32, u32) {
        let mapping_interval_len = mem::size_of::<u32>() * 2 + metadata.symbol_cstr_len;
        // schema_definition_length, symbols_count, partial_count, not_found_count, mappings_count
        let var_len_counts_size = mem::size_of::<u32>() * 5;

        let c_str_count =
            metadata.symbols.len() + metadata.partial.len() + metadata.not_found.len();
        let needed_len = (crate::METADATA_FIXED_LEN
            + var_len_counts_size
            + c_str_count * metadata.symbol_cstr_len
            + metadata
                .mappings
                .iter()
                .map(|m| {
                    metadata.symbol_cstr_len
                        + mem::size_of::<u32>()
                        + m.intervals.len() * mapping_interval_len
                })
                .sum::<usize>()) as u32;
        let rem = needed_len % 8;
        if metadata.version < 3 || rem == 0 {
            (needed_len, 0)
        } else {
            let end_padding = 8 - rem;
            // round up size to keep 8-byte alignment
            (needed_len + end_padding, end_padding)
        }
    }

    fn encode_range_and_counts(
        &mut self,
        version: u8,
        start: u64,
        end: Option<NonZeroU64>,
        limit: Option<NonZeroU64>,
    ) -> Result<()> {
        let metadata_err = |e| Error::io(e, "writing DBN metadata");
        self.writer
            .write_all(start.to_le_bytes().as_slice())
            .map_err(metadata_err)?;
        self.writer
            .write_all(
                end.map(|e| e.get())
                    .unwrap_or(UNDEF_TIMESTAMP)
                    .to_le_bytes()
                    .as_slice(),
            )
            .map_err(metadata_err)?;
        self.writer
            .write_all(
                limit
                    .map(|l| l.get())
                    .unwrap_or(NULL_LIMIT)
                    .to_le_bytes()
                    .as_slice(),
            )
            .map_err(metadata_err)?;
        if version == 1 {
            // backwards compatibility for record_count
            self.writer
                .write_all(NULL_RECORD_COUNT.to_le_bytes().as_slice())
                .map_err(metadata_err)?;
        }
        Ok(())
    }

    fn encode_repeated_symbol_cstr(
        &mut self,
        symbol_cstr_len: usize,
        symbols: &[String],
    ) -> Result<()> {
        self.writer
            .write_all((symbols.len() as u32).to_le_bytes().as_slice())
            .map_err(|e| Error::io(e, "writing repeated symbols length"))?;
        for symbol in symbols {
            self.encode_fixed_len_cstr(symbol_cstr_len, symbol)?;
        }

        Ok(())
    }

    fn encode_symbol_mappings(
        &mut self,
        symbol_cstr_len: usize,
        symbol_mappings: &[SymbolMapping],
    ) -> Result<()> {
        // encode mappings_count
        self.writer
            .write_all((symbol_mappings.len() as u32).to_le_bytes().as_slice())
            .map_err(|e| Error::io(e, "writing symbol mappings length"))?;
        for symbol_mapping in symbol_mappings {
            self.encode_symbol_mapping(symbol_cstr_len, symbol_mapping)?;
        }
        Ok(())
    }

    fn encode_symbol_mapping(
        &mut self,
        symbol_cstr_len: usize,
        symbol_mapping: &SymbolMapping,
    ) -> Result<()> {
        self.encode_fixed_len_cstr(symbol_cstr_len, &symbol_mapping.raw_symbol)?;
        // encode interval_count
        self.writer
            .write_all(
                (symbol_mapping.intervals.len() as u32)
                    .to_le_bytes()
                    .as_slice(),
            )
            .map_err(|e| Error::io(e, "writing symbol mapping interval count"))?;
        for interval in symbol_mapping.intervals.iter() {
            self.encode_date(interval.start_date)
                .map_err(|e| Error::io(e, "writing start date"))?;
            self.encode_date(interval.end_date)
                .map_err(|e| Error::io(e, "writing end date"))?;
            self.encode_fixed_len_cstr(symbol_cstr_len, &interval.symbol)?;
        }
        Ok(())
    }

    fn encode_fixed_len_cstr(&mut self, symbol_cstr_len: usize, string: &str) -> Result<()> {
        if !string.is_ascii() {
            return Err(Error::Conversion {
                input: string.to_owned(),
                desired_type: "ASCII",
            });
        }
        if string.len() >= symbol_cstr_len {
            return Err(Error::encode(
            format!(
                "'{string}' is too long to be encoded in DBN; it cannot be longer than {} characters", symbol_cstr_len - 1
            )));
        }
        let cstr_err = |e| Error::io(e, "writing cstr");
        self.writer.write_all(string.as_bytes()).map_err(cstr_err)?;
        // pad remaining space with null bytes
        for _ in string.len()..symbol_cstr_len {
            self.writer.write_all(&[0]).map_err(cstr_err)?;
        }
        Ok(())
    }

    fn encode_date(&mut self, date: time::Date) -> io::Result<()> {
        let mut date_int = date.year() as u32 * 10_000;
        date_int += date.month() as u32 * 100;
        date_int += date.day() as u32;
        self.writer.write_all(date_int.to_le_bytes().as_slice())
    }
}

impl<W> MetadataEncoder<W>
where
    W: io::Write + io::Seek,
{
    /// Updates the given metadata properties in an existing DBN buffer.
    ///
    /// # Errors
    /// This function returns an error if it's unable to seek to the position
    /// to update the metadata or it fails to write to the underlying writer.
    pub fn update_encoded(
        &mut self,
        version: u8,
        start: u64,
        end: Option<NonZeroU64>,
        limit: Option<NonZeroU64>,
    ) -> Result<()> {
        /// Byte position of the field `start`
        const START_SEEK_FROM: SeekFrom =
            SeekFrom::Start(MetadataEncoder::<Vec<u8>>::START_OFFSET as u64);

        self.writer
            .seek(START_SEEK_FROM)
            .map_err(|e| Error::io(e, "seeking to write position"))?;
        self.encode_range_and_counts(version, start, end, limit)?;
        self.writer
            .seek(SeekFrom::End(0))
            .map_err(|e| Error::io(e, "seeking back to end"))?;
        Ok(())
    }
}

/// Type for encoding Databento Binary Encoding (DBN) records (not metadata).
pub struct RecordEncoder<W>
where
    W: io::Write,
{
    writer: W,
}

impl<W> RecordEncoder<W>
where
    W: io::Write,
{
    /// Creates a new DBN [`RecordEncoder`] that will write to `writer`.
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Returns a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        &self.writer
    }

    /// Returns a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.writer
    }
}

impl<W> EncodeRecord for RecordEncoder<W>
where
    W: io::Write,
{
    fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> Result<()> {
        match self.writer.write_all(record.as_ref()) {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::io(e, format!("serializing {record:?}"))),
        }
    }

    fn flush(&mut self) -> Result<()> {
        self.writer
            .flush()
            .map_err(|e| Error::io(e, "flushing output"))
    }
}

impl<W> EncodeRecordRef for RecordEncoder<W>
where
    W: io::Write,
{
    fn encode_record_ref(&mut self, record: RecordRef) -> Result<()> {
        match self.writer.write_all(record.as_ref()) {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::io(e, format!("serializing {record:?}"))),
        }
    }

    /// Encodes a single DBN record.
    ///
    /// # Safety
    /// The DBN encoding a [`RecordRef`] is safe because no dispatching based on type
    /// is required.
    ///
    /// # Errors
    /// This function will return an error if it fails to encode `record` to
    /// `writer`.
    unsafe fn encode_record_ref_ts_out(&mut self, record: RecordRef, _ts_out: bool) -> Result<()> {
        self.encode_record_ref(record)
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Seek, mem};

    use rstest::rstest;

    use super::*;
    use crate::{
        compat::version_symbol_cstr_len,
        decode::{dbn::MetadataDecoder, FromLittleEndianSlice},
        Dataset, MappingInterval, MetadataBuilder, SType, Schema, VersionUpgradePolicy,
    };

    #[test]
    fn test_encode_decode_metadata_identity() {
        let metadata = Metadata {
            version: crate::DBN_VERSION,
            dataset: Dataset::GlbxMdp3.to_string(),
            schema: Some(Schema::Mbp10),
            stype_in: Some(SType::RawSymbol),
            stype_out: SType::InstrumentId,
            start: 1657230820000000000,
            end: NonZeroU64::new(1658960170000000000),
            limit: None,
            ts_out: true,
            symbol_cstr_len: crate::SYMBOL_CSTR_LEN,
            symbols: vec!["ES".to_owned(), "NG".to_owned()],
            partial: vec!["ESM2".to_owned()],
            not_found: vec!["QQQQQ".to_owned()],
            mappings: vec![
                SymbolMapping {
                    raw_symbol: "ES.0".to_owned(),
                    intervals: vec![MappingInterval {
                        start_date: time::Date::from_calendar_date(2022, time::Month::July, 26)
                            .unwrap(),
                        end_date: time::Date::from_calendar_date(2022, time::Month::September, 1)
                            .unwrap(),
                        symbol: "ESU2".to_owned(),
                    }],
                },
                SymbolMapping {
                    raw_symbol: "NG.0".to_owned(),
                    intervals: vec![
                        MappingInterval {
                            start_date: time::Date::from_calendar_date(2022, time::Month::July, 26)
                                .unwrap(),
                            end_date: time::Date::from_calendar_date(2022, time::Month::August, 29)
                                .unwrap(),
                            symbol: "NGU2".to_owned(),
                        },
                        MappingInterval {
                            start_date: time::Date::from_calendar_date(
                                2022,
                                time::Month::August,
                                29,
                            )
                            .unwrap(),
                            end_date: time::Date::from_calendar_date(
                                2022,
                                time::Month::September,
                                1,
                            )
                            .unwrap(),
                            symbol: "NGV2".to_owned(),
                        },
                    ],
                },
            ],
        };
        let mut buffer = Vec::new();
        let mut target = MetadataEncoder::new(&mut buffer);
        target.encode(&metadata).unwrap();
        dbg!(&buffer);
        let res = MetadataDecoder::new(&mut buffer.as_slice())
            .decode()
            .unwrap();
        dbg!(&res, &metadata);
        assert_eq!(res, metadata);
    }

    #[test]
    fn test_encode_repeated_symbol_cstr() {
        let mut buffer = Vec::new();
        let mut target = MetadataEncoder::new(&mut buffer);
        let symbols = vec![
            "NG".to_owned(),
            "HP".to_owned(),
            "HPQ".to_owned(),
            "LNQ".to_owned(),
        ];
        target
            .encode_repeated_symbol_cstr(crate::SYMBOL_CSTR_LEN, symbols.as_slice())
            .unwrap();
        assert_eq!(
            buffer.len(),
            mem::size_of::<u32>() + symbols.len() * crate::SYMBOL_CSTR_LEN
        );
        assert_eq!(u32::from_le_slice(&buffer[..4]), 4);
        for (i, symbol) in symbols.iter().enumerate() {
            let offset = i * crate::SYMBOL_CSTR_LEN;
            assert_eq!(
                &buffer[4 + offset..4 + offset + symbol.len()],
                symbol.as_bytes()
            );
        }
    }

    #[test]
    fn test_encode_fixed_len_cstr() {
        let mut buffer = Vec::new();
        let mut target = MetadataEncoder::new(&mut buffer);
        target
            .encode_fixed_len_cstr(crate::SYMBOL_CSTR_LEN, "NG")
            .unwrap();
        assert_eq!(buffer.len(), crate::SYMBOL_CSTR_LEN);
        assert_eq!(&buffer[..2], b"NG");
        for b in buffer[2..].iter() {
            assert_eq!(*b, 0);
        }
    }

    #[test]
    fn test_encode_date() {
        let date = time::Date::from_calendar_date(2020, time::Month::May, 17).unwrap();
        let mut buffer = Vec::new();
        let mut target = MetadataEncoder::new(&mut buffer);
        target.encode_date(date).unwrap();
        assert_eq!(buffer.len(), mem::size_of::<u32>());
        assert_eq!(buffer.as_slice(), 20200517u32.to_le_bytes().as_slice());
    }

    #[rstest]
    fn test_update_encoded(#[values(1, 2, 3)] version: u8) {
        let orig_metadata = Metadata {
            version,
            dataset: Dataset::GlbxMdp3.to_string(),
            schema: Some(Schema::Mbo),
            stype_in: Some(SType::Parent),
            stype_out: SType::RawSymbol,
            start: 1657230820000000000,
            end: NonZeroU64::new(1658960170000000000),
            limit: None,
            ts_out: true,
            symbol_cstr_len: version_symbol_cstr_len(version),
            symbols: vec![],
            partial: vec![],
            not_found: vec![],
            mappings: vec![],
        };
        let mut buffer = Vec::new();
        let mut target = MetadataEncoder::new(&mut buffer);
        target.encode(&orig_metadata).unwrap();
        let orig_res = MetadataDecoder::with_upgrade_policy(
            &mut buffer.as_slice(),
            VersionUpgradePolicy::AsIs,
        )
        .decode()
        .unwrap();
        assert_eq!(orig_metadata, orig_res);
        let mut cursor = io::Cursor::new(&mut buffer);
        assert_eq!(cursor.position(), 0);
        cursor.seek(SeekFrom::End(0)).unwrap();
        let before_pos = cursor.position();
        assert!(before_pos != 0);
        let new_start = 1697240529000000000;
        let new_end = NonZeroU64::new(17058980170000000000);
        let new_limit = NonZeroU64::new(10);
        MetadataEncoder::new(&mut cursor)
            .update_encoded(crate::DBN_VERSION, new_start, new_end, new_limit)
            .unwrap();
        assert_eq!(before_pos, cursor.position());
        let res = MetadataDecoder::new(&mut buffer.as_slice())
            .decode()
            .unwrap();
        assert!(res != orig_res);
        assert_eq!(res.start, new_start);
        assert_eq!(res.end, new_end);
        assert_eq!(res.limit, new_limit);
    }

    #[rstest]
    fn test_encode_decode_nulls(#[values(1, 2, 3)] version: u8) {
        let metadata = MetadataBuilder::new()
            .version(version)
            .dataset(Dataset::XnasItch)
            .schema(Some(Schema::Mbo))
            .start(1697240529000000000)
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .build();
        assert!(metadata.end.is_none());
        assert!(metadata.limit.is_none());
        let mut buffer = Vec::new();
        MetadataEncoder::new(&mut buffer).encode(&metadata).unwrap();
        let decoded = MetadataDecoder::new(buffer.as_slice()).decode().unwrap();
        assert!(decoded.end.is_none());
        assert!(decoded.limit.is_none());
    }

    #[rstest]
    fn test_metadata_min_encoded_size(#[values(1, 2, 3)] version: u8) {
        let metadata = MetadataBuilder::new()
            .version(version)
            .dataset(Dataset::XnasItch)
            .schema(Some(Schema::Mbo))
            .start(1697240529000000000)
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .build();
        let (calc_length, end_padding) = MetadataEncoder::<Vec<u8>>::calc_length(&metadata);
        let mut buffer = Vec::new();
        let mut encoder = MetadataEncoder::new(&mut buffer);
        encoder.encode(&metadata).unwrap();
        // plus 8 for prefix
        assert_eq!(calc_length as usize + 8, buffer.len());
        assert_eq!(MetadataEncoder::<Vec<u8>>::MIN_ENCODED_SIZE, buffer.len());
        assert_eq!(end_padding, 0);
    }

    #[rstest]
    fn test_metadata_calc_size_unconventional_length(#[values(1, 2, 3)] version: u8) {
        let mut metadata = MetadataBuilder::new()
            .version(version)
            .dataset(Dataset::XnasItch)
            .schema(Some(Schema::Mbo))
            .start(1697240529000000000)
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .symbols(vec![
                "META".to_owned(),
                "NVDA".to_owned(),
                "NFLX".to_owned(),
            ])
            .build();
        metadata.symbol_cstr_len = 50;
        let (calc_length, end_padding) = MetadataEncoder::<Vec<u8>>::calc_length(&metadata);
        let mut buffer = Vec::new();
        let mut encoder = MetadataEncoder::new(&mut buffer);
        encoder.encode(&metadata).unwrap();
        // plus 8 for prefix
        assert_eq!(calc_length as usize + 8, buffer.len());
        if version < 3 {
            assert_eq!(end_padding, 0);
        } else {
            assert!((1..8).contains(&end_padding));
        }
    }
}
