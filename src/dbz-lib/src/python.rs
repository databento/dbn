use std::{io, str::FromStr};

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};

use db_def::enums::{Compression, Dataset, Encoding, SType, Schema};

use crate::Metadata;

/// A Python module wrapping dbz-lib functions
#[pymodule] // The name of the function must match `lib.name` in `Cargo.toml`
fn dbz_lib(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // all functions exposed to Python need to be added here
    m.add_function(wrap_pyfunction!(decode_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(encode_metadata, m)?)?;
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
    dataset: &str,
    schema: u8,
    stype_in: u8,
    stype_out: u8,
    start: u64,
    end: u64,
    limit: Option<u64>,
    encoding: u8,
    compression: u8,
    nrows: u64,
    ncols: u16,
) -> PyResult<Py<PyBytes>> {
    let metadata = Metadata {
        version: 1,
        dataset: Dataset::from_str(dataset).map_err(to_val_err)?,
        schema: Schema::try_from(schema).map_err(to_val_err)?,
        stype_in: SType::try_from(stype_in).map_err(to_val_err)?,
        stype_out: SType::try_from(stype_out).map_err(to_val_err)?,
        start,
        end,
        limit: limit.unwrap_or(0),
        encoding: Encoding::try_from(encoding).map_err(to_val_err)?,
        compression: Compression::try_from(compression).map_err(to_val_err)?,
        nrows,
        ncols,
        // FIXME: variable JSON
        extra: serde_json::Map::default(),
    };
    let mut encoded = Vec::with_capacity(1024);
    let cursor = io::Cursor::new(&mut encoded);
    metadata.encode(cursor).map_err(to_val_err)?;
    Ok(PyBytes::new(py, encoded.as_slice()).into())
}

// [Metadata] gets converted into a plain Python `dict` when returned back to Python
impl IntoPy<PyObject> for Metadata {
    fn into_py(self, py: Python<'_>) -> PyObject {
        let dict = PyDict::new(py);
        dict.set_item("version", self.version).expect("set version");
        dict.set_item("dataset", self.dataset.as_str())
            .expect("set dataset");
        dict.set_item("schema", self.schema as u8)
            .expect("set schema");
        dict.set_item("stype_in", self.stype_in as u8)
            .expect("set stype_in");
        dict.set_item("stype_out", self.stype_out as u8)
            .expect("set stype_out");
        dict.set_item("start", self.start).expect("set start");
        dict.set_item("end", self.end).expect("set end");
        dict.set_item("limit", self.limit).expect("set limit");
        dict.set_item("encoding", self.encoding as u8)
            .expect("set encoding");
        dict.set_item("compression", self.compression as u8)
            .expect("set compression");
        dict.set_item("nrows", self.nrows).expect("set nrows");
        dict.set_item("ncols", self.ncols).expect("set ncols");
        for (k, v) in self.extra {
            // TODO: figure out better conversion:
            // 1. wait for more structured translation
            // 2. pull in pythonize crate as dependency
            dict.set_item(k, v.to_string()).expect("set extra");
        }
        dict.into_py(py)
    }
}

fn to_val_err(e: impl ToString) -> PyErr {
    PyValueError::new_err(e.to_string())
}
