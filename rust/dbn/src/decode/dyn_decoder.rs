#![allow(deprecated)] // DBZ

use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};

use crate::{Compression, HasRType, Metadata, RecordRef, VersionUpgradePolicy};

use super::{
    dbn, dbz, private, zstd, DbnMetadata, DecodeRecord, DecodeRecordRef, DecodeStream,
    StreamIterDecoder,
};

/// A decoder whose [`Encoding`](crate::enums::Encoding) and [`Compression`] are
/// determined at runtime by peeking at the first few bytes.
pub struct DynDecoder<'a, R>(DynDecoderImpl<'a, R>)
where
    R: io::BufRead;

enum DynDecoderImpl<'a, R>
where
    R: io::BufRead,
{
    Dbn(dbn::Decoder<R>),
    ZstdDbn(dbn::Decoder<::zstd::stream::Decoder<'a, R>>),
    LegacyDbz(dbz::Decoder<R>),
}

impl<R> DynDecoder<'_, BufReader<R>>
where
    R: io::Read,
{
    /// Creates a new [`DynDecoder`] from a reader, with the specified `compression`. It
    /// will decode records from previous DBN versions according to `upgrade_policy`.
    ///
    /// # Errors
    /// This function will return an error if it fails to parse the metadata.
    pub fn new(
        reader: R,
        compression: Compression,
        upgrade_policy: VersionUpgradePolicy,
    ) -> crate::Result<Self> {
        Self::with_buffer(BufReader::new(reader), compression, upgrade_policy)
    }

    /// Creates a new [`DynDecoder`] from a reader, inferring the encoding and
    /// compression. If `reader` also implements [`io::BufRead`], it is better to use
    /// [`inferred_with_buffer()`](Self::inferred_with_buffer). It will decode records
    /// from previous DBN versions according to `upgrade_policy`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to determine
    /// the encoding of `reader` or it fails to parse the metadata.
    pub fn new_inferred(reader: R, upgrade_policy: VersionUpgradePolicy) -> crate::Result<Self> {
        Self::inferred_with_buffer(BufReader::new(reader), upgrade_policy)
    }
}

impl<R> DynDecoder<'_, R>
where
    R: io::BufRead,
{
    /// Creates a new [`DynDecoder`] from a buffered reader with the specified
    /// `compression`.It will decode records from previous DBN versions according to
    /// `upgrade_policy`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to determine
    /// the encoding of `reader` or it fails to parse the metadata.
    pub fn with_buffer(
        reader: R,
        compression: Compression,
        upgrade_policy: VersionUpgradePolicy,
    ) -> crate::Result<Self> {
        match compression {
            Compression::None => Ok(Self(DynDecoderImpl::Dbn(
                dbn::Decoder::with_upgrade_policy(reader, upgrade_policy)?,
            ))),
            Compression::Zstd => Ok(Self(DynDecoderImpl::ZstdDbn(
                dbn::Decoder::with_upgrade_policy(
                    ::zstd::stream::Decoder::with_buffer(reader)
                        .map_err(|e| crate::Error::io(e, "creating zstd decoder"))?,
                    upgrade_policy,
                )?,
            ))),
        }
    }

    /// Creates a new [`DynDecoder`] from a buffered reader, inferring the encoding
    /// and compression.It will decode records from previous DBN versions according
    /// to `upgrade_policy`.
    ///
    /// # Errors
    /// This function will return an error if it is unable to determine
    /// the encoding of `reader` or it fails to parse the metadata.
    pub fn inferred_with_buffer(
        mut reader: R,
        upgrade_policy: VersionUpgradePolicy,
    ) -> crate::Result<Self> {
        let first_bytes = reader
            .fill_buf()
            .map_err(|e| crate::Error::io(e, "creating buffer to infer encoding"))?;
        if dbz::starts_with_prefix(first_bytes) {
            Ok(Self(DynDecoderImpl::LegacyDbz(
                dbz::Decoder::with_upgrade_policy(reader, upgrade_policy)?,
            )))
        } else if dbn::starts_with_prefix(first_bytes) {
            Ok(Self(DynDecoderImpl::Dbn(
                dbn::Decoder::with_upgrade_policy(reader, upgrade_policy)?,
            )))
        } else if zstd::starts_with_prefix(first_bytes) {
            Ok(Self(DynDecoderImpl::ZstdDbn(
                dbn::Decoder::with_upgrade_policy(
                    ::zstd::stream::Decoder::with_buffer(reader)
                        .map_err(|e| crate::Error::io(e, "creating zstd decoder"))?,
                    upgrade_policy,
                )?,
            )))
        } else {
            Err(crate::Error::decode("unable to determine encoding"))
        }
    }
}

impl DynDecoder<'_, BufReader<File>> {
    /// Creates a new [`DynDecoder`] from the file at `path`. It will decode records
    /// from previous DBN versions according to `upgrade_policy`.
    ///
    /// # Errors
    /// This function will return an error if the file doesn't exist, it is unable to
    /// determine the encoding of the file or it fails to parse the metadata.
    pub fn from_file(
        path: impl AsRef<Path>,
        upgrade_policy: VersionUpgradePolicy,
    ) -> crate::Result<Self> {
        let file = File::open(path.as_ref()).map_err(|e| {
            crate::Error::io(
                e,
                format!(
                    "opening file to decode at path '{}'",
                    path.as_ref().display()
                ),
            )
        })?;
        DynDecoder::new_inferred(file, upgrade_policy)
    }
}

impl<R> DecodeRecordRef for DynDecoder<'_, R>
where
    R: io::BufRead,
{
    fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef<'_>>> {
        match &mut self.0 {
            DynDecoderImpl::Dbn(decoder) => decoder.decode_record_ref(),
            DynDecoderImpl::ZstdDbn(decoder) => decoder.decode_record_ref(),
            DynDecoderImpl::LegacyDbz(decoder) => decoder.decode_record_ref(),
        }
    }
}

impl<R> DbnMetadata for DynDecoder<'_, R>
where
    R: io::BufRead,
{
    fn metadata(&self) -> &Metadata {
        match &self.0 {
            DynDecoderImpl::Dbn(decoder) => decoder.metadata(),
            DynDecoderImpl::ZstdDbn(decoder) => decoder.metadata(),
            DynDecoderImpl::LegacyDbz(decoder) => decoder.metadata(),
        }
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        match &mut self.0 {
            DynDecoderImpl::Dbn(decoder) => decoder.metadata_mut(),
            DynDecoderImpl::ZstdDbn(decoder) => decoder.metadata_mut(),
            DynDecoderImpl::LegacyDbz(decoder) => decoder.metadata_mut(),
        }
    }
}

impl<R> DecodeRecord for DynDecoder<'_, R>
where
    R: io::BufRead,
{
    fn decode_record<T: HasRType>(&mut self) -> crate::Result<Option<&T>> {
        match &mut self.0 {
            DynDecoderImpl::Dbn(decoder) => decoder.decode_record(),
            DynDecoderImpl::ZstdDbn(decoder) => decoder.decode_record(),
            DynDecoderImpl::LegacyDbz(decoder) => decoder.decode_record(),
        }
    }
}

impl<R> DecodeStream for DynDecoder<'_, R>
where
    R: io::BufRead,
{
    fn decode_stream<T: HasRType>(self) -> StreamIterDecoder<Self, T>
    where
        Self: Sized,
    {
        StreamIterDecoder::new(self)
    }
}

impl<R> private::LastRecord for DynDecoder<'_, R>
where
    R: io::BufRead,
{
    fn last_record(&self) -> Option<RecordRef<'_>> {
        match &self.0 {
            DynDecoderImpl::Dbn(decoder) => decoder.last_record(),
            DynDecoderImpl::ZstdDbn(decoder) => decoder.last_record(),
            DynDecoderImpl::LegacyDbz(decoder) => decoder.last_record(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use crate::{decode::tests::TEST_DATA_PATH, enums::VersionUpgradePolicy};

    use super::*;

    #[test]
    fn test_detects_any_dbn_version_as_dbn() {
        let mut buf = Vec::new();
        let mut file = File::open(format!("{TEST_DATA_PATH}/test_data.mbo.v3.dbn")).unwrap();
        file.read_to_end(&mut buf).unwrap();
        // change version
        buf[3] = crate::DBN_VERSION + 1;
        let res = DynDecoder::new_inferred(io::Cursor::new(buf), VersionUpgradePolicy::default());
        assert!(matches!(res, Err(e) if e
            .to_string()
            .contains("can't decode newer version of DBN")));
    }
}
