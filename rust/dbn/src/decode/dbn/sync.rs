#[cfg(feature = "zstd")]
use std::io::BufReader;
use std::{fs::File, io, path::Path};

use crate::{
    decode::{
        dbn::fsm::{DbnFsm, DecodeHeader, ProcessResult},
        private::LastRecord,
        DbnMetadata, DecodeRecord, DecodeRecordRef, DecodeStream, SkipBytes, StreamIterDecoder,
        VersionUpgradePolicy,
    },
    v3, Metadata, RecordRef, DBN_VERSION,
};

/// Type for decoding files and streams in Databento Binary Encoding (DBN), both metadata and records.
///
/// `H` is the record header type to decode. See [`DbnFsm`].
pub struct Decoder<R, H: DecodeHeader = v3::RecordHeader> {
    metadata: Metadata,
    decoder: RecordDecoder<R, H>,
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
}

impl<R, H> Decoder<R, H>
where
    R: io::Read,
    H: DecodeHeader,
{
    /// Creates a new DBN [`Decoder`] from `reader` that decodes with `fsm`, which
    /// determines the record header and DBN versions it reads.
    ///
    /// # Errors
    /// This function will return an error if it is unable to parse the metadata in
    /// `reader`, or `fsm` can't decode its DBN version.
    pub fn with_fsm(reader: R, fsm: DbnFsm<H>) -> crate::Result<Self> {
        let mut metadata_decoder = MetadataDecoder { reader, fsm };
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

    /// Tries to decode a generic reference to a record. Returns `Ok(None)` if
    /// the reader is exhausted.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an
    /// error of a kind other than `io::ErrorKind::UnexpectedEof` upon reading.
    /// It will also return an error if it encounters an invalid record.
    pub fn decode_ref(&mut self) -> crate::Result<Option<RecordRef<'_, H>>> {
        self.decoder.decode_ref()
    }
}

#[cfg(feature = "zstd")]
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

#[cfg(feature = "zstd")]
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

#[cfg(feature = "zstd")]
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
    fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef<'_>>> {
        self.decoder.decode_record_ref()
    }
}

impl<R, H: DecodeHeader> DbnMetadata for Decoder<R, H> {
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
    fn decode_record<T: v3::HasRType>(&mut self) -> crate::Result<Option<&T>> {
        self.decoder.decode()
    }
}

impl<R> DecodeStream for Decoder<R>
where
    R: io::Read,
{
    fn decode_stream<T: v3::HasRType>(self) -> StreamIterDecoder<Self, T> {
        StreamIterDecoder::new(self)
    }
}

impl<R> LastRecord for Decoder<R>
where
    R: io::Read,
{
    fn last_record(&self) -> Option<RecordRef<'_>> {
        self.decoder.last_record()
    }
}

impl<R, H: DecodeHeader> std::fmt::Debug for Decoder<R, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Decoder")
            .field("metadata", &self.metadata)
            .field("decoder", &self.decoder)
            .finish()
    }
}

/// A DBN decoder of records
///
/// `H` is the record header to decode. See [`DbnFsm`].
pub struct RecordDecoder<R, H: DecodeHeader = v3::RecordHeader> {
    reader: R,
    fsm: DbnFsm<H>,
}

impl<R, H> From<MetadataDecoder<R, H>> for RecordDecoder<R, H>
where
    R: io::Read,
    H: DecodeHeader,
{
    fn from(metadata_decoder: MetadataDecoder<R, H>) -> Self {
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

    /// Tries to decode the next record of type `T`. Returns `Ok(None)` if
    /// the reader is exhausted.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an
    /// error of a kind other than `io::ErrorKind::UnexpectedEof` upon reading.
    ///
    /// If the next record is of a different type than `T`,
    /// this function returns an error of kind `io::ErrorKind::InvalidData`.
    pub fn decode<T: v3::HasRType>(&mut self) -> crate::Result<Option<&T>> {
        self.decode_ref().and_then(|rec| {
            if let Some(rec) = rec {
                rec.try_get().map(Some)
            } else {
                Ok(None)
            }
        })
    }
}

impl<R, H> RecordDecoder<R, H>
where
    R: io::Read,
    H: DecodeHeader,
{
    /// Creates a new `RecordDecoder` that will decode from `reader` with `fsm`, which
    /// determines the record header and DBN versions it reads.
    pub fn with_fsm(reader: R, fsm: DbnFsm<H>) -> Self {
        Self { reader, fsm }
    }

    /// Sets the DBN version to expect when decoding.
    ///
    /// # Errors
    /// This function will return an error if the `version` isn't one `H` can decode
    /// or the `version` and `upgrade_policy` are incompatible.
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

    /// Tries to decode a generic reference a record. Returns `Ok(None)` if
    /// the reader is exhausted.
    ///
    /// # Errors
    /// This function returns an error if the underlying reader returns an
    /// error of a kind other than `io::ErrorKind::UnexpectedEof` upon reading.
    /// It will also return an error if it encounters an invalid record.
    pub fn decode_ref(&mut self) -> crate::Result<Option<RecordRef<'_, H>>> {
        while !self.fsm.has_buffered_record() {
            match self.fsm.process_batch() {
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
                ProcessResult::Record(_) => (),
                ProcessResult::Err(error) => return Err(error),
                ProcessResult::Metadata(_) => unreachable!("skipped metadata"),
            }
        }
        Ok(self.fsm.next_buffered_record())
    }
}

impl<R> DecodeRecord for RecordDecoder<R>
where
    R: io::Read,
{
    fn decode_record<T: v3::HasRType>(&mut self) -> crate::Result<Option<&T>> {
        self.decode()
    }
}

impl<R> DecodeRecordRef for RecordDecoder<R>
where
    R: io::Read,
{
    fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef<'_>>> {
        self.decode_ref()
    }
}

impl<R, H> RecordDecoder<R, H>
where
    R: io::Read + io::Seek,
    H: DecodeHeader,
{
    /// Seeks to absolute byte position `pos` and clears buffered decoder state.
    ///
    /// # Warning
    /// Callers are responsible for ensuring `pos` is a valid record boundary.
    ///
    /// # Errors
    /// This function returns an error if it fails to seek in the inner reader.
    pub fn seek_to(&mut self, pos: u64) -> crate::Result<()> {
        self.reader
            .seek(io::SeekFrom::Start(pos))
            .map(drop)
            .map_err(|err| crate::Error::io(err, format!("seeking to byte offset {pos}")))?;
        self.fsm.reset_for_seek();
        Ok(())
    }
}

#[cfg(feature = "zstd")]
impl<'a, R> RecordDecoder<crate::decode::DynReader<'a, R>>
where
    R: io::BufRead,
{
    /// Seeks to absolute byte position `pos` and clears buffered decoder state.
    ///
    /// # Warning
    /// Callers are responsible for ensuring `pos` is a valid record boundary.
    ///
    /// # Errors
    /// This function returns an error if it fails to seek in the inner reader, or if
    /// the input is Zstandard-compressed.
    pub fn seek_to(&mut self, pos: u64) -> crate::Result<()>
    where
        R: io::Seek,
    {
        if self.reader.is_compressed() {
            return Err(crate::Error::BadArgument {
                param_name: "self".to_owned(),
                desc: "absolute seek is unsupported for zstd-compressed input".to_owned(),
            });
        }
        self.reader
            .get_mut()
            .seek(io::SeekFrom::Start(pos))
            .map(drop)
            .map_err(|err| crate::Error::io(err, format!("seeking to byte offset {pos}")))?;
        self.fsm.reset_for_seek();
        Ok(())
    }

    /// Returns whether the input is Zstandard-compressed.
    pub fn is_compressed(&self) -> bool {
        self.reader.is_compressed()
    }
}

impl<R> LastRecord for RecordDecoder<R>
where
    R: io::Read,
{
    fn last_record(&self) -> Option<RecordRef<'_>> {
        self.fsm.last_record()
    }
}

impl<R, H> SkipBytes for RecordDecoder<R, H>
where
    R: SkipBytes,
    H: DecodeHeader,
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

impl<R, H: DecodeHeader> std::fmt::Debug for RecordDecoder<R, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordDecoder")
            .field("fsm", &self.fsm)
            .finish_non_exhaustive()
    }
}

/// Type for decoding [`Metadata`] from Databento Binary Encoding (DBN).
pub struct MetadataDecoder<R, H: DecodeHeader = v3::RecordHeader>
where
    R: io::Read,
{
    reader: R,
    fsm: DbnFsm<H>,
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
}

impl<R, H> MetadataDecoder<R, H>
where
    R: io::Read,
    H: DecodeHeader,
{
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
                let mut total_read = 0;
                loop {
                    // `fsm.space()` reclaims the consumed prefix as needed
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

impl<R, H: DecodeHeader> std::fmt::Debug for MetadataDecoder<R, H>
where
    R: io::Read,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetadataDecoder")
            .field("fsm", &self.fsm)
            .finish_non_exhaustive()
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

    #[cfg(feature = "zstd")]
    use std::fs::File;

    use rstest::rstest;

    use super::*;
    #[cfg(feature = "zstd")]
    use crate::{
        decode::DynReader,
        encode::{DbnEncodable, DynWriter},
        v1, v2, v3, Compression,
    };
    use crate::{
        decode::{
            dbn::tests::{
                fsm_v4, mbo_v4, metadata_v4, replace_layout, trade_v3, trade_v3_upgraded, trade_v4,
                trades_v4,
            },
            tests::TEST_DATA_PATH,
        },
        encode::{dbn::Encoder, DbnRecordEncoder, EncodeRecord, EncodeRecordRef},
        rtype, Dataset, Error, ErrorMsg, HasRType, MboMsg, MetadataBuilder, OhlcvMsg, Record,
        RecordBuf, RecordHeader, Result, SType, Schema, TradeMsg, WithTsOut,
    };
    use crate::{
        layout::{DynFieldAccess, Verdict},
        record::str_to_c_chars,
        v4::{
            fields,
            types::{Decimal, TimestampNs},
        },
        InstrumentClass, MatchAlgorithm, Side,
    };
    use std::ffi::c_char;

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
    #[cfg(feature = "zstd")]
    fn test_dbn_identity<R: DbnEncodable + v3::HasRType + PartialEq + Clone>(
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

    #[rstest]
    #[case::mbo(Schema::Mbo, mbo_v4())]
    #[case::trades(Schema::Trades, trade_v4())]
    fn test_dbn_identity_v4<R>(#[case] schema: Schema, #[case] rec: R) -> Result<()>
    where
        R: HasRType<Header = crate::v4::RecordHeader> + PartialEq + Clone + std::fmt::Debug,
    {
        let metadata = metadata_v4(schema);

        let collect = |buf: &[u8]| -> Result<(Metadata, Vec<R>)> {
            let mut decoder = crate::decode::dbn::v4::Decoder::with_fsm(buf, fsm_v4()?)?;
            let decoded_metadata = decoder.metadata().clone();
            let mut records = Vec::new();
            while let Some(rec_ref) = decoder.decode_ref()? {
                records.push(rec_ref.try_get::<R>()?.clone());
            }
            Ok((decoded_metadata, records))
        };

        let mut encoded = Vec::new();
        crate::encode::dbn::v4::Encoder::new(&mut encoded, &metadata)?.encode(&rec)?;
        let (decoded_metadata, decoded) = collect(encoded.as_slice())?;
        assert_eq!(decoded_metadata, metadata);
        assert_eq!(decoded, vec![rec]);

        let mut reencoded = Vec::new();
        {
            let mut encoder =
                crate::encode::dbn::v4::Encoder::new(&mut reencoded, &decoded_metadata)?;
            for rec in &decoded {
                encoder.encode(rec)?;
            }
        }
        assert_eq!(reencoded, encoded);
        Ok(())
    }

    #[rstest]
    fn test_v3_decoder_refuses_v4_input(
        #[values(
            VersionUpgradePolicy::AsIs,
            VersionUpgradePolicy::UpgradeToV2,
            VersionUpgradePolicy::UpgradeToV3,
            VersionUpgradePolicy::ExpectV4
        )]
        upgrade_policy: VersionUpgradePolicy,
    ) -> Result<()> {
        let encoded = trades_v4()?;
        let err = Decoder::with_upgrade_policy(encoded.as_slice(), upgrade_policy).unwrap_err();
        assert!(
            err.to_string()
                .contains("can't decode DBN version 4 with a decoder for DBN versions 1 to 3"),
            "{err}"
        );

        let mut fsm = DbnFsm::builder().upgrade_policy(upgrade_policy).build()?;
        fsm.write_all(&encoded);
        let mut recs = [const { None }; 4];
        assert!(matches!(fsm.process_many(&mut recs), ProcessResult::Err(_)));
        Ok(())
    }

    #[test]
    fn test_v3_record_decoder_refuses_v4() {
        assert!(RecordDecoder::with_version(
            [].as_slice(),
            crate::v4::DBN_VERSION,
            VersionUpgradePolicy::AsIs,
            false
        )
        .is_err());
        let mut decoder = RecordDecoder::new([].as_slice());
        assert!(decoder.set_version(crate::v4::DBN_VERSION).is_err());
    }

    fn trades_v3_metadata() -> Metadata {
        MetadataBuilder::new()
            .dataset(Dataset::XnasItch)
            .schema(Some(Schema::Trades))
            .start(0)
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .build()
    }

    fn trades_v3() -> Result<Vec<u8>> {
        let mut encoded = Vec::new();
        Encoder::new(&mut encoded, &trades_v3_metadata())?.encode_record(&trade_v3())?;
        Ok(encoded)
    }

    #[test]
    fn test_v3_decoder_refuses_upgrade_to_v4() -> Result<()> {
        let err = Decoder::with_upgrade_policy(
            trades_v3()?.as_slice(),
            VersionUpgradePolicy::UpgradeToV4,
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("can't upgrade to DBN version 4 with a decoder for DBN versions 1 to 3"),
            "{err}"
        );
        Ok(())
    }

    #[rstest]
    #[case::as_is(
        VersionUpgradePolicy::AsIs,
        "`VersionUpgradePolicy::AsIs` can't decode DBN version 3 with a decoder for DBN version 4"
    )]
    #[case::expect_v4(
        VersionUpgradePolicy::ExpectV4,
        "`VersionUpgradePolicy::ExpectV4` requires DBN version 4 input, got version 3"
    )]
    fn test_v4_decoder_refuses_v3_input(
        #[case] upgrade_policy: VersionUpgradePolicy,
        #[case] expected: &str,
    ) -> Result<()> {
        let fsm = DbnFsm::builder()
            .header::<crate::v4::RecordHeader>()
            .upgrade_policy(upgrade_policy)
            .build()?;
        let err =
            crate::decode::dbn::v4::Decoder::with_fsm(trades_v3()?.as_slice(), fsm).unwrap_err();
        assert!(err.to_string().contains(expected), "{err}");
        Ok(())
    }

    #[rstest]
    #[cfg(feature = "zstd")]
    fn test_v4_decoder_upgrades_tbbo_rtype(#[values(1, 2, 3)] version: u8) -> Result<()> {
        let mut decoder = crate::decode::dbn::v4::Decoder::with_fsm(
            DynReader::from_file(format!(
                "{TEST_DATA_PATH}/test_data.tbbo.v{version}.dbn.zst"
            ))?,
            fsm_v4()?,
        )?;
        let mut count = 0;
        while let Some(rec) = decoder.decode_ref()? {
            assert_eq!(rec.header().rtype, crate::v4::rtype::TBBO);
            count += 1;
        }
        assert!(count > 0);
        Ok(())
    }

    #[test]
    fn test_v4_decoder_upgrades_v3_input() -> Result<()> {
        let encoded = trades_v3()?;
        let mut decoder = crate::decode::dbn::v4::Decoder::with_fsm(encoded.as_slice(), fsm_v4()?)?;
        assert_eq!(decoder.metadata().version, crate::v4::DBN_VERSION);
        let rec = decoder.decode_ref()?.unwrap();
        assert_eq!(*rec.try_get::<crate::v4::TradeMsg>()?, trade_v3_upgraded());
        assert!(decoder.decode_ref()?.is_none());
        Ok(())
    }

    fn es_call_v3() -> v3::InstrumentDefMsg {
        v3::InstrumentDefMsg {
            hd: RecordHeader::new::<v3::InstrumentDefMsg>(
                rtype::INSTRUMENT_DEF,
                1,
                42_005_804,
                1_757_970_000_000_000_000,
            ),
            ts_recv: 1_757_970_000_000_000_100,
            min_price_increment: 250_000_000,
            display_factor: 1_000_000_000,
            expiration: 1_766_154_600_000_000_000,
            strike_price: 6_800_000_000_000,
            raw_instrument_id: 42_005_804,
            min_lot_size_round_lot: 1,
            channel_id: 13,
            currency: str_to_c_chars("USD").unwrap(),
            raw_symbol: str_to_c_chars("ESZ5 C6800").unwrap(),
            exchange: str_to_c_chars("XCME").unwrap(),
            asset: str_to_c_chars("ES").unwrap(),
            underlying: str_to_c_chars("ESZ5").unwrap(),
            instrument_class: InstrumentClass::Call as c_char,
            match_algorithm: MatchAlgorithm::Fifo as c_char,
            ..Default::default()
        }
    }

    fn es_call_v1() -> crate::v1::InstrumentDefMsg {
        let v3 = es_call_v3();
        crate::v1::InstrumentDefMsg {
            hd: v3.hd,
            ts_recv: v3.ts_recv,
            min_price_increment: v3.min_price_increment,
            display_factor: v3.display_factor,
            expiration: v3.expiration,
            strike_price: v3.strike_price,
            raw_instrument_id: 42_005_804,
            min_lot_size_round_lot: v3.min_lot_size_round_lot,
            channel_id: v3.channel_id,
            currency: v3.currency,
            raw_symbol: str_to_c_chars("ESZ5 C6800").unwrap(),
            exchange: v3.exchange,
            asset: str_to_c_chars("ES").unwrap(),
            underlying: v3.underlying,
            instrument_class: v3.instrument_class,
            match_algorithm: v3.match_algorithm,
            ..Default::default()
        }
    }

    fn definitions<R: crate::encode::DbnEncodable>(version: u8, recs: &[R]) -> Vec<u8> {
        let metadata = MetadataBuilder::new()
            .version(version)
            .dataset(Dataset::GlbxMdp3)
            .schema(Some(Schema::Definition))
            .start(1_757_970_000_000_000_000)
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .build();
        let mut encoded = Vec::new();
        let mut encoder = Encoder::new(&mut encoded, &metadata).unwrap();
        for rec in recs {
            encoder.encode_record(rec).unwrap();
        }
        encoded
    }

    #[rstest]
    #[case::v1(definitions(1, &[es_call_v1()]))]
    #[case::v3(definitions(3, &[es_call_v3()]))]
    fn test_v4_decoder_keeps_every_definition_field(#[case] encoded: Vec<u8>) {
        let mut decoder =
            crate::decode::dbn::v4::Decoder::with_fsm(encoded.as_slice(), fsm_v4().unwrap())
                .unwrap();
        let metadata = decoder.metadata().clone();
        let index = metadata.layout_index().unwrap();
        assert_eq!(
            index.verdict(crate::v4::rtype::INSTRUMENT_DEF),
            Some(Verdict::Compiled)
        );
        let rec = decoder.decode_ref().unwrap().unwrap();

        let def = rec.try_get::<crate::v4::InstrumentDefMsg>().unwrap();
        assert_eq!(def.hd.instrument_id, 42_005_804);
        assert_eq!(def.raw_instrument_id, 42_005_804);
        assert_eq!(def.min_price_increment, 250_000_000);
        assert_eq!(def.instrument_class, InstrumentClass::Call as c_char);
        assert_eq!(def.channel_id, 13);
        assert_eq!(rec.field(&index, fields::RAW_SYMBOL), Some("ESZ5 C6800"));
        assert_eq!(
            rec.field(&index, fields::STRIKE_PRICE),
            Some(Decimal {
                raw: 6_800_000_000_000,
                scale: -9
            })
        );
        assert_eq!(
            rec.field(&index, fields::EXPIRATION),
            Some(TimestampNs(1_766_154_600_000_000_000))
        );
        assert_eq!(rec.field(&index, fields::CURRENCY), Some("USD"));
        assert_eq!(rec.field(&index, fields::EXCHANGE), Some("XCME"));
        assert_eq!(rec.field(&index, fields::ASSET), Some("ES"));
        assert_eq!(rec.field(&index, fields::UNDERLYING), Some("ESZ5"));
        assert_eq!(rec.field(&index, fields::LEG_COUNT), Some(0));
        assert_eq!(rec.field(&index, fields::LEG_RAW_SYMBOL), Some(""));

        let mut json = Vec::new();
        let mut encoder =
            crate::encode::json::v4::Encoder::new(&mut json, &metadata, false, false, false)
                .unwrap();
        encoder.encode_record_ref(rec).unwrap();
        let json = String::from_utf8(json).unwrap();
        assert!(json.contains(r#""currency":"USD""#), "{json}");
        assert!(json.contains(r#""underlying":"ESZ5""#), "{json}");
    }

    #[test]
    fn test_v4_decoder_keeps_v3_definition_legs() {
        let spread_leg = v3::InstrumentDefMsg {
            hd: RecordHeader::new::<v3::InstrumentDefMsg>(
                rtype::INSTRUMENT_DEF,
                1,
                42_140_878,
                1_757_970_000_000_000_000,
            ),
            raw_instrument_id: 42_140_878,
            raw_symbol: str_to_c_chars("UD:1V: VT 0915976518").unwrap(),
            instrument_class: InstrumentClass::OptionSpread as c_char,
            leg_count: 2,
            leg_index: 1,
            leg_instrument_id: 42_005_804,
            leg_raw_symbol: str_to_c_chars("ESZ5 C6800").unwrap(),
            leg_instrument_class: InstrumentClass::Call as c_char,
            leg_side: Side::Ask as c_char,
            leg_ratio_qty_numerator: 1,
            leg_ratio_qty_denominator: 1,
            ..es_call_v3()
        };
        let encoded = definitions(3, &[spread_leg]);
        let mut decoder =
            crate::decode::dbn::v4::Decoder::with_fsm(encoded.as_slice(), fsm_v4().unwrap())
                .unwrap();
        let index = decoder.metadata().layout_index().unwrap();
        let rec = decoder.decode_ref().unwrap().unwrap();

        assert_eq!(rec.field(&index, fields::LEG_COUNT), Some(2));
        assert_eq!(rec.field(&index, fields::LEG_INDEX), Some(1));
        assert_eq!(
            rec.field(&index, fields::LEG_INSTRUMENT_ID),
            Some(42_005_804)
        );
        assert_eq!(
            rec.field(&index, fields::LEG_RAW_SYMBOL),
            Some("ESZ5 C6800")
        );
        assert_eq!(
            rec.field(&index, fields::LEG_SIDE).unwrap().get().unwrap(),
            Side::Ask
        );
    }

    #[test]
    fn test_v4_decoder_upgrades_v3_ts_out() -> Result<()> {
        const TS_OUT: u64 = 1_697_240_529_000_000_200;
        let mut metadata = trades_v3_metadata();
        metadata.ts_out = true;
        let mut encoded = Vec::new();
        Encoder::new(&mut encoded, &metadata)?
            .encode_record(&WithTsOut::new(trade_v3(), TS_OUT))?;
        let mut decoder = crate::decode::dbn::v4::Decoder::with_fsm(encoded.as_slice(), fsm_v4()?)?;
        let index = decoder.metadata().layout_index().unwrap();
        let rec = decoder.decode_ref()?.unwrap();

        let trade = rec.try_get::<crate::v4::TradeMsg>()?;
        assert_eq!(trade.price, trade_v4().price);
        assert_eq!(rec.field(&index, fields::TS_OUT), Some(TimestampNs(TS_OUT)));
        Ok(())
    }

    #[test]
    fn test_v4_record_decoder_upgrades_v3_ts_out_fragment() -> Result<()> {
        const TS_OUT: u64 = 1_697_240_529_000_000_200;
        let fsm = DbnFsm::builder()
            .header::<crate::v4::RecordHeader>()
            .skip_metadata(true)
            .input_dbn_version(Some(3))?
            .ts_out(true)
            .build()?;
        let trade = WithTsOut::new(trade_v3(), TS_OUT);
        let mut decoder = crate::decode::dbn::v4::RecordDecoder::with_fsm(trade.as_ref(), fsm);
        let rec = decoder.decode_ref()?.unwrap();
        let index = crate::layout::LayoutIndex::new(
            &crate::layout::upgrade_layout(true),
            crate::v4::layouts::default_layout(),
        );

        assert_eq!(rec.field(&index, fields::TS_OUT), Some(TimestampNs(TS_OUT)));
        Ok(())
    }

    #[test]
    fn test_v4_record_decoder_upgrades_v3_fragment() -> Result<()> {
        let fsm = DbnFsm::builder()
            .header::<crate::v4::RecordHeader>()
            .skip_metadata(true)
            .input_dbn_version(Some(3))?
            .build()?;
        let trade = trade_v3();
        let mut decoder = crate::decode::dbn::v4::RecordDecoder::with_fsm(trade.as_ref(), fsm);
        let rec = decoder.decode_ref()?.unwrap();
        assert_eq!(*rec.try_get::<crate::v4::TradeMsg>()?, trade_v3_upgraded());
        assert!(decoder.decode_ref()?.is_none());
        Ok(())
    }

    #[test]
    fn test_v4_decoder_keeps_body_without_a_layout() -> Result<()> {
        #[repr(C)]
        struct ClockMsg {
            hd: RecordHeader,
            ts_now: u64,
            ts_delta: i64,
            channel_id: u16,
            _reserved: [u8; 6],
        }
        let clock = ClockMsg {
            hd: RecordHeader {
                length: (size_of::<ClockMsg>() / RecordHeader::LENGTH_MULTIPLIER) as u8,
                rtype: 0xAD,
                publisher_id: 1,
                instrument_id: 5482,
                ts_event: 1_697_240_529_000_000_000,
            },
            ts_now: 1_697_240_529_000_000_250,
            ts_delta: 250,
            channel_id: 3,
            _reserved: [0; 6],
        };
        // SAFETY: `ClockMsg` is a `repr(C)` record of plain data
        let clock_bytes = unsafe {
            std::slice::from_raw_parts((&raw const clock).cast::<u8>(), size_of::<ClockMsg>())
        };
        let mut encoded = Vec::new();
        Encoder::new(&mut encoded, &trades_v3_metadata())?
            .encode_record_ref(unsafe { RecordRef::new(clock_bytes) })?;

        let mut decoder = crate::decode::dbn::v4::Decoder::with_fsm(encoded.as_slice(), fsm_v4()?)?;
        let rec = decoder.decode_ref()?.unwrap();
        let hd = rec.header();
        assert_eq!(
            (hd.rtype, hd.publisher_id, hd.instrument_id, hd.ts_event),
            (0xAD, 1, 5482, 1_697_240_529_000_000_000)
        );
        let header_len = size_of::<crate::v4::RecordHeader>();
        assert_eq!(
            rec.record_size(),
            header_len + size_of::<ClockMsg>() - size_of::<RecordHeader>()
        );
        assert_eq!(
            rec.as_ref()[header_len..],
            clock_bytes[size_of::<RecordHeader>()..]
        );
        Ok(())
    }

    #[test]
    fn test_v4_decoder_defaults_to_upgrade_to_v4() -> Result<()> {
        assert_eq!(
            fsm_v4()?.upgrade_policy(),
            VersionUpgradePolicy::UpgradeToV4
        );
        Ok(())
    }

    #[test]
    fn test_v4_decoder_refuses_missing_layout() -> Result<()> {
        let without_layout = replace_layout(&trades_v4()?, &[]);
        let err = crate::decode::dbn::v4::Decoder::with_fsm(without_layout.as_slice(), fsm_v4()?)
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("a DBN version 4 stream must carry a stream layout"),
            "{err}"
        );
        Ok(())
    }

    #[test]
    fn test_v4_decoder_refuses_layout_missing_required_field() -> Result<()> {
        let mut layout = crate::v4::layouts::default_layout().clone();
        let definition = layout
            .record_layouts
            .iter_mut()
            .find(|rec| rec.rtype == crate::v4::rtype::INSTRUMENT_DEF)
            .unwrap();
        // Blank out `raw_symbol`, leaving the byte coverage intact
        let raw_symbol = definition
            .field_defs
            .iter_mut()
            .find(|f| f.field_id == crate::v4::fields::RAW_SYMBOL.id())
            .unwrap();
        raw_symbol.field_id = crate::v4::fields::PADDING;
        raw_symbol.type_id = crate::v4::fields::PADDING_ID;
        definition.layout_hash = definition.layout_hash(&layout.struct_layouts);
        let mut layout_bytes = Vec::new();
        crate::layout::wire::encode(&mut layout_bytes, &layout)?;

        let invalid = replace_layout(&trades_v4()?, &layout_bytes);
        let err =
            crate::decode::dbn::v4::Decoder::with_fsm(invalid.as_slice(), fsm_v4()?).unwrap_err();
        assert!(
            err.to_string()
                .contains("missing required field `raw_symbol`"),
            "{err}"
        );
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
    fn test_seek_to_resets_buffered_records() {
        let mut first = MboMsg {
            hd: RecordHeader::new::<MboMsg>(rtype::MBO, 1, 100, 1),
            ..Default::default()
        };
        first.order_id = 1;
        let mut second = MboMsg {
            hd: RecordHeader::new::<MboMsg>(rtype::MBO, 1, 101, 2),
            ..Default::default()
        };
        second.order_id = 2;
        let mut buffer = Vec::new();
        buffer.extend_from_slice(first.as_ref());
        buffer.extend_from_slice(second.as_ref());

        let mut decoder = RecordDecoder::with_version(
            std::io::Cursor::new(buffer),
            DBN_VERSION,
            VersionUpgradePolicy::AsIs,
            false,
        )
        .unwrap();

        assert_eq!(*decoder.decode::<MboMsg>().unwrap().unwrap(), first);
        decoder
            .seek_to(std::mem::size_of::<MboMsg>() as u64)
            .unwrap();
        assert_eq!(*decoder.decode::<MboMsg>().unwrap().unwrap(), second);
        decoder.seek_to(0).unwrap();
        assert_eq!(*decoder.decode::<MboMsg>().unwrap().unwrap(), first);
    }

    #[test]
    #[cfg(feature = "zstd")]
    fn test_seek_to_compressed_returns_bad_argument() {
        let reader =
            DynReader::from_file(format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn.zst")).unwrap();
        let mut decoder =
            RecordDecoder::with_version(reader, DBN_VERSION, VersionUpgradePolicy::AsIs, false)
                .unwrap();

        assert!(matches!(
            decoder.seek_to(0).unwrap_err(),
            Error::BadArgument { param_name, desc }
                if param_name == "self" && desc.contains("zstd-compressed")
        ));
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
    fn test_last_record_matches_decode_ref() {
        use crate::decode::private::LastRecord;

        // More than one batch worth of records
        const REC_COUNT: u32 = 40;
        let mut buffer = Vec::new();
        let mut encoder = DbnRecordEncoder::new(&mut buffer);
        for instrument_id in 1..=REC_COUNT {
            encoder
                .encode_record(&TradeMsg {
                    hd: RecordHeader::new::<TradeMsg>(rtype::MBP_0, 1, instrument_id, 0),
                    ..Default::default()
                })
                .unwrap();
        }

        let mut decoder = RecordDecoder::new(buffer.as_slice());
        for instrument_id in 1..=REC_COUNT {
            let decoded = decoder
                .decode_ref()
                .unwrap()
                .unwrap()
                .get::<TradeMsg>()
                .unwrap()
                .clone();
            assert_eq!(decoded.hd.instrument_id, instrument_id);
            let last = decoder.last_record().unwrap().get::<TradeMsg>().unwrap();
            assert_eq!(&decoded, last);
        }
        assert!(decoder.decode_ref().unwrap().is_none());
    }

    #[test]
    fn test_skip_bytes_between_records() {
        const REC_COUNT: u32 = 8;
        // Padding that decodes as a record of its own length, so it lands in the batch
        const PAD: [u8; size_of::<TradeMsg>()] = [(size_of::<TradeMsg>()
            / RecordHeader::LENGTH_MULTIPLIER)
            as u8; size_of::<TradeMsg>()];
        let mut buffer = Vec::new();
        let mut encoder = DbnRecordEncoder::new(&mut buffer);
        for instrument_id in 1..=REC_COUNT {
            encoder
                .encode_record(&TradeMsg {
                    hd: RecordHeader::new::<TradeMsg>(rtype::MBP_0, 1, instrument_id, 0),
                    ..Default::default()
                })
                .unwrap();
            encoder.get_mut().extend_from_slice(&PAD);
        }

        let mut decoder = RecordDecoder::new(std::io::Cursor::new(buffer));
        for instrument_id in 1..=REC_COUNT {
            let decoded = decoder
                .decode_ref()
                .unwrap()
                .unwrap()
                .get::<TradeMsg>()
                .unwrap();
            assert_eq!(decoded.hd.instrument_id, instrument_id);
            decoder.skip_bytes(PAD.len()).unwrap();
        }
        assert!(decoder.decode_ref().unwrap().is_none());
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
    #[cfg(feature = "zstd")]
    fn test_decode_multiframe_zst_from_v1<R: v3::HasRType>(
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
    #[cfg(feature = "zstd")]
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
    #[cfg(feature = "zstd")]
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

    #[test]
    fn test_record_decoder_past_capacity() {
        const N: u64 = 5_000;
        let mut buf = Vec::new();
        {
            let mut encoder = DbnRecordEncoder::new(&mut buf);
            for i in 0..N {
                let mut rec = MboMsg::default();
                rec.hd.ts_event = i;
                encoder.encode_record(&rec).unwrap();
            }
        }
        assert!(buf.len() > DbnFsm::DEFAULT_BUF_SIZE * 2);
        let mut decoder = RecordDecoder::new(buf.as_slice());
        let mut count: u64 = 0;
        while let Some(rec) = decoder.decode::<MboMsg>().unwrap() {
            assert_eq!(rec.hd.ts_event, count);
            count += 1;
        }
        assert_eq!(count, N);
    }

    #[rstest]
    #[case::mbo(Schema::Mbo, v3::MboMsg::default())]
    #[case::trades(Schema::Trades, v3::TradeMsg::default())]
    #[case::tbbo(Schema::Tbbo, v3::TbboMsg::default())]
    #[case::mbp1(Schema::Mbp1, v3::Mbp1Msg::default())]
    #[case::mbp10(Schema::Mbp10, v3::Mbp10Msg::default())]
    #[case::ohlcv1m(Schema::Ohlcv1M, v3::OhlcvMsg::default_for_schema(Schema::Ohlcv1M))]
    #[case::definitions(Schema::Definition, v3::InstrumentDefMsg::default())]
    #[case::imbalance(Schema::Imbalance, v3::ImbalanceMsg::default())]
    #[case::statistics(Schema::Statistics, v3::StatMsg::default())]
    #[case::status(Schema::Status, v3::StatusMsg::default())]
    #[cfg(feature = "zstd")]
    fn test_decode_buf_iter<R: DbnEncodable + v3::HasRType + PartialEq + Clone>(
        #[case] schema: Schema,
        #[case] _rec: R,
    ) -> Result<()> {
        let expected =
            Decoder::from_zstd_file(format!("{TEST_DATA_PATH}/test_data.{schema}.v3.dbn.zst"))?
                .decode_records::<R>()?;
        let mut decoder =
            Decoder::from_zstd_file(format!("{TEST_DATA_PATH}/test_data.{schema}.v3.dbn.zst"))?;
        let buf_results: Vec<RecordBuf> = decoder.decode_buf_iter().collect::<Result<_>>()?;
        assert_eq!(buf_results.len(), expected.len());
        for (buf, exp) in buf_results.iter().zip(expected.iter()) {
            assert_eq!(*buf.get::<R>().unwrap(), *exp);
        }
        Ok(())
    }

    #[test]
    fn test_decode_buf_iter_mixed_types() -> crate::Result<()> {
        let trade = TradeMsg {
            hd: RecordHeader::new::<TradeMsg>(rtype::MBP_0, 1, 100, 1678284110000000000),
            price: 5_000_000_000,
            size: 10,
            ..TradeMsg::default()
        };
        let ohlcv = OhlcvMsg {
            hd: RecordHeader::new::<OhlcvMsg>(rtype::OHLCV_1S, 1, 200, 1678284120000000000),
            open: 100,
            high: 200,
            low: 75,
            close: 125,
            volume: 65,
        };
        let mut buffer = Vec::new();
        let mut encoder = DbnRecordEncoder::new(&mut buffer);
        encoder.encode_record(&trade)?;
        encoder.encode_record(&ohlcv)?;

        let mut decoder = RecordDecoder::new(buffer.as_slice());
        let results: Vec<RecordBuf> = decoder.decode_buf_iter().collect::<Result<_>>()?;

        assert_eq!(results.len(), 2);
        assert!(results[0].has::<TradeMsg>());
        assert!(results[1].has::<OhlcvMsg>());
        assert_eq!(*results[0].get::<TradeMsg>().unwrap(), trade);
        assert_eq!(*results[1].get::<OhlcvMsg>().unwrap(), ohlcv);
        Ok(())
    }

    #[test]
    fn test_decode_buf_iter_error_after_valid() {
        let trade = TradeMsg {
            hd: RecordHeader::new::<TradeMsg>(rtype::MBP_0, 1, 100, 167828411000000000),
            price: 5_000_000_000,
            size: 10,
            ..TradeMsg::default()
        };
        let mut buffer = Vec::new();
        let mut encoder = DbnRecordEncoder::new(&mut buffer);
        encoder.encode_record(&trade).unwrap();
        // Append a corrupt record
        buffer.extend_from_slice(&[0u8; std::mem::size_of::<RecordHeader>()]);

        let mut decoder = RecordDecoder::new(buffer.as_slice());
        let mut iter = decoder.decode_buf_iter();
        let first = iter.next().unwrap().unwrap();
        assert_eq!(*first.get::<TradeMsg>().unwrap(), trade);
        let second = iter.next().unwrap();
        assert!(matches!(second, Err(Error::Decode(_))));
    }
}
