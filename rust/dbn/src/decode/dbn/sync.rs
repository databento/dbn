use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};

use crate::{
    decode::{
        dbn::fsm::{DbnFsm, ProcessResult},
        private::LastRecord,
        DbnMetadata, DecodeRecord, DecodeRecordRef, DecodeStream, SkipBytes, StreamIterDecoder,
        VersionUpgradePolicy,
    },
    HasRType, Metadata, RecordRef, DBN_VERSION,
};

/// Type for decoding files and streams in Databento Binary Encoding (DBN), both metadata and records.
pub struct Decoder<R> {
    metadata: Metadata,
    decoder: RecordDecoder<R>,
}

impl<R> Decoder<R>
where
    R: io::Read,
{
    /// Creates a new DBN [`Decoder`] from `reader`. Will upgrade records from previous
    /// DBN version to the current version.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in
    /// `reader` or the input is encoded in a newer version of DBN.
    pub fn new(reader: R) -> crate::Result<Self> {
        let mut metadata_decoder = MetadataDecoder::new(reader);
        let metadata = metadata_decoder.decode()?;
        Ok(Self {
            decoder: RecordDecoder::from(metadata_decoder),
            metadata,
        })
    }

    /// Creates a new DBN [`Decoder`] from `reader`. It will decode records from
    /// previous DBN versions according to `upgrade_policy`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in
    /// `reader` or the input is encoded in a newer version of DBN.
    pub fn with_upgrade_policy(
        reader: R,
        upgrade_policy: VersionUpgradePolicy,
    ) -> crate::Result<Self> {
        let mut metadata_decoder = MetadataDecoder::with_upgrade_policy(reader, upgrade_policy);
        let metadata = metadata_decoder.decode()?;
        Ok(Self {
            decoder: RecordDecoder::from(metadata_decoder),
            metadata,
        })
    }

    /// Returns a mutable reference to the inner reader.
    pub fn get_mut(&mut self) -> &mut R {
        self.decoder.get_mut()
    }

    /// Returns a reference to the inner reader.
    pub fn get_ref(&self) -> &R {
        self.decoder.get_ref()
    }

    /// Consumes the decoder and returns the inner reader.
    pub fn into_inner(self) -> R {
        self.decoder.into_inner()
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
        self.decoder.set_upgrade_policy(upgrade_policy)?;
        self.metadata.upgrade(upgrade_policy);
        Ok(())
    }
}

impl<R> Decoder<zstd::stream::Decoder<'_, BufReader<R>>>
where
    R: io::Read,
{
    /// Creates a new DBN [`Decoder`] from Zstandard-compressed `reader`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in `reader`.
    pub fn with_zstd(reader: R) -> crate::Result<Self> {
        Decoder::new(
            zstd::stream::Decoder::new(reader)
                .map_err(|e| crate::Error::io(e, "creating zstd decoder"))?,
        )
    }
}

impl<R> Decoder<zstd::stream::Decoder<'_, R>>
where
    R: io::BufRead,
{
    /// Creates a new DBN [`Decoder`] from Zstandard-compressed buffered `reader`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in `reader`.
    pub fn with_zstd_buffer(reader: R) -> crate::Result<Self> {
        Decoder::new(
            zstd::stream::Decoder::with_buffer(reader)
                .map_err(|e| crate::Error::io(e, "creating zstd decoder"))?,
        )
    }
}

impl Decoder<File> {
    /// Creates a DBN [`Decoder`] from the file at `path`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to read the file at `path` or
    /// if it is unable to parse the metadata in the file.
    pub fn from_file(path: impl AsRef<Path>) -> crate::Result<Self> {
        let file = File::open(path.as_ref()).map_err(|e| {
            crate::Error::io(
                e,
                format!("opening DBN file at path '{}'", path.as_ref().display()),
            )
        })?;
        Self::new(file)
    }
}

impl Decoder<zstd::stream::Decoder<'_, BufReader<File>>> {
    /// Creates a DBN [`Decoder`] from the Zstandard-compressed file at `path`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to read the file at `path` or
    /// if it is unable to parse the metadata in the file.
    pub fn from_zstd_file(path: impl AsRef<Path>) -> crate::Result<Self> {
        let file = File::open(path.as_ref()).map_err(|e| {
            crate::Error::io(
                e,
                format!(
                    "opening Zstandard-compressed DBN file at path '{}'",
                    path.as_ref().display()
                ),
            )
        })?;
        Self::with_zstd(file)
    }
}

impl<R> DecodeRecordRef for Decoder<R>
where
    R: io::Read,
{
    fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef>> {
        self.decoder.decode_record_ref()
    }
}

impl<R> DbnMetadata for Decoder<R> {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }
}

impl<R> DecodeRecord for Decoder<R>
where
    R: io::Read,
{
    fn decode_record<T: HasRType>(&mut self) -> crate::Result<Option<&T>> {
        self.decoder.decode()
    }
}

impl<R> DecodeStream for Decoder<R>
where
    R: io::Read,
{
    fn decode_stream<T: HasRType>(self) -> StreamIterDecoder<Self, T> {
        StreamIterDecoder::new(self)
    }
}

impl<R> LastRecord for Decoder<R>
where
    R: io::Read,
{
    fn last_record(&self) -> Option<RecordRef> {
        self.decoder.last_record()
    }
}

/// A DBN decoder of records
pub struct RecordDecoder<R> {
    reader: R,
    fsm: DbnFsm,
}

impl<R> From<MetadataDecoder<R>> for RecordDecoder<R>
where
    R: io::Read,
{
    fn from(metadata_decoder: MetadataDecoder<R>) -> Self {
        let MetadataDecoder { reader, mut fsm } = metadata_decoder;
        if fsm
            .upgrade_policy()
            // Okay to unwrap because input DBN version will always be set after decoding metadata
            .is_upgrade_situation(fsm.input_dbn_version().unwrap())
        {
            fsm.grow_compat(DbnFsm::DEFAULT_BUF_SIZE);
        }
        Self { reader, fsm }
    }
}

impl<R> RecordDecoder<R>
where
    R: io::Read,
{
    /// Creates a new `RecordDecoder` that will decode from `reader`.
    ///
    /// Note: assumes the input is of the current DBN version. To decode records from a
    /// previous version, use [`RecordDecoder::with_version()`].
    pub fn new(reader: R) -> Self {
        // upgrade policy doesn't matter when decoding current DBN version
        Self::with_version(reader, DBN_VERSION, VersionUpgradePolicy::AsIs, false).unwrap()
    }

    /// Creates a new `RecordDecoder` that will decode from `reader` with the specified
    /// DBN version and update records according to `upgrade_policy`.
    ///
    /// # Errors
    /// This function will return an error if the `version` exceeds the highest
    /// supported version or the `version` and `upgrade_policy` are incompatible.
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

    /// Returns a mutable reference to the inner reader.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Returns a reference to the inner reader.
    pub fn get_ref(&self) -> &R {
        &self.reader
    }

    /// Consumes the decoder and returns the inner reader.
    pub fn into_inner(self) -> R {
        self.reader
    }

    /// Tries to decode the next record of type `T`. Returns `Ok(None)` if
    /// the reader is exhausted.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an
    /// error of a kind other than `io::ErrorKind::UnexpectedEof` upon reading.
    ///
    /// If the next record is of a different type than `T`,
    /// this function returns an error of kind `io::ErrorKind::InvalidData`.
    pub fn decode<T: HasRType>(&mut self) -> crate::Result<Option<&T>> {
        self.decode_ref().and_then(|rec| {
            if let Some(rec) = rec {
                rec.try_get().map(Some)
            } else {
                Ok(None)
            }
        })
    }

    /// Tries to decode a generic reference a record. Returns `Ok(None)` if
    /// the reader is exhausted.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an
    /// error of a kind other than `io::ErrorKind::UnexpectedEof` upon reading.
    /// It will also return an error if it encounters an invalid record.
    pub fn decode_ref(&mut self) -> crate::Result<Option<RecordRef>> {
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

impl<R> DecodeRecord for RecordDecoder<R>
where
    R: io::Read,
{
    fn decode_record<T: HasRType>(&mut self) -> crate::Result<Option<&T>> {
        self.decode()
    }
}

impl<R> DecodeRecordRef for RecordDecoder<R>
where
    R: io::Read,
{
    fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef>> {
        self.decode_ref()
    }
}

impl<R> LastRecord for RecordDecoder<R>
where
    R: io::Read,
{
    fn last_record(&self) -> Option<RecordRef> {
        self.fsm.last_record()
    }
}

impl<R> SkipBytes for RecordDecoder<R>
where
    R: SkipBytes,
{
    fn skip_bytes(&mut self, n_bytes: usize) -> crate::Result<()> {
        let skipped = self.fsm.skip(n_bytes);
        if skipped < n_bytes {
            self.reader.skip_bytes(n_bytes - skipped)
        } else {
            Ok(())
        }
    }
}

/// Type for decoding [`Metadata`] from Databento Binary Encoding (DBN).
pub struct MetadataDecoder<R>
where
    R: io::Read,
{
    reader: R,
    fsm: DbnFsm,
}

impl<R> MetadataDecoder<R>
where
    R: io::Read,
{
    /// Creates a new DBN [`MetadataDecoder`] from `reader`.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            fsm: DbnFsm::new(DbnFsm::DEFAULT_BUF_SIZE, 0),
        }
    }

    /// Creates a new DBN [`MetadataDecoder`] from `reader`. It will decode metadata from
    /// previous DBN versions according to `upgrade_policy`.
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
    /// This function will return an error if it is unable to parse the metadata.
    pub fn decode(&mut self) -> crate::Result<Metadata> {
        let io_err = |err| crate::Error::io(err, "decoding metadata");
        let nbytes = self.reader.read(self.fsm.space()).map_err(io_err)?;
        self.fsm.fill(nbytes);
        match self.fsm.process() {
            ProcessResult::ReadMore(n) => {
                // Fsm guarantees there's at least `n` bytes available in `space()`
                let mut total_read = 0;
                loop {
                    let read = self.reader.read(self.fsm.space()).map_err(io_err)?;
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
        match self.fsm.process() {
            ProcessResult::Metadata(metadata) => Ok(metadata),
            ProcessResult::Err(error) => Err(error),
            ProcessResult::ReadMore(_) => unreachable!("read requested number of bytes"),
            ProcessResult::Record(_) => unreachable!("metadata precedes records"),
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

pub(crate) fn decode_iso8601(raw: u32) -> Result<time::Date, String> {
    let year = raw / 10_000;
    let remaining = raw % 10_000;
    let raw_month = remaining / 100;
    let month = u8::try_from(raw_month)
        .map_err(|e| format!("{e:?} while parsing {raw} into date"))
        .and_then(|m| {
            time::Month::try_from(m).map_err(|e| format!("{e:?} while parsing {raw} into date"))
        })?;
    let day = remaining % 100;
    time::Date::from_calendar_date(year as i32, month, day as u8)
        .map_err(|e| format!("couldn't convert {raw} to a valid date: {e:?}"))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::clone_on_copy)]

    use std::fs::File;

    use rstest::rstest;

    use super::*;
    use crate::{
        decode::{tests::TEST_DATA_PATH, DynReader},
        encode::{
            dbn::Encoder, DbnEncodable, DbnRecordEncoder, DynWriter, EncodeDbn, EncodeRecord,
        },
        rtype, v1, v2, v3, Compression, Dataset, Error, ErrorMsg, MboMsg, MetadataBuilder,
        OhlcvMsg, Record, RecordHeader, Result, SType, Schema, WithTsOut,
    };

    #[test]
    fn test_decode_iso8601_valid() {
        let res = decode_iso8601(20151031).unwrap();
        let exp: time::Date =
            time::Date::from_calendar_date(2015, time::Month::October, 31).unwrap();
        assert_eq!(res, exp);
    }

    #[test]
    fn test_decode_iso8601_invalid_month() {
        let res = decode_iso8601(20101305);
        dbg!(&res);
        assert!(matches!(res, Err(e) if e.contains("month")));
    }

    #[test]
    fn test_decode_iso8601_invalid_day() {
        let res = decode_iso8601(20100600);
        dbg!(&res);
        assert!(matches!(res, Err(e) if e.contains("a valid date")));
    }

    #[rstest]
    #[case::mbo_v1(1, Schema::Mbo, v1::MboMsg::default())]
    #[case::trades_v1(1, Schema::Trades, v1::TradeMsg::default())]
    #[case::tbbo_v1(1, Schema::Tbbo, v1::TbboMsg::default())]
    #[case::mbp1_v1(1, Schema::Mbp1, v1::Mbp1Msg::default())]
    #[case::mbp10_v1(1, Schema::Mbp10, v1::Mbp10Msg::default())]
    #[case::ohlcv1d_v1(1, Schema::Ohlcv1D, v1::OhlcvMsg::default_for_schema(Schema::Ohlcv1D))]
    #[case::ohlcv1h_v1(1, Schema::Ohlcv1H, v1::OhlcvMsg::default_for_schema(Schema::Ohlcv1H))]
    #[case::ohlcv1m_v1(1, Schema::Ohlcv1M, v1::OhlcvMsg::default_for_schema(Schema::Ohlcv1M))]
    #[case::ohlcv1s_v1(1, Schema::Ohlcv1S, v1::OhlcvMsg::default_for_schema(Schema::Ohlcv1S))]
    #[case::definitions_v1(1, Schema::Definition, v1::InstrumentDefMsg::default())]
    #[case::imbalance_v1(1, Schema::Imbalance, v1::ImbalanceMsg::default())]
    #[case::statistics_v1(1, Schema::Statistics, v1::StatMsg::default())]
    #[case::mbo_v2(2, Schema::Mbo, v2::MboMsg::default())]
    #[case::trades_v2(2, Schema::Trades, v2::TradeMsg::default())]
    #[case::tbbo_v2(2, Schema::Tbbo, v2::TbboMsg::default())]
    #[case::mbp1_v2(2, Schema::Mbp1, v2::Mbp1Msg::default())]
    #[case::cmbp1_v2(2, Schema::Cmbp1, v2::Cmbp1Msg::default_for_schema(Schema::Cmbp1))]
    #[case::cbbo1s_v2(2, Schema::Cbbo1S, v2::Cbbo1SMsg::default_for_schema(Schema::Cbbo1S))]
    #[case::bbo1s_v2(2, Schema::Bbo1S, v2::Bbo1SMsg::default_for_schema(Schema::Bbo1S))]
    #[case::bbo1m_v2(2, Schema::Bbo1M, v2::Bbo1MMsg::default_for_schema(Schema::Bbo1M))]
    #[case::mbp10_v2(2, Schema::Mbp10, v2::Mbp10Msg::default())]
    #[case::ohlcv1d_v2(2, Schema::Ohlcv1D, v2::OhlcvMsg::default_for_schema(Schema::Ohlcv1D))]
    #[case::ohlcv1h_v2(2, Schema::Ohlcv1H, v2::OhlcvMsg::default_for_schema(Schema::Ohlcv1H))]
    #[case::ohlcv1m_v2(2, Schema::Ohlcv1M, v2::OhlcvMsg::default_for_schema(Schema::Ohlcv1M))]
    #[case::ohlcv1s_v2(2, Schema::Ohlcv1S, v2::OhlcvMsg::default_for_schema(Schema::Ohlcv1S))]
    #[case::definitions_v2(2, Schema::Definition, v2::InstrumentDefMsg::default())]
    #[case::imbalance_v2(2, Schema::Imbalance, v2::ImbalanceMsg::default())]
    #[case::statistics_v2(2, Schema::Statistics, v2::StatMsg::default())]
    #[case::status_v2(2, Schema::Status, v2::StatusMsg::default())]
    #[case::mbo_v3(3, Schema::Mbo, v3::MboMsg::default())]
    #[case::trades_v3(3, Schema::Trades, v3::TradeMsg::default())]
    #[case::tbbo_v3(3, Schema::Tbbo, v3::TbboMsg::default())]
    #[case::mbp1_v3(3, Schema::Mbp1, v3::Mbp1Msg::default())]
    #[case::cmbp1_v3(3, Schema::Cmbp1, v3::Cmbp1Msg::default_for_schema(Schema::Cmbp1))]
    #[case::cbbo1s_v3(3, Schema::Cbbo1S, v3::Cbbo1SMsg::default_for_schema(Schema::Cbbo1S))]
    #[case::bbo1s_v3(3, Schema::Bbo1S, v3::Bbo1SMsg::default_for_schema(Schema::Bbo1S))]
    #[case::bbo1m_v3(3, Schema::Bbo1M, v3::Bbo1MMsg::default_for_schema(Schema::Bbo1M))]
    #[case::mbp10_v3(3, Schema::Mbp10, v3::Mbp10Msg::default())]
    #[case::ohlcv1d_v3(3, Schema::Ohlcv1D, v3::OhlcvMsg::default_for_schema(Schema::Ohlcv1D))]
    #[case::ohlcv1h_v3(3, Schema::Ohlcv1H, v3::OhlcvMsg::default_for_schema(Schema::Ohlcv1H))]
    #[case::ohlcv1m_v3(3, Schema::Ohlcv1M, v3::OhlcvMsg::default_for_schema(Schema::Ohlcv1M))]
    #[case::ohlcv1s_v3(3, Schema::Ohlcv1S, v3::OhlcvMsg::default_for_schema(Schema::Ohlcv1S))]
    #[case::definitions_v3(3, Schema::Definition, v3::InstrumentDefMsg::default())]
    #[case::imbalance_v3(3, Schema::Imbalance, v3::ImbalanceMsg::default())]
    #[case::statistics_v3(3, Schema::Statistics, v3::StatMsg::default())]
    #[case::status_v3(3, Schema::Status, v3::StatusMsg::default())]
    fn test_dbn_identity<R: DbnEncodable + HasRType + PartialEq + Clone>(
        #[case] version: u8,
        #[case] schema: Schema,
        #[case] _rec: R,
    ) -> Result<()> {
        let file_decoder = Decoder::with_upgrade_policy(
            DynReader::from_file(format!(
                "{TEST_DATA_PATH}/test_data.{schema}.v{version}.dbn.zst",
            ))?,
            VersionUpgradePolicy::AsIs,
        )?;
        let file_metadata = file_decoder.metadata().clone();
        let decoded_records = file_decoder.decode_records::<R>()?;
        let mut buffer = Vec::new();

        Encoder::new(
            DynWriter::new(&mut buffer, Compression::Zstd)?,
            &file_metadata,
        )?
        .encode_records(decoded_records.as_slice())?;
        let buf_decoder = Decoder::with_upgrade_policy(
            DynReader::inferred_with_buffer(buffer.as_slice())?,
            VersionUpgradePolicy::AsIs,
        )?;
        assert_eq!(buf_decoder.metadata(), &file_metadata);
        assert_eq!(decoded_records, buf_decoder.decode_records()?);
        Ok(())
    }

    #[test]
    fn test_skip_bytes() {
        let mut decoder =
            Decoder::from_file(format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn")).unwrap();
        decoder
            .decoder
            .skip_bytes(std::mem::size_of::<MboMsg>())
            .unwrap();
        assert!(decoder.decode_record::<MboMsg>().unwrap().is_some());
        assert!(decoder.decode_record::<MboMsg>().unwrap().is_none());
    }

    #[test]
    fn test_dbn_identity_with_ts_out() -> Result<()> {
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
        let mut encoder = DbnRecordEncoder::new(&mut buffer);
        encoder.encode_record(&rec1)?;
        encoder.encode_record(&rec2)?;
        let mut decoder_with = RecordDecoder::new(buffer.as_slice());
        let res1_with = decoder_with
            .decode::<WithTsOut<OhlcvMsg>>()?
            .unwrap()
            .clone();
        let res2_with = decoder_with
            .decode::<WithTsOut<OhlcvMsg>>()?
            .unwrap()
            .clone();
        assert_eq!(rec1, res1_with);
        assert_eq!(rec2, res2_with);
        let mut decoder_without = RecordDecoder::new(buffer.as_slice());
        let res1_without = decoder_without.decode::<OhlcvMsg>()?.unwrap().clone();
        let res2_without = decoder_without.decode::<OhlcvMsg>()?.unwrap().clone();
        assert_eq!(rec1.rec, res1_without);
        assert_eq!(rec2.rec, res2_without);
        Ok(())
    }

    #[test]
    fn test_decode_record_ref() {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(
            &mut buffer,
            &MetadataBuilder::new()
                .dataset(Dataset::XnasItch.to_string())
                .schema(Some(Schema::Mbo))
                .start(0)
                .stype_in(Some(SType::InstrumentId))
                .stype_out(SType::InstrumentId)
                .build(),
        )
        .unwrap();
        const OHLCV_MSG: OhlcvMsg = OhlcvMsg {
            hd: RecordHeader::new::<OhlcvMsg>(rtype::OHLCV_1S, 1, 1, 0),
            open: 100,
            high: 200,
            low: 75,
            close: 125,
            volume: 65,
        };
        let error_msg = ErrorMsg::new(0, None, "Test failed successfully", true);
        encoder.encode_record(&OHLCV_MSG).unwrap();
        encoder.encode_record(&error_msg).unwrap();

        let mut decoder = Decoder::new(buffer.as_slice()).unwrap();
        let ref1 = decoder.decode_record_ref().unwrap().unwrap();
        assert_eq!(*ref1.get::<OhlcvMsg>().unwrap(), OHLCV_MSG);
        let ref2 = decoder.decode_record_ref().unwrap().unwrap();
        assert_eq!(*ref2.get::<ErrorMsg>().unwrap(), error_msg);
        assert!(decoder.decode_record_ref().unwrap().is_none());
    }

    #[test]
    fn test_decode_record_0_length() {
        let buf = vec![0; std::mem::size_of::<RecordHeader>()];
        let mut target = RecordDecoder::new(buf.as_slice());
        assert!(
            matches!(target.decode_ref(), Err(Error::Decode(msg)) if msg.starts_with("invalid record with impossible length"))
        );
    }

    #[test]
    fn test_decode_partial_record() {
        let buf = vec![6u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        assert!(buf[0] as usize * RecordHeader::LENGTH_MULTIPLIER > buf.len());

        let mut target = RecordDecoder::new(buf.as_slice());
        let res = target.decode_ref();
        dbg!(&res);
        assert!(matches!(res, Ok(None)));
    }

    #[test]
    fn test_decode_record_length_less_than_header() {
        let buf = vec![3u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let mut target = RecordDecoder::new(buf.as_slice());
        assert!(
            matches!(target.decode_ref(), Err(Error::Decode(msg)) if msg.starts_with("invalid record with impossible length"))
        );
    }

    #[test]
    fn test_decode_record_length_longer_than_buffer() {
        let rec = ErrorMsg::new(1680703198000000000, None, "Test", true);
        let mut target = RecordDecoder::new(&rec.as_ref()[..rec.record_size() - 1]);
        assert!(matches!(target.decode_ref(), Ok(None)));
    }

    #[rstest]
    #[case::v1_as_is(v1::InstrumentDefMsg::default(), VersionUpgradePolicy::AsIs)]
    #[case::v1_upgrade(v1::InstrumentDefMsg::default(), VersionUpgradePolicy::UpgradeToV2)]
    fn test_decode_multiframe_zst_from_v1<R: HasRType>(
        #[case] _r: R,
        #[case] upgrade_policy: VersionUpgradePolicy,
    ) {
        let mut decoder = RecordDecoder::with_version(
            zstd::stream::Decoder::new(
                File::open(format!(
                    "{TEST_DATA_PATH}/multi-frame.definition.v1.dbn.frag.zst"
                ))
                .unwrap(),
            )
            .unwrap(),
            1,
            upgrade_policy,
            false,
        )
        .unwrap();
        let mut count = 0;
        while let Some(_rec) = decoder.decode::<R>().unwrap() {
            count += 1;
        }
        assert_eq!(count, 8);
    }

    #[test]
    fn test_decode_upgrade_v2() -> crate::Result<()> {
        let decoder = Decoder::with_upgrade_policy(
            zstd::Decoder::new(
                File::open(format!("{TEST_DATA_PATH}/test_data.definition.v1.dbn.zst")).unwrap(),
            )
            .unwrap(),
            VersionUpgradePolicy::UpgradeToV2,
        )?;
        assert_eq!(decoder.metadata().version, 2);
        assert_eq!(decoder.metadata().symbol_cstr_len, crate::SYMBOL_CSTR_LEN);
        decoder.decode_records::<v2::InstrumentDefMsg>()?;
        Ok(())
    }

    #[test]
    fn test_decode_upgrade_v3() -> crate::Result<()> {
        let decoder = Decoder::with_upgrade_policy(
            zstd::Decoder::new(
                File::open(format!("{TEST_DATA_PATH}/test_data.definition.v1.dbn.zst")).unwrap(),
            )
            .unwrap(),
            VersionUpgradePolicy::UpgradeToV3,
        )?;
        assert_eq!(decoder.metadata().version, crate::DBN_VERSION);
        assert_eq!(decoder.metadata().symbol_cstr_len, crate::SYMBOL_CSTR_LEN);
        decoder.decode_records::<v3::InstrumentDefMsg>()?;
        Ok(())
    }
}
