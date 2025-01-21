use std::{collections::HashMap, io, num::NonZeroU64};

use pyo3::{
    intern,
    prelude::*,
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

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    /// Encodes Metadata back into DBN format.
    fn __bytes__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        self.py_encode(py)
    }

    #[getter]
    fn get_mappings<'py>(&self, py: Python<'py>) -> PyResult<HashMap<String, Bound<'py, PyAny>>> {
        let mut res = HashMap::new();
        for mapping in self.mappings.iter() {
            res.insert(
                mapping.raw_symbol.clone(),
                mapping.intervals.into_pyobject(py)?,
            );
        }
        Ok(res)
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
    fn py_encode<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let mut buffer = Vec::new();
        let mut encoder = MetadataEncoder::new(&mut buffer);
        encoder.encode(self)?;
        Ok(PyBytes::new(py, buffer.as_slice()))
    }
}

impl<'py> IntoPyObject<'py> for SymbolMapping {
    type Target = PyDict;
    type Output = Bound<'py, PyDict>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        dict.set_item(intern!(py, "raw_symbol"), &self.raw_symbol)?;
        dict.set_item(intern!(py, "intervals"), &self.intervals)?;
        Ok(dict)
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

impl<'py> IntoPyObject<'py> for &MappingInterval {
    type Target = PyDict;
    type Output = Bound<'py, PyDict>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        dict.set_item(
            intern!(py, "start_date"),
            PyDate::new(
                py,
                self.start_date.year(),
                self.start_date.month() as u8,
                self.start_date.day(),
            )?,
        )?;
        dict.set_item(
            intern!(py, "end_date"),
            PyDate::new(
                py,
                self.end_date.year(),
                self.end_date.month() as u8,
                self.end_date.day(),
            )?,
        )?;
        dict.set_item(intern!(py, "symbol"), &self.symbol)?;
        Ok(dict)
    }
}

fn extract_date(any: Bound<'_, PyAny>) -> PyResult<time::Date> {
    let py_date = any.downcast::<PyDate>().map_err(PyErr::from)?;
    py_to_time_date(py_date)
}
