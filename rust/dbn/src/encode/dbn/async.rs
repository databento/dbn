use std::{io::IoSlice, mem::MaybeUninit};

use async_compression::tokio::write::ZstdEncoder;
use tokio::io;

use crate::{
    encode::{
        async_zstd_encoder, io_utils::write_all_vectored_async, AsyncEncodeRecord,
        AsyncEncodeRecordRef, DbnEncodable,
    },
    record_ref::RecordRef,
    Error, Metadata, Result,
};

use super::{MAX_SLICES, TS_OUT_LEN};

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
    /// This function will return an error if `metadata` is version 4 or later, or if
    /// it fails to encode `metadata` to `writer`.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, then the
    /// metadata may have been partially written, but future calls will begin writing
    /// the encoded metadata from the beginning.
    pub async fn new(mut writer: W, metadata: &Metadata) -> Result<Self> {
        super::check_pre_v4(metadata)?;
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

    async fn encode_records<R: DbnEncodable>(&mut self, records: &[R]) -> Result<()> {
        self.record_encoder.encode_records(records).await
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

    async fn encode_record_refs(&mut self, record_refs: &[RecordRef<'_>]) -> Result<()> {
        self.record_encoder.encode_record_refs(record_refs).await
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

    async fn encode_record_ref_with_ts_out(
        &mut self,
        record_ref: RecordRef<'_>,
        ts_out: u64,
    ) -> Result<()> {
        self.record_encoder
            .encode_ref_with_ts_out(record_ref, ts_out)
            .await
    }

    async fn encode_record_refs_with_ts_out(
        &mut self,
        record_refs: &[RecordRef<'_>],
        ts_out: u64,
    ) -> Result<()> {
        self.record_encoder
            .encode_record_refs_with_ts_out(record_refs, ts_out)
            .await
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

    /// Encodes a single DBN record with an appended `ts_out` timestamp.
    ///
    /// This writes the record with the header length modified to account for the
    /// additional 8 bytes of `ts_out`.
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying writer
    /// or the record is too long to append ts_out.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, then the
    /// record may have been partially written, but future calls will begin writing the
    /// encoded record from the beginning.
    pub async fn encode_ref_with_ts_out(
        &mut self,
        record_ref: RecordRef<'_>,
        ts_out: u64,
    ) -> Result<()> {
        let record_bytes = record_ref.as_ref();
        let new_length = [record_bytes[0]
            .checked_add(TS_OUT_LEN)
            .ok_or_else(|| Error::encode("record length overflow adding ts_out"))?];
        let ts_out_bytes = ts_out.to_le_bytes();

        let mut slices = [
            IoSlice::new(&new_length),
            IoSlice::new(&record_bytes[1..]),
            IoSlice::new(&ts_out_bytes),
        ];

        write_all_vectored_async(&mut self.writer, &mut slices)
            .await
            .map_err(|e| Error::io(e, format!("encoding {record_ref:?} with ts_out")))
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

    async fn encode_records<R: DbnEncodable>(&mut self, records: &[R]) -> Result<()> {
        // SAFETY: DBN records have no implicit padding and are POD structs
        let slice = unsafe {
            std::slice::from_raw_parts::<u8>(records.as_ptr() as *const u8, size_of_val(records))
        };
        self.writer
            .write_all(slice)
            .await
            .map_err(|e| Error::io(e, format!("serializing {} records", records.len())))
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

    async fn encode_record_refs(&mut self, record_refs: &[RecordRef<'_>]) -> Result<()> {
        super::write_records_async(&mut self.writer, record_refs)
            .await
            .map_err(|e| {
                Error::io(
                    e,
                    format!("failed to encode {} RecordRefs", record_refs.len()),
                )
            })
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

    async fn encode_record_ref_with_ts_out(
        &mut self,
        record_ref: RecordRef<'_>,
        ts_out: u64,
    ) -> Result<()> {
        self.encode_ref_with_ts_out(record_ref, ts_out).await
    }

    async fn encode_record_refs_with_ts_out(
        &mut self,
        record_refs: &[RecordRef<'_>],
        ts_out: u64,
    ) -> Result<()> {
        // Slices per record: modified length (1 byte), record body (var), ts_out (8 bytes)
        const SLICES_PER_RECORD: usize = 3;
        const BATCH_SIZE: usize = MAX_SLICES / SLICES_PER_RECORD;

        let ts_out_bytes = ts_out.to_le_bytes();

        for record_chunk in record_refs.chunks(BATCH_SIZE) {
            let mut lengths = [0u8; BATCH_SIZE];
            let mut slices = [const { MaybeUninit::uninit() }; BATCH_SIZE * SLICES_PER_RECORD];

            // Build IoSlice array: for each record, SLICES_PER_RECORD slices
            for (i, (rec, len)) in record_chunk.iter().zip(lengths.iter_mut()).enumerate() {
                let record_bytes = rec.as_ref();
                *len = record_bytes[0]
                    .checked_add(TS_OUT_LEN)
                    .ok_or_else(|| Error::encode("record length overflow adding ts_out"))?;
                let base = i * SLICES_PER_RECORD;
                slices[base].write(IoSlice::new(std::slice::from_ref(len)));
                slices[base + 1].write(IoSlice::new(&record_bytes[1..]));
                slices[base + 2].write(IoSlice::new(&ts_out_bytes));
            }

            let slice_count = record_chunk.len() * SLICES_PER_RECORD;
            // SAFETY: Every element up to `slice_count` has been initialized
            let slices = unsafe {
                std::mem::transmute::<&mut [MaybeUninit<IoSlice<'_>>], &mut [IoSlice<'_>]>(
                    &mut slices[..slice_count],
                )
            };

            write_all_vectored_async(&mut self.writer, slices)
                .await
                .map_err(|e| {
                    Error::io(
                        e,
                        format!("encoding {} RecordRefs with ts_out", record_refs.len()),
                    )
                })?;
        }
        Ok(())
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
        let mut buf = Vec::with_capacity(super::MetadataEncoder::<Vec<u8>>::encoded_len(metadata));
        super::MetadataEncoder::new(&mut buf).encode(metadata)?;
        self.writer
            .write_all(&buf)
            .await
            .map_err(|e| Error::io(e, "writing DBN metadata"))
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
    use std::num::NonZeroU64;

    use rstest::rstest;

    use super::*;
    use crate::{
        compat::version_symbol_cstr_len,
        decode::{dbn::AsyncMetadataDecoder as MetadataDecoder, DecodeRecord, DecodeRecordRef},
        enums::rtype,
        record::{MboMsg, RecordHeader, TradeMsg},
        Dataset, FlagSet, MappingInterval, MetadataBuilder, SType, Schema, SymbolMapping,
    };

    #[tokio::test]
    async fn test_encode_decode_metadata_identity() {
        let metadata = Metadata {
            version: crate::DBN_VERSION,
            dataset: Dataset::GlbxMdp3.to_string(),
            layout: None,
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
            intervals: vec![],
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
            .version(crate::v4::DBN_VERSION + 1)
            .build();
        let mut buffer = Vec::new();
        let mut target = MetadataEncoder::new(&mut buffer);
        assert!(
            matches!(target.encode(&metadata).await, Err(Error::Encode(msg)) if msg.contains("can't encode Metadata with version"))
        );
    }

    #[tokio::test]
    async fn test_refuses_v4_metadata() {
        let mut metadata = MetadataBuilder::new()
            .dataset(Dataset::XnasItch.to_string())
            .schema(Some(Schema::Mbo))
            .start(0)
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .version(crate::v4::DBN_VERSION)
            .build();
        let mut sb = crate::layout::StreamLayoutBuilder::new(1, metadata.symbol_cstr_len as u16);
        sb.layout_of::<crate::v4::MboMsg>(crate::v4::rtype::MBO)
            .finish();
        metadata.layout = Some(sb.build().unwrap());
        let mut buffer = Vec::new();
        assert!(Encoder::new(&mut buffer, &metadata).await.is_err());
        assert!(buffer.is_empty());
    }

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

    #[tokio::test]
    async fn test_encode_record_refs_roundtrip() {
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
        encoder.encode_record_refs(&refs).await.unwrap();

        // Verify sizes
        let expected_size = std::mem::size_of::<MboMsg>() * 2 + std::mem::size_of::<TradeMsg>() * 2;
        assert_eq!(buffer.len(), expected_size);

        // Decode and verify roundtrip
        let mut decoder = crate::decode::dbn::RecordDecoder::new(&buffer[..]);

        let decoded = decoder.decode_record_ref().unwrap().unwrap();
        assert_eq!(decoded.header(), &mbo1.hd);

        let decoded = decoder.decode_record_ref().unwrap().unwrap();
        assert_eq!(decoded.header(), &trade1.hd);

        let decoded = decoder.decode_record_ref().unwrap().unwrap();
        assert_eq!(decoded.header(), &mbo2.hd);

        let decoded = decoder.decode_record_ref().unwrap().unwrap();
        assert_eq!(decoded.header(), &trade2.hd);

        assert!(decoder.decode_record_ref().unwrap().is_none());
    }

    #[tokio::test]
    async fn test_encode_record_refs_single_is_equivalent() {
        let mbo = make_mbo_msg(100, 1658441851000000000);
        let trade = make_trade_msg(101, 1658441851001000000);

        let refs: Vec<RecordRef> = vec![RecordRef::from(&mbo), RecordRef::from(&trade)];

        // Encode one at a time
        let mut buffer_single = Vec::new();
        let mut encoder_single = RecordEncoder::new(&mut buffer_single);
        for record_ref in &refs {
            encoder_single.encode_record_ref(*record_ref).await.unwrap();
        }

        // Encode as batch
        let mut buffer_batch = Vec::new();
        let mut encoder_batch = RecordEncoder::new(&mut buffer_batch);
        encoder_batch.encode_record_refs(&refs).await.unwrap();

        assert_eq!(buffer_single, buffer_batch);
    }

    #[tokio::test]
    async fn test_encode_record_refs_empty_slice() {
        let refs: Vec<RecordRef> = vec![];

        let mut buffer = Vec::new();
        let mut encoder = RecordEncoder::new(&mut buffer);
        encoder.encode_record_refs(&refs).await.unwrap();

        assert!(buffer.is_empty());
    }

    #[rstest]
    #[case::partial_batch(127)]
    #[case::exact_batch(128)]
    #[case::batch_plus_one(129)]
    #[case::multiple_batches(200)]
    #[tokio::test]
    async fn test_encode_record_refs_batch_sizes(#[case] count: usize) {
        let records: Vec<MboMsg> = (0..count)
            .map(|i| make_mbo_msg(100 + i as u32, 1658441851000000000 + i as u64 * 1000))
            .collect();

        let refs: Vec<RecordRef> = records.iter().map(RecordRef::from).collect();

        let mut buffer = Vec::new();
        let mut encoder = RecordEncoder::new(&mut buffer);
        encoder.encode_record_refs(&refs).await.unwrap();

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

    #[tokio::test]
    async fn test_encode_record_refs_with_ts_out_roundtrip() {
        use crate::WithTsOut;

        let mbo1 = make_mbo_msg(100, 1658441851000000000);
        let trade1 = make_trade_msg(101, 1658441851001000000);
        let mbo2 = make_mbo_msg(102, 1658441851002000000);

        let refs: Vec<RecordRef> = vec![
            RecordRef::from(&mbo1),
            RecordRef::from(&trade1),
            RecordRef::from(&mbo2),
        ];
        let ts_out = 1658441851999000000u64;

        let mut buffer = Vec::new();
        let mut encoder = RecordEncoder::new(&mut buffer);
        encoder
            .encode_record_refs_with_ts_out(&refs, ts_out)
            .await
            .unwrap();

        let expected_size = std::mem::size_of::<MboMsg>() * 2
            + std::mem::size_of::<TradeMsg>()
            + 3 * std::mem::size_of::<u64>();
        assert_eq!(buffer.len(), expected_size);

        let mut decoder = crate::decode::dbn::RecordDecoder::new(&buffer[..]);

        let decoded: &WithTsOut<MboMsg> = decoder.decode_record().unwrap().unwrap();
        assert_eq!(decoded.rec.hd.instrument_id, mbo1.hd.instrument_id);
        assert_eq!(decoded.ts_out, ts_out);

        let decoded: &WithTsOut<TradeMsg> = decoder.decode_record().unwrap().unwrap();
        assert_eq!(decoded.rec.hd.instrument_id, trade1.hd.instrument_id);
        assert_eq!(decoded.ts_out, ts_out);

        let decoded: &WithTsOut<MboMsg> = decoder.decode_record().unwrap().unwrap();
        assert_eq!(decoded.rec.hd.instrument_id, mbo2.hd.instrument_id);
        assert_eq!(decoded.ts_out, ts_out);
    }

    #[tokio::test]
    async fn test_encode_record_refs_with_ts_out_single_is_equivalent() {
        let mbo = make_mbo_msg(100, 1658441851000000000);
        let trade = make_trade_msg(101, 1658441851001000000);

        let refs: Vec<RecordRef> = vec![RecordRef::from(&mbo), RecordRef::from(&trade)];
        let ts_out = 1658441851999000000u64;

        let mut buffer_single = Vec::new();
        let mut encoder_single = RecordEncoder::new(&mut buffer_single);
        for record_ref in &refs {
            encoder_single
                .encode_record_ref_with_ts_out(*record_ref, ts_out)
                .await
                .unwrap();
        }

        let mut buffer_batch = Vec::new();
        let mut encoder_batch = RecordEncoder::new(&mut buffer_batch);
        encoder_batch
            .encode_record_refs_with_ts_out(&refs, ts_out)
            .await
            .unwrap();

        assert_eq!(buffer_single, buffer_batch);
    }

    #[rstest]
    #[case::partial_batch(41)]
    #[case::exact_batch(42)]
    #[case::batch_plus_one(43)]
    #[case::multiple_batches(100)]
    #[tokio::test]
    async fn test_encode_record_refs_with_ts_out_batch_sizes(#[case] count: usize) {
        use crate::WithTsOut;

        let records: Vec<MboMsg> = (0..count)
            .map(|i| make_mbo_msg(100 + i as u32, 1658441851000000000 + i as u64 * 1000))
            .collect();

        let refs: Vec<RecordRef> = records.iter().map(RecordRef::from).collect();
        let ts_out = 1658441851999000000u64;

        let mut buffer = Vec::new();
        let mut encoder = RecordEncoder::new(&mut buffer);
        encoder
            .encode_record_refs_with_ts_out(&refs, ts_out)
            .await
            .unwrap();

        let expected_size = count * (std::mem::size_of::<MboMsg>() + std::mem::size_of::<u64>());
        assert_eq!(buffer.len(), expected_size);

        let mut decoder = crate::decode::dbn::RecordDecoder::new(&buffer[..]);
        let mut decoded_count = 0;
        while let Some(decoded) = decoder.decode_record::<WithTsOut<MboMsg>>().unwrap() {
            assert_eq!(decoded.ts_out, ts_out);
            decoded_count += 1;
        }
        assert_eq!(decoded_count, count);
    }
}
