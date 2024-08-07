use std::{collections::HashMap, io, num::NonZeroU64};

use pyo3::{
    intern,
    prelude::*,
    pyclass::CompareOp,
    types::{PyBytes, PyDate, PyDict, PyType},
    Bound,
};

use crate::{
    decode::{DbnMetadata, DynDecoder},
    encode::dbn::MetadataEncoder,
    enums::{SType, Schema},
    MappingInterval, Metadata, SymbolMapping, VersionUpgradePolicy,
};

use super::{py_to_time_date, to_py_err};

#[pymethods]
impl Metadata {
    #[new]
    #[pyo3(signature = (
        dataset,
        start,
        stype_in,
        stype_out,
        schema,
        symbols=None,
        partial=None,
        not_found=None,
        mappings=None,
        end=None,
        limit=None,
        ts_out=None,
        version=crate::DBN_VERSION,
    ))]
    fn py_new(
        dataset: String,
        start: u64,
        stype_in: Option<SType>,
        stype_out: SType,
        schema: Option<Schema>,
        symbols: Option<Vec<String>>,
        partial: Option<Vec<String>>,
        not_found: Option<Vec<String>>,
        mappings: Option<Vec<SymbolMapping>>,
        end: Option<u64>,
        limit: Option<u64>,
        ts_out: Option<bool>,
        version: u8,
    ) -> Metadata {
        Metadata::builder()
            .dataset(dataset)
            .start(start)
            .stype_out(stype_out)
            .symbols(symbols.unwrap_or_default())
            .partial(partial.unwrap_or_default())
            .not_found(not_found.unwrap_or_default())
            .mappings(mappings.unwrap_or_default())
            .schema(schema)
            .stype_in(stype_in)
            .end(NonZeroU64::new(end.unwrap_or_default()))
            .limit(NonZeroU64::new(limit.unwrap_or_default()))
            .ts_out(ts_out.unwrap_or_default())
            .version(version)
            .build()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        match op {
            CompareOp::Eq => self.eq(other).into_py(py),
            CompareOp::Ne => self.ne(other).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    /// Encodes Metadata back into DBN format.
    fn __bytes__(&self, py: Python<'_>) -> PyResult<Py<PyBytes>> {
        self.py_encode(py)
    }

    #[getter]
    fn get_mappings(&self) -> HashMap<String, Vec<MappingInterval>> {
        let mut res = HashMap::new();
        for mapping in self.mappings.iter() {
            res.insert(mapping.raw_symbol.clone(), mapping.intervals.clone());
        }
        res
    }

    #[pyo3(name = "decode", signature = (data, upgrade_policy = VersionUpgradePolicy::default()))]
    #[classmethod]
    fn py_decode(
        _cls: &Bound<PyType>,
        data: &Bound<PyBytes>,
        upgrade_policy: VersionUpgradePolicy,
    ) -> PyResult<Metadata> {
        let reader = io::BufReader::new(data.as_bytes());
        let mut metadata = DynDecoder::inferred_with_buffer(reader, upgrade_policy)?
            .metadata()
            .clone();
        metadata.upgrade(upgrade_policy);
        Ok(metadata)
    }

    #[pyo3(name = "encode")]
    fn py_encode(&self, py: Python<'_>) -> PyResult<Py<PyBytes>> {
        let mut buffer = Vec::new();
        let mut encoder = MetadataEncoder::new(&mut buffer);
        encoder.encode(self)?;
        Ok(PyBytes::new_bound(py, buffer.as_slice()).into())
    }
}

impl IntoPy<PyObject> for SymbolMapping {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

// `ToPyObject` is about copying and is required for `PyDict::set_item`
impl ToPyObject for SymbolMapping {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let dict = PyDict::new_bound(py);
        dict.set_item(intern!(py, "raw_symbol"), &self.raw_symbol)
            .unwrap();
        dict.set_item(intern!(py, "intervals"), &self.intervals)
            .unwrap();
        dict.into_py(py)
    }
}

impl<'py> FromPyObject<'py> for MappingInterval {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let start_date = ob
            .getattr(intern!(ob.py(), "start_date"))
            .map_err(|_| to_py_err("Missing start_date".to_owned()))
            .and_then(extract_date)?;
        let end_date = ob
            .getattr(intern!(ob.py(), "end_date"))
            .map_err(|_| to_py_err("Missing end_date".to_owned()))
            .and_then(extract_date)?;
        let symbol = ob
            .getattr(intern!(ob.py(), "symbol"))
            .map_err(|_| to_py_err("Missing symbol".to_owned()))
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
        let dict = PyDict::new_bound(py);
        dict.set_item(
            intern!(py, "start_date"),
            PyDate::new_bound(
                py,
                self.start_date.year(),
                self.start_date.month() as u8,
                self.start_date.day(),
            )
            .unwrap(),
        )
        .unwrap();
        dict.set_item(
            intern!(py, "end_date"),
            PyDate::new_bound(
                py,
                self.end_date.year(),
                self.end_date.month() as u8,
                self.end_date.day(),
            )
            .unwrap(),
        )
        .unwrap();
        dict.set_item(intern!(py, "symbol"), &self.symbol).unwrap();
        dict.into_py(py)
    }
}

impl IntoPy<PyObject> for MappingInterval {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

fn extract_date(any: Bound<'_, PyAny>) -> PyResult<time::Date> {
    let py_date = any.downcast::<PyDate>().map_err(PyErr::from)?;
    py_to_time_date(py_date)
}
