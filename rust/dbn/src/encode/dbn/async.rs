use std::num::NonZeroU64;

use async_compression::tokio::write::ZstdEncoder;
use tokio::io;

use crate::{
    encode::{async_zstd_encoder, AsyncEncodeRecord, AsyncEncodeRecordRef, DbnEncodable},
    record_ref::RecordRef,
    Error, Metadata, Result, SymbolMapping, DBN_VERSION, NULL_LIMIT, NULL_RECORD_COUNT,
    NULL_SCHEMA, NULL_STYPE, UNDEF_TIMESTAMP,
};

/// An async encoder for DBN streams.
pub struct Encoder<W>
where
    W: io::AsyncWriteExt + Unpin,
{
    record_encoder: RecordEncoder<W>,
}

impl<W> Encoder<W>
where
    W: io::AsyncWriteExt + Unpin,
{
    /// Creates a new async DBN encoder that will write to `writer`.
    ///
    /// # Errors
    /// This function will return an error if it fails to encode `metadata` to
    /// `writer`.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, then the
    /// metadata may have been partially written, but future calls will begin writing
    /// the encoded metadata from the beginning.
    pub async fn new(mut writer: W, metadata: &Metadata) -> Result<Self> {
        MetadataEncoder::new(&mut writer).encode(metadata).await?;
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

impl<W> Encoder<ZstdEncoder<W>>
where
    W: io::AsyncWriteExt + Unpin,
{
    /// Creates a new async [`Encoder`] that will Zstandard compress the DBN data
    /// written to `writer`.
    ///
    /// # Errors
    /// This function will return an error if it fails to encode `metadata` to
    /// `writer`.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, then the
    /// metadata may have been partially written, but future calls will begin writing
    /// the encoded metadata from the beginning.
    pub async fn with_zstd(writer: W, metadata: &Metadata) -> Result<Self> {
        Self::new(async_zstd_encoder(writer), metadata).await
    }
}

impl<W> AsyncEncodeRecord for Encoder<W>
where
    W: io::AsyncWriteExt + Unpin,
{
    async fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> Result<()> {
        self.record_encoder.encode_record(record).await
    }

    async fn flush(&mut self) -> Result<()> {
        self.record_encoder.flush().await
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.record_encoder.shutdown().await
    }
}

impl<W> AsyncEncodeRecordRef for Encoder<W>
where
    W: io::AsyncWriteExt + Unpin,
{
    async fn encode_record_ref(&mut self, record_ref: RecordRef<'_>) -> Result<()> {
        self.record_encoder.encode_record_ref(record_ref).await
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
    async unsafe fn encode_record_ref_ts_out(
        &mut self,
        record_ref: RecordRef<'_>,
        _ts_out: bool,
    ) -> Result<()> {
        self.record_encoder.encode_ref(record_ref).await
    }
}

/// An async encoder of DBN records.
pub struct RecordEncoder<W>
where
    W: io::AsyncWriteExt + Unpin,
{
    writer: W,
}

impl<W> RecordEncoder<W>
where
    W: io::AsyncWriteExt + Unpin,
{
    /// Creates a new instance of [`RecordEncoder`] that will forward its output to
    /// `writer`.
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Encode a single DBN record of type `R`.
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying
    /// writer.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, then the
    /// record may have been partially written, but future calls will begin writing the
    /// encoded record from the beginning.
    pub async fn encode<R: DbnEncodable>(&mut self, record: &R) -> Result<()> {
        match self.writer.write_all(record.as_ref()).await {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::io(e, format!("serializing {record:?}"))),
        }
    }

    /// Encodes a single DBN record of type `R`.
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying writer.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, then the
    /// record may have been partially written, but future calls will begin writing the
    /// encoded record from the beginning.
    pub async fn encode_ref(&mut self, record_ref: RecordRef<'_>) -> Result<()> {
        match self.writer.write_all(record_ref.as_ref()).await {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::io(e, format!("serializing {record_ref:?}"))),
        }
    }

    /// Returns a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        &self.writer
    }

    /// Returns a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Consumes the encoder returning the original writer.
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<W> AsyncEncodeRecord for RecordEncoder<W>
where
    W: io::AsyncWriteExt + Unpin,
{
    async fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> Result<()> {
        self.encode(record).await
    }

    async fn flush(&mut self) -> Result<()> {
        self.writer
            .flush()
            .await
            .map_err(|e| Error::io(e, "flushing output"))
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.writer
            .shutdown()
            .await
            .map_err(|e| Error::io(e, "shutting down"))
    }
}

impl<W> AsyncEncodeRecordRef for RecordEncoder<W>
where
    W: io::AsyncWriteExt + Unpin,
{
    async fn encode_record_ref(&mut self, record_ref: RecordRef<'_>) -> Result<()> {
        self.encode_ref(record_ref).await
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
    async unsafe fn encode_record_ref_ts_out(
        &mut self,
        record_ref: RecordRef<'_>,
        _ts_out: bool,
    ) -> Result<()> {
        self.encode_ref(record_ref).await
    }
}

impl<W> From<MetadataEncoder<W>> for RecordEncoder<W>
where
    W: io::AsyncWriteExt + Unpin,
{
    fn from(meta_encoder: MetadataEncoder<W>) -> Self {
        Self::new(meta_encoder.into_inner())
    }
}

/// An async DBN [`Metadata`] encoder.
pub struct MetadataEncoder<W>
where
    W: io::AsyncWriteExt + Unpin,
{
    writer: W,
}

impl<W> MetadataEncoder<W>
where
    W: io::AsyncWriteExt + Unpin,
{
    /// Creates a new [`MetadataEncoder`] that will write to `writer`.
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Encodes `metadata` into DBN.
    ///
    /// # Errors
    /// This function returns an error if it fails to write to the underlying writer or
    /// `metadata` is from a newer DBN version than is supported.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, then the
    /// metadata may have been partially written, but future calls will begin writing
    /// the encoded metadata from the beginning.
    pub async fn encode(&mut self, metadata: &Metadata) -> Result<()> {
        let metadata_err = |e| Error::io(e, "writing DBN metadata");
        self.writer.write_all(b"DBN").await.map_err(metadata_err)?;
        if metadata.version > DBN_VERSION {
            return Err(Error::encode(format!("can't encode Metadata with version {} which is greater than the maximum supported version {DBN_VERSION}", metadata.version)));
        }
        self.writer
            // Never write version=0, which denotes legacy DBZ files
            .write_all(&[metadata.version.max(1)])
            .await
            .map_err(metadata_err)?;
        let (length, end_padding) = super::MetadataEncoder::<std::fs::File>::calc_length(metadata);
        self.writer
            .write_u32_le(length)
            .await
            .map_err(metadata_err)?;
        self.encode_fixed_len_cstr(crate::METADATA_DATASET_CSTR_LEN, &metadata.dataset)
            .await?;
        self.writer
            .write_u16_le(metadata.schema.map(|s| s as u16).unwrap_or(NULL_SCHEMA))
            .await
            .map_err(metadata_err)?;
        self.encode_range_and_counts(
            metadata.version,
            metadata.start,
            metadata.end,
            metadata.limit,
        )
        .await?;
        self.writer
            .write_u8(metadata.stype_in.map(|s| s as u8).unwrap_or(NULL_STYPE))
            .await
            .map_err(metadata_err)?;
        self.writer
            .write_u8(metadata.stype_out as u8)
            .await
            .map_err(metadata_err)?;
        self.writer
            .write_u8(metadata.ts_out as u8)
            .await
            .map_err(metadata_err)?;
        if metadata.version > 1 {
            self.writer
                .write_u16_le(metadata.symbol_cstr_len as u16)
                .await
                .map_err(metadata_err)?;
        }
        // padding
        self.writer
            .write_all(if metadata.version == 1 {
                &[0; crate::compat::METADATA_RESERVED_LEN_V1]
            } else {
                &[0; crate::METADATA_RESERVED_LEN]
            })
            .await
            .map_err(metadata_err)?;
        // schema_definition_length
        self.writer.write_u32_le(0).await.map_err(metadata_err)?;
        self.encode_repeated_symbol_cstr(metadata.symbol_cstr_len, &metadata.symbols)
            .await?;
        self.encode_repeated_symbol_cstr(metadata.symbol_cstr_len, &metadata.partial)
            .await?;
        self.encode_repeated_symbol_cstr(metadata.symbol_cstr_len, &metadata.not_found)
            .await?;
        self.encode_symbol_mappings(metadata.symbol_cstr_len, &metadata.mappings)
            .await?;
        if end_padding > 0 {
            let padding = [0; 7];
            self.writer
                .write_all(&padding[..end_padding as usize])
                .await
                .map_err(metadata_err)?;
        }

        Ok(())
    }

    /// Returns a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        &self.writer
    }

    /// Returns a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Flushes any buffered content to the true output.
    ///
    /// # Errors
    /// This function returns an error if it's unable to flush the underlying writer.
    pub async fn flush(&mut self) -> Result<()> {
        self.writer
            .flush()
            .await
            .map_err(|e| Error::io(e, "flushing output".to_owned()))
    }

    /// Initiates or attempts to shut down the inner writer.
    ///
    /// # Errors
    /// This function returns an error if the shut down did not complete successfully.
    pub async fn shutdown(mut self) -> Result<()> {
        self.writer
            .shutdown()
            .await
            .map_err(|e| Error::io(e, "shutting down".to_owned()))
    }

    /// Consumes the encoder returning the original writer.
    pub fn into_inner(self) -> W {
        self.writer
    }

    async fn encode_range_and_counts(
        &mut self,
        version: u8,
        start: u64,
        end: Option<NonZeroU64>,
        limit: Option<NonZeroU64>,
    ) -> Result<()> {
        let metadata_err = |e| Error::io(e, "writing DBN metadata");
        self.writer
            .write_u64_le(start)
            .await
            .map_err(metadata_err)?;
        self.writer
            .write_u64_le(end.map(|e| e.get()).unwrap_or(UNDEF_TIMESTAMP))
            .await
            .map_err(metadata_err)?;
        self.writer
            .write_u64_le(limit.map(|l| l.get()).unwrap_or(NULL_LIMIT))
            .await
            .map_err(metadata_err)?;
        if version == 1 {
            // Backwards compatibility with removed metadata field `record_count`
            self.writer
                .write_u64_le(NULL_RECORD_COUNT)
                .await
                .map_err(metadata_err)?;
        }
        Ok(())
    }

    async fn encode_repeated_symbol_cstr(
        &mut self,
        symbol_cstr_len: usize,
        symbols: &[String],
    ) -> Result<()> {
        self.writer
            .write_u32_le(symbols.len() as u32)
            .await
            .map_err(|e| Error::io(e, "writing repeated symbols length"))?;
        for symbol in symbols {
            self.encode_fixed_len_cstr(symbol_cstr_len, symbol).await?;
        }

        Ok(())
    }

    async fn encode_symbol_mappings(
        &mut self,
        symbol_cstr_len: usize,
        symbol_mappings: &[SymbolMapping],
    ) -> Result<()> {
        // encode mappings_count
        self.writer
            .write_u32_le(symbol_mappings.len() as u32)
            .await
            .map_err(|e| Error::io(e, "writing symbol mappings length"))?;
        for symbol_mapping in symbol_mappings {
            self.encode_symbol_mapping(symbol_cstr_len, symbol_mapping)
                .await?;
        }
        Ok(())
    }

    async fn encode_symbol_mapping(
        &mut self,
        symbol_cstr_len: usize,
        symbol_mapping: &SymbolMapping,
    ) -> Result<()> {
        self.encode_fixed_len_cstr(symbol_cstr_len, &symbol_mapping.raw_symbol)
            .await?;
        // encode interval_count
        self.writer
            .write_u32_le(symbol_mapping.intervals.len() as u32)
            .await
            .map_err(|e| Error::io(e, "writing symbol mapping interval count"))?;
        for interval in symbol_mapping.intervals.iter() {
            self.encode_date(interval.start_date)
                .await
                .map_err(|e| Error::io(e, "writing start date"))?;
            self.encode_date(interval.end_date)
                .await
                .map_err(|e| Error::io(e, "writing end date"))?;
            self.encode_fixed_len_cstr(symbol_cstr_len, &interval.symbol)
                .await?;
        }
        Ok(())
    }

    async fn encode_fixed_len_cstr(&mut self, symbol_cstr_len: usize, string: &str) -> Result<()> {
        if !string.is_ascii() {
            return Err(Error::Conversion {
                input: string.to_owned(),
                desired_type: "ASCII",
            });
        }
        if string.len() >= symbol_cstr_len {
            return Err(Error::encode(format!(
                                "'{string}' is too long to be encoded in DBN; it cannot be longer than {} characters",
            symbol_cstr_len - 1
                            )));
        }
        let cstr_err = |e| Error::io(e, "writing cstr");
        self.writer
            .write_all(string.as_bytes())
            .await
            .map_err(cstr_err)?;
        // pad remaining space with null bytes
        for _ in string.len()..symbol_cstr_len {
            self.writer.write_u8(0).await.map_err(cstr_err)?;
        }
        Ok(())
    }

    async fn encode_date(&mut self, date: time::Date) -> io::Result<()> {
        let mut date_int = date.year() as u32 * 10_000;
        date_int += date.month() as u32 * 100;
        date_int += date.day() as u32;
        self.writer.write_u32_le(date_int).await?;
        Ok(())
    }
}

impl<W> MetadataEncoder<ZstdEncoder<W>>
where
    W: io::AsyncWriteExt + Unpin,
{
    /// Creates a new [`MetadataEncoder`] that will Zstandard compress the DBN data
    /// written to `writer`.
    pub fn with_zstd(writer: W) -> Self {
        Self::new(async_zstd_encoder(writer))
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;
    use crate::{
        compat::version_symbol_cstr_len,
        decode::{dbn::AsyncMetadataDecoder as MetadataDecoder, FromLittleEndianSlice},
        Dataset, MappingInterval, MetadataBuilder, SType, Schema,
    };

    #[tokio::test]
    async fn test_encode_decode_metadata_identity() {
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
            symbol_cstr_len: version_symbol_cstr_len(crate::DBN_VERSION),
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
        target.encode(&metadata).await.unwrap();
        dbg!(&buffer);
        let res = MetadataDecoder::new(&mut buffer.as_slice())
            .decode()
            .await
            .unwrap();
        dbg!(&res, &metadata);
        assert_eq!(res, metadata);
    }

    #[tokio::test]
    async fn test_encode_repeated_symbol_cstr() {
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
            .await
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

    #[tokio::test]
    async fn test_encode_fixed_len_cstr() {
        let mut buffer = Vec::new();
        let mut target = MetadataEncoder::new(&mut buffer);
        target
            .encode_fixed_len_cstr(crate::SYMBOL_CSTR_LEN, "NG")
            .await
            .unwrap();
        assert_eq!(buffer.len(), crate::SYMBOL_CSTR_LEN);
        assert_eq!(&buffer[..2], b"NG");
        for b in buffer[2..].iter() {
            assert_eq!(*b, 0);
        }
    }

    #[tokio::test]
    async fn test_encode_date() {
        let date = time::Date::from_calendar_date(2020, time::Month::May, 17).unwrap();
        let mut buffer = Vec::new();
        let mut target = MetadataEncoder::new(&mut buffer);
        target.encode_date(date).await.unwrap();
        assert_eq!(buffer.len(), mem::size_of::<u32>());
        assert_eq!(buffer.as_slice(), 20200517u32.to_le_bytes().as_slice());
    }

    #[tokio::test]
    async fn test_encode_decode_nulls() {
        let metadata = MetadataBuilder::new()
            .dataset(Dataset::XnasItch.to_string())
            .schema(Some(Schema::Mbo))
            .start(1697240529000000000)
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .build();
        assert!(metadata.end.is_none());
        assert!(metadata.limit.is_none());
        let mut buffer = Vec::new();
        MetadataEncoder::new(&mut buffer)
            .encode(&metadata)
            .await
            .unwrap();
        let decoded = MetadataDecoder::new(buffer.as_slice())
            .decode()
            .await
            .unwrap();
        assert!(decoded.end.is_none());
        assert!(decoded.limit.is_none());
    }

    #[tokio::test]
    async fn test_fails_to_encode_newer_metadata() {
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
            matches!(target.encode(&metadata).await, Err(Error::Encode(msg)) if msg.contains("can't encode Metadata with version"))
        );
    }
}
