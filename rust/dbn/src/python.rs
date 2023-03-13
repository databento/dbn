//! Python wrappers around dbn functions. These are implemented here instead of in `python/`
//! to be able to implement [`pyo3`] traits for [`dbn`] types.
#![allow(clippy::too_many_arguments)]
use std::{collections::HashMap, ffi::c_char, fmt, io, io::SeekFrom, num::NonZeroU64};

use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
    types::{PyBytes, PyDate, PyDateAccess, PyDict},
    PyClass,
};
use time::Date;

use crate::{
    decode::{DecodeDbn, DynDecoder},
    encode::{
        dbn::{self, MetadataEncoder},
        DbnEncodable, DynWriter, EncodeDbn,
    },
    enums::{rtype, Compression, SType, Schema, SecurityUpdateAction},
    metadata::MetadataBuilder,
    record::{
        str_to_c_chars, BidAskPair, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg,
        RecordHeader, TbboMsg, TradeMsg,
    },
};
use crate::{MappingInterval, Metadata, SymbolMapping};

/// Decodes the given Python `bytes` to `Metadata`. Returns a `Metadata` object with
/// all the DBN metadata attributes.
///
/// # Errors
/// This function returns an error if the metadata cannot be parsed from `bytes`.
#[pyfunction]
pub fn decode_metadata(bytes: &PyBytes) -> PyResult<Metadata> {
    let reader = io::BufReader::new(bytes.as_bytes());
    Ok(DynDecoder::inferred_with_buffer(reader)
        .map_err(to_val_err)?
        .metadata()
        .clone())
}

/// Encodes the given metadata into the DBN metadata binary format.
/// Returns Python `bytes`.
///
/// # Errors
/// This function returns an error if any of the enum arguments cannot be converted to
/// their Rust equivalents. It will also return an error if there's an issue writing
/// the encoded metadata to bytes.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
pub fn encode_metadata(
    py: Python<'_>,
    dataset: String,
    schema: Schema,
    start: u64,
    stype_in: SType,
    stype_out: SType,
    symbols: Vec<String>,
    partial: Vec<String>,
    not_found: Vec<String>,
    mappings: Vec<SymbolMapping>,
    end: Option<u64>,
    limit: Option<u64>,
    record_count: Option<u64>,
) -> PyResult<Py<PyBytes>> {
    let metadata = MetadataBuilder::new()
        .dataset(dataset)
        .schema(schema)
        .start(start)
        .end(NonZeroU64::new(end.unwrap_or(0)))
        .record_count(record_count)
        .limit(NonZeroU64::new(limit.unwrap_or(0)))
        .stype_in(stype_in)
        .stype_out(stype_out)
        .symbols(symbols)
        .partial(partial)
        .not_found(not_found)
        .mappings(mappings)
        .build();
    let mut encoded = Vec::with_capacity(1024);
    MetadataEncoder::new(&mut encoded)
        .encode(&metadata)
        .map_err(|e| {
            println!("{e:?}");
            to_val_err(e)
        })?;
    Ok(PyBytes::new(py, encoded.as_slice()).into())
}

/// Updates existing fields that have already been written to the given file.
#[pyfunction]
pub fn update_encoded_metadata(
    _py: Python<'_>,
    file: PyFileLike,
    start: u64,
    end: Option<u64>,
    limit: Option<u64>,
    record_count: Option<u64>,
) -> PyResult<()> {
    MetadataEncoder::new(file)
        .update_encoded(
            start,
            end.and_then(NonZeroU64::new),
            limit.and_then(NonZeroU64::new),
            record_count,
        )
        .map_err(to_val_err)
}

pub struct PyFileLike {
    inner: PyObject,
}

/// Encodes the given data in the DBN encoding and writes it to `file`.
///
/// `records` is a list of record objects.
///
/// # Errors
/// This function returns an error if any of the enum arguments cannot be converted to
/// their Rust equivalents. It will also return an error if there's an issue writing
/// the encoded to bytes or an expected field is missing from one of the dicts.
#[pyfunction]
pub fn write_dbn_file(
    _py: Python<'_>,
    file: PyFileLike,
    compression: Compression,
    dataset: String,
    schema: Schema,
    start: u64,
    stype_in: SType,
    stype_out: SType,
    records: Vec<&PyAny>,
    end: Option<u64>,
) -> PyResult<()> {
    let mut metadata_builder = MetadataBuilder::new()
        .schema(schema)
        .dataset(dataset)
        .stype_in(stype_in)
        .stype_out(stype_out)
        .start(start)
        .record_count(Some(records.len() as u64));
    if let Some(end) = end {
        metadata_builder = metadata_builder.end(NonZeroU64::new(end))
    }
    let metadata = metadata_builder.build();
    let writer = DynWriter::new(file, compression).map_err(to_val_err)?;
    let encoder = dbn::Encoder::new(writer, &metadata).map_err(to_val_err)?;
    match schema {
        Schema::Mbo => encode_pydicts::<MboMsg>(encoder, &records),
        Schema::Mbp1 => encode_pydicts::<Mbp1Msg>(encoder, &records),
        Schema::Mbp10 => encode_pydicts::<Mbp10Msg>(encoder, &records),
        Schema::Tbbo => encode_pydicts::<TbboMsg>(encoder, &records),
        Schema::Trades => encode_pydicts::<TradeMsg>(encoder, &records),
        Schema::Ohlcv1S | Schema::Ohlcv1M | Schema::Ohlcv1H | Schema::Ohlcv1D => {
            encode_pydicts::<OhlcvMsg>(encoder, &records)
        }
        Schema::Definition => encode_pydicts::<InstrumentDefMsg>(encoder, &records),
        Schema::Statistics | Schema::Status => Err(PyValueError::new_err(
            "Unsupported schema type for writing DBN files",
        )),
    }
}

fn encode_pydicts<T: Clone + DbnEncodable + PyClass>(
    mut encoder: dbn::Encoder<DynWriter<PyFileLike>>,
    records: &[&PyAny],
) -> PyResult<()> {
    encoder
        .encode_records(
            records
                .iter()
                .map(|obj| obj.extract())
                .collect::<PyResult<Vec<T>>>()?
                .iter()
                .as_slice(),
        )
        .map_err(to_val_err)
}

impl<'source> FromPyObject<'source> for PyFileLike {
    fn extract(any: &'source PyAny) -> PyResult<Self> {
        Python::with_gil(|py| {
            let obj: PyObject = any.extract()?;
            if obj.getattr(py, "read").is_err() {
                return Err(PyTypeError::new_err(
                    "object is missing a `read()` method".to_owned(),
                ));
            }
            if obj.getattr(py, "write").is_err() {
                return Err(PyTypeError::new_err(
                    "object is missing a `write()` method".to_owned(),
                ));
            }
            if obj.getattr(py, "seek").is_err() {
                return Err(PyTypeError::new_err(
                    "object is missing a `seek()` method".to_owned(),
                ));
            }
            Ok(PyFileLike { inner: obj })
        })
    }
}

#[pymethods]
impl Metadata {
    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    #[getter]
    fn get_mappings(&self) -> HashMap<String, Vec<MappingInterval>> {
        let mut res = HashMap::new();
        for mapping in self.mappings.iter() {
            res.insert(mapping.native_symbol.clone(), mapping.intervals.clone());
        }
        res
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
        let dict = PyDict::new(py);
        dict.set_item("native_symbol", &self.native_symbol)
            .expect("set native_symbol");
        dict.set_item("intervals", &self.intervals)
            .expect("set intervals");
        dict.into_py(py)
    }
}

fn extract_date(any: &PyAny) -> PyResult<time::Date> {
    let py_date = any.downcast::<PyDate>().map_err(PyErr::from)?;
    let month =
        time::Month::try_from(py_date.get_month()).map_err(|e| to_val_err(e.to_string()))?;
    Date::from_calendar_date(py_date.get_year(), month, py_date.get_day())
        .map_err(|e| to_val_err(e.to_string()))
}

impl<'source> FromPyObject<'source> for MappingInterval {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let start_date = ob
            .getattr("start_date")
            .map_err(|_| to_val_err("Missing start_date".to_owned()))
            .and_then(extract_date)?;
        let end_date = ob
            .getattr("end_date")
            .map_err(|_| to_val_err("Missing end_date".to_owned()))
            .and_then(extract_date)?;
        let symbol = ob
            .getattr("symbol")
            .map_err(|_| to_val_err("Missing symbol".to_owned()))
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
        let dict = PyDict::new(py);
        dict.set_item(
            "start_date",
            PyDate::new(
                py,
                self.start_date.year(),
                self.start_date.month() as u8,
                self.start_date.day(),
            )
            .expect("valid start_date"),
        )
        .expect("set start_date");
        dict.set_item(
            "end_date",
            PyDate::new(
                py,
                self.end_date.year(),
                self.end_date.month() as u8,
                self.end_date.day(),
            )
            .expect("valid end_date"),
        )
        .expect("set end_date");
        dict.set_item("symbol", &self.symbol).expect("set symbol");
        dict.into_py(py)
    }
}

impl IntoPy<PyObject> for MappingInterval {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

pub fn to_val_err(e: impl fmt::Debug) -> PyErr {
    PyValueError::new_err(format!("{e:?}"))
}

fn py_to_rs_io_err(e: PyErr) -> io::Error {
    Python::with_gil(|py| {
        let e_as_object: PyObject = e.into_py(py);

        match e_as_object.call_method(py, "__str__", (), None) {
            Ok(repr) => match repr.extract::<String>(py) {
                Ok(s) => io::Error::new(io::ErrorKind::Other, s),
                Err(_e) => io::Error::new(io::ErrorKind::Other, "An unknown error has occurred"),
            },
            Err(_) => io::Error::new(io::ErrorKind::Other, "Err doesn't have __str__"),
        }
    })
}

impl io::Write for PyFileLike {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        Python::with_gil(|py| {
            let bytes = PyBytes::new(py, buf).to_object(py);
            let number_bytes_written = self
                .inner
                .call_method(py, "write", (bytes,), None)
                .map_err(py_to_rs_io_err)?;

            number_bytes_written.extract(py).map_err(py_to_rs_io_err)
        })
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        Python::with_gil(|py| {
            self.inner
                .call_method(py, "flush", (), None)
                .map_err(py_to_rs_io_err)?;

            Ok(())
        })
    }
}

impl io::Seek for PyFileLike {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, io::Error> {
        Python::with_gil(|py| {
            let (whence, offset) = match pos {
                SeekFrom::Start(i) => (0, i as i64),
                SeekFrom::Current(i) => (1, i),
                SeekFrom::End(i) => (2, i),
            };

            let new_position = self
                .inner
                .call_method(py, "seek", (offset, whence), None)
                .map_err(py_to_rs_io_err)?;

            new_position.extract(py).map_err(py_to_rs_io_err)
        })
    }
}

impl<'source> FromPyObject<'source> for Compression {
    fn extract(any: &'source PyAny) -> PyResult<Self> {
        let str: &str = any.extract()?;
        str.parse().map_err(to_val_err)
    }
}

impl<'source> FromPyObject<'source> for Schema {
    fn extract(any: &'source PyAny) -> PyResult<Self> {
        let str: &str = any.extract()?;
        str.parse().map_err(to_val_err)
    }
}

impl IntoPy<PyObject> for Schema {
    fn into_py(self, py: Python<'_>) -> PyObject {
        (self.as_str()).into_py(py)
    }
}

impl<'source> FromPyObject<'source> for SType {
    fn extract(any: &'source PyAny) -> PyResult<Self> {
        let str: &str = any.extract()?;
        str.parse().map_err(to_val_err)
    }
}

impl IntoPy<PyObject> for SType {
    fn into_py(self, py: Python<'_>) -> PyObject {
        (self.as_str()).into_py(py)
    }
}

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

#[pymethods]
impl MboMsg {
    #[new]
    fn py_new(
        publisher_id: u16,
        product_id: u32,
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
            hd: RecordHeader::new::<Self>(rtype::MBO, publisher_id, product_id, ts_event),
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
            bid_px: bid_px.unwrap_or_default(),
            ask_px: ask_px.unwrap_or_default(),
            bid_sz: bid_sz.unwrap_or_default(),
            ask_sz: ask_sz.unwrap_or_default(),
            bid_ct: bid_ct.unwrap_or_default(),
            ask_ct: ask_ct.unwrap_or_default(),
        }
    }
}

#[pymethods]
impl TradeMsg {
    #[new]
    fn py_new(
        publisher_id: u16,
        product_id: u32,
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
            hd: RecordHeader::new::<Self>(rtype::MBP_0, publisher_id, product_id, ts_event),
            price,
            size,
            action,
            side,
            flags: flags.unwrap_or_default(),
            depth,
            ts_recv,
            ts_in_delta,
            sequence,
            booklevel: [],
        }
    }
}

#[pymethods]
impl Mbp1Msg {
    #[new]
    fn py_new(
        publisher_id: u16,
        product_id: u32,
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
        booklevel: Option<BidAskPair>,
    ) -> Self {
        Self {
            hd: RecordHeader::new::<Self>(rtype::MBP_1, publisher_id, product_id, ts_event),
            price,
            size,
            action,
            side,
            flags,
            depth,
            ts_recv,
            ts_in_delta,
            sequence,
            booklevel: [booklevel.unwrap_or_default()],
        }
    }
}

#[pymethods]
impl Mbp10Msg {
    #[new]
    fn py_new(
        publisher_id: u16,
        product_id: u32,
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
        booklevel: Option<Vec<BidAskPair>>,
    ) -> PyResult<Self> {
        let booklevel = if let Some(booklevel) = booklevel {
            let mut arr: [BidAskPair; 10] = Default::default();
            if booklevel.len() > 10 {
                return Err(to_val_err("Only 10 booklevels are allowed"));
            }
            for (i, level) in booklevel.into_iter().enumerate() {
                arr[i] = level;
            }
            arr
        } else {
            Default::default()
        };
        Ok(Self {
            hd: RecordHeader::new::<Self>(rtype::MBP_10, publisher_id, product_id, ts_event),
            price,
            size,
            action,
            side,
            flags,
            depth,
            ts_recv,
            ts_in_delta,
            sequence,
            booklevel,
        })
    }
}

#[pymethods]
impl OhlcvMsg {
    #[new]
    fn py_new(
        rtype: u8,
        publisher_id: u16,
        product_id: u32,
        ts_event: u64,
        open: i64,
        high: i64,
        low: i64,
        close: i64,
        volume: u64,
    ) -> Self {
        Self {
            hd: RecordHeader::new::<Self>(rtype, publisher_id, product_id, ts_event),
            open,
            high,
            low,
            close,
            volume,
        }
    }
}

#[pymethods]
impl InstrumentDefMsg {
    #[new]
    fn py_new(
        publisher_id: u16,
        product_id: u32,
        ts_event: u64,
        ts_recv: u64,
        min_price_increment: i64,
        display_factor: i64,
        min_lot_size_round_lot: i32,
        symbol: &str,
        group: &str,
        exchange: &str,
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
        cleared_volume: Option<i32>,
        market_depth_implied: Option<i32>,
        market_depth: Option<i32>,
        market_segment_id: Option<u32>,
        max_trade_vol: Option<u32>,
        min_lot_size: Option<i32>,
        min_lot_size_block: Option<i32>,
        min_trade_vol: Option<u32>,
        open_interest_qty: Option<i32>,
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
        main_fraction: Option<u8>,
        price_display_format: Option<u8>,
        settl_price_type: Option<u8>,
        sub_fraction: Option<u8>,
        underlying_product: Option<u8>,
        maturity_month: Option<u8>,
        maturity_day: Option<u8>,
        maturity_week: Option<u8>,
        user_defined_instrument: Option<c_char>,
        contract_multiplier_unit: Option<i8>,
        flow_schedule_type: Option<i8>,
        tick_rule: Option<u8>,
    ) -> PyResult<Self> {
        Ok(Self {
            hd: RecordHeader::new::<Self>(
                rtype::INSTRUMENT_DEF,
                publisher_id,
                product_id,
                ts_event,
            ),
            ts_recv,
            min_price_increment,
            display_factor,
            expiration: expiration.unwrap_or(u64::MAX),
            activation: activation.unwrap_or(u64::MAX),
            high_limit_price: high_limit_price.unwrap_or(i64::MAX),
            low_limit_price: low_limit_price.unwrap_or(i64::MAX),
            max_price_variation: max_price_variation.unwrap_or(i64::MAX),
            trading_reference_price: trading_reference_price.unwrap_or(i64::MAX),
            unit_of_measure_qty: unit_of_measure_qty.unwrap_or(i64::MAX),
            min_price_increment_amount: min_price_increment_amount.unwrap_or(i64::MAX),
            price_ratio: price_ratio.unwrap_or(i64::MAX),
            inst_attrib_value: inst_attrib_value.unwrap_or(i32::MAX),
            underlying_id: underlying_id.unwrap_or_default(),
            cleared_volume: cleared_volume.unwrap_or(i32::MAX),
            market_depth_implied: market_depth_implied.unwrap_or(i32::MAX),
            market_depth: market_depth.unwrap_or(i32::MAX),
            market_segment_id: market_segment_id.unwrap_or(u32::MAX),
            max_trade_vol: max_trade_vol.unwrap_or(u32::MAX),
            min_lot_size: min_lot_size.unwrap_or(i32::MAX),
            min_lot_size_block: min_lot_size_block.unwrap_or(i32::MAX),
            min_lot_size_round_lot,
            min_trade_vol: min_trade_vol.unwrap_or(u32::MAX),
            open_interest_qty: open_interest_qty.unwrap_or(i32::MAX),
            contract_multiplier: contract_multiplier.unwrap_or(i32::MAX),
            decay_quantity: decay_quantity.unwrap_or(i32::MAX),
            original_contract_size: original_contract_size.unwrap_or(i32::MAX),
            related_security_id: 0,
            trading_reference_date: trading_reference_date.unwrap_or(u16::MAX),
            appl_id: appl_id.unwrap_or(i16::MAX),
            maturity_year: maturity_year.unwrap_or(u16::MAX),
            decay_start_date: decay_start_date.unwrap_or(u16::MAX),
            channel_id: channel_id.unwrap_or(u16::MAX),
            currency: str_to_c_chars(currency.unwrap_or_default()).map_err(to_val_err)?,
            settl_currency: str_to_c_chars(settl_currency.unwrap_or_default())
                .map_err(to_val_err)?,
            secsubtype: str_to_c_chars(secsubtype.unwrap_or_default()).map_err(to_val_err)?,
            symbol: str_to_c_chars(symbol).map_err(to_val_err)?,
            group: str_to_c_chars(group).map_err(to_val_err)?,
            exchange: str_to_c_chars(exchange).map_err(to_val_err)?,
            asset: str_to_c_chars(asset.unwrap_or_default()).map_err(to_val_err)?,
            cfi: str_to_c_chars(cfi.unwrap_or_default()).map_err(to_val_err)?,
            security_type: str_to_c_chars(security_type.unwrap_or_default()).map_err(to_val_err)?,
            unit_of_measure: str_to_c_chars(unit_of_measure.unwrap_or_default())
                .map_err(to_val_err)?,
            underlying: str_to_c_chars(underlying.unwrap_or_default()).map_err(to_val_err)?,
            related: Default::default(),
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
            user_defined_instrument: user_defined_instrument.unwrap_or('N' as c_char),
            contract_multiplier_unit: contract_multiplier_unit.unwrap_or(i8::MAX),
            flow_schedule_type: flow_schedule_type.unwrap_or(i8::MAX),
            tick_rule: tick_rule.unwrap_or(u8::MAX),
            _dummy: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek, Write};
    use std::sync::{Arc, Mutex};

    use streaming_iterator::StreamingIterator;

    use super::*;
    use crate::{
        decode::{dbn, DecodeDbn},
        encode::json,
    };

    const DBN_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/data");

    type JsonObj = serde_json::Map<String, serde_json::Value>;

    #[pyclass]
    struct MockPyFile {
        buf: Arc<Mutex<Cursor<Vec<u8>>>>,
    }

    #[pymethods]
    impl MockPyFile {
        fn read(&self) {
            unimplemented!();
        }

        fn write(&mut self, bytes: &[u8]) -> usize {
            self.buf.lock().unwrap().write_all(bytes).unwrap();
            bytes.len()
        }

        fn flush(&mut self) {
            self.buf.lock().unwrap().flush().unwrap();
        }

        fn seek(&self, offset: i64, whence: i32) -> u64 {
            self.buf
                .lock()
                .unwrap()
                .seek(match whence {
                    0 => SeekFrom::Start(offset as u64),
                    1 => SeekFrom::Current(offset),
                    2 => SeekFrom::End(offset),
                    _ => unimplemented!("whence value"),
                })
                .unwrap()
        }
    }

    impl MockPyFile {
        fn new() -> Self {
            Self {
                buf: Arc::new(Mutex::new(Cursor::new(Vec::new()))),
            }
        }

        fn inner(&self) -> Arc<Mutex<Cursor<Vec<u8>>>> {
            self.buf.clone()
        }
    }

    fn add_to_dict(py: Python<'_>, dict: &PyDict, key: &str, value: &serde_json::Value) {
        match value {
            serde_json::Value::Null => {
                dict.set_item(key, ()).unwrap();
            }
            serde_json::Value::Bool(v) => {
                dict.set_item(key, v).unwrap();
            }
            serde_json::Value::Number(n) => {
                if n.is_u64() {
                    dict.set_item(key, n.as_u64())
                } else if n.is_i64() {
                    dict.set_item(key, n.as_i64())
                } else {
                    dict.set_item(key, n.as_f64())
                }
                .unwrap();
            }
            serde_json::Value::String(s) if key.starts_with("ts_") => {
                dict.set_item(key, s.parse::<u64>().unwrap()).unwrap();
            }
            serde_json::Value::String(s) => {
                dict.set_item(key, s).unwrap();
            }
            serde_json::Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let nested = PyDict::new(py);
                    add_to_dict(py, nested, "", val);
                    for (k, v) in nested.iter() {
                        dict.set_item(format!("{}_0{i}", k.extract::<String>().unwrap()), v)
                            .unwrap();
                    }
                }
            }
            serde_json::Value::Object(nested) => {
                // flatten
                nested.iter().for_each(|(n_k, n_v)| {
                    add_to_dict(py, dict, n_k, n_v);
                });
            }
        }
    }

    const DATASET: &str = "GLBX.MDP3";
    const STYPE: SType = SType::ProductId;

    macro_rules! test_writing_dbn_from_python {
        ($test_name:ident, $record_type:ident, $schema:expr) => {
            #[test]
            fn $test_name() {
                // Required one-time setup
                pyo3::prepare_freethreaded_python();

                // Read in test data
                let decoder = dbn::Decoder::from_zstd_file(format!(
                    "{DBN_PATH}/test_data.{}.dbn.zst",
                    $schema.as_str()
                ))
                .unwrap();
                let rs_recs = decoder.decode_records::<$record_type>().unwrap();
                let output_buf = Python::with_gil(|py| -> PyResult<_> {
                    // Convert JSON objects to Python `dict`s
                    let recs: Vec<_> = rs_recs
                        .iter()
                        .map(|rs_rec| rs_rec.clone().into_py(py))
                        .collect();
                    let mock_file = MockPyFile::new();
                    let output_buf = mock_file.inner();
                    let mock_file = Py::new(py, mock_file).unwrap().into_py(py);
                    // Call target function
                    write_dbn_file(
                        py,
                        mock_file.extract(py).unwrap(),
                        Compression::ZStd,
                        DATASET.to_owned(),
                        $schema,
                        0,
                        STYPE,
                        STYPE,
                        recs.iter().map(|r| r.as_ref(py)).collect(),
                        None,
                    )
                    .unwrap();

                    Ok(output_buf.clone())
                })
                .unwrap();
                let output_buf = output_buf.lock().unwrap().clone().into_inner();

                assert!(!output_buf.is_empty());

                dbg!(&output_buf);
                dbg!(output_buf.len());
                // Reread output written with `write_dbn_file` and compare to original
                // contents
                let py_decoder = dbn::Decoder::with_zstd(Cursor::new(&output_buf)).unwrap();
                let metadata = py_decoder.metadata().clone();
                assert_eq!(metadata.schema, $schema);
                assert_eq!(metadata.dataset, DATASET);
                assert_eq!(metadata.stype_in, STYPE);
                assert_eq!(metadata.stype_out, STYPE);
                assert_eq!(metadata.record_count.unwrap() as usize, rs_recs.len());
                let decoder = dbn::Decoder::from_zstd_file(format!(
                    "{DBN_PATH}/test_data.{}.dbn.zst",
                    $schema.as_str()
                ))
                .unwrap();

                let mut py_iter = py_decoder.decode_stream::<$record_type>().unwrap();
                let mut expected_iter = decoder.decode_stream::<$record_type>().unwrap();
                let mut count = 0;
                while let Some((py_rec, exp_rec)) = py_iter
                    .next()
                    .and_then(|py_rec| expected_iter.next().map(|exp_rec| (py_rec, exp_rec)))
                {
                    assert_eq!(py_rec, exp_rec);
                    count += 1;
                }
                assert_eq!(count, metadata.record_count.unwrap());
            }
        };
    }

    test_writing_dbn_from_python!(test_writing_mbo_from_python, MboMsg, Schema::Mbo);
    test_writing_dbn_from_python!(test_writing_mbp1_from_python, Mbp1Msg, Schema::Mbp1);
    test_writing_dbn_from_python!(test_writing_mbp10_from_python, Mbp10Msg, Schema::Mbp10);
    test_writing_dbn_from_python!(test_writing_ohlcv1d_from_python, OhlcvMsg, Schema::Ohlcv1D);
    test_writing_dbn_from_python!(test_writing_ohlcv1h_from_python, OhlcvMsg, Schema::Ohlcv1H);
    test_writing_dbn_from_python!(test_writing_ohlcv1m_from_python, OhlcvMsg, Schema::Ohlcv1M);
    test_writing_dbn_from_python!(test_writing_ohlcv1s_from_python, OhlcvMsg, Schema::Ohlcv1S);
    test_writing_dbn_from_python!(test_writing_tbbo_from_python, TbboMsg, Schema::Tbbo);
    test_writing_dbn_from_python!(test_writing_trades_from_python, TradeMsg, Schema::Trades);
}
