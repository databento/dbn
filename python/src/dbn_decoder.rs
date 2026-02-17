use std::io::Write;

use pyo3::{exceptions::PyRuntimeError, prelude::*, IntoPyObjectExt};

use dbn::{
    decode::dbn::fsm::{DbnFsm, ProcessResult},
    python::to_py_err,
    rtype_dispatch, Compression, HasRType, VersionUpgradePolicy,
};

#[pyclass(module = "databento_dbn", name = "DBNDecoder")]
pub struct DbnDecoder {
    fsm: DbnFsm,
    zstd_decoder: Option<zstd::stream::write::Decoder<'static, Vec<u8>>>,
}

#[pymethods]
impl DbnDecoder {
    #[new]
    #[pyo3(signature = (
        has_metadata = true,
        ts_out = false,
        input_version = None,
        upgrade_policy = VersionUpgradePolicy::default(),
        compression = Compression::None,
    ))]
    fn new(
        has_metadata: bool,
        ts_out: bool,
        input_version: Option<u8>,
        upgrade_policy: VersionUpgradePolicy,
        compression: Compression,
    ) -> PyResult<Self> {
        let fsm = DbnFsm::builder()
            .ts_out(ts_out)
            .input_dbn_version(input_version)
            .map_err(to_py_err)?
            .upgrade_policy(upgrade_policy)
            .skip_metadata(!has_metadata)
            .compat_size(if upgrade_policy == VersionUpgradePolicy::AsIs {
                0
            } else {
                DbnFsm::DEFAULT_BUF_SIZE
            })
            .build()
            .map_err(to_py_err)?;
        let zstd_decoder = match compression {
            Compression::None => None,
            Compression::Zstd => Some(
                zstd::stream::write::Decoder::new(Vec::new())
                    .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?,
            ),
        };
        Ok(Self { fsm, zstd_decoder })
    }

    fn write(&mut self, bytes: &[u8]) -> PyResult<()> {
        if let Some(zstd_decoder) = &mut self.zstd_decoder {
            zstd_decoder
                .write_all(bytes)
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
        } else {
            self.fsm.write_all(bytes)
        }

        Ok(())
    }

    fn buffer(&self) -> &[u8] {
        self.fsm.data()
    }

    fn decode(&mut self, py: Python<'_>) -> PyResult<Vec<Py<PyAny>>> {
        // Flush all decompressed data to FSM
        if let Some(zstd_decoder) = &mut self.zstd_decoder {
            zstd_decoder
                .flush()
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
            let decompressed = zstd_decoder.get_mut();
            if !decompressed.is_empty() {
                self.fsm.write_all(decompressed);
                decompressed.clear();
            }
        }

        let mut ts_out = self.fsm.ts_out();
        let mut py_recs = Vec::new();
        loop {
            match self.fsm.process() {
                ProcessResult::ReadMore(_) => return Ok(py_recs),
                ProcessResult::Metadata(metadata) => {
                    ts_out = self.fsm.ts_out();
                    py_recs.push(metadata.into_py_any(py)?)
                }
                ProcessResult::Record(_) => {
                    // Bug in clippy generates an error here. trivial_copy feature isn't enabled,
                    // but clippy thinks these records are `Copy`
                    fn push_rec<'py, R>(rec: &R, py: Python<'py>, py_recs: &mut Vec<Py<PyAny>>)
                    where
                        R: Clone + HasRType + IntoPyObject<'py>,
                    {
                        py_recs.push(rec.clone().into_py_any(py).unwrap());
                    }

                    let rec = self.fsm.last_record().ok_or_else(|| {
                        PyRuntimeError::new_err("Error while decoding DBN stream")
                    })?;

                    // Safety: It's safe to cast to `WithTsOut` because we're passing in the `ts_out`
                    // from the metadata header.
                    rtype_dispatch!(rec, ts_out: ts_out, push_rec(py, &mut py_recs))
                        .map_err(PyErr::from)?;
                }
                ProcessResult::Err(error) => return Err(PyErr::from(error)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use dbn::{
        encode::{dbn::Encoder, EncodeRecord},
        enums::{rtype, SType, Schema},
        record::{ErrorMsg, OhlcvMsg, RecordHeader, SystemMsg},
        Dataset, MetadataBuilder,
    };
    use pyo3::{ffi::c_str, types::PyDict, types::PyString};
    use rstest::*;

    use super::*;
    use crate::tests::python;

    #[rstest]
    fn test_partial_metadata_and_records(_python: ()) {
        Python::attach(|py| {
            let mut target = DbnDecoder::new(
                true,
                false,
                None,
                VersionUpgradePolicy::default(),
                Compression::None,
            )
            .unwrap();
            let buffer = Vec::new();
            let mut encoder = Encoder::new(
                buffer,
                &MetadataBuilder::new()
                    .dataset(Dataset::XnasItch.to_string())
                    .schema(Some(Schema::Trades))
                    .stype_in(Some(SType::RawSymbol))
                    .stype_out(SType::InstrumentId)
                    .start(0)
                    .build(),
            )
            .unwrap();
            let metadata_split = encoder.get_ref().len() / 2;
            target.write(&encoder.get_ref()[..metadata_split]).unwrap();
            assert!(target.decode(py).unwrap().is_empty());
            target.write(&encoder.get_ref()[metadata_split..]).unwrap();
            let metadata_pos = encoder.get_ref().len();
            assert!(matches!(target.decode(py), Ok(recs) if recs.len() == 1));
            let rec = ErrorMsg::new(1680708278000000000, None, "Python", true);
            encoder.encode_record(&rec).unwrap();
            assert!(target.buffer().is_empty());
            let record_pos = encoder.get_ref().len();
            for i in metadata_pos..record_pos {
                target.write(&encoder.get_ref()[i..i + 1]).unwrap();
                assert_eq!(target.buffer().len(), i + 1 - metadata_pos);
                // wrote last byte
                if i == record_pos - 1 {
                    let res = target.decode(py);
                    assert_eq!(record_pos - metadata_pos, std::mem::size_of_val(&rec));
                    assert!(matches!(res, Ok(recs) if recs.len() == 1));
                } else {
                    let res = target.decode(py);
                    assert!(matches!(res, Ok(recs) if recs.is_empty()));
                }
            }
        });
    }

    #[rstest]
    fn test_full_with_partial_record(_python: ()) {
        let mut decoder = DbnDecoder::new(
            true,
            false,
            None,
            VersionUpgradePolicy::default(),
            Compression::None,
        )
        .unwrap();
        let buffer = Vec::new();
        let mut encoder = Encoder::new(
            buffer,
            &MetadataBuilder::new()
                .dataset(Dataset::XnasItch.to_string())
                .schema(Some(Schema::Ohlcv1S))
                .stype_in(Some(SType::RawSymbol))
                .stype_out(SType::InstrumentId)
                .start(0)
                .build(),
        )
        .unwrap();
        decoder.write(encoder.get_ref().as_slice()).unwrap();
        let metadata_pos = encoder.get_ref().len();
        let res = Python::attach(|py| decoder.decode(py));
        dbg!(&res);
        assert!(matches!(res, Ok(recs) if recs.len() == 1));
        // assert!(decoder.has_decoded_metadata);
        let rec1 = ErrorMsg::new(1680708278000000000, None, "Python", true);
        let rec2 = OhlcvMsg {
            hd: RecordHeader::new::<OhlcvMsg>(rtype::OHLCV_1S, 1, 1, 1681228173000000000),
            open: 100,
            high: 200,
            low: 50,
            close: 150,
            volume: 1000,
        };
        encoder.encode_record(&rec1).unwrap();
        let rec1_pos = encoder.get_ref().len();
        encoder.encode_record(&rec2).unwrap();
        assert!(decoder.buffer().is_empty());
        // Write first record and part of second
        decoder
            .write(&encoder.get_ref()[metadata_pos..rec1_pos + 4])
            .unwrap();
        // Read first record
        let res1 = Python::attach(|py| decoder.decode(py));
        assert!(matches!(res1, Ok(recs) if recs.len() == 1));
        // Write rest of second record
        decoder.write(&encoder.get_ref()[rec1_pos + 4..]).unwrap();
        let res2 = Python::attach(|py| decoder.decode(py));
        assert!(matches!(res2, Ok(recs) if recs.len() == 1));
    }

    #[rstest]
    fn test_dbn_decoder(_python: ()) {
        Python::attach(|py| {
            let path = PyString::new(
                py,
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../tests/data/test_data.mbo.v3.dbn"
                ),
            );
            let globals = PyDict::new(py);
            globals.set_item("path", path).unwrap();
            Python::run(
                py,
                c_str!(
                    r#"from _lib import DBNDecoder

decoder = DBNDecoder()
with open(path, 'rb') as fin:
    decoder.write(fin.read())
records = decoder.decode()
assert len(records) == 3
metadata = records[0]
for record in records[1:]:
    assert hasattr(record, "ts_out") == metadata.ts_out"#
                ),
                Some(&globals),
                None,
            )
            .unwrap();
        });
    }

    #[rstest]
    fn test_dbn_decoder_decoding_error(_python: ()) {
        Python::attach(|py| {
            Python::run(py,
                c_str!(r#"from _lib import DBNDecoder, DBNError, Metadata, Schema, SType

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
metadata_bytes = bytes(metadata)
decoder = DBNDecoder()
decoder.write(metadata_bytes)
decoder.write(bytes([0x04, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]))
try:
    records = decoder.decode()
    # If this code is called, the test will fail
    assert False
except DBNError as ex:
    assert "couldn't convert" in str(ex)
    assert "RType" in str(ex)
except Exception:
    assert False
"#),
                // Create an empty `globals` dict to keep tests hermetic
                Some(&PyDict::new(py)),
                None,
            )
        }).unwrap();
    }

    #[rstest]
    fn test_dbn_decoder_ts_out(_python: ()) {
        Python::attach(|py| {
            Python::run(
                py,
                c_str!(
                    r#"from _lib import DBNDecoder, DBNError, Metadata, Schema, SType

metadata = Metadata(
    version=3,
    dataset="IFUS.IMPACT",
    schema=Schema.MBO,
    start=1,
    stype_in=SType.RAW_SYMBOL,
    stype_out=SType.INSTRUMENT_ID,
    end=2,
    ts_out=True,
    symbols=[],
    partial=[],
    not_found=[],
    mappings=[]
)
metadata_bytes = bytes(metadata)
decoder = DBNDecoder()
decoder.write(metadata_bytes)
records = decoder.decode()
for record in records:
    assert record.ts_out is not None
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
    fn test_dbn_decoder_no_metadata(_python: ()) {
        Python::attach(|py| {
            Python::run(
                py,
                c_str!(
                    r#"from _lib import DBNDecoder, OHLCVMsg

decoder = DBNDecoder(has_metadata=False)
record = OHLCVMsg(0x20, 1, 10, 0, 0, 0, 0, 0, 0)
decoder.write(bytes(record))
records = decoder.decode()
assert len(records) == 1
assert records[0] == record
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
    fn test_decode_all_data_in_compat_situation(_python: ()) {
        Python::attach(|py| {
            Python::run(
                py,
                c_str!(
                    r#"from _lib import DBNDecoder, ErrorMsg, ErrorMsgV1, Metadata, Schema, SType

decoder = DBNDecoder()
metadata = Metadata(
    version=1,
    dataset="GLBX.MDP3",
    schema=Schema.MBO,
    start=0,
    stype_in=SType.RAW_SYMBOL,
    stype_out=SType.INSTRUMENT_ID,
)
decoder.write(bytes(metadata))
n = 100_000
for _ in range(n):
    error = ErrorMsgV1(0, "test")
    decoder.write(bytes(error))
records = decoder.decode()
assert len(records) == 1 + n, f"{len(records)=} {1+n=}"
assert isinstance(records[0], Metadata), f"{records[0].__class__.__name__}"
for r in records[1:]:
    assert isinstance(r, ErrorMsg), f"{r}"
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
    fn test_dbn_decoder_with_zstd_compression(_python: ()) {
        Python::attach(|py| {
            let path = PyString::new(
                py,
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../tests/data/test_data.mbo.v3.dbn.zst"
                ),
            );
            let globals = PyDict::new(py);
            globals.set_item("path", path).unwrap();
            Python::run(
                py,
                c_str!(
                    r#"from _lib import DBNDecoder, Compression

decoder = DBNDecoder(compression=Compression.ZSTD)
with open(path, 'rb') as f:
    decoder.write(f.read())
records = decoder.decode()
assert len(records) == 3  # metadata + 2 records
"#
                ),
                Some(&globals),
                None,
            )
            .unwrap();
        });
    }

    #[rstest]
    fn test_dbn_decoder_with_two_zstd_frames(_python: ()) {
        Python::attach(|py| {
            let buffer = Vec::new();
            let mut encoder = Encoder::new(
                buffer,
                &MetadataBuilder::new()
                    .dataset(Dataset::XnasItch.to_string())
                    .schema(Some(Schema::Trades))
                    .stype_in(Some(SType::RawSymbol))
                    .stype_out(SType::InstrumentId)
                    .start(0)
                    .build(),
            )
            .unwrap();
            let rec1 = SystemMsg::new(1680708278000000000, None, "first").unwrap();
            let rec2 = SystemMsg::new(1680708279000000000, None, "second").unwrap();
            encoder.encode_record(&rec1).unwrap();
            let split = encoder.get_ref().len();
            encoder.encode_record(&rec2).unwrap();
            let dbn_bytes = encoder.get_ref();

            // Compress as two separate zstd frames
            let frame1 = zstd::encode_all(std::io::Cursor::new(&dbn_bytes[..split]), 0).unwrap();
            let frame2 = zstd::encode_all(std::io::Cursor::new(&dbn_bytes[split..]), 0).unwrap();

            let mut target = DbnDecoder::new(
                true,
                false,
                None,
                VersionUpgradePolicy::default(),
                Compression::Zstd,
            )
            .unwrap();
            target.write(&frame1).unwrap();
            target.write(&frame2).unwrap();
            let records = target.decode(py).unwrap();
            // metadata + 2 records
            assert_eq!(records.len(), 3);
        });
    }
}
