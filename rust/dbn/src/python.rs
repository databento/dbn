//! Python wrappers around dbn functions. These are implemented here instead of in `python/`
//! to be able to implement [`pyo3`] traits for [`dbn`] types.
#![allow(clippy::borrow_deref_ref)]
// in generated code from `pyfunction` macro and `&PyBytes`
use std::{ffi::c_char, fmt, io, io::SeekFrom, mem, num::NonZeroU64};

use pyo3::{
    exceptions::{PyKeyError, PyTypeError, PyValueError},
    prelude::*,
    types::{PyBytes, PyDate, PyDateAccess, PyDict},
};
use time::Date;

use crate::{
    decode::dbn::MetadataDecoder,
    encode::{
        dbn::{self, MetadataEncoder},
        DbnEncodable, DynWriter, EncodeDbn,
    },
    enums::{Compression, SType, Schema},
    metadata::MetadataBuilder,
    record::{
        BidAskPair, HasRType, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, RecordHeader, TbboMsg, TradeMsg,
    },
};
use crate::{MappingInterval, Metadata, SymbolMapping};

/// Decodes the given Python `bytes` to `Metadata`. Returns a Python `dict` with
/// all the DBN metadata.
///
/// # Errors
/// This function returns an error if the metadata cannot be parsed from `bytes`.
#[pyfunction]
pub fn decode_metadata(bytes: &PyBytes) -> PyResult<Metadata> {
    let reader = io::BufReader::new(bytes.as_bytes());
    MetadataDecoder::new(reader).decode().map_err(to_val_err)
}

/// Encodes the given metadata into the DBN metadata binary format.
/// Returns Python `bytes`.
///
/// # Errors
/// This function returns an error if any of the enum arguments cannot be converted to
/// their Rust equivalents. It will also return an error if there's an issue writing
/// the encoded metadata to bytes.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
pub fn encode_metadata(
    py: Python<'_>,
    dataset: String,
    schema: Schema,
    start: u64,
    end: u64,
    limit: Option<u64>,
    record_count: u64,
    stype_in: SType,
    stype_out: SType,
    symbols: Vec<String>,
    partial: Vec<String>,
    not_found: Vec<String>,
    mappings: Vec<SymbolMapping>,
) -> PyResult<Py<PyBytes>> {
    let metadata = MetadataBuilder::new()
        .dataset(dataset)
        .schema(schema)
        .start(start)
        .end(end)
        .record_count(record_count)
        .limit(NonZeroU64::new(limit.unwrap_or(0)))
        .stype_in(stype_in)
        .stype_out(stype_out)
        .symbols(symbols)
        .partial(partial)
        .not_found(not_found)
        .mappings(mappings)
        .build();
    let mut encoded = Vec::with_capacity(1024);
    MetadataEncoder::new(encoded.as_mut_slice())
        .encode(&metadata)
        .map_err(to_val_err)?;
    Ok(PyBytes::new(py, encoded.as_slice()).into())
}

/// Updates existing fields that have already been written to the given file.
#[pyfunction]
pub fn update_encoded_metadata(
    _py: Python<'_>,
    file: PyFileLike,
    start: u64,
    end: u64,
    limit: Option<u64>,
    record_count: u64,
) -> PyResult<()> {
    MetadataEncoder::new(file)
        .update_encoded(start, end, limit.and_then(NonZeroU64::new), record_count)
        .map_err(to_val_err)
}

pub struct PyFileLike {
    inner: PyObject,
}

/// Encodes the given data in the DBN encoding and writes it to `file`. Most
/// metadata is inferred based on the arguments.
///
/// `records` is a list of **flat** dicts where the field names match the
/// record type corresponding with `schema`. For `Mbp1` and `Mbp10` schemas, the
/// `booklevel` fields should be suffixed with `_0{level}`, e.g. the first book
/// level ask price should be under the key `"ask_px_00"`.
///
/// # Errors
/// This function returns an error if any of the enum arguments cannot be converted to
/// their Rust equivalents. It will also return an error if there's an issue writing
/// the encoded to bytes or an expected field is missing from one of the dicts.
#[pyfunction]
pub fn write_dbn_file(
    _py: Python<'_>,
    file: PyFileLike,
    compression: Compression,
    schema: Schema,
    dataset: String,
    records: Vec<&PyDict>,
    stype: SType,
) -> PyResult<()> {
    let metadata = MetadataBuilder::new()
        .schema(schema)
        .dataset(dataset)
        .record_count(records.len() as u64)
        .stype_in(stype)
        .stype_out(stype)
        .start(0)
        .build();
    let writer = DynWriter::new(file, compression).map_err(to_val_err)?;
    let encoder = dbn::Encoder::new(writer, &metadata).map_err(to_val_err)?;
    match schema {
        Schema::Mbo => encode_pydicts::<MboMsg>(encoder, &records),
        Schema::Mbp1 => encode_pydicts::<Mbp1Msg>(encoder, &records),
        Schema::Mbp10 => encode_pydicts::<Mbp10Msg>(encoder, &records),
        Schema::Tbbo => encode_pydicts::<TbboMsg>(encoder, &records),
        Schema::Trades => encode_pydicts::<TradeMsg>(encoder, &records),
        Schema::Ohlcv1S => encode_pydicts::<OhlcvMsg>(encoder, &records),
        Schema::Ohlcv1M => encode_pydicts::<OhlcvMsg>(encoder, &records),
        Schema::Ohlcv1H => encode_pydicts::<OhlcvMsg>(encoder, &records),
        Schema::Ohlcv1D => encode_pydicts::<OhlcvMsg>(encoder, &records),
        Schema::Definition | Schema::Statistics | Schema::Status => Err(PyValueError::new_err(
            "Unsupported schema type for writing DBN files",
        )),
    }
}

fn encode_pydicts<T: DbnEncodable + FromPyDict>(
    mut encoder: dbn::Encoder<DynWriter<PyFileLike>>,
    records: &[&PyDict],
) -> PyResult<()> {
    encoder
        .encode_records(
            records
                .iter()
                .map(|dict| T::from_py_dict(dict))
                .collect::<PyResult<Vec<T>>>()?
                .iter()
                .as_slice(),
        )
        .map_err(to_val_err)
}

impl<'source> FromPyObject<'source> for PyFileLike {
    fn extract(any: &'source PyAny) -> PyResult<Self> {
        Python::with_gil(|py| {
            let obj: PyObject = any.extract()?;
            if obj.getattr(py, "read").is_err() {
                return Err(PyTypeError::new_err(
                    "object is missing a `read()` method".to_owned(),
                ));
            }
            if obj.getattr(py, "write").is_err() {
                return Err(PyTypeError::new_err(
                    "object is missing a `write()` method".to_owned(),
                ));
            }
            if obj.getattr(py, "seek").is_err() {
                return Err(PyTypeError::new_err(
                    "object is missing a `seek()` method".to_owned(),
                ));
            }
            Ok(PyFileLike { inner: obj })
        })
    }
}

// [Metadata] gets converted into a plain Python `dict` when returned back to Python
impl IntoPy<PyObject> for Metadata {
    fn into_py(self, py: Python<'_>) -> PyObject {
        let dict = PyDict::new(py);
        dict.set_item("version", self.version).expect("set version");
        dict.set_item("dataset", self.dataset).expect("set dataset");
        dict.set_item("schema", self.schema as u8)
            .expect("set schema");
        dict.set_item("start", self.start).expect("set start");
        dict.set_item("end", self.end).expect("set end");
        dict.set_item("limit", self.limit.map(|n| n.get()))
            .expect("set limit");
        dict.set_item("record_count", self.record_count)
            .expect("set record_count");
        dict.set_item("stype_in", self.stype_in as u8)
            .expect("set stype_in");
        dict.set_item("stype_out", self.stype_out as u8)
            .expect("set stype_out");
        dict.set_item("symbols", self.symbols).expect("set symbols");
        dict.set_item("partial", self.partial).expect("set partial");
        dict.set_item("not_found", self.not_found)
            .expect("set not_found");
        dict.set_item("mappings", self.mappings)
            .expect("set mappings");
        dict.into_py(py)
    }
}

// `ToPyObject` is about copying and is required for `PyDict::set_item`
impl ToPyObject for SymbolMapping {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let dict = PyDict::new(py);
        dict.set_item("native_symbol", &self.native_symbol)
            .expect("set native_symbol");
        dict.set_item("intervals", &self.intervals)
            .expect("set intervals");
        dict.into_py(py)
    }
}

fn extract_date(any: &PyAny) -> PyResult<time::Date> {
    let py_date = any.downcast::<PyDate>().map_err(PyErr::from)?;
    let month =
        time::Month::try_from(py_date.get_month()).map_err(|e| to_val_err(e.to_string()))?;
    Date::from_calendar_date(py_date.get_year(), month, py_date.get_day())
        .map_err(|e| to_val_err(e.to_string()))
}

impl<'source> FromPyObject<'source> for MappingInterval {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let start_date = ob
            .getattr("start_date")
            .map_err(|_| to_val_err("Missing start_date".to_owned()))
            .and_then(extract_date)?;
        let end_date = ob
            .getattr("end_date")
            .map_err(|_| to_val_err("Missing end_date".to_owned()))
            .and_then(extract_date)?;
        let symbol = ob
            .getattr("symbol")
            .map_err(|_| to_val_err("Missing symbol".to_owned()))
            .and_then(|d| d.extract::<String>())?;
        Ok(Self {
            start_date,
            end_date,
            symbol,
        })
    }
}

impl ToPyObject for MappingInterval {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let dict = PyDict::new(py);
        dict.set_item(
            "start_date",
            PyDate::new(
                py,
                self.start_date.year(),
                self.start_date.month() as u8,
                self.start_date.day(),
            )
            .expect("valid start_date"),
        )
        .expect("set start_date");
        dict.set_item(
            "end_date",
            PyDate::new(
                py,
                self.end_date.year(),
                self.end_date.month() as u8,
                self.end_date.day(),
            )
            .expect("valid end_date"),
        )
        .expect("set end_date");
        dict.set_item("symbol", &self.symbol).expect("set symbol");
        dict.into_py(py)
    }
}

fn to_val_err(e: impl fmt::Debug) -> PyErr {
    PyValueError::new_err(format!("{e:?}"))
}

fn py_to_rs_io_err(e: PyErr) -> io::Error {
    Python::with_gil(|py| {
        let e_as_object: PyObject = e.into_py(py);

        match e_as_object.call_method(py, "__str__", (), None) {
            Ok(repr) => match repr.extract::<String>(py) {
                Ok(s) => io::Error::new(io::ErrorKind::Other, s),
                Err(_e) => io::Error::new(io::ErrorKind::Other, "An unknown error has occurred"),
            },
            Err(_) => io::Error::new(io::ErrorKind::Other, "Err doesn't have __str__"),
        }
    })
}

impl io::Write for PyFileLike {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        Python::with_gil(|py| {
            let bytes = PyBytes::new(py, buf).to_object(py);
            let number_bytes_written = self
                .inner
                .call_method(py, "write", (bytes,), None)
                .map_err(py_to_rs_io_err)?;

            number_bytes_written.extract(py).map_err(py_to_rs_io_err)
        })
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        Python::with_gil(|py| {
            self.inner
                .call_method(py, "flush", (), None)
                .map_err(py_to_rs_io_err)?;

            Ok(())
        })
    }
}

impl io::Seek for PyFileLike {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, io::Error> {
        Python::with_gil(|py| {
            let (whence, offset) = match pos {
                SeekFrom::Start(i) => (0, i as i64),
                SeekFrom::Current(i) => (1, i),
                SeekFrom::End(i) => (2, i),
            };

            let new_position = self
                .inner
                .call_method(py, "seek", (offset, whence), None)
                .map_err(py_to_rs_io_err)?;

            new_position.extract(py).map_err(py_to_rs_io_err)
        })
    }
}

trait FromPyDict: Sized {
    fn from_py_dict(dict: &PyDict) -> PyResult<Self>;
}

fn try_get_item<'a>(dict: &'a PyDict, key: &str) -> PyResult<&'a PyAny> {
    dict.get_item(key)
        .ok_or_else(|| PyKeyError::new_err(format!("Missing {key}")))
}

fn try_extract_item<'a, D>(dict: &'a PyDict, key: &str) -> PyResult<D>
where
    D: FromPyObject<'a>,
{
    try_get_item(dict, key)?.extract::<D>()
}

fn header_from_dict<T: HasRType>(dict: &PyDict) -> PyResult<RecordHeader> {
    let rtype = try_extract_item::<u8>(dict, "rtype")?;
    if T::has_rtype(rtype) {
        Ok(RecordHeader {
            length: (mem::size_of::<T>() / 4) as u8,
            rtype,
            publisher_id: try_extract_item::<u16>(dict, "publisher_id")?,
            product_id: try_extract_item::<u32>(dict, "product_id")?,
            ts_event: try_extract_item::<u64>(dict, "ts_event")?,
        })
    } else {
        Err(PyValueError::new_err(format!(
            "Incorrect rtype {rtype:?} for message"
        )))
    }
}

impl FromPyDict for MboMsg {
    fn from_py_dict(dict: &PyDict) -> PyResult<Self> {
        Ok(Self {
            hd: header_from_dict::<Self>(dict)?,
            order_id: try_extract_item::<u64>(dict, "order_id")?,
            price: try_extract_item::<i64>(dict, "price")?,
            size: try_extract_item::<u32>(dict, "size")?,
            flags: try_extract_item::<u8>(dict, "flags")?,
            channel_id: try_extract_item::<u8>(dict, "channel_id")?,
            action: try_extract_item::<c_char>(dict, "action")?,
            side: try_extract_item::<c_char>(dict, "side")?,
            ts_recv: try_extract_item::<u64>(dict, "ts_recv")?,
            ts_in_delta: try_extract_item::<i32>(dict, "ts_in_delta")?,
            sequence: try_extract_item::<u32>(dict, "sequence")?,
        })
    }
}

fn ba_pair_from_dict<const LEVEL: u8>(dict: &PyDict) -> PyResult<BidAskPair> {
    Ok(BidAskPair {
        bid_px: try_extract_item::<i64>(dict, &format!("bid_px_0{LEVEL}"))?,
        ask_px: try_extract_item::<i64>(dict, &format!("ask_px_0{LEVEL}"))?,
        bid_sz: try_extract_item::<u32>(dict, &format!("bid_sz_0{LEVEL}"))?,
        ask_sz: try_extract_item::<u32>(dict, &format!("ask_sz_0{LEVEL}"))?,
        bid_ct: try_extract_item::<u32>(dict, &format!("bid_ct_0{LEVEL}"))?,
        ask_ct: try_extract_item::<u32>(dict, &format!("ask_ct_0{LEVEL}"))?,
    })
}

impl FromPyDict for TradeMsg {
    fn from_py_dict(dict: &PyDict) -> PyResult<Self> {
        Ok(Self {
            hd: header_from_dict::<Self>(dict)?,
            price: try_extract_item::<i64>(dict, "price")?,
            size: try_extract_item::<u32>(dict, "size")?,
            action: try_extract_item::<c_char>(dict, "action")?,
            side: try_extract_item::<c_char>(dict, "side")?,
            flags: try_extract_item::<u8>(dict, "flags")?,
            depth: try_extract_item::<u8>(dict, "depth")?,
            ts_recv: try_extract_item::<u64>(dict, "ts_recv")?,
            ts_in_delta: try_extract_item::<i32>(dict, "ts_in_delta")?,
            sequence: try_extract_item::<u32>(dict, "sequence")?,
            booklevel: [],
        })
    }
}

impl FromPyDict for Mbp1Msg {
    fn from_py_dict(dict: &PyDict) -> PyResult<Self> {
        Ok(Self {
            hd: header_from_dict::<Self>(dict)?,
            price: try_extract_item::<i64>(dict, "price")?,
            size: try_extract_item::<u32>(dict, "size")?,
            action: try_extract_item::<c_char>(dict, "action")?,
            side: try_extract_item::<c_char>(dict, "side")?,
            flags: try_extract_item::<u8>(dict, "flags")?,
            depth: try_extract_item::<u8>(dict, "depth")?,
            ts_recv: try_extract_item::<u64>(dict, "ts_recv")?,
            ts_in_delta: try_extract_item::<i32>(dict, "ts_in_delta")?,
            sequence: try_extract_item::<u32>(dict, "sequence")?,
            booklevel: [ba_pair_from_dict::<0>(dict)?],
        })
    }
}

impl FromPyDict for Mbp10Msg {
    fn from_py_dict(dict: &PyDict) -> PyResult<Self> {
        Ok(Self {
            hd: header_from_dict::<Self>(dict)?,
            price: try_extract_item::<i64>(dict, "price")?,
            size: try_extract_item::<u32>(dict, "size")?,
            action: try_extract_item::<c_char>(dict, "action")?,
            side: try_extract_item::<c_char>(dict, "side")?,
            flags: try_extract_item::<u8>(dict, "flags")?,
            depth: try_extract_item::<u8>(dict, "depth")?,
            ts_recv: try_extract_item::<u64>(dict, "ts_recv")?,
            ts_in_delta: try_extract_item::<i32>(dict, "ts_in_delta")?,
            sequence: try_extract_item::<u32>(dict, "sequence")?,
            booklevel: [
                ba_pair_from_dict::<0>(dict)?,
                ba_pair_from_dict::<1>(dict)?,
                ba_pair_from_dict::<2>(dict)?,
                ba_pair_from_dict::<3>(dict)?,
                ba_pair_from_dict::<4>(dict)?,
                ba_pair_from_dict::<5>(dict)?,
                ba_pair_from_dict::<6>(dict)?,
                ba_pair_from_dict::<7>(dict)?,
                ba_pair_from_dict::<8>(dict)?,
                ba_pair_from_dict::<9>(dict)?,
            ],
        })
    }
}

impl FromPyDict for OhlcvMsg {
    fn from_py_dict(dict: &PyDict) -> PyResult<Self> {
        Ok(Self {
            hd: header_from_dict::<Self>(dict)?,
            open: try_extract_item::<i64>(dict, "open")?,
            high: try_extract_item::<i64>(dict, "high")?,
            low: try_extract_item::<i64>(dict, "low")?,
            close: try_extract_item::<i64>(dict, "close")?,
            volume: try_extract_item::<u64>(dict, "volume")?,
        })
    }
}

impl<'source> FromPyObject<'source> for Compression {
    fn extract(any: &'source PyAny) -> PyResult<Self> {
        let str: &str = any.extract()?;
        str.parse().map_err(to_val_err)
    }
}

impl<'source> FromPyObject<'source> for Schema {
    fn extract(any: &'source PyAny) -> PyResult<Self> {
        let str: &str = any.extract()?;
        str.parse().map_err(to_val_err)
    }
}

impl<'source> FromPyObject<'source> for SType {
    fn extract(any: &'source PyAny) -> PyResult<Self> {
        let str: &str = any.extract()?;
        str.parse().map_err(to_val_err)
    }
}

#[cfg(all(test, feature = "python-test"))]
mod tests {
    use std::io::{Cursor, Seek, Write};
    use std::sync::{Arc, Mutex};

    use streaming_iterator::StreamingIterator;

    use super::*;
    use crate::{
        decode::{dbn, DecodeDbn},
        encode::json,
    };

    const DBN_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/data");

    type JsonObj = serde_json::Map<String, serde_json::Value>;

    #[pyclass]
    struct MockPyFile {
        buf: Arc<Mutex<Cursor<Vec<u8>>>>,
    }

    #[pymethods]
    impl MockPyFile {
        fn read(&self) {
            unimplemented!();
        }

        fn write(&mut self, bytes: &[u8]) -> usize {
            self.buf.lock().unwrap().write_all(bytes).unwrap();
            bytes.len()
        }

        fn flush(&mut self) {
            self.buf.lock().unwrap().flush().unwrap();
        }

        fn seek(&self, offset: i64, whence: i32) -> u64 {
            self.buf
                .lock()
                .unwrap()
                .seek(match whence {
                    0 => SeekFrom::Start(offset as u64),
                    1 => SeekFrom::Current(offset),
                    2 => SeekFrom::End(offset),
                    _ => unimplemented!("whence value"),
                })
                .unwrap()
        }
    }

    impl MockPyFile {
        fn new() -> Self {
            Self {
                buf: Arc::new(Mutex::new(Cursor::new(Vec::new()))),
            }
        }

        fn inner(&self) -> Arc<Mutex<Cursor<Vec<u8>>>> {
            self.buf.clone()
        }
    }

    fn add_to_dict(py: Python<'_>, dict: &PyDict, key: &str, value: &serde_json::Value) {
        match value {
            serde_json::Value::Null => {
                dict.set_item(key, ()).unwrap();
            }
            serde_json::Value::Bool(v) => {
                dict.set_item(key, v).unwrap();
            }
            serde_json::Value::Number(n) => {
                if n.is_u64() {
                    dict.set_item(key, n.as_u64())
                } else if n.is_i64() {
                    dict.set_item(key, n.as_i64())
                } else {
                    dict.set_item(key, n.as_f64())
                }
                .unwrap();
            }
            serde_json::Value::String(s) if key.starts_with("ts_") => {
                dict.set_item(key, s.parse::<u64>().unwrap()).unwrap();
            }
            serde_json::Value::String(s) => {
                dict.set_item(key, s).unwrap();
            }
            serde_json::Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let nested = PyDict::new(py);
                    add_to_dict(py, nested, "", val);
                    for (k, v) in nested.iter() {
                        dict.set_item(format!("{}_0{i}", k.extract::<String>().unwrap()), v)
                            .unwrap();
                    }
                }
            }
            serde_json::Value::Object(nested) => {
                // flatten
                nested.iter().for_each(|(n_k, n_v)| {
                    add_to_dict(py, dict, n_k, n_v);
                });
            }
        }
    }

    /// Converts parsed JSON to a Python dict.
    fn json_to_py_dict<'py>(py: Python<'py>, json: &JsonObj) -> &'py PyDict {
        let res = PyDict::new(py);
        json.iter().for_each(|(key, value)| {
            add_to_dict(py, res, key, value);
        });
        res
    }

    const DATASET: &str = "GLBX.MDP3";
    const STYPE: SType = SType::ProductId;

    macro_rules! test_writing_dbn_from_python {
        ($test_name:ident, $record_type:ident, $schema:expr) => {
            #[test]
            fn $test_name() {
                // Required one-time setup
                pyo3::prepare_freethreaded_python();

                // Read in test data
                let decoder = dbn::Decoder::from_zstd_file(format!(
                    "{DBN_PATH}/test_data.{}.dbn.zst",
                    $schema.as_str()
                ))
                .unwrap();
                // Serialize test data to JSON
                let mut writer = Cursor::new(Vec::new());
                let mut encoder = json::Encoder::new(&mut writer, true);
                encoder
                    .encode_stream(decoder.decode_stream::<$record_type>().unwrap())
                    .unwrap();
                // Read in JSON to generic serde JSON objects
                let input_buf = writer.into_inner();
                let json_input = String::from_utf8(input_buf).unwrap();
                let json_recs = serde_json::Deserializer::from_str(&json_input)
                    .into_iter()
                    .collect::<serde_json::Result<Vec<JsonObj>>>()
                    .unwrap();
                let output_buf = Python::with_gil(|py| -> PyResult<_> {
                    // Convert JSON objects to Python `dict`s
                    let recs: Vec<_> = json_recs
                        .iter()
                        .map(|json_rec| json_to_py_dict(py, json_rec))
                        .collect();
                    let mock_file = MockPyFile::new();
                    let output_buf = mock_file.inner();
                    let mock_file = Py::new(py, mock_file).unwrap().into_py(py);
                    // Call target function
                    write_dbn_file(
                        py,
                        mock_file.extract(py).unwrap(),
                        Compression::ZStd,
                        $schema,
                        DATASET.to_owned(),
                        recs,
                        STYPE,
                    )
                    .unwrap();

                    Ok(output_buf.clone())
                })
                .unwrap();
                let output_buf = output_buf.lock().unwrap().clone().into_inner();

                assert!(!output_buf.is_empty());

                dbg!(&output_buf);
                dbg!(output_buf.len());
                // Reread output written with `write_dbn_file` and compare to original
                // contents
                let py_decoder = dbn::Decoder::with_zstd(Cursor::new(&output_buf)).unwrap();
                let metadata = py_decoder.metadata().clone();
                assert_eq!(metadata.schema, $schema);
                assert_eq!(metadata.dataset, DATASET);
                assert_eq!(metadata.stype_in, STYPE);
                assert_eq!(metadata.stype_out, STYPE);
                assert_eq!(metadata.record_count as usize, json_recs.len());
                let decoder = dbn::Decoder::from_zstd_file(format!(
                    "{DBN_PATH}/test_data.{}.dbn.zst",
                    $schema.as_str()
                ))
                .unwrap();

                let mut py_iter = py_decoder.decode_stream::<$record_type>().unwrap();
                let mut expected_iter = decoder.decode_stream::<$record_type>().unwrap();
                let mut count = 0;
                while let Some((py_rec, exp_rec)) = py_iter
                    .next()
                    .and_then(|py_rec| expected_iter.next().map(|exp_rec| (py_rec, exp_rec)))
                {
                    assert_eq!(py_rec, exp_rec);
                    count += 1;
                }
                assert_eq!(count, metadata.record_count);
            }
        };
    }

    test_writing_dbn_from_python!(test_writing_mbo_from_python, MboMsg, Schema::Mbo);
    test_writing_dbn_from_python!(test_writing_mbp1_from_python, Mbp1Msg, Schema::Mbp1);
    test_writing_dbn_from_python!(test_writing_mbp10_from_python, Mbp10Msg, Schema::Mbp10);
    test_writing_dbn_from_python!(test_writing_ohlcv1d_from_python, OhlcvMsg, Schema::Ohlcv1D);
    test_writing_dbn_from_python!(test_writing_ohlcv1h_from_python, OhlcvMsg, Schema::Ohlcv1H);
    test_writing_dbn_from_python!(test_writing_ohlcv1m_from_python, OhlcvMsg, Schema::Ohlcv1M);
    test_writing_dbn_from_python!(test_writing_ohlcv1s_from_python, OhlcvMsg, Schema::Ohlcv1S);
    test_writing_dbn_from_python!(test_writing_tbbo_from_python, TbboMsg, Schema::Tbbo);
    test_writing_dbn_from_python!(test_writing_trades_from_python, TradeMsg, Schema::Trades);
}
