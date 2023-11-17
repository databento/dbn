use std::str::FromStr;

use pyo3::{prelude::*, pyclass::CompareOp, type_object::PyTypeInfo, types::PyType};

use crate::{
    enums::{Compression, Encoding, SType, Schema, SecurityUpdateAction, UserDefinedInstrument},
    RType,
};

use super::{to_val_err, EnumIterator, PyFieldDesc};

impl<'source> FromPyObject<'source> for SecurityUpdateAction {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        u8::extract(ob).and_then(|num| Self::try_from(num).map_err(to_val_err))
    }
}

impl IntoPy<PyObject> for SecurityUpdateAction {
    fn into_py(self, py: Python<'_>) -> PyObject {
        (self as u8).into_py(py)
    }
}

impl<'source> FromPyObject<'source> for UserDefinedInstrument {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        u8::extract(ob).and_then(|num| Self::try_from(num).map_err(to_val_err))
    }
}

impl IntoPy<PyObject> for UserDefinedInstrument {
    fn into_py(self, py: Python<'_>) -> PyObject {
        (self as u8).into_py(py)
    }
}

#[pymethods]
impl Compression {
    #[new]
    fn py_new(py: Python<'_>, value: &PyAny) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(t, value)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> &'static str {
        self.as_str()
    }

    fn __repr__(&self) -> String {
        format!("<Compression.{}: '{}'>", self.name(), self.value(),)
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        let Ok(other_enum) = Self::py_from_str(Self::type_object(py), other) else {
            return py.NotImplemented();
        };
        match op {
            CompareOp::Eq => self.eq(&other_enum).into_py(py),
            CompareOp::Ne => self.ne(&other_enum).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    #[getter]
    fn name(&self) -> String {
        self.as_str().to_uppercase()
    }

    #[getter]
    fn value(&self) -> &'static str {
        self.as_str()
    }

    // No metaclass support with pyo3, so `for c in Compression: ...` isn't possible
    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &PyType, value: &PyAny) -> PyResult<Self> {
        let value_str: &str = value.str().and_then(|s| s.extract())?;
        let tokenized = value_str.to_lowercase();
        Self::from_str(&tokenized).map_err(to_val_err)
    }
}

#[pymethods]
impl Encoding {
    #[new]
    fn py_new(py: Python<'_>, value: &PyAny) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(t, value)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> &'static str {
        self.as_str()
    }

    fn __repr__(&self) -> String {
        format!("<Encoding.{}: '{}'>", self.name(), self.value(),)
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        let Ok(other_enum) = Self::py_from_str(Self::type_object(py), other) else {
            return py.NotImplemented();
        };
        match op {
            CompareOp::Eq => self.eq(&other_enum).into_py(py),
            CompareOp::Ne => self.ne(&other_enum).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    #[getter]
    fn name(&self) -> String {
        self.as_str().to_uppercase()
    }

    #[getter]
    fn value(&self) -> &'static str {
        self.as_str()
    }

    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &PyType, value: &PyAny) -> PyResult<Self> {
        let value_str: &str = value.str().and_then(|s| s.extract())?;
        let tokenized = value_str.to_lowercase();
        Self::from_str(&tokenized).map_err(to_val_err)
    }
}

#[pymethods]
impl Schema {
    #[new]
    fn py_new(py: Python<'_>, value: &PyAny) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(t, value)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> &'static str {
        self.as_str()
    }

    fn __repr__(&self) -> String {
        format!("<Schema.{}: '{}'>", self.name(), self.value(),)
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        let Ok(other_enum) = Self::py_from_str(Self::type_object(py), other) else {
            return py.NotImplemented();
        };
        match op {
            CompareOp::Eq => self.eq(&other_enum).into_py(py),
            CompareOp::Ne => self.ne(&other_enum).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    #[getter]
    fn name(&self) -> String {
        self.as_str().to_uppercase()
    }

    #[getter]
    fn value(&self) -> &'static str {
        self.as_str()
    }

    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &PyType, value: &PyAny) -> PyResult<Self> {
        let value_str: &str = value.str().and_then(|s| s.extract())?;
        let tokenized = value_str.replace('_', "-").to_lowercase();
        Self::from_str(&tokenized).map_err(to_val_err)
    }
}

#[pymethods]
impl SType {
    #[new]
    fn py_new(py: Python<'_>, value: &PyAny) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(t, value)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> &'static str {
        self.as_str()
    }

    fn __repr__(&self) -> String {
        format!("<SType.{}: '{}'>", self.name(), self.value(),)
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        let Ok(other_enum) = Self::py_from_str(Self::type_object(py), other) else {
            return py.NotImplemented();
        };
        match op {
            CompareOp::Eq => self.eq(&other_enum).into_py(py),
            CompareOp::Ne => self.ne(&other_enum).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    #[getter]
    fn name(&self) -> String {
        self.as_str().to_uppercase()
    }

    #[getter]
    fn value(&self) -> &'static str {
        self.as_str()
    }

    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &PyType, value: &PyAny) -> PyResult<Self> {
        let value_str: &str = value.str().and_then(|s| s.extract())?;
        let tokenized = value_str.replace('-', "_").to_lowercase();
        Self::from_str(&tokenized).map_err(to_val_err)
    }
}

#[pymethods]
impl RType {
    #[new]
    fn py_new(py: Python<'_>, value: &PyAny) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(t, value).or_else(|_| Self::py_from_int(t, value))
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> &'static str {
        self.as_str()
    }

    fn __repr__(&self) -> String {
        format!("<RType.{}: '{}'>", self.name(), self.value(),)
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        if let Ok(other_enum) = Self::py_from_str(Self::type_object(py), other)
            .or_else(|_| Self::py_from_int(Self::type_object(py), other))
        {
            match op {
                CompareOp::Eq => self.eq(&other_enum).into_py(py),
                CompareOp::Ne => self.ne(&other_enum).into_py(py),
                _ => py.NotImplemented(),
            }
        } else {
            py.NotImplemented()
        }
    }

    #[getter]
    fn name(&self) -> String {
        self.as_str().to_uppercase()
    }

    #[getter]
    fn value(&self) -> u8 {
        *self as u8
    }

    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &PyType, value: &PyAny) -> PyResult<Self> {
        let value_str: &str = value.str().and_then(|s| s.extract())?;
        let tokenized = value_str.replace('-', "_").to_lowercase();
        Self::from_str(&tokenized).map_err(to_val_err)
    }

    #[classmethod]
    #[pyo3(name = "from_int")]
    fn py_from_int(_: &PyType, value: &PyAny) -> PyResult<Self> {
        let value: u8 = value.extract()?;
        Self::try_from(value).map_err(to_val_err)
    }

    #[classmethod]
    #[pyo3(name = "from_schema")]
    fn py_from_schema(pytype: &PyType, value: &PyAny) -> PyResult<Self> {
        let schema: Schema = value
            .extract()
            .or_else(|_| Schema::py_from_str(Schema::type_object(pytype.py()), value))
            .map_err(to_val_err)?;
        Ok(Self::from(schema))
    }
}

impl PyFieldDesc for SecurityUpdateAction {
    fn field_dtypes(field_name: &str) -> Vec<(String, String)> {
        vec![(field_name.to_owned(), "S1".to_owned())]
    }
}

impl PyFieldDesc for UserDefinedInstrument {
    fn field_dtypes(field_name: &str) -> Vec<(String, String)> {
        vec![(field_name.to_owned(), "S1".to_owned())]
    }
}
