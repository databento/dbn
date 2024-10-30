use std::{ffi::c_char, mem};

use pyo3::{
    intern,
    prelude::*,
    pyclass::CompareOp,
    types::{timezone_utc_bound, PyDateTime, PyDict},
};

use crate::{
    compat::{ErrorMsgV1, InstrumentDefMsgV1, SymbolMappingMsgV1, SystemMsgV1},
    pretty::px_to_f64,
    record::str_to_c_chars,
    rtype, BboMsg, BidAskPair, CbboMsg, Cmbp1Msg, ConsolidatedBidAskPair, ErrorMsg, FlagSet,
    HasRType, ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, Record,
    RecordHeader, SType, SecurityUpdateAction, StatMsg, StatUpdateAction, StatusAction, StatusMsg,
    StatusReason, SymbolMappingMsg, SystemMsg, TradeMsg, TradingEvent, TriState,
    UserDefinedInstrument, WithTsOut, UNDEF_ORDER_SIZE, UNDEF_PRICE, UNDEF_TIMESTAMP,
};

use super::{to_py_err, PyFieldDesc};

fn char_to_c_char(c: char) -> crate::Result<c_char> {
    if c.is_ascii() {
        Ok(c as c_char)
    } else {
        Err(crate::Error::Conversion {
            input: c.to_string(),
            desired_type: "c_char",
        })
    }
}

#[pymethods]
impl MboMsg {
    #[new]
    #[pyo3(signature = (
        publisher_id,
        instrument_id,
        ts_event,
        order_id,
        price,
        size,
        action,
        side,
        ts_recv,
        flags = None,
        channel_id = None,
        ts_in_delta = None,
        sequence = None,
    ))]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        order_id: u64,
        price: i64,
        size: u32,
        action: char,
        side: char,
        ts_recv: u64,
        flags: Option<FlagSet>,
        channel_id: Option<u8>,
        ts_in_delta: Option<i32>,
        sequence: Option<u32>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::MBO, publisher_id, instrument_id, ts_event),
            order_id,
            price,
            size,
            flags: flags.unwrap_or_default(),
            channel_id: channel_id.unwrap_or_default(),
            action: char_to_c_char(action)?,
            side: char_to_c_char(side)?,
            ts_recv,
            ts_in_delta: ts_in_delta.unwrap_or_default(),
            sequence: sequence.unwrap_or_default(),
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[getter]
    fn get_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[getter]
    fn get_action(&self) -> char {
        self.action as u8 as char
    }
    #[setter]
    fn set_action(&mut self, action: char) -> PyResult<()> {
        self.action = char_to_c_char(action)?;
        Ok(())
    }

    #[getter]
    fn get_side(&self) -> char {
        self.side as u8 as char
    }
    #[setter]
    fn set_side(&mut self, side: char) -> PyResult<()> {
        self.side = char_to_c_char(side)?;
        Ok(())
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
    #[pyo3(signature = (
        bid_px = UNDEF_PRICE,
        ask_px = UNDEF_PRICE,
        bid_sz = 0,
        ask_sz = 0,
        bid_ct = 0,
        ask_ct = 0,
    ))]
    fn py_new(
        bid_px: i64,
        ask_px: i64,
        bid_sz: u32,
        ask_sz: u32,
        bid_ct: u32,
        ask_ct: u32,
    ) -> Self {
        Self {
            bid_px,
            ask_px,
            bid_sz,
            ask_sz,
            bid_ct,
            ask_ct,
        }
    }

    #[getter]
    fn get_pretty_ask_px(&self) -> f64 {
        self.ask_px_f64()
    }

    #[getter]
    fn get_pretty_bid_px(&self) -> f64 {
        self.bid_px_f64()
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
impl BboMsg {
    #[new]
    #[pyo3(signature = (
        rtype,
        publisher_id,
        instrument_id,
        ts_event,
        price,
        size,
        side,
        ts_recv,
        flags = None,
        sequence = None,
        levels = None,
    ))]
    fn py_new(
        rtype: u8,
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        price: i64,
        size: u32,
        side: char,
        ts_recv: u64,
        flags: Option<FlagSet>,
        sequence: Option<u32>,
        levels: Option<BidAskPair>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype, publisher_id, instrument_id, ts_event),
            price,
            size,
            side: char_to_c_char(side)?,
            flags: flags.unwrap_or_default(),
            ts_recv,
            sequence: sequence.unwrap_or_default(),
            levels: [levels.unwrap_or_default()],
            _reserved1: Default::default(),
            _reserved2: Default::default(),
            _reserved3: Default::default(),
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[getter]
    fn get_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[getter]
    fn get_side(&self) -> char {
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
impl Cmbp1Msg {
    #[new]
    #[pyo3(signature= (
        rtype,
        publisher_id,
        instrument_id,
        ts_event,
        price,
        size,
        action,
        side,
        ts_recv,
        flags = None,
        ts_in_delta = None,
        levels = None,
    ))]
    fn py_new(
        rtype: u8,
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        price: i64,
        size: u32,
        action: char,
        side: char,
        ts_recv: u64,
        flags: Option<FlagSet>,
        ts_in_delta: Option<i32>,
        levels: Option<ConsolidatedBidAskPair>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype, publisher_id, instrument_id, ts_event),
            price,
            size,
            action: char_to_c_char(action)?,
            side: char_to_c_char(side)?,
            flags: flags.unwrap_or_default(),
            ts_recv,
            ts_in_delta: ts_in_delta.unwrap_or_default(),
            levels: [levels.unwrap_or_default()],
            _reserved1: Default::default(),
            _reserved2: Default::default(),
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
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[getter]
    fn get_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[getter]
    fn get_action(&self) -> char {
        self.action as u8 as char
    }
    #[setter]
    fn set_action(&mut self, action: char) -> PyResult<()> {
        self.action = char_to_c_char(action)?;
        Ok(())
    }

    #[getter]
    fn get_side(&self) -> char {
        self.side as u8 as char
    }
    #[setter]
    fn set_side(&mut self, side: char) -> PyResult<()> {
        self.side = char_to_c_char(side)?;
        Ok(())
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
impl CbboMsg {
    #[new]
    #[pyo3(signature = (
        rtype,
        publisher_id,
        instrument_id,
        ts_event,
        price,
        size,
        side,
        ts_recv,
        flags = None,
        levels = None,
    ))]
    fn py_new(
        rtype: u8,
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        price: i64,
        size: u32,
        side: char,
        ts_recv: u64,
        flags: Option<FlagSet>,
        levels: Option<ConsolidatedBidAskPair>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype, publisher_id, instrument_id, ts_event),
            price,
            size,
            side: char_to_c_char(side)?,
            flags: flags.unwrap_or_default(),
            ts_recv,
            levels: [levels.unwrap_or_default()],
            _reserved1: Default::default(),
            _reserved2: Default::default(),
            _reserved3: Default::default(),
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
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[getter]
    fn get_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[getter]
    fn get_side(&self) -> char {
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
    #[pyo3(signature = (
        bid_px = UNDEF_PRICE,
        ask_px = UNDEF_PRICE,
        bid_sz = 0,
        ask_sz = 0,
        bid_pb = 0,
        ask_pb = 0,
    ))]
    fn py_new(
        bid_px: i64,
        ask_px: i64,
        bid_sz: u32,
        ask_sz: u32,
        bid_pb: u16,
        ask_pb: u16,
    ) -> Self {
        Self {
            bid_px,
            ask_px,
            bid_sz,
            ask_sz,
            bid_pb,
            ask_pb,
            _reserved1: Default::default(),
            _reserved2: Default::default(),
        }
    }

    #[getter]
    fn get_pretty_ask_px(&self) -> f64 {
        self.ask_px_f64()
    }

    #[getter]
    fn get_pretty_bid_px(&self) -> f64 {
        self.bid_px_f64()
    }

    #[getter]
    fn get_pretty_ask_pb(&self) -> Option<String> {
        self.ask_pb().map(|pb| pb.to_string()).ok()
    }

    #[getter]
    fn get_pretty_bid_pb(&self) -> Option<String> {
        self.bid_pb().map(|pb| pb.to_string()).ok()
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
    #[pyo3(signature= (
        publisher_id,
        instrument_id,
        ts_event,
        price,
        size,
        action,
        side,
        depth,
        ts_recv,
        flags = None,
        ts_in_delta = None,
        sequence = None,
    ))]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        price: i64,
        size: u32,
        action: char,
        side: char,
        depth: u8,
        ts_recv: u64,
        flags: Option<FlagSet>,
        ts_in_delta: Option<i32>,
        sequence: Option<u32>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::MBP_0, publisher_id, instrument_id, ts_event),
            price,
            size,
            action: char_to_c_char(action)?,
            side: char_to_c_char(side)?,
            flags: flags.unwrap_or_default(),
            depth,
            ts_recv,
            ts_in_delta: ts_in_delta.unwrap_or_default(),
            sequence: sequence.unwrap_or_default(),
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[getter]
    fn get_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[getter]
    fn get_action(&self) -> char {
        self.action as u8 as char
    }
    #[setter]
    fn set_action(&mut self, action: char) -> PyResult<()> {
        self.action = char_to_c_char(action)?;
        Ok(())
    }

    #[getter]
    fn get_side(&self) -> char {
        self.side as u8 as char
    }
    #[setter]
    fn set_side(&mut self, side: char) -> PyResult<()> {
        self.side = char_to_c_char(side)?;
        Ok(())
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
    #[pyo3(signature = (
        publisher_id,
        instrument_id,
        ts_event,
        price,
        size,
        action,
        side,
        depth,
        ts_recv,
        flags = None,
        ts_in_delta = None,
        sequence = None,
        levels = None,
    ))]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        price: i64,
        size: u32,
        action: char,
        side: char,
        depth: u8,
        ts_recv: u64,
        flags: Option<FlagSet>,
        ts_in_delta: Option<i32>,
        sequence: Option<u32>,
        levels: Option<BidAskPair>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::MBP_1, publisher_id, instrument_id, ts_event),
            price,
            size,
            action: char_to_c_char(action)?,
            side: char_to_c_char(side)?,
            flags: flags.unwrap_or_default(),
            depth,
            ts_recv,
            ts_in_delta: ts_in_delta.unwrap_or_default(),
            sequence: sequence.unwrap_or_default(),
            levels: [levels.unwrap_or_default()],
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[getter]
    fn get_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[getter]
    fn get_action(&self) -> char {
        self.action as u8 as char
    }
    #[setter]
    fn set_action(&mut self, action: char) -> PyResult<()> {
        self.action = char_to_c_char(action)?;
        Ok(())
    }

    #[getter]
    fn get_side(&self) -> char {
        self.side as u8 as char
    }
    #[setter]
    fn set_side(&mut self, side: char) -> PyResult<()> {
        self.side = char_to_c_char(side)?;
        Ok(())
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
    #[pyo3(signature = (
        publisher_id,
        instrument_id,
        ts_event,
        price,
        size,
        action,
        side,
        depth,
        ts_recv,
        flags = None,
        ts_in_delta = None,
        sequence = None,
        levels = None,
    ))]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        price: i64,
        size: u32,
        action: char,
        side: char,
        depth: u8,
        ts_recv: u64,
        flags: Option<FlagSet>,
        ts_in_delta: Option<i32>,
        sequence: Option<u32>,
        levels: Option<Vec<BidAskPair>>,
    ) -> PyResult<Self> {
        let levels = if let Some(level) = levels {
            let mut arr: [BidAskPair; 10] = Default::default();
            if level.len() > 10 {
                return Err(to_py_err("Only 10 levels are allowed"));
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
            action: char_to_c_char(action)?,
            side: char_to_c_char(side)?,
            flags: flags.unwrap_or_default(),
            depth,
            ts_recv,
            ts_in_delta: ts_in_delta.unwrap_or_default(),
            sequence: sequence.unwrap_or_default(),
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[getter]
    fn get_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[getter]
    fn get_action(&self) -> char {
        self.action as u8 as char
    }
    #[setter]
    fn set_action(&mut self, action: char) -> PyResult<()> {
        self.action = char_to_c_char(action)?;
        Ok(())
    }

    #[getter]
    fn get_side(&self) -> char {
        self.side as u8 as char
    }
    #[setter]
    fn set_side(&mut self, side: char) -> PyResult<()> {
        self.side = char_to_c_char(side)?;
        Ok(())
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_open(&self) -> f64 {
        self.open_f64()
    }

    #[getter]
    fn get_pretty_high(&self) -> f64 {
        self.high_f64()
    }

    #[getter]
    fn get_pretty_low(&self) -> f64 {
        self.low_f64()
    }

    #[getter]
    fn get_pretty_close(&self) -> f64 {
        self.close_f64()
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
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
    #[pyo3(signature = (
        publisher_id,
        instrument_id,
        ts_event,
        ts_recv,
        action = None,
        reason = None,
        trading_event = None,
        is_trading = None,
        is_quoting = None,
        is_short_sell_restricted = None,
    ))]
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[getter]
    fn get_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[getter]
    fn get_is_trading(&self) -> Option<bool> {
        self.is_trading()
    }

    #[getter]
    fn get_is_quoting(&self) -> Option<bool> {
        self.is_quoting()
    }

    #[getter]
    fn get_is_short_sell_restricted(&self) -> Option<bool> {
        self.is_short_sell_restricted()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
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
    #[pyo3(signature = (
        publisher_id,
        instrument_id,
        ts_event,
        ts_recv,
        min_price_increment,
        display_factor,
        min_lot_size_round_lot,
        raw_symbol,
        group,
        exchange,
        instrument_class,
        match_algorithm,
        md_security_trading_status,
        security_update_action,
        expiration = UNDEF_TIMESTAMP,
        activation = UNDEF_TIMESTAMP,
        high_limit_price = UNDEF_PRICE,
        low_limit_price = UNDEF_PRICE,
        max_price_variation = UNDEF_PRICE,
        trading_reference_price = UNDEF_PRICE,
        unit_of_measure_qty = UNDEF_PRICE,
        min_price_increment_amount = UNDEF_PRICE,
        price_ratio = UNDEF_PRICE,
        inst_attrib_value = None,
        underlying_id = None,
        raw_instrument_id = None,
        market_depth_implied = None,
        market_depth = None,
        market_segment_id = None,
        max_trade_vol = None,
        min_lot_size = None,
        min_lot_size_block = None,
        min_trade_vol = None,
        contract_multiplier = None,
        decay_quantity = None,
        original_contract_size = None,
        trading_reference_date = None,
        appl_id = None,
        maturity_year = None,
        decay_start_date = None,
        channel_id = None,
        currency = "",
        settl_currency = "",
        secsubtype = "",
        asset = "",
        cfi = "",
        security_type = "",
        unit_of_measure = "",
        underlying = "",
        strike_price_currency = "",
        strike_price = UNDEF_PRICE,
        main_fraction = None,
        price_display_format = None,
        settl_price_type = None,
        sub_fraction = None,
        underlying_product = None,
        maturity_month = None,
        maturity_day = None,
        maturity_week = None,
        user_defined_instrument = None,
        contract_multiplier_unit = None,
        flow_schedule_type = None,
        tick_rule = None,
  ))]
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
        instrument_class: char,
        match_algorithm: char,
        md_security_trading_status: u8,
        security_update_action: SecurityUpdateAction,
        expiration: u64,
        activation: u64,
        high_limit_price: i64,
        low_limit_price: i64,
        max_price_variation: i64,
        trading_reference_price: i64,
        unit_of_measure_qty: i64,
        min_price_increment_amount: i64,
        price_ratio: i64,
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
        currency: &str,
        settl_currency: &str,
        secsubtype: &str,
        asset: &str,
        cfi: &str,
        security_type: &str,
        unit_of_measure: &str,
        underlying: &str,
        strike_price_currency: &str,
        strike_price: i64,
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
            expiration,
            activation,
            high_limit_price,
            low_limit_price,
            max_price_variation,
            trading_reference_price,
            unit_of_measure_qty,
            min_price_increment_amount,
            price_ratio,
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
            currency: str_to_c_chars(currency)?,
            settl_currency: str_to_c_chars(settl_currency)?,
            secsubtype: str_to_c_chars(secsubtype)?,
            raw_symbol: str_to_c_chars(raw_symbol)?,
            group: str_to_c_chars(group)?,
            exchange: str_to_c_chars(exchange)?,
            asset: str_to_c_chars(asset)?,
            cfi: str_to_c_chars(cfi)?,
            security_type: str_to_c_chars(security_type)?,
            unit_of_measure: str_to_c_chars(unit_of_measure)?,
            underlying: str_to_c_chars(underlying)?,
            strike_price_currency: str_to_c_chars(strike_price_currency)?,
            instrument_class: char_to_c_char(instrument_class)?,
            strike_price,
            match_algorithm: char_to_c_char(match_algorithm)?,
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_min_price_increment(&self) -> f64 {
        px_to_f64(self.min_price_increment)
    }

    #[getter]
    fn get_pretty_high_limit_price(&self) -> f64 {
        px_to_f64(self.high_limit_price)
    }

    #[getter]
    fn get_pretty_low_limit_price(&self) -> f64 {
        px_to_f64(self.low_limit_price)
    }

    #[getter]
    fn get_pretty_max_price_variation(&self) -> f64 {
        px_to_f64(self.max_price_variation)
    }

    #[getter]
    fn get_pretty_trading_reference_price(&self) -> f64 {
        px_to_f64(self.trading_reference_price)
    }

    #[getter]
    fn get_pretty_min_price_increment_amount(&self) -> f64 {
        px_to_f64(self.min_price_increment_amount)
    }

    #[getter]
    fn get_pretty_price_ratio(&self) -> f64 {
        px_to_f64(self.price_ratio)
    }

    #[getter]
    fn get_pretty_strike_price(&self) -> f64 {
        self.strike_price_f64()
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[getter]
    fn get_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[getter]
    fn get_pretty_activation(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.activation)
    }

    #[getter]
    fn get_pretty_expiration(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.expiration)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[getter]
    fn get_currency(&self) -> PyResult<&str> {
        Ok(self.currency()?)
    }

    #[getter]
    fn get_settl_currency(&self) -> PyResult<&str> {
        Ok(self.settl_currency()?)
    }

    #[getter]
    fn get_secsubtype(&self) -> PyResult<&str> {
        Ok(self.secsubtype()?)
    }

    #[getter]
    fn get_raw_symbol(&self) -> PyResult<&str> {
        Ok(self.raw_symbol()?)
    }

    #[getter]
    fn get_group(&self) -> PyResult<&str> {
        Ok(self.group()?)
    }

    #[getter]
    fn get_exchange(&self) -> PyResult<&str> {
        Ok(self.exchange()?)
    }

    #[getter]
    fn get_asset(&self) -> PyResult<&str> {
        Ok(self.asset()?)
    }

    #[getter]
    fn get_cfi(&self) -> PyResult<&str> {
        Ok(self.cfi()?)
    }

    #[getter]
    fn get_security_type(&self) -> PyResult<&str> {
        Ok(self.security_type()?)
    }

    #[getter]
    fn get_unit_of_measure(&self) -> PyResult<&str> {
        Ok(self.unit_of_measure()?)
    }

    #[getter]
    fn get_underlying(&self) -> PyResult<&str> {
        Ok(self.underlying()?)
    }

    #[getter]
    fn get_strike_price_currency(&self) -> PyResult<&str> {
        Ok(self.strike_price_currency()?)
    }

    #[getter]
    fn get_instrument_class(&self) -> char {
        self.instrument_class as u8 as char
    }
    #[setter]
    fn set_instrument_class(&mut self, instrument_class: char) -> PyResult<()> {
        self.instrument_class = char_to_c_char(instrument_class)?;
        Ok(())
    }

    #[getter]
    fn get_match_algorithm(&self) -> char {
        self.match_algorithm as u8 as char
    }
    #[setter]
    fn set_match_algorithm(&mut self, match_algorithm: char) -> PyResult<()> {
        self.match_algorithm = char_to_c_char(match_algorithm)?;
        Ok(())
    }

    #[getter]
    fn get_security_update_action(&self) -> char {
        self.security_update_action as u8 as char
    }
    #[setter]
    fn set_security_update_action(&mut self, security_update_action: char) -> PyResult<()> {
        self.security_update_action = char_to_c_char(security_update_action)?;
        Ok(())
    }

    #[getter]
    fn get_user_defined_instrument(&self) -> char {
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
    #[pyo3(signature = (
        publisher_id,
        instrument_id,
        ts_event,
        ts_recv,
        min_price_increment,
        display_factor,
        min_lot_size_round_lot,
        raw_symbol,
        group,
        exchange,
        instrument_class,
        match_algorithm,
        md_security_trading_status,
        security_update_action,
        expiration = UNDEF_TIMESTAMP,
        activation = UNDEF_TIMESTAMP,
        high_limit_price = UNDEF_PRICE,
        low_limit_price = UNDEF_PRICE,
        max_price_variation = UNDEF_PRICE,
        trading_reference_price = UNDEF_PRICE,
        unit_of_measure_qty = UNDEF_PRICE,
        min_price_increment_amount = UNDEF_PRICE,
        price_ratio = UNDEF_PRICE,
        inst_attrib_value = None,
        underlying_id = None,
        raw_instrument_id = None,
        market_depth_implied = None,
        market_depth = None,
        market_segment_id = None,
        max_trade_vol = None,
        min_lot_size = None,
        min_lot_size_block = None,
        min_trade_vol = None,
        contract_multiplier = None,
        decay_quantity = None,
        original_contract_size = None,
        trading_reference_date = None,
        appl_id = None,
        maturity_year = None,
        decay_start_date = None,
        channel_id = None,
        currency = "",
        settl_currency = "",
        secsubtype = "",
        asset = "",
        cfi = "",
        security_type = "",
        unit_of_measure = "",
        underlying = "",
        strike_price_currency = "",
        strike_price = UNDEF_PRICE,
        main_fraction = None,
        price_display_format = None,
        settl_price_type = None,
        sub_fraction = None,
        underlying_product = None,
        maturity_month = None,
        maturity_day = None,
        maturity_week = None,
        user_defined_instrument = None,
        contract_multiplier_unit = None,
        flow_schedule_type = None,
        tick_rule = None,
   ))]
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
        instrument_class: char,
        match_algorithm: char,
        md_security_trading_status: u8,
        security_update_action: SecurityUpdateAction,
        expiration: u64,
        activation: u64,
        high_limit_price: i64,
        low_limit_price: i64,
        max_price_variation: i64,
        trading_reference_price: i64,
        unit_of_measure_qty: i64,
        min_price_increment_amount: i64,
        price_ratio: i64,
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
        currency: &str,
        settl_currency: &str,
        secsubtype: &str,
        asset: &str,
        cfi: &str,
        security_type: &str,
        unit_of_measure: &str,
        underlying: &str,
        strike_price_currency: &str,
        strike_price: i64,
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
            expiration,
            activation,
            high_limit_price,
            low_limit_price,
            max_price_variation,
            trading_reference_price,
            unit_of_measure_qty,
            min_price_increment_amount,
            price_ratio,
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
            currency: str_to_c_chars(currency)?,
            settl_currency: str_to_c_chars(settl_currency)?,
            secsubtype: str_to_c_chars(secsubtype)?,
            raw_symbol: str_to_c_chars(raw_symbol)?,
            group: str_to_c_chars(group)?,
            exchange: str_to_c_chars(exchange)?,
            asset: str_to_c_chars(asset)?,
            cfi: str_to_c_chars(cfi)?,
            security_type: str_to_c_chars(security_type)?,
            unit_of_measure: str_to_c_chars(unit_of_measure)?,
            underlying: str_to_c_chars(underlying)?,
            strike_price_currency: str_to_c_chars(strike_price_currency)?,
            instrument_class: char_to_c_char(instrument_class)?,
            strike_price,
            match_algorithm: char_to_c_char(match_algorithm)?,
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_min_price_increment(&self) -> f64 {
        px_to_f64(self.min_price_increment)
    }

    #[getter]
    fn get_pretty_high_limit_price(&self) -> f64 {
        px_to_f64(self.high_limit_price)
    }

    #[getter]
    fn get_pretty_low_limit_price(&self) -> f64 {
        px_to_f64(self.low_limit_price)
    }

    #[getter]
    fn get_pretty_max_price_variation(&self) -> f64 {
        px_to_f64(self.max_price_variation)
    }

    #[getter]
    fn get_pretty_trading_reference_price(&self) -> f64 {
        px_to_f64(self.trading_reference_price)
    }

    #[getter]
    fn get_pretty_min_price_increment_amount(&self) -> f64 {
        px_to_f64(self.min_price_increment_amount)
    }

    #[getter]
    fn get_pretty_price_ratio(&self) -> f64 {
        px_to_f64(self.price_ratio)
    }

    #[getter]
    fn get_pretty_strike_price(&self) -> f64 {
        px_to_f64(self.strike_price)
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[getter]
    fn get_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[getter]
    fn get_pretty_activation(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.activation)
    }

    #[getter]
    fn get_pretty_expiration(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.expiration)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[getter]
    fn get_currency(&self) -> PyResult<&str> {
        Ok(self.currency()?)
    }

    #[getter]
    fn get_settl_currency(&self) -> PyResult<&str> {
        Ok(self.settl_currency()?)
    }

    #[getter]
    fn get_secsubtype(&self) -> PyResult<&str> {
        Ok(self.secsubtype()?)
    }

    #[getter]
    fn get_raw_symbol(&self) -> PyResult<&str> {
        Ok(self.raw_symbol()?)
    }

    #[getter]
    fn get_group(&self) -> PyResult<&str> {
        Ok(self.group()?)
    }

    #[getter]
    fn get_exchange(&self) -> PyResult<&str> {
        Ok(self.exchange()?)
    }

    #[getter]
    fn get_asset(&self) -> PyResult<&str> {
        Ok(self.asset()?)
    }

    #[getter]
    fn get_cfi(&self) -> PyResult<&str> {
        Ok(self.cfi()?)
    }

    #[getter]
    fn get_security_type(&self) -> PyResult<&str> {
        Ok(self.security_type()?)
    }

    #[getter]
    fn get_unit_of_measure(&self) -> PyResult<&str> {
        Ok(self.unit_of_measure()?)
    }

    #[getter]
    fn get_underlying(&self) -> PyResult<&str> {
        Ok(self.underlying()?)
    }

    #[getter]
    fn get_strike_price_currency(&self) -> PyResult<&str> {
        Ok(self.strike_price_currency()?)
    }

    #[getter]
    fn get_instrument_class(&self) -> char {
        self.instrument_class as u8 as char
    }
    #[setter]
    fn set_instrument_class(&mut self, instrument_class: char) -> PyResult<()> {
        self.instrument_class = char_to_c_char(instrument_class)?;
        Ok(())
    }

    #[getter]
    fn get_match_algorithm(&self) -> char {
        self.match_algorithm as u8 as char
    }
    #[setter]
    fn set_match_algorithm(&mut self, match_algorithm: char) -> PyResult<()> {
        self.match_algorithm = char_to_c_char(match_algorithm)?;
        Ok(())
    }

    #[getter]
    fn get_security_update_action(&self) -> char {
        self.security_update_action as u8 as char
    }

    #[getter]
    fn get_user_defined_instrument(&self) -> char {
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
        auction_type: char,
        side: char,
        significant_imbalance: char,
    ) -> PyResult<Self> {
        Ok(Self {
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
            auction_type: char_to_c_char(auction_type)?,
            side: char_to_c_char(side)?,
            auction_status: 0,
            freeze_status: 0,
            num_extensions: 0,
            unpaired_side: 0,
            significant_imbalance: char_to_c_char(significant_imbalance)?,
            _reserved: [0],
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_auct_interest_clr_price(&self) -> f64 {
        self.auct_interest_clr_price_f64()
    }

    #[getter]
    fn get_pretty_cont_book_clear_price(&self) -> f64 {
        self.cont_book_clr_price_f64()
    }

    #[getter]
    fn get_pretty_ref_price(&self) -> f64 {
        self.ref_price_f64()
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[getter]
    fn get_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[getter]
    fn get_auction_type(&self) -> char {
        self.auction_type as u8 as char
    }
    #[setter]
    fn set_auction_type(&mut self, auction_type: char) -> PyResult<()> {
        self.auction_type = char_to_c_char(auction_type)?;
        Ok(())
    }

    #[getter]
    fn get_side(&self) -> char {
        self.side as u8 as char
    }
    #[setter]
    fn set_side(&mut self, side: char) -> PyResult<()> {
        self.side = char_to_c_char(side)?;
        Ok(())
    }

    #[getter]
    fn get_unpaired_side(&self) -> char {
        self.unpaired_side as u8 as char
    }
    #[setter]
    fn set_unpaired_side(&mut self, unpaired_side: char) -> PyResult<()> {
        self.unpaired_side = char_to_c_char(unpaired_side)?;
        Ok(())
    }

    #[getter]
    fn get_significant_imbalance(&self) -> char {
        self.significant_imbalance as u8 as char
    }
    #[setter]
    fn set_significant_imbalance(&mut self, significant_imbalance: char) -> PyResult<()> {
        self.significant_imbalance = char_to_c_char(significant_imbalance)?;
        Ok(())
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
    #[pyo3(signature= (
        publisher_id,
        instrument_id,
        ts_event,
        ts_recv,
        ts_ref,
        price,
        quantity,
        stat_type,
        sequence = None,
        ts_in_delta = None,
        channel_id = None,
        update_action = None,
        stat_flags = 0,
    ))]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        ts_recv: u64,
        ts_ref: u64,
        price: i64,
        quantity: i32,
        stat_type: u16,
        sequence: Option<u32>,
        ts_in_delta: Option<i32>,
        channel_id: Option<u16>,
        update_action: Option<u8>,
        stat_flags: u8,
    ) -> Self {
        Self {
            hd: RecordHeader::new::<Self>(rtype::STATISTICS, publisher_id, instrument_id, ts_event),
            ts_recv,
            ts_ref,
            price,
            quantity,
            sequence: sequence.unwrap_or_default(),
            ts_in_delta: ts_in_delta.unwrap_or_default(),
            stat_type,
            channel_id: channel_id.unwrap_or_default(),
            update_action: update_action.unwrap_or(StatUpdateAction::New as u8),
            stat_flags,
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[getter]
    fn get_pretty_ts_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[getter]
    fn get_pretty_ts_ref(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_ref)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
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
    #[pyo3(signature = (ts_event, err, is_last = true))]
    fn py_new(ts_event: u64, err: &str, is_last: bool) -> PyResult<Self> {
        Ok(ErrorMsg::new(ts_event, err, is_last))
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[getter]
    fn get_err(&self) -> PyResult<&str> {
        Ok(self.err()?)
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[getter]
    fn get_err(&self) -> PyResult<&str> {
        Ok(self.err()?)
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
            stype_in_symbol: str_to_c_chars(stype_in_symbol)?,
            stype_out: stype_out as u8,
            stype_out_symbol: str_to_c_chars(stype_out_symbol)?,
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[getter]
    fn get_pretty_end_ts(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.end_ts)
    }

    #[getter]
    fn get_pretty_start_ts(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.start_ts)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[getter]
    fn get_stype_in(&self) -> PyResult<SType> {
        Ok(self.stype_in()?)
    }

    #[getter]
    fn get_stype_in_symbol(&self) -> PyResult<&str> {
        Ok(self.stype_in_symbol()?)
    }

    #[getter]
    fn get_stype_out(&self) -> PyResult<SType> {
        Ok(self.stype_out()?)
    }

    #[getter]
    fn get_stype_out_symbol(&self) -> PyResult<&str> {
        Ok(self.stype_out_symbol()?)
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
            stype_in_symbol: str_to_c_chars(stype_in_symbol)?,
            stype_out_symbol: str_to_c_chars(stype_out_symbol)?,
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[getter]
    fn get_pretty_end_ts(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.end_ts)
    }

    #[getter]
    fn get_pretty_start_ts(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.start_ts)
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[getter]
    fn get_stype_in_symbol(&self) -> PyResult<&str> {
        Ok(self.stype_in_symbol()?)
    }

    #[getter]
    fn get_stype_out_symbol(&self) -> PyResult<&str> {
        Ok(self.stype_out_symbol()?)
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
        Ok(SystemMsg::new(ts_event, msg)?)
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[getter]
    fn get_msg(&self) -> PyResult<&str> {
        Ok(self.msg()?)
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
        Ok(Self::new(ts_event, msg)?)
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

    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[getter]
    fn get_pretty_ts_event(&self, py: Python<'_>) -> PyResult<PyObject> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[getter]
    fn get_msg(&self) -> PyResult<&str> {
        Ok(self.msg()?)
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

fn new_py_timestamp_or_datetime(py: Python<'_>, timestamp: u64) -> PyResult<PyObject> {
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
