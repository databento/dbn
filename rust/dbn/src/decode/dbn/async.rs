use std::path::Path;

use async_compression::tokio::bufread::ZstdDecoder;
use tokio::{
    fs::File,
    io::{self, BufReader},
};

use crate::{
    compat,
    decode::{FromLittleEndianSlice, VersionUpgradePolicy},
    error::silence_eof_error,
    HasRType, Metadata, Record, RecordHeader, RecordRef, Result, DBN_VERSION, METADATA_FIXED_LEN,
};

/// Helper to always set multiple members.
fn zstd_decoder<R>(reader: R) -> ZstdDecoder<R>
where
    R: io::AsyncBufReadExt + Unpin,
{
    let mut zstd_decoder = ZstdDecoder::new(reader);
    // explicitly enable decoding multiple frames
    zstd_decoder.multiple_members(true);
    zstd_decoder
}

/// An async decoder for Databento Binary Encoding (DBN), both metadata and records.
pub struct Decoder<R>
where
    R: io::AsyncReadExt + Unpin,
{
    metadata: Metadata,
    decoder: RecordDecoder<R>,
}

impl<R> Decoder<R>
where
    R: io::AsyncReadExt + Unpin,
{
    /// Creates a new async DBN [`Decoder`] from `reader`. Will decode records from
    /// previous DBN versions as-is.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in
    /// `reader` or the input is encoded in a newer version of DBN.
    pub async fn new(mut reader: R) -> crate::Result<Self> {
        let metadata = MetadataDecoder::new(&mut reader).decode().await?;
        Ok(Self {
            decoder: RecordDecoder::with_version(
                reader,
                metadata.version,
                VersionUpgradePolicy::AsIs,
            )?,
            metadata,
        })
    }

    /// Creates a new async DBN [`Decoder`] from `reader`. It will decode records from
    /// previous DBN versions according to `upgrade_policy`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in
    /// `reader` or the input is encoded in a newer version of DBN.
    pub async fn with_upgrade_policy(
        mut reader: R,
        upgrade_policy: VersionUpgradePolicy,
    ) -> crate::Result<Self> {
        let mut metadata = MetadataDecoder::new(&mut reader).decode().await?;
        // need to get the original version
        let version = metadata.version;
        metadata.upgrade(upgrade_policy);
        Ok(Self {
            decoder: RecordDecoder::with_version(reader, version, upgrade_policy)?,
            metadata,
        })
    }

    /// Returns a mutable reference to the inner reader.
    pub fn get_mut(&mut self) -> &mut R {
        self.decoder.get_mut()
    }

    /// Consumes the decoder and returns the inner reader.
    pub fn into_inner(self) -> R {
        self.decoder.into_inner()
    }

    /// Returns a reference to the decoded metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Tries to decode a single record and returns a reference to the record that
    /// lasts until the next method call. Returns `Ok(None)` if `reader` has been
    /// exhausted.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an
    /// error of a kind other than `io::ErrorKind::UnexpectedEof` upon reading.
    ///
    /// If the next record is of a different type than `T`,
    /// this function returns an error of kind `io::ErrorKind::InvalidData`.
    pub async fn decode_record<'a, T: HasRType + 'a>(&'a mut self) -> Result<Option<&T>> {
        self.decoder.decode().await
    }

    /// Tries to decode a single record and returns a reference to the record that
    /// lasts until the next method call. Returns `Ok(None)` if `reader` has been
    /// exhausted.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an
    /// error of a kind other than `io::ErrorKind::UnexpectedEof` upon reading.
    /// It will also return an error if it encounters an invalid record.
    pub async fn decode_record_ref(&mut self) -> Result<Option<RecordRef>> {
        self.decoder.decode_ref().await
    }
}

impl<R> Decoder<ZstdDecoder<BufReader<R>>>
where
    R: io::AsyncReadExt + Unpin,
{
    /// Creates a new async DBN [`Decoder`] from Zstandard-compressed `reader`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in `reader`.
    pub async fn with_zstd(reader: R) -> crate::Result<Self> {
        Decoder::new(zstd_decoder(BufReader::new(reader))).await
    }
}

impl<R> Decoder<ZstdDecoder<R>>
where
    R: io::AsyncBufReadExt + Unpin,
{
    /// Creates a new async DBN [`Decoder`] from Zstandard-compressed buffered `reader`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in `reader`.
    pub async fn with_zstd_buffer(reader: R) -> crate::Result<Self> {
        Decoder::new(zstd_decoder(reader)).await
    }
}

impl Decoder<BufReader<File>> {
    /// Creates a new async DBN [`Decoder`] from the file at `path`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to read the file at `path` or
    /// if it is unable to parse the metadata in the file.
    pub async fn from_file(path: impl AsRef<Path>) -> crate::Result<Self> {
        let file = File::open(path.as_ref()).await.map_err(|e| {
            crate::Error::io(
                e,
                format!(
                    "Error opening DBN file at path '{}'",
                    path.as_ref().display()
                ),
            )
        })?;
        Self::new(BufReader::new(file)).await
    }
}

impl Decoder<ZstdDecoder<BufReader<File>>> {
    /// Creates a new async DBN [`Decoder`] from the Zstandard-compressed file at `path`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to read the file at `path` or
    /// if it is unable to parse the metadata in the file.
    pub async fn from_zstd_file(path: impl AsRef<Path>) -> crate::Result<Self> {
        let file = File::open(path.as_ref()).await.map_err(|e| {
            crate::Error::io(
                e,
                format!(
                    "Error opening Zstandard-compressed DBN file at path '{}'",
                    path.as_ref().display()
                ),
            )
        })?;
        Self::with_zstd(file).await
    }
}

/// An async decoder for files and streams of Databento Binary Encoding (DBN) records.
pub struct RecordDecoder<R>
where
    R: io::AsyncReadExt + Unpin,
{
    version: u8,
    upgrade_policy: VersionUpgradePolicy,
    reader: R,
    read_buffer: Vec<u8>,
    compat_buffer: [u8; crate::MAX_RECORD_LEN],
}

impl<R> RecordDecoder<R>
where
    R: io::AsyncReadExt + Unpin,
{
    /// Creates a new DBN [`RecordDecoder`] from `reader`.
    ///
    /// Note: assumes the input is of the current DBN version. To decode records from a
    /// previous version, use [`RecordDecoder::with_version()`].
    pub fn new(reader: R) -> Self {
        Self::with_version(reader, DBN_VERSION, VersionUpgradePolicy::AsIs).unwrap()
    }

    /// Creates a new `RecordDecoder` that will decode from `reader`
    /// with the specified DBN version.
    ///
    /// # Errors
    /// This function will return an error if the `version` exceeds the supported version.
    pub fn with_version(
        reader: R,
        version: u8,
        upgrade_policy: VersionUpgradePolicy,
    ) -> crate::Result<Self> {
        if version > DBN_VERSION {
            return Err(crate::Error::decode(format!("Can't decode newer version of DBN. Decoder version is {DBN_VERSION}, input version is {version}")));
        }
        Ok(Self {
            version,
            upgrade_policy,
            reader,
            // `read_buffer` should have capacity for reading `length`
            read_buffer: vec![0],
            compat_buffer: [0; crate::MAX_RECORD_LEN],
        })
    }

    /// Sets the DBN version to expect when decoding.
    ///
    /// # Errors
    /// This function will return an error if the `version` exceeds the highest
    /// supported version.
    pub fn set_version(&mut self, version: u8) -> crate::Result<()> {
        if version > DBN_VERSION {
            Err(crate::Error::decode(format!("Can't decode newer version of DBN. Decoder version is {DBN_VERSION}, input version is {version}")))
        } else {
            self.version = version;
            Ok(())
        }
    }

    /// Tries to decode a single record and returns a reference to the record that
    /// lasts until the next method call. Returns `None` if `reader` has been
    /// exhausted.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an
    /// error of a kind other than `io::ErrorKind::UnexpectedEof` upon reading.
    ///
    /// If the next record is of a different type than `T`,
    /// this function returns an error of kind `io::ErrorKind::InvalidData`.
    pub async fn decode<'a, T: HasRType + 'a>(&'a mut self) -> Result<Option<&T>> {
        let rec_ref = self.decode_ref().await?;
        if let Some(rec_ref) = rec_ref {
            rec_ref
                .get::<T>()
                .ok_or_else(|| {
                    crate::Error::conversion::<T>(format!(
                        "record with rtype {}",
                        rec_ref.header().rtype
                    ))
                })
                .map(Some)
        } else {
            Ok(None)
        }
    }

    /// Tries to decode a single record and returns a reference to the record that
    /// lasts until the next method call. Returns `None` if `reader` has been
    /// exhausted.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an
    /// error of a kind other than `io::ErrorKind::UnexpectedEof` upon reading.
    /// It will also return an error if it encounters an invalid record.
    pub async fn decode_ref(&mut self) -> Result<Option<RecordRef>> {
        let io_err = |e| crate::Error::io(e, "decoding record reference");
        if let Err(err) = self.reader.read_exact(&mut self.read_buffer[..1]).await {
            return silence_eof_error(err).map_err(io_err);
        }
        let length = self.read_buffer[0] as usize * RecordHeader::LENGTH_MULTIPLIER;
        if length > self.read_buffer.len() {
            self.read_buffer.resize(length, 0);
        }
        if length < std::mem::size_of::<RecordHeader>() {
            return Err(crate::Error::decode(format!(
                "Invalid record with length {length} shorter than header"
            )));
        }
        if let Err(err) = self
            .reader
            .read_exact(&mut self.read_buffer[1..length])
            .await
        {
            return silence_eof_error(err).map_err(io_err);
        }
        // Safety: `read_buffer` is resized to contain at least `length` bytes.
        Ok(Some(unsafe {
            compat::decode_record_ref(
                self.version,
                self.upgrade_policy,
                &mut self.compat_buffer,
                &self.read_buffer,
            )
        }))
    }

    /// Returns a mutable reference to the inner reader.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Consumes the decoder and returns the inner reader.
    pub fn into_inner(self) -> R {
        self.reader
    }
}

impl<R> RecordDecoder<ZstdDecoder<BufReader<R>>>
where
    R: io::AsyncReadExt + Unpin,
{
    /// Creates a new async DBN [`RecordDecoder`] from a Zstandard-compressed `reader`.
    pub fn with_zstd(reader: R) -> Self {
        RecordDecoder::new(zstd_decoder(BufReader::new(reader)))
    }
}

impl<R> RecordDecoder<ZstdDecoder<R>>
where
    R: io::AsyncBufReadExt + Unpin,
{
    /// Creates a new async DBN [`RecordDecoder`] from a Zstandard-compressed buffered `reader`.
    pub fn with_zstd_buffer(reader: R) -> Self {
        RecordDecoder::new(zstd_decoder(reader))
    }
}

impl<R> From<MetadataDecoder<R>> for RecordDecoder<R>
where
    R: io::AsyncReadExt + Unpin,
{
    fn from(meta_decoder: MetadataDecoder<R>) -> Self {
        RecordDecoder::new(meta_decoder.into_inner())
    }
}

/// An async decoder for the metadata in files and streams in Databento Binary Encoding (DBN).
pub struct MetadataDecoder<R>
where
    R: io::AsyncReadExt + Unpin,
{
    reader: R,
}

impl<R> MetadataDecoder<R>
where
    R: io::AsyncReadExt + Unpin,
{
    /// Creates a new async DBN [`MetadataDecoder`] from `reader`.
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Decodes and returns a DBN [`Metadata`].
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata or the
    /// input is encoded in a newere version of DBN.
    pub async fn decode(&mut self) -> Result<Metadata> {
        let mut prelude_buffer = [0u8; 8];
        self.reader
            .read_exact(&mut prelude_buffer)
            .await
            .map_err(|e| crate::Error::io(e, "reading metadata prelude"))?;
        if &prelude_buffer[..super::DBN_PREFIX_LEN] != super::DBN_PREFIX {
            return Err(crate::Error::decode("Invalid DBN header"));
        }
        let version = prelude_buffer[super::DBN_PREFIX_LEN];
        if version > DBN_VERSION {
            return Err(crate::Error::decode(format!("Can't decode newer version of DBN. Decoder version is {DBN_VERSION}, input version is {version}")));
        }
        let length = u32::from_le_slice(&prelude_buffer[4..]);
        if (length as usize) < METADATA_FIXED_LEN {
            return Err(crate::Error::decode(
                "Invalid DBN metadata. Metadata length shorter than fixed length.",
            ));
        }

        let mut metadata_buffer = vec![0u8; length as usize];
        self.reader
            .read_exact(&mut metadata_buffer)
            .await
            .map_err(|e| crate::Error::io(e, "reading fixed metadata"))?;
        super::MetadataDecoder::<std::fs::File>::decode_metadata_fields(version, metadata_buffer)
    }

    /// Returns a mutable reference to the inner reader.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Consumes the decoder and returns the inner reader.
    pub fn into_inner(self) -> R {
        self.reader
    }
}

impl<R> MetadataDecoder<ZstdDecoder<BufReader<R>>>
where
    R: io::AsyncReadExt + Unpin,
{
    /// Creates a new async DBN [`MetadataDecoder`] from a Zstandard-compressed `reader`.
    pub fn with_zstd(reader: R) -> Self {
        MetadataDecoder::new(zstd_decoder(BufReader::new(reader)))
    }
}

impl<R> MetadataDecoder<ZstdDecoder<R>>
where
    R: io::AsyncBufReadExt + Unpin,
{
    /// Creates a new async DBN [`MetadataDecoder`] from a Zstandard-compressed buffered `reader`.
    pub fn with_zstd_buffer(reader: R) -> Self {
        MetadataDecoder::new(zstd_decoder(reader))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use tokio::io::AsyncWriteExt;

    use super::*;
    use crate::{
        compat::InstrumentDefMsgV1,
        decode::tests::TEST_DATA_PATH,
        encode::{
            dbn::{AsyncEncoder, AsyncRecordEncoder},
            DbnEncodable,
        },
        rtype, Error, ErrorMsg, ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg,
        OhlcvMsg, RecordHeader, Result, Schema, StatMsg, TbboMsg, TradeMsg, WithTsOut,
    };

    #[rstest]
    #[case::mbo(Schema::Mbo, MboMsg::default())]
    #[case::trades(Schema::Trades, TradeMsg::default())]
    #[case::tbbo(Schema::Tbbo, TbboMsg::default())]
    #[case::mbp1(Schema::Mbp1, Mbp1Msg::default())]
    #[case::mbp10(Schema::Mbp10, Mbp10Msg::default())]
    #[case::ohlcv1d(Schema::Ohlcv1D, OhlcvMsg::default_for_schema(Schema::Ohlcv1D))]
    #[case::ohlcv1h(Schema::Ohlcv1H, OhlcvMsg::default_for_schema(Schema::Ohlcv1H))]
    #[case::ohlcv1m(Schema::Ohlcv1M, OhlcvMsg::default_for_schema(Schema::Ohlcv1M))]
    #[case::ohlcv1s(Schema::Ohlcv1S, OhlcvMsg::default_for_schema(Schema::Ohlcv1S))]
    #[case::definitions(Schema::Definition, InstrumentDefMsg::default())]
    #[case::imbalance(Schema::Imbalance, ImbalanceMsg::default())]
    #[case::statistics(Schema::Statistics, StatMsg::default())]
    #[tokio::test]
    async fn test_dbn_identity<R: DbnEncodable + HasRType + PartialEq + Clone>(
        #[case] schema: Schema,
        #[case] _rec: R,
    ) -> Result<()> {
        let mut file_decoder =
            Decoder::from_file(format!("{TEST_DATA_PATH}/test_data.{schema}.dbn")).await?;
        let file_metadata = file_decoder.metadata().clone();
        let mut file_records = Vec::new();
        while let Some(record) = file_decoder.decode_record::<R>().await? {
            file_records.push(record.clone());
        }
        assert_eq!(file_records.is_empty(), schema == Schema::Ohlcv1D);
        let mut buffer = Vec::new();
        let mut buf_encoder = AsyncEncoder::new(&mut buffer, &file_metadata).await?;

        for record in file_records.iter() {
            buf_encoder.encode_record(record).await.unwrap();
        }
        let mut buf_cursor = std::io::Cursor::new(&mut buffer);
        let mut buf_decoder = Decoder::new(&mut buf_cursor).await?;
        assert_eq!(*buf_decoder.metadata(), file_metadata);
        let mut buf_records = Vec::new();
        while let Some(record) = buf_decoder.decode_record::<R>().await? {
            buf_records.push(record.clone());
        }
        assert_eq!(buf_records, file_records);
        Ok(())
    }

    #[rstest]
    #[case::mbo(Schema::Mbo, MboMsg::default())]
    #[case::trades(Schema::Trades, TradeMsg::default())]
    #[case::tbbo(Schema::Tbbo, TbboMsg::default())]
    #[case::mbp1(Schema::Mbp1, Mbp1Msg::default())]
    #[case::mbp10(Schema::Mbp10, Mbp10Msg::default())]
    #[case::ohlcv1d(Schema::Ohlcv1D, OhlcvMsg::default_for_schema(Schema::Ohlcv1D))]
    #[case::ohlcv1h(Schema::Ohlcv1H, OhlcvMsg::default_for_schema(Schema::Ohlcv1H))]
    #[case::ohlcv1m(Schema::Ohlcv1M, OhlcvMsg::default_for_schema(Schema::Ohlcv1M))]
    #[case::ohlcv1s(Schema::Ohlcv1S, OhlcvMsg::default_for_schema(Schema::Ohlcv1S))]
    #[case::definitions(Schema::Definition, InstrumentDefMsg::default())]
    #[case::imbalance(Schema::Imbalance, ImbalanceMsg::default())]
    #[case::statistics(Schema::Statistics, StatMsg::default())]
    #[tokio::test]
    async fn test_dbn_zstd_identity<R: DbnEncodable + HasRType + PartialEq + Clone>(
        #[case] schema: Schema,
        #[case] _rec: R,
    ) -> Result<()> {
        let mut file_decoder =
            Decoder::from_zstd_file(format!("{TEST_DATA_PATH}/test_data.{schema}.dbn.zst")).await?;
        let file_metadata = file_decoder.metadata().clone();
        let mut file_records = Vec::new();
        while let Some(record) = file_decoder.decode_record::<R>().await? {
            file_records.push(record.clone());
        }
        assert_eq!(file_records.is_empty(), schema == Schema::Ohlcv1D);
        let mut buffer = Vec::new();
        let mut buf_encoder = AsyncEncoder::with_zstd(&mut buffer, &file_metadata).await?;

        for record in file_records.iter() {
            buf_encoder.encode_record(record).await.unwrap();
        }
        buf_encoder.get_mut().shutdown().await.unwrap();
        let mut buf_cursor = std::io::Cursor::new(&mut buffer);
        let mut buf_decoder = Decoder::with_zstd(&mut buf_cursor).await?;
        assert_eq!(*buf_decoder.metadata(), file_metadata);
        let mut buf_records = Vec::new();
        while let Some(record) = buf_decoder.decode_record::<R>().await? {
            buf_records.push(record.clone());
        }
        assert_eq!(buf_records, file_records);
        Ok(())
    }

    #[tokio::test]
    async fn test_dbn_identity_with_ts_out() {
        let rec1 = WithTsOut {
            rec: OhlcvMsg {
                hd: RecordHeader::new::<WithTsOut<OhlcvMsg>>(rtype::OHLCV_1D, 1, 446, 1678284110),
                open: 160270000000,
                high: 161870000000,
                low: 157510000000,
                close: 158180000000,
                volume: 3170000,
            },
            ts_out: 1678486110,
        };
        let mut rec2 = rec1.clone();
        rec2.rec.hd.instrument_id += 1;
        rec2.ts_out = 1678486827;
        let mut buffer = Vec::new();
        let mut encoder = AsyncRecordEncoder::new(&mut buffer);
        encoder.encode(&rec1).await.unwrap();
        encoder.encode(&rec2).await.unwrap();
        let mut decoder_with = RecordDecoder::new(buffer.as_slice());
        let res1_with = decoder_with
            .decode::<WithTsOut<OhlcvMsg>>()
            .await
            .unwrap()
            .unwrap()
            .clone();
        let res2_with = decoder_with
            .decode::<WithTsOut<OhlcvMsg>>()
            .await
            .unwrap()
            .unwrap()
            .clone();
        assert_eq!(rec1, res1_with);
        assert_eq!(rec2, res2_with);
        let mut decoder_without = RecordDecoder::new(buffer.as_slice());
        let res1_without = decoder_without
            .decode::<OhlcvMsg>()
            .await
            .unwrap()
            .unwrap()
            .clone();
        let res2_without = decoder_without
            .decode::<OhlcvMsg>()
            .await
            .unwrap()
            .unwrap()
            .clone();
        assert_eq!(rec1.rec, res1_without);
        assert_eq!(rec2.rec, res2_without);
    }

    #[tokio::test]
    async fn test_decode_record_0_length() {
        let buf = vec![0];
        let mut target = RecordDecoder::new(buf.as_slice());
        assert!(
            matches!(target.decode_ref().await, Err(Error::Decode(msg)) if msg.starts_with("Invalid record with length"))
        );
    }

    #[tokio::test]
    async fn test_decode_record_length_less_than_header() {
        let buf = vec![3u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        assert_eq!(buf[0] as usize * RecordHeader::LENGTH_MULTIPLIER, buf.len());

        let mut target = RecordDecoder::new(buf.as_slice());
        assert!(
            matches!(target.decode_ref().await, Err(Error::Decode(msg)) if msg.starts_with("Invalid record with length"))
        );
    }

    #[tokio::test]
    async fn test_decode_record_length_longer_than_buffer() {
        let rec = ErrorMsg::new(1680703198000000000, "Test");
        let mut target = RecordDecoder::new(&rec.as_ref()[..rec.record_size() - 1]);
        assert!(matches!(target.decode_ref().await, Ok(None)));
    }

    #[tokio::test]
    async fn test_decode_multiframe_zst() {
        let mut decoder = RecordDecoder::with_zstd(
            tokio::fs::File::open(format!(
                "{TEST_DATA_PATH}/multi-frame.definition.v1.dbn.frag.zst"
            ))
            .await
            .unwrap(),
        );
        let mut count = 0;
        while let Some(_rec) = decoder.decode::<InstrumentDefMsgV1>().await.unwrap() {
            count += 1;
        }
        assert_eq!(count, 8);
    }

    #[tokio::test]
    async fn test_decode_upgrade() -> crate::Result<()> {
        let mut decoder = Decoder::with_upgrade_policy(
            tokio::fs::File::open(format!("{TEST_DATA_PATH}/test_data.definition.v1.dbn"))
                .await
                .unwrap(),
            VersionUpgradePolicy::Upgrade,
        )
        .await?;
        assert_eq!(decoder.metadata().version, crate::DBN_VERSION);
        assert_eq!(decoder.metadata().symbol_cstr_len, crate::SYMBOL_CSTR_LEN);
        let mut has_decoded = false;
        while let Some(_rec) = decoder.decode_record::<InstrumentDefMsg>().await? {
            has_decoded = true;
        }
        assert!(has_decoded);
        Ok(())
    }
}
