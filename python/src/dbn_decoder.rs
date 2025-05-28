use pyo3::{prelude::*, IntoPyObjectExt};

use dbn::{
    decode::dbn::fsm::{DbnFsm, ProcessResult},
    python::to_py_err,
    rtype_dispatch, HasRType, VersionUpgradePolicy,
};

#[pyclass(module = "databento_dbn", name = "DBNDecoder")]
pub struct DbnDecoder {
    fsm: DbnFsm,
}

#[pymethods]
impl DbnDecoder {
    #[new]
    #[pyo3(signature = (
        has_metadata = true,
        ts_out = false,
        input_version = None,
        upgrade_policy = VersionUpgradePolicy::default()
    ))]
    fn new(
        has_metadata: bool,
        ts_out: bool,
        input_version: Option<u8>,
        upgrade_policy: VersionUpgradePolicy,
    ) -> PyResult<Self> {
        let fsm = DbnFsm::builder()
            .ts_out(ts_out)
            .input_dbn_version(input_version)
            .map_err(to_py_err)?
            .upgrade_policy(upgrade_policy)
            .skip_metadata(!has_metadata)
            .build()
            .map_err(to_py_err)?;
        Ok(Self { fsm })
    }

    fn write(&mut self, bytes: &[u8]) -> PyResult<()> {
        self.fsm.write_all(bytes);
        Ok(())
    }

    fn buffer(&self) -> &[u8] {
        self.fsm.data()
    }

    fn decode(&mut self) -> PyResult<Vec<PyObject>> {
        let ts_out = self.fsm.ts_out();
        let mut py_recs = Vec::new();
        loop {
            let mut rec_refs = Vec::new();
            match self.fsm.process_all(&mut rec_refs, None) {
                ProcessResult::ReadMore(_) => return Ok(py_recs),
                ProcessResult::Metadata(metadata) => {
                    py_recs.push(Python::with_gil(|py| metadata.into_py_any(py))?)
                }
                ProcessResult::Record(_) => {
                    // Bug in clippy generates an error here. trivial_copy feature isn't enabled,
                    // but clippy thinks these records are `Copy`
                    fn push_rec<'py, R>(rec: &R, py: Python<'py>, py_recs: &mut Vec<PyObject>)
                    where
                        R: Clone + HasRType + IntoPyObject<'py>,
                    {
                        py_recs.push(rec.clone().into_py_any(py).unwrap());
                    }

                    Python::with_gil(|py| -> PyResult<()> {
                        for rec in rec_refs {
                            // Safety: It's safe to cast to `WithTsOut` because we're passing in the `ts_out`
                            // from the metadata header.
                            rtype_dispatch!(rec, ts_out: ts_out, push_rec(py, &mut py_recs))
                                .map_err(PyErr::from)?;
                        }
                        Ok(())
                    })?;
                    return Ok(py_recs);
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
        record::{ErrorMsg, OhlcvMsg, RecordHeader},
        Dataset, MetadataBuilder,
    };
    use pyo3::{ffi::c_str, py_run, types::PyString};

    use super::*;
    use crate::tests::setup;

    #[test]
    fn test_partial_metadata_and_records() {
        setup();
        let mut target =
            DbnDecoder::new(true, false, None, VersionUpgradePolicy::default()).unwrap();
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
        assert!(target.decode().unwrap().is_empty());
        target.write(&encoder.get_ref()[metadata_split..]).unwrap();
        let metadata_pos = encoder.get_ref().len();
        assert!(matches!(target.decode(), Ok(recs) if recs.len() == 1));
        let rec = ErrorMsg::new(1680708278000000000, None, "Python", true);
        encoder.encode_record(&rec).unwrap();
        assert!(target.buffer().is_empty());
        let record_pos = encoder.get_ref().len();
        for i in metadata_pos..record_pos {
            target.write(&encoder.get_ref()[i..i + 1]).unwrap();
            assert_eq!(target.buffer().len(), i + 1 - metadata_pos);
            // wrote last byte
            if i == record_pos - 1 {
                let res = target.decode();
                assert_eq!(record_pos - metadata_pos, std::mem::size_of_val(&rec));
                assert!(matches!(res, Ok(recs) if recs.len() == 1));
            } else {
                let res = target.decode();
                assert!(matches!(res, Ok(recs) if recs.is_empty()));
            }
        }
    }

    #[test]
    fn test_full_with_partial_record() {
        setup();
        let mut decoder =
            DbnDecoder::new(true, false, None, VersionUpgradePolicy::default()).unwrap();
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
        let res = decoder.decode();
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
                    "/../tests/data/test_data.mbo.v3.dbn"
                ),
            );
            py_run!(
                py,
                path,
                r#"from _lib import DBNDecoder

decoder = DBNDecoder()
with open(path, 'rb') as fin:
    decoder.write(fin.read())
records = decoder.decode()
assert len(records) == 3
metadata = records[0]
for record in records[1:]:
    assert hasattr(record, "ts_out") == metadata.ts_out"#
            )
        });
    }

    #[test]
    fn test_dbn_decoder_decoding_error() {
        setup();
        Python::with_gil(|py| {
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
                None,
                None,
            )
        }).unwrap();
    }

    #[test]
    fn test_dbn_decoder_no_metadata() {
        setup();
        Python::with_gil(|py| {
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
                None,
                None,
            )
        })
        .unwrap();
    }
}
