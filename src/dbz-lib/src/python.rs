#![allow(clippy::borrow_deref_ref)] // in generated code from `pyfunction` macro and `&PyBytes`
use std::{fmt, io, io::SeekFrom};

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDate, PyDateAccess, PyDict, PyString};
use time::Date;

use databento_defs::enums::{Compression, SType, Schema};

use crate::{MappingInterval, Metadata, SymbolMapping};

/// A Python module wrapping dbz-lib functions
#[pymodule] // The name of the function must match `lib.name` in `Cargo.toml`
fn dbz_lib(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // all functions exposed to Python need to be added here
    m.add_function(wrap_pyfunction!(decode_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(encode_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(update_encoded_metadata, m)?)?;
    Ok(())
}

/// Decodes the given Python `bytes` to `Metadata`. Returns a Python `dict` with
/// all the DBZ metadata.
///
/// # Errors
/// This function returns an error if the metadata cannot be parsed from `bytes`.
#[pyfunction]
fn decode_metadata(bytes: &PyBytes) -> PyResult<Metadata> {
    let mut reader = io::BufReader::new(bytes.as_bytes());
    Metadata::read(&mut reader).map_err(to_val_err)
}

/// Encodes the given metadata into the DBZ metadata binary format.
/// Returns Python `bytes`.
///
/// # Errors
/// This function returns an error if any of the enum arguments cannot be converted to
/// their Rust equivalents. It will also return an error if there's an issue writing
/// the encoded metadata to bytes.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn encode_metadata(
    py: Python<'_>,
    dataset: String,
    schema: u16,
    start: u64,
    end: u64,
    limit: Option<u64>,
    record_count: u64,
    compression: u8,
    stype_in: u8,
    stype_out: u8,
    symbols: Vec<String>,
    partial: Vec<String>,
    not_found: Vec<String>,
    mappings: Vec<SymbolMapping>,
) -> PyResult<Py<PyBytes>> {
    let metadata = Metadata {
        version: 1,
        dataset,
        schema: Schema::try_from(schema).map_err(to_val_err)?,
        start,
        end,
        limit: limit.unwrap_or(0),
        record_count,
        compression: Compression::try_from(compression).map_err(to_val_err)?,
        stype_in: SType::try_from(stype_in).map_err(to_val_err)?,
        stype_out: SType::try_from(stype_out).map_err(to_val_err)?,
        symbols,
        partial,
        not_found,
        mappings,
    };
    let mut encoded = Vec::with_capacity(1024);
    let cursor = io::Cursor::new(&mut encoded);
    metadata.encode(cursor).map_err(to_val_err)?;
    Ok(PyBytes::new(py, encoded.as_slice()).into())
}

/// Updates existing fields that have already been written to the given file.
#[pyfunction]
fn update_encoded_metadata(
    _py: Python<'_>,
    file: PyFileLike,
    start: u64,
    end: u64,
    limit: Option<u64>,
    record_count: u64,
) -> PyResult<()> {
    Metadata::update_encoded(file, start, end, limit.unwrap_or(0), record_count).map_err(to_val_err)
}

struct PyFileLike {
    inner: PyObject,
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
        dict.set_item("limit", self.limit).expect("set limit");
        dict.set_item("record_count", self.record_count)
            .expect("set record_count");
        dict.set_item("compression", self.compression as u8)
            .expect("set compression");
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
        dict.set_item("native", &self.native).expect("set native");
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
            .and_then(|d| d.downcast::<PyString>().map_err(PyErr::from))?
            .to_str()?
            .to_owned();
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
                SeekFrom::Current(i) => (1, i as i64),
                SeekFrom::End(i) => (2, i as i64),
            };

            let new_position = self
                .inner
                .call_method(py, "seek", (offset, whence), None)
                .map_err(py_to_rs_io_err)?;

            new_position.extract(py).map_err(py_to_rs_io_err)
        })
    }
}
