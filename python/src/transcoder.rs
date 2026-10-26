#![allow(clippy::too_many_arguments)] // many args aren't as bad in Python with kwargs

use std::{
    collections::HashMap,
    io::{BufWriter, Write},
    mem,
};

use dbn::{
    decode::dbn::fsm::{DbnFsm, ProcessResult},
    encode::{
        CsvEncoder, DbnMetadataEncoder, DbnRecordEncoder, DynEncoder, DynWriter, EncodeRecord,
        EncodeRecordRef, EncodeRecordTextExt, JsonEncoder, OutputEncoding,
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
use sync_wrapper::SyncWrapper;

use crate::encode::PyFileLike;

#[pyclass(module = "databento_dbn")]
pub struct Transcoder(SyncWrapper<Box<dyn Transcode + Send>>);

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
        encoding: EncodingArg,
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
                    let start_ts = py_to_time_date(&start_date)?
                        .midnight()
                        .assume_utc()
                        .unix_timestamp_nanos() as u64;
                    let end_ts = py_to_time_date(&end_date)?
                        .midnight()
                        .assume_utc()
                        .unix_timestamp_nanos() as u64;
                    symbol_map.insert(iid, start_ts, end_ts, symbol)?;
                }
            }
            Some(symbol_map)
        } else {
            None
        };
        let encoding = OutputEncoding::try_from(encoding)?;
        Ok(Self(SyncWrapper::new(match encoding {
            OutputEncoding::Dbn => Box::new(Inner::<{ Encoding::Dbn as u8 }>::new(
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
            OutputEncoding::Csv => Box::new(Inner::<{ Encoding::Csv as u8 }>::new(
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
            OutputEncoding::Json => Box::new(Inner::<{ Encoding::Json as u8 }>::new(
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
            encoding => Box::new(ColumnarInner::new(
                file,
                encoding,
                compression,
                pretty_px,
                map_symbols,
                has_metadata,
                symbol_map,
                schema,
                input_version,
                upgrade_policy,
            )?),
        })))
    }

    fn write(&mut self, py: Python<'_>, bytes: &[u8]) -> PyResult<()> {
        py.detach(|| self.0.get_mut().write(bytes))
    }

    fn flush(&mut self, py: Python<'_>) -> PyResult<()> {
        py.detach(|| self.0.get_mut().flush())
    }

    /// Finalize the output stream, writing any epilogue required by the
    /// compression, i.e. the Zstandard end-of-frame block and checksum.
    ///
    /// Calling `finish()` more than once has no effect.
    /// Writing to or flushing a finished transcoder raises a `ValueError`.
    fn finish(&mut self, py: Python<'_>) -> PyResult<()> {
        py.detach(|| self.0.get_mut().finish())
    }

    fn buffer<'py>(&mut self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, self.0.get_mut().buffer())
    }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __exit__(
        &mut self,
        py: Python<'_>,
        exc_type: Option<&Bound<'_, PyAny>>,
        _exc_value: Option<&Bound<'_, PyAny>>,
        _traceback: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        if exc_type.is_none() {
            self.finish(py)
        } else {
            Ok(())
        }
    }
}

trait Transcode {
    fn write(&mut self, bytes: &[u8]) -> PyResult<()>;

    fn flush(&mut self) -> PyResult<()>;

    fn finish(&mut self) -> PyResult<()>;

    fn buffer(&self) -> &[u8];
}

struct Inner<const E: u8> {
    fsm: DbnFsm,
    // wrap in buffered writer to minimize calls to Python
    output: DynWriter<'static, BufWriter<PyFileLike>>,
    finished: bool,
    use_pretty_px: bool,
    use_pretty_ts: bool,
    map_symbols: bool,
    symbol_map: SymbolMap,
    schema: Option<Schema>,
}

impl<const E: u8> Transcode for Inner<E> {
    fn write(&mut self, bytes: &[u8]) -> PyResult<()> {
        if self.finished {
            return Err(finished_err());
        }
        self.fsm.write_all(bytes);
        self.encode()
    }

    fn flush(&mut self) -> PyResult<()> {
        if self.finished {
            return Err(finished_err());
        }
        self.encode()?;
        self.output.flush()?;
        Ok(())
    }

    fn finish(&mut self) -> PyResult<()> {
        if self.finished {
            return Ok(());
        }
        self.encode()?;
        self.output.finish().map_err(PyErr::from)?;
        self.finished = true;
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
            finished: false,
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
            .with_symbol(self.map_symbols)
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
            let symbol_map = self.map_symbols.then_some(&self.symbol_map);
            unsafe { encode_with_symbol(&mut encoder, symbol_map, rec, self.fsm.ts_out()) }?;
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
        }
        let symbol_map = self.map_symbols.then_some(&self.symbol_map);
        unsafe { encode_with_symbol(&mut encoder, symbol_map, rec, self.fsm.ts_out()) }
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
            self.symbol_map.init_from_metadata(&metadata)?;
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

#[derive(FromPyObject)]
enum EncodingArg {
    Encoding(Encoding),
    Columnar(String),
}

impl TryFrom<EncodingArg> for OutputEncoding {
    type Error = PyErr;

    fn try_from(arg: EncodingArg) -> PyResult<Self> {
        match arg {
            EncodingArg::Encoding(encoding) => Ok(encoding.into()),
            EncodingArg::Columnar(name) => match name.as_str() {
                "arrow" => Ok(Self::Arrow),
                "parquet" => Ok(Self::Parquet),
                _ => Err(PyValueError::new_err(format!(
                    "unknown encoding '{name}', expected an Encoding, 'arrow', or 'parquet'"
                ))),
            },
        }
    }
}

struct ColumnarInner {
    fsm: DbnFsm,
    state: ColumnarState,
    encoding: OutputEncoding,
    compression: Compression,
    use_pretty_px: bool,
    map_symbols: bool,
    symbol_map: SymbolMap,
    schema: Option<Schema>,
}

enum ColumnarState {
    AwaitingMetadata(BufWriter<PyFileLike>),
    Encoding {
        encoder: Box<DynEncoder<'static, BufWriter<PyFileLike>>>,
        rtype: RType,
    },
    Finished,
}

impl Transcode for ColumnarInner {
    fn write(&mut self, bytes: &[u8]) -> PyResult<()> {
        if matches!(self.state, ColumnarState::Finished) {
            return Err(finished_err());
        }
        self.fsm.write_all(bytes);
        self.encode()
    }

    fn flush(&mut self) -> PyResult<()> {
        if matches!(self.state, ColumnarState::Finished) {
            return Err(finished_err());
        }
        self.encode()?;
        if let ColumnarState::Encoding { encoder, .. } = &mut self.state {
            encoder.flush().map_err(to_py_err)?;
        }
        Ok(())
    }

    fn finish(&mut self) -> PyResult<()> {
        if matches!(self.state, ColumnarState::Finished) {
            return Ok(());
        }
        self.encode()?;
        match mem::replace(&mut self.state, ColumnarState::Finished) {
            ColumnarState::AwaitingMetadata(mut writer) => writer.flush()?,
            ColumnarState::Encoding { encoder, .. } => encoder.finish().map_err(to_py_err)?,
            ColumnarState::Finished => {}
        }
        Ok(())
    }

    fn buffer(&self) -> &[u8] {
        self.fsm.data()
    }
}

impl ColumnarInner {
    fn new(
        file: PyFileLike,
        encoding: OutputEncoding,
        compression: Compression,
        pretty_px: bool,
        map_symbols: Option<bool>,
        has_metadata: bool,
        symbol_map: Option<TsSymbolMap>,
        schema: Option<Schema>,
        input_version: Option<u8>,
        upgrade_policy: VersionUpgradePolicy,
    ) -> PyResult<Self> {
        if !has_metadata {
            return Err(PyValueError::new_err(
                "has_metadata=False is incompatible with Parquet and Arrow encodings",
            ));
        }
        let fsm = DbnFsm::builder()
            .input_dbn_version(input_version)
            .map_err(to_py_err)?
            .upgrade_policy(upgrade_policy)
            .build()
            .map_err(to_py_err)?;
        Ok(Self {
            fsm,
            state: ColumnarState::AwaitingMetadata(BufWriter::new(file)),
            encoding,
            compression,
            use_pretty_px: pretty_px,
            map_symbols: map_symbols.unwrap_or(true),
            symbol_map: symbol_map.map(SymbolMap::Historical).unwrap_or_default(),
            schema,
        })
    }

    fn encode(&mut self) -> PyResult<()> {
        loop {
            match self.fsm.process() {
                ProcessResult::ReadMore(_) => return Ok(()),
                ProcessResult::Err(e) => return Err(PyErr::from(e)),
                ProcessResult::Metadata(metadata) => self.build_encoder(metadata)?,
                ProcessResult::Record(_) => self.encode_record().map_err(to_py_err)?,
            }
        }
    }

    fn build_encoder(&mut self, mut metadata: Metadata) -> PyResult<()> {
        if self.map_symbols {
            self.symbol_map.init_from_metadata(&metadata)?;
        }
        if self.schema.is_some() {
            metadata.schema = self.schema;
        }
        let Some(schema) = metadata.schema else {
            return Err(PyValueError::new_err(
                "A schema must be specified when transcoding mixed schema DBN to Parquet or Arrow",
            ));
        };
        let ColumnarState::AwaitingMetadata(writer) =
            mem::replace(&mut self.state, ColumnarState::Finished)
        else {
            return Err(PyValueError::new_err(
                "Transcoder received a second DBN metadata header",
            ));
        };
        let encoder = DynEncoder::builder(writer, self.encoding, self.compression, &metadata)
            .use_pretty_px(self.use_pretty_px)
            .with_symbol(self.map_symbols)
            .build()
            .map_err(to_py_err)?;
        self.state = ColumnarState::Encoding {
            encoder: Box::new(encoder),
            rtype: RType::from(schema),
        };
        Ok(())
    }

    fn encode_record(&mut self) -> dbn::Result<()> {
        let ColumnarState::Encoding { encoder, rtype } = &mut self.state else {
            return Err(dbn::Error::encode("received a record before DBN metadata"));
        };
        let rec = self.fsm.last_record().unwrap();
        if self.map_symbols {
            self.symbol_map.update_live(rec);
        }
        if rec.rtype().is_ok_and(|r| r == *rtype) {
            let symbol_map = self.map_symbols.then_some(&self.symbol_map);
            unsafe { encode_with_symbol(encoder.as_mut(), symbol_map, rec, self.fsm.ts_out()) }?;
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

    fn init_from_metadata(&mut self, metadata: &Metadata) -> PyResult<()> {
        if metadata.schema.is_some() {
            // historical
            // only read from metadata mappings if symbol_map is unpopulated,
            // i.e. no `symbol_map` was passed in
            if self.is_empty() {
                *self = metadata.symbol_map().map(SymbolMap::Historical)?;
            }
        } else {
            // live
            *self = SymbolMap::Live(Default::default());
        }
        Ok(())
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

unsafe fn encode_with_symbol(
    encoder: &mut impl EncodeRecordTextExt,
    symbol_map: Option<&SymbolMap>,
    rec: RecordRef,
    ts_out: bool,
) -> dbn::Result<()> {
    if let Some(symbol_map) = symbol_map {
        let symbol = symbol_map.get_for_rec(&rec).map(|s| s.as_str());
        unsafe { encoder.encode_ref_ts_out_with_sym(rec, ts_out, symbol) }
    } else {
        unsafe { encoder.encode_record_ref_ts_out(rec, ts_out) }
    }
}

fn finished_err() -> PyErr {
    PyValueError::new_err("I/O operation on finished Transcoder")
}

#[cfg(test)]
mod tests {
    use std::{io::Read, num::NonZeroU64};

    use arrow::{
        array::{AsArray, RecordBatch},
        datatypes::{Float64Type, TimestampNanosecondType},
        ipc::reader::FileReader,
    };

    use dbn::{
        encode::{DbnEncoder, EncodeRecord},
        rtype, Dataset, ErrorMsg, MappingInterval, MetadataBuilder, OhlcvMsg, RecordHeader, SType,
        Schema, SymbolMapping, SymbolMappingMsg, WithTsOut, DBN_VERSION, UNDEF_TIMESTAMP,
    };
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use rstest::*;
    use time::macros::{date, datetime};

    use crate::{
        encode::tests::MockPyFile,
        tests::{python, TEST_DATA_PATH},
    };

    use super::*;

    #[rstest]
    fn test_zstd_frame_is_finished(_python: ()) {
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut transcoder = Python::attach(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                EncodingArg::Encoding(Encoding::Dbn),
                Compression::Zstd,
                false,
                false,
                Some(false),
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
        encoder
            .encode_record(&ErrorMsg::new(
                1680708278000000000,
                None,
                "This is a test",
                true,
            ))
            .unwrap();
        Python::attach(|py| transcoder.write(py, encoder.get_ref().as_slice())).unwrap();
        // flush() alone cannot terminate the zstd frame
        Python::attach(|py| transcoder.flush(py)).unwrap();
        let flushed = output_buf.lock().unwrap().get_ref().clone();
        assert!(zstd::stream::decode_all(flushed.as_slice()).is_err());
        // finish() writes the end-of-frame block and checksum, making the
        // output a complete zstd frame
        Python::attach(|py| transcoder.finish(py)).unwrap();
        let output = output_buf.lock().unwrap().get_ref().clone();
        let decompressed = zstd::stream::decode_all(output.as_slice()).unwrap();
        assert!(!decompressed.is_empty());
        // finish() is idempotent
        Python::attach(|py| transcoder.finish(py)).unwrap();
        // writing to or flushing a finished transcoder raises
        assert!(Python::attach(|py| transcoder.write(py, &[0u8; 4])).is_err());
        assert!(Python::attach(|py| transcoder.flush(py)).is_err());
    }

    #[rstest]
    fn test_partial_metadata_and_records(_python: ()) {
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut target = Python::attach(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                EncodingArg::Encoding(Encoding::Json),
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
        Python::attach(|py| target.write(py, &encoder.get_ref()[..metadata_split]))
            // no error
            .unwrap();
        Python::attach(|py| target.write(py, &encoder.get_ref()[metadata_split..])).unwrap();
        // Metadata doesn't get transcoded for JSON
        assert!(output_buf.lock().unwrap().get_ref().is_empty());
        let metadata_pos = encoder.get_ref().len();
        let rec = ErrorMsg::new(1680708278000000000, None, "This is a test", true);
        encoder.encode_record(&rec).unwrap();
        Python::attach(|py| {
            assert!(target.buffer(py).is_empty().unwrap());
        });
        let record_pos = encoder.get_ref().len();
        // write record byte by byte
        for i in metadata_pos..record_pos {
            Python::attach(|py| target.write(py, &encoder.get_ref()[i..i + 1])).unwrap();
            // wrote last byte
            if i == record_pos - 1 {
                break;
            }
            Python::attach(|py| {
                assert_eq!(target.buffer(py).len().unwrap(), i + 1 - metadata_pos);
            });
        }
        // writing the remainder of the record should have the transcoder
        // transcode the record to the output file
        Python::attach(|py| {
            assert!(target.buffer(py).is_empty().unwrap());
        });
        assert_eq!(record_pos - metadata_pos, std::mem::size_of_val(&rec));
        Python::attach(|py| target.flush(py)).unwrap();
        let output = output_buf.lock().unwrap();
        let output = std::str::from_utf8(output.get_ref().as_slice()).unwrap();
        assert_eq!(output.chars().filter(|c| *c == '\n').count(), 1);
    }

    #[rstest]
    fn test_full_with_partial_record(_python: ()) {
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut transcoder = Python::attach(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                EncodingArg::Encoding(Encoding::Csv),
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
        Python::attach(|py| transcoder.write(py, encoder.get_ref().as_slice())).unwrap();
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
        Python::attach(|py| {
            assert!(transcoder.buffer(py).is_empty().unwrap());
        });
        // Write first record and part of second
        Python::attach(|py| transcoder.write(py, &encoder.get_ref()[metadata_pos..rec1_pos + 4]))
            .unwrap();
        // Write rest of second record
        Python::attach(|py| transcoder.write(py, &encoder.get_ref()[rec1_pos + 4..])).unwrap();
        Python::attach(|py| transcoder.flush(py)).unwrap();
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
    fn test_map_symbols_historical(
        _python: (),
        #[case] encoding: Encoding,
        #[case] map_symbols: bool,
    ) {
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut transcoder = Python::attach(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                EncodingArg::Encoding(encoding),
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
        Python::attach(|py| {
            assert!(transcoder.buffer(py).is_empty().unwrap());
        });
        // Write first record and part of second
        Python::attach(|py| transcoder.write(py, encoder.get_ref())).unwrap();
        Python::attach(|py| transcoder.flush(py)).unwrap();
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
    fn test_map_symbols_live(_python: (), #[case] encoding: Encoding, #[case] map_symbols: bool) {
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut transcoder = Python::attach(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                EncodingArg::Encoding(encoding),
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
        Python::attach(|py| {
            assert!(transcoder.buffer(py).is_empty().unwrap());
        });
        // Write first record and part of second
        Python::attach(|py| transcoder.write(py, encoder.get_ref())).unwrap();
        Python::attach(|py| transcoder.flush(py)).unwrap();
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
    fn test_from_test_data_file(_python: (), #[case] encoding: Encoding, #[case] schema: Schema) {
        let input = zstd::stream::decode_all(
            std::fs::File::open(format!("{TEST_DATA_PATH}/test_data.{schema}.v3.dbn.zst")).unwrap(),
        )
        .unwrap();
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut transcoder = Python::attach(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                EncodingArg::Encoding(encoding),
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
        Python::attach(|py| transcoder.write(py, &input)).unwrap();
        Python::attach(|py| transcoder.flush(py)).unwrap();
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
    fn test_skip_metadata_csv_header_still_written(_python: ()) {
        let mut input = Vec::new();
        let mut input_file =
            std::fs::File::open(format!("{TEST_DATA_PATH}/test_data.definition.v3.dbn.frag"))
                .unwrap();
        input_file.read_to_end(&mut input).unwrap();
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut transcoder = Python::attach(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                EncodingArg::Encoding(Encoding::Csv),
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
        Python::attach(|py| transcoder.write(py, &input)).unwrap();
        Python::attach(|py| transcoder.flush(py)).unwrap();
        let output = output_buf.lock().unwrap();
        let output = std::str::from_utf8(output.get_ref().as_slice()).unwrap();
        let lines = output.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].ends_with(",symbol"));
        // ends an empty field because there was no metadata and no symbol map was provided
        assert!(lines[1].ends_with(','));
        assert!(lines[2].ends_with(','));
    }

    #[rstest]
    fn test_columnar_historical(
        _python: (),
        #[values("arrow", "parquet")] name: &str,
        #[values(false, true)] ts_out: bool,
        #[values(Compression::None, Compression::Zstd)] compression: Compression,
    ) {
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut transcoder = Python::attach(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                EncodingArg::Columnar(name.to_owned()),
                compression,
                true,
                true,
                None,
                true,
                false,
                None,
                None,
                None,
                VersionUpgradePolicy::default(),
            )
            .unwrap()
        });
        Python::attach(|py| transcoder.write(py, &ohlcv_dbn(ts_out))).unwrap();
        Python::attach(|py| transcoder.finish(py)).unwrap();
        let output = output_buf.lock().unwrap().get_ref().clone();
        let (metadata, batches) = read_file(name, output).unwrap();
        assert_eq!(batches.len(), 1);
        let batch = &batches[0];
        assert_eq!(batch.num_rows(), 2);
        let open = batch
            .column_by_name("open")
            .unwrap()
            .as_primitive::<Float64Type>();
        assert_eq!(open.values(), &[376.93, 357.39]);
        let symbol = batch.column_by_name("symbol").unwrap().as_string::<i32>();
        assert_eq!(symbol.value(0), "NFLX");
        assert_eq!(symbol.value(1), "QQQ");
        if ts_out {
            let ts_out = batch
                .column_by_name("ts_out")
                .unwrap()
                .as_primitive::<TimestampNanosecondType>();
            assert_eq!(ts_out.values(), &[1, 2]);
        } else {
            assert!(batch.column_by_name("ts_out").is_none());
        }
        assert!(metadata["databento:metadata"].contains("\"dataset\":\"XNAS.ITCH\""));
    }

    #[rstest]
    #[case::arrow("arrow", b"ARROW1".as_slice())]
    #[case::parquet("parquet", b"PAR1".as_slice())]
    fn test_columnar_footer_written_on_finish(
        _python: (),
        #[case] name: &str,
        #[case] magic: &[u8],
    ) {
        let file = MockPyFile::new();
        let output_buf = file.inner();
        let mut transcoder = Python::attach(|py| {
            Transcoder::new(
                Py::new(py, file).unwrap().extract(py).unwrap(),
                EncodingArg::Columnar(name.to_owned()),
                Compression::None,
                true,
                true,
                None,
                true,
                false,
                None,
                None,
                None,
                VersionUpgradePolicy::default(),
            )
            .unwrap()
        });
        Python::attach(|py| transcoder.write(py, &ohlcv_dbn(false))).unwrap();
        Python::attach(|py| transcoder.flush(py)).unwrap();
        let flushed = output_buf.lock().unwrap().get_ref().clone();
        assert!(read_file(name, flushed).is_none());
        Python::attach(|py| transcoder.finish(py)).unwrap();
        let finished = output_buf.lock().unwrap().get_ref().clone();
        assert!(finished.ends_with(magic));
        assert!(read_file(name, finished.clone()).is_some());
        Python::attach(|py| transcoder.finish(py)).unwrap();
        assert_eq!(output_buf.lock().unwrap().get_ref().len(), finished.len());
        assert!(Python::attach(|py| transcoder.write(py, &[0u8; 4])).is_err());
        assert!(Python::attach(|py| transcoder.flush(py)).is_err());
    }

    #[rstest]
    fn test_columnar_requires_metadata(_python: (), #[values("arrow", "parquet")] name: &str) {
        Python::attach(|py| {
            let err = Transcoder::new(
                Py::new(py, MockPyFile::new()).unwrap().extract(py).unwrap(),
                EncodingArg::Columnar(name.to_owned()),
                Compression::None,
                true,
                true,
                None,
                false,
                false,
                None,
                Some(Schema::Ohlcv1S),
                Some(DBN_VERSION),
                VersionUpgradePolicy::default(),
            )
            .err()
            .unwrap();
            assert!(err.is_instance_of::<PyValueError>(py));
            assert!(err.to_string().contains("has_metadata=False"));
        });
    }

    #[rstest]
    fn test_unknown_encoding_name(_python: ()) {
        Python::attach(|py| {
            let err = Transcoder::new(
                Py::new(py, MockPyFile::new()).unwrap().extract(py).unwrap(),
                EncodingArg::Columnar("feather".to_owned()),
                Compression::None,
                true,
                true,
                None,
                true,
                false,
                None,
                None,
                None,
                VersionUpgradePolicy::default(),
            )
            .err()
            .unwrap();
            assert!(err.is_instance_of::<PyValueError>(py));
            assert!(err.to_string().contains("'feather'"));
        });
    }

    fn ohlcv_dbn(ts_out: bool) -> Vec<u8> {
        const QQQ: &str = "QQQ";
        const QQQ_ID: u32 = 48933;
        const NFLX: &str = "NFLX";
        const NFLX_ID: u32 = 58501;
        let mut encoder = DbnEncoder::new(
            Vec::new(),
            &MetadataBuilder::new()
                .dataset(Dataset::XnasItch.to_string())
                .schema(Some(Schema::Ohlcv1S))
                .stype_in(Some(SType::RawSymbol))
                .stype_out(SType::InstrumentId)
                .ts_out(ts_out)
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
        let recs = [
            OhlcvMsg {
                hd: RecordHeader::new::<OhlcvMsg>(
                    rtype::OHLCV_1S,
                    1,
                    NFLX_ID,
                    datetime!(2023-09-27 13:30:01 UTC).unix_timestamp_nanos() as u64,
                ),
                open: 376_930_000_000,
                high: 377_050_000_000,
                low: 376_900_000_000,
                close: 377_010_000_000,
                volume: 1_204,
            },
            OhlcvMsg {
                hd: RecordHeader::new::<OhlcvMsg>(
                    rtype::OHLCV_1S,
                    1,
                    QQQ_ID,
                    datetime!(2023-09-27 13:30:01 UTC).unix_timestamp_nanos() as u64,
                ),
                open: 357_390_000_000,
                high: 357_460_000_000,
                low: 357_350_000_000,
                close: 357_420_000_000,
                volume: 18_530,
            },
        ];
        for (ts_out_val, rec) in (1..).zip(recs) {
            if ts_out {
                encoder
                    .encode_record(&WithTsOut::new(rec, ts_out_val))
                    .unwrap();
            } else {
                encoder.encode_record(&rec).unwrap();
            }
        }
        encoder.get_ref().clone()
    }

    // `None` if the file is incomplete
    fn read_file(
        name: &str,
        output: Vec<u8>,
    ) -> Option<(HashMap<String, String>, Vec<RecordBatch>)> {
        if name == "parquet" {
            let builder =
                ParquetRecordBatchReaderBuilder::try_new(bytes::Bytes::from(output)).ok()?;
            // The builder's schema carries the key-value metadata; the reader's doesn't
            let metadata = builder.schema().metadata().clone().into();
            let batches = builder.build().unwrap().map(Result::unwrap).collect();
            Some((metadata, batches))
        } else {
            let reader = FileReader::try_new(std::io::Cursor::new(output), None).ok()?;
            let metadata = reader.schema().metadata().clone().into();
            Some((metadata, reader.map(Result::unwrap).collect()))
        }
    }
}
