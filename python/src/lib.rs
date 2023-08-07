//! Python bindings for the [`dbn`] crate using [`pyo3`].

use pyo3::{prelude::*, wrap_pyfunction, PyClass};

use dbn::{
    enums::{Compression, Encoding, SType, Schema},
    python::EnumIterator,
    record::{
        BidAskPair, ErrorMsg, ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg,
        RecordHeader, StatMsg, StatusMsg, SymbolMappingMsg, SystemMsg, TradeMsg,
    },
    Metadata, FIXED_PRICE_SCALE,
};

mod dbn_decoder;
mod encode;

/// A Python module wrapping dbn functions
#[pymodule] // The name of the function must match `lib.name` in `Cargo.toml`
fn databento_dbn(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    fn checked_add_class<T: PyClass>(m: &PyModule) -> PyResult<()> {
        // ensure a module was specified, otherwise it defaults to builtins
        assert_eq!(T::MODULE.unwrap(), "databento_dbn");
        m.add_class::<T>()
    }
    // all functions exposed to Python need to be added here
    m.add_wrapped(wrap_pyfunction!(encode::update_encoded_metadata))?;
    m.add_wrapped(wrap_pyfunction!(encode::write_dbn_file))?;
    checked_add_class::<dbn_decoder::DbnDecoder>(m)?;
    checked_add_class::<Metadata>(m)?;
    checked_add_class::<EnumIterator>(m)?;
    // Records
    checked_add_class::<RecordHeader>(m)?;
    checked_add_class::<MboMsg>(m)?;
    checked_add_class::<BidAskPair>(m)?;
    checked_add_class::<TradeMsg>(m)?;
    checked_add_class::<Mbp1Msg>(m)?;
    checked_add_class::<Mbp10Msg>(m)?;
    checked_add_class::<OhlcvMsg>(m)?;
    checked_add_class::<ImbalanceMsg>(m)?;
    checked_add_class::<StatusMsg>(m)?;
    checked_add_class::<InstrumentDefMsg>(m)?;
    checked_add_class::<ErrorMsg>(m)?;
    checked_add_class::<SymbolMappingMsg>(m)?;
    checked_add_class::<SystemMsg>(m)?;
    checked_add_class::<StatMsg>(m)?;
    // PyClass enums
    checked_add_class::<Compression>(m)?;
    checked_add_class::<Encoding>(m)?;
    checked_add_class::<Schema>(m)?;
    checked_add_class::<SType>(m)?;
    // constants
    m.add("FIXED_PRICE_SCALE", FIXED_PRICE_SCALE)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use dbn::enums::SType;

    use super::*;

    pub fn setup() {
        if unsafe { pyo3::ffi::Py_IsInitialized() } == 0 {
            // add to available modules
            pyo3::append_to_inittab!(databento_dbn);
        }
        // initialize interpreter
        pyo3::prepare_freethreaded_python();
    }

    #[test]
    fn test_metadata_identity() {
        // initialize interpreter
        setup();
        let stype_in = SType::RawSymbol as u8;
        let stype_out = SType::InstrumentId as u8;
        Python::with_gil(|py| {
            pyo3::py_run!(
                  py,
                  stype_in stype_out,
                  r#"from databento_dbn import Metadata, Schema, SType

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

    #[test]
    fn test_dbn_decoder_metadata_error() {
        setup();
        Python::with_gil(|py| {
            py.run(
                r#"from databento_dbn import DBNDecoder

decoder = DBNDecoder()
try:
    records = decoder.decode()
    # If this code is called, the test will fail
    assert False
except Exception:
    pass
"#,
                None,
                None,
            )
        })
        .unwrap();
    }
}
