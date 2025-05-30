use std::path::Path;

use async_compression::tokio::bufread::ZstdDecoder;
use tokio::{
    fs::File,
    io::{self, BufReader},
};

use crate::{
    decode::{
        dbn::fsm::{DbnFsm, ProcessResult},
        r#async::{AsyncSkipBytes, ZSTD_FILE_BUFFER_CAPACITY},
        zstd::zstd_decoder,
        VersionUpgradePolicy,
    },
    HasRType, Metadata, Record, RecordRef, Result, DBN_VERSION,
};

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
    /// Creates a new async DBN [`Decoder`] from `reader`. Will upgrade records from
    /// previous DBN version to the current version.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in
    /// `reader` or the input is encoded in a newer version of DBN.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, the metadata
    /// may have been partially read, corrupting the stream.
    pub async fn new(reader: R) -> crate::Result<Self> {
        let mut metadata_decoder = MetadataDecoder::new(reader);
        let mut metadata = metadata_decoder.decode().await?;
        metadata.upgrade(VersionUpgradePolicy::default());
        Ok(Self {
            decoder: RecordDecoder::from(metadata_decoder),
            metadata,
        })
    }

    /// Creates a new async DBN [`Decoder`] from `reader`. It will decode records from
    /// previous DBN versions according to `upgrade_policy`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in
    /// `reader` or the input is encoded in a newer version of DBN.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, the metadata
    /// may have been partially read, corrupting the stream.
    pub async fn with_upgrade_policy(
        reader: R,
        upgrade_policy: VersionUpgradePolicy,
    ) -> crate::Result<Self> {
        let mut metadata_decoder = MetadataDecoder::with_upgrade_policy(reader, upgrade_policy);
        let metadata = metadata_decoder.decode().await?;
        Ok(Self {
            decoder: RecordDecoder::from(metadata_decoder),
            metadata,
        })
    }

    /// Returns a mutable reference to the inner reader.
    ///
    /// Note: be careful not to modify the inner reader after beginning decoding.
    #[deprecated(
        since = "0.29.0",
        note = "Will be removed in a future version because modifying the inner reader through this method can leave the decoder in an inconsistent state"
    )]
    pub fn get_mut(&mut self) -> &mut R {
        #[expect(deprecated)]
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

    /// Sets the behavior for decoding DBN data of previous versions.
    ///
    /// # Errors
    /// This function will return an error if the `version` and `upgrade_policy` are
    /// incompatible.
    pub fn set_upgrade_policy(&mut self, upgrade_policy: VersionUpgradePolicy) -> Result<()> {
        self.decoder.set_upgrade_policy(upgrade_policy)?;
        self.metadata.upgrade(upgrade_policy);
        Ok(())
    }

    /// Tries to decode a single record and returns a reference to the record that
    /// lasts until the next method call. Returns `Ok(None)` if `reader` has been
    /// exhausted.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an error. If
    /// the next record is of a different type than `T`, this function returns a
    /// [`Error::Conversion`](crate::Error::Conversion) error.
    ///
    /// # Cancel safety
    /// This method is cancel safe. It can be used within a `tokio::select!` statement
    /// without the potential for corrupting the input stream.
    pub async fn decode_record<'a, T: HasRType + 'a>(&'a mut self) -> Result<Option<&'a T>> {
        self.decoder.decode().await
    }

    /// Tries to decode all records into a `Vec`. This eagerly decodes the data.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an error. If
    /// the next record is of a different type than `T`, this function returns a
    /// [`Error::Conversion`](crate::Error::Conversion) error.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If used within a `tokio::select!` statement
    /// partially decoded records will be lost and the stream may be corrupted.
    pub async fn decode_records<T: HasRType + Clone>(&mut self) -> Result<Vec<T>> {
        self.decoder.decode_records().await
    }

    /// Tries to decode a single record and returns a reference to the record that
    /// lasts until the next method call. Returns `Ok(None)` if `reader` has been
    /// exhausted.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an error. It
    /// will also return an error if it encounters an invalid record.
    ///
    /// # Cancel safety
    /// This method is cancel safe. It can be used within a `tokio::select!` statement
    /// without the potential for corrupting the input stream.
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
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, the metadata
    /// may have been partially read, corrupting the stream.
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
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, the metadata
    /// may have been partially read, corrupting the stream.
    pub async fn with_zstd_buffer(reader: R) -> crate::Result<Self> {
        Decoder::new(zstd_decoder(reader)).await
    }
}

impl Decoder<File> {
    /// Creates a new async DBN [`Decoder`] from the file at `path`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to read the file at `path` or
    /// if it is unable to parse the metadata in the file.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, the metadata
    /// may have been partially read, corrupting the stream.
    pub async fn from_file(path: impl AsRef<Path>) -> crate::Result<Self> {
        let file = File::open(path.as_ref()).await.map_err(|e| {
            crate::Error::io(
                e,
                format!("opening DBN file at path '{}'", path.as_ref().display()),
            )
        })?;
        Self::new(file).await
    }
}

impl Decoder<ZstdDecoder<BufReader<File>>> {
    /// Creates a new async DBN [`Decoder`] from the Zstandard-compressed file at `path`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to read the file at `path` or
    /// if it is unable to parse the metadata in the file.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, the metadata
    /// may have been partially read, corrupting the stream.
    pub async fn from_zstd_file(path: impl AsRef<Path>) -> crate::Result<Self> {
        let file = File::open(path.as_ref()).await.map_err(|e| {
            crate::Error::io(
                e,
                format!(
                    "opening Zstandard-compressed DBN file at path '{}'",
                    path.as_ref().display()
                ),
            )
        })?;
        Self::with_zstd_buffer(BufReader::with_capacity(ZSTD_FILE_BUFFER_CAPACITY, file)).await
    }
}

/// An async decoder for files and streams of Databento Binary Encoding (DBN) records.
pub struct RecordDecoder<R>
where
    R: io::AsyncReadExt + Unpin,
{
    reader: R,
    fsm: DbnFsm,
}

impl<R> From<MetadataDecoder<R>> for RecordDecoder<R>
where
    R: io::AsyncReadExt + Unpin,
{
    fn from(metadata_decoder: MetadataDecoder<R>) -> Self {
        let MetadataDecoder { reader, mut fsm } = metadata_decoder;
        if fsm
            .upgrade_policy()
            .is_upgrade_situation(fsm.input_dbn_version().unwrap())
        {
            fsm.grow_compat(DbnFsm::DEFAULT_BUF_SIZE);
        }
        Self { reader, fsm }
    }
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
        Self::with_version(reader, DBN_VERSION, VersionUpgradePolicy::AsIs, false).unwrap()
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
        ts_out: bool,
    ) -> crate::Result<Self> {
        let fsm = DbnFsm::builder()
            .compat_size(crate::MAX_RECORD_LEN)
            .skip_metadata(true)
            .input_dbn_version(Some(version))?
            .upgrade_policy(upgrade_policy)
            .ts_out(ts_out)
            .build()?;
        Ok(Self { reader, fsm })
    }

    /// Sets the DBN version to expect when decoding.
    ///
    /// # Errors
    /// This function will return an error if the `version` exceeds the highest
    /// supported version or the `version` and `upgrade_policy` are incompatible.
    pub fn set_version(&mut self, version: u8) -> crate::Result<()> {
        self.fsm.set_input_dbn_version(version).map(drop)
    }

    /// Sets the behavior for decoding DBN data of previous versions.
    ///
    /// # Errors
    /// This function will return an error if the `version` and `upgrade_policy` are
    /// incompatible.
    pub fn set_upgrade_policy(
        &mut self,
        upgrade_policy: VersionUpgradePolicy,
    ) -> crate::Result<()> {
        self.fsm.set_upgrade_policy(upgrade_policy)
    }

    /// Sets whether to expect a send timestamp appended after every record.
    pub fn set_ts_out(&mut self, ts_out: bool) {
        self.fsm.set_ts_out(ts_out);
    }

    /// Tries to decode a single record and returns a reference to the record that
    /// lasts until the next method call. Returns `None` if `reader` has been
    /// exhausted.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an error. If
    /// the next record is of a different type than `T`, this function returns a
    /// [`Error::Conversion`](crate::Error::Conversion) error.
    ///
    /// # Cancel safety
    /// This method is cancel safe. It can be used within a `tokio::select!` statement
    /// without the potential for corrupting the input stream.
    pub async fn decode<'a, T: HasRType + 'a>(&'a mut self) -> Result<Option<&'a T>> {
        let rec_ref = self.decode_ref().await?;
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

    /// Tries to decode all records into a `Vec`. This eagerly decodes the data.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an error. If
    /// the next record is of a different type than `T`, this function returns a
    /// [`Error::Conversion`](crate::Error::Conversion) error.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If used within a `tokio::select!` statement
    /// partially decoded records will be lost and the stream may be corrupted.
    pub async fn decode_records<T: HasRType + Clone>(&mut self) -> Result<Vec<T>> {
        let mut res = Vec::new();
        while let Some(rec) = self.decode::<T>().await? {
            res.push(rec.clone());
        }
        Ok(res)
    }

    /// Tries to decode a single record and returns a reference to the record that
    /// lasts until the next method call. Returns `None` if `reader` has been
    /// exhausted.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an
    /// error of a kind other than `io::ErrorKind::UnexpectedEof` upon reading.
    /// It will also return an error if it encounters an invalid record.
    ///
    /// # Cancel safety
    /// This method is cancel safe. It can be used within a `tokio::select!` statement
    /// without the potential for corrupting the input stream.
    pub async fn decode_ref(&mut self) -> Result<Option<RecordRef>> {
        loop {
            match self.fsm.process() {
                ProcessResult::ReadMore(_) => match self.reader.read(self.fsm.space()).await {
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

    /// Returns a mutable reference to the inner reader.
    ///
    /// Note: be careful not to modify the inner reader after beginning decoding.
    #[deprecated(
        since = "0.29.0",
        note = "Will be removed in a future version because modifying the inner reader through this method can leave the decoder in an inconsistent state"
    )]
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Consumes the decoder and returns the inner reader.
    pub fn into_inner(self) -> R {
        self.reader
    }
}

impl<R> RecordDecoder<R>
where
    R: AsyncSkipBytes + io::AsyncReadExt + Unpin,
{
    /// Seeks forward the specified number of bytes.
    ///
    /// # Cancel safety
    /// This method may not be cancellation safe, depending on the cancellation safety
    /// of `skip_bytes()` of the inner reader `R`.
    ///
    /// # Errors
    /// This function returns an error if it fails to seek ahead in the inner reader.
    pub async fn skip_bytes(&mut self, n_bytes: usize) -> crate::Result<()> {
        let skipped = self.fsm.skip(n_bytes);
        if skipped < n_bytes {
            self.reader.skip_bytes(n_bytes - skipped).await
        } else {
            Ok(())
        }
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

/// An async decoder for the metadata in files and streams in Databento Binary Encoding (DBN).
pub struct MetadataDecoder<R>
where
    R: io::AsyncReadExt + Unpin,
{
    reader: R,
    fsm: DbnFsm,
}

impl<R> MetadataDecoder<R>
where
    R: io::AsyncReadExt + Unpin,
{
    /// Creates a new async DBN [`MetadataDecoder`] from `reader`.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            fsm: DbnFsm::new(DbnFsm::DEFAULT_BUF_SIZE, 0),
        }
    }

    /// Creates a new async DBN [`MetadataDecoder`] from `reader`. It will decode
    /// metadata from previous DBN versions according to `upgrade_policy`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in
    /// `reader`.
    pub fn with_upgrade_policy(reader: R, upgrade_policy: VersionUpgradePolicy) -> Self {
        let fsm = DbnFsm::builder()
            .compat_size(0)
            .upgrade_policy(upgrade_policy)
            .build()
            // No error because `input_dbn_version` wasn't overwritten
            .unwrap();
        Self { reader, fsm }
    }

    /// Decodes and returns a DBN [`Metadata`].
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata or the
    /// input is encoded in a newere version of DBN.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, the metadata
    /// may have been partially read, corrupting the stream.
    pub async fn decode(&mut self) -> Result<Metadata> {
        let io_err = |err| crate::Error::io(err, "decoding metadata");
        let nbytes = self.reader.read(self.fsm.space()).await.map_err(io_err)?;
        self.fsm.fill(nbytes);
        loop {
            match self.fsm.process() {
                ProcessResult::ReadMore(n) => {
                    // asm guarantees there's at least `n` bytes available in `space()`
                    let mut total_read = 0;
                    loop {
                        let read = self.reader.read(self.fsm.space()).await.map_err(io_err)?;
                        if read == 0 {
                            return Err(crate::Error::io(
                                io::Error::from(io::ErrorKind::UnexpectedEof),
                                "decoding metadata",
                            ));
                        }
                        self.fsm.fill(read);
                        total_read += read;
                        if total_read >= n {
                            break;
                        }
                    }
                }
                ProcessResult::Metadata(metadata) => return Ok(metadata),
                ProcessResult::Record(_) => unreachable!("metadata precedes records"),
                ProcessResult::Err(error) => return Err(error),
            }
        }
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
    #![allow(clippy::clone_on_copy)]

    use rstest::rstest;
    use tokio::io::AsyncWriteExt;

    use super::*;
    use crate::{
        decode::tests::TEST_DATA_PATH,
        encode::{
            dbn::{AsyncEncoder, AsyncRecordEncoder},
            DbnEncodable,
        },
        rtype, v1, v2, Bbo1SMsg, CbboMsg, Cmbp1Msg, Error, ErrorMsg, ImbalanceMsg,
        InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, RecordHeader, Result, Schema,
        StatMsg, StatusMsg, TbboMsg, TradeMsg, WithTsOut,
    };

    #[rstest]
    #[case::mbo(Schema::Mbo, MboMsg::default())]
    #[case::trades(Schema::Trades, TradeMsg::default())]
    #[case::cmbp1(Schema::Cmbp1, Cmbp1Msg::default_for_schema(Schema::Cmbp1))]
    #[case::cbbo1s(Schema::Cbbo1S, CbboMsg::default_for_schema(Schema::Cbbo1S))]
    #[case::bbo1s(Schema::Bbo1S, Bbo1SMsg::default_for_schema(Schema::Bbo1S))]
    #[case::bbo1m(Schema::Bbo1M, Bbo1SMsg::default_for_schema(Schema::Bbo1M))]
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
    #[case::status(Schema::Status, StatusMsg::default())]
    #[tokio::test]
    async fn test_dbn_identity<R: DbnEncodable + HasRType + PartialEq + Clone>(
        #[case] schema: Schema,
        #[case] _rec: R,
    ) -> Result<()> {
        let mut file_decoder =
            Decoder::from_zstd_file(format!("{TEST_DATA_PATH}/test_data.{schema}.v3.dbn.zst"))
                .await?;
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
    async fn test_skip_bytes() {
        let mut decoder = Decoder::from_file(format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn"))
            .await
            .unwrap();
        decoder
            .decoder
            .skip_bytes(std::mem::size_of::<MboMsg>())
            .await
            .unwrap();
        assert!(decoder.decode_record::<MboMsg>().await.unwrap().is_some());
        assert!(decoder.decode_record::<MboMsg>().await.unwrap().is_none());
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
        let buf = vec![0; std::mem::size_of::<RecordHeader>()];
        let mut target = RecordDecoder::new(buf.as_slice());
        assert!(
            matches!(target.decode_ref().await, Err(Error::Decode(msg)) if msg.starts_with("invalid record with impossible length"))
        );
    }

    #[tokio::test]
    async fn test_decode_record_length_less_than_header() {
        let buf = vec![3u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let mut target = RecordDecoder::new(buf.as_slice());
        let res = target.decode_ref().await;
        dbg!(&res);
        assert!(
            matches!(res, Err(Error::Decode(msg)) if msg.starts_with("invalid record with impossible length"))
        );
    }

    #[tokio::test]
    async fn test_decode_partial_record() {
        let buf = vec![6u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        assert!(buf[0] as usize * RecordHeader::LENGTH_MULTIPLIER > buf.len());

        let mut target = RecordDecoder::new(buf.as_slice());
        let res = target.decode_ref().await;
        dbg!(&res);
        assert!(matches!(res, Ok(None)));
    }

    #[tokio::test]
    async fn test_decode_record_length_longer_than_buffer() {
        let rec = ErrorMsg::new(1680703198000000000, None, "Test", true);
        let mut target = RecordDecoder::new(&rec.as_ref()[..rec.record_size() - 1]);
        let res = target.decode_ref().await;
        dbg!(&res);
        assert!(matches!(res, Ok(None)));
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
        while let Some(_rec) = decoder.decode::<v1::InstrumentDefMsg>().await.unwrap() {
            count += 1;
        }
        assert_eq!(count, 8);
    }

    #[tokio::test]
    async fn test_decode_upgrade_v2() -> crate::Result<()> {
        let mut decoder = Decoder::with_upgrade_policy(
            zstd_decoder(BufReader::new(
                tokio::fs::File::open(format!("{TEST_DATA_PATH}/test_data.definition.v1.dbn.zst"))
                    .await
                    .unwrap(),
            )),
            VersionUpgradePolicy::UpgradeToV2,
        )
        .await?;
        assert_eq!(decoder.metadata().version, 2);
        assert_eq!(decoder.metadata().symbol_cstr_len, crate::SYMBOL_CSTR_LEN);
        let mut has_decoded = false;
        while let Some(_rec) = decoder.decode_record::<v2::InstrumentDefMsg>().await? {
            has_decoded = true;
        }
        assert!(has_decoded);
        Ok(())
    }

    #[tokio::test]
    async fn test_decode_upgrade_v3() -> crate::Result<()> {
        let mut decoder = Decoder::with_upgrade_policy(
            zstd_decoder(BufReader::new(
                tokio::fs::File::open(format!("{TEST_DATA_PATH}/test_data.definition.v1.dbn.zst"))
                    .await
                    .unwrap(),
            )),
            VersionUpgradePolicy::UpgradeToV3,
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
