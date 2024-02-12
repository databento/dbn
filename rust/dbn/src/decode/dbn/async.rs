use std::{io::Cursor, path::Path};

use async_compression::tokio::bufread::ZstdDecoder;
use tokio::{
    fs::File,
    io::{self, BufReader},
};

use crate::{
    compat,
    decode::{FromLittleEndianSlice, VersionUpgradePolicy},
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
    pub async fn new(mut reader: R) -> crate::Result<Self> {
        let metadata = MetadataDecoder::new(&mut reader).decode().await?;
        Ok(Self {
            decoder: RecordDecoder::with_version(
                reader,
                metadata.version,
                VersionUpgradePolicy::Upgrade,
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
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, the metadata
    /// may have been partially read, corrupting the stream.
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

    /// Sets the behavior for decoding DBN data of previous versions.
    pub fn set_upgrade_policy(&mut self, upgrade_policy: VersionUpgradePolicy) {
        self.metadata.upgrade(upgrade_policy);
        self.decoder.set_upgrade_policy(upgrade_policy);
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
    pub async fn decode_record<'a, T: HasRType + 'a>(&'a mut self) -> Result<Option<&T>> {
        self.decoder.decode().await
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

impl Decoder<BufReader<File>> {
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
        Self::new(BufReader::new(file)).await
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
    state: DecoderState,
    framer: RecordFrameDecoder,
    read_buf: Cursor<Vec<u8>>,
    compat_buf: [u8; crate::MAX_RECORD_LEN],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecoderState {
    Read,
    Yield,
    Eof,
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
            return Err(crate::Error::decode(format!("can't decode newer version of DBN. Decoder version is {DBN_VERSION}, input version is {version}")));
        }
        Ok(Self {
            version,
            upgrade_policy,
            reader,
            state: DecoderState::Read,
            framer: RecordFrameDecoder::Head,
            read_buf: Cursor::default(),
            compat_buf: [0; crate::MAX_RECORD_LEN],
        })
    }

    /// Sets the DBN version to expect when decoding.
    ///
    /// # Errors
    /// This function will return an error if the `version` exceeds the highest
    /// supported version.
    pub fn set_version(&mut self, version: u8) -> crate::Result<()> {
        if version > DBN_VERSION {
            Err(crate::Error::decode(format!("can't decode newer version of DBN. Decoder version is {DBN_VERSION}, input version is {version}")))
        } else {
            self.version = version;
            Ok(())
        }
    }

    /// Sets the behavior for decoding DBN data of previous versions.
    pub fn set_upgrade_policy(&mut self, upgrade_policy: VersionUpgradePolicy) {
        self.upgrade_policy = upgrade_policy;
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
    pub async fn decode<'a, T: HasRType + 'a>(&'a mut self) -> Result<Option<&T>> {
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
        let io_err = |e| crate::Error::io(e, "decoding record reference");
        loop {
            // maybe read more into buffer
            if self.state == DecoderState::Read {
                // read_buf is cancellation safe
                let bytes_read = self
                    .reader
                    .read_buf(self.read_buf.get_mut())
                    .await
                    .map_err(io_err)?;
                self.state = if bytes_read == 0 {
                    DecoderState::Eof
                } else {
                    DecoderState::Yield
                };
            }
            // yield if a complete record is in the buffer
            if let Some(frame) = self.framer.decode(&mut self.read_buf) {
                // sanity check
                return if frame.len() < std::mem::size_of::<RecordHeader>() {
                    Err(crate::Error::decode(format!(
                        "invalid record with length {} shorter than header",
                        frame.len()
                    )))
                } else {
                    Ok(Some(unsafe {
                        compat::decode_record_ref(
                            self.version,
                            self.upgrade_policy,
                            &mut self.compat_buf,
                            // Recreate slice to get around borrow checker
                            std::slice::from_raw_parts(frame.as_ptr(), frame.len()),
                        )
                    }))
                };
            } else if self.state == DecoderState::Eof {
                // there should be no remaining bytes in the buffer after reaching EOF
                // and yielding all complete records
                return if self.read_buf.remaining() == 0 {
                    Ok(None)
                } else {
                    Err(crate::Error::decode(format!(
                        "unexpected partial record remaining in stream: {} bytes",
                        self.read_buf.remaining()
                    )))
                };
            } else {
                self.state = DecoderState::Read;
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
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, the metadata
    /// may have been partially read, corrupting the stream.
    pub async fn decode(&mut self) -> Result<Metadata> {
        let mut prelude_buffer = [0u8; 8];
        self.reader
            .read_exact(&mut prelude_buffer)
            .await
            .map_err(|e| crate::Error::io(e, "reading metadata prelude"))?;
        if &prelude_buffer[..super::DBN_PREFIX_LEN] != super::DBN_PREFIX {
            return Err(crate::Error::decode("invalid DBN header"));
        }
        let version = prelude_buffer[super::DBN_PREFIX_LEN];
        if version > DBN_VERSION {
            return Err(crate::Error::decode(format!("can't decode newer version of DBN. Decoder version is {DBN_VERSION}, input version is {version}")));
        }
        let length = u32::from_le_slice(&prelude_buffer[4..]);
        if (length as usize) < METADATA_FIXED_LEN {
            return Err(crate::Error::decode(
                "invalid DBN metadata. Metadata length shorter than fixed length.",
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

// Reworked version of `tokio_util::codec::LengthDelimitedCodec` because
// - it lacked support for a multiplier for the length
// - it assumed some operations were fallible
// - returning bytes slice didn't work well with the existing lifetime requirements for RecordRef
#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Eq))]
enum RecordFrameDecoder {
    Head,
    Data(usize),
}

impl RecordFrameDecoder {
    fn decode_head(&mut self, src: &mut Cursor<Vec<u8>>) -> Option<usize> {
        if src.remaining() == 0 {
            // Not enough data
            return None;
        }
        // Unlike the original implementation, we don't want to advance the position in buffer
        // because each record includes the `length`.
        let n = src.remaining_bytes()[0] as usize * RecordHeader::LENGTH_MULTIPLIER;

        // Ensure that the buffer has enough space to read the incoming
        // payload
        let additional = n.saturating_sub(src.remaining());
        src.reserve(additional);

        Some(n)
    }

    fn validate_len(&self, n: usize, src: &Cursor<Vec<u8>>) -> Option<usize> {
        if src.remaining() < n {
            return None;
        }
        Some(n)
    }

    fn decode<'a>(&mut self, src: &'a mut Cursor<Vec<u8>>) -> Option<&'a [u8]> {
        let n = match self {
            RecordFrameDecoder::Head => match self.decode_head(src) {
                Some(n) => {
                    *self = RecordFrameDecoder::Data(n);
                    n
                }
                None => return None,
            },
            RecordFrameDecoder::Data(n) => *n,
        };
        match self.validate_len(n, src) {
            Some(n) => {
                // Update the decode state
                *self = RecordFrameDecoder::Head;

                // Make sure the buffer has enough space to read the next head
                let additional = 1usize.saturating_sub(src.remaining());
                src.reserve(additional);
                // Need to advance position after reserving so as not to free the
                // slice being returned
                let pos = src.position() as usize;
                src.advance(n); // modifies `position`
                let end = pos + n;

                Some(&src.get_ref()[pos..end])
            }
            None => None,
        }
    }
}

/// Helper methods for working with `Cursor<Vec<u8>>`. Inspired by `bytes::BytesMut`.
trait Buffer {
    /// Returns the length of the remaining unread bytes.
    fn remaining(&self) -> usize;
    /// Returns a slice of the bytes after `position`. Based on unstable `remaining_slice`.
    fn remaining_bytes(&self) -> &[u8];
    /// Reserves capacity for `additional` more bytes. May shift `position` to reclaim
    /// previously read bytes.
    fn reserve(&mut self, additional: usize);
    /// Advances the read position by `n` bytes.
    fn advance(&mut self, n: usize);
}

impl Buffer for Cursor<Vec<u8>> {
    fn remaining(&self) -> usize {
        self.get_ref()
            .len()
            .saturating_sub(self.position() as usize)
    }

    fn remaining_bytes(&self) -> &[u8] {
        &self.get_ref()[(self.position() as usize).min(self.get_ref().len())..]
    }

    fn reserve(&mut self, additional: usize) {
        // short circuit
        if self.get_ref().capacity() - self.get_ref().len() >= additional {
            return;
        }
        let reclaimable = self.position() as usize;
        if reclaimable >= additional && reclaimable >= self.remaining() {
            let pos = (self.position() as usize).min(self.get_ref().len());
            let remaining = self.remaining();
            // Safety: Checked `reclaimable` is greater than or equal to `remaining` so there's no overlap
            unsafe {
                // Use pointer arithmetic to handle special case where `remaining` = 0. Regular indexing
                // would panic.
                let pos_mut_ptr = self.get_mut().as_mut_ptr().add(pos);
                std::ptr::copy_nonoverlapping(pos_mut_ptr, self.get_mut().as_mut_ptr(), remaining)
            }
            // Update vector length so `remaining` remains unchanged and new data will
            // be read into the correct place
            self.get_mut().truncate(remaining);
            self.set_position(0);
        } else {
            self.get_mut().reserve(additional);
        }
    }

    fn advance(&mut self, n: usize) {
        let new_pos = self.position() + n as u64;
        debug_assert!(new_pos as usize <= self.get_ref().len());
        self.set_position(new_pos);
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
            matches!(target.decode_ref().await, Err(Error::Decode(msg)) if msg.starts_with("invalid record with length"))
        );
    }

    #[tokio::test]
    async fn test_decode_record_length_less_than_header() {
        let buf = vec![3u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        assert_eq!(buf[0] as usize * RecordHeader::LENGTH_MULTIPLIER, buf.len());

        let mut target = RecordDecoder::new(buf.as_slice());
        let res = target.decode_ref().await;
        dbg!(&res);
        assert!(
            matches!(res, Err(Error::Decode(msg)) if msg.starts_with("invalid record with length"))
        );
    }

    #[tokio::test]
    async fn test_decode_record_length_longer_than_buffer() {
        let rec = ErrorMsg::new(1680703198000000000, "Test", true);
        let mut target = RecordDecoder::new(&rec.as_ref()[..rec.record_size() - 1]);
        let res = target.decode_ref().await;
        dbg!(&res);
        assert!(matches!(res, Err(Error::Decode(msg)) if msg.starts_with("unexpected")));
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

    fn dummy_buffer(pos: usize, size: usize) -> Cursor<Vec<u8>> {
        let mut res = Cursor::new((0..size).map(|i| i as u8).collect());
        res.set_position(pos as u64);
        res
    }

    #[rstest]
    #[case::empty(0, 0, None)]
    #[case::end(16, 16, None)]
    #[case::last(15, 16, Some(15 * RecordHeader::LENGTH_MULTIPLIER))]
    #[case::middle(8, 16, Some(8 * RecordHeader::LENGTH_MULTIPLIER))]
    fn frame_decoder_decode_head(
        #[case] pos: usize,
        #[case] size: usize,
        #[case] exp: Option<usize>,
    ) {
        let mut src = dummy_buffer(pos, size);
        let res = RecordFrameDecoder::Head.decode_head(&mut src);
        assert_eq!(res, exp);
    }

    #[rstest]
    #[case::empty(0, 0, 32, None)]
    #[case::end(16, 16, 64, None)]
    #[case::last(15, 16, 56, None)]
    #[case::middle(8, 16, 8, Some(8))]
    #[case::extra(24, 256, 128, Some(128))]
    fn frame_decoder_validate_len(
        #[case] pos: usize,
        #[case] size: usize,
        #[case] n: usize,
        #[case] exp: Option<usize>,
    ) {
        let src = dummy_buffer(pos, size);
        let res = RecordFrameDecoder::Head.validate_len(n, &src);
        assert_eq!(res, exp);
    }

    #[rstest]
    #[case::empty(0, 0, RecordFrameDecoder::Head, None, 0, RecordFrameDecoder::Head)]
    #[case::end(16, 16, RecordFrameDecoder::Head, None, 16, RecordFrameDecoder::Head)]
    #[case::last(15, 16, RecordFrameDecoder::Head, None, 15, RecordFrameDecoder::Data(15 * RecordHeader::LENGTH_MULTIPLIER))]
    #[case::last(2, 12, RecordFrameDecoder::Head, Some(2..10), 10, RecordFrameDecoder::Head)]
    #[case::middle(8, 16, RecordFrameDecoder::Data(8), Some(8..16), 16, RecordFrameDecoder::Head)]
    #[case::extra(24, 256, RecordFrameDecoder::Data(56), Some(24..80), 80, RecordFrameDecoder::Head)]
    fn frame_decoder_decode(
        #[case] pos: usize,
        #[case] size: usize,
        #[case] mut decoder: RecordFrameDecoder,
        #[case] exp: Option<std::ops::Range<u8>>,
        #[case] exp_pos: usize,
        #[case] exp_decoder: RecordFrameDecoder,
    ) {
        let mut src = dummy_buffer(pos, size);
        let res = decoder.decode(&mut src);
        let exp = exp.map(|exp| exp.collect::<Vec<u8>>());
        assert_eq!(res, exp.as_deref());
        assert_eq!(decoder, exp_decoder);
        assert_eq!(src.position() as usize, exp_pos);
    }

    #[rstest]
    #[case::short_circuit(4, 8, 16, 4, 4)]
    #[case::reclaim(100, 142, 150, 32, 0)]
    #[case::reclaim(124, 124, 148, 56, 0)]
    #[case::nothing_to_reclaim(0, 10, 16, 16, 0)]
    #[case::nothing_to_reclaim(0, 16, 16, 16, 0)]
    #[case::nothing_to_reclaim(16, 16, 16, 32, 16)]
    #[case::vec_reserve(10, 42, 50, 32, 10)]
    fn buffer_reserve(
        #[case] pos: usize,
        #[case] size: usize,
        #[case] capacity: usize,
        #[case] additional: usize,
        #[case] exp_pos: usize,
    ) {
        assert!(capacity >= size);
        let mut vec = Vec::with_capacity(capacity);
        assert_eq!(vec.capacity(), capacity);
        for i in 0..size {
            vec.push(i as u8);
        }
        let mut target = Cursor::new(vec);
        target.set_position(pos as u64);

        let remaining = target.remaining();
        assert_eq!(remaining, size - pos);
        let remaining_bytes = target.remaining_bytes().to_vec();

        target.reserve(additional);

        assert_eq!(target.position() as usize, exp_pos);
        // the contents should never be changed
        assert_eq!(target.remaining(), remaining);
        assert_eq!(target.remaining_bytes(), remaining_bytes.as_slice());
    }
}
