//! Encoding DBN version 4 records.

use std::io;

use crate::{
    record::{HasRType, Record},
    v4, Error, Metadata, Result,
};

/// Encodes a DBN version 4 stream.
pub struct Encoder<W>
where
    W: io::Write,
{
    record_encoder: RecordEncoder<W>,
    layout_check: LayoutCheck,
}

impl<W> Encoder<W>
where
    W: io::Write,
{
    /// Writes `metadata` to `writer`, leaving the encoder positioned for records.
    ///
    /// # Errors
    /// This function returns an error if `metadata` is not a valid version 4 header,
    /// notably when it carries no stream layout, or if writing fails.
    pub fn new(mut writer: W, metadata: &Metadata) -> Result<Self> {
        let layout_check = LayoutCheck::new(metadata)?;
        super::MetadataEncoder::new(&mut writer).encode(metadata)?;
        Ok(Self {
            record_encoder: RecordEncoder::new(writer),
            layout_check,
        })
    }

    /// Returns a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        self.record_encoder.get_ref()
    }

    /// Returns a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        self.record_encoder.get_mut()
    }

    /// Encodes a single record.
    ///
    /// # Errors
    /// This function returns an error if the stream layout describes the record's rtype
    /// differently than `R`, if the record's header length differs from `R`'s size, or
    /// if it fails to write to the underlying writer.
    pub fn encode<R>(&mut self, record: &R) -> Result<()>
    where
        R: HasRType<Header = v4::RecordHeader>,
    {
        self.layout_check.check(record)?;
        self.record_encoder.encode(record)
    }

    /// Encodes a single record of a type known only at run time.
    ///
    /// # Errors
    /// This function returns an error if the record's size differs from its rtype's size
    /// in the stream layout, or if it fails to write to the underlying writer.
    pub fn encode_ref(&mut self, record: v4::RecordRef) -> Result<()> {
        self.layout_check.check_ref(record)?;
        self.record_encoder.encode_ref(record)
    }

    /// Encodes several records of types known only at run time. Writes nothing if any
    /// record's size differs from its rtype's size in the stream layout.
    ///
    /// # Errors
    /// This function returns an error if a record's size differs from its rtype's size in
    /// the stream layout, or if it fails to write to the underlying writer.
    pub fn encode_refs(&mut self, records: &[v4::RecordRef]) -> Result<()> {
        self.layout_check.check_refs(records)?;
        self.record_encoder.encode_refs(records)
    }
}

/// Encodes DBN version 4 records, without metadata.
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
    /// Creates a new encoder that will write to `writer`.
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

    /// Encodes a single record.
    ///
    /// # Errors
    /// This function returns an error if it fails to write to the underlying writer.
    pub fn encode<R>(&mut self, record: &R) -> Result<()>
    where
        R: HasRType<Header = v4::RecordHeader>,
    {
        self.writer
            .write_all(record.as_ref())
            .map_err(|e| Error::io(e, "serializing record"))
    }

    /// Encodes a single record of a type known only at run time.
    ///
    /// # Errors
    /// This function returns an error if it fails to write to the underlying writer.
    pub fn encode_ref(&mut self, record: v4::RecordRef) -> Result<()> {
        self.writer
            .write_all(record.as_ref())
            .map_err(|e| Error::io(e, format!("serializing {record:?}")))
    }

    /// Encodes several records of types known only at run time.
    ///
    /// # Errors
    /// This function returns an error if it fails to write to the underlying writer.
    pub fn encode_refs(&mut self, records: &[v4::RecordRef]) -> Result<()> {
        super::write_records(&mut self.writer, records)
            .map_err(|e| Error::io(e, format!("failed to encode {} RecordRefs", records.len())))
    }
}

struct LayoutCheck {
    rtypes: Vec<RTypeInfo>,
}

struct RTypeInfo {
    rtype: u16,
    record_size: u16,
    matches_compiled: bool,
}

impl LayoutCheck {
    fn new(metadata: &Metadata) -> Result<Self> {
        if metadata.version < v4::DBN_VERSION {
            return Err(Error::encode(format!(
                "a version 4 encoder can't write version {} metadata",
                metadata.version
            )));
        }
        let compiled = v4::layouts::default_layout();
        let Some(layout) = &metadata.layout else {
            return Err(Error::encode("version 4 metadata requires a layout"));
        };
        let mut rtypes: Vec<_> = layout
            .record_layouts
            .iter()
            .map(|rec| RTypeInfo {
                rtype: rec.rtype,
                record_size: rec.record_size(),
                matches_compiled: compiled
                    .record_layout(rec.rtype)
                    .is_some_and(|c| c.layout_hash == rec.layout_hash),
            })
            .collect();
        rtypes.sort_unstable_by_key(|info: &RTypeInfo| info.rtype);
        Ok(Self { rtypes })
    }

    fn check_size(&self, rtype: u16, record_size: usize) -> Result<&RTypeInfo> {
        let info = self
            .rtypes
            .binary_search_by_key(&rtype, |info| info.rtype)
            .map(|i| &self.rtypes[i])
            .map_err(|_| {
                Error::encode(format!("the stream layout carries no rtype {rtype:#06X}"))
            })?;
        if record_size == usize::from(info.record_size) {
            Ok(info)
        } else {
            Err(Error::encode(format!(
                "rtype {rtype:#06X} is {} bytes in this stream's layout, but the record is \
                 {record_size}",
                info.record_size
            )))
        }
    }

    fn check<R>(&self, record: &R) -> Result<()>
    where
        R: HasRType<Header = v4::RecordHeader>,
    {
        let rtype = record.raw_rtype();
        if record.record_size() != size_of::<R>() {
            return Err(Error::encode(format!(
                "rtype {rtype:#06X} record's header length says {} bytes, but it's {}",
                record.record_size(),
                size_of::<R>()
            )));
        }
        if !self
            .check_size(rtype, record.record_size())?
            .matches_compiled
        {
            return Err(Error::encode(format!(
                "the stream layout describes rtype {rtype:#06X} differently than the compiled \
                 struct; encode it with `encode_ref`"
            )));
        }
        Ok(())
    }

    fn check_ref(&self, record: v4::RecordRef) -> Result<()> {
        self.check_size(record.raw_rtype(), record.record_size())
            .map(|_| ())
    }

    fn check_refs(&self, records: &[v4::RecordRef]) -> Result<()> {
        records
            .iter()
            .try_for_each(|record| self.check_ref(*record))
    }
}

#[cfg(feature = "zstd")]
impl<W> Encoder<zstd::stream::AutoFinishEncoder<'_, W>>
where
    W: io::Write,
{
    /// Writes Zstandard-compressed output to `writer`.
    ///
    /// # Errors
    /// This function returns an error if it fails to encode `metadata`.
    pub fn with_zstd(writer: W, metadata: &Metadata) -> Result<Self> {
        Encoder::new(crate::encode::zstd_encoder(writer)?, metadata)
    }
}

/// Encodes a DBN version 4 stream asynchronously.
#[cfg(feature = "async")]
pub struct AsyncEncoder<W>
where
    W: tokio::io::AsyncWriteExt + Unpin,
{
    record_encoder: AsyncRecordEncoder<W>,
    layout_check: LayoutCheck,
}

#[cfg(feature = "async")]
impl<W> AsyncEncoder<W>
where
    W: tokio::io::AsyncWriteExt + Unpin,
{
    /// Writes `metadata` to `writer`, leaving the encoder positioned for records.
    ///
    /// # Errors
    /// This function returns an error if `metadata` is not a valid version 4 header,
    /// notably when it carries no stream layout, or if writing fails.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, then the
    /// metadata may have been partially written, but future calls will begin writing
    /// the encoded metadata from the beginning.
    pub async fn new(mut writer: W, metadata: &Metadata) -> Result<Self> {
        let layout_check = LayoutCheck::new(metadata)?;
        super::AsyncMetadataEncoder::new(&mut writer)
            .encode(metadata)
            .await?;
        Ok(Self {
            record_encoder: AsyncRecordEncoder::new(writer),
            layout_check,
        })
    }

    /// Returns a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        self.record_encoder.get_ref()
    }

    /// Returns a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        self.record_encoder.get_mut()
    }

    /// Encodes a single record.
    ///
    /// # Errors
    /// This function returns an error if the stream layout describes the record's rtype
    /// differently than `R`, if the record's header length differs from `R`'s size, or
    /// if it fails to write to the underlying writer.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, then the
    /// record may have been partially written, but future calls will begin writing the
    /// encoded record from the beginning.
    pub async fn encode<R>(&mut self, record: &R) -> Result<()>
    where
        R: HasRType<Header = v4::RecordHeader>,
    {
        self.layout_check.check(record)?;
        self.record_encoder.encode(record).await
    }

    /// Encodes a single record of a type known only at run time.
    ///
    /// # Errors
    /// This function returns an error if the record's size differs from its rtype's size
    /// in the stream layout, or if it fails to write to the underlying writer.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, then the
    /// record may have been partially written, but future calls will begin writing the
    /// encoded record from the beginning.
    pub async fn encode_ref(&mut self, record: v4::RecordRef<'_>) -> Result<()> {
        self.layout_check.check_ref(record)?;
        self.record_encoder.encode_ref(record).await
    }

    /// Encodes several records of types known only at run time. Writes nothing if any
    /// record's size differs from its rtype's size in the stream layout.
    ///
    /// # Errors
    /// This function returns an error if a record's size differs from its rtype's size in
    /// the stream layout, or if it fails to write to the underlying writer.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, then some
    /// records may have been written, and future calls will not rewrite them.
    pub async fn encode_refs(&mut self, records: &[v4::RecordRef<'_>]) -> Result<()> {
        self.layout_check.check_refs(records)?;
        self.record_encoder.encode_refs(records).await
    }

    /// Flushes any buffered content to the true output.
    ///
    /// # Errors
    /// This function returns an error if it's unable to flush the underlying writer.
    pub async fn flush(&mut self) -> Result<()> {
        self.record_encoder.flush().await
    }

    /// Initiates or attempts to shut down the inner writer, finishing any Zstandard
    /// frame.
    ///
    /// # Errors
    /// This function returns an error if the shut down did not complete successfully.
    pub async fn shutdown(&mut self) -> Result<()> {
        self.record_encoder.shutdown().await
    }
}

#[cfg(feature = "async")]
impl<W> AsyncEncoder<async_compression::tokio::write::ZstdEncoder<W>>
where
    W: tokio::io::AsyncWriteExt + Unpin,
{
    /// Writes Zstandard-compressed output to `writer`.
    ///
    /// # Errors
    /// This function returns an error if it fails to encode `metadata`.
    pub async fn with_zstd(writer: W, metadata: &Metadata) -> Result<Self> {
        AsyncEncoder::new(crate::encode::async_zstd_encoder(writer), metadata).await
    }
}

/// Encodes DBN version 4 records asynchronously, without metadata.
#[cfg(feature = "async")]
pub struct AsyncRecordEncoder<W>
where
    W: tokio::io::AsyncWriteExt + Unpin,
{
    writer: W,
}

#[cfg(feature = "async")]
impl<W> AsyncRecordEncoder<W>
where
    W: tokio::io::AsyncWriteExt + Unpin,
{
    /// Creates a new encoder that will write to `writer`.
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

    /// Encodes a single record.
    ///
    /// # Errors
    /// This function returns an error if it fails to write to the underlying writer.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, then the
    /// record may have been partially written, but future calls will begin writing the
    /// encoded record from the beginning.
    pub async fn encode<R>(&mut self, record: &R) -> Result<()>
    where
        R: HasRType<Header = v4::RecordHeader>,
    {
        self.writer
            .write_all(record.as_ref())
            .await
            .map_err(|e| Error::io(e, "serializing record"))
    }

    /// Encodes a single record of a type known only at run time.
    ///
    /// # Errors
    /// This function returns an error if it fails to write to the underlying writer.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, then the
    /// record may have been partially written, but future calls will begin writing the
    /// encoded record from the beginning.
    pub async fn encode_ref(&mut self, record: v4::RecordRef<'_>) -> Result<()> {
        self.writer
            .write_all(record.as_ref())
            .await
            .map_err(|e| Error::io(e, format!("serializing {record:?}")))
    }

    /// Encodes several records of types known only at run time.
    ///
    /// # Errors
    /// This function returns an error if it fails to write to the underlying writer.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, then some
    /// records may have been written, and future calls will not rewrite them.
    pub async fn encode_refs(&mut self, records: &[v4::RecordRef<'_>]) -> Result<()> {
        super::write_records_async(&mut self.writer, records)
            .await
            .map_err(|e| Error::io(e, format!("failed to encode {} RecordRefs", records.len())))
    }

    /// Flushes any buffered content to the true output.
    ///
    /// # Errors
    /// This function returns an error if it's unable to flush the underlying writer.
    pub async fn flush(&mut self) -> Result<()> {
        self.writer
            .flush()
            .await
            .map_err(|e| Error::io(e, "flushing output"))
    }

    /// Initiates or attempts to shut down the inner writer, finishing any Zstandard
    /// frame.
    ///
    /// # Errors
    /// This function returns an error if the shut down did not complete successfully.
    pub async fn shutdown(&mut self) -> Result<()> {
        self.writer
            .shutdown()
            .await
            .map_err(|e| Error::io(e, "shutting down"))
    }
}

#[cfg(test)]
mod tests {
    use std::os::raw::c_char;

    use super::{super::MAX_SLICES, *};
    use crate::{
        layout::{RecordLayoutDesc, StreamLayoutBuilder},
        v4::{fields, rtype},
        Action, Dataset, FlagSet, MetadataBuilder, Record, SType, Schema, Side, UNDEF_PRICE,
        UNDEF_TIMESTAMP,
    };

    const TS_EVENT: u64 = 1_609_160_400_000_000_000;

    fn metadata() -> Metadata {
        let mut metadata = MetadataBuilder::new()
            .version(v4::DBN_VERSION)
            .dataset(Dataset::GlbxMdp3)
            .schema(Some(Schema::Mbo))
            .start(TS_EVENT)
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .build();
        let mut sb = StreamLayoutBuilder::new(1, metadata.symbol_cstr_len as u16);
        sb.record_of::<v4::MboMsg>(rtype::MBO)
            .record_of::<v4::TradeMsg>(rtype::MBP_0);
        metadata.layout = Some(sb.build().unwrap());
        metadata
    }

    fn trade() -> v4::TradeMsg {
        v4::TradeMsg {
            hd: v4::RecordHeader::new::<v4::TradeMsg>(rtype::MBP_0, 1, 42_000, TS_EVENT),
            price: 4_215_500_000_000,
            size: 7,
            action: Action::Trade as c_char,
            side: Side::Bid as c_char,
            flags: FlagSet::empty(),
            _reserved: 0,
            ts_recv: TS_EVENT + 1_500,
            ts_in_delta: 1_200,
            sequence: 88,
        }
    }

    fn mbo() -> v4::MboMsg {
        v4::MboMsg {
            hd: v4::RecordHeader::new::<v4::MboMsg>(rtype::MBO, 1, 42_000, TS_EVENT),
            order_id: 90_112,
            price: UNDEF_PRICE,
            size: 3,
            flags: FlagSet::empty(),
            channel_id: 0,
            action: Action::Add as c_char,
            side: Side::Ask as c_char,
            ts_recv: UNDEF_TIMESTAMP,
            ts_in_delta: 900,
            sequence: 87,
        }
    }

    #[test]
    fn a_record_is_written_verbatim() {
        let trade = trade();
        let mut buffer = Vec::new();
        RecordEncoder::new(&mut buffer).encode(&trade).unwrap();

        assert_eq!(buffer.len(), size_of::<v4::TradeMsg>());
        assert_eq!(buffer, trade.as_ref());
        let length = u16::from_le_bytes([buffer[0], buffer[1]]);
        assert_eq!(
            length as usize,
            size_of::<v4::TradeMsg>() - size_of::<v4::RecordHeader>()
        );
        assert_eq!(trade.record_size(), buffer.len());
    }

    #[rstest::rstest]
    #[case::one(1)]
    #[case::a_full_batch(MAX_SLICES)]
    #[case::past_a_batch(MAX_SLICES + 1)]
    fn encode_refs_matches_one_at_a_time(#[case] count: usize) {
        let trade = trade();
        let records = vec![v4::RecordRef::from(&trade); count];

        let mut batched = Vec::new();
        RecordEncoder::new(&mut batched)
            .encode_refs(&records)
            .unwrap();

        let mut one_by_one = Vec::new();
        let mut encoder = RecordEncoder::new(&mut one_by_one);
        for rec in &records {
            encoder.encode_ref(*rec).unwrap();
        }

        assert_eq!(batched, one_by_one);
        assert_eq!(batched.len(), count * size_of::<v4::TradeMsg>());
    }

    #[test]
    fn a_stream_is_metadata_then_records() {
        let metadata = metadata();
        let mut expected_metadata = Vec::new();
        super::super::MetadataEncoder::new(&mut expected_metadata)
            .encode(&metadata)
            .unwrap();

        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer, &metadata).unwrap();
        let mbo = mbo();
        let trade = trade();
        encoder.encode(&mbo).unwrap();
        encoder.encode(&trade).unwrap();

        assert_eq!(&buffer[..expected_metadata.len()], &expected_metadata[..]);
        let records = &buffer[expected_metadata.len()..];
        assert_eq!(&records[..size_of::<v4::MboMsg>()], mbo.as_ref());
        assert_eq!(&records[size_of::<v4::MboMsg>()..], trade.as_ref());
    }

    #[test]
    fn an_older_metadata_is_refused() {
        let mut metadata = metadata();
        metadata.version = 3;
        metadata.layout = None;
        let mut buffer = Vec::new();
        assert!(Encoder::new(&mut buffer, &metadata).is_err());
    }

    #[test]
    fn a_v4_metadata_without_a_layout_is_refused() {
        let mut metadata = metadata();
        metadata.layout = None;
        assert!(Encoder::new(Vec::new(), &metadata).is_err());
    }

    #[test]
    fn a_batch_with_a_mis_sized_record_writes_nothing() {
        let mut encoder = Encoder::new(Vec::new(), &metadata()).unwrap();
        let metadata_len = encoder.get_ref().len();
        let mut mislabeled = trade();
        mislabeled.hd.rtype = rtype::MBO;
        let trade = trade();

        let records = [
            v4::RecordRef::from(&trade),
            v4::RecordRef::from(&mislabeled),
        ];
        assert!(encoder.encode_refs(&records).is_err());
        assert_eq!(encoder.get_ref().len(), metadata_len);
    }

    fn ts_out_metadata() -> Metadata {
        let mut metadata = metadata();
        metadata.schema = Some(Schema::Trades);
        metadata.ts_out = true;
        let mut sb = StreamLayoutBuilder::new(1, metadata.symbol_cstr_len as u16);
        {
            let mut r = sb.record(rtype::MBP_0, Schema::Trades as u16, v4::TradeMsg::NAME);
            v4::TradeMsg::add_fields(&mut r);
            r.ext_field(fields::TS_OUT, "ts_out");
            r.finish();
        }
        metadata.layout = Some(sb.build().unwrap());
        metadata
    }

    #[test]
    fn a_typed_record_needs_the_compiled_layout() {
        let mut metadata = metadata();
        let layout = metadata.layout.as_mut().unwrap();
        let rec = layout
            .record_layouts
            .iter_mut()
            .find(|r| r.rtype == rtype::MBP_0);
        rec.unwrap().layout_hash ^= 1;
        let mut encoder = Encoder::new(Vec::new(), &metadata).unwrap();
        let metadata_len = encoder.get_ref().len();
        let trade = trade();

        assert!(encoder.encode(&trade).is_err());
        assert_eq!(encoder.get_ref().len(), metadata_len);
        encoder.encode_ref(v4::RecordRef::from(&trade)).unwrap();
        assert_eq!(
            encoder.get_ref().len(),
            metadata_len + size_of::<v4::TradeMsg>()
        );
    }

    #[test]
    fn an_extension_tail_goes_through_encode_ref() {
        let mut encoder = Encoder::new(Vec::new(), &ts_out_metadata()).unwrap();
        let metadata_len = encoder.get_ref().len();
        assert!(encoder.encode(&trade()).is_err());
        assert_eq!(encoder.get_ref().len(), metadata_len);

        let mut buf = v4::RecordBuf::<64>::from(trade());
        let raw = buf.raw_buf_mut();
        raw[size_of::<v4::TradeMsg>()..].copy_from_slice(&777u64.to_le_bytes());
        raw[..2].copy_from_slice(&((64 - size_of::<v4::RecordHeader>()) as u16).to_le_bytes());
        encoder.encode_ref(buf.as_rec_ref()).unwrap();
        assert_eq!(&encoder.get_ref()[metadata_len..], buf.as_ref());
    }

    #[test]
    fn an_inflated_header_length_is_refused() {
        let mut encoder = Encoder::new(Vec::new(), &ts_out_metadata()).unwrap();
        let metadata_len = encoder.get_ref().len();
        let mut trade = trade();
        trade.hd = v4::RecordHeader::new::<v4::MboMsg>(rtype::MBP_0, 1, 42_000, TS_EVENT);

        assert_eq!(trade.record_size(), 64);
        let err = encoder.encode(&trade).unwrap_err();
        assert!(err.to_string().contains("header length"), "{err}");
        assert_eq!(encoder.get_ref().len(), metadata_len);
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn the_async_stream_matches_the_sync_stream() {
        let metadata = metadata();
        let mbo = mbo();
        let trade = trade();
        let refs = [v4::RecordRef::from(&trade), v4::RecordRef::from(&mbo)];

        let mut sync_encoder = Encoder::new(Vec::new(), &metadata).unwrap();
        sync_encoder.encode(&mbo).unwrap();
        sync_encoder
            .encode_ref(v4::RecordRef::from(&trade))
            .unwrap();
        sync_encoder.encode_refs(&refs).unwrap();

        let mut async_encoder = AsyncEncoder::new(Vec::new(), &metadata).await.unwrap();
        async_encoder.encode(&mbo).await.unwrap();
        async_encoder
            .encode_ref(v4::RecordRef::from(&trade))
            .await
            .unwrap();
        async_encoder.encode_refs(&refs).await.unwrap();

        assert_eq!(async_encoder.get_ref(), sync_encoder.get_ref());
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn the_async_encoder_checks_the_layout() {
        let mut metadata = metadata();
        metadata.version = 3;
        metadata.layout = None;
        assert!(AsyncEncoder::new(Vec::new(), &metadata).await.is_err());

        let mut encoder = AsyncEncoder::new(Vec::new(), &ts_out_metadata())
            .await
            .unwrap();
        let metadata_len = encoder.get_ref().len();
        let mut mislabeled = trade();
        let trade = trade();
        mislabeled.hd.rtype = rtype::MBO;

        assert!(encoder.encode(&trade).await.is_err());
        assert!(encoder
            .encode_ref(v4::RecordRef::from(&mislabeled))
            .await
            .is_err());
        assert!(encoder
            .encode_refs(&[v4::RecordRef::from(&mislabeled)])
            .await
            .is_err());
        assert_eq!(encoder.get_ref().len(), metadata_len);
    }
}
