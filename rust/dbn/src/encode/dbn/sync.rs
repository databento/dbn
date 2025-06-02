use std::{
    io::{self, IoSlice, SeekFrom},
    mem::{self, transmute, MaybeUninit},
    num::NonZeroU64,
};

use crate::{
    encode::{
        io_utils::write_all_vectored, zstd_encoder, DbnEncodable, EncodeDbn, EncodeRecord,
        EncodeRecordRef,
    },
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

    fn encode_records<R: DbnEncodable>(&mut self, records: &[R]) -> Result<()> {
        self.record_encoder.encode_records(records)
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

    fn encode_record_refs(&mut self, records: &[RecordRef]) -> Result<()> {
        self.record_encoder.encode_record_refs(records)
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
    /// This function returns an error if it fails to write to the underlying writer or
    /// `metadata` is from a newer DBN version than is supported.
    pub fn encode(&mut self, metadata: &Metadata) -> Result<()> {
        let metadata_err = |e| Error::io(e, "writing DBN metadata");
        self.writer.write_all(b"DBN").map_err(metadata_err)?;
        if metadata.version > DBN_VERSION {
            return Err(Error::encode(format!("can't encode Metadata with version {} which is greater than the maximum supported version {DBN_VERSION}", metadata.version)));
        }
        self.writer
            // Never write version=0, which denotes legacy DBZ files
            .write_all(&[metadata.version.max(1)])
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
        self.writer
            .write_all(record.as_ref())
            .map_err(|e| Error::io(e, format!("serializing {record:?}")))
    }

    fn encode_records<R: DbnEncodable>(&mut self, records: &[R]) -> Result<()> {
        // SAFETY: DBN records have no implicit padding and are POD structs
        let slice = unsafe {
            std::slice::from_raw_parts::<u8>(records.as_ptr() as *const u8, size_of_val(records))
        };
        self.writer
            .write_all(slice)
            .map_err(|e| Error::io(e, format!("serializing {} records", records.len())))
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
        self.writer
            .write_all(record.as_ref())
            .map_err(|e| Error::io(e, format!("serializing {record:?}")))
    }

    fn encode_record_refs(&mut self, records: &[RecordRef]) -> Result<()> {
        const BATCH_SIZE: usize = 128;
        let mut slices = [const { MaybeUninit::uninit() }; BATCH_SIZE];
        for record_chunk in records.chunks(BATCH_SIZE) {
            for (elem, rec) in slices.iter_mut().zip(record_chunk.iter()) {
                elem.write(IoSlice::from(*rec));
            }
            let slices =
                // SAFETY: Every element up to `record_chunk.len()` has been initialized
                unsafe { transmute::<&mut [MaybeUninit<IoSlice<'_>>], &mut [IoSlice<'_>]>(&mut slices[..record_chunk.len()]) };
            write_all_vectored(&mut self.writer, slices).map_err(|e| {
                Error::io(e, format!("failed to encode {} RecordRefs", records.len()))
            })?;
        }
        Ok(())
    }

    /// Encodes a single DBN record.
    ///
    /// # Safety
    /// DBN encoding a [`RecordRef`] is safe because no dispatching based on type
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

    #[rstest]
    fn test_fails_to_encode_newer_metadata() {
        let metadata = MetadataBuilder::new()
            .dataset(Dataset::XeeeEobi.to_string())
            .schema(Some(Schema::Definition))
            .start(0)
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .version(DBN_VERSION + 1)
            .build();
        let mut buffer = Vec::new();
        let mut target = MetadataEncoder::new(&mut buffer);
        assert!(
            matches!(target.encode(&metadata), Err(Error::Encode(msg)) if msg.contains("can't encode Metadata with version"))
        );
    }

    mod batch {
        use rstest::rstest;

        use crate::{
            decode::{DecodeRecord, DecodeRecordRef},
            encode::{dbn::RecordEncoder, EncodeRecord, EncodeRecordRef},
            record::{MboMsg, RecordHeader, TradeMsg},
            rtype, FlagSet, Record, RecordRef,
        };

        fn make_mbo_msg(instrument_id: u32, ts_event: u64) -> MboMsg {
            MboMsg {
                hd: RecordHeader::new::<MboMsg>(rtype::MBO, 1, instrument_id, ts_event),
                order_id: 123456,
                price: 100_000_000_000,
                size: 10,
                flags: FlagSet::default(),
                channel_id: 0,
                action: b'A' as i8,
                side: b'B' as i8,
                ts_recv: ts_event + 1000,
                ts_in_delta: 500,
                sequence: 1,
            }
        }

        fn make_trade_msg(instrument_id: u32, ts_event: u64) -> TradeMsg {
            TradeMsg {
                hd: RecordHeader::new::<TradeMsg>(rtype::MBP_0, 1, instrument_id, ts_event),
                price: 100_000_000_000,
                size: 5,
                action: b'T' as i8,
                side: b'A' as i8,
                flags: FlagSet::default(),
                depth: 0,
                ts_recv: ts_event + 1000,
                ts_in_delta: 500,
                sequence: 1,
            }
        }

        #[test]
        fn test_encode_records_typed_roundtrip() {
            // Create test records
            let records: Vec<MboMsg> = (0..10)
                .map(|i| make_mbo_msg(100 + i, 1658441851000000000 + i as u64 * 1000))
                .collect();

            // Encode using batch method
            let mut buffer = Vec::new();
            let mut encoder = RecordEncoder::new(&mut buffer);
            encoder.encode_records(&records).unwrap();

            // Verify buffer size matches expected
            assert_eq!(buffer.len(), records.len() * std::mem::size_of::<MboMsg>());

            // Decode and verify roundtrip
            let mut decoder = crate::decode::dbn::RecordDecoder::new(&buffer[..]);
            for (i, original) in records.iter().enumerate() {
                let decoded: Option<&MboMsg> = decoder.decode_record().unwrap();
                assert!(decoded.is_some(), "Failed to decode record {i}");
                assert_eq!(decoded.unwrap(), original, "Record {i} mismatch");
            }
            // Verify no more records
            let extra: Option<&MboMsg> = decoder.decode_record().unwrap();
            assert!(extra.is_none());
        }

        #[test]
        fn test_encode_records_single_is_equivalent() {
            let records: Vec<MboMsg> = (0..5)
                .map(|i| make_mbo_msg(100 + i, 1658441851000000000 + i as u64 * 1000))
                .collect();

            // Encode one at a time
            let mut buffer_single = Vec::new();
            let mut encoder_single = RecordEncoder::new(&mut buffer_single);
            for record in &records {
                encoder_single.encode_record(record).unwrap();
            }

            // Encode as batch
            let mut buffer_batch = Vec::new();
            let mut encoder_batch = RecordEncoder::new(&mut buffer_batch);
            encoder_batch.encode_records(&records).unwrap();

            assert_eq!(buffer_single, buffer_batch);
        }

        #[test]
        fn test_encode_records_empty_slice() {
            let records: Vec<MboMsg> = vec![];

            let mut buffer = Vec::new();
            let mut encoder = RecordEncoder::new(&mut buffer);
            encoder.encode_records(&records).unwrap();

            assert!(buffer.is_empty());
        }

        #[test]
        fn test_encode_record_refs_roundtrip() {
            // Create mixed record types
            let mbo1 = make_mbo_msg(100, 1658441851000000000);
            let trade1 = make_trade_msg(101, 1658441851001000000);
            let mbo2 = make_mbo_msg(102, 1658441851002000000);
            let trade2 = make_trade_msg(103, 1658441851003000000);

            let refs: Vec<RecordRef> = vec![
                RecordRef::from(&mbo1),
                RecordRef::from(&trade1),
                RecordRef::from(&mbo2),
                RecordRef::from(&trade2),
            ];

            // Encode using batch method
            let mut buffer = Vec::new();
            let mut encoder = RecordEncoder::new(&mut buffer);
            encoder.encode_record_refs(&refs).unwrap();

            // Verify sizes
            let expected_size =
                std::mem::size_of::<MboMsg>() * 2 + std::mem::size_of::<TradeMsg>() * 2;
            assert_eq!(buffer.len(), expected_size);

            // Decode and verify roundtrip
            let mut decoder = crate::decode::dbn::RecordDecoder::new(&buffer[..]);

            let decoded = decoder.decode_record_ref().unwrap().unwrap();
            assert_eq!(decoded.header(), mbo1.header());

            let decoded = decoder.decode_record_ref().unwrap().unwrap();
            assert_eq!(decoded.header(), trade1.header());

            let decoded = decoder.decode_record_ref().unwrap().unwrap();
            assert_eq!(decoded.header(), mbo2.header());

            let decoded = decoder.decode_record_ref().unwrap().unwrap();
            assert_eq!(decoded.header(), trade2.header());

            assert!(decoder.decode_record_ref().unwrap().is_none());
        }

        #[test]
        fn test_encode_record_refs_single_is_equivalent() {
            let mbo = make_mbo_msg(100, 1658441851000000000);
            let trade = make_trade_msg(101, 1658441851001000000);

            let refs: Vec<RecordRef> = vec![RecordRef::from(&mbo), RecordRef::from(&trade)];

            // Encode one at a time
            let mut buffer_single = Vec::new();
            let mut encoder_single = RecordEncoder::new(&mut buffer_single);
            for record_ref in &refs {
                encoder_single.encode_record_ref(*record_ref).unwrap();
            }

            // Encode as batch
            let mut buffer_batch = Vec::new();
            let mut encoder_batch = RecordEncoder::new(&mut buffer_batch);
            encoder_batch.encode_record_refs(&refs).unwrap();

            assert_eq!(buffer_single, buffer_batch);
        }

        #[test]
        fn test_encode_record_refs_empty_slice() {
            let refs: Vec<RecordRef> = vec![];

            let mut buffer = Vec::new();
            let mut encoder = RecordEncoder::new(&mut buffer);
            encoder.encode_record_refs(&refs).unwrap();

            assert!(buffer.is_empty());
        }

        #[rstest]
        #[case::partial_batch(127)]
        #[case::exact_batch(128)]
        #[case::batch_plus_one(129)]
        #[case::multiple_batches(200)]
        fn test_encode_record_refs_batch_sizes(#[case] count: usize) {
            let records: Vec<MboMsg> = (0..count)
                .map(|i| make_mbo_msg(100 + i as u32, 1658441851000000000 + i as u64 * 1000))
                .collect();

            let refs: Vec<RecordRef> = records.iter().map(RecordRef::from).collect();

            let mut buffer = Vec::new();
            let mut encoder = RecordEncoder::new(&mut buffer);
            encoder.encode_record_refs(&refs).unwrap();

            // Verify buffer size
            assert_eq!(buffer.len(), records.len() * std::mem::size_of::<MboMsg>());

            // Decode and count records
            let mut decoder = crate::decode::dbn::RecordDecoder::new(&buffer[..]);
            let mut decoded_count = 0;
            while decoder.decode_record_ref().unwrap().is_some() {
                decoded_count += 1;
            }
            assert_eq!(decoded_count, count);
        }
    }
}
