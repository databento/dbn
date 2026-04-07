//! Python bindings for the [`dbn`] crate using [`pyo3`].

use pyo3::{prelude::*, wrap_pyfunction, PyClass};

use dbn::{
    flags,
    python::{
        record::{
            PyBboMsg, PyCbboMsg, PyCmbp1Msg, PyErrorMsg, PyImbalanceMsg, PyInstrumentDefMsg,
            PyMboMsg, PyMbp10Msg, PyMbp1Msg, PyOhlcvMsg, PyStatMsg, PyStatusMsg,
            PySymbolMappingMsg, PySystemMsg, PyTradeMsg, PyV1ErrorMsg, PyV1InstrumentDefMsg,
            PyV1StatMsg, PyV1SymbolMappingMsg, PyV1SystemMsg, PyV2InstrumentDefMsg,
        },
        DBNError, EnumIterator,
    },
    Action, BidAskPair, Compression, ConsolidatedBidAskPair, Encoding, ErrorCode, InstrumentClass,
    MatchAlgorithm, Metadata, RType, SType, Schema, SecurityUpdateAction, Side, StatType,
    StatUpdateAction, StatusAction, StatusReason, SystemCode, TradingEvent, TriState,
    UserDefinedInstrument, VersionUpgradePolicy, DBN_VERSION, FIXED_PRICE_SCALE, SYMBOL_CSTR_LEN,
    UNDEF_ORDER_SIZE, UNDEF_PRICE, UNDEF_STAT_QUANTITY, UNDEF_TIMESTAMP,
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
    checked_add_class::<PyMboMsg>(m)?;
    checked_add_class::<BidAskPair>(m)?;
    checked_add_class::<ConsolidatedBidAskPair>(m)?;
    checked_add_class::<PyTradeMsg>(m)?;
    checked_add_class::<PyMbp1Msg>(m)?;
    checked_add_class::<PyMbp10Msg>(m)?;
    checked_add_class::<PyOhlcvMsg>(m)?;
    checked_add_class::<PyImbalanceMsg>(m)?;
    checked_add_class::<PyStatusMsg>(m)?;
    checked_add_class::<PyInstrumentDefMsg>(m)?;
    checked_add_class::<PyV2InstrumentDefMsg>(m)?;
    checked_add_class::<PyV1InstrumentDefMsg>(m)?;
    checked_add_class::<PyErrorMsg>(m)?;
    checked_add_class::<PyV1ErrorMsg>(m)?;
    checked_add_class::<PySymbolMappingMsg>(m)?;
    checked_add_class::<PyV1SymbolMappingMsg>(m)?;
    checked_add_class::<PySystemMsg>(m)?;
    checked_add_class::<PyV1SystemMsg>(m)?;
    checked_add_class::<PyStatMsg>(m)?;
    checked_add_class::<PyV1StatMsg>(m)?;
    checked_add_class::<PyBboMsg>(m)?;
    checked_add_class::<PyCbboMsg>(m)?;
    checked_add_class::<PyCmbp1Msg>(m)?;
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
    m.add("SYMBOL_CSTR_LEN", SYMBOL_CSTR_LEN)?;
    m.add("SYMBOL_CSTR_LEN_V1", dbn::compat::SYMBOL_CSTR_LEN_V1)?;
    m.add("SYMBOL_CSTR_LEN_V2", dbn::compat::SYMBOL_CSTR_LEN_V2)?;
    m.add("SYMBOL_CSTR_LEN_V3", dbn::compat::SYMBOL_CSTR_LEN_V3)?;
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
    m.add("F_PUBLISHER_SPECIFIC", flags::PUBLISHER_SPECIFIC)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use dbn::enums::SType;
    use pyo3::ffi::c_str;
    use pyo3::types::PyDict;
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
            Python::initialize();
        });
    }

    #[rstest]
    fn test_metadata_identity(_python: ()) {
        let stype_in = SType::RawSymbol as u8;
        let stype_out = SType::InstrumentId as u8;
        Python::attach(|py| {
            let globals = PyDict::new(py);
            globals.set_item("stype_in", stype_in).unwrap();
            globals.set_item("stype_out", stype_out).unwrap();
            Python::run(
                py,
                c_str!(
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
                ),
                Some(&globals),
                None,
            )
            .unwrap();
        });
    }

    #[rstest]
    fn test_dbn_decoder_metadata_error(_python: ()) {
        Python::attach(|py| {
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
                // Create an empty `globals` dict to keep tests hermetic
                Some(&PyDict::new(py)),
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
        Python::attach(|py| {
            let globals = PyDict::new(py);
            globals.set_item("enum_name", enum_name).unwrap();
            globals.set_item("variant", variant).unwrap();
            globals.set_item("val", val).unwrap();
            Python::run(
                py,
                c_str!(
                    r#"import _lib as db

enum_type = getattr(db, enum_name)
variant = getattr(enum_type, variant)
assert variant == enum_type(val)
assert variant == val
assert val == variant
assert hash(val) == hash(variant)"#
                ),
                Some(&globals),
                None,
            )
            .unwrap();
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
        Python::attach(|py| {
            let globals = PyDict::new(py);
            globals.set_item("enum_name", enum_name).unwrap();
            globals.set_item("variant", variant).unwrap();
            globals.set_item("val", val).unwrap();
            Python::run(
                py,
                c_str!(
                    r#"import _lib as db

enum_type = getattr(db, enum_name)
variant = getattr(enum_type, variant)
assert variant == enum_type(val)
assert variant == val
assert val == variant
assert hash(val) == hash(variant)"#
                ),
                Some(&globals),
                None,
            )
            .unwrap();
        });
    }

    #[rstest]
    #[case("RType", "MBO", RType::Mbo as u32)]
    #[case("Schema", "MBO", Schema::Mbo as u32)]
    #[case("Action", "TRADE", b'T' as u32)]
    #[case("Side", "BID", b'B' as u32)]
    #[case("InstrumentClass", "CALL", b'C' as u32)]
    fn test_enum_index(
        _python: (),
        #[case] enum_name: &str,
        #[case] variant: &str,
        #[case] expected: u32,
    ) {
        Python::attach(|py| {
            let globals = PyDict::new(py);
            globals.set_item("enum_name", enum_name).unwrap();
            globals.set_item("variant", variant).unwrap();
            globals.set_item("expected", expected).unwrap();
            Python::run(
                py,
                c_str!(
                    r#"import _lib as db
import operator

enum_type = getattr(db, enum_name)
variant = getattr(enum_type, variant)
assert operator.index(variant) == expected
assert hex(variant) == hex(expected)
"#
                ),
                Some(&globals),
                None,
            )
            .unwrap();
        });
    }

    #[rstest]
    fn test_record_size_is_method(_python: ()) {
        Python::attach(|py| {
            Python::run(
                py,
                c_str!(
                    r#"from _lib import MBOMsg, Action, Side

msg = MBOMsg(
    publisher_id=1,
    instrument_id=2,
    ts_event=3,
    order_id=4,
    price=5,
    size=6,
    action=Action.ADD,
    side=Side.BID,
    ts_recv=7,
)
# record_size is a method, not a property
assert callable(msg.record_size)
assert msg.record_size() > 0
assert msg.record_size() == msg.size_hint
"#
                ),
                Some(&PyDict::new(py)),
                None,
            )
            .unwrap();
        });
    }

    /// Helper to run Python assertions against a record.
    fn run_py(py: Python<'_>, code: &str) {
        Python::run(
            py,
            &std::ffi::CString::new(code).unwrap(),
            Some(&PyDict::new(py)),
            None,
        )
        .unwrap();
    }

    const MBO_CTOR: &str = "MBOMsg(publisher_id=1, instrument_id=2, ts_event=3, order_id=4, price=5, size=6, action=Action.ADD, side=Side.BID, ts_recv=7)";
    const TRADE_CTOR: &str = "TradeMsg(1, 2, 3, 5, 6, Action.ADD, Side.BID, 0, 7)";
    const OHLCV_CTOR: &str = "OHLCVMsg(0x20, 1, 2, 3, 0, 0, 0, 0, 0)";

    #[rstest]
    #[case(MBO_CTOR)]
    #[case(TRADE_CTOR)]
    #[case(OHLCV_CTOR)]
    fn test_ts_out_default_and_settable(_python: (), #[case] ctor: &str) {
        Python::attach(|py| {
            run_py(
                py,
                &format!(
                    r#"from _lib import *
msg = {ctor}
assert msg.ts_out == UNDEF_TIMESTAMP
assert msg.pretty_ts_out is None
msg.ts_out = 1000
assert msg.ts_out == 1000
assert msg.pretty_ts_out is not None
"#
                ),
            );
        });
    }

    #[rstest]
    fn test_ts_out_in_constructor(_python: ()) {
        Python::attach(|py| {
            run_py(
                py,
                r#"from _lib import *
msg = MBOMsg(publisher_id=1, instrument_id=2, ts_event=3, order_id=4, price=5, size=6, action=Action.ADD, side=Side.BID, ts_recv=7, ts_out=42)
assert msg.ts_out == 42
"#,
            );
        });
    }

    #[rstest]
    #[case(MBO_CTOR)]
    #[case(TRADE_CTOR)]
    #[case(OHLCV_CTOR)]
    fn test_bytes_excludes_ts_out_when_not_set(_python: (), #[case] ctor: &str) {
        Python::attach(|py| {
            run_py(
                py,
                &format!(
                    r#"from _lib import *
msg = {ctor}
record_bytes = bytes(msg)
assert len(record_bytes) == msg.size_hint
assert len(record_bytes) == msg.record_size()
"#
                ),
            );
        });
    }

    #[rstest]
    #[case(MBO_CTOR)]
    #[case(TRADE_CTOR)]
    #[case(OHLCV_CTOR)]
    fn test_no_arbitrary_attrs_on_records(_python: (), #[case] ctor: &str) {
        Python::attach(|py| {
            run_py(
                py,
                &format!(
                    r#"from _lib import *
msg = {ctor}
try:
    msg.foo = "bar"
    assert False, "should not allow arbitrary attrs"
except AttributeError:
    pass
"#
                ),
            );
        });
    }

    #[rstest]
    #[case(MBO_CTOR)]
    #[case(TRADE_CTOR)]
    #[case(OHLCV_CTOR)]
    fn test_bytes_includes_ts_out_when_set(_python: (), #[case] ctor: &str) {
        Python::attach(|py| {
            run_py(
                py,
                &format!(
                    r#"import struct
from _lib import *
msg = {ctor}
base_size = msg.record_size()
msg.ts_out = 42
record_bytes = bytes(msg)
assert len(record_bytes) == base_size + 8
assert len(record_bytes) == msg.record_size()
# last 8 bytes should be ts_out in little-endian
ts_out_bytes = struct.unpack('<Q', record_bytes[-8:])[0]
assert ts_out_bytes == 42
"#
                ),
            );
        });
    }

    #[rstest]
    fn test_bytes_with_ts_out_in_constructor(_python: ()) {
        Python::attach(|py| {
            run_py(
                py,
                r#"import struct
from _lib import *
msg = MBOMsg(publisher_id=1, instrument_id=2, ts_event=3, order_id=4, price=5, size=6, action=Action.ADD, side=Side.BID, ts_recv=7)
base_size = msg.record_size()
msg_with_ts = MBOMsg(publisher_id=1, instrument_id=2, ts_event=3, order_id=4, price=5, size=6, action=Action.ADD, side=Side.BID, ts_recv=7, ts_out=100)
record_bytes = bytes(msg_with_ts)
assert len(record_bytes) == base_size + 8
ts_out_bytes = struct.unpack('<Q', record_bytes[-8:])[0]
assert ts_out_bytes == 100
"#,
            );
        });
    }

    #[rstest]
    #[case(MBO_CTOR)]
    #[case(TRADE_CTOR)]
    #[case(OHLCV_CTOR)]
    fn test_ts_out_setter_restores_size(_python: (), #[case] ctor: &str) {
        Python::attach(|py| {
            run_py(
                py,
                &format!(
                    r#"from _lib import *
msg = {ctor}
base_size = msg.record_size()
# set ts_out, size should grow
msg.ts_out = 42
assert msg.record_size() == base_size + 8
# reset to UNDEF, size should shrink back
msg.ts_out = UNDEF_TIMESTAMP
assert msg.record_size() == base_size
assert len(bytes(msg)) == base_size
"#
                ),
            );
        });
    }

    #[rstest]
    fn test_record_field_setters(_python: ()) {
        Python::attach(|py| {
            Python::run(
                py,
                c_str!(
                    r#"from _lib import MBOMsg, Action, Side

msg = MBOMsg(
    publisher_id=1,
    instrument_id=2,
    ts_event=3,
    order_id=4,
    price=5,
    size=6,
    action=Action.ADD,
    side=Side.BID,
    ts_recv=7,
)
# Test header field setters
msg.publisher_id = 10
assert msg.publisher_id == 10
msg.instrument_id = 20
assert msg.instrument_id == 20
msg.ts_event = 30
assert msg.ts_event == 30

# Test plain int field setter
msg.price = 100
assert msg.price == 100
msg.size = 200
assert msg.size == 200

# Test try_into field setter
msg.action = Action.CANCEL
assert msg.action == Action.CANCEL
msg.side = Side.ASK
assert msg.side == Side.ASK
"#
                ),
                Some(&PyDict::new(py)),
                None,
            )
            .unwrap();
        });
    }
}
