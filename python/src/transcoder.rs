use std::io::{self, BufWriter, Write};

use dbn::{
    decode::{
        zstd::starts_with_prefix, DbnMetadataDecoder, DbnRecordDecoder, DecodeRecordRef, DynReader,
    },
    encode::{
        CsvEncoder, DbnMetadataEncoder, DbnRecordEncoder, DynWriter, EncodeRecordRef, JsonEncoder,
    },
    enums::{Compression, Encoding},
    python::to_val_err,
};
use pyo3::prelude::*;

use crate::encode::PyFileLike;

#[pyclass(module = "databento_dbn")]
pub struct Transcoder {
    buffer: io::Cursor<Vec<u8>>,
    // wrap in buffered writer to minimize calls to Python
    output: DynWriter<'static, BufWriter<PyFileLike>>,
    output_encoding: Encoding,
    use_pretty_px: bool,
    use_pretty_ts: bool,
    has_decoded_metadata: bool,
    ts_out: bool,
    input_compression: Option<Compression>,
}

#[pymethods]
impl Transcoder {
    #[allow(clippy::too_many_arguments)]
    #[new]
    fn new(
        file: PyFileLike,
        encoding: Encoding,
        compression: Compression,
        pretty_px: Option<bool>,
        pretty_ts: Option<bool>,
        has_metadata: Option<bool>,
        ts_out: Option<bool>,
        input_compression: Option<Compression>,
    ) -> PyResult<Self> {
        Ok(Self {
            buffer: io::Cursor::default(),
            output: DynWriter::new(BufWriter::new(file), compression).map_err(to_val_err)?,
            output_encoding: encoding,
            use_pretty_px: pretty_px.unwrap_or(true),
            use_pretty_ts: pretty_ts.unwrap_or(true),
            has_decoded_metadata: !has_metadata.unwrap_or(true),
            ts_out: ts_out.unwrap_or_default(),
            input_compression,
        })
    }

    fn write(&mut self, bytes: &[u8]) -> PyResult<()> {
        self.buffer.write_all(bytes).map_err(to_val_err)?;
        self.maybe_encode()
    }

    fn flush(&mut self) -> PyResult<()> {
        self.maybe_encode()?;
        self.output.flush()?;
        Ok(())
    }

    fn buffer(&self) -> &[u8] {
        self.buffer.get_ref().as_slice()
    }
}

impl Transcoder {
    fn maybe_encode(&mut self) -> PyResult<()> {
        let orig_position = self.buffer.position();
        self.buffer.set_position(0);
        if !self.detect_compression() {
            return Ok(());
        }
        self.maybe_decode_metadata(orig_position)?;
        // main
        let mut read_position = self.buffer.position() as usize;
        let mut decoder = DbnRecordDecoder::new(
            DynReader::with_buffer(&mut self.buffer, self.input_compression.unwrap())
                .map_err(to_val_err)?,
        );
        let mut encoder = Self::record_encoder(
            &mut self.output,
            self.output_encoding,
            self.use_pretty_px,
            self.use_pretty_ts,
        );
        loop {
            match decoder.decode_record_ref() {
                Ok(Some(rec)) => {
                    unsafe { encoder.encode_record_ref(rec, self.ts_out) }.map_err(to_val_err)?;
                    // keep track of position after last _successful_ decoding to
                    // ensure buffer is left in correct state in the case where one
                    // or more successful decodings is followed by a partial one, i.e.
                    // `decode_record_ref` returning `Ok(None)`
                    read_position = decoder.get_ref().get_ref().position() as usize;
                }
                Ok(None) => {
                    break;
                }
                Err(err) => {
                    self.buffer.set_position(orig_position);
                    return Err(to_val_err(err));
                }
            }
        }
        drop(encoder);
        self.shift_buffer(read_position);
        Ok(())
    }

    fn detect_compression(&mut self) -> bool {
        if self.input_compression.is_none() {
            if self.buffer.get_ref().len() < 4 {
                return false;
            }
            self.input_compression =
                Some(if starts_with_prefix(self.buffer.get_ref().as_slice()) {
                    Compression::ZStd
                } else {
                    Compression::None
                });
        }
        true
    }

    fn maybe_decode_metadata(&mut self, orig_position: u64) -> PyResult<()> {
        if !self.has_decoded_metadata {
            match DbnMetadataDecoder::new(&mut self.buffer).decode() {
                Ok(metadata) => {
                    self.ts_out = metadata.ts_out;
                    self.has_decoded_metadata = true;
                    if self.output_encoding == Encoding::Dbn {
                        DbnMetadataEncoder::new(&mut self.output)
                            .encode(&metadata)
                            .map_err(to_val_err)?;
                    }
                }
                Err(err) => {
                    self.buffer.set_position(orig_position);
                    // haven't read enough data for metadata
                    return Err(to_val_err(err));
                }
            }
        }
        Ok(())
    }

    fn record_encoder<'a>(
        writer: &'a mut DynWriter<'static, BufWriter<PyFileLike>>,
        output_encoding: Encoding,
        use_pretty_px: bool,
        use_pretty_ts: bool,
    ) -> Box<dyn EncodeRecordRef + 'a> {
        match output_encoding {
            Encoding::Dbn => Box::new(DbnRecordEncoder::new(writer)),
            Encoding::Csv => Box::new(CsvEncoder::new(writer, use_pretty_px, use_pretty_ts)),
            Encoding::Json => Box::new(JsonEncoder::new(
                writer,
                false,
                use_pretty_px,
                use_pretty_px,
            )),
        }
    }

    fn shift_buffer(&mut self, read_position: usize) {
        let inner_buf = self.buffer.get_mut();
        let length = inner_buf.len();
        let new_length = length - read_position;
        inner_buf.drain(..read_position);
        debug_assert_eq!(inner_buf.len(), new_length);
        self.buffer.set_position(new_length as u64);
    }
}

#[cfg(test)]
mod tests {
    use dbn::{
        datasets::XNAS_ITCH,
        encode::{DbnEncoder, EncodeRecord},
        enums::{rtype, SType, Schema},
        record::{ErrorMsg, OhlcvMsg, RecordHeader},
        MetadataBuilder,
    };

    use crate::{encode::tests::MockPyFile, tests::setup};

    use super::*;

    #[test]
    fn test_partial_records() {
        setup();
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut transcoder = Python::with_gil(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                Encoding::Json,
                Compression::None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap()
        });
        assert!(!transcoder.has_decoded_metadata);
        let mut encoder = DbnEncoder::new(
            Vec::new(),
            &MetadataBuilder::new()
                .dataset(XNAS_ITCH.to_owned())
                .schema(Some(Schema::Trades))
                .stype_in(Some(SType::RawSymbol))
                .stype_out(SType::InstrumentId)
                .start(0)
                .build(),
        )
        .unwrap();
        transcoder.write(encoder.get_ref().as_slice()).unwrap();
        // Metadata doesn't get transcoded for JSON
        assert!(output_buf.lock().unwrap().get_ref().is_empty());
        assert!(transcoder.has_decoded_metadata);
        let metadata_pos = encoder.get_ref().len();
        let rec = ErrorMsg::new(1680708278000000000, "This is a test");
        encoder.encode_record(&rec).unwrap();
        assert!(transcoder.buffer.get_ref().is_empty());
        let record_pos = encoder.get_ref().len();
        // write record byte by byte
        for i in metadata_pos..record_pos {
            transcoder.write(&encoder.get_ref()[i..i + 1]).unwrap();
            // wrote last byte
            if i == record_pos - 1 {
                break;
            }
            assert_eq!(transcoder.buffer.get_ref().len(), i + 1 - metadata_pos);
        }
        // writing the remainder of the record should have the transcoder
        // transcode the record to the output file
        assert!(transcoder.buffer.get_ref().is_empty());
        assert_eq!(record_pos - metadata_pos, std::mem::size_of_val(&rec));
        transcoder.flush().unwrap();
        let output = output_buf.lock().unwrap();
        let output = std::str::from_utf8(output.get_ref().as_slice()).unwrap();
        assert_eq!(output.chars().filter(|c| *c == '\n').count(), 1);
    }

    #[test]
    fn test_full_with_partial_record() {
        setup();
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut transcoder = Python::with_gil(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                Encoding::Csv,
                Compression::None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap()
        });
        let buffer = Vec::new();
        let mut encoder = DbnEncoder::new(
            buffer,
            &MetadataBuilder::new()
                .dataset(XNAS_ITCH.to_owned())
                .schema(Some(Schema::Ohlcv1S))
                .stype_in(Some(SType::RawSymbol))
                .stype_out(SType::InstrumentId)
                .start(0)
                .build(),
        )
        .unwrap();
        transcoder.write(encoder.get_ref().as_slice()).unwrap();
        let metadata_pos = encoder.get_ref().len();
        assert!(transcoder.has_decoded_metadata);
        let rec1 = ErrorMsg::new(1680708278000000000, "This is a test");
        let rec2 = OhlcvMsg {
            hd: RecordHeader::new::<OhlcvMsg>(rtype::OHLCV_1S, 1, 1, 1681228173000000000),
            open: 100,
            high: 200,
            low: 50,
            close: 150,
            volume: 1000,
        };
        encoder.encode_record(&rec1).unwrap();
        let rec1_pos = encoder.get_ref().len();
        encoder.encode_record(&rec2).unwrap();
        assert!(transcoder.buffer.get_ref().is_empty());
        // Write first record and part of second
        transcoder
            .write(&encoder.get_ref()[metadata_pos..rec1_pos + 4])
            .unwrap();
        // Write rest of second record
        transcoder
            .write(&encoder.get_ref()[rec1_pos + 4..])
            .unwrap();
        transcoder.flush().unwrap();
        let output = output_buf.lock().unwrap();
        let output = std::str::from_utf8(output.get_ref().as_slice()).unwrap();
        assert_eq!(output.chars().filter(|c| *c == '\n').count(), 2);
    }
}
