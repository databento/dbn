use std::ffi::c_char;

use pyo3::{
    conversion::IntoPyObjectExt,
    intern,
    prelude::*,
    types::{PyDateTime, PyDict, PyTzInfo},
};

use crate::{
    python::PyFieldDesc, BidAskPair, ConsolidatedBidAskPair, HasRType, WithTsOut, UNDEF_TIMESTAMP,
};

pub fn char_to_c_char(c: char) -> crate::Result<c_char> {
    if c.is_ascii() {
        Ok(c as c_char)
    } else {
        Err(crate::Error::Conversion {
            input: c.to_string(),
            desired_type: "c_char",
        })
    }
}

impl<const N: usize> PyFieldDesc for [BidAskPair; N] {
    fn field_dtypes(_field_name: &str) -> Vec<(String, String)> {
        let mut res = Vec::new();
        let field_dtypes = BidAskPair::field_dtypes("");
        for level in 0..N {
            let mut dtypes = field_dtypes.clone();
            for dtype in dtypes.iter_mut() {
                dtype.0.push_str(&format!("_{level:02}"));
            }
            res.extend(dtypes);
        }
        res
    }

    fn price_fields(_field_name: &str) -> Vec<String> {
        append_level_suffix::<N>(BidAskPair::price_fields(""))
    }

    fn ordered_fields(_field_name: &str) -> Vec<String> {
        append_level_suffix::<N>(BidAskPair::ordered_fields(""))
    }
}

impl<const N: usize> PyFieldDesc for [ConsolidatedBidAskPair; N] {
    fn field_dtypes(_field_name: &str) -> Vec<(String, String)> {
        let mut res = Vec::new();
        let field_dtypes = ConsolidatedBidAskPair::field_dtypes("");
        for level in 0..N {
            let mut dtypes = field_dtypes.clone();
            for dtype in dtypes.iter_mut() {
                dtype.0.push_str(&format!("_{level:02}"));
            }
            res.extend(dtypes);
        }
        res
    }

    fn price_fields(_field_name: &str) -> Vec<String> {
        append_level_suffix::<N>(ConsolidatedBidAskPair::price_fields(""))
    }

    fn ordered_fields(_field_name: &str) -> Vec<String> {
        append_level_suffix::<N>(ConsolidatedBidAskPair::ordered_fields(""))
    }

    fn hidden_fields(_field_name: &str) -> Vec<String> {
        append_level_suffix::<N>(ConsolidatedBidAskPair::hidden_fields(""))
    }
}

pub fn append_level_suffix<const N: usize>(fields: Vec<String>) -> Vec<String> {
    let mut res = Vec::new();
    for level in 0..N {
        let mut fields = fields.clone();
        for field in fields.iter_mut() {
            field.push_str(&format!("_{level:02}"));
        }
        res.extend(fields);
    }
    res
}

/// `WithTsOut` adds a `ts_out` field to the main record when converted to Python.
impl<'py, R> IntoPyObject<'py> for WithTsOut<R>
where
    R: HasRType + IntoPyObject<'py>,
{
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let obj = self.rec.into_bound_py_any(py)?;
        obj.setattr(intern!(py, "ts_out"), self.ts_out).unwrap();
        Ok(obj)
    }
}

pub fn new_py_timestamp_or_datetime(
    py: Python<'_>,
    timestamp: u64,
) -> PyResult<Option<Bound<'_, PyAny>>> {
    if timestamp == UNDEF_TIMESTAMP {
        return Ok(None);
    }
    if let Ok(pandas) = PyModule::import(py, intern!(py, "pandas")) {
        let kwargs = PyDict::new(py);
        if kwargs.set_item(intern!(py, "utc"), true).is_ok()
            && kwargs
                .set_item(intern!(py, "errors"), intern!(py, "coerce"))
                .is_ok()
            && kwargs
                .set_item(intern!(py, "unit"), intern!(py, "ns"))
                .is_ok()
        {
            return pandas
                .call_method(intern!(py, "to_datetime"), (timestamp,), Some(&kwargs))
                .map(|o| Some(o.into_pyobject(py).unwrap()));
        }
    }
    let utc_tz = PyTzInfo::utc(py)?;
    let timestamp_ms = timestamp as f64 / 1_000_000.0;
    PyDateTime::from_timestamp(py, timestamp_ms, Some(&utc_tz))
        .map(|o| Some(o.into_pyobject(py).unwrap().into_any()))
}
