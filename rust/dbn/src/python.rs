//! Python wrappers around dbn functions. These are implemented here instead of in `python/`
//! to be able to implement [`pyo3`] traits for DBN types.
#![allow(clippy::too_many_arguments)]

use std::{convert::Infallible, fmt};

use pyo3::{
    create_exception,
    exceptions::PyException,
    prelude::*,
    types::{PyDate, PyDateAccess, PyInt},
    IntoPyObjectExt,
};
use strum::IntoEnumIterator;

use crate::{Error, FlagSet};

mod enums;
mod metadata;
mod record;

create_exception!(
    databento_dbn,
    DBNError,
    PyException,
    "An exception from databento_dbn Rust code."
);

/// A helper function for converting any type that implements `Debug` to a Python
/// `ValueError`.
pub fn to_py_err(e: impl fmt::Display) -> PyErr {
    DBNError::new_err(format!("{e}"))
}

impl From<Error> for PyErr {
    fn from(err: Error) -> Self {
        DBNError::new_err(format!("{err}"))
    }
}

/// Python iterator over the variants of an enum.
#[pyclass(module = "databento_dbn")]
pub struct EnumIterator {
    // Type erasure for code reuse. Generic types can't be exposed to Python.
    iter: Box<dyn Iterator<Item = PyObject> + Send + Sync>,
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
    fn new<'py, E>(py: Python<'py>) -> PyResult<Self>
    where
        E: strum::IntoEnumIterator + IntoPyObject<'py>,
        <E as IntoEnumIterator>::Iterator: Send + Sync,
    {
        Ok(Self {
            iter: Box::new(
                E::iter()
                    .map(|var| var.into_py_any(py))
                    // force eager evaluation because `py` isn't `Send`
                    .collect::<PyResult<Vec<_>>>()?
                    .into_iter(),
            ),
        })
    }
}

impl<'py> IntoPyObject<'py> for FlagSet {
    type Target = PyInt;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.raw().into_pyobject(py)
    }
}

/// Tries to convert `py_date` to a [`time::Date`].
///
/// # Errors
/// This function returns an error if input has an invalid month.
pub fn py_to_time_date(py_date: &Bound<'_, PyDate>) -> PyResult<time::Date> {
    let month =
        time::Month::try_from(py_date.get_month()).map_err(|e| DBNError::new_err(e.to_string()))?;
    time::Date::from_calendar_date(py_date.get_year(), month, py_date.get_day())
        .map_err(|e| DBNError::new_err(e.to_string()))
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
impl PyFieldDesc for FlagSet {
    fn field_dtypes(field_name: &str) -> Vec<(String, String)> {
        u8::field_dtypes(field_name)
    }
}

#[cfg(test)]
mod tests {
    use super::PyFieldDesc;
    use crate::{
        record::{Cmbp1Msg, InstrumentDefMsg, MboMsg, Mbp10Msg},
        ASSET_CSTR_LEN, SYMBOL_CSTR_LEN,
    };

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
    fn test_cmbp1_dtypes() {
        let dtypes = Cmbp1Msg::field_dtypes("");
        let exp = with_record_header_dtype(vec![
            ("price".to_owned(), "i8".to_owned()),
            ("size".to_owned(), "u4".to_owned()),
            ("action".to_owned(), "S1".to_owned()),
            ("side".to_owned(), "S1".to_owned()),
            ("flags".to_owned(), "u1".to_owned()),
            ("_reserved1".to_owned(), "S1".to_owned()),
            ("ts_recv".to_owned(), "u8".to_owned()),
            ("ts_in_delta".to_owned(), "i4".to_owned()),
            ("_reserved2".to_owned(), "S4".to_owned()),
            ("bid_px_00".to_owned(), "i8".to_owned()),
            ("ask_px_00".to_owned(), "i8".to_owned()),
            ("bid_sz_00".to_owned(), "u4".to_owned()),
            ("ask_sz_00".to_owned(), "u4".to_owned()),
            ("bid_pb_00".to_owned(), "u2".to_owned()),
            ("_reserved1_00".to_owned(), "S2".to_owned()),
            ("ask_pb_00".to_owned(), "u2".to_owned()),
            ("_reserved2_00".to_owned(), "S2".to_owned()),
        ]);
        assert_eq!(dtypes, exp);
    }

    #[test]
    fn test_cbbo_fields() {
        let mut exp_price = vec!["price".to_owned()];
        exp_price.push("bid_px_00".to_owned());
        exp_price.push("ask_px_00".to_owned());
        assert_eq!(Cmbp1Msg::price_fields(""), exp_price);
        assert_eq!(
            Cmbp1Msg::hidden_fields(""),
            vec![
                "length".to_owned(),
                "_reserved1".to_owned(),
                "_reserved2".to_owned(),
                "_reserved1_00".to_owned(),
                "_reserved2_00".to_owned()
            ]
        );
        assert_eq!(
            Cmbp1Msg::timestamp_fields(""),
            vec!["ts_event".to_owned(), "ts_recv".to_owned()]
        );
    }

    #[test]
    fn test_cbbo_ordered() {
        let exp = vec![
            "ts_recv".to_owned(),
            "ts_event".to_owned(),
            "rtype".to_owned(),
            "publisher_id".to_owned(),
            "instrument_id".to_owned(),
            "action".to_owned(),
            "side".to_owned(),
            "price".to_owned(),
            "size".to_owned(),
            "flags".to_owned(),
            "ts_in_delta".to_owned(),
            "bid_px_00".to_owned(),
            "ask_px_00".to_owned(),
            "bid_sz_00".to_owned(),
            "ask_sz_00".to_owned(),
            "bid_pb_00".to_owned(),
            "ask_pb_00".to_owned(),
        ];
        assert_eq!(Cmbp1Msg::ordered_fields(""), exp)
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
            ("unit_of_measure_qty".to_owned(), "i8".to_owned()),
            ("min_price_increment_amount".to_owned(), "i8".to_owned()),
            ("price_ratio".to_owned(), "i8".to_owned()),
            ("strike_price".to_owned(), "i8".to_owned()),
            ("raw_instrument_id".to_owned(), "u8".to_owned()),
            ("leg_price".to_owned(), "i8".to_owned()),
            ("leg_delta".to_owned(), "i8".to_owned()),
            ("inst_attrib_value".to_owned(), "i4".to_owned()),
            ("underlying_id".to_owned(), "u4".to_owned()),
            ("market_depth_implied".to_owned(), "i4".to_owned()),
            ("market_depth".to_owned(), "i4".to_owned()),
            ("market_segment_id".to_owned(), "u4".to_owned()),
            ("max_trade_vol".to_owned(), "u4".to_owned()),
            ("min_lot_size".to_owned(), "i4".to_owned()),
            ("min_lot_size_block".to_owned(), "i4".to_owned()),
            ("min_lot_size_round_lot".to_owned(), "i4".to_owned()),
            ("min_trade_vol".to_owned(), "u4".to_owned()),
            ("contract_multiplier".to_owned(), "i4".to_owned()),
            ("decay_quantity".to_owned(), "i4".to_owned()),
            ("original_contract_size".to_owned(), "i4".to_owned()),
            ("leg_instrument_id".to_owned(), "u4".to_owned()),
            ("leg_ratio_price_numerator".to_owned(), "i4".to_owned()),
            ("leg_ratio_price_denominator".to_owned(), "i4".to_owned()),
            ("leg_ratio_qty_numerator".to_owned(), "i4".to_owned()),
            ("leg_ratio_qty_denominator".to_owned(), "i4".to_owned()),
            ("leg_underlying_id".to_owned(), "u4".to_owned()),
            ("appl_id".to_owned(), "i2".to_owned()),
            ("maturity_year".to_owned(), "u2".to_owned()),
            ("decay_start_date".to_owned(), "u2".to_owned()),
            ("channel_id".to_owned(), "u2".to_owned()),
            ("leg_count".to_owned(), "u2".to_owned()),
            ("leg_index".to_owned(), "u2".to_owned()),
            ("currency".to_owned(), "S4".to_owned()),
            ("settl_currency".to_owned(), "S4".to_owned()),
            ("secsubtype".to_owned(), "S6".to_owned()),
            ("raw_symbol".to_owned(), format!("S{SYMBOL_CSTR_LEN}")),
            ("group".to_owned(), "S21".to_owned()),
            ("exchange".to_owned(), "S5".to_owned()),
            ("asset".to_owned(), format!("S{ASSET_CSTR_LEN}")),
            ("cfi".to_owned(), "S7".to_owned()),
            ("security_type".to_owned(), "S7".to_owned()),
            ("unit_of_measure".to_owned(), "S31".to_owned()),
            ("underlying".to_owned(), "S21".to_owned()),
            ("strike_price_currency".to_owned(), "S4".to_owned()),
            ("leg_raw_symbol".to_owned(), format!("S{SYMBOL_CSTR_LEN}")),
            ("instrument_class".to_owned(), "S1".to_owned()),
            ("match_algorithm".to_owned(), "S1".to_owned()),
            ("main_fraction".to_owned(), "u1".to_owned()),
            ("price_display_format".to_owned(), "u1".to_owned()),
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
            ("leg_instrument_class".to_owned(), "S1".to_owned()),
            ("leg_side".to_owned(), "S1".to_owned()),
            ("_reserved".to_owned(), "S17".to_owned()),
        ]);
        assert_eq!(dtypes, exp);
    }

    #[test]
    fn test_definition_fields() {
        assert_eq!(
            InstrumentDefMsg::price_fields(""),
            vec![
                "min_price_increment".to_owned(),
                "display_factor".to_owned(),
                "high_limit_price".to_owned(),
                "low_limit_price".to_owned(),
                "max_price_variation".to_owned(),
                "unit_of_measure_qty".to_owned(),
                "min_price_increment_amount".to_owned(),
                "price_ratio".to_owned(),
                "strike_price".to_owned(),
                "leg_price".to_owned(),
                "leg_delta".to_owned(),
            ]
        );
        assert_eq!(
            InstrumentDefMsg::hidden_fields(""),
            vec!["length".to_owned(), "_reserved".to_owned(),]
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
                "main_fraction".to_owned(),
                "price_display_format".to_owned(),
                "sub_fraction".to_owned(),
                "underlying_product".to_owned(),
                "maturity_month".to_owned(),
                "maturity_day".to_owned(),
                "maturity_week".to_owned(),
                "user_defined_instrument".to_owned(),
                "contract_multiplier_unit".to_owned(),
                "flow_schedule_type".to_owned(),
                "tick_rule".to_owned(),
                "leg_count".to_owned(),
                "leg_index".to_owned(),
                "leg_instrument_id".to_owned(),
                "leg_raw_symbol".to_owned(),
                "leg_instrument_class".to_owned(),
                "leg_side".to_owned(),
                "leg_price".to_owned(),
                "leg_delta".to_owned(),
                "leg_ratio_price_numerator".to_owned(),
                "leg_ratio_price_denominator".to_owned(),
                "leg_ratio_qty_numerator".to_owned(),
                "leg_ratio_qty_denominator".to_owned(),
                "leg_underlying_id".to_owned(),
            ]
        )
    }
}
