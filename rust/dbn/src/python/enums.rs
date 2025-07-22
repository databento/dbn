use std::str::FromStr;

use pyo3::{prelude::*, type_object::PyTypeInfo, types::PyType, Bound};

use crate::{
    enums::{Compression, Encoding, SType, Schema, SecurityUpdateAction, UserDefinedInstrument},
    Action, ErrorCode, InstrumentClass, MatchAlgorithm, RType, Side, StatType, StatUpdateAction,
    StatusAction, StatusReason, SystemCode, TradingEvent, TriState, VersionUpgradePolicy,
};

use super::{to_py_err, EnumIterator, PyFieldDesc};

#[pymethods]
impl RType {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(&t, value).or_else(|_| Self::py_from_int(&t, value))
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> &'static str {
        self.as_str()
    }

    fn __repr__(&self) -> String {
        format!("<RType.{}: {}>", self.name(), self.value())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::Mbp0 => "MBP_0",
            Self::Mbp1 => "MBP_1",
            Self::Mbp10 => "MBP_10",
            #[allow(deprecated)]
            Self::OhlcvDeprecated => "OHLCV_DEPRECATED",
            Self::Ohlcv1S => "OHLCV_1S",
            Self::Ohlcv1M => "OHLCV_1M",
            Self::Ohlcv1H => "OHLCV_1H",
            Self::Ohlcv1D => "OHLCV_1D",
            Self::OhlcvEod => "OHLCV_EOD",
            Self::Status => "STATUS",
            Self::InstrumentDef => "INSTRUMENT_DEF",
            Self::Imbalance => "IMBALANCE",
            Self::Error => "ERROR",
            Self::SymbolMapping => "SYMBOL_MAPPING",
            Self::System => "SYSTEM",
            Self::Statistics => "STATISTICS",
            Self::Mbo => "MBO",
            Self::Cmbp1 => "CMBP_1",
            Self::Cbbo1S => "CBBO_1S",
            Self::Cbbo1M => "CBBO_1M",
            Self::Tcbbo => "TCBBO",
            Self::Bbo1S => "BBO_1S",
            Self::Bbo1M => "BBO_1M",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> u8 {
        *self as u8
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }
    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value_str: String = value.str().and_then(|s| s.extract())?;

        let tokenized = value_str.replace('_', "-").to_lowercase();

        Ok(Self::from_str(&tokenized)?)
    }

    #[classmethod]
    #[pyo3(name = "from_int")]
    fn py_from_int(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value: u8 = value.extract()?;
        Self::try_from(value).map_err(to_py_err)
    }
    #[classmethod]
    #[pyo3(name = "from_schema")]
    fn py_from_schema(pytype: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let schema: Schema = value
            .extract()
            .or_else(|_| Schema::py_from_str(&Schema::type_object(pytype.py()), value))
            .map_err(to_py_err)?;
        Ok(Self::from(schema))
    }
}

#[pymethods]
impl Side {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let Ok(i) = value.extract::<u8>() else {
            let t = Self::type_object(py);
            let c = value.extract::<char>().map_err(to_py_err)?;
            return Self::py_from_str(&t, c);
        };
        Self::try_from(i).map_err(to_py_err)
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

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::Ask => "ASK",
            Self::Bid => "BID",
            Self::None => "NONE",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &Bound<PyType>, value: char) -> PyResult<Self> {
        Self::try_from(value as u8).map_err(to_py_err)
    }
}

#[pymethods]
impl Action {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let Ok(i) = value.extract::<u8>() else {
            let t = Self::type_object(py);
            let c = value.extract::<char>().map_err(to_py_err)?;
            return Self::py_from_str(&t, c);
        };
        Self::try_from(i).map_err(to_py_err)
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

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::Modify => "MODIFY",
            Self::Trade => "TRADE",
            Self::Fill => "FILL",
            Self::Cancel => "CANCEL",
            Self::Add => "ADD",
            Self::Clear => "CLEAR",
            Self::None => "NONE",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &Bound<PyType>, value: char) -> PyResult<Self> {
        Self::try_from(value as u8).map_err(to_py_err)
    }
}

#[pymethods]
impl InstrumentClass {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let Ok(i) = value.extract::<u8>() else {
            let t = Self::type_object(py);
            let c = value.extract::<char>().map_err(to_py_err)?;
            return Self::py_from_str(&t, c);
        };
        Self::try_from(i).map_err(to_py_err)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> String {
        format!("{}", *self as u8 as char)
    }

    fn __repr__(&self) -> String {
        format!("<InstrumentClass.{}: '{}'>", self.name(), self.value())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::Bond => "BOND",
            Self::Call => "CALL",
            Self::Future => "FUTURE",
            Self::Stock => "STOCK",
            Self::MixedSpread => "MIXED_SPREAD",
            Self::Put => "PUT",
            Self::FutureSpread => "FUTURE_SPREAD",
            Self::OptionSpread => "OPTION_SPREAD",
            Self::FxSpot => "FX_SPOT",
            Self::CommoditySpot => "COMMODITY_SPOT",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &Bound<PyType>, value: char) -> PyResult<Self> {
        Self::try_from(value as u8).map_err(to_py_err)
    }
}

#[pymethods]
impl MatchAlgorithm {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let Ok(i) = value.extract::<u8>() else {
            let t = Self::type_object(py);
            let c = value.extract::<char>().map_err(to_py_err)?;
            return Self::py_from_str(&t, c);
        };
        Self::try_from(i).map_err(to_py_err)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> String {
        format!("{}", *self as u8 as char)
    }

    fn __repr__(&self) -> String {
        format!("<MatchAlgorithm.{}: '{}'>", self.name(), self.value())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::Undefined => "UNDEFINED",
            Self::Fifo => "FIFO",
            Self::Configurable => "CONFIGURABLE",
            Self::ProRata => "PRO_RATA",
            Self::FifoLmm => "FIFO_LMM",
            Self::ThresholdProRata => "THRESHOLD_PRO_RATA",
            Self::FifoTopLmm => "FIFO_TOP_LMM",
            Self::ThresholdProRataLmm => "THRESHOLD_PRO_RATA_LMM",
            Self::EurodollarFutures => "EURODOLLAR_FUTURES",
            Self::TimeProRata => "TIME_PRO_RATA",
            Self::InstitutionalPrioritization => "INSTITUTIONAL_PRIORITIZATION",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &Bound<PyType>, value: char) -> PyResult<Self> {
        Self::try_from(value as u8).map_err(to_py_err)
    }
}

#[pymethods]
impl UserDefinedInstrument {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let Ok(i) = value.extract::<u8>() else {
            let t = Self::type_object(py);
            let c = value.extract::<char>().map_err(to_py_err)?;
            return Self::py_from_str(&t, c);
        };
        Self::try_from(i).map_err(to_py_err)
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

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::No => "NO",
            Self::Yes => "YES",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &Bound<PyType>, value: char) -> PyResult<Self> {
        Self::try_from(value as u8).map_err(to_py_err)
    }
}

impl PyFieldDesc for UserDefinedInstrument {
    fn field_dtypes(field_name: &str) -> Vec<(String, String)> {
        vec![(field_name.to_owned(), "S1".to_owned())]
    }
}

#[pymethods]
impl SecurityUpdateAction {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let Ok(i) = value.extract::<u8>() else {
            let t = Self::type_object(py);
            let c = value.extract::<char>().map_err(to_py_err)?;
            return Self::py_from_str(&t, c);
        };
        Self::try_from(i).map_err(to_py_err)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> String {
        format!("{}", *self as u8 as char)
    }

    fn __repr__(&self) -> String {
        format!("<SecurityUpdateAction.{}: '{}'>", self.name(), self.value())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::Add => "ADD",
            Self::Modify => "MODIFY",
            Self::Delete => "DELETE",
            #[allow(deprecated)]
            Self::Invalid => "INVALID",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &Bound<PyType>, value: char) -> PyResult<Self> {
        Self::try_from(value as u8).map_err(to_py_err)
    }
}

impl PyFieldDesc for SecurityUpdateAction {
    fn field_dtypes(field_name: &str) -> Vec<(String, String)> {
        vec![(field_name.to_owned(), "S1".to_owned())]
    }
}

#[pymethods]
impl SType {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(&t, value).or_else(|_| Self::py_from_int(&t, value))
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> &'static str {
        self.as_str()
    }

    fn __repr__(&self) -> String {
        format!("<SType.{}: '{}'>", self.name(), self.value())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::InstrumentId => "INSTRUMENT_ID",
            Self::RawSymbol => "RAW_SYMBOL",
            #[allow(deprecated)]
            Self::Smart => "SMART",
            Self::Continuous => "CONTINUOUS",
            Self::Parent => "PARENT",
            Self::NasdaqSymbol => "NASDAQ_SYMBOL",
            Self::CmsSymbol => "CMS_SYMBOL",
            Self::Isin => "ISIN",
            Self::UsCode => "US_CODE",
            Self::BbgCompId => "BBG_COMP_ID",
            Self::BbgCompTicker => "BBG_COMP_TICKER",
            Self::Figi => "FIGI",
            Self::FigiTicker => "FIGI_TICKER",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> &'static str {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }
    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value_str: String = value.str().and_then(|s| s.extract())?;

        let tokenized = value_str.replace('-', "_").to_lowercase();

        Ok(Self::from_str(&tokenized)?)
    }

    #[classmethod]
    #[pyo3(name = "from_int")]
    fn py_from_int(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value: u8 = value.extract()?;
        Self::try_from(value).map_err(to_py_err)
    }
}

#[pymethods]
impl Schema {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(&t, value).or_else(|_| Self::py_from_int(&t, value))
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> &'static str {
        self.as_str()
    }

    fn __repr__(&self) -> String {
        format!("<Schema.{}: '{}'>", self.name(), self.value())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::Mbo => "MBO",
            Self::Mbp1 => "MBP_1",
            Self::Mbp10 => "MBP_10",
            Self::Tbbo => "TBBO",
            Self::Trades => "TRADES",
            Self::Ohlcv1S => "OHLCV_1S",
            Self::Ohlcv1M => "OHLCV_1M",
            Self::Ohlcv1H => "OHLCV_1H",
            Self::Ohlcv1D => "OHLCV_1D",
            Self::Definition => "DEFINITION",
            Self::Statistics => "STATISTICS",
            Self::Status => "STATUS",
            Self::Imbalance => "IMBALANCE",
            Self::OhlcvEod => "OHLCV_EOD",
            Self::Cmbp1 => "CMBP_1",
            Self::Cbbo1S => "CBBO_1S",
            Self::Cbbo1M => "CBBO_1M",
            Self::Tcbbo => "TCBBO",
            Self::Bbo1S => "BBO_1S",
            Self::Bbo1M => "BBO_1M",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> &'static str {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }
    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value_str: String = value.str().and_then(|s| s.extract())?;

        let tokenized = value_str.replace('_', "-").to_lowercase();

        Ok(Self::from_str(&tokenized)?)
    }

    #[classmethod]
    #[pyo3(name = "from_int")]
    fn py_from_int(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value: u16 = value.extract()?;
        Self::try_from(value).map_err(to_py_err)
    }
}

#[pymethods]
impl Encoding {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(&t, value).or_else(|_| Self::py_from_int(&t, value))
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> &'static str {
        self.as_str()
    }

    fn __repr__(&self) -> String {
        format!("<Encoding.{}: '{}'>", self.name(), self.value())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::Dbn => "DBN",
            Self::Csv => "CSV",
            Self::Json => "JSON",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> &'static str {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }
    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value_str: String = value.str().and_then(|s| s.extract())?;

        let tokenized = value_str.replace('-', "_").to_lowercase();

        Ok(Self::from_str(&tokenized)?)
    }

    #[classmethod]
    #[pyo3(name = "from_int")]
    fn py_from_int(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value: u8 = value.extract()?;
        Self::try_from(value).map_err(to_py_err)
    }
}

#[pymethods]
impl Compression {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(&t, value).or_else(|_| Self::py_from_int(&t, value))
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> &'static str {
        self.as_str()
    }

    fn __repr__(&self) -> String {
        format!("<Compression.{}: '{}'>", self.name(), self.value())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Zstd => "ZSTD",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> &'static str {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }
    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value_str: String = value.str().and_then(|s| s.extract())?;

        let tokenized = value_str.replace('-', "_").to_lowercase();

        Ok(Self::from_str(&tokenized)?)
    }

    #[classmethod]
    #[pyo3(name = "from_int")]
    fn py_from_int(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value: u8 = value.extract()?;
        Self::try_from(value).map_err(to_py_err)
    }
}

#[pymethods]
impl StatType {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_int(&t, value)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __repr__(&self) -> String {
        format!("<StatType.{}: {}>", self.name(), self.value())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::OpeningPrice => "OPENING_PRICE",
            Self::IndicativeOpeningPrice => "INDICATIVE_OPENING_PRICE",
            Self::SettlementPrice => "SETTLEMENT_PRICE",
            Self::TradingSessionLowPrice => "TRADING_SESSION_LOW_PRICE",
            Self::TradingSessionHighPrice => "TRADING_SESSION_HIGH_PRICE",
            Self::ClearedVolume => "CLEARED_VOLUME",
            Self::LowestOffer => "LOWEST_OFFER",
            Self::HighestBid => "HIGHEST_BID",
            Self::OpenInterest => "OPEN_INTEREST",
            Self::FixingPrice => "FIXING_PRICE",
            Self::ClosePrice => "CLOSE_PRICE",
            Self::NetChange => "NET_CHANGE",
            Self::Vwap => "VWAP",
            Self::Volatility => "VOLATILITY",
            Self::Delta => "DELTA",
            Self::UncrossingPrice => "UNCROSSING_PRICE",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> u16 {
        *self as u16
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }
    #[classmethod]
    #[pyo3(name = "from_int")]
    fn py_from_int(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value: u16 = value.extract()?;
        Self::try_from(value).map_err(to_py_err)
    }
}

#[pymethods]
impl StatUpdateAction {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_int(&t, value)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __repr__(&self) -> String {
        format!("<StatUpdateAction.{}: {}>", self.name(), self.value())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::New => "NEW",
            Self::Delete => "DELETE",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> u8 {
        *self as u8
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }
    #[classmethod]
    #[pyo3(name = "from_int")]
    fn py_from_int(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value: u8 = value.extract()?;
        Self::try_from(value).map_err(to_py_err)
    }
}

#[pymethods]
impl StatusAction {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_int(&t, value)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __repr__(&self) -> String {
        format!("<StatusAction.{}: {}>", self.name(), self.value())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::PreOpen => "PRE_OPEN",
            Self::PreCross => "PRE_CROSS",
            Self::Quoting => "QUOTING",
            Self::Cross => "CROSS",
            Self::Rotation => "ROTATION",
            Self::NewPriceIndication => "NEW_PRICE_INDICATION",
            Self::Trading => "TRADING",
            Self::Halt => "HALT",
            Self::Pause => "PAUSE",
            Self::Suspend => "SUSPEND",
            Self::PreClose => "PRE_CLOSE",
            Self::Close => "CLOSE",
            Self::PostClose => "POST_CLOSE",
            Self::SsrChange => "SSR_CHANGE",
            Self::NotAvailableForTrading => "NOT_AVAILABLE_FOR_TRADING",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> u16 {
        *self as u16
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }
    #[classmethod]
    #[pyo3(name = "from_int")]
    fn py_from_int(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value: u16 = value.extract()?;
        Self::try_from(value).map_err(to_py_err)
    }
}

#[pymethods]
impl StatusReason {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_int(&t, value)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __repr__(&self) -> String {
        format!("<StatusReason.{}: {}>", self.name(), self.value())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Scheduled => "SCHEDULED",
            Self::SurveillanceIntervention => "SURVEILLANCE_INTERVENTION",
            Self::MarketEvent => "MARKET_EVENT",
            Self::InstrumentActivation => "INSTRUMENT_ACTIVATION",
            Self::InstrumentExpiration => "INSTRUMENT_EXPIRATION",
            Self::RecoveryInProcess => "RECOVERY_IN_PROCESS",
            Self::Regulatory => "REGULATORY",
            Self::Administrative => "ADMINISTRATIVE",
            Self::NonCompliance => "NON_COMPLIANCE",
            Self::FilingsNotCurrent => "FILINGS_NOT_CURRENT",
            Self::SecTradingSuspension => "SEC_TRADING_SUSPENSION",
            Self::NewIssue => "NEW_ISSUE",
            Self::IssueAvailable => "ISSUE_AVAILABLE",
            Self::IssuesReviewed => "ISSUES_REVIEWED",
            Self::FilingReqsSatisfied => "FILING_REQS_SATISFIED",
            Self::NewsPending => "NEWS_PENDING",
            Self::NewsReleased => "NEWS_RELEASED",
            Self::NewsAndResumptionTimes => "NEWS_AND_RESUMPTION_TIMES",
            Self::NewsNotForthcoming => "NEWS_NOT_FORTHCOMING",
            Self::OrderImbalance => "ORDER_IMBALANCE",
            Self::LuldPause => "LULD_PAUSE",
            Self::Operational => "OPERATIONAL",
            Self::AdditionalInformationRequested => "ADDITIONAL_INFORMATION_REQUESTED",
            Self::MergerEffective => "MERGER_EFFECTIVE",
            Self::Etf => "ETF",
            Self::CorporateAction => "CORPORATE_ACTION",
            Self::NewSecurityOffering => "NEW_SECURITY_OFFERING",
            Self::MarketWideHaltLevel1 => "MARKET_WIDE_HALT_LEVEL1",
            Self::MarketWideHaltLevel2 => "MARKET_WIDE_HALT_LEVEL2",
            Self::MarketWideHaltLevel3 => "MARKET_WIDE_HALT_LEVEL3",
            Self::MarketWideHaltCarryover => "MARKET_WIDE_HALT_CARRYOVER",
            Self::MarketWideHaltResumption => "MARKET_WIDE_HALT_RESUMPTION",
            Self::QuotationNotAvailable => "QUOTATION_NOT_AVAILABLE",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> u16 {
        *self as u16
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }
    #[classmethod]
    #[pyo3(name = "from_int")]
    fn py_from_int(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value: u16 = value.extract()?;
        Self::try_from(value).map_err(to_py_err)
    }
}

#[pymethods]
impl TradingEvent {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_int(&t, value)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __repr__(&self) -> String {
        format!("<TradingEvent.{}: {}>", self.name(), self.value())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::NoCancel => "NO_CANCEL",
            Self::ChangeTradingSession => "CHANGE_TRADING_SESSION",
            Self::ImpliedMatchingOn => "IMPLIED_MATCHING_ON",
            Self::ImpliedMatchingOff => "IMPLIED_MATCHING_OFF",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> u16 {
        *self as u16
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }
    #[classmethod]
    #[pyo3(name = "from_int")]
    fn py_from_int(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value: u16 = value.extract()?;
        Self::try_from(value).map_err(to_py_err)
    }
}

#[pymethods]
impl TriState {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let Ok(i) = value.extract::<u8>() else {
            let t = Self::type_object(py);
            let c = value.extract::<char>().map_err(to_py_err)?;
            return Self::py_from_str(&t, c);
        };
        Self::try_from(i).map_err(to_py_err)
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> String {
        format!("{}", *self as u8 as char)
    }

    fn __repr__(&self) -> String {
        format!("<TriState.{}: '{}'>", self.name(), self.value())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::NotAvailable => "NOT_AVAILABLE",
            Self::No => "NO",
            Self::Yes => "YES",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> String {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }

    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &Bound<PyType>, value: char) -> PyResult<Self> {
        Self::try_from(value as u8).map_err(to_py_err)
    }

    fn opt_bool(&self) -> Option<bool> {
        Option::from(*self)
    }
}

#[pymethods]
impl VersionUpgradePolicy {
    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __repr__(&self) -> String {
        format!("<VersionUpgradePolicy.{}>", self.name())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::AsIs => "AS_IS",
            Self::UpgradeToV2 => "UPGRADE_TO_V2",
            Self::UpgradeToV3 => "UPGRADE_TO_V3",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, _py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>() else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }
}

#[pymethods]
impl ErrorCode {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(&t, value).or_else(|_| Self::py_from_int(&t, value))
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> &'static str {
        self.as_str()
    }

    fn __repr__(&self) -> String {
        format!("<ErrorCode.{}: '{}'>", self.name(), self.value())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::AuthFailed => "AUTH_FAILED",
            Self::ApiKeyDeactivated => "API_KEY_DEACTIVATED",
            Self::ConnectionLimitExceeded => "CONNECTION_LIMIT_EXCEEDED",
            Self::SymbolResolutionFailed => "SYMBOL_RESOLUTION_FAILED",
            Self::InvalidSubscription => "INVALID_SUBSCRIPTION",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> &'static str {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }
    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value_str: String = value.str().and_then(|s| s.extract())?;

        let tokenized = value_str.replace('-', "_").to_lowercase();

        Ok(Self::from_str(&tokenized)?)
    }

    #[classmethod]
    #[pyo3(name = "from_int")]
    fn py_from_int(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value: u8 = value.extract()?;
        Self::try_from(value).map_err(to_py_err)
    }
}

#[pymethods]
impl SystemCode {
    #[new]
    fn py_new(py: Python<'_>, value: &Bound<PyAny>) -> PyResult<Self> {
        let t = Self::type_object(py);
        Self::py_from_str(&t, value).or_else(|_| Self::py_from_int(&t, value))
    }

    fn __hash__(&self) -> isize {
        *self as isize
    }

    fn __str__(&self) -> &'static str {
        self.as_str()
    }

    fn __repr__(&self) -> String {
        format!("<SystemCode.{}: '{}'>", self.name(), self.value())
    }

    #[getter]
    fn name(&self) -> &'static str {
        match self {
            Self::Heartbeat => "HEARTBEAT",
            Self::SubscriptionAck => "SUBSCRIPTION_ACK",
            Self::SlowReaderWarning => "SLOW_READER_WARNING",
            Self::ReplayCompleted => "REPLAY_COMPLETED",
        }
    }

    fn __eq__(&self, other: &Bound<PyAny>, py: Python<'_>) -> bool {
        let Ok(other_enum) = other.extract::<Self>().or_else(|_| Self::py_new(py, other)) else {
            return false;
        };
        self.eq(&other_enum)
    }

    #[getter]
    fn value(&self) -> &'static str {
        self.__str__()
    }

    #[classmethod]
    fn variants(_: &Bound<PyType>, py: Python<'_>) -> PyResult<EnumIterator> {
        EnumIterator::new::<Self>(py)
    }
    #[classmethod]
    #[pyo3(name = "from_str")]
    fn py_from_str(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value_str: String = value.str().and_then(|s| s.extract())?;

        let tokenized = value_str.replace('-', "_").to_lowercase();

        Ok(Self::from_str(&tokenized)?)
    }

    #[classmethod]
    #[pyo3(name = "from_int")]
    fn py_from_int(_: &Bound<PyType>, value: &Bound<PyAny>) -> PyResult<Self> {
        let value: u8 = value.extract()?;
        Self::try_from(value).map_err(to_py_err)
    }
}
