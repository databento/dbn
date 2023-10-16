//! Python wrappers around dbn functions. These are implemented here instead of in `python/`
//! to be able to implement [`pyo3`] traits for DBN types.
#![allow(clippy::too_many_arguments)]

use std::fmt;

use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyDate, PyDateAccess},
};
use strum::IntoEnumIterator;

mod enums;
mod metadata;
mod record;

/// A helper function for converting any type that implements `Debug` to a Python
/// `ValueError`.
pub fn to_val_err(e: impl fmt::Debug) -> PyErr {
    PyValueError::new_err(format!("{e:?}"))
}

/// Python iterator over the variants of an enum.
#[pyclass(module = "databento_dbn")]
pub struct EnumIterator {
    // Type erasure for code reuse. Generic types can't be exposed to Python.
    iter: Box<dyn Iterator<Item = PyObject> + Send>,
}

#[pymethods]
impl EnumIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyObject> {
        slf.iter.next()
    }
}

impl EnumIterator {
    fn new<E>(py: Python<'_>) -> Self
    where
        E: strum::IntoEnumIterator + IntoPy<Py<PyAny>>,
        <E as IntoEnumIterator>::Iterator: Send,
    {
        Self {
            iter: Box::new(
                E::iter()
                    .map(|var| var.into_py(py))
                    // force eager evaluation because `py` isn't `Send`
                    .collect::<Vec<_>>()
                    .into_iter(),
            ),
        }
    }
}

/// Tries to convert `py_date` to a [`time::Date`].
///
/// # Errors
/// This function returns an error if input has an invalid month.
pub fn py_to_time_date(py_date: &PyDate) -> PyResult<time::Date> {
    let month =
        time::Month::try_from(py_date.get_month()).map_err(|e| to_val_err(e.to_string()))?;
    time::Date::from_calendar_date(py_date.get_year(), month, py_date.get_day())
        .map_err(|e| to_val_err(e.to_string()))
}

/// A trait for records that provide descriptions of their fields.
pub(crate) trait PyFieldDesc {
    /// Returns a list of all fields and their numpy dtypes.
    fn field_dtypes(field_name: &str) -> Vec<(String, String)>;
    /// Returns a list of fields that should be hidden in Python.
    fn hidden_fields(_field_name: &str) -> Vec<String> {
        Vec::new()
    }
    /// Returns a list of the fixed-precision price fields.
    fn price_fields(_field_name: &str) -> Vec<String> {
        Vec::new()
    }
    /// Returns a list of UNIX nanosecond timestamp fields.
    fn timestamp_fields(_field_name: &str) -> Vec<String> {
        Vec::new()
    }
    /// Ordered list of fields excluding hidden fields.
    fn ordered_fields(field_name: &str) -> Vec<String> {
        vec![field_name.to_owned()]
    }
}

impl PyFieldDesc for i64 {
    fn field_dtypes(field_name: &str) -> Vec<(String, String)> {
        vec![(field_name.to_owned(), "i8".to_owned())]
    }
}
impl PyFieldDesc for i32 {
    fn field_dtypes(field_name: &str) -> Vec<(String, String)> {
        vec![(field_name.to_owned(), "i4".to_owned())]
    }
}
impl PyFieldDesc for i16 {
    fn field_dtypes(field_name: &str) -> Vec<(String, String)> {
        vec![(field_name.to_owned(), "i2".to_owned())]
    }
}
impl PyFieldDesc for i8 {
    fn field_dtypes(field_name: &str) -> Vec<(String, String)> {
        vec![(field_name.to_owned(), "i1".to_owned())]
    }
}
impl PyFieldDesc for u64 {
    fn field_dtypes(field_name: &str) -> Vec<(String, String)> {
        vec![(field_name.to_owned(), "u8".to_owned())]
    }
}
impl PyFieldDesc for u32 {
    fn field_dtypes(field_name: &str) -> Vec<(String, String)> {
        vec![(field_name.to_owned(), "u4".to_owned())]
    }
}
impl PyFieldDesc for u16 {
    fn field_dtypes(field_name: &str) -> Vec<(String, String)> {
        vec![(field_name.to_owned(), "u2".to_owned())]
    }
}
impl PyFieldDesc for u8 {
    fn field_dtypes(field_name: &str) -> Vec<(String, String)> {
        vec![(field_name.to_owned(), "u1".to_owned())]
    }
}
impl<const N: usize> PyFieldDesc for [i8; N] {
    fn field_dtypes(field_name: &str) -> Vec<(String, String)> {
        vec![(field_name.to_owned(), format!("S{N}"))]
    }
}
impl<const N: usize> PyFieldDesc for [u8; N] {
    fn field_dtypes(field_name: &str) -> Vec<(String, String)> {
        vec![(field_name.to_owned(), format!("S{N}"))]
    }
}

#[cfg(test)]
mod tests {
    use super::PyFieldDesc;
    use crate::record::{InstrumentDefMsg, MboMsg, Mbp10Msg};

    fn with_record_header_dtype(dtypes: Vec<(String, String)>) -> Vec<(String, String)> {
        let mut res = vec![
            ("length".to_owned(), "u1".to_owned()),
            ("rtype".to_owned(), "u1".to_owned()),
            ("publisher_id".to_owned(), "u2".to_owned()),
            ("instrument_id".to_owned(), "u4".to_owned()),
            ("ts_event".to_owned(), "u8".to_owned()),
        ];
        res.extend(dtypes);
        res
    }

    #[test]
    fn test_mbo_dtypes() {
        let dtypes = MboMsg::field_dtypes("");
        let exp = with_record_header_dtype(vec![
            ("order_id".to_owned(), "u8".to_owned()),
            ("price".to_owned(), "i8".to_owned()),
            ("size".to_owned(), "u4".to_owned()),
            ("flags".to_owned(), "u1".to_owned()),
            ("channel_id".to_owned(), "u1".to_owned()),
            ("action".to_owned(), "S1".to_owned()),
            ("side".to_owned(), "S1".to_owned()),
            ("ts_recv".to_owned(), "u8".to_owned()),
            ("ts_in_delta".to_owned(), "i4".to_owned()),
            ("sequence".to_owned(), "u4".to_owned()),
        ]);
        assert_eq!(dtypes, exp);
    }

    #[test]
    fn test_mbo_fields() {
        assert_eq!(MboMsg::price_fields(""), vec!["price".to_owned()]);
        assert_eq!(MboMsg::hidden_fields(""), vec!["length".to_owned()]);
        assert_eq!(
            MboMsg::timestamp_fields(""),
            vec!["ts_event".to_owned(), "ts_recv".to_owned()]
        );
    }

    #[test]
    fn test_mbo_ordered() {
        assert_eq!(
            MboMsg::ordered_fields(""),
            vec![
                "ts_recv".to_owned(),
                "ts_event".to_owned(),
                "rtype".to_owned(),
                "publisher_id".to_owned(),
                "instrument_id".to_owned(),
                "action".to_owned(),
                "side".to_owned(),
                "price".to_owned(),
                "size".to_owned(),
                "channel_id".to_owned(),
                "order_id".to_owned(),
                "flags".to_owned(),
                "ts_in_delta".to_owned(),
                "sequence".to_owned(),
            ]
        )
    }

    #[test]
    fn test_mbp10_dtypes() {
        let dtypes = Mbp10Msg::field_dtypes("");
        let mut exp = with_record_header_dtype(vec![
            ("price".to_owned(), "i8".to_owned()),
            ("size".to_owned(), "u4".to_owned()),
            ("action".to_owned(), "S1".to_owned()),
            ("side".to_owned(), "S1".to_owned()),
            ("flags".to_owned(), "u1".to_owned()),
            ("depth".to_owned(), "u1".to_owned()),
            ("ts_recv".to_owned(), "u8".to_owned()),
            ("ts_in_delta".to_owned(), "i4".to_owned()),
            ("sequence".to_owned(), "u4".to_owned()),
        ]);
        for i in 0..10 {
            exp.push((format!("bid_px_{i:02}"), "i8".to_owned()));
            exp.push((format!("ask_px_{i:02}"), "i8".to_owned()));
            exp.push((format!("bid_sz_{i:02}"), "u4".to_owned()));
            exp.push((format!("ask_sz_{i:02}"), "u4".to_owned()));
            exp.push((format!("bid_ct_{i:02}"), "u4".to_owned()));
            exp.push((format!("ask_ct_{i:02}"), "u4".to_owned()));
        }
        assert_eq!(dtypes, exp);
    }

    #[test]
    fn test_mbp10_fields() {
        let mut exp_price = vec!["price".to_owned()];
        for i in 0..10 {
            exp_price.push(format!("bid_px_{i:02}"));
            exp_price.push(format!("ask_px_{i:02}"));
        }
        assert_eq!(Mbp10Msg::price_fields(""), exp_price);
        assert_eq!(Mbp10Msg::hidden_fields(""), vec!["length".to_owned()]);
        assert_eq!(
            Mbp10Msg::timestamp_fields(""),
            vec!["ts_event".to_owned(), "ts_recv".to_owned()]
        );
    }

    #[test]
    fn test_mbp10_ordered() {
        let mut exp = vec![
            "ts_recv".to_owned(),
            "ts_event".to_owned(),
            "rtype".to_owned(),
            "publisher_id".to_owned(),
            "instrument_id".to_owned(),
            "action".to_owned(),
            "side".to_owned(),
            "depth".to_owned(),
            "price".to_owned(),
            "size".to_owned(),
            "flags".to_owned(),
            "ts_in_delta".to_owned(),
            "sequence".to_owned(),
        ];
        for i in 0..10 {
            exp.push(format!("bid_px_{i:02}"));
            exp.push(format!("ask_px_{i:02}"));
            exp.push(format!("bid_sz_{i:02}"));
            exp.push(format!("ask_sz_{i:02}"));
            exp.push(format!("bid_ct_{i:02}"));
            exp.push(format!("ask_ct_{i:02}"));
        }
        assert_eq!(Mbp10Msg::ordered_fields(""), exp)
    }

    #[test]
    fn test_definition_dtypes() {
        let dtypes = InstrumentDefMsg::field_dtypes("");
        let exp = with_record_header_dtype(vec![
            ("ts_recv".to_owned(), "u8".to_owned()),
            ("min_price_increment".to_owned(), "i8".to_owned()),
            ("display_factor".to_owned(), "i8".to_owned()),
            ("expiration".to_owned(), "u8".to_owned()),
            ("activation".to_owned(), "u8".to_owned()),
            ("high_limit_price".to_owned(), "i8".to_owned()),
            ("low_limit_price".to_owned(), "i8".to_owned()),
            ("max_price_variation".to_owned(), "i8".to_owned()),
            ("trading_reference_price".to_owned(), "i8".to_owned()),
            ("unit_of_measure_qty".to_owned(), "i8".to_owned()),
            ("min_price_increment_amount".to_owned(), "i8".to_owned()),
            ("price_ratio".to_owned(), "i8".to_owned()),
            ("inst_attrib_value".to_owned(), "i4".to_owned()),
            ("underlying_id".to_owned(), "u4".to_owned()),
            ("raw_instrument_id".to_owned(), "u4".to_owned()),
            ("market_depth_implied".to_owned(), "i4".to_owned()),
            ("market_depth".to_owned(), "i4".to_owned()),
            ("market_segment_id".to_owned(), "u4".to_owned()),
            ("max_trade_vol".to_owned(), "u4".to_owned()),
            ("min_lot_size".to_owned(), "i4".to_owned()),
            ("min_lot_size_block".to_owned(), "i4".to_owned()),
            ("min_lot_size_round_lot".to_owned(), "i4".to_owned()),
            ("min_trade_vol".to_owned(), "u4".to_owned()),
            ("_reserved2".to_owned(), "S4".to_owned()),
            ("contract_multiplier".to_owned(), "i4".to_owned()),
            ("decay_quantity".to_owned(), "i4".to_owned()),
            ("original_contract_size".to_owned(), "i4".to_owned()),
            ("_reserved3".to_owned(), "S4".to_owned()),
            ("trading_reference_date".to_owned(), "u2".to_owned()),
            ("appl_id".to_owned(), "i2".to_owned()),
            ("maturity_year".to_owned(), "u2".to_owned()),
            ("decay_start_date".to_owned(), "u2".to_owned()),
            ("channel_id".to_owned(), "u2".to_owned()),
            ("currency".to_owned(), "S4".to_owned()),
            ("settl_currency".to_owned(), "S4".to_owned()),
            ("secsubtype".to_owned(), "S6".to_owned()),
            ("raw_symbol".to_owned(), "S22".to_owned()),
            ("group".to_owned(), "S21".to_owned()),
            ("exchange".to_owned(), "S5".to_owned()),
            ("asset".to_owned(), "S7".to_owned()),
            ("cfi".to_owned(), "S7".to_owned()),
            ("security_type".to_owned(), "S7".to_owned()),
            ("unit_of_measure".to_owned(), "S31".to_owned()),
            ("underlying".to_owned(), "S21".to_owned()),
            ("strike_price_currency".to_owned(), "S4".to_owned()),
            ("instrument_class".to_owned(), "S1".to_owned()),
            ("_reserved4".to_owned(), "S2".to_owned()),
            ("strike_price".to_owned(), "i8".to_owned()),
            ("_reserved5".to_owned(), "S6".to_owned()),
            ("match_algorithm".to_owned(), "S1".to_owned()),
            ("md_security_trading_status".to_owned(), "u1".to_owned()),
            ("main_fraction".to_owned(), "u1".to_owned()),
            ("price_display_format".to_owned(), "u1".to_owned()),
            ("settl_price_type".to_owned(), "u1".to_owned()),
            ("sub_fraction".to_owned(), "u1".to_owned()),
            ("underlying_product".to_owned(), "u1".to_owned()),
            ("security_update_action".to_owned(), "S1".to_owned()),
            ("maturity_month".to_owned(), "u1".to_owned()),
            ("maturity_day".to_owned(), "u1".to_owned()),
            ("maturity_week".to_owned(), "u1".to_owned()),
            ("user_defined_instrument".to_owned(), "S1".to_owned()),
            ("contract_multiplier_unit".to_owned(), "i1".to_owned()),
            ("flow_schedule_type".to_owned(), "i1".to_owned()),
            ("tick_rule".to_owned(), "u1".to_owned()),
            ("_dummy".to_owned(), "S3".to_owned()),
        ]);
        assert_eq!(dtypes, exp);
    }

    #[test]
    fn test_definition_fields() {
        assert_eq!(
            InstrumentDefMsg::price_fields(""),
            vec![
                "min_price_increment".to_owned(),
                "high_limit_price".to_owned(),
                "low_limit_price".to_owned(),
                "max_price_variation".to_owned(),
                "trading_reference_price".to_owned(),
                "min_price_increment_amount".to_owned(),
                "price_ratio".to_owned(),
                "strike_price".to_owned(),
            ]
        );
        assert_eq!(
            InstrumentDefMsg::hidden_fields(""),
            vec![
                "length".to_owned(),
                "_reserved2".to_owned(),
                "_reserved3".to_owned(),
                "_reserved4".to_owned(),
                "_reserved5".to_owned(),
                "_dummy".to_owned(),
            ]
        );
        assert_eq!(
            InstrumentDefMsg::timestamp_fields(""),
            vec![
                "ts_event".to_owned(),
                "ts_recv".to_owned(),
                "expiration".to_owned(),
                "activation".to_owned()
            ]
        );
    }

    #[test]
    fn test_definition_ordered() {
        assert_eq!(
            InstrumentDefMsg::ordered_fields(""),
            vec![
                "ts_recv".to_owned(),
                "ts_event".to_owned(),
                "rtype".to_owned(),
                "publisher_id".to_owned(),
                "instrument_id".to_owned(),
                "raw_symbol".to_owned(),
                "security_update_action".to_owned(),
                "instrument_class".to_owned(),
                "min_price_increment".to_owned(),
                "display_factor".to_owned(),
                "expiration".to_owned(),
                "activation".to_owned(),
                "high_limit_price".to_owned(),
                "low_limit_price".to_owned(),
                "max_price_variation".to_owned(),
                "trading_reference_price".to_owned(),
                "unit_of_measure_qty".to_owned(),
                "min_price_increment_amount".to_owned(),
                "price_ratio".to_owned(),
                "inst_attrib_value".to_owned(),
                "underlying_id".to_owned(),
                "raw_instrument_id".to_owned(),
                "market_depth_implied".to_owned(),
                "market_depth".to_owned(),
                "market_segment_id".to_owned(),
                "max_trade_vol".to_owned(),
                "min_lot_size".to_owned(),
                "min_lot_size_block".to_owned(),
                "min_lot_size_round_lot".to_owned(),
                "min_trade_vol".to_owned(),
                "contract_multiplier".to_owned(),
                "decay_quantity".to_owned(),
                "original_contract_size".to_owned(),
                "trading_reference_date".to_owned(),
                "appl_id".to_owned(),
                "maturity_year".to_owned(),
                "decay_start_date".to_owned(),
                "channel_id".to_owned(),
                "currency".to_owned(),
                "settl_currency".to_owned(),
                "secsubtype".to_owned(),
                "group".to_owned(),
                "exchange".to_owned(),
                "asset".to_owned(),
                "cfi".to_owned(),
                "security_type".to_owned(),
                "unit_of_measure".to_owned(),
                "underlying".to_owned(),
                "strike_price_currency".to_owned(),
                "strike_price".to_owned(),
                "match_algorithm".to_owned(),
                "md_security_trading_status".to_owned(),
                "main_fraction".to_owned(),
                "price_display_format".to_owned(),
                "settl_price_type".to_owned(),
                "sub_fraction".to_owned(),
                "underlying_product".to_owned(),
                "maturity_month".to_owned(),
                "maturity_day".to_owned(),
                "maturity_week".to_owned(),
                "user_defined_instrument".to_owned(),
                "contract_multiplier_unit".to_owned(),
                "flow_schedule_type".to_owned(),
                "tick_rule".to_owned(),
            ]
        )
    }
}
