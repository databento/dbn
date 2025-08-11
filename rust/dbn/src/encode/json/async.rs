use tokio::io::{self, AsyncWriteExt};

use super::serialize::{to_json_in_buf, to_json_with_sym_in_buf};
use crate::{
    encode::{AsyncEncodeRecord, AsyncEncodeRecordRef, AsyncEncodeRecordTextExt, DbnEncodable},
    record_ref::RecordRef,
    rtype_dispatch, Error, Metadata, Result,
};

/// Type for encoding files and streams of DBN records in JSON lines.
pub struct Encoder<W>
where
    W: io::AsyncWriteExt + Unpin,
{
    writer: W,
    should_pretty_print: bool,
    buf: String,
    use_pretty_px: bool,
    use_pretty_ts: bool,
}

impl<W> Encoder<W>
where
    W: io::AsyncWriteExt + Unpin,
{
    /// Creates a new instance of [`Encoder`]. If `should_pretty_print` is `true`,
    /// each JSON object will be nicely formatted and indented, instead of the default
    /// compact output with no whitespace between key-value pairs.
    pub fn new(
        writer: W,
        should_pretty_print: bool,
        use_pretty_px: bool,
        use_pretty_ts: bool,
    ) -> Self {
        Self {
            writer,
            should_pretty_print,
            use_pretty_px,
            use_pretty_ts,
            buf: String::new(),
        }
    }

    /// Encodes `metadata` into JSON.
    ///
    /// # Errors
    /// This function returns an error if there's an error writing to `writer`.
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// `tokio::select!` statement and another branch completes first, then the
    /// metadata JSON may have been partially written, but future calls will begin writing
    /// the metadata JSON from the beginning.
    pub async fn encode_metadata(&mut self, metadata: &Metadata) -> Result<()> {
        to_json_in_buf(
            &mut self.buf,
            metadata,
            self.should_pretty_print,
            self.use_pretty_px,
            self.use_pretty_ts,
        );
        let io_err = |e| Error::io(e, "writing metadata");
        self.write_buf(io_err).await?;
        self.writer.flush().await.map_err(io_err)?;
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

    /// Writes to `self.buf`, but not the writer.
    fn encode_to_buf<R: DbnEncodable>(&mut self, record: &R) {
        to_json_in_buf(
            &mut self.buf,
            record,
            self.should_pretty_print,
            self.use_pretty_px,
            self.use_pretty_ts,
        );
    }

    async fn write_buf<F>(&mut self, handle_err: F) -> crate::Result<()>
    where
        F: FnOnce(io::Error) -> Error,
    {
        let res = self
            .writer
            .write_all(self.buf.as_bytes())
            .await
            .map_err(handle_err);
        // Always clear `buf`
        self.buf.clear();
        res
    }
}

impl<W> AsyncEncodeRecord for Encoder<W>
where
    W: AsyncWriteExt + Unpin,
{
    async fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> Result<()> {
        self.encode_to_buf(record);
        self.write_buf(|e| Error::io(e, "writing record")).await
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

impl<W> AsyncEncodeRecordRef for Encoder<W>
where
    W: AsyncWriteExt + Unpin,
{
    async fn encode_record_ref(&mut self, record_ref: RecordRef<'_>) -> Result<()> {
        rtype_dispatch!(record_ref, self.encode_record().await)?
    }

    async unsafe fn encode_record_ref_ts_out(
        &mut self,
        record_ref: RecordRef<'_>,
        ts_out: bool,
    ) -> Result<()> {
        rtype_dispatch!(record_ref, ts_out: ts_out, self.encode_record().await)?
    }
}

impl<W> AsyncEncodeRecordTextExt for Encoder<W>
where
    W: AsyncWriteExt + Unpin,
{
    async fn encode_record_with_sym<R: DbnEncodable>(
        &mut self,
        record: &R,
        symbol: Option<&str>,
    ) -> Result<()> {
        to_json_with_sym_in_buf(
            &mut self.buf,
            record,
            self.should_pretty_print,
            self.use_pretty_px,
            self.use_pretty_ts,
            symbol,
        );
        self.write_buf(|e| Error::io(e, "writing record")).await
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::c_char;

    use tokio::io::{AsyncWriteExt, BufWriter};

    use crate::{encode::test_data::RECORD_HEADER, enums::rtype, MboMsg, RecordHeader};

    use super::*;

    async fn write_to_json_string<R>(
        record: &R,
        should_pretty_print: bool,
        use_pretty_px: bool,
        use_pretty_ts: bool,
    ) -> String
    where
        R: DbnEncodable,
    {
        let mut buffer = Vec::new();
        let mut writer = BufWriter::new(&mut buffer);
        Encoder::new(
            &mut writer,
            should_pretty_print,
            use_pretty_px,
            use_pretty_ts,
        )
        .encode_record(record)
        .await
        .unwrap();
        writer.flush().await.unwrap();
        String::from_utf8(buffer).expect("valid UTF-8")
    }

    async fn write_ref_to_json_string(
        record: RecordRef<'_>,
        should_pretty_print: bool,
        use_pretty_px: bool,
        use_pretty_ts: bool,
    ) -> String {
        let mut buffer = Vec::new();
        let mut writer = BufWriter::new(&mut buffer);
        Encoder::new(
            &mut writer,
            should_pretty_print,
            use_pretty_px,
            use_pretty_ts,
        )
        .encode_record_ref(record)
        .await
        .unwrap();
        writer.flush().await.unwrap();
        String::from_utf8(buffer).expect("valid UTF-8")
    }

    #[tokio::test]
    async fn test_mbo_write_json() {
        let record = MboMsg {
            hd: RecordHeader::new::<MboMsg>(
                rtype::MBO,
                RECORD_HEADER.publisher_id,
                RECORD_HEADER.instrument_id,
                RECORD_HEADER.ts_event,
            ),
            order_id: 16,
            price: 5500,
            size: 3,
            flags: 128.into(),
            channel_id: 14,
            action: 'R' as c_char,
            side: 'N' as c_char,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
        };
        let res = write_to_json_string(&record, false, true, false).await;
        let ref_res = write_ref_to_json_string(RecordRef::from(&record), false, true, false).await;

        assert_eq!(res, ref_res);
        assert_eq!(
            ref_res,
            format!(
                "{{{},{},{}}}\n",
                r#""ts_recv":"1658441891000000000""#,
                r#""hd":{"ts_event":"1658441851000000000","rtype":160,"publisher_id":1,"instrument_id":323}"#,
                r#""action":"R","side":"N","price":"0.000005500","size":3,"channel_id":14,"order_id":"16","flags":128,"ts_in_delta":22000,"sequence":1002375"#
            )
        );
    }
}
