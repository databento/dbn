use std::str::FromStr;

use pyo3::{prelude::*, pyclass::CompareOp, type_object::PyTypeInfo, types::PyType};

use crate::{
    enums::{Compression, Encoding, SType, Schema, SecurityUpdateAction, UserDefinedInstrument},
    Action, InstrumentClass, MatchAlgorithm, RType, Side, StatType, StatusAction, StatusReason,
    TradingEvent, TriState, VersionUpgradePolicy,
};

use super::{to_val_err, EnumIterator, PyFieldDesc};

#[pymethods]
impl Side {
    #[new]
    fn py_new(py: Python<'_>, value: &PyAny) -> PyResult<Self> {
        let Ok(i) = value.extract::<u8>() else {
            let t = Self::type_object(py);
            let c = value.extract::<char>().map_err(to_val_err)?;
            return Self::py_from_str(t, c);
        };
        Self::try_from(i).map_err(to_val_err)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> String {
        format!("{}", *self as u8 as char)
    }

    fn __repr__(&self) -> String {
        format!("<Side.{}: '{}'>", self.name(), self.value())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
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
        self.as_ref().to_ascii_uppercase()
    }

    #[getter]
    fn value(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &PyType, value: char) -> PyResult<Self> {
        Self::try_from(value as u8).map_err(to_val_err)
    }
}

#[pymethods]
impl Action {
    #[new]
    fn py_new(py: Python<'_>, value: &PyAny) -> PyResult<Self> {
        let Ok(i) = value.extract::<u8>() else {
            let t = Self::type_object(py);
            let c = value.extract::<char>().map_err(to_val_err)?;
            return Self::py_from_str(t, c);
        };
        Self::try_from(i).map_err(to_val_err)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> String {
        format!("{}", *self as u8 as char)
    }

    fn __repr__(&self) -> String {
        format!("<Action.{}: '{}'>", self.name(), self.value())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
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
        self.as_ref().to_ascii_uppercase()
    }

    #[getter]
    fn value(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &PyType, value: char) -> PyResult<Self> {
        Self::try_from(value as u8).map_err(to_val_err)
    }
}

#[pymethods]
impl InstrumentClass {
    #[new]
    fn py_new(py: Python<'_>, value: &PyAny) -> PyResult<Self> {
        let Ok(i) = value.extract::<u8>() else {
            let t = Self::type_object(py);
            let c = value.extract::<char>().map_err(to_val_err)?;
            return Self::py_from_str(t, c);
        };
        Self::try_from(i).map_err(to_val_err)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> String {
        format!("{}", *self as u8 as char)
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return py.NotImplemented();
        };
        match op {
            CompareOp::Eq => self.eq(&other_enum).into_py(py),
            CompareOp::Ne => self.ne(&other_enum).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    #[getter]
    fn value(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &PyType, value: char) -> PyResult<Self> {
        Self::try_from(value as u8).map_err(to_val_err)
    }
}

#[pymethods]
impl MatchAlgorithm {
    #[new]
    fn py_new(py: Python<'_>, value: &PyAny) -> PyResult<Self> {
        let Ok(i) = value.extract::<u8>() else {
            let t = Self::type_object(py);
            let c = value.extract::<char>().map_err(to_val_err)?;
            return Self::py_from_str(t, c);
        };
        Self::try_from(i).map_err(to_val_err)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> String {
        format!("{}", *self as u8 as char)
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return py.NotImplemented();
        };
        match op {
            CompareOp::Eq => self.eq(&other_enum).into_py(py),
            CompareOp::Ne => self.ne(&other_enum).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    #[getter]
    fn value(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &PyType, value: char) -> PyResult<Self> {
        Self::try_from(value as u8).map_err(to_val_err)
    }
}

#[pymethods]
impl UserDefinedInstrument {
    #[new]
    fn py_new(py: Python<'_>, value: &PyAny) -> PyResult<Self> {
        let Ok(i) = value.extract::<u8>() else {
            let t = Self::type_object(py);
            let c = value.extract::<char>().map_err(to_val_err)?;
            return Self::py_from_str(t, c);
        };
        Self::try_from(i).map_err(to_val_err)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> String {
        format!("{}", *self as u8 as char)
    }

    fn __repr__(&self) -> String {
        format!(
            "<UserDefinedInstrument.{}: '{}'>",
            self.name(),
            self.value()
        )
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
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
        self.as_ref().to_ascii_uppercase()
    }

    #[getter]
    fn value(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &PyType, value: char) -> PyResult<Self> {
        Self::try_from(value as u8).map_err(to_val_err)
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
impl SecurityUpdateAction {
    #[new]
    fn py_new(py: Python<'_>, value: &PyAny) -> PyResult<Self> {
        let Ok(i) = value.extract::<u8>() else {
            let t = Self::type_object(py);
            let c = value.extract::<char>().map_err(to_val_err)?;
            return Self::py_from_str(t, c);
        };
        Self::try_from(i).map_err(to_val_err)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __repr__(&self) -> String {
        format!("<SecurityUpdateAction.{}: '{}'>", self.name(), self.value())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
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
        self.as_ref().to_ascii_uppercase()
    }

    #[getter]
    fn value(&self) -> u16 {
        *self as u16
    }

    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &PyType, value: char) -> PyResult<Self> {
        Self::try_from(value as u8).map_err(to_val_err)
    }
}

#[pymethods]
impl StatType {
    #[new]
    fn py_new(value: &PyAny) -> PyResult<Self> {
        let i = value.extract::<u16>().map_err(to_val_err)?;
        Self::try_from(i).map_err(to_val_err)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(other)) else {
            return py.NotImplemented();
        };
        match op {
            CompareOp::Eq => self.eq(&other_enum).into_py(py),
            CompareOp::Ne => self.ne(&other_enum).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    #[getter]
    fn value(&self) -> u16 {
        *self as u16
    }

    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
    }
}

#[pymethods]
impl StatusAction {
    #[new]
    fn py_new(value: &PyAny) -> PyResult<Self> {
        let i = value.extract::<u16>().map_err(to_val_err)?;
        Self::try_from(i).map_err(to_val_err)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(other)) else {
            return py.NotImplemented();
        };
        match op {
            CompareOp::Eq => self.eq(&other_enum).into_py(py),
            CompareOp::Ne => self.ne(&other_enum).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    #[getter]
    fn value(&self) -> u16 {
        *self as u16
    }

    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
    }
}

#[pymethods]
impl StatusReason {
    #[new]
    fn py_new(value: &PyAny) -> PyResult<Self> {
        let i = value.extract::<u16>().map_err(to_val_err)?;
        Self::try_from(i).map_err(to_val_err)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(other)) else {
            return py.NotImplemented();
        };
        match op {
            CompareOp::Eq => self.eq(&other_enum).into_py(py),
            CompareOp::Ne => self.ne(&other_enum).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    #[getter]
    fn value(&self) -> u16 {
        *self as u16
    }

    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
    }
}

#[pymethods]
impl TradingEvent {
    #[new]
    fn py_new(value: &PyAny) -> PyResult<Self> {
        let i = value.extract::<u16>().map_err(to_val_err)?;
        Self::try_from(i).map_err(to_val_err)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(other)) else {
            return py.NotImplemented();
        };
        match op {
            CompareOp::Eq => self.eq(&other_enum).into_py(py),
            CompareOp::Ne => self.ne(&other_enum).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    #[getter]
    fn value(&self) -> u16 {
        *self as u16
    }

    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
    }
}

#[pymethods]
impl TriState {
    #[new]
    fn py_new(py: Python<'_>, value: &PyAny) -> PyResult<Self> {
        let Ok(i) = value.extract::<u8>() else {
            let t = Self::type_object(py);
            let c = value.extract::<char>().map_err(to_val_err)?;
            return Self::py_from_str(t, c);
        };
        Self::try_from(i).map_err(to_val_err)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> String {
        format!("{}", *self as u8 as char)
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return py.NotImplemented();
        };
        match op {
            CompareOp::Eq => self.eq(&other_enum).into_py(py),
            CompareOp::Ne => self.ne(&other_enum).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    fn opt_bool(&self) -> Option<bool> {
        Option::from(*self)
    }

    #[getter]
    fn value(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &PyType, value: char) -> PyResult<Self> {
        Self::try_from(value as u8).map_err(to_val_err)
    }
}

#[pymethods]
impl VersionUpgradePolicy {
    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp, py: Python<'_>) -> Py<PyAny> {
        let Ok(other_enum) = other.extract::<Self>() else {
            return py.NotImplemented();
        };
        match op {
            CompareOp::Eq => self.eq(&other_enum).into_py(py),
            CompareOp::Ne => self.ne(&other_enum).into_py(py),
            _ => py.NotImplemented(),
        }
    }

    #[classmethod]
    fn variants(_: &PyType, py: Python<'_>) -> EnumIterator {
        EnumIterator::new::<Self>(py)
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
