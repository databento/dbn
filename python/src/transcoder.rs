#![allow(clippy::too_many_arguments)] // many args aren't as bad in Python with kwargs

use std::{
    collections::HashMap,
    io::{BufWriter, Write},
    sync::{Arc, Mutex},
};

use dbn::{
    decode::dbn::fsm::{DbnFsm, ProcessResult},
    encode::{
        CsvEncoder, DbnMetadataEncoder, DbnRecordEncoder, DynWriter, EncodeRecordRef,
        EncodeRecordTextExt, JsonEncoder,
    },
    python::{py_to_time_date, to_py_err},
    Compression, Encoding, Metadata, PitSymbolMap, RType, Record, RecordRef, Schema, SymbolIndex,
    TsSymbolMap, VersionUpgradePolicy,
};
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyBytes, PyDate},
};

use crate::encode::PyFileLike;

#[pyclass(module = "databento_dbn")]
pub struct Transcoder(Mutex<Box<dyn Transcode + Send>>);

pub type PySymbolIntervalMap<'py> =
    HashMap<u32, Vec<(Bound<'py, PyDate>, Bound<'py, PyDate>, String)>>;

#[pymethods]
impl Transcoder {
    #[new]
    #[pyo3(signature  = (
        file,
        encoding,
        compression,
        pretty_px = true,
        pretty_ts = true,
        map_symbols = None,
        has_metadata = true,
        ts_out = false,
        symbol_interval_map = None,
        schema = None,
        input_version = None,
        upgrade_policy = VersionUpgradePolicy::default(),
    ))]
    fn new(
        file: PyFileLike,
        encoding: Encoding,
        compression: Compression,
        pretty_px: bool,
        pretty_ts: bool,
        map_symbols: Option<bool>,
        has_metadata: bool,
        ts_out: bool,
        symbol_interval_map: Option<PySymbolIntervalMap>,
        schema: Option<Schema>,
        input_version: Option<u8>,
        upgrade_policy: VersionUpgradePolicy,
    ) -> PyResult<Self> {
        let symbol_map = if let Some(symbol_interval_map) = symbol_interval_map {
            let mut symbol_map = TsSymbolMap::new();
            for (iid, py_intervals) in symbol_interval_map {
                for (start_date, end_date, symbol) in py_intervals {
                    if symbol.is_empty() {
                        continue;
                    }
                    let start_date = py_to_time_date(&start_date)?;
                    let end_date = py_to_time_date(&end_date)?;
                    symbol_map.insert(iid, start_date, end_date, Arc::new(symbol))?;
                }
            }
            Some(symbol_map)
        } else {
            None
        };
        Ok(Self(Mutex::new(match encoding {
            Encoding::Dbn => Box::new(Inner::<{ Encoding::Dbn as u8 }>::new(
                file,
                compression,
                pretty_px,
                pretty_ts,
                map_symbols,
                has_metadata,
                ts_out,
                symbol_map,
                schema,
                input_version,
                upgrade_policy,
            )?),
            Encoding::Csv => Box::new(Inner::<{ Encoding::Csv as u8 }>::new(
                file,
                compression,
                pretty_px,
                pretty_ts,
                map_symbols,
                has_metadata,
                ts_out,
                symbol_map,
                schema,
                input_version,
                upgrade_policy,
            )?),
            Encoding::Json => Box::new(Inner::<{ Encoding::Json as u8 }>::new(
                file,
                compression,
                pretty_px,
                pretty_ts,
                map_symbols,
                has_metadata,
                ts_out,
                symbol_map,
                schema,
                input_version,
                upgrade_policy,
            )?),
        })))
    }

    fn write(&mut self, bytes: &[u8]) -> PyResult<()> {
        self.0.lock().unwrap().write(bytes)
    }

    fn flush(&mut self) -> PyResult<()> {
        self.0.lock().unwrap().flush()
    }

    fn buffer<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, self.0.lock().unwrap().buffer())
    }
}

trait Transcode {
    fn write(&mut self, bytes: &[u8]) -> PyResult<()>;

    fn flush(&mut self) -> PyResult<()>;

    fn buffer(&self) -> &[u8];
}

struct Inner<const E: u8> {
    fsm: DbnFsm,
    // wrap in buffered writer to minimize calls to Python
    output: DynWriter<'static, BufWriter<PyFileLike>>,
    use_pretty_px: bool,
    use_pretty_ts: bool,
    map_symbols: bool,
    symbol_map: SymbolMap,
    schema: Option<Schema>,
}

impl<const E: u8> Transcode for Inner<E> {
    fn write(&mut self, bytes: &[u8]) -> PyResult<()> {
        self.fsm.write_all(bytes);
        self.encode()
    }

    fn flush(&mut self) -> PyResult<()> {
        self.encode()?;
        self.output.flush()?;
        Ok(())
    }

    fn buffer(&self) -> &[u8] {
        self.fsm.data()
    }
}

impl<const OUTPUT_ENC: u8> Inner<OUTPUT_ENC> {
    fn new(
        file: PyFileLike,
        compression: Compression,
        pretty_px: bool,
        pretty_ts: bool,
        map_symbols: Option<bool>,
        has_metadata: bool,
        ts_out: bool,
        symbol_map: Option<TsSymbolMap>,
        schema: Option<Schema>,
        input_version: Option<u8>,
        upgrade_policy: VersionUpgradePolicy,
    ) -> PyResult<Self> {
        if OUTPUT_ENC == Encoding::Dbn as u8 && map_symbols.unwrap_or(false) {
            return Err(PyValueError::new_err(
                "map_symbols=True is incompatible with DBN encoding",
            ));
        }
        let fsm = DbnFsm::builder()
            .skip_metadata(!has_metadata)
            .input_dbn_version(input_version)
            .map_err(to_py_err)?
            .upgrade_policy(upgrade_policy)
            .ts_out(ts_out)
            .build()
            .map_err(to_py_err)?;

        let mut output = DynWriter::new(BufWriter::new(file), compression)?;
        let map_symbols = map_symbols.unwrap_or(true);
        if !has_metadata {
            // if there's metadata, the header will be encoded when the metadata is processed
            Self::encode_header_if_csv(
                &mut output,
                pretty_px,
                pretty_ts,
                ts_out,
                map_symbols,
                upgrade_policy,
                input_version,
                schema,
            )?;
        }

        Ok(Self {
            fsm,
            output,
            use_pretty_px: pretty_px,
            use_pretty_ts: pretty_ts,
            map_symbols,
            symbol_map: symbol_map.map(SymbolMap::Historical).unwrap_or_default(),
            schema,
        })
    }

    fn encode(&mut self) -> PyResult<()> {
        loop {
            match self.fsm.process() {
                ProcessResult::ReadMore(_) => return Ok(()),
                ProcessResult::Err(e) => return Err(PyErr::from(e)),
                ProcessResult::Metadata(metadata) => self.encode_metadata(metadata)?,
                ProcessResult::Record(_) => {
                    if OUTPUT_ENC == Encoding::Dbn as u8 {
                        self.encode_dbn()
                    } else if OUTPUT_ENC == Encoding::Csv as u8 {
                        self.encode_csv()
                    } else {
                        self.encode_json()
                    }
                    .map_err(to_py_err)?;
                }
            }
        }
    }

    fn encode_dbn(&mut self) -> dbn::Result<()> {
        let mut encoder = DbnRecordEncoder::new(&mut self.output);
        let rec = self.fsm.last_record().unwrap();
        unsafe { encoder.encode_record_ref_ts_out(rec, self.fsm.ts_out()) }
    }

    fn encode_csv(&mut self) -> dbn::Result<()> {
        let mut encoder = CsvEncoder::builder(&mut self.output)
            .use_pretty_px(self.use_pretty_px)
            .use_pretty_ts(self.use_pretty_ts)
            .write_header(false)
            .build()?;
        let rec = self.fsm.last_record().unwrap();
        if self.map_symbols {
            self.symbol_map.update_live(rec);
        }
        // Filter by rtype based on metadata schema or schema parameter
        if rec
            .rtype()
            // Schema must be set for CSV. Checked in [`maybe_decode_metadata`]
            .map(|rtype| rtype == RType::from(self.schema.unwrap()))
            .unwrap_or(false)
        {
            if self.map_symbols {
                let symbol = self.symbol_map.get_for_rec(&rec).map(|s| s.as_str());
                unsafe { encoder.encode_ref_ts_out_with_sym(rec, self.fsm.ts_out(), symbol) }
            } else {
                unsafe { encoder.encode_record_ref_ts_out(rec, self.fsm.ts_out()) }
            }?;
        }
        Ok(())
    }

    fn encode_json(&mut self) -> dbn::Result<()> {
        let mut encoder = JsonEncoder::builder(&mut self.output)
            .use_pretty_px(self.use_pretty_px)
            .use_pretty_ts(self.use_pretty_ts)
            .build();
        let rec = self.fsm.last_record().unwrap();
        if self.map_symbols {
            self.symbol_map.update_live(rec);
            let symbol = self.symbol_map.get_for_rec(&rec).map(|s| s.as_str());
            unsafe { encoder.encode_ref_ts_out_with_sym(rec, self.fsm.ts_out(), symbol) }
        } else {
            unsafe { encoder.encode_record_ref_ts_out(rec, self.fsm.ts_out()) }
        }
    }

    // returns `false` if more data is required to decode the metadata
    fn encode_metadata(&mut self, metadata: Metadata) -> PyResult<()> {
        if self.schema.is_none() {
            self.schema = metadata.schema;
        }
        if OUTPUT_ENC == Encoding::Dbn as u8 {
            DbnMetadataEncoder::new(&mut self.output).encode(&metadata)?;
        // CSV or JSON
        } else if self.map_symbols {
            if metadata.schema.is_some() {
                // historical
                // only read from metadata mappings if symbol_map is unpopulated,
                // i.e. no `symbol_map` was passed in
                if self.symbol_map.is_empty() {
                    self.symbol_map = metadata.symbol_map().map(SymbolMap::Historical)?;
                }
            } else {
                // live
                self.symbol_map = SymbolMap::Live(Default::default());
            }
        }
        // decoding metadata and the header are both done once at the beginning
        Self::encode_header_if_csv(
            &mut self.output,
            self.use_pretty_px,
            self.use_pretty_ts,
            self.fsm.ts_out(),
            self.map_symbols,
            self.fsm.upgrade_policy(),
            self.fsm.input_dbn_version(),
            self.schema,
        )
    }

    fn encode_header_if_csv(
        output: &mut DynWriter<BufWriter<PyFileLike>>,
        use_pretty_px: bool,
        use_pretty_ts: bool,
        ts_out: bool,
        map_symbols: bool,
        upgrade_policy: VersionUpgradePolicy,
        input_version: Option<u8>,
        schema: Option<Schema>,
    ) -> PyResult<()> {
        if OUTPUT_ENC == Encoding::Csv as u8 {
            let Some(input_version) = input_version else {
                return Err(PyValueError::new_err(
                    "must specify input_version when has_metadata=False",
                ));
            };
            let output_version = upgrade_policy.output_version(input_version);
            let Some(schema) = schema else {
                return Err(PyValueError::new_err(
                    "A schema must be specified when transcoding mixed schema DBN to CSV",
                ));
            };
            let mut encoder = CsvEncoder::new(output, use_pretty_px, use_pretty_ts);
            encoder.encode_header_for_schema(output_version, schema, ts_out, map_symbols)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
enum SymbolMap {
    Historical(TsSymbolMap),
    Live(PitSymbolMap),
}

impl SymbolIndex for SymbolMap {
    fn get_for_rec<R: Record>(&self, record: &R) -> Option<&String> {
        match self {
            SymbolMap::Historical(sm) => sm.get_for_rec(record),
            SymbolMap::Live(sm) => sm.get_for_rec(record),
        }
    }
}

impl SymbolMap {
    fn is_empty(&self) -> bool {
        match self {
            SymbolMap::Historical(symbol_map) => symbol_map.is_empty(),
            SymbolMap::Live(symbol_map) => symbol_map.is_empty(),
        }
    }

    fn update_live(&mut self, rec: RecordRef) {
        let SymbolMap::Live(ref mut symbol_map) = self else {
            return;
        };
        // ignore errors
        let _ = symbol_map.on_record(rec);
    }
}

impl Default for SymbolMap {
    fn default() -> Self {
        Self::Historical(TsSymbolMap::default())
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Read, num::NonZeroU64};

    use dbn::{
        encode::{DbnEncoder, EncodeRecord},
        rtype, Dataset, ErrorMsg, MappingInterval, MetadataBuilder, OhlcvMsg, RecordHeader, SType,
        Schema, SymbolMapping, SymbolMappingMsg, WithTsOut, DBN_VERSION, UNDEF_TIMESTAMP,
    };
    use rstest::rstest;
    use time::macros::{date, datetime};

    use crate::{
        encode::tests::MockPyFile,
        tests::{setup, TEST_DATA_PATH},
    };

    use super::*;

    #[test]
    fn test_partial_metadata_and_records() {
        setup();
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut target = Python::with_gil(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                Encoding::Json,
                Compression::None,
                true,
                true,
                None,
                true,
                false,
                None,
                None,
                Some(DBN_VERSION),
                VersionUpgradePolicy::default(),
            )
            .unwrap()
        });
        let mut encoder = DbnEncoder::new(
            Vec::new(),
            &MetadataBuilder::new()
                .dataset(Dataset::XnasItch.to_string())
                .schema(Some(Schema::Trades))
                .stype_in(Some(SType::RawSymbol))
                .stype_out(SType::InstrumentId)
                .start(0)
                .build(),
        )
        .unwrap();
        let metadata_split = encoder.get_ref().len() / 2;
        target
            .write(&encoder.get_ref()[..metadata_split])
            // no error
            .unwrap();
        target.write(&encoder.get_ref()[metadata_split..]).unwrap();
        // Metadata doesn't get transcoded for JSON
        assert!(output_buf.lock().unwrap().get_ref().is_empty());
        let metadata_pos = encoder.get_ref().len();
        let rec = ErrorMsg::new(1680708278000000000, None, "This is a test", true);
        encoder.encode_record(&rec).unwrap();
        Python::with_gil(|py| {
            assert!(target.buffer(py).is_empty().unwrap());
        });
        let record_pos = encoder.get_ref().len();
        // write record byte by byte
        for i in metadata_pos..record_pos {
            target.write(&encoder.get_ref()[i..i + 1]).unwrap();
            // wrote last byte
            if i == record_pos - 1 {
                break;
            }
            Python::with_gil(|py| {
                assert_eq!(target.buffer(py).len().unwrap(), i + 1 - metadata_pos);
            });
        }
        // writing the remainder of the record should have the transcoder
        // transcode the record to the output file
        Python::with_gil(|py| {
            assert!(target.buffer(py).is_empty().unwrap());
        });
        assert_eq!(record_pos - metadata_pos, std::mem::size_of_val(&rec));
        target.flush().unwrap();
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
                true,
                true,
                None,
                true,
                false,
                None,
                None,
                Some(DBN_VERSION),
                VersionUpgradePolicy::default(),
            )
            .unwrap()
        });
        let buffer = Vec::new();
        let mut encoder = DbnEncoder::new(
            buffer,
            &MetadataBuilder::new()
                .dataset(Dataset::XnasItch.to_string())
                .schema(Some(Schema::Ohlcv1S))
                .stype_in(Some(SType::RawSymbol))
                .stype_out(SType::InstrumentId)
                .start(0)
                .build(),
        )
        .unwrap();
        transcoder.write(encoder.get_ref().as_slice()).unwrap();
        let metadata_pos = encoder.get_ref().len();
        let rec1 = ErrorMsg::new(1680708278000000000, None, "This is a test", true);
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
        Python::with_gil(|py| {
            assert!(transcoder.buffer(py).is_empty().unwrap());
        });
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
        // header + 1 record, ErrorMsg ignored because of a different schema
        dbg!(&output);
        assert_eq!(output.chars().filter(|c| *c == '\n').count(), 2);
    }

    #[rstest]
    #[case::csv(Encoding::Csv, false)]
    #[case::csv_map_symbols(Encoding::Csv, true)]
    #[case::json(Encoding::Json, false)]
    #[case::json_map_symbols(Encoding::Json, true)]
    fn test_map_symbols_historical(#[case] encoding: Encoding, #[case] map_symbols: bool) {
        setup();
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut transcoder = Python::with_gil(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                encoding,
                Compression::None,
                true,
                false,
                Some(map_symbols),
                true,
                true,
                None,
                None,
                None,
                VersionUpgradePolicy::default(),
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
                .dataset(Dataset::XnasItch.to_string())
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
        Python::with_gil(|py| {
            assert!(transcoder.buffer(py).is_empty().unwrap());
        });
        // Write first record and part of second
        transcoder.write(encoder.get_ref()).unwrap();
        transcoder.flush().unwrap();
        let output = output_buf.lock().unwrap();
        let output = std::str::from_utf8(output.get_ref().as_slice()).unwrap();
        let lines = output.lines().collect::<Vec<_>>();
        dbg!(&lines);
        if encoding == Encoding::Csv {
            assert_eq!(lines.len(), 3);
            if map_symbols {
                assert!(lines[0].ends_with(",ts_out,symbol"));
                assert!(lines[1].contains(",1,NFLX"));
                assert!(lines[2].contains(",2,QQQ"));
            } else {
                assert!(lines[0].ends_with(",ts_out"));
                assert!(lines[1].ends_with(",1"));
                assert!(lines[2].ends_with(",2"));
            }
        } else {
            assert_eq!(lines.len(), 2);
            assert_eq!(lines[0].contains("\"symbol\":\"NFLX\""), map_symbols);
            assert!(lines[0].contains("\"ts_out\":\"1\""));
            assert_eq!(lines[1].contains("\"symbol\":\"QQQ\""), map_symbols);
            assert!(lines[1].contains("\"ts_out\":\"2\""));
        }
    }

    #[rstest]
    #[case::csv(Encoding::Csv, false)]
    #[case::csv_map_symbols(Encoding::Csv, true)]
    #[case::json(Encoding::Json, false)]
    #[case::json_map_symbols(Encoding::Json, true)]
    fn test_map_symbols_live(#[case] encoding: Encoding, #[case] map_symbols: bool) {
        setup();
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut transcoder = Python::with_gil(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                encoding,
                Compression::None,
                true,
                true,
                Some(map_symbols),
                true,
                false,
                None,
                Some(Schema::Ohlcv1S),
                None,
                VersionUpgradePolicy::default(),
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
                .dataset(Dataset::XnasItch.to_string())
                .schema(None) // Live: mixed schema
                .stype_in(Some(SType::RawSymbol))
                .stype_out(SType::InstrumentId)
                .start(datetime!(2023-09-27 00:00:00 UTC).unix_timestamp_nanos() as u64)
                .end(None)
                .build(),
        )
        .unwrap();
        let rec1 = OhlcvMsg {
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
        };
        let rec2 = OhlcvMsg {
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
        };
        encoder
            .encode_record(
                &SymbolMappingMsg::new(
                    NFLX_ID,
                    0,
                    SType::RawSymbol,
                    NFLX,
                    SType::RawSymbol,
                    NFLX,
                    0,
                    UNDEF_TIMESTAMP,
                )
                .unwrap(),
            )
            .unwrap();
        encoder
            .encode_record(
                &SymbolMappingMsg::new(
                    QQQ_ID,
                    1,
                    SType::RawSymbol,
                    QQQ,
                    SType::RawSymbol,
                    QQQ,
                    1,
                    UNDEF_TIMESTAMP,
                )
                .unwrap(),
            )
            .unwrap();
        encoder.encode_record(&rec1).unwrap();
        encoder.encode_record(&rec2).unwrap();
        Python::with_gil(|py| {
            assert!(transcoder.buffer(py).is_empty().unwrap());
        });
        // Write first record and part of second
        transcoder.write(encoder.get_ref()).unwrap();
        transcoder.flush().unwrap();
        let output = output_buf.lock().unwrap();
        let output = std::str::from_utf8(output.get_ref().as_slice()).unwrap();
        let lines = output.lines().collect::<Vec<_>>();
        dbg!(&lines);
        if encoding == Encoding::Csv {
            assert_eq!(lines.len(), 3);
            if map_symbols {
                assert!(lines[0].ends_with(",symbol"));
            } else {
                assert!(lines[0].ends_with(",volume"));
            }
            assert_eq!(lines[1].contains(",NFLX"), map_symbols);
            assert_eq!(lines[2].contains(",QQQ"), map_symbols);
        } else {
            assert_eq!(lines.len(), 4);
            assert!(lines[0].contains("\"stype_out_symbol\":\"NFLX\""));
            assert!(lines[1].contains("\"stype_out_symbol\":\"QQQ\""));
            assert_eq!(lines[2].contains("\"symbol\":\"NFLX\""), map_symbols);
            assert_eq!(lines[3].contains("\"symbol\":\"QQQ\""), map_symbols);
        }
    }

    #[rstest]
    #[case::csv_mbo(Encoding::Csv, Schema::Mbo)]
    #[case::csv_definition(Encoding::Csv, Schema::Definition)]
    #[case::csv_trades(Encoding::Csv, Schema::Trades)]
    fn test_from_test_data_file(#[case] encoding: Encoding, #[case] schema: Schema) {
        setup();

        let input = zstd::stream::decode_all(
            std::fs::File::open(format!("{TEST_DATA_PATH}/test_data.{schema}.v3.dbn.zst")).unwrap(),
        )
        .unwrap();
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut transcoder = Python::with_gil(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                encoding,
                Compression::None,
                true,
                true,
                None,
                true,
                false,
                None,
                Some(schema),
                None,
                VersionUpgradePolicy::default(),
            )
            .unwrap()
        });
        // Write first record and part of second
        transcoder.write(&input).unwrap();
        transcoder.flush().unwrap();
        let output = output_buf.lock().unwrap();
        let output = std::str::from_utf8(output.get_ref().as_slice()).unwrap();
        let lines = output.lines().collect::<Vec<_>>();
        dbg!(&lines);
        if encoding == Encoding::Csv {
            assert_eq!(lines.len(), 3);
            assert!(lines[0].ends_with(",symbol"));
            // ensure ends with a symbol not an empty cell
            assert!(!lines[1].ends_with(','));
            assert!(!lines[2].ends_with(','));
        }
    }

    #[rstest]
    fn test_skip_metadata_csv_header_still_written() {
        setup();

        let mut input = Vec::new();
        let mut input_file =
            std::fs::File::open(format!("{TEST_DATA_PATH}/test_data.definition.v3.dbn.frag"))
                .unwrap();
        input_file.read_to_end(&mut input).unwrap();
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut transcoder = Python::with_gil(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                Encoding::Csv,
                Compression::None,
                true,
                true,
                None,
                false,
                false,
                None,
                Some(Schema::Definition),
                Some(3),
                VersionUpgradePolicy::default(),
            )
            .unwrap()
        });
        // Write first record and part of second
        transcoder.write(&input).unwrap();
        transcoder.flush().unwrap();
        let output = output_buf.lock().unwrap();
        let output = std::str::from_utf8(output.get_ref().as_slice()).unwrap();
        let lines = output.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].ends_with(",symbol"));
        // ends an empty field because there was no metadata and no symbol map was provided
        assert!(lines[1].ends_with(','));
        assert!(lines[2].ends_with(','));
    }
}
