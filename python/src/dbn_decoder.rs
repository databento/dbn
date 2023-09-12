use std::io::{self, Write};

use pyo3::prelude::*;

use dbn::{
    decode::dbn::{MetadataDecoder, RecordDecoder},
    python::to_val_err,
    record::HasRType,
    rtype_ts_out_dispatch,
};

#[pyclass(module = "databento_dbn", name = "DBNDecoder")]
pub struct DbnDecoder {
    buffer: io::Cursor<Vec<u8>>,
    has_decoded_metadata: bool,
    ts_out: bool,
}

#[pymethods]
impl DbnDecoder {
    #[new]
    fn new(has_metadata: Option<bool>, ts_out: Option<bool>) -> Self {
        Self {
            buffer: io::Cursor::default(),
            has_decoded_metadata: !has_metadata.unwrap_or(true),
            ts_out: ts_out.unwrap_or_default(),
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
                    Python::with_gil(|py| recs.push(metadata.into_py(py)));
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
                    recs.push(rec.clone().into_py(py))
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
                // keep track of position after last _successful_ decoding to
                // ensure buffer is left in correct state in the case where one
                // or more successful decodings is followed by a partial one, i.e.
                // `decode_record_ref` returning `Ok(None)`
                read_position = decoder.get_ref().position() as usize;
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
    use dbn::{
        datasets::XNAS_ITCH,
        encode::{dbn::Encoder, EncodeRecord},
        enums::{rtype, SType, Schema},
        record::{ErrorMsg, OhlcvMsg, RecordHeader},
        MetadataBuilder,
    };
    use pyo3::{py_run, types::PyString};

    use super::*;
    use crate::tests::setup;

    #[test]
    fn test_partial_records() {
        setup();
        let mut decoder = DbnDecoder::new(None, None);
        let buffer = Vec::new();
        let mut encoder = Encoder::new(
            buffer,
            &MetadataBuilder::new()
                .dataset(XNAS_ITCH.to_owned())
                .schema(Some(Schema::Trades))
                .stype_in(Some(SType::RawSymbol))
                .stype_out(SType::InstrumentId)
                .start(0)
                .build(),
        )
        .unwrap();
        decoder.write(encoder.get_ref().as_slice()).unwrap();
        let metadata_pos = encoder.get_ref().len();
        assert!(matches!(decoder.decode(), Ok(recs) if recs.len() == 1));
        assert!(decoder.has_decoded_metadata);
        let rec = ErrorMsg::new(1680708278000000000, "Python");
        encoder.encode_record(&rec).unwrap();
        assert!(decoder.buffer.get_ref().is_empty());
        let record_pos = encoder.get_ref().len();
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
        let mut decoder = DbnDecoder::new(None, None);
        let buffer = Vec::new();
        let mut encoder = Encoder::new(
            buffer,
            &MetadataBuilder::new()
                .dataset(XNAS_ITCH.to_owned())
                .schema(Some(Schema::Ohlcv1S))
                .stype_in(Some(SType::RawSymbol))
                .stype_out(SType::InstrumentId)
                .start(0)
                .build(),
        )
        .unwrap();
        decoder.write(encoder.get_ref().as_slice()).unwrap();
        let metadata_pos = encoder.get_ref().len();
        assert!(matches!(decoder.decode(), Ok(recs) if recs.len() == 1));
        assert!(decoder.has_decoded_metadata);
        let rec1 = ErrorMsg::new(1680708278000000000, "Python");
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
                r#"from databento_dbn import DBNDecoder

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
            py.run(
                r#"from databento_dbn import DBNDecoder, Metadata, Schema, SType

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
except Exception as ex:
    assert "Invalid rtype" in str(ex)
"#,
                None,
                None,
            )
        }).unwrap();
    }

    #[test]
    fn test_dbn_decoder_no_metadata() {
        setup();
        Python::with_gil(|py| {
            py.run(
                r#"from databento_dbn import DBNDecoder, OHLCVMsg

decoder = DBNDecoder(has_metadata=False)
record = OHLCVMsg(0x20, 1, 10, 0, 0, 0, 0, 0, 0)
decoder.write(bytes(record))
records = decoder.decode()
assert len(records) == 1
assert records[0] == record
"#,
                None,
                None,
            )
        })
        .unwrap();
    }
}
