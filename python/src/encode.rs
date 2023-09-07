use std::{io, num::NonZeroU64};

use dbn::{
    encode::{
        dbn::{Encoder as DbnEncoder, MetadataEncoder},
        DbnEncodable, DynWriter, EncodeDbn,
    },
    enums::{Compression, Schema},
    python::to_val_err,
    record::{
        ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, StatMsg, TbboMsg,
        TradeMsg,
    },
    Metadata,
};
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    intern,
    prelude::*,
    types::PyBytes,
    PyClass,
};

/// Updates existing fields that have already been written to the given file.
#[pyfunction]
pub fn update_encoded_metadata(
    _py: Python<'_>,
    file: PyFileLike,
    start: u64,
    end: Option<u64>,
    limit: Option<u64>,
) -> PyResult<()> {
    MetadataEncoder::new(file)
        .update_encoded(
            start,
            end.and_then(NonZeroU64::new),
            limit.and_then(NonZeroU64::new),
        )
        .map_err(to_val_err)
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
    metadata: &Metadata,
    records: Vec<&PyAny>,
) -> PyResult<()> {
    let writer = DynWriter::new(file, compression).map_err(to_val_err)?;
    let encoder = DbnEncoder::new(writer, metadata).map_err(to_val_err)?;
    match metadata.schema {
        Some(Schema::Mbo) => encode_pyrecs::<MboMsg>(encoder, &records),
        Some(Schema::Mbp1) => encode_pyrecs::<Mbp1Msg>(encoder, &records),
        Some(Schema::Mbp10) => encode_pyrecs::<Mbp10Msg>(encoder, &records),
        Some(Schema::Tbbo) => encode_pyrecs::<TbboMsg>(encoder, &records),
        Some(Schema::Trades) => encode_pyrecs::<TradeMsg>(encoder, &records),
        Some(Schema::Ohlcv1S)
        | Some(Schema::Ohlcv1M)
        | Some(Schema::Ohlcv1H)
        | Some(Schema::Ohlcv1D)
        | Some(Schema::OhlcvEod) => encode_pyrecs::<OhlcvMsg>(encoder, &records),
        Some(Schema::Definition) => encode_pyrecs::<InstrumentDefMsg>(encoder, &records),
        Some(Schema::Imbalance) => encode_pyrecs::<ImbalanceMsg>(encoder, &records),
        Some(Schema::Statistics) => encode_pyrecs::<StatMsg>(encoder, &records),
        Some(Schema::Status) | None => Err(PyValueError::new_err(
            "Unsupported schema type for writing DBN files",
        )),
    }
}

fn encode_pyrecs<T: Clone + DbnEncodable + PyClass>(
    mut encoder: DbnEncoder<DynWriter<PyFileLike>>,
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

/// A Python object that implements the Python file interface.
pub struct PyFileLike {
    inner: PyObject,
}

impl<'source> FromPyObject<'source> for PyFileLike {
    fn extract(any: &'source PyAny) -> PyResult<Self> {
        Python::with_gil(|py| {
            let obj: PyObject = any.extract()?;
            if obj.getattr(py, intern!(py, "read")).is_err() {
                return Err(PyTypeError::new_err(
                    "object is missing a `read()` method".to_owned(),
                ));
            }
            if obj.getattr(py, intern!(py, "write")).is_err() {
                return Err(PyTypeError::new_err(
                    "object is missing a `write()` method".to_owned(),
                ));
            }
            if obj.getattr(py, intern!(py, "seek")).is_err() {
                return Err(PyTypeError::new_err(
                    "object is missing a `seek()` method".to_owned(),
                ));
            }
            Ok(PyFileLike { inner: obj })
        })
    }
}

impl io::Write for PyFileLike {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        Python::with_gil(|py| {
            let bytes = PyBytes::new(py, buf).to_object(py);
            let number_bytes_written = self
                .inner
                .call_method(py, intern!(py, "write"), (bytes,), None)
                .map_err(py_to_rs_io_err)?;

            number_bytes_written.extract(py).map_err(py_to_rs_io_err)
        })
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        Python::with_gil(|py| {
            self.inner
                .call_method(py, intern!(py, "flush"), (), None)
                .map_err(py_to_rs_io_err)?;

            Ok(())
        })
    }
}

impl io::Seek for PyFileLike {
    fn seek(&mut self, pos: io::SeekFrom) -> Result<u64, io::Error> {
        Python::with_gil(|py| {
            let (whence, offset) = match pos {
                io::SeekFrom::Start(i) => (0, i as i64),
                io::SeekFrom::Current(i) => (1, i),
                io::SeekFrom::End(i) => (2, i),
            };

            let new_position = self
                .inner
                .call_method(py, intern!(py, "seek"), (offset, whence), None)
                .map_err(py_to_rs_io_err)?;

            new_position.extract(py).map_err(py_to_rs_io_err)
        })
    }
}

fn py_to_rs_io_err(e: PyErr) -> io::Error {
    Python::with_gil(|py| {
        let e_as_object: PyObject = e.into_py(py);

        match e_as_object.call_method(py, intern!(py, "__str__"), (), None) {
            Ok(repr) => match repr.extract::<String>(py) {
                Ok(s) => io::Error::new(io::ErrorKind::Other, s),
                Err(_e) => io::Error::new(io::ErrorKind::Other, "An unknown error has occurred"),
            },
            Err(_) => io::Error::new(io::ErrorKind::Other, "Err doesn't have __str__"),
        }
    })
}

#[cfg(test)]
pub mod tests {

    use std::io::{Cursor, Seek, Write};
    use std::sync::{Arc, Mutex};

    use dbn::datasets::GLBX_MDP3;
    use dbn::{
        decode::{dbn::Decoder as DbnDecoder, DecodeDbn},
        enums::SType,
        metadata::MetadataBuilder,
        record::TbboMsg,
    };

    use super::*;

    const DBN_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/data");

    #[pyclass]
    pub struct MockPyFile {
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
                    0 => io::SeekFrom::Start(offset as u64),
                    1 => io::SeekFrom::Current(offset),
                    2 => io::SeekFrom::End(offset),
                    _ => unimplemented!("whence value"),
                })
                .unwrap()
        }
    }

    impl MockPyFile {
        pub fn new() -> Self {
            Self {
                buf: Arc::new(Mutex::new(Cursor::new(Vec::new()))),
            }
        }

        pub fn inner(&self) -> Arc<Mutex<Cursor<Vec<u8>>>> {
            self.buf.clone()
        }
    }

    const DATASET: &str = GLBX_MDP3;
    const STYPE: SType = SType::InstrumentId;

    macro_rules! test_writing_dbn_from_python {
        ($test_name:ident, $record_type:ident, $schema:expr) => {
            #[test]
            fn $test_name() {
                // Required one-time setup
                pyo3::prepare_freethreaded_python();

                // Read in test data
                let decoder = DbnDecoder::from_zstd_file(format!(
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
                    let metadata = MetadataBuilder::new()
                        .dataset(DATASET.to_owned())
                        .schema(Some($schema))
                        .start(0)
                        .stype_in(Some(STYPE))
                        .stype_out(STYPE)
                        .build();
                    // Call target function
                    write_dbn_file(
                        py,
                        mock_file.extract(py).unwrap(),
                        Compression::ZStd,
                        &metadata,
                        recs.iter().map(|r| r.as_ref(py)).collect(),
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
                let py_decoder = DbnDecoder::with_zstd(Cursor::new(&output_buf)).unwrap();
                let metadata = py_decoder.metadata().clone();
                assert_eq!(metadata.schema, Some($schema));
                assert_eq!(metadata.dataset, DATASET);
                assert_eq!(metadata.stype_in, Some(STYPE));
                assert_eq!(metadata.stype_out, STYPE);
                let decoder = DbnDecoder::from_zstd_file(format!(
                    "{DBN_PATH}/test_data.{}.dbn.zst",
                    $schema.as_str()
                ))
                .unwrap();

                let py_recs = py_decoder.decode_records::<$record_type>().unwrap();
                let exp_recs = decoder.decode_records::<$record_type>().unwrap();
                assert_eq!(py_recs.len(), exp_recs.len());
                for (py_rec, exp_rec) in py_recs.iter().zip(exp_recs.iter()) {
                    assert_eq!(py_rec, exp_rec);
                }
                assert_eq!(
                    py_recs.len(),
                    if $schema == Schema::Ohlcv1D { 0 } else { 2 }
                );
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
