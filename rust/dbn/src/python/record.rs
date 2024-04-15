use std::{ffi::c_char, mem};

use pyo3::{
    intern,
    prelude::*,
    pyclass::CompareOp,
    types::{timezone_utc_bound, PyDateTime, PyDict},
};

use crate::{
    compat::{ErrorMsgV1, InstrumentDefMsgV1, SymbolMappingMsgV1, SystemMsgV1},
    record::{str_to_c_chars, CbboMsg, ConsolidatedBidAskPair},
    rtype, BidAskPair, ErrorMsg, HasRType, ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg,
    Mbp1Msg, OhlcvMsg, Publisher, Record, RecordHeader, SType, SecurityUpdateAction, StatMsg,
    StatUpdateAction, StatusAction, StatusMsg, StatusReason, SymbolMappingMsg, SystemMsg, TradeMsg,
    TradingEvent, TriState, UserDefinedInstrument, WithTsOut, FIXED_PRICE_SCALE, UNDEF_ORDER_SIZE,
    UNDEF_PRICE, UNDEF_TIMESTAMP,
};

use super::{to_val_err, PyFieldDesc};

#[pymethods]
impl MboMsg {
    #[new]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        order_id: u64,
        price: i64,
        size: u32,
        channel_id: u8,
        action: c_char,
        side: c_char,
        ts_recv: u64,
        ts_in_delta: i32,
        sequence: u32,
        flags: Option<u8>,
    ) -> Self {
        Self {
            hd: RecordHeader::new::<Self>(rtype::MBO, publisher_id, instrument_id, ts_event),
            order_id,
            price,
            size,
            flags: flags.unwrap_or_default(),
            channel_id,
            action,
            side,
            ts_recv,
            ts_in_delta,
            sequence,
        }
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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

    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_price")]
    fn py_pretty_price(&self) -> f64 {
        self.price as f64 / FIXED_PRICE_SCALE as f64
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[getter]
    #[pyo3(name = "pretty_ts_recv")]
    fn py_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_recv)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<MboMsg>())
    }

    #[getter]
    #[pyo3(name = "action")]
    fn py_action(&self) -> char {
        self.action as u8 as char
    }

    #[getter]
    #[pyo3(name = "side")]
    fn py_side(&self) -> char {
        self.side as u8 as char
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
    }
}

#[pymethods]
impl BidAskPair {
    #[new]
    fn py_new(
        bid_px: Option<i64>,
        ask_px: Option<i64>,
        bid_sz: Option<u32>,
        ask_sz: Option<u32>,
        bid_ct: Option<u32>,
        ask_ct: Option<u32>,
    ) -> Self {
        Self {
            bid_px: bid_px.unwrap_or(UNDEF_PRICE),
            ask_px: ask_px.unwrap_or(UNDEF_PRICE),
            bid_sz: bid_sz.unwrap_or_default(),
            ask_sz: ask_sz.unwrap_or_default(),
            bid_ct: bid_ct.unwrap_or_default(),
            ask_ct: ask_ct.unwrap_or_default(),
        }
    }

    #[getter]
    #[pyo3(name = "pretty_ask_px")]
    fn py_pretty_ask_px(&self) -> f64 {
        match self.ask_px {
            UNDEF_PRICE => f64::NAN,
            _ => self.ask_px as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_bid_px")]
    fn py_pretty_bid_px(&self) -> f64 {
        match self.bid_px {
            UNDEF_PRICE => f64::NAN,
            _ => self.bid_px as f64 / FIXED_PRICE_SCALE as f64,
        }
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
}

#[pymethods]
impl CbboMsg {
    #[new]
    fn py_new(
        rtype: u8,
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        price: i64,
        size: u32,
        action: c_char,
        side: c_char,
        flags: u8,
        ts_recv: u64,
        ts_in_delta: i32,
        sequence: u32,
        levels: Option<ConsolidatedBidAskPair>,
    ) -> Self {
        Self {
            hd: RecordHeader::new::<Self>(rtype, publisher_id, instrument_id, ts_event),
            price,
            size,
            action,
            side,
            flags,
            _reserved: [0; 1],
            ts_recv,
            ts_in_delta,
            sequence,
            levels: [levels.unwrap_or_default()],
        }
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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

    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_price")]
    fn py_pretty_price(&self) -> f64 {
        self.price as f64 / FIXED_PRICE_SCALE as f64
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[getter]
    #[pyo3(name = "pretty_ts_recv")]
    fn py_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_recv)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<CbboMsg>())
    }

    #[getter]
    #[pyo3(name = "action")]
    fn py_action(&self) -> char {
        self.action as u8 as char
    }

    #[getter]
    #[pyo3(name = "side")]
    fn py_side(&self) -> char {
        self.side as u8 as char
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
    }
}

#[pymethods]
impl ConsolidatedBidAskPair {
    #[new]
    fn py_new(
        bid_px: Option<i64>,
        ask_px: Option<i64>,
        bid_sz: Option<u32>,
        ask_sz: Option<u32>,
        bid_pb: Option<u16>,
        ask_pb: Option<u16>,
    ) -> Self {
        Self {
            bid_px: bid_px.unwrap_or(UNDEF_PRICE),
            ask_px: ask_px.unwrap_or(UNDEF_PRICE),
            bid_sz: bid_sz.unwrap_or_default(),
            ask_sz: ask_sz.unwrap_or_default(),
            bid_pb: bid_pb.unwrap_or_default(),
            ask_pb: ask_pb.unwrap_or_default(),
            _reserved1: [0; 2],
            _reserved2: [0; 2],
        }
    }

    #[getter]
    #[pyo3(name = "pretty_ask_px")]
    fn py_pretty_ask_px(&self) -> f64 {
        match self.ask_px {
            UNDEF_PRICE => f64::NAN,
            _ => self.ask_px as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_bid_px")]
    fn py_pretty_bid_px(&self) -> f64 {
        match self.bid_px {
            UNDEF_PRICE => f64::NAN,
            _ => self.bid_px as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_ask_pb")]
    fn py_pretty_ask_pb(&self) -> Option<String> {
        Publisher::try_from(self.ask_pb)
            .map(|pb| pb.as_str().to_owned())
            .ok()
    }

    #[getter]
    #[pyo3(name = "pretty_bid_pb")]
    fn py_pretty_bid_pb(&self) -> Option<String> {
        Publisher::try_from(self.bid_pb)
            .map(|pb| pb.as_str().to_owned())
            .ok()
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
}

#[pymethods]
impl TradeMsg {
    #[new]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        price: i64,
        size: u32,
        action: c_char,
        side: c_char,
        depth: u8,
        ts_recv: u64,
        ts_in_delta: i32,
        sequence: u32,
        flags: Option<u8>,
    ) -> Self {
        Self {
            hd: RecordHeader::new::<Self>(rtype::MBP_0, publisher_id, instrument_id, ts_event),
            price,
            size,
            action,
            side,
            flags: flags.unwrap_or_default(),
            depth,
            ts_recv,
            ts_in_delta,
            sequence,
        }
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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

    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_price")]
    fn py_pretty_price(&self) -> f64 {
        self.price as f64 / FIXED_PRICE_SCALE as f64
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[getter]
    #[pyo3(name = "pretty_ts_recv")]
    fn py_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_recv)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<TradeMsg>())
    }

    #[getter]
    #[pyo3(name = "action")]
    fn py_action(&self) -> char {
        self.action as u8 as char
    }

    #[getter]
    #[pyo3(name = "side")]
    fn py_side(&self) -> char {
        self.side as u8 as char
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
    }
}

#[pymethods]
impl Mbp1Msg {
    #[new]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        price: i64,
        size: u32,
        action: c_char,
        side: c_char,
        flags: u8,
        depth: u8,
        ts_recv: u64,
        ts_in_delta: i32,
        sequence: u32,
        levels: Option<BidAskPair>,
    ) -> Self {
        Self {
            hd: RecordHeader::new::<Self>(rtype::MBP_1, publisher_id, instrument_id, ts_event),
            price,
            size,
            action,
            side,
            flags,
            depth,
            ts_recv,
            ts_in_delta,
            sequence,
            levels: [levels.unwrap_or_default()],
        }
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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

    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_price")]
    fn py_pretty_price(&self) -> f64 {
        self.price as f64 / FIXED_PRICE_SCALE as f64
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[getter]
    #[pyo3(name = "pretty_ts_recv")]
    fn py_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_recv)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Mbp1Msg>())
    }

    #[getter]
    #[pyo3(name = "action")]
    fn py_action(&self) -> char {
        self.action as u8 as char
    }

    #[getter]
    #[pyo3(name = "side")]
    fn py_side(&self) -> char {
        self.side as u8 as char
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
    }
}

#[pymethods]
impl Mbp10Msg {
    #[new]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        price: i64,
        size: u32,
        action: c_char,
        side: c_char,
        flags: u8,
        depth: u8,
        ts_recv: u64,
        ts_in_delta: i32,
        sequence: u32,
        levels: Option<Vec<BidAskPair>>,
    ) -> PyResult<Self> {
        let levels = if let Some(level) = levels {
            let mut arr: [BidAskPair; 10] = Default::default();
            if level.len() > 10 {
                return Err(to_val_err("Only 10 levels are allowed"));
            }
            for (i, level) in level.into_iter().enumerate() {
                arr[i] = level;
            }
            arr
        } else {
            Default::default()
        };
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::MBP_10, publisher_id, instrument_id, ts_event),
            price,
            size,
            action,
            side,
            flags,
            depth,
            ts_recv,
            ts_in_delta,
            sequence,
            levels,
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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

    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_price")]
    fn py_pretty_price(&self) -> f64 {
        self.price as f64 / FIXED_PRICE_SCALE as f64
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[getter]
    #[pyo3(name = "pretty_ts_recv")]
    fn py_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_recv)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Mbp10Msg>())
    }

    #[getter]
    #[pyo3(name = "action")]
    fn py_action(&self) -> char {
        self.action as u8 as char
    }

    #[getter]
    #[pyo3(name = "side")]
    fn py_side(&self) -> char {
        self.side as u8 as char
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
    }
}

#[pymethods]
impl OhlcvMsg {
    #[new]
    fn py_new(
        rtype: u8,
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        open: i64,
        high: i64,
        low: i64,
        close: i64,
        volume: u64,
    ) -> Self {
        Self {
            hd: RecordHeader::new::<Self>(rtype, publisher_id, instrument_id, ts_event),
            open,
            high,
            low,
            close,
            volume,
        }
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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

    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_open")]
    fn py_pretty_open(&self) -> f64 {
        self.open as f64 / FIXED_PRICE_SCALE as f64
    }

    #[getter]
    #[pyo3(name = "pretty_high")]
    fn py_pretty_high(&self) -> f64 {
        self.high as f64 / FIXED_PRICE_SCALE as f64
    }

    #[getter]
    #[pyo3(name = "pretty_low")]
    fn py_pretty_low(&self) -> f64 {
        self.low as f64 / FIXED_PRICE_SCALE as f64
    }

    #[getter]
    #[pyo3(name = "pretty_close")]
    fn py_pretty_close(&self) -> f64 {
        self.close as f64 / FIXED_PRICE_SCALE as f64
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<OhlcvMsg>())
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
    }
}

#[pymethods]
impl StatusMsg {
    #[new]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        ts_recv: u64,
        action: Option<u16>,
        reason: Option<u16>,
        trading_event: Option<u16>,
        is_trading: Option<bool>,
        is_quoting: Option<bool>,
        is_short_sell_restricted: Option<bool>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::STATUS, publisher_id, instrument_id, ts_event),
            ts_recv,
            action: action.unwrap_or_else(|| StatusAction::default() as u16),
            reason: reason.unwrap_or_else(|| StatusReason::default() as u16),
            trading_event: trading_event.unwrap_or_else(|| TradingEvent::default() as u16),
            is_trading: TriState::from(is_trading) as u8 as c_char,
            is_quoting: TriState::from(is_quoting) as u8 as c_char,
            is_short_sell_restricted: TriState::from(is_short_sell_restricted) as u8 as c_char,
            _reserved: Default::default(),
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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

    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[getter]
    #[pyo3(name = "pretty_ts_recv")]
    fn py_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_recv)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[getter]
    #[pyo3(name = "is_trading")]
    fn py_is_trading(&self) -> Option<bool> {
        self.is_trading()
    }

    #[getter]
    #[pyo3(name = "is_quoting")]
    fn py_is_quoting(&self) -> Option<bool> {
        self.is_quoting()
    }

    #[getter]
    #[pyo3(name = "is_short_sell_restricted")]
    fn py_is_short_sell_restricted(&self) -> Option<bool> {
        self.is_short_sell_restricted()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<StatMsg>())
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
    }
}

#[pymethods]
impl InstrumentDefMsg {
    #[new]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        ts_recv: u64,
        min_price_increment: i64,
        display_factor: i64,
        min_lot_size_round_lot: i32,
        raw_symbol: &str,
        group: &str,
        exchange: &str,
        instrument_class: c_char,
        match_algorithm: c_char,
        md_security_trading_status: u8,
        security_update_action: SecurityUpdateAction,
        expiration: Option<u64>,
        activation: Option<u64>,
        high_limit_price: Option<i64>,
        low_limit_price: Option<i64>,
        max_price_variation: Option<i64>,
        trading_reference_price: Option<i64>,
        unit_of_measure_qty: Option<i64>,
        min_price_increment_amount: Option<i64>,
        price_ratio: Option<i64>,
        inst_attrib_value: Option<i32>,
        underlying_id: Option<u32>,
        raw_instrument_id: Option<u32>,
        market_depth_implied: Option<i32>,
        market_depth: Option<i32>,
        market_segment_id: Option<u32>,
        max_trade_vol: Option<u32>,
        min_lot_size: Option<i32>,
        min_lot_size_block: Option<i32>,
        min_trade_vol: Option<u32>,
        contract_multiplier: Option<i32>,
        decay_quantity: Option<i32>,
        original_contract_size: Option<i32>,
        trading_reference_date: Option<u16>,
        appl_id: Option<i16>,
        maturity_year: Option<u16>,
        decay_start_date: Option<u16>,
        channel_id: Option<u16>,
        currency: Option<&str>,
        settl_currency: Option<&str>,
        secsubtype: Option<&str>,
        asset: Option<&str>,
        cfi: Option<&str>,
        security_type: Option<&str>,
        unit_of_measure: Option<&str>,
        underlying: Option<&str>,
        strike_price_currency: Option<&str>,
        strike_price: Option<i64>,
        main_fraction: Option<u8>,
        price_display_format: Option<u8>,
        settl_price_type: Option<u8>,
        sub_fraction: Option<u8>,
        underlying_product: Option<u8>,
        maturity_month: Option<u8>,
        maturity_day: Option<u8>,
        maturity_week: Option<u8>,
        user_defined_instrument: Option<UserDefinedInstrument>,
        contract_multiplier_unit: Option<i8>,
        flow_schedule_type: Option<i8>,
        tick_rule: Option<u8>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(
                rtype::INSTRUMENT_DEF,
                publisher_id,
                instrument_id,
                ts_event,
            ),
            ts_recv,
            min_price_increment,
            display_factor,
            expiration: expiration.unwrap_or(UNDEF_TIMESTAMP),
            activation: activation.unwrap_or(UNDEF_TIMESTAMP),
            high_limit_price: high_limit_price.unwrap_or(UNDEF_PRICE),
            low_limit_price: low_limit_price.unwrap_or(UNDEF_PRICE),
            max_price_variation: max_price_variation.unwrap_or(UNDEF_PRICE),
            trading_reference_price: trading_reference_price.unwrap_or(UNDEF_PRICE),
            unit_of_measure_qty: unit_of_measure_qty.unwrap_or(i64::MAX),
            min_price_increment_amount: min_price_increment_amount.unwrap_or(UNDEF_PRICE),
            price_ratio: price_ratio.unwrap_or(UNDEF_PRICE),
            inst_attrib_value: inst_attrib_value.unwrap_or(i32::MAX),
            underlying_id: underlying_id.unwrap_or_default(),
            raw_instrument_id: raw_instrument_id.unwrap_or(instrument_id),
            market_depth_implied: market_depth_implied.unwrap_or(i32::MAX),
            market_depth: market_depth.unwrap_or(i32::MAX),
            market_segment_id: market_segment_id.unwrap_or(u32::MAX),
            max_trade_vol: max_trade_vol.unwrap_or(u32::MAX),
            min_lot_size: min_lot_size.unwrap_or(i32::MAX),
            min_lot_size_block: min_lot_size_block.unwrap_or(i32::MAX),
            min_lot_size_round_lot,
            min_trade_vol: min_trade_vol.unwrap_or(u32::MAX),
            contract_multiplier: contract_multiplier.unwrap_or(i32::MAX),
            decay_quantity: decay_quantity.unwrap_or(i32::MAX),
            original_contract_size: original_contract_size.unwrap_or(i32::MAX),
            trading_reference_date: trading_reference_date.unwrap_or(u16::MAX),
            appl_id: appl_id.unwrap_or(i16::MAX),
            maturity_year: maturity_year.unwrap_or(u16::MAX),
            decay_start_date: decay_start_date.unwrap_or(u16::MAX),
            channel_id: channel_id.unwrap_or(u16::MAX),
            currency: str_to_c_chars(currency.unwrap_or_default()).map_err(to_val_err)?,
            settl_currency: str_to_c_chars(settl_currency.unwrap_or_default())
                .map_err(to_val_err)?,
            secsubtype: str_to_c_chars(secsubtype.unwrap_or_default()).map_err(to_val_err)?,
            raw_symbol: str_to_c_chars(raw_symbol).map_err(to_val_err)?,
            group: str_to_c_chars(group).map_err(to_val_err)?,
            exchange: str_to_c_chars(exchange).map_err(to_val_err)?,
            asset: str_to_c_chars(asset.unwrap_or_default()).map_err(to_val_err)?,
            cfi: str_to_c_chars(cfi.unwrap_or_default()).map_err(to_val_err)?,
            security_type: str_to_c_chars(security_type.unwrap_or_default()).map_err(to_val_err)?,
            unit_of_measure: str_to_c_chars(unit_of_measure.unwrap_or_default())
                .map_err(to_val_err)?,
            underlying: str_to_c_chars(underlying.unwrap_or_default()).map_err(to_val_err)?,
            strike_price_currency: str_to_c_chars(strike_price_currency.unwrap_or_default())
                .map_err(to_val_err)?,
            instrument_class,
            strike_price: strike_price.unwrap_or(UNDEF_PRICE),
            match_algorithm,
            md_security_trading_status,
            main_fraction: main_fraction.unwrap_or(u8::MAX),
            price_display_format: price_display_format.unwrap_or(u8::MAX),
            settl_price_type: settl_price_type.unwrap_or(u8::MAX),
            sub_fraction: sub_fraction.unwrap_or(u8::MAX),
            underlying_product: underlying_product.unwrap_or(u8::MAX),
            security_update_action: security_update_action as c_char,
            maturity_month: maturity_month.unwrap_or(u8::MAX),
            maturity_day: maturity_day.unwrap_or(u8::MAX),
            maturity_week: maturity_week.unwrap_or(u8::MAX),
            user_defined_instrument: user_defined_instrument.unwrap_or_default(),
            contract_multiplier_unit: contract_multiplier_unit.unwrap_or(i8::MAX),
            flow_schedule_type: flow_schedule_type.unwrap_or(i8::MAX),
            tick_rule: tick_rule.unwrap_or(u8::MAX),
            _reserved: Default::default(),
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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

    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_min_price_increment")]
    fn py_pretty_min_price_increment(&self) -> f64 {
        self.min_price_increment as f64 / FIXED_PRICE_SCALE as f64
    }

    #[getter]
    #[pyo3(name = "pretty_high_limit_price")]
    fn py_pretty_high_limit_price(&self) -> f64 {
        match self.high_limit_price {
            UNDEF_PRICE => f64::NAN,
            _ => self.high_limit_price as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_low_limit_price")]
    fn py_pretty_low_limit_price(&self) -> f64 {
        match self.low_limit_price {
            UNDEF_PRICE => f64::NAN,
            _ => self.low_limit_price as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_max_price_variation")]
    fn py_pretty_max_price_variation(&self) -> f64 {
        match self.max_price_variation {
            UNDEF_PRICE => f64::NAN,
            _ => self.max_price_variation as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_trading_reference_price")]
    fn py_pretty_trading_reference_price(&self) -> f64 {
        match self.trading_reference_price {
            UNDEF_PRICE => f64::NAN,
            _ => self.trading_reference_price as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_min_price_increment_amount")]
    fn py_pretty_min_price_increment_amount(&self) -> f64 {
        match self.min_price_increment_amount {
            UNDEF_PRICE => f64::NAN,
            _ => self.min_price_increment_amount as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_price_ratio")]
    fn py_pretty_price_ratio(&self) -> f64 {
        match self.price_ratio {
            UNDEF_PRICE => f64::NAN,
            _ => self.price_ratio as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_strike_price")]
    fn py_pretty_strike_price(&self) -> f64 {
        match self.strike_price {
            UNDEF_PRICE => f64::NAN,
            _ => self.strike_price as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[getter]
    #[pyo3(name = "pretty_ts_recv")]
    fn py_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_recv)
    }

    #[getter]
    #[pyo3(name = "pretty_activation")]
    fn py_pretty_activation(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.expiration)
    }

    #[getter]
    #[pyo3(name = "pretty_expiration")]
    fn py_pretty_expiration(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.expiration)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<InstrumentDefMsg>())
    }

    #[getter]
    #[pyo3(name = "currency")]
    fn py_currency(&self) -> PyResult<&str> {
        self.currency().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "settl_currency")]
    fn py_settl_currency(&self) -> PyResult<&str> {
        self.settl_currency().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "secsubtype")]
    fn py_secsubtype(&self) -> PyResult<&str> {
        self.secsubtype().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "raw_symbol")]
    fn py_raw_symbol(&self) -> PyResult<&str> {
        self.raw_symbol().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "group")]
    fn py_group(&self) -> PyResult<&str> {
        self.group().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "exchange")]
    fn py_exchange(&self) -> PyResult<&str> {
        self.exchange().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "asset")]
    fn py_asset(&self) -> PyResult<&str> {
        self.asset().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "cfi")]
    fn py_cfi(&self) -> PyResult<&str> {
        self.cfi().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "security_type")]
    fn py_security_type(&self) -> PyResult<&str> {
        self.security_type().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "unit_of_measure")]
    fn py_unit_of_measure(&self) -> PyResult<&str> {
        self.unit_of_measure().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "underlying")]
    fn py_underlying(&self) -> PyResult<&str> {
        self.underlying().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "strike_price_currency")]
    fn py_strike_price_currency(&self) -> PyResult<&str> {
        self.strike_price_currency().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "instrument_class")]
    fn py_instrument_class(&self) -> char {
        self.instrument_class as u8 as char
    }

    #[getter]
    #[pyo3(name = "match_algorithm")]
    fn py_match_algorithm(&self) -> char {
        self.match_algorithm as u8 as char
    }

    #[getter]
    #[pyo3(name = "security_update_action")]
    fn py_security_update_action(&self) -> char {
        self.security_update_action as u8 as char
    }

    #[getter]
    #[pyo3(name = "user_defined_instrument")]
    fn py_user_defined_instrument(&self) -> char {
        self.user_defined_instrument as u8 as char
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
    }
}

#[pymethods]
impl InstrumentDefMsgV1 {
    #[new]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        ts_recv: u64,
        min_price_increment: i64,
        display_factor: i64,
        min_lot_size_round_lot: i32,
        raw_symbol: &str,
        group: &str,
        exchange: &str,
        instrument_class: c_char,
        match_algorithm: c_char,
        md_security_trading_status: u8,
        security_update_action: SecurityUpdateAction,
        expiration: Option<u64>,
        activation: Option<u64>,
        high_limit_price: Option<i64>,
        low_limit_price: Option<i64>,
        max_price_variation: Option<i64>,
        trading_reference_price: Option<i64>,
        unit_of_measure_qty: Option<i64>,
        min_price_increment_amount: Option<i64>,
        price_ratio: Option<i64>,
        inst_attrib_value: Option<i32>,
        underlying_id: Option<u32>,
        raw_instrument_id: Option<u32>,
        market_depth_implied: Option<i32>,
        market_depth: Option<i32>,
        market_segment_id: Option<u32>,
        max_trade_vol: Option<u32>,
        min_lot_size: Option<i32>,
        min_lot_size_block: Option<i32>,
        min_trade_vol: Option<u32>,
        contract_multiplier: Option<i32>,
        decay_quantity: Option<i32>,
        original_contract_size: Option<i32>,
        trading_reference_date: Option<u16>,
        appl_id: Option<i16>,
        maturity_year: Option<u16>,
        decay_start_date: Option<u16>,
        channel_id: Option<u16>,
        currency: Option<&str>,
        settl_currency: Option<&str>,
        secsubtype: Option<&str>,
        asset: Option<&str>,
        cfi: Option<&str>,
        security_type: Option<&str>,
        unit_of_measure: Option<&str>,
        underlying: Option<&str>,
        strike_price_currency: Option<&str>,
        strike_price: Option<i64>,
        main_fraction: Option<u8>,
        price_display_format: Option<u8>,
        settl_price_type: Option<u8>,
        sub_fraction: Option<u8>,
        underlying_product: Option<u8>,
        maturity_month: Option<u8>,
        maturity_day: Option<u8>,
        maturity_week: Option<u8>,
        user_defined_instrument: Option<UserDefinedInstrument>,
        contract_multiplier_unit: Option<i8>,
        flow_schedule_type: Option<i8>,
        tick_rule: Option<u8>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(
                rtype::INSTRUMENT_DEF,
                publisher_id,
                instrument_id,
                ts_event,
            ),
            ts_recv,
            min_price_increment,
            display_factor,
            expiration: expiration.unwrap_or(UNDEF_TIMESTAMP),
            activation: activation.unwrap_or(UNDEF_TIMESTAMP),
            high_limit_price: high_limit_price.unwrap_or(UNDEF_PRICE),
            low_limit_price: low_limit_price.unwrap_or(UNDEF_PRICE),
            max_price_variation: max_price_variation.unwrap_or(UNDEF_PRICE),
            trading_reference_price: trading_reference_price.unwrap_or(UNDEF_PRICE),
            unit_of_measure_qty: unit_of_measure_qty.unwrap_or(i64::MAX),
            min_price_increment_amount: min_price_increment_amount.unwrap_or(UNDEF_PRICE),
            price_ratio: price_ratio.unwrap_or(UNDEF_PRICE),
            inst_attrib_value: inst_attrib_value.unwrap_or(i32::MAX),
            underlying_id: underlying_id.unwrap_or_default(),
            raw_instrument_id: raw_instrument_id.unwrap_or(instrument_id),
            market_depth_implied: market_depth_implied.unwrap_or(i32::MAX),
            market_depth: market_depth.unwrap_or(i32::MAX),
            market_segment_id: market_segment_id.unwrap_or(u32::MAX),
            max_trade_vol: max_trade_vol.unwrap_or(u32::MAX),
            min_lot_size: min_lot_size.unwrap_or(i32::MAX),
            min_lot_size_block: min_lot_size_block.unwrap_or(i32::MAX),
            min_lot_size_round_lot,
            min_trade_vol: min_trade_vol.unwrap_or(u32::MAX),
            contract_multiplier: contract_multiplier.unwrap_or(i32::MAX),
            decay_quantity: decay_quantity.unwrap_or(i32::MAX),
            original_contract_size: original_contract_size.unwrap_or(i32::MAX),
            trading_reference_date: trading_reference_date.unwrap_or(u16::MAX),
            appl_id: appl_id.unwrap_or(i16::MAX),
            maturity_year: maturity_year.unwrap_or(u16::MAX),
            decay_start_date: decay_start_date.unwrap_or(u16::MAX),
            channel_id: channel_id.unwrap_or(u16::MAX),
            currency: str_to_c_chars(currency.unwrap_or_default()).map_err(to_val_err)?,
            settl_currency: str_to_c_chars(settl_currency.unwrap_or_default())
                .map_err(to_val_err)?,
            secsubtype: str_to_c_chars(secsubtype.unwrap_or_default()).map_err(to_val_err)?,
            raw_symbol: str_to_c_chars(raw_symbol).map_err(to_val_err)?,
            group: str_to_c_chars(group).map_err(to_val_err)?,
            exchange: str_to_c_chars(exchange).map_err(to_val_err)?,
            asset: str_to_c_chars(asset.unwrap_or_default()).map_err(to_val_err)?,
            cfi: str_to_c_chars(cfi.unwrap_or_default()).map_err(to_val_err)?,
            security_type: str_to_c_chars(security_type.unwrap_or_default()).map_err(to_val_err)?,
            unit_of_measure: str_to_c_chars(unit_of_measure.unwrap_or_default())
                .map_err(to_val_err)?,
            underlying: str_to_c_chars(underlying.unwrap_or_default()).map_err(to_val_err)?,
            strike_price_currency: str_to_c_chars(strike_price_currency.unwrap_or_default())
                .map_err(to_val_err)?,
            instrument_class,
            strike_price: strike_price.unwrap_or(UNDEF_PRICE),
            match_algorithm,
            md_security_trading_status,
            main_fraction: main_fraction.unwrap_or(u8::MAX),
            price_display_format: price_display_format.unwrap_or(u8::MAX),
            settl_price_type: settl_price_type.unwrap_or(u8::MAX),
            sub_fraction: sub_fraction.unwrap_or(u8::MAX),
            underlying_product: underlying_product.unwrap_or(u8::MAX),
            security_update_action,
            maturity_month: maturity_month.unwrap_or(u8::MAX),
            maturity_day: maturity_day.unwrap_or(u8::MAX),
            maturity_week: maturity_week.unwrap_or(u8::MAX),
            user_defined_instrument: user_defined_instrument.unwrap_or_default(),
            contract_multiplier_unit: contract_multiplier_unit.unwrap_or(i8::MAX),
            flow_schedule_type: flow_schedule_type.unwrap_or(i8::MAX),
            tick_rule: tick_rule.unwrap_or(u8::MAX),
            _reserved2: Default::default(),
            _reserved3: Default::default(),
            _reserved4: Default::default(),
            _reserved5: Default::default(),
            _dummy: Default::default(),
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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

    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_min_price_increment")]
    fn py_pretty_min_price_increment(&self) -> f64 {
        self.min_price_increment as f64 / FIXED_PRICE_SCALE as f64
    }

    #[getter]
    #[pyo3(name = "pretty_high_limit_price")]
    fn py_pretty_high_limit_price(&self) -> f64 {
        match self.high_limit_price {
            UNDEF_PRICE => f64::NAN,
            _ => self.high_limit_price as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_low_limit_price")]
    fn py_pretty_low_limit_price(&self) -> f64 {
        match self.low_limit_price {
            UNDEF_PRICE => f64::NAN,
            _ => self.low_limit_price as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_max_price_variation")]
    fn py_pretty_max_price_variation(&self) -> f64 {
        match self.max_price_variation {
            UNDEF_PRICE => f64::NAN,
            _ => self.max_price_variation as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_trading_reference_price")]
    fn py_pretty_trading_reference_price(&self) -> f64 {
        match self.trading_reference_price {
            UNDEF_PRICE => f64::NAN,
            _ => self.trading_reference_price as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_min_price_increment_amount")]
    fn py_pretty_min_price_increment_amount(&self) -> f64 {
        match self.min_price_increment_amount {
            UNDEF_PRICE => f64::NAN,
            _ => self.min_price_increment_amount as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_price_ratio")]
    fn py_pretty_price_ratio(&self) -> f64 {
        match self.price_ratio {
            UNDEF_PRICE => f64::NAN,
            _ => self.price_ratio as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_strike_price")]
    fn py_pretty_strike_price(&self) -> f64 {
        match self.strike_price {
            UNDEF_PRICE => f64::NAN,
            _ => self.strike_price as f64 / FIXED_PRICE_SCALE as f64,
        }
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[getter]
    #[pyo3(name = "pretty_ts_recv")]
    fn py_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_recv)
    }

    #[getter]
    #[pyo3(name = "pretty_activation")]
    fn py_pretty_activation(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.expiration)
    }

    #[getter]
    #[pyo3(name = "pretty_expiration")]
    fn py_pretty_expiration(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.expiration)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<InstrumentDefMsgV1>())
    }

    #[getter]
    #[pyo3(name = "currency")]
    fn py_currency(&self) -> PyResult<&str> {
        self.currency().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "settl_currency")]
    fn py_settl_currency(&self) -> PyResult<&str> {
        self.settl_currency().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "secsubtype")]
    fn py_secsubtype(&self) -> PyResult<&str> {
        self.secsubtype().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "raw_symbol")]
    fn py_raw_symbol(&self) -> PyResult<&str> {
        self.raw_symbol().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "group")]
    fn py_group(&self) -> PyResult<&str> {
        self.group().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "exchange")]
    fn py_exchange(&self) -> PyResult<&str> {
        self.exchange().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "asset")]
    fn py_asset(&self) -> PyResult<&str> {
        self.asset().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "cfi")]
    fn py_cfi(&self) -> PyResult<&str> {
        self.cfi().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "security_type")]
    fn py_security_type(&self) -> PyResult<&str> {
        self.security_type().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "unit_of_measure")]
    fn py_unit_of_measure(&self) -> PyResult<&str> {
        self.unit_of_measure().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "underlying")]
    fn py_underlying(&self) -> PyResult<&str> {
        self.underlying().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "strike_price_currency")]
    fn py_strike_price_currency(&self) -> PyResult<&str> {
        self.strike_price_currency().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "instrument_class")]
    fn py_instrument_class(&self) -> char {
        self.instrument_class as u8 as char
    }

    #[getter]
    #[pyo3(name = "match_algorithm")]
    fn py_match_algorithm(&self) -> char {
        self.match_algorithm as u8 as char
    }

    #[getter]
    #[pyo3(name = "security_update_action")]
    fn py_security_update_action(&self) -> char {
        self.security_update_action as u8 as char
    }

    #[getter]
    #[pyo3(name = "user_defined_instrument")]
    fn py_user_defined_instrument(&self) -> char {
        self.user_defined_instrument as u8 as char
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
    }
}

#[pymethods]
impl ImbalanceMsg {
    #[new]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        ts_recv: u64,
        ref_price: i64,
        cont_book_clr_price: i64,
        auct_interest_clr_price: i64,
        paired_qty: u32,
        total_imbalance_qty: u32,
        auction_type: c_char,
        side: c_char,
        significant_imbalance: c_char,
    ) -> Self {
        Self {
            hd: RecordHeader::new::<Self>(rtype::IMBALANCE, publisher_id, instrument_id, ts_event),
            ts_recv,
            ref_price,
            auction_time: 0,
            cont_book_clr_price,
            auct_interest_clr_price,
            ssr_filling_price: UNDEF_PRICE,
            ind_match_price: UNDEF_PRICE,
            upper_collar: UNDEF_PRICE,
            lower_collar: UNDEF_PRICE,
            paired_qty,
            total_imbalance_qty,
            market_imbalance_qty: UNDEF_ORDER_SIZE,
            unpaired_qty: UNDEF_ORDER_SIZE,
            auction_type,
            side,
            auction_status: 0,
            freeze_status: 0,
            num_extensions: 0,
            unpaired_side: 0,
            significant_imbalance,
            _reserved: [0],
        }
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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

    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_auct_interest_clr_price")]
    fn py_pretty_auct_interest_clr_price(&self) -> f64 {
        self.auct_interest_clr_price as f64 / FIXED_PRICE_SCALE as f64
    }

    #[getter]
    #[pyo3(name = "pretty_cont_book_clear_price")]
    fn py_pretty_cont_book_clear_price(&self) -> f64 {
        self.cont_book_clr_price as f64 / FIXED_PRICE_SCALE as f64
    }

    #[getter]
    #[pyo3(name = "pretty_ref_price")]
    fn py_pretty_ref_price(&self) -> f64 {
        self.ref_price as f64 / FIXED_PRICE_SCALE as f64
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[getter]
    #[pyo3(name = "pretty_ts_recv")]
    fn py_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_recv)
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<ImbalanceMsg>())
    }

    #[getter]
    #[pyo3(name = "auction_type")]
    fn py_auction_type(&self) -> char {
        self.auction_type as u8 as char
    }

    #[getter]
    #[pyo3(name = "side")]
    fn py_side(&self) -> char {
        self.side as u8 as char
    }

    #[getter]
    #[pyo3(name = "unpaired_side")]
    fn py_unpaired_side(&self) -> char {
        self.unpaired_side as u8 as char
    }

    #[getter]
    #[pyo3(name = "significant_imbalance")]
    fn py_significant_imbalance(&self) -> char {
        self.significant_imbalance as u8 as char
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
    }
}

#[pymethods]
impl StatMsg {
    #[new]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        ts_recv: u64,
        ts_ref: u64,
        price: i64,
        quantity: i32,
        sequence: u32,
        ts_in_delta: i32,
        stat_type: u16,
        channel_id: u16,
        update_action: Option<u8>,
        stat_flags: Option<u8>,
    ) -> Self {
        Self {
            hd: RecordHeader::new::<Self>(rtype::STATISTICS, publisher_id, instrument_id, ts_event),
            ts_recv,
            ts_ref,
            price,
            quantity,
            sequence,
            ts_in_delta,
            stat_type,
            channel_id,
            update_action: update_action.unwrap_or(StatUpdateAction::New as u8),
            stat_flags: stat_flags.unwrap_or_default(),
            _reserved: Default::default(),
        }
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_price")]
    fn py_pretty_price(&self) -> f64 {
        self.price as f64 / FIXED_PRICE_SCALE as f64
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[getter]
    #[pyo3(name = "pretty_ts_recv")]
    fn py_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_recv)
    }

    #[getter]
    #[pyo3(name = "pretty_ts_ref")]
    fn py_pretty_ts_ref(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_ref)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<StatMsg>())
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
    }
}

#[pymethods]
impl ErrorMsg {
    #[new]
    fn py_new(ts_event: u64, err: &str, is_last: Option<bool>) -> PyResult<Self> {
        Ok(ErrorMsg::new(ts_event, err, is_last.unwrap_or(true)))
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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

    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<ErrorMsg>())
    }

    #[getter]
    #[pyo3(name = "err")]
    fn py_err(&self) -> PyResult<&str> {
        self.err().map_err(to_val_err)
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
    }
}

#[pymethods]
impl ErrorMsgV1 {
    #[new]
    fn py_new(ts_event: u64, err: &str) -> PyResult<Self> {
        Ok(ErrorMsgV1::new(ts_event, err))
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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

    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<ErrorMsg>())
    }

    #[getter]
    #[pyo3(name = "err")]
    fn py_err(&self) -> PyResult<&str> {
        self.err().map_err(to_val_err)
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
    }
}

#[pymethods]
impl SymbolMappingMsg {
    #[new]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        stype_in: SType,
        stype_in_symbol: &str,
        stype_out: SType,
        stype_out_symbol: &str,
        start_ts: u64,
        end_ts: u64,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(
                rtype::SYMBOL_MAPPING,
                publisher_id,
                instrument_id,
                ts_event,
            ),
            stype_in: stype_in as u8,
            stype_in_symbol: str_to_c_chars(stype_in_symbol).map_err(to_val_err)?,
            stype_out: stype_out as u8,
            stype_out_symbol: str_to_c_chars(stype_out_symbol).map_err(to_val_err)?,
            start_ts,
            end_ts,
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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

    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[getter]
    #[pyo3(name = "pretty_end_ts")]
    fn py_pretty_end_ts(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.end_ts)
    }

    #[getter]
    #[pyo3(name = "pretty_start_ts")]
    fn py_pretty_start_ts(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.start_ts)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<SymbolMappingMsg>())
    }

    #[getter]
    #[pyo3(name = "stype_in")]
    fn py_stype_in(&self) -> PyResult<SType> {
        self.stype_in().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "stype_in_symbol")]
    fn py_stype_in_symbol(&self) -> PyResult<&str> {
        self.stype_in_symbol().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "stype_out")]
    fn py_stype_out(&self) -> PyResult<SType> {
        self.stype_out().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "stype_out_symbol")]
    fn py_stype_out_symbol(&self) -> PyResult<&str> {
        self.stype_out_symbol().map_err(to_val_err)
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
    }
}

#[pymethods]
impl SymbolMappingMsgV1 {
    #[new]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        stype_in_symbol: &str,
        stype_out_symbol: &str,
        start_ts: u64,
        end_ts: u64,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(
                rtype::SYMBOL_MAPPING,
                publisher_id,
                instrument_id,
                ts_event,
            ),
            stype_in_symbol: str_to_c_chars(stype_in_symbol).map_err(to_val_err)?,
            stype_out_symbol: str_to_c_chars(stype_out_symbol).map_err(to_val_err)?,
            start_ts,
            end_ts,
            _dummy: Default::default(),
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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

    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[getter]
    #[pyo3(name = "pretty_end_ts")]
    fn py_pretty_end_ts(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.end_ts)
    }

    #[getter]
    #[pyo3(name = "pretty_start_ts")]
    fn py_pretty_start_ts(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.start_ts)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<SymbolMappingMsgV1>())
    }

    #[getter]
    #[pyo3(name = "stype_in_symbol")]
    fn py_stype_in_symbol(&self) -> PyResult<&str> {
        self.stype_in_symbol().map_err(to_val_err)
    }

    #[getter]
    #[pyo3(name = "stype_out_symbol")]
    fn py_stype_out_symbol(&self) -> PyResult<&str> {
        self.stype_out_symbol().map_err(to_val_err)
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
    }
}

#[pymethods]
impl SystemMsg {
    #[new]
    fn py_new(ts_event: u64, msg: &str) -> PyResult<Self> {
        SystemMsg::new(ts_event, msg).map_err(to_val_err)
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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

    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<SystemMsg>())
    }

    #[getter]
    #[pyo3(name = "msg")]
    fn py_msg(&self) -> PyResult<&str> {
        self.msg().map_err(to_val_err)
    }

    #[pyo3(name = "is_heartbeat")]
    fn py_is_heartbeat(&self) -> bool {
        self.is_heartbeat()
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
    }
}

#[pymethods]
impl SystemMsgV1 {
    #[new]
    fn py_new(ts_event: u64, msg: &str) -> PyResult<Self> {
        Self::new(ts_event, msg).map_err(to_val_err)
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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

    #[getter]
    fn rtype(&self) -> u8 {
        self.hd.rtype
    }

    #[getter]
    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    #[getter]
    fn instrument_id(&self) -> u32 {
        self.hd.instrument_id
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    #[getter]
    #[pyo3(name = "pretty_ts_event")]
    fn py_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        get_utc_nanosecond_timestamp(py, self.ts_event())
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<SystemMsg>())
    }

    #[getter]
    #[pyo3(name = "msg")]
    fn py_msg(&self) -> PyResult<&str> {
        self.msg().map_err(to_val_err)
    }

    #[pyo3(name = "is_heartbeat")]
    fn py_is_heartbeat(&self) -> bool {
        self.is_heartbeat()
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Self::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Self::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Self::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Self::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Self::ordered_fields("")
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
        let mut res = Vec::new();
        let price_fields = BidAskPair::price_fields("");
        for level in 0..N {
            let mut fields = price_fields.clone();
            for field in fields.iter_mut() {
                field.push_str(&format!("_{level:02}"));
            }
            res.extend(fields);
        }
        res
    }

    fn ordered_fields(_field_name: &str) -> Vec<String> {
        let mut res = Vec::new();
        let ordered_fields = BidAskPair::ordered_fields("");
        for level in 0..N {
            let mut fields = ordered_fields.clone();
            for field in fields.iter_mut() {
                field.push_str(&format!("_{level:02}"));
            }
            res.extend(fields);
        }
        res
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
        let mut res = Vec::new();
        let price_fields = ConsolidatedBidAskPair::price_fields("");
        for level in 0..N {
            let mut fields = price_fields.clone();
            for field in fields.iter_mut() {
                field.push_str(&format!("_{level:02}"));
            }
            res.extend(fields);
        }
        res
    }

    fn ordered_fields(_field_name: &str) -> Vec<String> {
        let mut res = Vec::new();
        let ordered_fields = ConsolidatedBidAskPair::ordered_fields("");
        for level in 0..N {
            let mut fields = ordered_fields.clone();
            for field in fields.iter_mut() {
                field.push_str(&format!("_{level:02}"));
            }
            res.extend(fields);
        }
        res
    }

    fn hidden_fields(_field_name: &str) -> Vec<String> {
        Vec::new()
    }

    fn timestamp_fields(_field_name: &str) -> Vec<String> {
        Vec::new()
    }
}

// `WithTsOut` is converted to a 2-tuple in Python
impl<R: HasRType + IntoPy<Py<PyAny>>> IntoPy<PyObject> for WithTsOut<R> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        let obj = self.rec.into_py(py);
        obj.setattr(py, intern!(py, "ts_out"), self.ts_out).unwrap();
        obj
    }
}

fn get_utc_nanosecond_timestamp(py: Python<'_>, timestamp: u64) -> PyResult<PyObject> {
    if let Ok(pandas) = PyModule::import_bound(py, intern!(py, "pandas")) {
        let kwargs = PyDict::new_bound(py);
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
                .map(|o| o.into_py(py));
        }
    }
    let utc_tz = timezone_utc_bound(py);
    let timestamp_ms = timestamp as f64 / 1_000_000.0;
    PyDateTime::from_timestamp_bound(py, timestamp_ms, Some(&utc_tz)).map(|o| o.into_py(py))
}
