//! Python bindings for the [`dbn`] crate using [`pyo3`].
use std::io::{self, Write};

use pyo3::{exceptions::PyIOError, prelude::*, wrap_pyfunction, PyClass};

use dbn::{
    decode::dbn::{MetadataDecoder, RecordDecoder},
    enums::rtype,
    python::to_val_err,
    record::{
        BidAskPair, ErrorMsg, ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg,
        RecordHeader, SymbolMappingMsg, SystemMsg, TradeMsg,
    },
};

/// A Python module wrapping dbn functions
#[pymodule] // The name of the function must match `lib.name` in `Cargo.toml`
fn databento_dbn(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    fn checked_add_class<T: PyClass>(m: &PyModule) -> PyResult<()> {
        // ensure a module was specified, otherwise it defaults to builtins
        assert_eq!(T::MODULE.unwrap(), "databento_dbn");
        m.add_class::<T>()
    }
    // all functions exposed to Python need to be added here
    m.add_wrapped(wrap_pyfunction!(dbn::python::decode_metadata))?;
    m.add_wrapped(wrap_pyfunction!(dbn::python::encode_metadata))?;
    m.add_wrapped(wrap_pyfunction!(dbn::python::update_encoded_metadata))?;
    m.add_wrapped(wrap_pyfunction!(dbn::python::write_dbn_file))?;
    checked_add_class::<DbnDecoder>(m)?;
    checked_add_class::<dbn::Metadata>(m)?;
    checked_add_class::<RecordHeader>(m)?;
    checked_add_class::<MboMsg>(m)?;
    checked_add_class::<BidAskPair>(m)?;
    checked_add_class::<TradeMsg>(m)?;
    checked_add_class::<Mbp1Msg>(m)?;
    checked_add_class::<Mbp10Msg>(m)?;
    checked_add_class::<OhlcvMsg>(m)?;
    checked_add_class::<ImbalanceMsg>(m)?;
    checked_add_class::<InstrumentDefMsg>(m)?;
    checked_add_class::<ErrorMsg>(m)?;
    checked_add_class::<SymbolMappingMsg>(m)?;
    checked_add_class::<SystemMsg>(m)?;
    Ok(())
}

#[pyclass(module = "databento_dbn")]
struct DbnDecoder {
    buffer: io::Cursor<Vec<u8>>,
    has_decoded_metadata: bool,
}

#[pymethods]
impl DbnDecoder {
    #[new]
    fn new() -> Self {
        Self {
            buffer: io::Cursor::default(),
            has_decoded_metadata: false,
        }
    }

    fn write(&mut self, bytes: &[u8]) -> PyResult<()> {
        self.buffer.write_all(bytes).map_err(to_val_err)
    }

    fn buffer(&self) -> &[u8] {
        self.buffer.get_ref().as_slice()
    }

    fn decode(&mut self) -> PyResult<Vec<PyObject>> {
        let mut recs = Vec::new();
        let position = self.buffer.position();
        self.buffer.set_position(0);
        if !self.has_decoded_metadata {
            match MetadataDecoder::new(&mut self.buffer).decode() {
                Ok(metadata) => {
                    Python::with_gil(|py| recs.push(metadata.into_py(py)));
                    self.has_decoded_metadata = true;
                }
                Err(err) => {
                    self.buffer.set_position(position);
                    // haven't read enough data for metadata
                    return Err(to_val_err(err));
                }
            }
        }
        let mut decoder = RecordDecoder::new(&mut self.buffer);
        Python::with_gil(|py| -> PyResult<()> {
            while let Some(rec) = decoder
                .decode_ref()
                .map_err(|err| PyIOError::new_err(format!("{err:?}")))?
            {
                // Bug in clippy generates an error here. trivial_copy feature isn't enabled,
                // but clippy thinks these records are `Copy`
                #[allow(clippy::clone_on_copy)]
                match rec.header().rtype {
                    rtype::MBP_0 => recs.push(rec.get::<TradeMsg>().unwrap().clone().into_py(py)),
                    rtype::MBP_1 => recs.push(rec.get::<Mbp1Msg>().unwrap().clone().into_py(py)),
                    rtype::MBP_10 => recs.push(rec.get::<Mbp10Msg>().unwrap().clone().into_py(py)),
                    #[allow(deprecated)]
                    rtype::OHLCV_DEPRECATED
                    | rtype::OHLCV_1S
                    | rtype::OHLCV_1M
                    | rtype::OHLCV_1H
                    | rtype::OHLCV_1D => {
                        recs.push(rec.get::<OhlcvMsg>().unwrap().clone().into_py(py))
                    }
                    rtype::INSTRUMENT_DEF => {
                        recs.push(rec.get::<InstrumentDefMsg>().unwrap().clone().into_py(py))
                    }
                    rtype::ERROR => recs.push(rec.get::<ErrorMsg>().unwrap().clone().into_py(py)),
                    rtype::SYMBOL_MAPPING => {
                        recs.push(rec.get::<SymbolMappingMsg>().unwrap().clone().into_py(py))
                    }
                    rtype::MBO => recs.push(rec.get::<MboMsg>().unwrap().clone().into_py(py)),
                    rtype => {
                        return Err(to_val_err(format!("Invalid rtype {rtype} found in record")))
                    }
                };
            }
            Ok(())
        })?;
        let read_position = self.buffer.position() as usize;
        self.buffer.get_mut().drain(..read_position);
        Ok(recs)
    }
}

#[cfg(test)]
mod tests {
    use dbn::enums::SType;
    use pyo3::{py_run, types::PyString};

    use super::*;

    fn setup() {
        if unsafe { pyo3::ffi::Py_IsInitialized() } == 0 {
            // add to available modules
            pyo3::append_to_inittab!(databento_dbn);
        }
        // initialize interpreter
        pyo3::prepare_freethreaded_python();
    }

    #[test]
    fn test_dbn_decoder() {
        setup();
        Python::with_gil(|py| {
            let path = PyString::new(
                py,
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../tests/data/test_data.mbo.dbn"
                ),
            );
            py_run!(
                py,
                path,
                r#"from databento_dbn import DbnDecoder

decoder = DbnDecoder()
with open(path, 'rb') as fin:
    decoder.write(fin.read())
records = decoder.decode()
assert len(records) == 3"#
            )
        });
    }

    #[test]
    fn test_metadata_identity() {
        // initialize interpreter
        setup();
        let stype_in = SType::Native as u8;
        let stype_out = SType::ProductId as u8;
        Python::with_gil(|py| {
            pyo3::py_run!(
                  py,
                  stype_in stype_out,
                  r#"from databento_dbn import decode_metadata, encode_metadata

metadata_bytes = encode_metadata("GLBX.MDP3", "mbo", 1, "native", "product_id", [], [], [], [], 2, None)
metadata = decode_metadata(metadata_bytes)
assert metadata.dataset == "GLBX.MDP3"
assert metadata.schema == "mbo"
assert metadata.start == 1
assert metadata.end == 2
assert metadata.limit is None
assert metadata.stype_in == "native"
assert metadata.stype_out == "product_id"
assert metadata.ts_out is False"#
            );
        });
    }

    #[test]
    fn test_dbn_decoder_metadata_error() {
        setup();
        Python::with_gil(|py| {
            py.run(
                r#"from databento_dbn import DbnDecoder

decoder = DbnDecoder()
try:
    records = decoder.decode()
    # If this code is called, the test will fail
    assert 1 == 0
except Exception:
    pass
"#,
                None,
                None,
            )
        })
        .unwrap();
    }

    #[test]
    fn test_dbn_decoder_decoding_error() {
        setup();
        Python::with_gil(|py| {
            py.run(
                r#"from databento_dbn import DbnDecoder, encode_metadata

metadata_bytes = encode_metadata("GLBX.MDP3", "mbo", 1, "native", "product_id", [], [], [], [], 2, None)
decoder = DbnDecoder()
decoder.write(metadata_bytes)
decoder.write(bytes([0x04, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]))
try:
    records = decoder.decode()
    # If this code is called, the test will fail
    assert 1 == 0
except Exception as ex:
    assert "Invalid rtype" in str(ex)
"#,
                None,
                None,
            )
        }).unwrap();
    }
}
