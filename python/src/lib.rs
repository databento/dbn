//! Python bindings for the [`dbn`] crate using [`pyo3`].
use std::io::{self, Write};

use pyo3::{prelude::*, types::PyTuple, wrap_pyfunction, PyClass};

use dbn::{
    decode::dbn::{MetadataDecoder, RecordDecoder},
    python::to_val_err,
    record::{
        BidAskPair, ErrorMsg, HasRType, ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg,
        OhlcvMsg, RecordHeader, StatusMsg, SymbolMappingMsg, SystemMsg, TradeMsg,
    },
    rtype_ts_out_dispatch,
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
    checked_add_class::<StatusMsg>(m)?;
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
    ts_out: bool,
}

#[pymethods]
impl DbnDecoder {
    #[new]
    fn new() -> Self {
        Self {
            buffer: io::Cursor::default(),
            has_decoded_metadata: false,
            ts_out: false,
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
        let orig_position = self.buffer.position();
        self.buffer.set_position(0);
        if !self.has_decoded_metadata {
            match MetadataDecoder::new(&mut self.buffer).decode() {
                Ok(metadata) => {
                    self.ts_out = metadata.ts_out;
                    Python::with_gil(|py| recs.push((metadata, py.None()).into_py(py)));
                    self.has_decoded_metadata = true;
                }
                Err(err) => {
                    self.buffer.set_position(orig_position);
                    // haven't read enough data for metadata
                    return Err(to_val_err(err));
                }
            }
        }
        let mut read_position = self.buffer.position() as usize;
        let mut decoder = RecordDecoder::new(&mut self.buffer);
        Python::with_gil(|py| -> PyResult<()> {
            while let Some(rec) = decoder.decode_ref().map_err(to_val_err)? {
                // Bug in clippy generates an error here. trivial_copy feature isn't enabled,
                // but clippy thinks these records are `Copy`
                fn push_rec<R: Clone + HasRType + IntoPy<Py<PyAny>>>(
                    rec: &R,
                    py: Python,
                    recs: &mut Vec<Py<PyAny>>,
                ) {
                    let pyrec = rec.clone().into_py(py);
                    recs.push(
                        // Convert non `WithTsOut` records to a (rec, None)
                        // for consistent typing
                        if pyrec
                            .as_ref(py)
                            .is_instance_of::<PyTuple>()
                            .unwrap_or_default()
                        {
                            pyrec
                        } else {
                            (pyrec, py.None()).into_py(py)
                        },
                    )
                }

                // Safety: It's safe to cast to `WithTsOut` because we're passing in the `ts_out`
                // from the metadata header.
                if unsafe { rtype_ts_out_dispatch!(rec, self.ts_out, push_rec, py, &mut recs) }
                    .is_err()
                {
                    return Err(to_val_err(format!(
                        "Invalid rtype {} found in record",
                        rec.header().rtype,
                    )));
                }
                // keep track of position after last _successful_ decoding to ensure
                // buffer is left in correct state in the case where one or more
                // successful decodings is followed by a partial one, i.e. `decode_ref`
                // returning `Ok(None)`
                read_position = decoder.get_mut().position() as usize;
            }
            Ok(())
        })
        .map_err(|e| {
            self.buffer.set_position(orig_position);
            e
        })?;
        if recs.is_empty() {
            self.buffer.set_position(orig_position);
        } else {
            self.shift_buffer(read_position);
        }
        Ok(recs)
    }
}

impl DbnDecoder {
    fn shift_buffer(&mut self, read_position: usize) {
        let inner_buf = self.buffer.get_mut();
        let length = inner_buf.len();
        let new_length = length - read_position;
        inner_buf.drain(..read_position);
        debug_assert_eq!(inner_buf.len(), new_length);
        self.buffer.set_position(new_length as u64);
    }
}

#[cfg(test)]
mod tests {
    use dbn::{encode::EncodeDbn, enums::rtype::OHLCV_1S};
    use pyo3::{py_run, types::PyString};

    use ::dbn::{
        encode::dbn::Encoder,
        enums::{SType, Schema},
        MetadataBuilder,
    };

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
    fn test_partial_records() {
        setup();
        let mut decoder = DbnDecoder::new();
        let buffer = Vec::new();
        let mut encoder = Encoder::new(
            buffer,
            &MetadataBuilder::new()
                .dataset("XNAS.ITCH".to_owned())
                .schema(Schema::Trades)
                .stype_in(SType::Native)
                .stype_out(SType::ProductId)
                .start(0)
                .build(),
        )
        .unwrap();
        decoder.write(encoder.get_ref().as_slice()).unwrap();
        let metadata_pos = encoder.get_ref().len() as usize;
        assert!(matches!(decoder.decode(), Ok(recs) if recs.len() == 1));
        assert!(decoder.has_decoded_metadata);
        let rec = ErrorMsg::new(1680708278000000000, "Python");
        encoder.encode_record(&rec).unwrap();
        assert!(decoder.buffer.get_ref().is_empty());
        let record_pos = encoder.get_ref().len() as usize;
        for i in metadata_pos..record_pos {
            decoder.write(&encoder.get_ref()[i..i + 1]).unwrap();
            assert_eq!(decoder.buffer.get_ref().len(), i + 1 - metadata_pos);
            // wrote last byte
            if i == record_pos - 1 {
                let res = decoder.decode();
                assert_eq!(record_pos - metadata_pos, std::mem::size_of_val(&rec));
                assert!(matches!(res, Ok(recs) if recs.len() == 1));
            } else {
                let res = decoder.decode();
                assert!(matches!(res, Ok(recs) if recs.is_empty()));
            }
        }
    }

    #[test]
    fn test_full_with_partial_record() {
        setup();
        let mut decoder = DbnDecoder::new();
        let buffer = Vec::new();
        let mut encoder = Encoder::new(
            buffer,
            &MetadataBuilder::new()
                .dataset("XNAS.ITCH".to_owned())
                .schema(Schema::Ohlcv1S)
                .stype_in(SType::Native)
                .stype_out(SType::ProductId)
                .start(0)
                .build(),
        )
        .unwrap();
        decoder.write(encoder.get_ref().as_slice()).unwrap();
        let metadata_pos = encoder.get_ref().len() as usize;
        assert!(matches!(decoder.decode(), Ok(recs) if recs.len() == 1));
        assert!(decoder.has_decoded_metadata);
        let rec1 = ErrorMsg::new(1680708278000000000, "Python");
        let rec2 = OhlcvMsg {
            hd: RecordHeader::new::<OhlcvMsg>(OHLCV_1S, 1, 1, 1681228173000000000),
            open: 100,
            high: 200,
            low: 50,
            close: 150,
            volume: 1000,
        };
        encoder.encode_record(&rec1).unwrap();
        let rec1_pos = encoder.get_ref().len() as usize;
        encoder.encode_record(&rec2).unwrap();
        assert!(decoder.buffer.get_ref().is_empty());
        // Write first record and part of second
        decoder
            .write(&encoder.get_ref()[metadata_pos..rec1_pos + 4])
            .unwrap();
        // Read first record
        let res1 = decoder.decode();
        assert!(matches!(res1, Ok(recs) if recs.len() == 1));
        // Write rest of second record
        decoder.write(&encoder.get_ref()[rec1_pos + 4..]).unwrap();
        let res2 = decoder.decode();
        assert!(matches!(res2, Ok(recs) if recs.len() == 1));
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
assert len(records) == 3
metadata, _ = records[0]
for _, ts_out in records[1:]:
    if metadata.ts_out:
        assert ts_out is not None
    else:
        assert ts_out is None"#
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
    assert False
except Exception as ex:
    assert "Invalid rtype" in str(ex)
"#,
                None,
                None,
            )
        }).unwrap();
    }
}
