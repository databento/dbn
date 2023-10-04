#![allow(clippy::too_many_arguments)] // many args aren't as bad in Python with kwargs

use std::{
    collections::HashMap,
    io::{self, BufWriter, Write},
};

use dbn::{
    decode::{
        zstd::starts_with_prefix, DbnMetadataDecoder, DbnRecordDecoder, DecodeRecordRef, DynReader,
    },
    encode::{
        CsvEncoder, DbnMetadataEncoder, DbnRecordEncoder, DynWriter, EncodeRecordRef,
        EncodeRecordTextExt, JsonEncoder,
    },
    enums::{Compression, Encoding},
    python::{py_to_time_date, to_val_err},
};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyDate};

use crate::encode::PyFileLike;

#[pyclass(module = "databento_dbn")]
pub struct Transcoder(Box<dyn Transcode + Send>);

pub type PySymbolMap<'py> = HashMap<u32, Vec<(&'py PyDate, &'py PyDate, String)>>;

#[pymethods]
impl Transcoder {
    #[new]
    fn new(
        file: PyFileLike,
        encoding: Encoding,
        compression: Compression,
        pretty_px: Option<bool>,
        pretty_ts: Option<bool>,
        map_symbols: Option<bool>,
        has_metadata: Option<bool>,
        ts_out: Option<bool>,
        input_compression: Option<Compression>,
        symbol_map: Option<PySymbolMap>,
    ) -> PyResult<Self> {
        let symbol_map = if let Some(py_symbol_map) = symbol_map {
            let mut symbol_map = HashMap::new();
            for (iid, py_intervals) in py_symbol_map {
                for (start_date, end_date, symbol) in py_intervals {
                    if symbol.is_empty() {
                        continue;
                    }
                    let start_date = py_to_time_date(start_date)?;
                    let end_date = py_to_time_date(end_date)?;
                    let mut day = start_date;
                    loop {
                        symbol_map.insert((day, iid), symbol.clone());
                        day = day.next_day().unwrap();
                        if day == end_date {
                            break;
                        }
                    }
                }
            }
            Some(symbol_map)
        } else {
            None
        };
        Ok(Self(match encoding {
            Encoding::Dbn => Box::new(Inner::<{ Encoding::Dbn as u8 }>::new(
                file,
                compression,
                pretty_px,
                pretty_ts,
                map_symbols,
                has_metadata,
                ts_out,
                input_compression,
                symbol_map,
            )?),
            Encoding::Csv => Box::new(Inner::<{ Encoding::Csv as u8 }>::new(
                file,
                compression,
                pretty_px,
                pretty_ts,
                map_symbols,
                has_metadata,
                ts_out,
                input_compression,
                symbol_map,
            )?),
            Encoding::Json => Box::new(Inner::<{ Encoding::Json as u8 }>::new(
                file,
                compression,
                pretty_px,
                pretty_ts,
                map_symbols,
                has_metadata,
                ts_out,
                input_compression,
                symbol_map,
            )?),
        }))
    }

    fn write(&mut self, bytes: &[u8]) -> PyResult<()> {
        self.0.write(bytes)
    }

    fn flush(&mut self) -> PyResult<()> {
        self.0.flush()
    }

    fn buffer(&self) -> &[u8] {
        self.0.buffer()
    }
}

trait Transcode {
    fn write(&mut self, bytes: &[u8]) -> PyResult<()>;

    fn flush(&mut self) -> PyResult<()>;

    fn buffer(&self) -> &[u8];
}

struct Inner<const E: u8> {
    buffer: io::Cursor<Vec<u8>>,
    // wrap in buffered writer to minimize calls to Python
    output: DynWriter<'static, BufWriter<PyFileLike>>,
    use_pretty_px: bool,
    use_pretty_ts: bool,
    map_symbols: bool,
    has_decoded_metadata: bool,
    ts_out: bool,
    input_compression: Option<Compression>,
    symbol_map: HashMap<(time::Date, u32), String>,
}

impl<const E: u8> Transcode for Inner<E> {
    fn write(&mut self, bytes: &[u8]) -> PyResult<()> {
        self.buffer.write_all(bytes).map_err(to_val_err)?;
        self.encode()
    }

    fn flush(&mut self) -> PyResult<()> {
        self.encode()?;
        self.output.flush()?;
        Ok(())
    }

    fn buffer(&self) -> &[u8] {
        self.buffer.get_ref().as_slice()
    }
}

impl<const OUTPUT_ENC: u8> Inner<OUTPUT_ENC> {
    fn new(
        file: PyFileLike,
        compression: Compression,
        pretty_px: Option<bool>,
        pretty_ts: Option<bool>,
        map_symbols: Option<bool>,
        has_metadata: Option<bool>,
        ts_out: Option<bool>,
        input_compression: Option<Compression>,
        symbol_map: Option<HashMap<(time::Date, u32), String>>,
    ) -> PyResult<Self> {
        if OUTPUT_ENC == Encoding::Dbn as u8 && map_symbols.unwrap_or(false) {
            return Err(PyValueError::new_err(
                "map_symbols=True is incompatible with DBN encoding",
            ));
        }
        Ok(Self {
            buffer: io::Cursor::default(),
            output: DynWriter::new(BufWriter::new(file), compression).map_err(to_val_err)?,
            use_pretty_px: pretty_px.unwrap_or(true),
            use_pretty_ts: pretty_ts.unwrap_or(true),
            map_symbols: map_symbols.unwrap_or(true),
            has_decoded_metadata: !has_metadata.unwrap_or(true),
            ts_out: ts_out.unwrap_or(false),
            input_compression,
            symbol_map: symbol_map.unwrap_or_default(),
        })
    }

    fn encode(&mut self) -> PyResult<()> {
        let orig_position = self.buffer.position();
        self.buffer.set_position(0);
        if !self.detect_compression() {
            return Ok(());
        }
        self.maybe_decode_metadata(orig_position)?;
        let read_position = if OUTPUT_ENC == Encoding::Dbn as u8 {
            self.encode_dbn(orig_position)
        } else if OUTPUT_ENC == Encoding::Csv as u8 {
            self.encode_csv(orig_position)
        } else {
            self.encode_json(orig_position)
        }?;
        self.shift_buffer(read_position);
        Ok(())
    }

    fn encode_dbn(&mut self, orig_position: u64) -> PyResult<usize> {
        let mut read_position = self.buffer.position() as usize;
        // Ok to unwrap `input_compression` because it will be set in `detect_compression`
        let mut decoder = DbnRecordDecoder::new(
            DynReader::with_buffer(&mut self.buffer, self.input_compression.unwrap())
                .map_err(to_val_err)?,
        );
        let mut encoder = DbnRecordEncoder::new(&mut self.output);
        loop {
            match decoder.decode_record_ref() {
                Ok(Some(rec)) => {
                    unsafe { encoder.encode_record_ref_ts_out(rec, self.ts_out) }
                        .map_err(to_val_err)?;
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
        Ok(read_position)
    }

    fn encode_csv(&mut self, orig_position: u64) -> PyResult<usize> {
        let mut read_position = self.buffer.position() as usize;
        // Ok to unwrap `input_compression` because it will be set in `detect_compression`
        let mut decoder = DbnRecordDecoder::new(
            DynReader::with_buffer(&mut self.buffer, self.input_compression.unwrap())
                .map_err(to_val_err)?,
        );

        let mut encoder = CsvEncoder::new(&mut self.output, self.use_pretty_px, self.use_pretty_ts);
        loop {
            match decoder.decode_record_ref() {
                Ok(Some(rec)) => {
                    let symbol = rec.header().ts_event().and_then(|ts_event| {
                        self.symbol_map
                            .get(&(ts_event.date(), rec.header().instrument_id))
                            .map(|s| s.as_str())
                    });
                    unsafe { encoder.encode_ref_ts_out_with_sym(rec, self.ts_out, symbol) }
                        .map_err(to_val_err)?;
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
        Ok(read_position)
    }

    fn encode_json(&mut self, orig_position: u64) -> PyResult<usize> {
        let mut read_position = self.buffer.position() as usize;
        // Ok to unwrap `input_compression` because it will be set in `detect_compression`
        let mut decoder = DbnRecordDecoder::new(
            DynReader::with_buffer(&mut self.buffer, self.input_compression.unwrap())
                .map_err(to_val_err)?,
        );

        let mut encoder = JsonEncoder::new(
            &mut self.output,
            false,
            self.use_pretty_px,
            self.use_pretty_ts,
        );
        loop {
            match decoder.decode_record_ref() {
                Ok(Some(rec)) => {
                    let symbol = rec.header().ts_event().and_then(|ts_event| {
                        self.symbol_map
                            .get(&(ts_event.date(), rec.header().instrument_id))
                            .map(|s| s.as_str())
                    });
                    unsafe { encoder.encode_ref_ts_out_with_sym(rec, self.ts_out, symbol) }
                        .map_err(to_val_err)?;
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
        Ok(read_position)
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
                    if OUTPUT_ENC == Encoding::Dbn as u8 {
                        DbnMetadataEncoder::new(&mut self.output)
                            .encode(&metadata)
                            .map_err(to_val_err)?;
                    // CSV or JSON
                    } else if self.map_symbols {
                        self.symbol_map = metadata.symbol_map().map_err(to_val_err)?;
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
    use std::num::NonZeroU64;

    use dbn::{
        datasets::XNAS_ITCH,
        encode::{DbnEncoder, EncodeRecord},
        enums::{rtype, SType, Schema},
        record::{ErrorMsg, OhlcvMsg, RecordHeader},
        MappingInterval, MetadataBuilder, SymbolMapping, WithTsOut,
    };
    use time::macros::{date, datetime};

    use crate::{encode::tests::MockPyFile, tests::setup};

    use super::*;

    impl Transcoder {
        fn downcast_unchecked<const E: u8>(&self) -> &Inner<E> {
            unsafe {
                let ptr = &*self.0 as *const (dyn Transcode + Send);
                ptr.cast::<Inner<E>>().as_ref().unwrap()
            }
        }
    }

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
                None,
                None,
            )
            .unwrap()
        });
        assert!(
            !transcoder
                .downcast_unchecked::<{ Encoding::Json as u8 }>()
                .has_decoded_metadata
        );
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
        assert!(
            transcoder
                .downcast_unchecked::<{ Encoding::Json as u8 }>()
                .has_decoded_metadata
        );
        let metadata_pos = encoder.get_ref().len();
        let rec = ErrorMsg::new(1680708278000000000, "This is a test");
        encoder.encode_record(&rec).unwrap();
        assert!(transcoder.buffer().is_empty());
        let record_pos = encoder.get_ref().len();
        // write record byte by byte
        for i in metadata_pos..record_pos {
            transcoder.write(&encoder.get_ref()[i..i + 1]).unwrap();
            // wrote last byte
            if i == record_pos - 1 {
                break;
            }
            assert_eq!(transcoder.buffer().len(), i + 1 - metadata_pos);
        }
        // writing the remainder of the record should have the transcoder
        // transcode the record to the output file
        assert!(transcoder.buffer().is_empty());
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
        assert!(
            transcoder
                .downcast_unchecked::<{ Encoding::Csv as u8 }>()
                .has_decoded_metadata
        );
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
        assert!(transcoder.buffer().is_empty());
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

    #[test]
    fn test_map_symbols() {
        setup();
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut transcoder = Python::with_gil(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                Encoding::Json,
                Compression::None,
                None,
                Some(false),
                Some(true),
                None,
                Some(true),
                None,
                None,
            )
            .unwrap()
        });
        const QQQ: &str = "QQQ";
        const QQQ_ID: u32 = 48933;
        const NFLX: &str = "NFLX";
        const NFLX_ID: u32 = 58501;
        let buffer = Vec::new();
        let mut encoder = DbnEncoder::new(
            buffer,
            &MetadataBuilder::new()
                .dataset(XNAS_ITCH.to_owned())
                .schema(Some(Schema::Ohlcv1S))
                .stype_in(Some(SType::RawSymbol))
                .stype_out(SType::InstrumentId)
                .ts_out(true)
                .start(datetime!(2023-09-27 00:00:00 UTC).unix_timestamp_nanos() as u64)
                .end(NonZeroU64::new(
                    datetime!(2023-09-29 00:00:00 UTC).unix_timestamp_nanos() as u64,
                ))
                .mappings(vec![
                    SymbolMapping {
                        raw_symbol: QQQ.to_owned(),
                        intervals: vec![MappingInterval {
                            start_date: date!(2023 - 09 - 27),
                            end_date: date!(2023 - 09 - 29),
                            symbol: format!("{QQQ_ID}"),
                        }],
                    },
                    SymbolMapping {
                        raw_symbol: NFLX.to_owned(),
                        intervals: vec![MappingInterval {
                            start_date: date!(2023 - 09 - 27),
                            end_date: date!(2023 - 09 - 29),
                            symbol: format!("{NFLX_ID}"),
                        }],
                    },
                ])
                .build(),
        )
        .unwrap();
        let rec1 = WithTsOut::new(
            OhlcvMsg {
                hd: RecordHeader::new::<OhlcvMsg>(
                    rtype::OHLCV_1S,
                    1,
                    NFLX_ID,
                    datetime!(2023-09-27 00:10:07 UTC).unix_timestamp_nanos() as u64,
                ),
                open: 100,
                high: 200,
                low: 50,
                close: 150,
                volume: 1000,
            },
            1,
        );
        let rec2 = WithTsOut::new(
            OhlcvMsg {
                hd: RecordHeader::new::<OhlcvMsg>(
                    rtype::OHLCV_1S,
                    1,
                    QQQ_ID,
                    datetime!(2023-09-27 00:10:10 UTC).unix_timestamp_nanos() as u64,
                ),
                open: 100,
                high: 200,
                low: 50,
                close: 150,
                volume: 1000,
            },
            2,
        );
        encoder.encode_record(&rec1).unwrap();
        encoder.encode_record(&rec2).unwrap();
        assert!(transcoder.buffer().is_empty());
        // Write first record and part of second
        transcoder.write(&encoder.get_ref()).unwrap();
        transcoder.flush().unwrap();
        let output = output_buf.lock().unwrap();
        let output = std::str::from_utf8(output.get_ref().as_slice()).unwrap();
        let lines = output.lines().collect::<Vec<_>>();
        dbg!(&lines);
        assert!(lines[0].contains("\"symbol\":\"NFLX\""));
        assert!(lines[0].contains("\"ts_out\":\"1\""));
        assert!(lines[1].contains("\"symbol\":\"QQQ\""));
        assert!(lines[1].contains("\"ts_out\":\"2\""));
    }
}
