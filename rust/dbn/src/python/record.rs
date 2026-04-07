use std::{ffi::c_char, mem};

use pyo3::{conversion::IntoPyObjectExt, prelude::*};

use crate::{
    python::WritePyRepr, record::str_to_c_chars, rtype, v1, v2, Action, BboMsg, BidAskPair,
    CbboMsg, Cmbp1Msg, ConsolidatedBidAskPair, ErrorCode, ErrorMsg, FlagSet, ImbalanceMsg,
    InstrumentClass, InstrumentDefMsg, MatchAlgorithm, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, RType,
    Record, RecordHeader, SType, SecurityUpdateAction, Side, StatMsg, StatType, StatUpdateAction,
    StatusAction, StatusMsg, StatusReason, SymbolMappingMsg, SystemCode, SystemMsg, TradeMsg,
    TradingEvent, TriState, UserDefinedInstrument, WithTsOut, UNDEF_ORDER_SIZE, UNDEF_PRICE,
    UNDEF_TIMESTAMP,
};

use super::{
    conversions::{char_to_c_char, new_py_timestamp_or_datetime},
    to_py_err, PyFieldDesc,
};

/// Python wrapper for [`MboMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "MBOMsg")]
pub struct PyMboMsg {
    pub(crate) inner: MboMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyMboMsg {
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
        ts_in_delta = 0,
        sequence = 0,
        ts_out = UNDEF_TIMESTAMP,
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
        channel_id: Option<u8>,
        ts_in_delta: i32,
        sequence: u32,
        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: MboMsg {
                hd: RecordHeader::new::<MboMsg>(rtype::MBO, publisher_id, instrument_id, ts_event),
                order_id,
                price,
                size,
                flags: flags.unwrap_or_default(),
                channel_id: channel_id.unwrap_or(u8::MAX),
                action: action as u8 as c_char,
                side: side as u8 as c_char,
                ts_recv,
                ts_in_delta,
                sequence,
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<MboMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<MboMsg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]

    fn get_order_id(&self) -> u64 {
        self.inner.order_id
    }
    #[setter]
    fn set_order_id(&mut self, order_id: u64) {
        self.inner.order_id = order_id;
    }
    #[getter]

    fn get_price(&self) -> i64 {
        self.inner.price
    }
    #[setter]
    fn set_price(&mut self, price: i64) {
        self.inner.price = price;
    }
    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.inner.price_f64()
    }
    #[getter]

    fn get_size(&self) -> u32 {
        self.inner.size
    }
    #[setter]
    fn set_size(&mut self, size: u32) {
        self.inner.size = size;
    }
    #[getter]

    fn get_flags(&self) -> FlagSet {
        self.inner.flags
    }
    #[setter]
    fn set_flags(&mut self, flags: FlagSet) {
        self.inner.flags = flags;
    }
    #[getter]

    fn get_channel_id(&self) -> u8 {
        self.inner.channel_id
    }
    #[setter]
    fn set_channel_id(&mut self, channel_id: u8) {
        self.inner.channel_id = channel_id;
    }
    #[getter]
    fn get_action<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .action()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.action as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_action(&mut self, action: Action) {
        self.inner.action = action as u8 as c_char;
    }
    #[getter]
    fn get_side<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .side()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.side as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_side(&mut self, side: Side) {
        self.inner.side = side as u8 as c_char;
    }
    #[getter]

    fn get_ts_recv(&self) -> u64 {
        self.inner.ts_recv
    }
    #[setter]
    fn set_ts_recv(&mut self, ts_recv: u64) {
        self.inner.ts_recv = ts_recv;
    }
    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.ts_recv)
    }
    #[getter]

    fn get_ts_in_delta(&self) -> i32 {
        self.inner.ts_in_delta
    }
    #[setter]
    fn set_ts_in_delta(&mut self, ts_in_delta: i32) {
        self.inner.ts_in_delta = ts_in_delta;
    }
    #[getter]

    fn get_sequence(&self) -> u32 {
        self.inner.sequence
    }
    #[setter]
    fn set_sequence(&mut self, sequence: u32) {
        self.inner.sequence = sequence;
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        MboMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        MboMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        MboMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        MboMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        MboMsg::ordered_fields("")
    }
}

/// Convert bare [`MboMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for MboMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyMboMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<MboMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<MboMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyMboMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
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
        let mut s = String::new();
        self.write_py_repr(&mut s).unwrap();
        s
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
        let mut s = String::new();
        self.write_py_repr(&mut s).unwrap();
        s
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

/// Python wrapper for [`TradeMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "TradeMsg")]
pub struct PyTradeMsg {
    pub(crate) inner: TradeMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyTradeMsg {
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
        ts_out = UNDEF_TIMESTAMP,
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
        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: TradeMsg {
                hd: RecordHeader::new::<TradeMsg>(
                    rtype::MBP_0,
                    publisher_id,
                    instrument_id,
                    ts_event,
                ),
                price,
                size,
                action: action as u8 as c_char,
                side: side as u8 as c_char,
                flags: flags.unwrap_or_default(),
                depth,
                ts_recv,
                ts_in_delta,
                sequence,
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<TradeMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<TradeMsg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]

    fn get_price(&self) -> i64 {
        self.inner.price
    }
    #[setter]
    fn set_price(&mut self, price: i64) {
        self.inner.price = price;
    }
    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.inner.price_f64()
    }
    #[getter]

    fn get_size(&self) -> u32 {
        self.inner.size
    }
    #[setter]
    fn set_size(&mut self, size: u32) {
        self.inner.size = size;
    }
    #[getter]
    fn get_action<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .action()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.action as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_action(&mut self, action: Action) {
        self.inner.action = action as u8 as c_char;
    }
    #[getter]
    fn get_side<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .side()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.side as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_side(&mut self, side: Side) {
        self.inner.side = side as u8 as c_char;
    }
    #[getter]

    fn get_flags(&self) -> FlagSet {
        self.inner.flags
    }
    #[setter]
    fn set_flags(&mut self, flags: FlagSet) {
        self.inner.flags = flags;
    }
    #[getter]

    fn get_depth(&self) -> u8 {
        self.inner.depth
    }
    #[setter]
    fn set_depth(&mut self, depth: u8) {
        self.inner.depth = depth;
    }
    #[getter]

    fn get_ts_recv(&self) -> u64 {
        self.inner.ts_recv
    }
    #[setter]
    fn set_ts_recv(&mut self, ts_recv: u64) {
        self.inner.ts_recv = ts_recv;
    }
    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.ts_recv)
    }
    #[getter]

    fn get_ts_in_delta(&self) -> i32 {
        self.inner.ts_in_delta
    }
    #[setter]
    fn set_ts_in_delta(&mut self, ts_in_delta: i32) {
        self.inner.ts_in_delta = ts_in_delta;
    }
    #[getter]

    fn get_sequence(&self) -> u32 {
        self.inner.sequence
    }
    #[setter]
    fn set_sequence(&mut self, sequence: u32) {
        self.inner.sequence = sequence;
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        TradeMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        TradeMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        TradeMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        TradeMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        TradeMsg::ordered_fields("")
    }
}

/// Convert bare [`TradeMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for TradeMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyTradeMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<TradeMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<TradeMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyTradeMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`Mbp1Msg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "MBP1Msg")]
pub struct PyMbp1Msg {
    pub(crate) inner: Mbp1Msg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyMbp1Msg {
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
        ts_out = UNDEF_TIMESTAMP,
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
        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: Mbp1Msg {
                hd: RecordHeader::new::<Mbp1Msg>(
                    rtype::MBP_1,
                    publisher_id,
                    instrument_id,
                    ts_event,
                ),
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
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Mbp1Msg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<Mbp1Msg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]

    fn get_price(&self) -> i64 {
        self.inner.price
    }
    #[setter]
    fn set_price(&mut self, price: i64) {
        self.inner.price = price;
    }
    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.inner.price_f64()
    }
    #[getter]

    fn get_size(&self) -> u32 {
        self.inner.size
    }
    #[setter]
    fn set_size(&mut self, size: u32) {
        self.inner.size = size;
    }
    #[getter]
    fn get_action<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .action()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.action as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_action(&mut self, action: Action) {
        self.inner.action = action as u8 as c_char;
    }
    #[getter]
    fn get_side<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .side()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.side as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_side(&mut self, side: Side) {
        self.inner.side = side as u8 as c_char;
    }
    #[getter]

    fn get_flags(&self) -> FlagSet {
        self.inner.flags
    }
    #[setter]
    fn set_flags(&mut self, flags: FlagSet) {
        self.inner.flags = flags;
    }
    #[getter]

    fn get_depth(&self) -> u8 {
        self.inner.depth
    }
    #[setter]
    fn set_depth(&mut self, depth: u8) {
        self.inner.depth = depth;
    }
    #[getter]

    fn get_ts_recv(&self) -> u64 {
        self.inner.ts_recv
    }
    #[setter]
    fn set_ts_recv(&mut self, ts_recv: u64) {
        self.inner.ts_recv = ts_recv;
    }
    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.ts_recv)
    }
    #[getter]

    fn get_ts_in_delta(&self) -> i32 {
        self.inner.ts_in_delta
    }
    #[setter]
    fn set_ts_in_delta(&mut self, ts_in_delta: i32) {
        self.inner.ts_in_delta = ts_in_delta;
    }
    #[getter]

    fn get_sequence(&self) -> u32 {
        self.inner.sequence
    }
    #[setter]
    fn set_sequence(&mut self, sequence: u32) {
        self.inner.sequence = sequence;
    }
    #[getter]
    #[allow(clippy::clone_on_copy)]
    fn get_levels(&self) -> [BidAskPair; 1] {
        self.inner.levels.clone()
    }
    #[setter]
    fn set_levels(&mut self, levels: [BidAskPair; 1]) {
        self.inner.levels = levels;
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Mbp1Msg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Mbp1Msg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Mbp1Msg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Mbp1Msg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Mbp1Msg::ordered_fields("")
    }
}

/// Convert bare [`Mbp1Msg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for Mbp1Msg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyMbp1Msg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<Mbp1Msg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<Mbp1Msg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyMbp1Msg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`Mbp10Msg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "MBP10Msg")]
pub struct PyMbp10Msg {
    pub(crate) inner: Mbp10Msg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyMbp10Msg {
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
        ts_out = UNDEF_TIMESTAMP,
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
        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: Mbp10Msg {
                hd: RecordHeader::new::<Mbp10Msg>(
                    rtype::MBP_10,
                    publisher_id,
                    instrument_id,
                    ts_event,
                ),
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
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Mbp10Msg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<Mbp10Msg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]

    fn get_price(&self) -> i64 {
        self.inner.price
    }
    #[setter]
    fn set_price(&mut self, price: i64) {
        self.inner.price = price;
    }
    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.inner.price_f64()
    }
    #[getter]

    fn get_size(&self) -> u32 {
        self.inner.size
    }
    #[setter]
    fn set_size(&mut self, size: u32) {
        self.inner.size = size;
    }
    #[getter]
    fn get_action<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .action()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.action as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_action(&mut self, action: Action) {
        self.inner.action = action as u8 as c_char;
    }
    #[getter]
    fn get_side<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .side()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.side as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_side(&mut self, side: Side) {
        self.inner.side = side as u8 as c_char;
    }
    #[getter]

    fn get_flags(&self) -> FlagSet {
        self.inner.flags
    }
    #[setter]
    fn set_flags(&mut self, flags: FlagSet) {
        self.inner.flags = flags;
    }
    #[getter]

    fn get_depth(&self) -> u8 {
        self.inner.depth
    }
    #[setter]
    fn set_depth(&mut self, depth: u8) {
        self.inner.depth = depth;
    }
    #[getter]

    fn get_ts_recv(&self) -> u64 {
        self.inner.ts_recv
    }
    #[setter]
    fn set_ts_recv(&mut self, ts_recv: u64) {
        self.inner.ts_recv = ts_recv;
    }
    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.ts_recv)
    }
    #[getter]

    fn get_ts_in_delta(&self) -> i32 {
        self.inner.ts_in_delta
    }
    #[setter]
    fn set_ts_in_delta(&mut self, ts_in_delta: i32) {
        self.inner.ts_in_delta = ts_in_delta;
    }
    #[getter]

    fn get_sequence(&self) -> u32 {
        self.inner.sequence
    }
    #[setter]
    fn set_sequence(&mut self, sequence: u32) {
        self.inner.sequence = sequence;
    }
    #[getter]
    #[allow(clippy::clone_on_copy)]
    fn get_levels(&self) -> [BidAskPair; 10] {
        self.inner.levels.clone()
    }
    #[setter]
    fn set_levels(&mut self, levels: [BidAskPair; 10]) {
        self.inner.levels = levels;
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Mbp10Msg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Mbp10Msg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Mbp10Msg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Mbp10Msg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Mbp10Msg::ordered_fields("")
    }
}

/// Convert bare [`Mbp10Msg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for Mbp10Msg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyMbp10Msg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<Mbp10Msg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<Mbp10Msg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyMbp10Msg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`BboMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "BBOMsg")]
pub struct PyBboMsg {
    pub(crate) inner: BboMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyBboMsg {
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
        ts_out = UNDEF_TIMESTAMP,
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
        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: BboMsg {
                hd: RecordHeader::new::<BboMsg>(rtype, publisher_id, instrument_id, ts_event),
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
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<BboMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<BboMsg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]

    fn get_price(&self) -> i64 {
        self.inner.price
    }
    #[setter]
    fn set_price(&mut self, price: i64) {
        self.inner.price = price;
    }
    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.inner.price_f64()
    }
    #[getter]

    fn get_size(&self) -> u32 {
        self.inner.size
    }
    #[setter]
    fn set_size(&mut self, size: u32) {
        self.inner.size = size;
    }
    #[getter]
    fn get_side<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .side()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.side as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_side(&mut self, side: Side) {
        self.inner.side = side as u8 as c_char;
    }
    #[getter]

    fn get_flags(&self) -> FlagSet {
        self.inner.flags
    }
    #[setter]
    fn set_flags(&mut self, flags: FlagSet) {
        self.inner.flags = flags;
    }
    #[getter]

    fn get_ts_recv(&self) -> u64 {
        self.inner.ts_recv
    }
    #[setter]
    fn set_ts_recv(&mut self, ts_recv: u64) {
        self.inner.ts_recv = ts_recv;
    }
    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.ts_recv)
    }
    #[getter]

    fn get_sequence(&self) -> u32 {
        self.inner.sequence
    }
    #[setter]
    fn set_sequence(&mut self, sequence: u32) {
        self.inner.sequence = sequence;
    }
    #[getter]
    #[allow(clippy::clone_on_copy)]
    fn get_levels(&self) -> [BidAskPair; 1] {
        self.inner.levels.clone()
    }
    #[setter]
    fn set_levels(&mut self, levels: [BidAskPair; 1]) {
        self.inner.levels = levels;
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        BboMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        BboMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        BboMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        BboMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        BboMsg::ordered_fields("")
    }
}

/// Convert bare [`BboMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for BboMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyBboMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<BboMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<BboMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyBboMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`Cmbp1Msg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "CMBP1Msg")]
pub struct PyCmbp1Msg {
    pub(crate) inner: Cmbp1Msg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyCmbp1Msg {
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
        ts_out = UNDEF_TIMESTAMP,
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
        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: Cmbp1Msg {
                hd: RecordHeader::new::<Cmbp1Msg>(rtype, publisher_id, instrument_id, ts_event),
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
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<Cmbp1Msg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<Cmbp1Msg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]

    fn get_price(&self) -> i64 {
        self.inner.price
    }
    #[setter]
    fn set_price(&mut self, price: i64) {
        self.inner.price = price;
    }
    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.inner.price_f64()
    }
    #[getter]

    fn get_size(&self) -> u32 {
        self.inner.size
    }
    #[setter]
    fn set_size(&mut self, size: u32) {
        self.inner.size = size;
    }
    #[getter]
    fn get_action<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .action()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.action as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_action(&mut self, action: Action) {
        self.inner.action = action as u8 as c_char;
    }
    #[getter]
    fn get_side<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .side()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.side as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_side(&mut self, side: Side) {
        self.inner.side = side as u8 as c_char;
    }
    #[getter]

    fn get_flags(&self) -> FlagSet {
        self.inner.flags
    }
    #[setter]
    fn set_flags(&mut self, flags: FlagSet) {
        self.inner.flags = flags;
    }
    #[getter]

    fn get_ts_recv(&self) -> u64 {
        self.inner.ts_recv
    }
    #[setter]
    fn set_ts_recv(&mut self, ts_recv: u64) {
        self.inner.ts_recv = ts_recv;
    }
    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.ts_recv)
    }
    #[getter]

    fn get_ts_in_delta(&self) -> i32 {
        self.inner.ts_in_delta
    }
    #[setter]
    fn set_ts_in_delta(&mut self, ts_in_delta: i32) {
        self.inner.ts_in_delta = ts_in_delta;
    }
    #[getter]
    #[allow(clippy::clone_on_copy)]
    fn get_levels(&self) -> [ConsolidatedBidAskPair; 1] {
        self.inner.levels.clone()
    }
    #[setter]
    fn set_levels(&mut self, levels: [ConsolidatedBidAskPair; 1]) {
        self.inner.levels = levels;
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        Cmbp1Msg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        Cmbp1Msg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        Cmbp1Msg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        Cmbp1Msg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        Cmbp1Msg::ordered_fields("")
    }
}

/// Convert bare [`Cmbp1Msg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for Cmbp1Msg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyCmbp1Msg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<Cmbp1Msg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<Cmbp1Msg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyCmbp1Msg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`CbboMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "CBBOMsg")]
pub struct PyCbboMsg {
    pub(crate) inner: CbboMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyCbboMsg {
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
        ts_out = UNDEF_TIMESTAMP,
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
        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: CbboMsg {
                hd: RecordHeader::new::<CbboMsg>(rtype, publisher_id, instrument_id, ts_event),
                price,
                size,
                _reserved1: Default::default(),
                side: side as u8 as c_char,
                flags: flags.unwrap_or_default(),
                _reserved2: Default::default(),
                ts_recv,
                _reserved3: Default::default(),
                levels: [levels.unwrap_or_default()],
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<CbboMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<CbboMsg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]

    fn get_price(&self) -> i64 {
        self.inner.price
    }
    #[setter]
    fn set_price(&mut self, price: i64) {
        self.inner.price = price;
    }
    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.inner.price_f64()
    }
    #[getter]

    fn get_size(&self) -> u32 {
        self.inner.size
    }
    #[setter]
    fn set_size(&mut self, size: u32) {
        self.inner.size = size;
    }
    #[getter]
    fn get_side<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .side()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.side as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_side(&mut self, side: Side) {
        self.inner.side = side as u8 as c_char;
    }
    #[getter]

    fn get_flags(&self) -> FlagSet {
        self.inner.flags
    }
    #[setter]
    fn set_flags(&mut self, flags: FlagSet) {
        self.inner.flags = flags;
    }
    #[getter]

    fn get_ts_recv(&self) -> u64 {
        self.inner.ts_recv
    }
    #[setter]
    fn set_ts_recv(&mut self, ts_recv: u64) {
        self.inner.ts_recv = ts_recv;
    }
    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.ts_recv)
    }
    #[getter]
    #[allow(clippy::clone_on_copy)]
    fn get_levels(&self) -> [ConsolidatedBidAskPair; 1] {
        self.inner.levels.clone()
    }
    #[setter]
    fn set_levels(&mut self, levels: [ConsolidatedBidAskPair; 1]) {
        self.inner.levels = levels;
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        CbboMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        CbboMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        CbboMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        CbboMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        CbboMsg::ordered_fields("")
    }
}

/// Convert bare [`CbboMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for CbboMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyCbboMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<CbboMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<CbboMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyCbboMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`OhlcvMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "OHLCVMsg")]
pub struct PyOhlcvMsg {
    pub(crate) inner: OhlcvMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyOhlcvMsg {
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
        ts_out = UNDEF_TIMESTAMP,
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

        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: OhlcvMsg {
                hd: RecordHeader::new::<OhlcvMsg>(rtype, publisher_id, instrument_id, ts_event),
                open,
                high,
                low,
                close,
                volume,
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<OhlcvMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<OhlcvMsg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]

    fn get_open(&self) -> i64 {
        self.inner.open
    }
    #[setter]
    fn set_open(&mut self, open: i64) {
        self.inner.open = open;
    }
    #[getter]
    fn get_pretty_open(&self) -> f64 {
        self.inner.open_f64()
    }
    #[getter]

    fn get_high(&self) -> i64 {
        self.inner.high
    }
    #[setter]
    fn set_high(&mut self, high: i64) {
        self.inner.high = high;
    }
    #[getter]
    fn get_pretty_high(&self) -> f64 {
        self.inner.high_f64()
    }
    #[getter]

    fn get_low(&self) -> i64 {
        self.inner.low
    }
    #[setter]
    fn set_low(&mut self, low: i64) {
        self.inner.low = low;
    }
    #[getter]
    fn get_pretty_low(&self) -> f64 {
        self.inner.low_f64()
    }
    #[getter]

    fn get_close(&self) -> i64 {
        self.inner.close
    }
    #[setter]
    fn set_close(&mut self, close: i64) {
        self.inner.close = close;
    }
    #[getter]
    fn get_pretty_close(&self) -> f64 {
        self.inner.close_f64()
    }
    #[getter]

    fn get_volume(&self) -> u64 {
        self.inner.volume
    }
    #[setter]
    fn set_volume(&mut self, volume: u64) {
        self.inner.volume = volume;
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        OhlcvMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        OhlcvMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        OhlcvMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        OhlcvMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        OhlcvMsg::ordered_fields("")
    }
}

/// Convert bare [`OhlcvMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for OhlcvMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyOhlcvMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<OhlcvMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<OhlcvMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyOhlcvMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`StatusMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "StatusMsg")]
pub struct PyStatusMsg {
    pub(crate) inner: StatusMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyStatusMsg {
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
        ts_out = UNDEF_TIMESTAMP,
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
        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: StatusMsg {
                hd: RecordHeader::new::<StatusMsg>(
                    rtype::STATUS,
                    publisher_id,
                    instrument_id,
                    ts_event,
                ),
                ts_recv,
                action: action.unwrap_or_default() as u16,
                reason: reason.unwrap_or_default() as u16,
                trading_event: trading_event.unwrap_or_default() as u16,
                is_trading: is_trading.unwrap_or_default() as u8 as c_char,
                is_quoting: is_quoting.unwrap_or_default() as u8 as c_char,
                is_short_sell_restricted: is_short_sell_restricted.unwrap_or_default() as u8
                    as c_char,
                _reserved: Default::default(),
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<StatusMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<StatusMsg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]

    fn get_ts_recv(&self) -> u64 {
        self.inner.ts_recv
    }
    #[setter]
    fn set_ts_recv(&mut self, ts_recv: u64) {
        self.inner.ts_recv = ts_recv;
    }
    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.ts_recv)
    }
    #[getter]
    fn get_action<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .action()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| self.inner.action.into_bound_py_any(py))
    }
    #[setter]
    fn set_action(&mut self, action: StatusAction) {
        self.inner.action = action as u16;
    }
    #[getter]
    fn get_reason<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .reason()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| self.inner.reason.into_bound_py_any(py))
    }
    #[setter]
    fn set_reason(&mut self, reason: StatusReason) {
        self.inner.reason = reason as u16;
    }
    #[getter]
    fn get_trading_event<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .trading_event()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| self.inner.trading_event.into_bound_py_any(py))
    }
    #[setter]
    fn set_trading_event(&mut self, trading_event: TradingEvent) {
        self.inner.trading_event = trading_event as u16;
    }
    #[getter]
    fn get_is_trading(&self) -> Option<bool> {
        self.inner.is_trading()
    }
    #[getter]
    fn get_is_quoting(&self) -> Option<bool> {
        self.inner.is_quoting()
    }
    #[getter]
    fn get_is_short_sell_restricted(&self) -> Option<bool> {
        self.inner.is_short_sell_restricted()
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        StatusMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        StatusMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        StatusMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        StatusMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        StatusMsg::ordered_fields("")
    }
}

/// Convert bare [`StatusMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for StatusMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyStatusMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<StatusMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<StatusMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyStatusMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`InstrumentDefMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "InstrumentDefMsg")]
pub struct PyInstrumentDefMsg {
    pub(crate) inner: InstrumentDefMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyInstrumentDefMsg {
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
        inst_attrib_value = None,
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
        channel_id = None,
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
        ts_out = UNDEF_TIMESTAMP,
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
        inst_attrib_value: Option<i32>,
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
        channel_id: Option<u16>,
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
        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: InstrumentDefMsg {
                hd: RecordHeader::new::<InstrumentDefMsg>(
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
                inst_attrib_value: inst_attrib_value.unwrap_or(i32::MAX),
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
                channel_id: channel_id.unwrap_or(u16::MAX),
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
                user_defined_instrument: user_defined_instrument.unwrap_or_default() as u8
                    as c_char,
                contract_multiplier_unit: contract_multiplier_unit.unwrap_or(i8::MAX),
                flow_schedule_type: flow_schedule_type.unwrap_or(i8::MAX),
                tick_rule: tick_rule.unwrap_or(u8::MAX),
                leg_instrument_class: leg_instrument_class
                    .map(|leg_instrument_class| leg_instrument_class as u8 as c_char)
                    .unwrap_or_default(),
                leg_side: leg_side.unwrap_or_default() as u8 as c_char,
                _reserved: Default::default(),
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<InstrumentDefMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<InstrumentDefMsg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]

    fn get_ts_recv(&self) -> u64 {
        self.inner.ts_recv
    }
    #[setter]
    fn set_ts_recv(&mut self, ts_recv: u64) {
        self.inner.ts_recv = ts_recv;
    }
    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.ts_recv)
    }
    #[getter]

    fn get_min_price_increment(&self) -> i64 {
        self.inner.min_price_increment
    }
    #[setter]
    fn set_min_price_increment(&mut self, min_price_increment: i64) {
        self.inner.min_price_increment = min_price_increment;
    }
    #[getter]
    fn get_pretty_min_price_increment(&self) -> f64 {
        self.inner.min_price_increment_f64()
    }
    #[getter]

    fn get_display_factor(&self) -> i64 {
        self.inner.display_factor
    }
    #[setter]
    fn set_display_factor(&mut self, display_factor: i64) {
        self.inner.display_factor = display_factor;
    }
    #[getter]
    fn get_pretty_display_factor(&self) -> f64 {
        self.inner.display_factor_f64()
    }
    #[getter]

    fn get_expiration(&self) -> u64 {
        self.inner.expiration
    }
    #[setter]
    fn set_expiration(&mut self, expiration: u64) {
        self.inner.expiration = expiration;
    }
    #[getter]
    fn get_pretty_expiration<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.expiration)
    }
    #[getter]

    fn get_activation(&self) -> u64 {
        self.inner.activation
    }
    #[setter]
    fn set_activation(&mut self, activation: u64) {
        self.inner.activation = activation;
    }
    #[getter]
    fn get_pretty_activation<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.activation)
    }
    #[getter]

    fn get_high_limit_price(&self) -> i64 {
        self.inner.high_limit_price
    }
    #[setter]
    fn set_high_limit_price(&mut self, high_limit_price: i64) {
        self.inner.high_limit_price = high_limit_price;
    }
    #[getter]
    fn get_pretty_high_limit_price(&self) -> f64 {
        self.inner.high_limit_price_f64()
    }
    #[getter]

    fn get_low_limit_price(&self) -> i64 {
        self.inner.low_limit_price
    }
    #[setter]
    fn set_low_limit_price(&mut self, low_limit_price: i64) {
        self.inner.low_limit_price = low_limit_price;
    }
    #[getter]
    fn get_pretty_low_limit_price(&self) -> f64 {
        self.inner.low_limit_price_f64()
    }
    #[getter]

    fn get_max_price_variation(&self) -> i64 {
        self.inner.max_price_variation
    }
    #[setter]
    fn set_max_price_variation(&mut self, max_price_variation: i64) {
        self.inner.max_price_variation = max_price_variation;
    }
    #[getter]
    fn get_pretty_max_price_variation(&self) -> f64 {
        self.inner.max_price_variation_f64()
    }
    #[getter]

    fn get_unit_of_measure_qty(&self) -> i64 {
        self.inner.unit_of_measure_qty
    }
    #[setter]
    fn set_unit_of_measure_qty(&mut self, unit_of_measure_qty: i64) {
        self.inner.unit_of_measure_qty = unit_of_measure_qty;
    }
    #[getter]
    fn get_pretty_unit_of_measure_qty(&self) -> f64 {
        self.inner.unit_of_measure_qty_f64()
    }
    #[getter]

    fn get_min_price_increment_amount(&self) -> i64 {
        self.inner.min_price_increment_amount
    }
    #[setter]
    fn set_min_price_increment_amount(&mut self, min_price_increment_amount: i64) {
        self.inner.min_price_increment_amount = min_price_increment_amount;
    }
    #[getter]
    fn get_pretty_min_price_increment_amount(&self) -> f64 {
        self.inner.min_price_increment_amount_f64()
    }
    #[getter]

    fn get_price_ratio(&self) -> i64 {
        self.inner.price_ratio
    }
    #[setter]
    fn set_price_ratio(&mut self, price_ratio: i64) {
        self.inner.price_ratio = price_ratio;
    }
    #[getter]
    fn get_pretty_price_ratio(&self) -> f64 {
        self.inner.price_ratio_f64()
    }
    #[getter]

    fn get_strike_price(&self) -> i64 {
        self.inner.strike_price
    }
    #[setter]
    fn set_strike_price(&mut self, strike_price: i64) {
        self.inner.strike_price = strike_price;
    }
    #[getter]
    fn get_pretty_strike_price(&self) -> f64 {
        self.inner.strike_price_f64()
    }
    #[getter]

    fn get_raw_instrument_id(&self) -> u64 {
        self.inner.raw_instrument_id
    }
    #[setter]
    fn set_raw_instrument_id(&mut self, raw_instrument_id: u64) {
        self.inner.raw_instrument_id = raw_instrument_id;
    }
    #[getter]

    fn get_leg_price(&self) -> i64 {
        self.inner.leg_price
    }
    #[setter]
    fn set_leg_price(&mut self, leg_price: i64) {
        self.inner.leg_price = leg_price;
    }
    #[getter]
    fn get_pretty_leg_price(&self) -> f64 {
        self.inner.leg_price_f64()
    }
    #[getter]

    fn get_leg_delta(&self) -> i64 {
        self.inner.leg_delta
    }
    #[setter]
    fn set_leg_delta(&mut self, leg_delta: i64) {
        self.inner.leg_delta = leg_delta;
    }
    #[getter]
    fn get_pretty_leg_delta(&self) -> f64 {
        self.inner.leg_delta_f64()
    }
    #[getter]

    fn get_inst_attrib_value(&self) -> i32 {
        self.inner.inst_attrib_value
    }
    #[setter]
    fn set_inst_attrib_value(&mut self, inst_attrib_value: i32) {
        self.inner.inst_attrib_value = inst_attrib_value;
    }
    #[getter]

    fn get_underlying_id(&self) -> u32 {
        self.inner.underlying_id
    }
    #[setter]
    fn set_underlying_id(&mut self, underlying_id: u32) {
        self.inner.underlying_id = underlying_id;
    }
    #[getter]

    fn get_market_depth_implied(&self) -> i32 {
        self.inner.market_depth_implied
    }
    #[setter]
    fn set_market_depth_implied(&mut self, market_depth_implied: i32) {
        self.inner.market_depth_implied = market_depth_implied;
    }
    #[getter]

    fn get_market_depth(&self) -> i32 {
        self.inner.market_depth
    }
    #[setter]
    fn set_market_depth(&mut self, market_depth: i32) {
        self.inner.market_depth = market_depth;
    }
    #[getter]

    fn get_market_segment_id(&self) -> u32 {
        self.inner.market_segment_id
    }
    #[setter]
    fn set_market_segment_id(&mut self, market_segment_id: u32) {
        self.inner.market_segment_id = market_segment_id;
    }
    #[getter]

    fn get_max_trade_vol(&self) -> u32 {
        self.inner.max_trade_vol
    }
    #[setter]
    fn set_max_trade_vol(&mut self, max_trade_vol: u32) {
        self.inner.max_trade_vol = max_trade_vol;
    }
    #[getter]

    fn get_min_lot_size(&self) -> i32 {
        self.inner.min_lot_size
    }
    #[setter]
    fn set_min_lot_size(&mut self, min_lot_size: i32) {
        self.inner.min_lot_size = min_lot_size;
    }
    #[getter]

    fn get_min_lot_size_block(&self) -> i32 {
        self.inner.min_lot_size_block
    }
    #[setter]
    fn set_min_lot_size_block(&mut self, min_lot_size_block: i32) {
        self.inner.min_lot_size_block = min_lot_size_block;
    }
    #[getter]

    fn get_min_lot_size_round_lot(&self) -> i32 {
        self.inner.min_lot_size_round_lot
    }
    #[setter]
    fn set_min_lot_size_round_lot(&mut self, min_lot_size_round_lot: i32) {
        self.inner.min_lot_size_round_lot = min_lot_size_round_lot;
    }
    #[getter]

    fn get_min_trade_vol(&self) -> u32 {
        self.inner.min_trade_vol
    }
    #[setter]
    fn set_min_trade_vol(&mut self, min_trade_vol: u32) {
        self.inner.min_trade_vol = min_trade_vol;
    }
    #[getter]

    fn get_contract_multiplier(&self) -> i32 {
        self.inner.contract_multiplier
    }
    #[setter]
    fn set_contract_multiplier(&mut self, contract_multiplier: i32) {
        self.inner.contract_multiplier = contract_multiplier;
    }
    #[getter]

    fn get_decay_quantity(&self) -> i32 {
        self.inner.decay_quantity
    }
    #[setter]
    fn set_decay_quantity(&mut self, decay_quantity: i32) {
        self.inner.decay_quantity = decay_quantity;
    }
    #[getter]

    fn get_original_contract_size(&self) -> i32 {
        self.inner.original_contract_size
    }
    #[setter]
    fn set_original_contract_size(&mut self, original_contract_size: i32) {
        self.inner.original_contract_size = original_contract_size;
    }
    #[getter]

    fn get_leg_instrument_id(&self) -> u32 {
        self.inner.leg_instrument_id
    }
    #[setter]
    fn set_leg_instrument_id(&mut self, leg_instrument_id: u32) {
        self.inner.leg_instrument_id = leg_instrument_id;
    }
    #[getter]

    fn get_leg_ratio_price_numerator(&self) -> i32 {
        self.inner.leg_ratio_price_numerator
    }
    #[setter]
    fn set_leg_ratio_price_numerator(&mut self, leg_ratio_price_numerator: i32) {
        self.inner.leg_ratio_price_numerator = leg_ratio_price_numerator;
    }
    #[getter]

    fn get_leg_ratio_price_denominator(&self) -> i32 {
        self.inner.leg_ratio_price_denominator
    }
    #[setter]
    fn set_leg_ratio_price_denominator(&mut self, leg_ratio_price_denominator: i32) {
        self.inner.leg_ratio_price_denominator = leg_ratio_price_denominator;
    }
    #[getter]

    fn get_leg_ratio_qty_numerator(&self) -> i32 {
        self.inner.leg_ratio_qty_numerator
    }
    #[setter]
    fn set_leg_ratio_qty_numerator(&mut self, leg_ratio_qty_numerator: i32) {
        self.inner.leg_ratio_qty_numerator = leg_ratio_qty_numerator;
    }
    #[getter]

    fn get_leg_ratio_qty_denominator(&self) -> i32 {
        self.inner.leg_ratio_qty_denominator
    }
    #[setter]
    fn set_leg_ratio_qty_denominator(&mut self, leg_ratio_qty_denominator: i32) {
        self.inner.leg_ratio_qty_denominator = leg_ratio_qty_denominator;
    }
    #[getter]

    fn get_leg_underlying_id(&self) -> u32 {
        self.inner.leg_underlying_id
    }
    #[setter]
    fn set_leg_underlying_id(&mut self, leg_underlying_id: u32) {
        self.inner.leg_underlying_id = leg_underlying_id;
    }
    #[getter]

    fn get_appl_id(&self) -> i16 {
        self.inner.appl_id
    }
    #[setter]
    fn set_appl_id(&mut self, appl_id: i16) {
        self.inner.appl_id = appl_id;
    }
    #[getter]

    fn get_maturity_year(&self) -> u16 {
        self.inner.maturity_year
    }
    #[setter]
    fn set_maturity_year(&mut self, maturity_year: u16) {
        self.inner.maturity_year = maturity_year;
    }
    #[getter]

    fn get_decay_start_date(&self) -> u16 {
        self.inner.decay_start_date
    }
    #[setter]
    fn set_decay_start_date(&mut self, decay_start_date: u16) {
        self.inner.decay_start_date = decay_start_date;
    }
    #[getter]

    fn get_channel_id(&self) -> u16 {
        self.inner.channel_id
    }
    #[setter]
    fn set_channel_id(&mut self, channel_id: u16) {
        self.inner.channel_id = channel_id;
    }
    #[getter]

    fn get_leg_count(&self) -> u16 {
        self.inner.leg_count
    }
    #[setter]
    fn set_leg_count(&mut self, leg_count: u16) {
        self.inner.leg_count = leg_count;
    }
    #[getter]

    fn get_leg_index(&self) -> u16 {
        self.inner.leg_index
    }
    #[setter]
    fn set_leg_index(&mut self, leg_index: u16) {
        self.inner.leg_index = leg_index;
    }
    #[getter]
    fn get_currency(&self) -> PyResult<&str> {
        Ok(self.inner.currency()?)
    }
    #[getter]
    fn get_settl_currency(&self) -> PyResult<&str> {
        Ok(self.inner.settl_currency()?)
    }
    #[getter]
    fn get_secsubtype(&self) -> PyResult<&str> {
        Ok(self.inner.secsubtype()?)
    }
    #[getter]
    fn get_raw_symbol(&self) -> PyResult<&str> {
        Ok(self.inner.raw_symbol()?)
    }
    #[getter]
    fn get_group(&self) -> PyResult<&str> {
        Ok(self.inner.group()?)
    }
    #[getter]
    fn get_exchange(&self) -> PyResult<&str> {
        Ok(self.inner.exchange()?)
    }
    #[getter]
    fn get_asset(&self) -> PyResult<&str> {
        Ok(self.inner.asset()?)
    }
    #[getter]
    fn get_cfi(&self) -> PyResult<&str> {
        Ok(self.inner.cfi()?)
    }
    #[getter]
    fn get_security_type(&self) -> PyResult<&str> {
        Ok(self.inner.security_type()?)
    }
    #[getter]
    fn get_unit_of_measure(&self) -> PyResult<&str> {
        Ok(self.inner.unit_of_measure()?)
    }
    #[getter]
    fn get_underlying(&self) -> PyResult<&str> {
        Ok(self.inner.underlying()?)
    }
    #[getter]
    fn get_strike_price_currency(&self) -> PyResult<&str> {
        Ok(self.inner.strike_price_currency()?)
    }
    #[getter]
    fn get_leg_raw_symbol(&self) -> PyResult<&str> {
        Ok(self.inner.leg_raw_symbol()?)
    }
    #[getter]
    fn get_instrument_class<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .instrument_class()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.instrument_class as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_instrument_class(&mut self, instrument_class: InstrumentClass) {
        self.inner.instrument_class = instrument_class as u8 as c_char;
    }
    #[getter]
    fn get_match_algorithm<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .match_algorithm()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.match_algorithm as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_match_algorithm(&mut self, match_algorithm: MatchAlgorithm) {
        self.inner.match_algorithm = match_algorithm as u8 as c_char;
    }
    #[getter]

    fn get_main_fraction(&self) -> u8 {
        self.inner.main_fraction
    }
    #[setter]
    fn set_main_fraction(&mut self, main_fraction: u8) {
        self.inner.main_fraction = main_fraction;
    }
    #[getter]

    fn get_price_display_format(&self) -> u8 {
        self.inner.price_display_format
    }
    #[setter]
    fn set_price_display_format(&mut self, price_display_format: u8) {
        self.inner.price_display_format = price_display_format;
    }
    #[getter]

    fn get_sub_fraction(&self) -> u8 {
        self.inner.sub_fraction
    }
    #[setter]
    fn set_sub_fraction(&mut self, sub_fraction: u8) {
        self.inner.sub_fraction = sub_fraction;
    }
    #[getter]

    fn get_underlying_product(&self) -> u8 {
        self.inner.underlying_product
    }
    #[setter]
    fn set_underlying_product(&mut self, underlying_product: u8) {
        self.inner.underlying_product = underlying_product;
    }
    #[getter]
    fn get_security_update_action<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .security_update_action()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| {
                (self.inner.security_update_action as u8 as char).into_bound_py_any(py)
            })
    }
    #[setter]
    fn set_security_update_action(&mut self, security_update_action: SecurityUpdateAction) {
        self.inner.security_update_action = security_update_action as u8 as c_char;
    }
    #[getter]

    fn get_maturity_month(&self) -> u8 {
        self.inner.maturity_month
    }
    #[setter]
    fn set_maturity_month(&mut self, maturity_month: u8) {
        self.inner.maturity_month = maturity_month;
    }
    #[getter]

    fn get_maturity_day(&self) -> u8 {
        self.inner.maturity_day
    }
    #[setter]
    fn set_maturity_day(&mut self, maturity_day: u8) {
        self.inner.maturity_day = maturity_day;
    }
    #[getter]

    fn get_maturity_week(&self) -> u8 {
        self.inner.maturity_week
    }
    #[setter]
    fn set_maturity_week(&mut self, maturity_week: u8) {
        self.inner.maturity_week = maturity_week;
    }
    #[getter]
    fn get_user_defined_instrument<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .user_defined_instrument()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| {
                (self.inner.user_defined_instrument as u8 as char).into_bound_py_any(py)
            })
    }
    #[setter]
    fn set_user_defined_instrument(&mut self, user_defined_instrument: UserDefinedInstrument) {
        self.inner.user_defined_instrument = user_defined_instrument as u8 as c_char;
    }
    #[getter]

    fn get_contract_multiplier_unit(&self) -> i8 {
        self.inner.contract_multiplier_unit
    }
    #[setter]
    fn set_contract_multiplier_unit(&mut self, contract_multiplier_unit: i8) {
        self.inner.contract_multiplier_unit = contract_multiplier_unit;
    }
    #[getter]

    fn get_flow_schedule_type(&self) -> i8 {
        self.inner.flow_schedule_type
    }
    #[setter]
    fn set_flow_schedule_type(&mut self, flow_schedule_type: i8) {
        self.inner.flow_schedule_type = flow_schedule_type;
    }
    #[getter]

    fn get_tick_rule(&self) -> u8 {
        self.inner.tick_rule
    }
    #[setter]
    fn set_tick_rule(&mut self, tick_rule: u8) {
        self.inner.tick_rule = tick_rule;
    }
    #[getter]
    fn get_leg_instrument_class<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .leg_instrument_class()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| {
                (self.inner.leg_instrument_class as u8 as char).into_bound_py_any(py)
            })
    }
    #[setter]
    fn set_leg_instrument_class(&mut self, leg_instrument_class: InstrumentClass) {
        self.inner.leg_instrument_class = leg_instrument_class as u8 as c_char;
    }
    #[getter]
    fn get_leg_side<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .leg_side()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.leg_side as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_leg_side(&mut self, leg_side: Side) {
        self.inner.leg_side = leg_side as u8 as c_char;
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        InstrumentDefMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        InstrumentDefMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        InstrumentDefMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        InstrumentDefMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        InstrumentDefMsg::ordered_fields("")
    }
}

/// Convert bare [`InstrumentDefMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for InstrumentDefMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyInstrumentDefMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<InstrumentDefMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<InstrumentDefMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyInstrumentDefMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`ImbalanceMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "ImbalanceMsg")]
pub struct PyImbalanceMsg {
    pub(crate) inner: ImbalanceMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyImbalanceMsg {
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
        ts_out = UNDEF_TIMESTAMP,
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
        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: ImbalanceMsg {
                hd: RecordHeader::new::<ImbalanceMsg>(
                    rtype::IMBALANCE,
                    publisher_id,
                    instrument_id,
                    ts_event,
                ),
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
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<ImbalanceMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<ImbalanceMsg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]

    fn get_ts_recv(&self) -> u64 {
        self.inner.ts_recv
    }
    #[setter]
    fn set_ts_recv(&mut self, ts_recv: u64) {
        self.inner.ts_recv = ts_recv;
    }
    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.ts_recv)
    }
    #[getter]

    fn get_ref_price(&self) -> i64 {
        self.inner.ref_price
    }
    #[setter]
    fn set_ref_price(&mut self, ref_price: i64) {
        self.inner.ref_price = ref_price;
    }
    #[getter]
    fn get_pretty_ref_price(&self) -> f64 {
        self.inner.ref_price_f64()
    }
    #[getter]

    fn get_auction_time(&self) -> u64 {
        self.inner.auction_time
    }
    #[setter]
    fn set_auction_time(&mut self, auction_time: u64) {
        self.inner.auction_time = auction_time;
    }
    #[getter]
    fn get_pretty_auction_time<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.auction_time)
    }
    #[getter]

    fn get_cont_book_clr_price(&self) -> i64 {
        self.inner.cont_book_clr_price
    }
    #[setter]
    fn set_cont_book_clr_price(&mut self, cont_book_clr_price: i64) {
        self.inner.cont_book_clr_price = cont_book_clr_price;
    }
    #[getter]
    fn get_pretty_cont_book_clr_price(&self) -> f64 {
        self.inner.cont_book_clr_price_f64()
    }
    #[getter]

    fn get_auct_interest_clr_price(&self) -> i64 {
        self.inner.auct_interest_clr_price
    }
    #[setter]
    fn set_auct_interest_clr_price(&mut self, auct_interest_clr_price: i64) {
        self.inner.auct_interest_clr_price = auct_interest_clr_price;
    }
    #[getter]
    fn get_pretty_auct_interest_clr_price(&self) -> f64 {
        self.inner.auct_interest_clr_price_f64()
    }
    #[getter]

    fn get_ssr_filling_price(&self) -> i64 {
        self.inner.ssr_filling_price
    }
    #[setter]
    fn set_ssr_filling_price(&mut self, ssr_filling_price: i64) {
        self.inner.ssr_filling_price = ssr_filling_price;
    }
    #[getter]
    fn get_pretty_ssr_filling_price(&self) -> f64 {
        self.inner.ssr_filling_price_f64()
    }
    #[getter]

    fn get_ind_match_price(&self) -> i64 {
        self.inner.ind_match_price
    }
    #[setter]
    fn set_ind_match_price(&mut self, ind_match_price: i64) {
        self.inner.ind_match_price = ind_match_price;
    }
    #[getter]
    fn get_pretty_ind_match_price(&self) -> f64 {
        self.inner.ind_match_price_f64()
    }
    #[getter]

    fn get_upper_collar(&self) -> i64 {
        self.inner.upper_collar
    }
    #[setter]
    fn set_upper_collar(&mut self, upper_collar: i64) {
        self.inner.upper_collar = upper_collar;
    }
    #[getter]
    fn get_pretty_upper_collar(&self) -> f64 {
        self.inner.upper_collar_f64()
    }
    #[getter]

    fn get_lower_collar(&self) -> i64 {
        self.inner.lower_collar
    }
    #[setter]
    fn set_lower_collar(&mut self, lower_collar: i64) {
        self.inner.lower_collar = lower_collar;
    }
    #[getter]
    fn get_pretty_lower_collar(&self) -> f64 {
        self.inner.lower_collar_f64()
    }
    #[getter]

    fn get_paired_qty(&self) -> u32 {
        self.inner.paired_qty
    }
    #[setter]
    fn set_paired_qty(&mut self, paired_qty: u32) {
        self.inner.paired_qty = paired_qty;
    }
    #[getter]

    fn get_total_imbalance_qty(&self) -> u32 {
        self.inner.total_imbalance_qty
    }
    #[setter]
    fn set_total_imbalance_qty(&mut self, total_imbalance_qty: u32) {
        self.inner.total_imbalance_qty = total_imbalance_qty;
    }
    #[getter]

    fn get_market_imbalance_qty(&self) -> u32 {
        self.inner.market_imbalance_qty
    }
    #[setter]
    fn set_market_imbalance_qty(&mut self, market_imbalance_qty: u32) {
        self.inner.market_imbalance_qty = market_imbalance_qty;
    }
    #[getter]

    fn get_unpaired_qty(&self) -> u32 {
        self.inner.unpaired_qty
    }
    #[setter]
    fn set_unpaired_qty(&mut self, unpaired_qty: u32) {
        self.inner.unpaired_qty = unpaired_qty;
    }
    #[getter]
    fn get_auction_type(&self) -> char {
        self.inner.auction_type as u8 as char
    }
    #[setter]
    fn set_auction_type(&mut self, auction_type: char) -> PyResult<()> {
        self.inner.auction_type = char_to_c_char(auction_type)?;
        Ok(())
    }
    #[getter]
    fn get_side<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .side()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.side as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_side(&mut self, side: Side) {
        self.inner.side = side as u8 as c_char;
    }
    #[getter]

    fn get_auction_status(&self) -> u8 {
        self.inner.auction_status
    }
    #[setter]
    fn set_auction_status(&mut self, auction_status: u8) {
        self.inner.auction_status = auction_status;
    }
    #[getter]

    fn get_freeze_status(&self) -> u8 {
        self.inner.freeze_status
    }
    #[setter]
    fn set_freeze_status(&mut self, freeze_status: u8) {
        self.inner.freeze_status = freeze_status;
    }
    #[getter]

    fn get_num_extensions(&self) -> u8 {
        self.inner.num_extensions
    }
    #[setter]
    fn set_num_extensions(&mut self, num_extensions: u8) {
        self.inner.num_extensions = num_extensions;
    }
    #[getter]
    fn get_unpaired_side<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .unpaired_side()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.unpaired_side as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_unpaired_side(&mut self, unpaired_side: Side) {
        self.inner.unpaired_side = unpaired_side as u8 as c_char;
    }
    #[getter]
    fn get_significant_imbalance(&self) -> char {
        self.inner.significant_imbalance as u8 as char
    }
    #[setter]
    fn set_significant_imbalance(&mut self, significant_imbalance: char) -> PyResult<()> {
        self.inner.significant_imbalance = char_to_c_char(significant_imbalance)?;
        Ok(())
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        ImbalanceMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        ImbalanceMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        ImbalanceMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        ImbalanceMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        ImbalanceMsg::ordered_fields("")
    }
}

/// Convert bare [`ImbalanceMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for ImbalanceMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyImbalanceMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<ImbalanceMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<ImbalanceMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyImbalanceMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`StatMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "StatMsg")]
pub struct PyStatMsg {
    pub(crate) inner: StatMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyStatMsg {
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
        channel_id = None,
        update_action = None,
        stat_flags = 0,
        ts_out = UNDEF_TIMESTAMP,
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
        channel_id: Option<u16>,
        update_action: Option<StatUpdateAction>,
        stat_flags: u8,
        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: StatMsg {
                hd: RecordHeader::new::<StatMsg>(
                    rtype::STATISTICS,
                    publisher_id,
                    instrument_id,
                    ts_event,
                ),
                ts_recv,
                ts_ref,
                price,
                quantity,
                sequence,
                ts_in_delta,
                stat_type: stat_type as u16,
                channel_id: channel_id.unwrap_or(u16::MAX),
                update_action: update_action.unwrap_or_default() as u8,
                stat_flags,
                _reserved: Default::default(),
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<StatMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<StatMsg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]

    fn get_ts_recv(&self) -> u64 {
        self.inner.ts_recv
    }
    #[setter]
    fn set_ts_recv(&mut self, ts_recv: u64) {
        self.inner.ts_recv = ts_recv;
    }
    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.ts_recv)
    }
    #[getter]

    fn get_ts_ref(&self) -> u64 {
        self.inner.ts_ref
    }
    #[setter]
    fn set_ts_ref(&mut self, ts_ref: u64) {
        self.inner.ts_ref = ts_ref;
    }
    #[getter]
    fn get_pretty_ts_ref<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.ts_ref)
    }
    #[getter]

    fn get_price(&self) -> i64 {
        self.inner.price
    }
    #[setter]
    fn set_price(&mut self, price: i64) {
        self.inner.price = price;
    }
    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.inner.price_f64()
    }
    #[getter]

    fn get_quantity(&self) -> i64 {
        self.inner.quantity
    }
    #[setter]
    fn set_quantity(&mut self, quantity: i64) {
        self.inner.quantity = quantity;
    }
    #[getter]

    fn get_sequence(&self) -> u32 {
        self.inner.sequence
    }
    #[setter]
    fn set_sequence(&mut self, sequence: u32) {
        self.inner.sequence = sequence;
    }
    #[getter]

    fn get_ts_in_delta(&self) -> i32 {
        self.inner.ts_in_delta
    }
    #[setter]
    fn set_ts_in_delta(&mut self, ts_in_delta: i32) {
        self.inner.ts_in_delta = ts_in_delta;
    }
    #[getter]
    fn get_stat_type<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .stat_type()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| self.inner.stat_type.into_bound_py_any(py))
    }
    #[setter]
    fn set_stat_type(&mut self, stat_type: StatType) {
        self.inner.stat_type = stat_type as u16;
    }
    #[getter]

    fn get_channel_id(&self) -> u16 {
        self.inner.channel_id
    }
    #[setter]
    fn set_channel_id(&mut self, channel_id: u16) {
        self.inner.channel_id = channel_id;
    }
    #[getter]
    fn get_update_action<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .update_action()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| self.inner.update_action.into_bound_py_any(py))
    }
    #[setter]
    fn set_update_action(&mut self, update_action: StatUpdateAction) {
        self.inner.update_action = update_action as u8;
    }
    #[getter]

    fn get_stat_flags(&self) -> u8 {
        self.inner.stat_flags
    }
    #[setter]
    fn set_stat_flags(&mut self, stat_flags: u8) {
        self.inner.stat_flags = stat_flags;
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        StatMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        StatMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        StatMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        StatMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        StatMsg::ordered_fields("")
    }
}

/// Convert bare [`StatMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for StatMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyStatMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<StatMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<StatMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyStatMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`ErrorMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "ErrorMsg")]
pub struct PyErrorMsg {
    pub(crate) inner: ErrorMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyErrorMsg {
    #[new]
    #[pyo3(signature = (ts_event, err, is_last = true, code = None, ts_out = UNDEF_TIMESTAMP))]
    fn py_new(
        ts_event: u64,
        err: &str,
        is_last: bool,
        code: Option<ErrorCode>,
        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: ErrorMsg::new(ts_event, code, err, is_last),
            ts_out,
        };
        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<ErrorMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<ErrorMsg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]
    fn get_err(&self) -> PyResult<&str> {
        Ok(self.inner.err()?)
    }
    #[getter]
    fn get_code<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .code()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| self.inner.code.into_bound_py_any(py))
    }
    #[setter]
    fn set_code(&mut self, code: ErrorCode) {
        self.inner.code = code as u8;
    }
    #[getter]

    fn get_is_last(&self) -> u8 {
        self.inner.is_last
    }
    #[setter]
    fn set_is_last(&mut self, is_last: u8) {
        self.inner.is_last = is_last;
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        ErrorMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        ErrorMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        ErrorMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        ErrorMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        ErrorMsg::ordered_fields("")
    }
}

/// Convert bare [`ErrorMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for ErrorMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyErrorMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<ErrorMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<ErrorMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyErrorMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`SymbolMappingMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "SymbolMappingMsg")]
pub struct PySymbolMappingMsg {
    pub(crate) inner: SymbolMappingMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PySymbolMappingMsg {
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
        ts_out = UNDEF_TIMESTAMP,
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

        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: SymbolMappingMsg {
                hd: RecordHeader::new::<SymbolMappingMsg>(
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
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<SymbolMappingMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<SymbolMappingMsg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]
    fn get_stype_in<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .stype_in()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| self.inner.stype_in.into_bound_py_any(py))
    }
    #[setter]
    fn set_stype_in(&mut self, stype_in: SType) {
        self.inner.stype_in = stype_in as u8;
    }
    #[getter]
    fn get_stype_in_symbol(&self) -> PyResult<&str> {
        Ok(self.inner.stype_in_symbol()?)
    }
    #[getter]
    fn get_stype_out<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .stype_out()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| self.inner.stype_out.into_bound_py_any(py))
    }
    #[setter]
    fn set_stype_out(&mut self, stype_out: SType) {
        self.inner.stype_out = stype_out as u8;
    }
    #[getter]
    fn get_stype_out_symbol(&self) -> PyResult<&str> {
        Ok(self.inner.stype_out_symbol()?)
    }
    #[getter]

    fn get_start_ts(&self) -> u64 {
        self.inner.start_ts
    }
    #[setter]
    fn set_start_ts(&mut self, start_ts: u64) {
        self.inner.start_ts = start_ts;
    }
    #[getter]
    fn get_pretty_start_ts<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.start_ts)
    }
    #[getter]

    fn get_end_ts(&self) -> u64 {
        self.inner.end_ts
    }
    #[setter]
    fn set_end_ts(&mut self, end_ts: u64) {
        self.inner.end_ts = end_ts;
    }
    #[getter]
    fn get_pretty_end_ts<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.end_ts)
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        SymbolMappingMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        SymbolMappingMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        SymbolMappingMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        SymbolMappingMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        SymbolMappingMsg::ordered_fields("")
    }
}

/// Convert bare [`SymbolMappingMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for SymbolMappingMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PySymbolMappingMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<SymbolMappingMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<SymbolMappingMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PySymbolMappingMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`SystemMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "SystemMsg")]
pub struct PySystemMsg {
    pub(crate) inner: SystemMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PySystemMsg {
    #[new]
    #[pyo3(signature = (ts_event, msg, code = None, ts_out = UNDEF_TIMESTAMP))]
    fn py_new(ts_event: u64, msg: &str, code: Option<SystemCode>, ts_out: u64) -> PyResult<Self> {
        let mut res = Self {
            inner: SystemMsg::new(ts_event, code, msg)?,
            ts_out,
        };
        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<SystemMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<SystemMsg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[pyo3(name = "is_heartbeat")]
    fn py_is_heartbeat(&self) -> bool {
        self.inner.is_heartbeat()
    }

    #[getter]
    fn get_msg(&self) -> PyResult<&str> {
        Ok(self.inner.msg()?)
    }
    #[getter]
    fn get_code<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .code()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| self.inner.code.into_bound_py_any(py))
    }
    #[setter]
    fn set_code(&mut self, code: SystemCode) {
        self.inner.code = code as u8;
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        SystemMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        SystemMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        SystemMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        SystemMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        SystemMsg::ordered_fields("")
    }
}

/// Convert bare [`SystemMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for SystemMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PySystemMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<SystemMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<SystemMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PySystemMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`v1::ErrorMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "ErrorMsgV1")]
pub struct PyV1ErrorMsg {
    pub(crate) inner: v1::ErrorMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyV1ErrorMsg {
    #[new]
    fn py_new(ts_event: u64, err: &str) -> PyResult<Self> {
        Ok(Self {
            inner: v1::ErrorMsg::new(ts_event, err),
            ts_out: UNDEF_TIMESTAMP,
        })
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<v1::ErrorMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<v1::ErrorMsg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]
    fn get_err(&self) -> PyResult<&str> {
        Ok(self.inner.err()?)
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        v1::ErrorMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        v1::ErrorMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        v1::ErrorMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        v1::ErrorMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        v1::ErrorMsg::ordered_fields("")
    }
}

/// Convert bare [`v1::ErrorMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for v1::ErrorMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyV1ErrorMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<v1::ErrorMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<v1::ErrorMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyV1ErrorMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`v1::InstrumentDefMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "InstrumentDefMsgV1")]
pub struct PyV1InstrumentDefMsg {
    pub(crate) inner: v1::InstrumentDefMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyV1InstrumentDefMsg {
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
        ts_out = UNDEF_TIMESTAMP,
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

        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: v1::InstrumentDefMsg {
                hd: RecordHeader::new::<v1::InstrumentDefMsg>(
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
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<v1::InstrumentDefMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length = (std::mem::size_of::<v1::InstrumentDefMsg>()
                / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]

    fn get_ts_recv(&self) -> u64 {
        self.inner.ts_recv
    }
    #[setter]
    fn set_ts_recv(&mut self, ts_recv: u64) {
        self.inner.ts_recv = ts_recv;
    }
    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.ts_recv)
    }
    #[getter]

    fn get_min_price_increment(&self) -> i64 {
        self.inner.min_price_increment
    }
    #[setter]
    fn set_min_price_increment(&mut self, min_price_increment: i64) {
        self.inner.min_price_increment = min_price_increment;
    }
    #[getter]
    fn get_pretty_min_price_increment(&self) -> f64 {
        self.inner.min_price_increment_f64()
    }
    #[getter]

    fn get_display_factor(&self) -> i64 {
        self.inner.display_factor
    }
    #[setter]
    fn set_display_factor(&mut self, display_factor: i64) {
        self.inner.display_factor = display_factor;
    }
    #[getter]
    fn get_pretty_display_factor(&self) -> f64 {
        self.inner.display_factor_f64()
    }
    #[getter]

    fn get_expiration(&self) -> u64 {
        self.inner.expiration
    }
    #[setter]
    fn set_expiration(&mut self, expiration: u64) {
        self.inner.expiration = expiration;
    }
    #[getter]
    fn get_pretty_expiration<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.expiration)
    }
    #[getter]

    fn get_activation(&self) -> u64 {
        self.inner.activation
    }
    #[setter]
    fn set_activation(&mut self, activation: u64) {
        self.inner.activation = activation;
    }
    #[getter]
    fn get_pretty_activation<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.activation)
    }
    #[getter]

    fn get_high_limit_price(&self) -> i64 {
        self.inner.high_limit_price
    }
    #[setter]
    fn set_high_limit_price(&mut self, high_limit_price: i64) {
        self.inner.high_limit_price = high_limit_price;
    }
    #[getter]
    fn get_pretty_high_limit_price(&self) -> f64 {
        self.inner.high_limit_price_f64()
    }
    #[getter]

    fn get_low_limit_price(&self) -> i64 {
        self.inner.low_limit_price
    }
    #[setter]
    fn set_low_limit_price(&mut self, low_limit_price: i64) {
        self.inner.low_limit_price = low_limit_price;
    }
    #[getter]
    fn get_pretty_low_limit_price(&self) -> f64 {
        self.inner.low_limit_price_f64()
    }
    #[getter]

    fn get_max_price_variation(&self) -> i64 {
        self.inner.max_price_variation
    }
    #[setter]
    fn set_max_price_variation(&mut self, max_price_variation: i64) {
        self.inner.max_price_variation = max_price_variation;
    }
    #[getter]
    fn get_pretty_max_price_variation(&self) -> f64 {
        self.inner.max_price_variation_f64()
    }
    #[getter]

    fn get_trading_reference_price(&self) -> i64 {
        self.inner.trading_reference_price
    }
    #[setter]
    fn set_trading_reference_price(&mut self, trading_reference_price: i64) {
        self.inner.trading_reference_price = trading_reference_price;
    }
    #[getter]
    fn get_pretty_trading_reference_price(&self) -> f64 {
        self.inner.trading_reference_price_f64()
    }
    #[getter]

    fn get_unit_of_measure_qty(&self) -> i64 {
        self.inner.unit_of_measure_qty
    }
    #[setter]
    fn set_unit_of_measure_qty(&mut self, unit_of_measure_qty: i64) {
        self.inner.unit_of_measure_qty = unit_of_measure_qty;
    }
    #[getter]
    fn get_pretty_unit_of_measure_qty(&self) -> f64 {
        self.inner.unit_of_measure_qty_f64()
    }
    #[getter]

    fn get_min_price_increment_amount(&self) -> i64 {
        self.inner.min_price_increment_amount
    }
    #[setter]
    fn set_min_price_increment_amount(&mut self, min_price_increment_amount: i64) {
        self.inner.min_price_increment_amount = min_price_increment_amount;
    }
    #[getter]
    fn get_pretty_min_price_increment_amount(&self) -> f64 {
        self.inner.min_price_increment_amount_f64()
    }
    #[getter]

    fn get_price_ratio(&self) -> i64 {
        self.inner.price_ratio
    }
    #[setter]
    fn set_price_ratio(&mut self, price_ratio: i64) {
        self.inner.price_ratio = price_ratio;
    }
    #[getter]
    fn get_pretty_price_ratio(&self) -> f64 {
        self.inner.price_ratio_f64()
    }
    #[getter]

    fn get_inst_attrib_value(&self) -> i32 {
        self.inner.inst_attrib_value
    }
    #[setter]
    fn set_inst_attrib_value(&mut self, inst_attrib_value: i32) {
        self.inner.inst_attrib_value = inst_attrib_value;
    }
    #[getter]

    fn get_underlying_id(&self) -> u32 {
        self.inner.underlying_id
    }
    #[setter]
    fn set_underlying_id(&mut self, underlying_id: u32) {
        self.inner.underlying_id = underlying_id;
    }
    #[getter]

    fn get_raw_instrument_id(&self) -> u32 {
        self.inner.raw_instrument_id
    }
    #[setter]
    fn set_raw_instrument_id(&mut self, raw_instrument_id: u32) {
        self.inner.raw_instrument_id = raw_instrument_id;
    }
    #[getter]

    fn get_market_depth_implied(&self) -> i32 {
        self.inner.market_depth_implied
    }
    #[setter]
    fn set_market_depth_implied(&mut self, market_depth_implied: i32) {
        self.inner.market_depth_implied = market_depth_implied;
    }
    #[getter]

    fn get_market_depth(&self) -> i32 {
        self.inner.market_depth
    }
    #[setter]
    fn set_market_depth(&mut self, market_depth: i32) {
        self.inner.market_depth = market_depth;
    }
    #[getter]

    fn get_market_segment_id(&self) -> u32 {
        self.inner.market_segment_id
    }
    #[setter]
    fn set_market_segment_id(&mut self, market_segment_id: u32) {
        self.inner.market_segment_id = market_segment_id;
    }
    #[getter]

    fn get_max_trade_vol(&self) -> u32 {
        self.inner.max_trade_vol
    }
    #[setter]
    fn set_max_trade_vol(&mut self, max_trade_vol: u32) {
        self.inner.max_trade_vol = max_trade_vol;
    }
    #[getter]

    fn get_min_lot_size(&self) -> i32 {
        self.inner.min_lot_size
    }
    #[setter]
    fn set_min_lot_size(&mut self, min_lot_size: i32) {
        self.inner.min_lot_size = min_lot_size;
    }
    #[getter]

    fn get_min_lot_size_block(&self) -> i32 {
        self.inner.min_lot_size_block
    }
    #[setter]
    fn set_min_lot_size_block(&mut self, min_lot_size_block: i32) {
        self.inner.min_lot_size_block = min_lot_size_block;
    }
    #[getter]

    fn get_min_lot_size_round_lot(&self) -> i32 {
        self.inner.min_lot_size_round_lot
    }
    #[setter]
    fn set_min_lot_size_round_lot(&mut self, min_lot_size_round_lot: i32) {
        self.inner.min_lot_size_round_lot = min_lot_size_round_lot;
    }
    #[getter]

    fn get_min_trade_vol(&self) -> u32 {
        self.inner.min_trade_vol
    }
    #[setter]
    fn set_min_trade_vol(&mut self, min_trade_vol: u32) {
        self.inner.min_trade_vol = min_trade_vol;
    }
    #[getter]

    fn get_contract_multiplier(&self) -> i32 {
        self.inner.contract_multiplier
    }
    #[setter]
    fn set_contract_multiplier(&mut self, contract_multiplier: i32) {
        self.inner.contract_multiplier = contract_multiplier;
    }
    #[getter]

    fn get_decay_quantity(&self) -> i32 {
        self.inner.decay_quantity
    }
    #[setter]
    fn set_decay_quantity(&mut self, decay_quantity: i32) {
        self.inner.decay_quantity = decay_quantity;
    }
    #[getter]

    fn get_original_contract_size(&self) -> i32 {
        self.inner.original_contract_size
    }
    #[setter]
    fn set_original_contract_size(&mut self, original_contract_size: i32) {
        self.inner.original_contract_size = original_contract_size;
    }
    #[getter]

    fn get_trading_reference_date(&self) -> u16 {
        self.inner.trading_reference_date
    }
    #[setter]
    fn set_trading_reference_date(&mut self, trading_reference_date: u16) {
        self.inner.trading_reference_date = trading_reference_date;
    }
    #[getter]

    fn get_appl_id(&self) -> i16 {
        self.inner.appl_id
    }
    #[setter]
    fn set_appl_id(&mut self, appl_id: i16) {
        self.inner.appl_id = appl_id;
    }
    #[getter]

    fn get_maturity_year(&self) -> u16 {
        self.inner.maturity_year
    }
    #[setter]
    fn set_maturity_year(&mut self, maturity_year: u16) {
        self.inner.maturity_year = maturity_year;
    }
    #[getter]

    fn get_decay_start_date(&self) -> u16 {
        self.inner.decay_start_date
    }
    #[setter]
    fn set_decay_start_date(&mut self, decay_start_date: u16) {
        self.inner.decay_start_date = decay_start_date;
    }
    #[getter]

    fn get_channel_id(&self) -> u16 {
        self.inner.channel_id
    }
    #[setter]
    fn set_channel_id(&mut self, channel_id: u16) {
        self.inner.channel_id = channel_id;
    }
    #[getter]
    fn get_currency(&self) -> PyResult<&str> {
        Ok(self.inner.currency()?)
    }
    #[getter]
    fn get_settl_currency(&self) -> PyResult<&str> {
        Ok(self.inner.settl_currency()?)
    }
    #[getter]
    fn get_secsubtype(&self) -> PyResult<&str> {
        Ok(self.inner.secsubtype()?)
    }
    #[getter]
    fn get_raw_symbol(&self) -> PyResult<&str> {
        Ok(self.inner.raw_symbol()?)
    }
    #[getter]
    fn get_group(&self) -> PyResult<&str> {
        Ok(self.inner.group()?)
    }
    #[getter]
    fn get_exchange(&self) -> PyResult<&str> {
        Ok(self.inner.exchange()?)
    }
    #[getter]
    fn get_asset(&self) -> PyResult<&str> {
        Ok(self.inner.asset()?)
    }
    #[getter]
    fn get_cfi(&self) -> PyResult<&str> {
        Ok(self.inner.cfi()?)
    }
    #[getter]
    fn get_security_type(&self) -> PyResult<&str> {
        Ok(self.inner.security_type()?)
    }
    #[getter]
    fn get_unit_of_measure(&self) -> PyResult<&str> {
        Ok(self.inner.unit_of_measure()?)
    }
    #[getter]
    fn get_underlying(&self) -> PyResult<&str> {
        Ok(self.inner.underlying()?)
    }
    #[getter]
    fn get_strike_price_currency(&self) -> PyResult<&str> {
        Ok(self.inner.strike_price_currency()?)
    }
    #[getter]
    fn get_instrument_class<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .instrument_class()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.instrument_class as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_instrument_class(&mut self, instrument_class: InstrumentClass) {
        self.inner.instrument_class = instrument_class as u8 as c_char;
    }
    #[getter]

    fn get_strike_price(&self) -> i64 {
        self.inner.strike_price
    }
    #[setter]
    fn set_strike_price(&mut self, strike_price: i64) {
        self.inner.strike_price = strike_price;
    }
    #[getter]
    fn get_pretty_strike_price(&self) -> f64 {
        self.inner.strike_price_f64()
    }
    #[getter]
    fn get_match_algorithm<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .match_algorithm()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.match_algorithm as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_match_algorithm(&mut self, match_algorithm: MatchAlgorithm) {
        self.inner.match_algorithm = match_algorithm as u8 as c_char;
    }
    #[getter]

    fn get_md_security_trading_status(&self) -> u8 {
        self.inner.md_security_trading_status
    }
    #[setter]
    fn set_md_security_trading_status(&mut self, md_security_trading_status: u8) {
        self.inner.md_security_trading_status = md_security_trading_status;
    }
    #[getter]

    fn get_main_fraction(&self) -> u8 {
        self.inner.main_fraction
    }
    #[setter]
    fn set_main_fraction(&mut self, main_fraction: u8) {
        self.inner.main_fraction = main_fraction;
    }
    #[getter]

    fn get_price_display_format(&self) -> u8 {
        self.inner.price_display_format
    }
    #[setter]
    fn set_price_display_format(&mut self, price_display_format: u8) {
        self.inner.price_display_format = price_display_format;
    }
    #[getter]

    fn get_settl_price_type(&self) -> u8 {
        self.inner.settl_price_type
    }
    #[setter]
    fn set_settl_price_type(&mut self, settl_price_type: u8) {
        self.inner.settl_price_type = settl_price_type;
    }
    #[getter]

    fn get_sub_fraction(&self) -> u8 {
        self.inner.sub_fraction
    }
    #[setter]
    fn set_sub_fraction(&mut self, sub_fraction: u8) {
        self.inner.sub_fraction = sub_fraction;
    }
    #[getter]

    fn get_underlying_product(&self) -> u8 {
        self.inner.underlying_product
    }
    #[setter]
    fn set_underlying_product(&mut self, underlying_product: u8) {
        self.inner.underlying_product = underlying_product;
    }
    #[getter]

    fn get_security_update_action(&self) -> SecurityUpdateAction {
        self.inner.security_update_action
    }
    #[setter]
    fn set_security_update_action(&mut self, security_update_action: SecurityUpdateAction) {
        self.inner.security_update_action = security_update_action;
    }
    #[getter]

    fn get_maturity_month(&self) -> u8 {
        self.inner.maturity_month
    }
    #[setter]
    fn set_maturity_month(&mut self, maturity_month: u8) {
        self.inner.maturity_month = maturity_month;
    }
    #[getter]

    fn get_maturity_day(&self) -> u8 {
        self.inner.maturity_day
    }
    #[setter]
    fn set_maturity_day(&mut self, maturity_day: u8) {
        self.inner.maturity_day = maturity_day;
    }
    #[getter]

    fn get_maturity_week(&self) -> u8 {
        self.inner.maturity_week
    }
    #[setter]
    fn set_maturity_week(&mut self, maturity_week: u8) {
        self.inner.maturity_week = maturity_week;
    }
    #[getter]

    fn get_user_defined_instrument(&self) -> UserDefinedInstrument {
        self.inner.user_defined_instrument
    }
    #[setter]
    fn set_user_defined_instrument(&mut self, user_defined_instrument: UserDefinedInstrument) {
        self.inner.user_defined_instrument = user_defined_instrument;
    }
    #[getter]

    fn get_contract_multiplier_unit(&self) -> i8 {
        self.inner.contract_multiplier_unit
    }
    #[setter]
    fn set_contract_multiplier_unit(&mut self, contract_multiplier_unit: i8) {
        self.inner.contract_multiplier_unit = contract_multiplier_unit;
    }
    #[getter]

    fn get_flow_schedule_type(&self) -> i8 {
        self.inner.flow_schedule_type
    }
    #[setter]
    fn set_flow_schedule_type(&mut self, flow_schedule_type: i8) {
        self.inner.flow_schedule_type = flow_schedule_type;
    }
    #[getter]

    fn get_tick_rule(&self) -> u8 {
        self.inner.tick_rule
    }
    #[setter]
    fn set_tick_rule(&mut self, tick_rule: u8) {
        self.inner.tick_rule = tick_rule;
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        v1::InstrumentDefMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        v1::InstrumentDefMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        v1::InstrumentDefMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        v1::InstrumentDefMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        v1::InstrumentDefMsg::ordered_fields("")
    }
}

/// Convert bare [`v1::InstrumentDefMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for v1::InstrumentDefMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyV1InstrumentDefMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<v1::InstrumentDefMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<v1::InstrumentDefMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyV1InstrumentDefMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`v1::StatMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "StatMsgV1")]
pub struct PyV1StatMsg {
    pub(crate) inner: v1::StatMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyV1StatMsg {
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
        channel_id = None,
        update_action = None,
        stat_flags = 0,
        ts_out = UNDEF_TIMESTAMP,
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
        channel_id: Option<u16>,
        update_action: Option<StatUpdateAction>,
        stat_flags: u8,
        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: v1::StatMsg {
                hd: RecordHeader::new::<v1::StatMsg>(
                    rtype::STATISTICS,
                    publisher_id,
                    instrument_id,
                    ts_event,
                ),
                ts_recv,
                ts_ref,
                price,
                quantity,
                sequence,
                ts_in_delta,
                stat_type: stat_type as u16,
                channel_id: channel_id.unwrap_or(u16::MAX),
                update_action: update_action.unwrap_or_default() as u8,
                stat_flags,
                _reserved: Default::default(),
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<v1::StatMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<v1::StatMsg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]

    fn get_ts_recv(&self) -> u64 {
        self.inner.ts_recv
    }
    #[setter]
    fn set_ts_recv(&mut self, ts_recv: u64) {
        self.inner.ts_recv = ts_recv;
    }
    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.ts_recv)
    }
    #[getter]

    fn get_ts_ref(&self) -> u64 {
        self.inner.ts_ref
    }
    #[setter]
    fn set_ts_ref(&mut self, ts_ref: u64) {
        self.inner.ts_ref = ts_ref;
    }
    #[getter]
    fn get_pretty_ts_ref<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.ts_ref)
    }
    #[getter]

    fn get_price(&self) -> i64 {
        self.inner.price
    }
    #[setter]
    fn set_price(&mut self, price: i64) {
        self.inner.price = price;
    }
    #[getter]
    fn get_pretty_price(&self) -> f64 {
        self.inner.price_f64()
    }
    #[getter]

    fn get_quantity(&self) -> i32 {
        self.inner.quantity
    }
    #[setter]
    fn set_quantity(&mut self, quantity: i32) {
        self.inner.quantity = quantity;
    }
    #[getter]

    fn get_sequence(&self) -> u32 {
        self.inner.sequence
    }
    #[setter]
    fn set_sequence(&mut self, sequence: u32) {
        self.inner.sequence = sequence;
    }
    #[getter]

    fn get_ts_in_delta(&self) -> i32 {
        self.inner.ts_in_delta
    }
    #[setter]
    fn set_ts_in_delta(&mut self, ts_in_delta: i32) {
        self.inner.ts_in_delta = ts_in_delta;
    }
    #[getter]
    fn get_stat_type<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .stat_type()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| self.inner.stat_type.into_bound_py_any(py))
    }
    #[setter]
    fn set_stat_type(&mut self, stat_type: StatType) {
        self.inner.stat_type = stat_type as u16;
    }
    #[getter]

    fn get_channel_id(&self) -> u16 {
        self.inner.channel_id
    }
    #[setter]
    fn set_channel_id(&mut self, channel_id: u16) {
        self.inner.channel_id = channel_id;
    }
    #[getter]
    fn get_update_action<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .update_action()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| self.inner.update_action.into_bound_py_any(py))
    }
    #[setter]
    fn set_update_action(&mut self, update_action: StatUpdateAction) {
        self.inner.update_action = update_action as u8;
    }
    #[getter]

    fn get_stat_flags(&self) -> u8 {
        self.inner.stat_flags
    }
    #[setter]
    fn set_stat_flags(&mut self, stat_flags: u8) {
        self.inner.stat_flags = stat_flags;
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        v1::StatMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        v1::StatMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        v1::StatMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        v1::StatMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        v1::StatMsg::ordered_fields("")
    }
}

/// Convert bare [`v1::StatMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for v1::StatMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyV1StatMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<v1::StatMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<v1::StatMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyV1StatMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`v1::SymbolMappingMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "SymbolMappingMsgV1")]
pub struct PyV1SymbolMappingMsg {
    pub(crate) inner: v1::SymbolMappingMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyV1SymbolMappingMsg {
    #[new]
    #[pyo3(signature = (
        publisher_id,
        instrument_id,
        ts_event,
        stype_in_symbol,
        stype_out_symbol,
        start_ts,
        end_ts,
        ts_out = UNDEF_TIMESTAMP,
    ))]
    fn py_new(
        publisher_id: u16,
        instrument_id: u32,
        ts_event: u64,
        stype_in_symbol: &str,
        stype_out_symbol: &str,
        start_ts: u64,
        end_ts: u64,

        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: v1::SymbolMappingMsg {
                hd: RecordHeader::new::<v1::SymbolMappingMsg>(
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
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<v1::SymbolMappingMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length = (std::mem::size_of::<v1::SymbolMappingMsg>()
                / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]
    fn get_stype_in_symbol(&self) -> PyResult<&str> {
        Ok(self.inner.stype_in_symbol()?)
    }
    #[getter]
    fn get_stype_out_symbol(&self) -> PyResult<&str> {
        Ok(self.inner.stype_out_symbol()?)
    }
    #[getter]

    fn get_start_ts(&self) -> u64 {
        self.inner.start_ts
    }
    #[setter]
    fn set_start_ts(&mut self, start_ts: u64) {
        self.inner.start_ts = start_ts;
    }
    #[getter]
    fn get_pretty_start_ts<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.start_ts)
    }
    #[getter]

    fn get_end_ts(&self) -> u64 {
        self.inner.end_ts
    }
    #[setter]
    fn set_end_ts(&mut self, end_ts: u64) {
        self.inner.end_ts = end_ts;
    }
    #[getter]
    fn get_pretty_end_ts<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.end_ts)
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        v1::SymbolMappingMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        v1::SymbolMappingMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        v1::SymbolMappingMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        v1::SymbolMappingMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        v1::SymbolMappingMsg::ordered_fields("")
    }
}

/// Convert bare [`v1::SymbolMappingMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for v1::SymbolMappingMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyV1SymbolMappingMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<v1::SymbolMappingMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<v1::SymbolMappingMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyV1SymbolMappingMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`v1::SystemMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "SystemMsgV1")]
pub struct PyV1SystemMsg {
    pub(crate) inner: v1::SystemMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyV1SystemMsg {
    #[new]
    fn py_new(ts_event: u64, msg: &str) -> PyResult<Self> {
        Ok(Self {
            inner: v1::SystemMsg::new(ts_event, msg)?,
            ts_out: UNDEF_TIMESTAMP,
        })
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<v1::SystemMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length =
                (std::mem::size_of::<v1::SystemMsg>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[pyo3(name = "is_heartbeat")]
    fn py_is_heartbeat(&self) -> bool {
        self.inner.is_heartbeat()
    }

    #[getter]
    fn get_msg(&self) -> PyResult<&str> {
        Ok(self.inner.msg()?)
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        v1::SystemMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        v1::SystemMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        v1::SystemMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        v1::SystemMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        v1::SystemMsg::ordered_fields("")
    }
}

/// Convert bare [`v1::SystemMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for v1::SystemMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyV1SystemMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<v1::SystemMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<v1::SystemMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyV1SystemMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}

/// Python wrapper for [`v2::InstrumentDefMsg`] that always includes space for `ts_out`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash)]
#[pyclass(eq, module = "databento_dbn", name = "InstrumentDefMsgV2")]
pub struct PyV2InstrumentDefMsg {
    pub(crate) inner: v2::InstrumentDefMsg,
    /// The live gateway send timestamp expressed as the number of nanoseconds since
    /// the UNIX epoch. Set to [`UNDEF_TIMESTAMP`] when not applicable.
    #[pyo3(get)]
    pub ts_out: u64,
}

#[pymethods]
impl PyV2InstrumentDefMsg {
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
        ts_out = UNDEF_TIMESTAMP,
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

        ts_out: u64,
    ) -> PyResult<Self> {
        let mut res = Self {
            inner: v2::InstrumentDefMsg {
                hd: RecordHeader::new::<v2::InstrumentDefMsg>(
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
            },
            ts_out,
        };

        if ts_out != UNDEF_TIMESTAMP {
            res.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
        Ok(res)
    }

    fn __bytes__(&self) -> &[u8] {
        let size = self.inner.hd.record_size();
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size) }
    }

    fn __repr__(&self) -> String {
        let mut s = String::new();
        self.inner.write_py_repr(&mut s).unwrap();
        s
    }
    #[getter]
    fn rtype(&self) -> PyResult<RType> {
        self.inner.hd.rtype().map_err(to_py_err)
    }

    #[getter]
    fn get_publisher_id(&self) -> u16 {
        self.inner.hd.publisher_id
    }
    #[setter]
    fn set_publisher_id(&mut self, publisher_id: u16) {
        self.inner.hd.publisher_id = publisher_id;
    }

    #[getter]
    fn get_instrument_id(&self) -> u32 {
        self.inner.hd.instrument_id
    }
    #[setter]
    fn set_instrument_id(&mut self, instrument_id: u32) {
        self.inner.hd.instrument_id = instrument_id;
    }

    #[getter]
    fn ts_event(&self) -> u64 {
        self.inner.hd.ts_event
    }
    #[getter]
    fn get_pretty_ts_event<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.hd.ts_event)
    }
    #[setter]
    fn set_ts_event(&mut self, ts_event: u64) {
        self.inner.hd.ts_event = ts_event;
    }

    #[pyo3(name = "record_size")]
    fn py_record_size(&self) -> usize {
        self.inner.record_size()
    }

    #[classattr]
    fn size_hint() -> PyResult<usize> {
        Ok(mem::size_of::<v2::InstrumentDefMsg>())
    }

    #[getter]
    fn ts_index(&self) -> u64 {
        self.inner.raw_index_ts()
    }
    #[getter]
    fn get_pretty_ts_index<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.raw_index_ts())
    }

    #[setter]
    fn set_ts_out(&mut self, ts_out: u64) {
        self.ts_out = ts_out;
        if ts_out != UNDEF_TIMESTAMP {
            self.inner.hd.length =
                (std::mem::size_of::<Self>() / RecordHeader::LENGTH_MULTIPLIER) as u8;
        } else {
            self.inner.hd.length = (std::mem::size_of::<v2::InstrumentDefMsg>()
                / RecordHeader::LENGTH_MULTIPLIER) as u8;
        }
    }

    #[getter]
    fn get_pretty_ts_out<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.ts_out)
    }

    #[getter]

    fn get_ts_recv(&self) -> u64 {
        self.inner.ts_recv
    }
    #[setter]
    fn set_ts_recv(&mut self, ts_recv: u64) {
        self.inner.ts_recv = ts_recv;
    }
    #[getter]
    fn get_pretty_ts_recv<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.ts_recv)
    }
    #[getter]

    fn get_min_price_increment(&self) -> i64 {
        self.inner.min_price_increment
    }
    #[setter]
    fn set_min_price_increment(&mut self, min_price_increment: i64) {
        self.inner.min_price_increment = min_price_increment;
    }
    #[getter]
    fn get_pretty_min_price_increment(&self) -> f64 {
        self.inner.min_price_increment_f64()
    }
    #[getter]

    fn get_display_factor(&self) -> i64 {
        self.inner.display_factor
    }
    #[setter]
    fn set_display_factor(&mut self, display_factor: i64) {
        self.inner.display_factor = display_factor;
    }
    #[getter]
    fn get_pretty_display_factor(&self) -> f64 {
        self.inner.display_factor_f64()
    }
    #[getter]

    fn get_expiration(&self) -> u64 {
        self.inner.expiration
    }
    #[setter]
    fn set_expiration(&mut self, expiration: u64) {
        self.inner.expiration = expiration;
    }
    #[getter]
    fn get_pretty_expiration<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.expiration)
    }
    #[getter]

    fn get_activation(&self) -> u64 {
        self.inner.activation
    }
    #[setter]
    fn set_activation(&mut self, activation: u64) {
        self.inner.activation = activation;
    }
    #[getter]
    fn get_pretty_activation<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        new_py_timestamp_or_datetime(py, self.inner.activation)
    }
    #[getter]

    fn get_high_limit_price(&self) -> i64 {
        self.inner.high_limit_price
    }
    #[setter]
    fn set_high_limit_price(&mut self, high_limit_price: i64) {
        self.inner.high_limit_price = high_limit_price;
    }
    #[getter]
    fn get_pretty_high_limit_price(&self) -> f64 {
        self.inner.high_limit_price_f64()
    }
    #[getter]

    fn get_low_limit_price(&self) -> i64 {
        self.inner.low_limit_price
    }
    #[setter]
    fn set_low_limit_price(&mut self, low_limit_price: i64) {
        self.inner.low_limit_price = low_limit_price;
    }
    #[getter]
    fn get_pretty_low_limit_price(&self) -> f64 {
        self.inner.low_limit_price_f64()
    }
    #[getter]

    fn get_max_price_variation(&self) -> i64 {
        self.inner.max_price_variation
    }
    #[setter]
    fn set_max_price_variation(&mut self, max_price_variation: i64) {
        self.inner.max_price_variation = max_price_variation;
    }
    #[getter]
    fn get_pretty_max_price_variation(&self) -> f64 {
        self.inner.max_price_variation_f64()
    }
    #[getter]

    fn get_trading_reference_price(&self) -> i64 {
        self.inner.trading_reference_price
    }
    #[setter]
    fn set_trading_reference_price(&mut self, trading_reference_price: i64) {
        self.inner.trading_reference_price = trading_reference_price;
    }
    #[getter]
    fn get_pretty_trading_reference_price(&self) -> f64 {
        self.inner.trading_reference_price_f64()
    }
    #[getter]

    fn get_unit_of_measure_qty(&self) -> i64 {
        self.inner.unit_of_measure_qty
    }
    #[setter]
    fn set_unit_of_measure_qty(&mut self, unit_of_measure_qty: i64) {
        self.inner.unit_of_measure_qty = unit_of_measure_qty;
    }
    #[getter]
    fn get_pretty_unit_of_measure_qty(&self) -> f64 {
        self.inner.unit_of_measure_qty_f64()
    }
    #[getter]

    fn get_min_price_increment_amount(&self) -> i64 {
        self.inner.min_price_increment_amount
    }
    #[setter]
    fn set_min_price_increment_amount(&mut self, min_price_increment_amount: i64) {
        self.inner.min_price_increment_amount = min_price_increment_amount;
    }
    #[getter]
    fn get_pretty_min_price_increment_amount(&self) -> f64 {
        self.inner.min_price_increment_amount_f64()
    }
    #[getter]

    fn get_price_ratio(&self) -> i64 {
        self.inner.price_ratio
    }
    #[setter]
    fn set_price_ratio(&mut self, price_ratio: i64) {
        self.inner.price_ratio = price_ratio;
    }
    #[getter]
    fn get_pretty_price_ratio(&self) -> f64 {
        self.inner.price_ratio_f64()
    }
    #[getter]

    fn get_strike_price(&self) -> i64 {
        self.inner.strike_price
    }
    #[setter]
    fn set_strike_price(&mut self, strike_price: i64) {
        self.inner.strike_price = strike_price;
    }
    #[getter]
    fn get_pretty_strike_price(&self) -> f64 {
        self.inner.strike_price_f64()
    }
    #[getter]

    fn get_inst_attrib_value(&self) -> i32 {
        self.inner.inst_attrib_value
    }
    #[setter]
    fn set_inst_attrib_value(&mut self, inst_attrib_value: i32) {
        self.inner.inst_attrib_value = inst_attrib_value;
    }
    #[getter]

    fn get_underlying_id(&self) -> u32 {
        self.inner.underlying_id
    }
    #[setter]
    fn set_underlying_id(&mut self, underlying_id: u32) {
        self.inner.underlying_id = underlying_id;
    }
    #[getter]

    fn get_raw_instrument_id(&self) -> u32 {
        self.inner.raw_instrument_id
    }
    #[setter]
    fn set_raw_instrument_id(&mut self, raw_instrument_id: u32) {
        self.inner.raw_instrument_id = raw_instrument_id;
    }
    #[getter]

    fn get_market_depth_implied(&self) -> i32 {
        self.inner.market_depth_implied
    }
    #[setter]
    fn set_market_depth_implied(&mut self, market_depth_implied: i32) {
        self.inner.market_depth_implied = market_depth_implied;
    }
    #[getter]

    fn get_market_depth(&self) -> i32 {
        self.inner.market_depth
    }
    #[setter]
    fn set_market_depth(&mut self, market_depth: i32) {
        self.inner.market_depth = market_depth;
    }
    #[getter]

    fn get_market_segment_id(&self) -> u32 {
        self.inner.market_segment_id
    }
    #[setter]
    fn set_market_segment_id(&mut self, market_segment_id: u32) {
        self.inner.market_segment_id = market_segment_id;
    }
    #[getter]

    fn get_max_trade_vol(&self) -> u32 {
        self.inner.max_trade_vol
    }
    #[setter]
    fn set_max_trade_vol(&mut self, max_trade_vol: u32) {
        self.inner.max_trade_vol = max_trade_vol;
    }
    #[getter]

    fn get_min_lot_size(&self) -> i32 {
        self.inner.min_lot_size
    }
    #[setter]
    fn set_min_lot_size(&mut self, min_lot_size: i32) {
        self.inner.min_lot_size = min_lot_size;
    }
    #[getter]

    fn get_min_lot_size_block(&self) -> i32 {
        self.inner.min_lot_size_block
    }
    #[setter]
    fn set_min_lot_size_block(&mut self, min_lot_size_block: i32) {
        self.inner.min_lot_size_block = min_lot_size_block;
    }
    #[getter]

    fn get_min_lot_size_round_lot(&self) -> i32 {
        self.inner.min_lot_size_round_lot
    }
    #[setter]
    fn set_min_lot_size_round_lot(&mut self, min_lot_size_round_lot: i32) {
        self.inner.min_lot_size_round_lot = min_lot_size_round_lot;
    }
    #[getter]

    fn get_min_trade_vol(&self) -> u32 {
        self.inner.min_trade_vol
    }
    #[setter]
    fn set_min_trade_vol(&mut self, min_trade_vol: u32) {
        self.inner.min_trade_vol = min_trade_vol;
    }
    #[getter]

    fn get_contract_multiplier(&self) -> i32 {
        self.inner.contract_multiplier
    }
    #[setter]
    fn set_contract_multiplier(&mut self, contract_multiplier: i32) {
        self.inner.contract_multiplier = contract_multiplier;
    }
    #[getter]

    fn get_decay_quantity(&self) -> i32 {
        self.inner.decay_quantity
    }
    #[setter]
    fn set_decay_quantity(&mut self, decay_quantity: i32) {
        self.inner.decay_quantity = decay_quantity;
    }
    #[getter]

    fn get_original_contract_size(&self) -> i32 {
        self.inner.original_contract_size
    }
    #[setter]
    fn set_original_contract_size(&mut self, original_contract_size: i32) {
        self.inner.original_contract_size = original_contract_size;
    }
    #[getter]

    fn get_trading_reference_date(&self) -> u16 {
        self.inner.trading_reference_date
    }
    #[setter]
    fn set_trading_reference_date(&mut self, trading_reference_date: u16) {
        self.inner.trading_reference_date = trading_reference_date;
    }
    #[getter]

    fn get_appl_id(&self) -> i16 {
        self.inner.appl_id
    }
    #[setter]
    fn set_appl_id(&mut self, appl_id: i16) {
        self.inner.appl_id = appl_id;
    }
    #[getter]

    fn get_maturity_year(&self) -> u16 {
        self.inner.maturity_year
    }
    #[setter]
    fn set_maturity_year(&mut self, maturity_year: u16) {
        self.inner.maturity_year = maturity_year;
    }
    #[getter]

    fn get_decay_start_date(&self) -> u16 {
        self.inner.decay_start_date
    }
    #[setter]
    fn set_decay_start_date(&mut self, decay_start_date: u16) {
        self.inner.decay_start_date = decay_start_date;
    }
    #[getter]

    fn get_channel_id(&self) -> u16 {
        self.inner.channel_id
    }
    #[setter]
    fn set_channel_id(&mut self, channel_id: u16) {
        self.inner.channel_id = channel_id;
    }
    #[getter]
    fn get_currency(&self) -> PyResult<&str> {
        Ok(self.inner.currency()?)
    }
    #[getter]
    fn get_settl_currency(&self) -> PyResult<&str> {
        Ok(self.inner.settl_currency()?)
    }
    #[getter]
    fn get_secsubtype(&self) -> PyResult<&str> {
        Ok(self.inner.secsubtype()?)
    }
    #[getter]
    fn get_raw_symbol(&self) -> PyResult<&str> {
        Ok(self.inner.raw_symbol()?)
    }
    #[getter]
    fn get_group(&self) -> PyResult<&str> {
        Ok(self.inner.group()?)
    }
    #[getter]
    fn get_exchange(&self) -> PyResult<&str> {
        Ok(self.inner.exchange()?)
    }
    #[getter]
    fn get_asset(&self) -> PyResult<&str> {
        Ok(self.inner.asset()?)
    }
    #[getter]
    fn get_cfi(&self) -> PyResult<&str> {
        Ok(self.inner.cfi()?)
    }
    #[getter]
    fn get_security_type(&self) -> PyResult<&str> {
        Ok(self.inner.security_type()?)
    }
    #[getter]
    fn get_unit_of_measure(&self) -> PyResult<&str> {
        Ok(self.inner.unit_of_measure()?)
    }
    #[getter]
    fn get_underlying(&self) -> PyResult<&str> {
        Ok(self.inner.underlying()?)
    }
    #[getter]
    fn get_strike_price_currency(&self) -> PyResult<&str> {
        Ok(self.inner.strike_price_currency()?)
    }
    #[getter]
    fn get_instrument_class<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .instrument_class()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.instrument_class as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_instrument_class(&mut self, instrument_class: InstrumentClass) {
        self.inner.instrument_class = instrument_class as u8 as c_char;
    }
    #[getter]
    fn get_match_algorithm<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .match_algorithm()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| (self.inner.match_algorithm as u8 as char).into_bound_py_any(py))
    }
    #[setter]
    fn set_match_algorithm(&mut self, match_algorithm: MatchAlgorithm) {
        self.inner.match_algorithm = match_algorithm as u8 as c_char;
    }
    #[getter]

    fn get_md_security_trading_status(&self) -> u8 {
        self.inner.md_security_trading_status
    }
    #[setter]
    fn set_md_security_trading_status(&mut self, md_security_trading_status: u8) {
        self.inner.md_security_trading_status = md_security_trading_status;
    }
    #[getter]

    fn get_main_fraction(&self) -> u8 {
        self.inner.main_fraction
    }
    #[setter]
    fn set_main_fraction(&mut self, main_fraction: u8) {
        self.inner.main_fraction = main_fraction;
    }
    #[getter]

    fn get_price_display_format(&self) -> u8 {
        self.inner.price_display_format
    }
    #[setter]
    fn set_price_display_format(&mut self, price_display_format: u8) {
        self.inner.price_display_format = price_display_format;
    }
    #[getter]

    fn get_settl_price_type(&self) -> u8 {
        self.inner.settl_price_type
    }
    #[setter]
    fn set_settl_price_type(&mut self, settl_price_type: u8) {
        self.inner.settl_price_type = settl_price_type;
    }
    #[getter]

    fn get_sub_fraction(&self) -> u8 {
        self.inner.sub_fraction
    }
    #[setter]
    fn set_sub_fraction(&mut self, sub_fraction: u8) {
        self.inner.sub_fraction = sub_fraction;
    }
    #[getter]

    fn get_underlying_product(&self) -> u8 {
        self.inner.underlying_product
    }
    #[setter]
    fn set_underlying_product(&mut self, underlying_product: u8) {
        self.inner.underlying_product = underlying_product;
    }
    #[getter]
    fn get_security_update_action<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .security_update_action()
            .map(|c| c.into_bound_py_any(py))
            .unwrap_or_else(|_| {
                (self.inner.security_update_action as u8 as char).into_bound_py_any(py)
            })
    }
    #[setter]
    fn set_security_update_action(&mut self, security_update_action: SecurityUpdateAction) {
        self.inner.security_update_action = security_update_action as u8 as c_char;
    }
    #[getter]

    fn get_maturity_month(&self) -> u8 {
        self.inner.maturity_month
    }
    #[setter]
    fn set_maturity_month(&mut self, maturity_month: u8) {
        self.inner.maturity_month = maturity_month;
    }
    #[getter]

    fn get_maturity_day(&self) -> u8 {
        self.inner.maturity_day
    }
    #[setter]
    fn set_maturity_day(&mut self, maturity_day: u8) {
        self.inner.maturity_day = maturity_day;
    }
    #[getter]

    fn get_maturity_week(&self) -> u8 {
        self.inner.maturity_week
    }
    #[setter]
    fn set_maturity_week(&mut self, maturity_week: u8) {
        self.inner.maturity_week = maturity_week;
    }
    #[getter]

    fn get_user_defined_instrument(&self) -> UserDefinedInstrument {
        self.inner.user_defined_instrument
    }
    #[setter]
    fn set_user_defined_instrument(&mut self, user_defined_instrument: UserDefinedInstrument) {
        self.inner.user_defined_instrument = user_defined_instrument;
    }
    #[getter]

    fn get_contract_multiplier_unit(&self) -> i8 {
        self.inner.contract_multiplier_unit
    }
    #[setter]
    fn set_contract_multiplier_unit(&mut self, contract_multiplier_unit: i8) {
        self.inner.contract_multiplier_unit = contract_multiplier_unit;
    }
    #[getter]

    fn get_flow_schedule_type(&self) -> i8 {
        self.inner.flow_schedule_type
    }
    #[setter]
    fn set_flow_schedule_type(&mut self, flow_schedule_type: i8) {
        self.inner.flow_schedule_type = flow_schedule_type;
    }
    #[getter]

    fn get_tick_rule(&self) -> u8 {
        self.inner.tick_rule
    }
    #[setter]
    fn set_tick_rule(&mut self, tick_rule: u8) {
        self.inner.tick_rule = tick_rule;
    }

    #[classattr]
    #[pyo3(name = "_dtypes")]
    fn py_dtypes() -> Vec<(String, String)> {
        v2::InstrumentDefMsg::field_dtypes("")
    }

    #[classattr]
    #[pyo3(name = "_price_fields")]
    fn py_price_fields() -> Vec<String> {
        v2::InstrumentDefMsg::price_fields("")
    }

    #[classattr]
    #[pyo3(name = "_timestamp_fields")]
    fn py_timestamp_fields() -> Vec<String> {
        v2::InstrumentDefMsg::timestamp_fields("")
    }

    #[classattr]
    #[pyo3(name = "_hidden_fields")]
    fn py_hidden_fields() -> Vec<String> {
        v2::InstrumentDefMsg::hidden_fields("")
    }

    #[classattr]
    #[pyo3(name = "_ordered_fields")]
    fn py_ordered_fields() -> Vec<String> {
        v2::InstrumentDefMsg::ordered_fields("")
    }
}

/// Convert bare [`v2::InstrumentDefMsg`] to Python by wrapping with `UNDEF_TIMESTAMP` for `ts_out`.
impl<'py> IntoPyObject<'py> for v2::InstrumentDefMsg {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyV2InstrumentDefMsg {
            inner: self,
            ts_out: UNDEF_TIMESTAMP,
        }
        .into_bound_py_any(py)
    }
}

/// Convert [`WithTsOut<v2::InstrumentDefMsg>`] to Python, preserving the `ts_out` value.
impl<'py> IntoPyObject<'py> for WithTsOut<v2::InstrumentDefMsg> {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        PyV2InstrumentDefMsg {
            inner: self.rec,
            ts_out: self.ts_out,
        }
        .into_bound_py_any(py)
    }
}
