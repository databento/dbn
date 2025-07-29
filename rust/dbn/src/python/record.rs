use std::{ffi::c_char, mem};

use pyo3::prelude::*;

use crate::{
    record::str_to_c_chars, rtype, v1, v2, Action, BboMsg, BidAskPair, CbboMsg, Cmbp1Msg,
    ConsolidatedBidAskPair, ErrorCode, ErrorMsg, FlagSet, ImbalanceMsg, InstrumentClass,
    InstrumentDefMsg, MatchAlgorithm, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, Record, RecordHeader,
    SType, SecurityUpdateAction, Side, StatMsg, StatType, StatUpdateAction, StatusAction,
    StatusMsg, StatusReason, SymbolMappingMsg, SystemCode, SystemMsg, TradeMsg, TradingEvent,
    TriState, UserDefinedInstrument, UNDEF_ORDER_SIZE, UNDEF_PRICE, UNDEF_TIMESTAMP,
};

use super::{
    conversions::{char_to_c_char, new_py_timestamp_or_datetime},
    to_py_err, PyFieldDesc,
};

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
        channel_id = 0,
        ts_in_delta = 0,
        sequence = 0,
    ))]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        order_id: u64,
        price: i64,
        size: u32,
        action: Action,
        side: Side,
        ts_recv: u64,
        flags: Option<FlagSet>,
        channel_id: u8,
        ts_in_delta: i32,
        sequence: u32,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::MBO, publisher_id, instrument_id, ts_event),
            order_id,
            price,
            size,
            flags: flags.unwrap_or_default(),
            channel_id,
            action: action as u8 as c_char,
            side: side as u8 as c_char,
            ts_recv,
            ts_in_delta,
            sequence,
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_action(&self) -> PyResult<Action> {
        self.action().map_err(to_py_err)
    }
    #[setter]
    fn set_action(&mut self, action: Action) {
        self.action = action as u8 as c_char;
    }

    #[getter]
    fn get_side(&self) -> PyResult<Side> {
        self.side().map_err(to_py_err)
    }
    #[setter]
    fn set_side(&mut self, side: Side) {
        self.side = side as u8 as c_char;
    }

    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
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

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    #[getter]
    fn get_pretty_bid_px(&self) -> f64 {
        self.bid_px_f64()
    }

    #[getter]
    fn get_pretty_ask_px(&self) -> f64 {
        self.ask_px_f64()
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
            _reserved1: Default::default(),
            ask_pb,
            _reserved2: Default::default(),
        }
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    #[getter]
    fn get_pretty_bid_px(&self) -> f64 {
        self.bid_px_f64()
    }

    #[getter]
    fn get_pretty_ask_px(&self) -> f64 {
        self.ask_px_f64()
    }
}

#[pymethods]
impl TradeMsg {
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
        ts_in_delta = 0,
        sequence = 0,
    ))]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        price: i64,
        size: u32,
        action: Action,
        side: Side,
        depth: u8,
        ts_recv: u64,
        flags: Option<FlagSet>,
        ts_in_delta: i32,
        sequence: u32,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::MBP_0, publisher_id, instrument_id, ts_event),
            price,
            size,
            action: action as u8 as c_char,
            side: side as u8 as c_char,
            flags: flags.unwrap_or_default(),
            depth,
            ts_recv,
            ts_in_delta,
            sequence,
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_action(&self) -> PyResult<Action> {
        self.action().map_err(to_py_err)
    }
    #[setter]
    fn set_action(&mut self, action: Action) {
        self.action = action as u8 as c_char;
    }

    #[getter]
    fn get_side(&self) -> PyResult<Side> {
        self.side().map_err(to_py_err)
    }
    #[setter]
    fn set_side(&mut self, side: Side) {
        self.side = side as u8 as c_char;
    }

    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
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
        ts_in_delta = 0,
        sequence = 0,
        levels = None,
    ))]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        price: i64,
        size: u32,
        action: Action,
        side: Side,
        depth: u8,
        ts_recv: u64,
        flags: Option<FlagSet>,
        ts_in_delta: i32,
        sequence: u32,
        levels: Option<BidAskPair>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::MBP_1, publisher_id, instrument_id, ts_event),
            price,
            size,
            action: action as u8 as c_char,
            side: side as u8 as c_char,
            flags: flags.unwrap_or_default(),
            depth,
            ts_recv,
            ts_in_delta,
            sequence,
            levels: [levels.unwrap_or_default()],
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_action(&self) -> PyResult<Action> {
        self.action().map_err(to_py_err)
    }
    #[setter]
    fn set_action(&mut self, action: Action) {
        self.action = action as u8 as c_char;
    }

    #[getter]
    fn get_side(&self) -> PyResult<Side> {
        self.side().map_err(to_py_err)
    }
    #[setter]
    fn set_side(&mut self, side: Side) {
        self.side = side as u8 as c_char;
    }

    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
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
        ts_in_delta = 0,
        sequence = 0,
        levels = None,
    ))]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        price: i64,
        size: u32,
        action: Action,
        side: Side,
        depth: u8,
        ts_recv: u64,
        flags: Option<FlagSet>,
        ts_in_delta: i32,
        sequence: u32,
        levels: Option<[BidAskPair; 10]>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::MBP_10, publisher_id, instrument_id, ts_event),
            price,
            size,
            action: action as u8 as c_char,
            side: side as u8 as c_char,
            flags: flags.unwrap_or_default(),
            depth,
            ts_recv,
            ts_in_delta,
            sequence,
            levels: levels.unwrap_or_default(),
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_action(&self) -> PyResult<Action> {
        self.action().map_err(to_py_err)
    }
    #[setter]
    fn set_action(&mut self, action: Action) {
        self.action = action as u8 as c_char;
    }

    #[getter]
    fn get_side(&self) -> PyResult<Side> {
        self.side().map_err(to_py_err)
    }
    #[setter]
    fn set_side(&mut self, side: Side) {
        self.side = side as u8 as c_char;
    }

    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
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
        sequence = 0,
        levels = None,
    ))]
    fn py_new(
        rtype: u8,
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        price: i64,
        size: u32,
        side: Side,
        ts_recv: u64,
        flags: Option<FlagSet>,
        sequence: u32,
        levels: Option<BidAskPair>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype, publisher_id, instrument_id, ts_event),
            price,
            size,
            _reserved1: Default::default(),
            side: side as u8 as c_char,
            flags: flags.unwrap_or_default(),
            _reserved2: Default::default(),
            ts_recv,
            _reserved3: Default::default(),
            sequence,
            levels: [levels.unwrap_or_default()],
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_side(&self) -> PyResult<Side> {
        self.side().map_err(to_py_err)
    }
    #[setter]
    fn set_side(&mut self, side: Side) {
        self.side = side as u8 as c_char;
    }

    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
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
    #[pyo3(signature = (
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
        ts_in_delta = 0,
        levels = None,
    ))]
    fn py_new(
        rtype: u8,
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        price: i64,
        size: u32,
        action: Action,
        side: Side,
        ts_recv: u64,
        flags: Option<FlagSet>,
        ts_in_delta: i32,
        levels: Option<ConsolidatedBidAskPair>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype, publisher_id, instrument_id, ts_event),
            price,
            size,
            action: action as u8 as c_char,
            side: side as u8 as c_char,
            flags: flags.unwrap_or_default(),
            _reserved1: Default::default(),
            ts_recv,
            ts_in_delta,
            _reserved2: Default::default(),
            levels: [levels.unwrap_or_default()],
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_action(&self) -> PyResult<Action> {
        self.action().map_err(to_py_err)
    }
    #[setter]
    fn set_action(&mut self, action: Action) {
        self.action = action as u8 as c_char;
    }

    #[getter]
    fn get_side(&self) -> PyResult<Side> {
        self.side().map_err(to_py_err)
    }
    #[setter]
    fn set_side(&mut self, side: Side) {
        self.side = side as u8 as c_char;
    }

    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
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
        side: Side,
        ts_recv: u64,
        flags: Option<FlagSet>,
        levels: Option<ConsolidatedBidAskPair>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype, publisher_id, instrument_id, ts_event),
            price,
            size,
            _reserved1: Default::default(),
            side: side as u8 as c_char,
            flags: flags.unwrap_or_default(),
            _reserved2: Default::default(),
            ts_recv,
            _reserved3: Default::default(),
            levels: [levels.unwrap_or_default()],
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_side(&self) -> PyResult<Side> {
        self.side().map_err(to_py_err)
    }
    #[setter]
    fn set_side(&mut self, side: Side) {
        self.side = side as u8 as c_char;
    }

    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
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
    #[pyo3(signature = (
        rtype,
        publisher_id,
        instrument_id,
        ts_event,
        open,
        high,
        low,
        close,
        volume,
    ))]
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
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype, publisher_id, instrument_id, ts_event),
            open,
            high,
            low,
            close,
            volume,
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
        action: Option<StatusAction>,
        reason: Option<StatusReason>,
        trading_event: Option<TradingEvent>,
        is_trading: Option<TriState>,
        is_quoting: Option<TriState>,
        is_short_sell_restricted: Option<TriState>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::STATUS, publisher_id, instrument_id, ts_event),
            ts_recv,
            action: action.unwrap_or_default() as u16,
            reason: reason.unwrap_or_default() as u16,
            trading_event: trading_event.unwrap_or_default() as u16,
            is_trading: is_trading.unwrap_or_default() as u8 as c_char,
            is_quoting: is_quoting.unwrap_or_default() as u8 as c_char,
            is_short_sell_restricted: is_short_sell_restricted.unwrap_or_default() as u8 as c_char,
            _reserved: Default::default(),
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[getter]
    fn get_action(&self) -> PyResult<StatusAction> {
        self.action().map_err(to_py_err)
    }
    #[setter]
    fn set_action(&mut self, action: StatusAction) {
        self.action = action as u16;
    }

    #[getter]
    fn get_reason(&self) -> PyResult<StatusReason> {
        self.reason().map_err(to_py_err)
    }
    #[setter]
    fn set_reason(&mut self, reason: StatusReason) {
        self.reason = reason as u16;
    }

    #[getter]
    fn get_trading_event(&self) -> PyResult<TradingEvent> {
        self.trading_event().map_err(to_py_err)
    }
    #[setter]
    fn set_trading_event(&mut self, trading_event: TradingEvent) {
        self.trading_event = trading_event as u16;
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
        raw_symbol,
        asset,
        security_type,
        instrument_class,
        security_update_action,
        expiration = UNDEF_TIMESTAMP,
        activation = UNDEF_TIMESTAMP,
        high_limit_price = UNDEF_PRICE,
        low_limit_price = UNDEF_PRICE,
        max_price_variation = UNDEF_PRICE,
        unit_of_measure_qty = UNDEF_PRICE,
        min_price_increment_amount = UNDEF_PRICE,
        price_ratio = UNDEF_PRICE,
        strike_price = UNDEF_PRICE,
        raw_instrument_id = 0,
        leg_price = UNDEF_PRICE,
        leg_delta = UNDEF_PRICE,
        inst_attrib_value = 0,
        underlying_id = 0,
        market_depth_implied = None,
        market_depth = None,
        market_segment_id = None,
        max_trade_vol = None,
        min_lot_size = None,
        min_lot_size_block = None,
        min_lot_size_round_lot = None,
        min_trade_vol = None,
        contract_multiplier = None,
        decay_quantity = None,
        original_contract_size = None,
        leg_instrument_id = 0,
        leg_ratio_price_numerator = 0,
        leg_ratio_price_denominator = 0,
        leg_ratio_qty_numerator = 0,
        leg_ratio_qty_denominator = 0,
        leg_underlying_id = 0,
        appl_id = None,
        maturity_year = None,
        decay_start_date = None,
        channel_id = 0,
        leg_count = 0,
        leg_index = 0,
        currency = "",
        settl_currency = "",
        secsubtype = "",
        group = "",
        exchange = "",
        cfi = "",
        unit_of_measure = "",
        underlying = "",
        strike_price_currency = "",
        leg_raw_symbol = "",
        match_algorithm = None,
        main_fraction = None,
        price_display_format = None,
        sub_fraction = None,
        underlying_product = None,
        maturity_month = None,
        maturity_day = None,
        maturity_week = None,
        user_defined_instrument = None,
        contract_multiplier_unit = None,
        flow_schedule_type = None,
        tick_rule = None,
        leg_instrument_class = None,
        leg_side = None,
    ))]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        ts_recv: u64,
        min_price_increment: i64,
        display_factor: i64,
        raw_symbol: &str,
        asset: &str,
        security_type: &str,
        instrument_class: InstrumentClass,
        security_update_action: SecurityUpdateAction,
        expiration: u64,
        activation: u64,
        high_limit_price: i64,
        low_limit_price: i64,
        max_price_variation: i64,
        unit_of_measure_qty: i64,
        min_price_increment_amount: i64,
        price_ratio: i64,
        strike_price: i64,
        raw_instrument_id: u64,
        leg_price: i64,
        leg_delta: i64,
        inst_attrib_value: i32,
        underlying_id: u32,
        market_depth_implied: Option<i32>,
        market_depth: Option<i32>,
        market_segment_id: Option<u32>,
        max_trade_vol: Option<u32>,
        min_lot_size: Option<i32>,
        min_lot_size_block: Option<i32>,
        min_lot_size_round_lot: Option<i32>,
        min_trade_vol: Option<u32>,
        contract_multiplier: Option<i32>,
        decay_quantity: Option<i32>,
        original_contract_size: Option<i32>,
        leg_instrument_id: u32,
        leg_ratio_price_numerator: i32,
        leg_ratio_price_denominator: i32,
        leg_ratio_qty_numerator: i32,
        leg_ratio_qty_denominator: i32,
        leg_underlying_id: u32,
        appl_id: Option<i16>,
        maturity_year: Option<u16>,
        decay_start_date: Option<u16>,
        channel_id: u16,
        leg_count: u16,
        leg_index: u16,
        currency: &str,
        settl_currency: &str,
        secsubtype: &str,
        group: &str,
        exchange: &str,
        cfi: &str,
        unit_of_measure: &str,
        underlying: &str,
        strike_price_currency: &str,
        leg_raw_symbol: &str,
        match_algorithm: Option<MatchAlgorithm>,
        main_fraction: Option<u8>,
        price_display_format: Option<u8>,
        sub_fraction: Option<u8>,
        underlying_product: Option<u8>,
        maturity_month: Option<u8>,
        maturity_day: Option<u8>,
        maturity_week: Option<u8>,
        user_defined_instrument: Option<UserDefinedInstrument>,
        contract_multiplier_unit: Option<i8>,
        flow_schedule_type: Option<i8>,
        tick_rule: Option<u8>,
        leg_instrument_class: Option<InstrumentClass>,
        leg_side: Option<Side>,
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
            unit_of_measure_qty,
            min_price_increment_amount,
            price_ratio,
            strike_price,
            raw_instrument_id,
            leg_price,
            leg_delta,
            inst_attrib_value,
            underlying_id,
            market_depth_implied: market_depth_implied.unwrap_or(i32::MAX),
            market_depth: market_depth.unwrap_or(i32::MAX),
            market_segment_id: market_segment_id.unwrap_or(u32::MAX),
            max_trade_vol: max_trade_vol.unwrap_or(u32::MAX),
            min_lot_size: min_lot_size.unwrap_or(i32::MAX),
            min_lot_size_block: min_lot_size_block.unwrap_or(i32::MAX),
            min_lot_size_round_lot: min_lot_size_round_lot.unwrap_or(i32::MAX),
            min_trade_vol: min_trade_vol.unwrap_or(u32::MAX),
            contract_multiplier: contract_multiplier.unwrap_or(i32::MAX),
            decay_quantity: decay_quantity.unwrap_or(i32::MAX),
            original_contract_size: original_contract_size.unwrap_or(i32::MAX),
            leg_instrument_id,
            leg_ratio_price_numerator,
            leg_ratio_price_denominator,
            leg_ratio_qty_numerator,
            leg_ratio_qty_denominator,
            leg_underlying_id,
            appl_id: appl_id.unwrap_or(i16::MAX),
            maturity_year: maturity_year.unwrap_or(u16::MAX),
            decay_start_date: decay_start_date.unwrap_or(u16::MAX),
            channel_id,
            leg_count,
            leg_index,
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
            leg_raw_symbol: str_to_c_chars(leg_raw_symbol)?,
            instrument_class: instrument_class as u8 as c_char,
            match_algorithm: match_algorithm.unwrap_or_default() as u8 as c_char,
            main_fraction: main_fraction.unwrap_or(u8::MAX),
            price_display_format: price_display_format.unwrap_or(u8::MAX),
            sub_fraction: sub_fraction.unwrap_or(u8::MAX),
            underlying_product: underlying_product.unwrap_or(u8::MAX),
            security_update_action: security_update_action as u8 as c_char,
            maturity_month: maturity_month.unwrap_or(u8::MAX),
            maturity_day: maturity_day.unwrap_or(u8::MAX),
            maturity_week: maturity_week.unwrap_or(u8::MAX),
            user_defined_instrument: user_defined_instrument.unwrap_or_default() as u8 as c_char,
            contract_multiplier_unit: contract_multiplier_unit.unwrap_or(i8::MAX),
            flow_schedule_type: flow_schedule_type.unwrap_or(i8::MAX),
            tick_rule: tick_rule.unwrap_or(u8::MAX),
            leg_instrument_class: leg_instrument_class
                .map(|leg_instrument_class| leg_instrument_class as u8 as c_char)
                .unwrap_or_default(),
            leg_side: leg_side.unwrap_or_default() as u8 as c_char,
            _reserved: Default::default(),
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[getter]
    fn get_pretty_min_price_increment(&self) -> f64 {
        self.min_price_increment_f64()
    }

    #[getter]
    fn get_pretty_display_factor(&self) -> f64 {
        self.display_factor_f64()
    }

    #[getter]
    fn get_pretty_expiration<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.expiration)
    }

    #[getter]
    fn get_pretty_activation<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.activation)
    }

    #[getter]
    fn get_pretty_high_limit_price(&self) -> f64 {
        self.high_limit_price_f64()
    }

    #[getter]
    fn get_pretty_low_limit_price(&self) -> f64 {
        self.low_limit_price_f64()
    }

    #[getter]
    fn get_pretty_max_price_variation(&self) -> f64 {
        self.max_price_variation_f64()
    }

    #[getter]
    fn get_pretty_unit_of_measure_qty(&self) -> f64 {
        self.unit_of_measure_qty_f64()
    }

    #[getter]
    fn get_pretty_min_price_increment_amount(&self) -> f64 {
        self.min_price_increment_amount_f64()
    }

    #[getter]
    fn get_pretty_price_ratio(&self) -> f64 {
        self.price_ratio_f64()
    }

    #[getter]
    fn get_pretty_strike_price(&self) -> f64 {
        self.strike_price_f64()
    }

    #[getter]
    fn get_pretty_leg_price(&self) -> f64 {
        self.leg_price_f64()
    }

    #[getter]
    fn get_pretty_leg_delta(&self) -> f64 {
        self.leg_delta_f64()
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
    fn get_leg_raw_symbol(&self) -> PyResult<&str> {
        Ok(self.leg_raw_symbol()?)
    }

    #[getter]
    fn get_instrument_class(&self) -> PyResult<InstrumentClass> {
        self.instrument_class().map_err(to_py_err)
    }
    #[setter]
    fn set_instrument_class(&mut self, instrument_class: InstrumentClass) {
        self.instrument_class = instrument_class as u8 as c_char;
    }

    #[getter]
    fn get_match_algorithm(&self) -> PyResult<MatchAlgorithm> {
        self.match_algorithm().map_err(to_py_err)
    }
    #[setter]
    fn set_match_algorithm(&mut self, match_algorithm: MatchAlgorithm) {
        self.match_algorithm = match_algorithm as u8 as c_char;
    }

    #[getter]
    fn get_security_update_action(&self) -> PyResult<SecurityUpdateAction> {
        self.security_update_action().map_err(to_py_err)
    }
    #[setter]
    fn set_security_update_action(&mut self, security_update_action: SecurityUpdateAction) {
        self.security_update_action = security_update_action as u8 as c_char;
    }

    #[getter]
    fn get_user_defined_instrument(&self) -> PyResult<UserDefinedInstrument> {
        self.user_defined_instrument().map_err(to_py_err)
    }
    #[setter]
    fn set_user_defined_instrument(&mut self, user_defined_instrument: UserDefinedInstrument) {
        self.user_defined_instrument = user_defined_instrument as u8 as c_char;
    }

    #[getter]
    fn get_leg_instrument_class(&self) -> PyResult<InstrumentClass> {
        self.leg_instrument_class().map_err(to_py_err)
    }
    #[setter]
    fn set_leg_instrument_class(&mut self, leg_instrument_class: InstrumentClass) {
        self.leg_instrument_class = leg_instrument_class as u8 as c_char;
    }

    #[getter]
    fn get_leg_side(&self) -> PyResult<Side> {
        self.leg_side().map_err(to_py_err)
    }
    #[setter]
    fn set_leg_side(&mut self, leg_side: Side) {
        self.leg_side = leg_side as u8 as c_char;
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
    #[pyo3(signature = (
        publisher_id,
        instrument_id,
        ts_event,
        ts_recv,
        ref_price,
        auction_time,
        cont_book_clr_price,
        auct_interest_clr_price,
        paired_qty,
        total_imbalance_qty,
        auction_type,
        side,
        significant_imbalance,
        ssr_filling_price = UNDEF_PRICE,
        ind_match_price = UNDEF_PRICE,
        upper_collar = UNDEF_PRICE,
        lower_collar = UNDEF_PRICE,
        market_imbalance_qty = UNDEF_ORDER_SIZE,
        unpaired_qty = UNDEF_ORDER_SIZE,
        auction_status = 0,
        freeze_status = 0,
        num_extensions = 0,
        unpaired_side = None,
    ))]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        ts_recv: u64,
        ref_price: i64,
        auction_time: u64,
        cont_book_clr_price: i64,
        auct_interest_clr_price: i64,
        paired_qty: u32,
        total_imbalance_qty: u32,
        auction_type: char,
        side: Side,
        significant_imbalance: char,
        ssr_filling_price: i64,
        ind_match_price: i64,
        upper_collar: i64,
        lower_collar: i64,
        market_imbalance_qty: u32,
        unpaired_qty: u32,
        auction_status: u8,
        freeze_status: u8,
        num_extensions: u8,
        unpaired_side: Option<Side>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::IMBALANCE, publisher_id, instrument_id, ts_event),
            ts_recv,
            ref_price,
            auction_time,
            cont_book_clr_price,
            auct_interest_clr_price,
            ssr_filling_price,
            ind_match_price,
            upper_collar,
            lower_collar,
            paired_qty,
            total_imbalance_qty,
            market_imbalance_qty,
            unpaired_qty,
            auction_type: char_to_c_char(auction_type)?,
            side: side as u8 as c_char,
            auction_status,
            freeze_status,
            num_extensions,
            unpaired_side: unpaired_side.unwrap_or_default() as u8 as c_char,

            significant_imbalance: char_to_c_char(significant_imbalance)?,
            _reserved: Default::default(),
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[getter]
    fn get_pretty_ref_price(&self) -> f64 {
        self.ref_price_f64()
    }

    #[getter]
    fn get_pretty_auction_time<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.auction_time)
    }

    #[getter]
    fn get_pretty_cont_book_clr_price(&self) -> f64 {
        self.cont_book_clr_price_f64()
    }

    #[getter]
    fn get_pretty_auct_interest_clr_price(&self) -> f64 {
        self.auct_interest_clr_price_f64()
    }

    #[getter]
    fn get_pretty_ssr_filling_price(&self) -> f64 {
        self.ssr_filling_price_f64()
    }

    #[getter]
    fn get_pretty_ind_match_price(&self) -> f64 {
        self.ind_match_price_f64()
    }

    #[getter]
    fn get_pretty_upper_collar(&self) -> f64 {
        self.upper_collar_f64()
    }

    #[getter]
    fn get_pretty_lower_collar(&self) -> f64 {
        self.lower_collar_f64()
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
    fn get_side(&self) -> PyResult<Side> {
        self.side().map_err(to_py_err)
    }
    #[setter]
    fn set_side(&mut self, side: Side) {
        self.side = side as u8 as c_char;
    }

    #[getter]
    fn get_unpaired_side(&self) -> PyResult<Side> {
        self.unpaired_side().map_err(to_py_err)
    }
    #[setter]
    fn set_unpaired_side(&mut self, unpaired_side: Side) {
        self.unpaired_side = unpaired_side as u8 as c_char;
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
    #[pyo3(signature = (
        publisher_id,
        instrument_id,
        ts_event,
        ts_recv,
        ts_ref,
        price,
        quantity,
        stat_type,
        sequence = 0,
        ts_in_delta = 0,
        channel_id = 0,
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
        quantity: i64,
        stat_type: StatType,
        sequence: u32,
        ts_in_delta: i32,
        channel_id: u16,
        update_action: Option<StatUpdateAction>,
        stat_flags: u8,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::STATISTICS, publisher_id, instrument_id, ts_event),
            ts_recv,
            ts_ref,
            price,
            quantity,
            sequence,
            ts_in_delta,
            stat_type: stat_type as u16,
            channel_id,
            update_action: update_action.unwrap_or_default() as u8,
            stat_flags,
            _reserved: Default::default(),
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[getter]
    fn get_pretty_ts_ref<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_ref)
    }

    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_stat_type(&self) -> PyResult<StatType> {
        self.stat_type().map_err(to_py_err)
    }
    #[setter]
    fn set_stat_type(&mut self, stat_type: StatType) {
        self.stat_type = stat_type as u16;
    }

    #[getter]
    fn get_update_action(&self) -> PyResult<StatUpdateAction> {
        self.update_action().map_err(to_py_err)
    }
    #[setter]
    fn set_update_action(&mut self, update_action: StatUpdateAction) {
        self.update_action = update_action as u8;
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
    #[pyo3(signature = (ts_event, err, is_last = true, code = None))]
    fn py_new(ts_event: u64, err: &str, is_last: bool, code: Option<ErrorCode>) -> PyResult<Self> {
        Ok(Self::new(ts_event, code, err, is_last))
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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

    #[getter]
    fn get_code(&self) -> PyResult<ErrorCode> {
        self.code().map_err(to_py_err)
    }
    #[setter]
    fn set_code(&mut self, code: ErrorCode) {
        self.code = code as u8;
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
    #[pyo3(signature = (
        publisher_id,
        instrument_id,
        ts_event,
        stype_in,
        stype_in_symbol,
        stype_out,
        stype_out_symbol,
        start_ts,
        end_ts,
    ))]
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
        self.stype_in().map_err(to_py_err)
    }
    #[setter]
    fn set_stype_in(&mut self, stype_in: SType) {
        self.stype_in = stype_in as u8;
    }

    #[getter]
    fn get_stype_in_symbol(&self) -> PyResult<&str> {
        Ok(self.stype_in_symbol()?)
    }

    #[getter]
    fn get_stype_out(&self) -> PyResult<SType> {
        self.stype_out().map_err(to_py_err)
    }
    #[setter]
    fn set_stype_out(&mut self, stype_out: SType) {
        self.stype_out = stype_out as u8;
    }

    #[getter]
    fn get_stype_out_symbol(&self) -> PyResult<&str> {
        Ok(self.stype_out_symbol()?)
    }

    #[getter]
    fn get_pretty_start_ts<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.start_ts)
    }

    #[getter]
    fn get_pretty_end_ts<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.end_ts)
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
    #[pyo3(signature = (ts_event, msg, code = None))]
    fn py_new(ts_event: u64, msg: &str, code: Option<SystemCode>) -> PyResult<Self> {
        Ok(Self::new(ts_event, code, msg)?)
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[pyo3(name = "is_heartbeat")]
    fn py_is_heartbeat(&self) -> bool {
        self.is_heartbeat()
    }

    #[getter]
    fn get_msg(&self) -> PyResult<&str> {
        Ok(self.msg()?)
    }

    #[getter]
    fn get_code(&self) -> PyResult<SystemCode> {
        self.code().map_err(to_py_err)
    }
    #[setter]
    fn set_code(&mut self, code: SystemCode) {
        self.code = code as u8;
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
impl v1::ErrorMsg {
    #[new]
    fn py_new(ts_event: u64, err: &str) -> PyResult<Self> {
        Ok(Self::new(ts_event, err))
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
impl v1::InstrumentDefMsg {
    #[new]
    #[pyo3(signature = (
        publisher_id,
        instrument_id,
        ts_event,
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
        inst_attrib_value,
        underlying_id,
        raw_instrument_id,
        market_depth_implied,
        market_depth,
        market_segment_id,
        max_trade_vol,
        min_lot_size,
        min_lot_size_block,
        min_lot_size_round_lot,
        min_trade_vol,
        contract_multiplier,
        decay_quantity,
        original_contract_size,
        trading_reference_date,
        appl_id,
        maturity_year,
        decay_start_date,
        channel_id,
        currency,
        settl_currency,
        secsubtype,
        raw_symbol,
        group,
        exchange,
        asset,
        cfi,
        security_type,
        unit_of_measure,
        underlying,
        strike_price_currency,
        instrument_class,
        strike_price,
        match_algorithm,
        md_security_trading_status,
        main_fraction,
        price_display_format,
        settl_price_type,
        sub_fraction,
        underlying_product,
        security_update_action,
        maturity_month,
        maturity_day,
        maturity_week,
        user_defined_instrument,
        contract_multiplier_unit,
        flow_schedule_type,
        tick_rule,
    ))]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        ts_recv: u64,
        min_price_increment: i64,
        display_factor: i64,
        expiration: u64,
        activation: u64,
        high_limit_price: i64,
        low_limit_price: i64,
        max_price_variation: i64,
        trading_reference_price: i64,
        unit_of_measure_qty: i64,
        min_price_increment_amount: i64,
        price_ratio: i64,
        inst_attrib_value: i32,
        underlying_id: u32,
        raw_instrument_id: u32,
        market_depth_implied: i32,
        market_depth: i32,
        market_segment_id: u32,
        max_trade_vol: u32,
        min_lot_size: i32,
        min_lot_size_block: i32,
        min_lot_size_round_lot: i32,
        min_trade_vol: u32,
        contract_multiplier: i32,
        decay_quantity: i32,
        original_contract_size: i32,
        trading_reference_date: u16,
        appl_id: i16,
        maturity_year: u16,
        decay_start_date: u16,
        channel_id: u16,
        currency: &str,
        settl_currency: &str,
        secsubtype: &str,
        raw_symbol: &str,
        group: &str,
        exchange: &str,
        asset: &str,
        cfi: &str,
        security_type: &str,
        unit_of_measure: &str,
        underlying: &str,
        strike_price_currency: &str,
        instrument_class: InstrumentClass,
        strike_price: i64,
        match_algorithm: MatchAlgorithm,
        md_security_trading_status: u8,
        main_fraction: u8,
        price_display_format: u8,
        settl_price_type: u8,
        sub_fraction: u8,
        underlying_product: u8,
        security_update_action: SecurityUpdateAction,
        maturity_month: u8,
        maturity_day: u8,
        maturity_week: u8,
        user_defined_instrument: UserDefinedInstrument,
        contract_multiplier_unit: i8,
        flow_schedule_type: i8,
        tick_rule: u8,
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
            inst_attrib_value,
            underlying_id,
            raw_instrument_id,
            market_depth_implied,
            market_depth,
            market_segment_id,
            max_trade_vol,
            min_lot_size,
            min_lot_size_block,
            min_lot_size_round_lot,
            min_trade_vol,
            _reserved2: Default::default(),
            contract_multiplier,
            decay_quantity,
            original_contract_size,
            _reserved3: Default::default(),
            trading_reference_date,
            appl_id,
            maturity_year,
            decay_start_date,
            channel_id,
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
            instrument_class: instrument_class as u8 as c_char,
            _reserved4: Default::default(),
            strike_price,
            _reserved5: Default::default(),
            match_algorithm: match_algorithm as u8 as c_char,
            md_security_trading_status,
            main_fraction,
            price_display_format,
            settl_price_type,
            sub_fraction,
            underlying_product,
            security_update_action,
            maturity_month,
            maturity_day,
            maturity_week,
            user_defined_instrument,
            contract_multiplier_unit,
            flow_schedule_type,
            tick_rule,
            _dummy: Default::default(),
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[getter]
    fn get_pretty_min_price_increment(&self) -> f64 {
        self.min_price_increment_f64()
    }

    #[getter]
    fn get_pretty_display_factor(&self) -> f64 {
        self.display_factor_f64()
    }

    #[getter]
    fn get_pretty_expiration<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.expiration)
    }

    #[getter]
    fn get_pretty_activation<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.activation)
    }

    #[getter]
    fn get_pretty_high_limit_price(&self) -> f64 {
        self.high_limit_price_f64()
    }

    #[getter]
    fn get_pretty_low_limit_price(&self) -> f64 {
        self.low_limit_price_f64()
    }

    #[getter]
    fn get_pretty_max_price_variation(&self) -> f64 {
        self.max_price_variation_f64()
    }

    #[getter]
    fn get_pretty_trading_reference_price(&self) -> f64 {
        self.trading_reference_price_f64()
    }

    #[getter]
    fn get_pretty_unit_of_measure_qty(&self) -> f64 {
        self.unit_of_measure_qty_f64()
    }

    #[getter]
    fn get_pretty_min_price_increment_amount(&self) -> f64 {
        self.min_price_increment_amount_f64()
    }

    #[getter]
    fn get_pretty_price_ratio(&self) -> f64 {
        self.price_ratio_f64()
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
    fn get_instrument_class(&self) -> PyResult<InstrumentClass> {
        self.instrument_class().map_err(to_py_err)
    }
    #[setter]
    fn set_instrument_class(&mut self, instrument_class: InstrumentClass) {
        self.instrument_class = instrument_class as u8 as c_char;
    }

    #[getter]
    fn get_pretty_strike_price(&self) -> f64 {
        self.strike_price_f64()
    }

    #[getter]
    fn get_match_algorithm(&self) -> PyResult<MatchAlgorithm> {
        self.match_algorithm().map_err(to_py_err)
    }
    #[setter]
    fn set_match_algorithm(&mut self, match_algorithm: MatchAlgorithm) {
        self.match_algorithm = match_algorithm as u8 as c_char;
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
impl v1::StatMsg {
    #[new]
    #[pyo3(signature = (
        publisher_id,
        instrument_id,
        ts_event,
        ts_recv,
        ts_ref,
        price,
        quantity,
        stat_type,
        sequence = 0,
        ts_in_delta = 0,
        channel_id = 0,
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
        stat_type: StatType,
        sequence: u32,
        ts_in_delta: i32,
        channel_id: u16,
        update_action: Option<StatUpdateAction>,
        stat_flags: u8,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::STATISTICS, publisher_id, instrument_id, ts_event),
            ts_recv,
            ts_ref,
            price,
            quantity,
            sequence,
            ts_in_delta,
            stat_type: stat_type as u16,
            channel_id,
            update_action: update_action.unwrap_or_default() as u8,
            stat_flags,
            _reserved: Default::default(),
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[getter]
    fn get_pretty_ts_ref<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_ref)
    }

    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.price_f64()
    }

    #[getter]
    fn get_stat_type(&self) -> PyResult<StatType> {
        self.stat_type().map_err(to_py_err)
    }
    #[setter]
    fn set_stat_type(&mut self, stat_type: StatType) {
        self.stat_type = stat_type as u16;
    }

    #[getter]
    fn get_update_action(&self) -> PyResult<StatUpdateAction> {
        self.update_action().map_err(to_py_err)
    }
    #[setter]
    fn set_update_action(&mut self, update_action: StatUpdateAction) {
        self.update_action = update_action as u8;
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
impl v1::SymbolMappingMsg {
    #[new]
    #[pyo3(signature = (
        publisher_id,
        instrument_id,
        ts_event,
        stype_in_symbol,
        stype_out_symbol,
        start_ts,
        end_ts,
    ))]
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
            _dummy: Default::default(),
            start_ts,
            end_ts,
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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

    #[getter]
    fn get_pretty_start_ts<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.start_ts)
    }

    #[getter]
    fn get_pretty_end_ts<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.end_ts)
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
impl v1::SystemMsg {
    #[new]
    fn py_new(ts_event: u64, msg: &str) -> PyResult<Self> {
        Ok(Self::new(ts_event, msg)?)
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Self>())
    }

    #[pyo3(name = "is_heartbeat")]
    fn py_is_heartbeat(&self) -> bool {
        self.is_heartbeat()
    }

    #[getter]
    fn get_msg(&self) -> PyResult<&str> {
        Ok(self.msg()?)
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
impl v2::InstrumentDefMsg {
    #[new]
    #[pyo3(signature = (
        publisher_id,
        instrument_id,
        ts_event,
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
        strike_price,
        inst_attrib_value,
        underlying_id,
        raw_instrument_id,
        market_depth_implied,
        market_depth,
        market_segment_id,
        max_trade_vol,
        min_lot_size,
        min_lot_size_block,
        min_lot_size_round_lot,
        min_trade_vol,
        contract_multiplier,
        decay_quantity,
        original_contract_size,
        trading_reference_date,
        appl_id,
        maturity_year,
        decay_start_date,
        channel_id,
        currency,
        settl_currency,
        secsubtype,
        raw_symbol,
        group,
        exchange,
        asset,
        cfi,
        security_type,
        unit_of_measure,
        underlying,
        strike_price_currency,
        instrument_class,
        match_algorithm,
        md_security_trading_status,
        main_fraction,
        price_display_format,
        settl_price_type,
        sub_fraction,
        underlying_product,
        security_update_action,
        maturity_month,
        maturity_day,
        maturity_week,
        user_defined_instrument,
        contract_multiplier_unit,
        flow_schedule_type,
        tick_rule,
    ))]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        ts_recv: u64,
        min_price_increment: i64,
        display_factor: i64,
        expiration: u64,
        activation: u64,
        high_limit_price: i64,
        low_limit_price: i64,
        max_price_variation: i64,
        trading_reference_price: i64,
        unit_of_measure_qty: i64,
        min_price_increment_amount: i64,
        price_ratio: i64,
        strike_price: i64,
        inst_attrib_value: i32,
        underlying_id: u32,
        raw_instrument_id: u32,
        market_depth_implied: i32,
        market_depth: i32,
        market_segment_id: u32,
        max_trade_vol: u32,
        min_lot_size: i32,
        min_lot_size_block: i32,
        min_lot_size_round_lot: i32,
        min_trade_vol: u32,
        contract_multiplier: i32,
        decay_quantity: i32,
        original_contract_size: i32,
        trading_reference_date: u16,
        appl_id: i16,
        maturity_year: u16,
        decay_start_date: u16,
        channel_id: u16,
        currency: &str,
        settl_currency: &str,
        secsubtype: &str,
        raw_symbol: &str,
        group: &str,
        exchange: &str,
        asset: &str,
        cfi: &str,
        security_type: &str,
        unit_of_measure: &str,
        underlying: &str,
        strike_price_currency: &str,
        instrument_class: InstrumentClass,
        match_algorithm: MatchAlgorithm,
        md_security_trading_status: u8,
        main_fraction: u8,
        price_display_format: u8,
        settl_price_type: u8,
        sub_fraction: u8,
        underlying_product: u8,
        security_update_action: SecurityUpdateAction,
        maturity_month: u8,
        maturity_day: u8,
        maturity_week: u8,
        user_defined_instrument: UserDefinedInstrument,
        contract_multiplier_unit: i8,
        flow_schedule_type: i8,
        tick_rule: u8,
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
            strike_price,
            inst_attrib_value,
            underlying_id,
            raw_instrument_id,
            market_depth_implied,
            market_depth,
            market_segment_id,
            max_trade_vol,
            min_lot_size,
            min_lot_size_block,
            min_lot_size_round_lot,
            min_trade_vol,
            contract_multiplier,
            decay_quantity,
            original_contract_size,
            trading_reference_date,
            appl_id,
            maturity_year,
            decay_start_date,
            channel_id,
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
            instrument_class: instrument_class as u8 as c_char,
            match_algorithm: match_algorithm as u8 as c_char,
            md_security_trading_status,
            main_fraction,
            price_display_format,
            settl_price_type,
            sub_fraction,
            underlying_product,
            security_update_action: security_update_action as u8 as c_char,
            maturity_month,
            maturity_day,
            maturity_week,
            user_defined_instrument,
            contract_multiplier_unit,
            flow_schedule_type,
            tick_rule,
            _reserved: Default::default(),
        })
    }

    fn __bytes__(&self) -> &[u8] {
        self.as_ref()
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
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_event())
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.hd.ts_event = ts_event;
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
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_recv)
    }

    #[getter]
    fn get_pretty_min_price_increment(&self) -> f64 {
        self.min_price_increment_f64()
    }

    #[getter]
    fn get_pretty_display_factor(&self) -> f64 {
        self.display_factor_f64()
    }

    #[getter]
    fn get_pretty_expiration<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.expiration)
    }

    #[getter]
    fn get_pretty_activation<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.activation)
    }

    #[getter]
    fn get_pretty_high_limit_price(&self) -> f64 {
        self.high_limit_price_f64()
    }

    #[getter]
    fn get_pretty_low_limit_price(&self) -> f64 {
        self.low_limit_price_f64()
    }

    #[getter]
    fn get_pretty_max_price_variation(&self) -> f64 {
        self.max_price_variation_f64()
    }

    #[getter]
    fn get_pretty_trading_reference_price(&self) -> f64 {
        self.trading_reference_price_f64()
    }

    #[getter]
    fn get_pretty_unit_of_measure_qty(&self) -> f64 {
        self.unit_of_measure_qty_f64()
    }

    #[getter]
    fn get_pretty_min_price_increment_amount(&self) -> f64 {
        self.min_price_increment_amount_f64()
    }

    #[getter]
    fn get_pretty_price_ratio(&self) -> f64 {
        self.price_ratio_f64()
    }

    #[getter]
    fn get_pretty_strike_price(&self) -> f64 {
        self.strike_price_f64()
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
    fn get_instrument_class(&self) -> PyResult<InstrumentClass> {
        self.instrument_class().map_err(to_py_err)
    }
    #[setter]
    fn set_instrument_class(&mut self, instrument_class: InstrumentClass) {
        self.instrument_class = instrument_class as u8 as c_char;
    }

    #[getter]
    fn get_match_algorithm(&self) -> PyResult<MatchAlgorithm> {
        self.match_algorithm().map_err(to_py_err)
    }
    #[setter]
    fn set_match_algorithm(&mut self, match_algorithm: MatchAlgorithm) {
        self.match_algorithm = match_algorithm as u8 as c_char;
    }

    #[getter]
    fn get_security_update_action(&self) -> PyResult<SecurityUpdateAction> {
        self.security_update_action().map_err(to_py_err)
    }
    #[setter]
    fn set_security_update_action(&mut self, security_update_action: SecurityUpdateAction) {
        self.security_update_action = security_update_action as u8 as c_char;
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
