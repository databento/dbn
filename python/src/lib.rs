//! Python bindings for the [`dbn`] crate using [`pyo3`].

use pyo3::{prelude::*, wrap_pyfunction, PyClass};

use dbn::{
    compat::{
        ErrorMsgV1, InstrumentDefMsgV1, InstrumentDefMsgV2, StatMsgV1, SymbolMappingMsgV1,
        SystemMsgV1,
    },
    flags,
    python::{DBNError, EnumIterator},
    Action, BboMsg, BidAskPair, CbboMsg, Cmbp1Msg, Compression, ConsolidatedBidAskPair, Encoding,
    ErrorCode, ErrorMsg, ImbalanceMsg, InstrumentClass, InstrumentDefMsg, MatchAlgorithm, MboMsg,
    Mbp10Msg, Mbp1Msg, Metadata, OhlcvMsg, RType, SType, Schema, SecurityUpdateAction, Side,
    StatMsg, StatType, StatUpdateAction, StatusAction, StatusMsg, StatusReason, SymbolMappingMsg,
    SystemCode, SystemMsg, TradeMsg, TradingEvent, TriState, UserDefinedInstrument,
    VersionUpgradePolicy, DBN_VERSION, FIXED_PRICE_SCALE, UNDEF_ORDER_SIZE, UNDEF_PRICE,
    UNDEF_STAT_QUANTITY, UNDEF_TIMESTAMP,
};

mod dbn_decoder;
mod encode;
mod enums;
mod transcoder;

/// A Python module wrapping dbn functions
#[pymodule] // The name of the function must match `lib.name` in `Cargo.toml`
#[pyo3(name = "_lib")]
fn databento_dbn(_py: Python<'_>, m: &Bound<PyModule>) -> PyResult<()> {
    fn checked_add_class<T: PyClass>(m: &Bound<PyModule>) -> PyResult<()> {
        // ensure a module was specified, otherwise it defaults to builtins
        assert_eq!(T::MODULE.unwrap(), "databento_dbn");
        m.add_class::<T>()
    }
    // all functions exposed to Python need to be added here
    m.add_wrapped(wrap_pyfunction!(encode::update_encoded_metadata))?;
    m.add("DBNError", m.py().get_type::<DBNError>())?;
    checked_add_class::<EnumIterator>(m)?;
    checked_add_class::<Metadata>(m)?;
    checked_add_class::<dbn_decoder::DbnDecoder>(m)?;
    checked_add_class::<transcoder::Transcoder>(m)?;
    // Records
    checked_add_class::<MboMsg>(m)?;
    checked_add_class::<BidAskPair>(m)?;
    checked_add_class::<ConsolidatedBidAskPair>(m)?;
    checked_add_class::<TradeMsg>(m)?;
    checked_add_class::<Mbp1Msg>(m)?;
    checked_add_class::<Mbp10Msg>(m)?;
    checked_add_class::<OhlcvMsg>(m)?;
    checked_add_class::<ImbalanceMsg>(m)?;
    checked_add_class::<StatusMsg>(m)?;
    checked_add_class::<InstrumentDefMsg>(m)?;
    checked_add_class::<InstrumentDefMsgV2>(m)?;
    checked_add_class::<InstrumentDefMsgV1>(m)?;
    checked_add_class::<ErrorMsg>(m)?;
    checked_add_class::<ErrorMsgV1>(m)?;
    checked_add_class::<SymbolMappingMsg>(m)?;
    checked_add_class::<SymbolMappingMsgV1>(m)?;
    checked_add_class::<SystemMsg>(m)?;
    checked_add_class::<SystemMsgV1>(m)?;
    checked_add_class::<StatMsg>(m)?;
    checked_add_class::<StatMsgV1>(m)?;
    checked_add_class::<BboMsg>(m)?;
    checked_add_class::<CbboMsg>(m)?;
    checked_add_class::<Cmbp1Msg>(m)?;
    // PyClass enums
    checked_add_class::<Action>(m)?;
    checked_add_class::<Compression>(m)?;
    checked_add_class::<Encoding>(m)?;
    checked_add_class::<ErrorCode>(m)?;
    checked_add_class::<InstrumentClass>(m)?;
    checked_add_class::<MatchAlgorithm>(m)?;
    checked_add_class::<RType>(m)?;
    checked_add_class::<SType>(m)?;
    checked_add_class::<Schema>(m)?;
    checked_add_class::<SecurityUpdateAction>(m)?;
    checked_add_class::<Side>(m)?;
    checked_add_class::<StatType>(m)?;
    checked_add_class::<StatUpdateAction>(m)?;
    checked_add_class::<StatusAction>(m)?;
    checked_add_class::<StatusReason>(m)?;
    checked_add_class::<SystemCode>(m)?;
    checked_add_class::<TradingEvent>(m)?;
    checked_add_class::<TriState>(m)?;
    checked_add_class::<UserDefinedInstrument>(m)?;
    checked_add_class::<VersionUpgradePolicy>(m)?;
    // constants
    m.add("DBN_VERSION", DBN_VERSION)?;
    m.add("FIXED_PRICE_SCALE", FIXED_PRICE_SCALE)?;
    m.add("UNDEF_PRICE", UNDEF_PRICE)?;
    m.add("UNDEF_ORDER_SIZE", UNDEF_ORDER_SIZE)?;
    m.add("UNDEF_STAT_QUANTITY", UNDEF_STAT_QUANTITY)?;
    m.add("UNDEF_TIMESTAMP", UNDEF_TIMESTAMP)?;
    m.add("F_LAST", flags::LAST)?;
    m.add("F_TOB", flags::TOB)?;
    m.add("F_SNAPSHOT", flags::SNAPSHOT)?;
    m.add("F_MBP", flags::MBP)?;
    m.add("F_BAD_TS_RECV", flags::BAD_TS_RECV)?;
    m.add("F_MAYBE_BAD_BOOK", flags::MAYBE_BAD_BOOK)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use dbn::enums::SType;
    use pyo3::ffi::c_str;
    use rstest::*;

    use super::*;

    pub const TEST_DATA_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/data");

    static INIT: Once = Once::new();

    #[fixture]
    pub fn python() {
        INIT.call_once(|| {
            // add to available modules
            pyo3::append_to_inittab!(databento_dbn);
            // initialize interpreter
            pyo3::prepare_freethreaded_python();
        });
    }

    #[rstest]
    fn test_metadata_identity(_python: ()) {
        let stype_in = SType::RawSymbol as u8;
        let stype_out = SType::InstrumentId as u8;
        Python::with_gil(|py| {
            pyo3::py_run!(
                  py,
                  stype_in stype_out,
                  r#"from _lib import Metadata, Schema, SType

metadata = Metadata(
    dataset="GLBX.MDP3",
    schema=Schema.MBO,
    start=1,
    stype_in=SType.RAW_SYMBOL,
    stype_out=SType.INSTRUMENT_ID,
    end=2,
    symbols=[],
    partial=[],
    not_found=[],
    mappings=[]
)
metadata_bytes = metadata.encode()
metadata = Metadata.decode(metadata_bytes)
assert metadata.dataset == "GLBX.MDP3"
assert metadata.schema == Schema.MBO
assert metadata.start == 1
assert metadata.end == 2
assert metadata.limit is None
assert metadata.stype_in == SType.RAW_SYMBOL
assert metadata.stype_out == SType.INSTRUMENT_ID
assert metadata.ts_out is False"#
            );
        });
    }

    #[rstest]
    fn test_dbn_decoder_metadata_error(_python: ()) {
        Python::with_gil(|py| {
            py.run(
                c_str!(
                    r#"from _lib import DBNDecoder

decoder = DBNDecoder()
try:
    records = decoder.decode()
    # If this code is called, the test will fail
    assert False
except Exception:
    pass
"#
                ),
                None,
                None,
            )
        })
        .unwrap();
    }

    #[rstest]
    #[case("InstrumentClass", "CALL", "C")]
    #[case("SType", "CONTINUOUS", "continuous")]
    #[case("Action", "CLEAR", "R")]
    #[case("Schema", "MBO", "mbo")]
    fn test_enum_str_hash(
        _python: (),
        #[case] enum_name: &str,
        #[case] variant: &str,
        #[case] val: &str,
    ) {
        Python::with_gil(|py| {
            pyo3::py_run!(
                  py,
                  enum_name variant val,
                  r#"import _lib as db

enum_type = getattr(db, enum_name)
variant = getattr(enum_type, variant)
assert variant == enum_type(val)
assert variant == val
assert val == variant
assert hash(val) == hash(variant), f"{val = }, {variant = } {hash(val) = }, {hash(variant) = }""#
            );
        });
    }

    #[rstest]
    #[case("RType", "MBO", RType::Mbo as u32)]
    #[case("StatType", "OPEN_INTEREST", StatType::OpenInterest as u32)]
    #[case("ErrorCode", "INTERNAL_ERROR", ErrorCode::InternalError as u32)]
    #[case("StatusReason", "NEWS_PENDING", StatusReason::NewsPending as u32)]
    fn test_enum_int_hash(
        _python: (),
        #[case] enum_name: &str,
        #[case] variant: &str,
        #[case] val: u32,
    ) {
        Python::with_gil(|py| {
            pyo3::py_run!(
                  py,
                  enum_name variant val,
                  r#"import _lib as db

enum_type = getattr(db, enum_name)
variant = getattr(enum_type, variant)
assert variant == enum_type(val)
assert variant == val
assert val == variant
assert hash(val) == hash(variant), f"{val = }, {variant = } {hash(val) = }, {hash(variant) = }""#
            );
        });
    }
}
