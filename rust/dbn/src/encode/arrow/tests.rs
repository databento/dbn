//! Tests shared by the Arrow IPC and Parquet encoders.

use std::collections::HashMap;
use std::io::{self, Write};
use std::num::NonZeroUsize;
use std::os::raw::c_char;

use ::arrow::array::{
    Array, Float64Array, Int64Array, RecordBatch, StringArray, TimestampNanosecondArray,
    UInt32Array,
};
use ::arrow::datatypes::SchemaRef;
use ::arrow::ipc::reader::FileReader;
use rstest::rstest;
use time::macros::date;

use super::sync::EncoderBuilder;
use super::{ColumnarWriter, IpcFile, METADATA_KEY};
use crate::decode::dbn::Decoder as DbnDecoder;
use crate::encode::{
    DbnEncoder, EncodeDbn, EncodeRecord, EncodeRecordRef, EncodeRecordTextExt, JsonEncoder,
};
use crate::enums::rtype;
use crate::metadata::{MappingInterval, SymbolMapping};
use crate::record::{MboMsg, RecordHeader, TradeMsg, WithTsOut};
use crate::{
    Compression, Metadata, RecordRef, SType, Schema, FIXED_PRICE_SCALE, UNDEF_PRICE,
    UNDEF_TIMESTAMP,
};

/// One of the two columnar formats, with the readers the tests need.
trait Format {
    type File<W: Write + Send>: ColumnarWriter<Inner = W>;

    /// Reads the schema and all rows (zero if the file holds none).
    fn read(buf: &[u8]) -> (SchemaRef, RecordBatch);

    /// Reads the DBN metadata as a reader of this format would see it.
    fn metadata(buf: &[u8]) -> HashMap<String, String>;
}

struct Arrow;

impl Format for Arrow {
    type File<W: Write + Send> = IpcFile<W>;

    fn read(buf: &[u8]) -> (SchemaRef, RecordBatch) {
        let reader = FileReader::try_new(io::Cursor::new(buf), None).unwrap();
        let schema = reader.schema();
        let batches: Vec<_> = reader.map(|b| b.unwrap()).collect();
        let batch = ::arrow::compute::concat_batches(&schema, &batches).unwrap();
        (schema, batch)
    }

    fn metadata(buf: &[u8]) -> HashMap<String, String> {
        Self::read(buf).0.metadata().clone().into()
    }
}

#[cfg(feature = "parquet")]
struct Parquet;

#[cfg(feature = "parquet")]
impl Format for Parquet {
    type File<W: Write + Send> = crate::encode::parquet::ParquetFile<W>;

    fn read(buf: &[u8]) -> (SchemaRef, RecordBatch) {
        let builder = parquet_reader(buf);
        // The builder's schema carries the key-value metadata; the reader's doesn't
        let schema = builder.schema().clone();
        let batches: Vec<_> = builder.build().unwrap().map(|b| b.unwrap()).collect();
        let batch = ::arrow::compute::concat_batches(&schema, &batches).unwrap();
        (schema, batch)
    }

    fn metadata(buf: &[u8]) -> HashMap<String, String> {
        parquet_reader(buf)
            .metadata()
            .file_metadata()
            .key_value_metadata()
            .expect("expected key-value metadata")
            .iter()
            .filter(|kv| kv.key != "ARROW:schema")
            .map(|kv| (kv.key.clone(), kv.value.clone().unwrap_or_default()))
            .collect()
    }
}

#[cfg(feature = "parquet")]
fn parquet_reader(
    buf: &[u8],
) -> ::parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder<bytes::Bytes> {
    ::parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder::try_new(
        bytes::Bytes::copy_from_slice(buf),
    )
    .unwrap()
}

fn builder<F: Format>(buf: &mut Vec<u8>) -> EncoderBuilder<F::File<&mut Vec<u8>>> {
    EncoderBuilder::new(buf).schema(Schema::Mbo)
}

fn make_mbo(id: u32, price_units: i64) -> MboMsg {
    MboMsg {
        hd: RecordHeader::new::<MboMsg>(rtype::MBO, 1, id, 1658441851000000000),
        order_id: id as u64,
        price: price_units * FIXED_PRICE_SCALE,
        size: 10,
        flags: 0.into(),
        channel_id: 1,
        action: 'A' as c_char,
        side: 'B' as c_char,
        ts_recv: 1658441891000000000,
        ts_in_delta: 100,
        sequence: id,
    }
}

fn make_trade(id: u32) -> TradeMsg {
    TradeMsg {
        hd: RecordHeader::new::<TradeMsg>(rtype::MBP_0, 1, id, 1658441851000000000),
        ..Default::default()
    }
}

fn test_metadata() -> Metadata {
    Metadata {
        version: 3,
        dataset: "GLBX.MDP3".to_owned(),
        layout: None,
        schema: Some(Schema::Mbo),
        start: 1658441851000000000,
        end: None,
        limit: None,
        stype_in: Some(SType::RawSymbol),
        stype_out: SType::InstrumentId,
        ts_out: false,
        symbol_cstr_len: 22,
        symbols: vec!["ESM4".to_owned()],
        partial: vec![],
        not_found: vec![],
        intervals: vec![],
        mappings: vec![SymbolMapping {
            raw_symbol: "ESM4".to_owned(),
            intervals: vec![MappingInterval {
                start_date: date!(2024 - 03 - 18),
                end_date: date!(2024 - 06 - 21),
                symbol: "12345".to_owned(),
            }],
        }],
    }
}

/// The DBN metadata as `dbn --metadata` prints it.
fn metadata_json(metadata: &Metadata) -> String {
    let mut buf = Vec::new();
    JsonEncoder::new(&mut buf, false, false, false)
        .encode_metadata(metadata)
        .unwrap();
    String::from_utf8(buf).unwrap().trim_end().to_owned()
}

fn column<'a, A: 'static>(batch: &'a RecordBatch, name: &str) -> &'a A {
    batch
        .column_by_name(name)
        .unwrap_or_else(|| panic!("missing column {name}"))
        .as_any()
        .downcast_ref::<A>()
        .unwrap_or_else(|| panic!("column {name} has unexpected type"))
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_basic_roundtrip<F: Format>(#[case] _format: F) {
    let mut buf = Vec::new();
    {
        let mut encoder = builder::<F>(&mut buf).build().unwrap();
        for i in 0..5 {
            encoder.encode_record(&make_mbo(i, 100 + i as i64)).unwrap();
        }
        encoder.close().unwrap();
    }
    let (_, batch) = F::read(&buf);
    assert_eq!(batch.num_rows(), 5);
    let ids = column::<UInt32Array>(&batch, "instrument_id");
    let prices = column::<Int64Array>(&batch, "price");
    for i in 0..5 {
        assert_eq!(ids.value(i), i as u32);
        assert_eq!(prices.value(i), (100 + i as i64) * FIXED_PRICE_SCALE);
    }
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_batch_flushing<F: Format>(#[case] _format: F) {
    let mut buf = Vec::new();
    {
        let mut encoder = builder::<F>(&mut buf)
            .batch_size(NonZeroUsize::new(3).unwrap())
            .build()
            .unwrap();
        for i in 0..10 {
            encoder.encode_record(&make_mbo(i, 100)).unwrap();
        }
        encoder.close().unwrap();
    }
    let (_, batch) = F::read(&buf);
    assert_eq!(batch.num_rows(), 10);
    let ids = column::<UInt32Array>(&batch, "instrument_id");
    assert_eq!(ids.values().as_ref(), (0..10).collect::<Vec<u32>>());
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_drain_inner_between_flushes<F: Format>(#[case] _format: F) {
    let mut encoder = EncoderBuilder::<F::File<Vec<u8>>>::new(Vec::new())
        .schema(Schema::Mbo)
        .build()
        .unwrap();
    let mut chunks = Vec::new();
    for i in 0..10 {
        encoder.encode_record(&make_mbo(i, 100)).unwrap();
        encoder.flush().unwrap();
        chunks.push(std::mem::take(encoder.get_mut()));
    }
    encoder.close().unwrap();
    chunks.push(std::mem::take(encoder.get_mut()));
    assert!(chunks.iter().all(|c| !c.is_empty()));
    let (_, batch) = F::read(&chunks.concat());
    assert_eq!(batch.num_rows(), 10);
    let ids = column::<UInt32Array>(&batch, "instrument_id");
    assert_eq!(ids.values().as_ref(), (0..10).collect::<Vec<u32>>());
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_zstd_compression<F: Format>(#[case] _format: F) {
    let mut buf = Vec::new();
    {
        let mut encoder = builder::<F>(&mut buf)
            .compression(Compression::Zstd)
            .build()
            .unwrap();
        for i in 0..5 {
            encoder.encode_record(&make_mbo(i, 100)).unwrap();
        }
        encoder.close().unwrap();
    }
    let (_, batch) = F::read(&buf);
    assert_eq!(batch.num_rows(), 5);
    assert_eq!(column::<UInt32Array>(&batch, "instrument_id").value(4), 4);
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_pretty_px<F: Format>(#[case] _format: F) {
    let mut buf = Vec::new();
    {
        let mut encoder = builder::<F>(&mut buf).pretty_px(true).build().unwrap();
        encoder.encode_record(&make_mbo(1, 175)).unwrap();
        encoder.close().unwrap();
    }
    let (_, batch) = F::read(&buf);
    let prices = column::<Float64Array>(&batch, "price");
    assert!((prices.value(0) - 175.0).abs() < 1e-9);
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_sentinel_nulls<F: Format>(#[case] _format: F) {
    let mut record = make_mbo(1, 100);
    record.price = UNDEF_PRICE;
    record.hd.ts_event = UNDEF_TIMESTAMP;
    let mut buf = Vec::new();
    {
        let mut encoder = builder::<F>(&mut buf).build().unwrap();
        encoder.encode_record(&record).unwrap();
        encoder.close().unwrap();
    }
    let (_, batch) = F::read(&buf);
    assert!(batch.column_by_name("price").unwrap().is_null(0));
    assert!(batch.column_by_name("ts_event").unwrap().is_null(0));
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_ts_out_column<F: Format>(#[case] _format: F) {
    const TS_OUT_VAL: u64 = 1_700_000_000_000_000_000;
    let mut buf = Vec::new();
    {
        let mut encoder = builder::<F>(&mut buf).ts_out(true).build().unwrap();
        encoder
            .encode_record(&WithTsOut::new(make_mbo(1, 100), TS_OUT_VAL))
            .unwrap();
        encoder.close().unwrap();
    }
    let (_, batch) = F::read(&buf);
    assert_eq!(batch.num_rows(), 1);
    let ts_out = column::<TimestampNanosecondArray>(&batch, "ts_out");
    assert!(!ts_out.is_null(0));
    assert_eq!(ts_out.value(0) as u64, TS_OUT_VAL);
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_ts_out_via_record_ref<F: Format>(#[case] _format: F) {
    const TS_OUT_VAL: u64 = 1_678_480_044_000_000_000;
    let wrapped = WithTsOut::new(make_mbo(7, 100), TS_OUT_VAL);
    let mut buf = Vec::new();
    {
        let mut encoder = builder::<F>(&mut buf).ts_out(true).build().unwrap();
        // SAFETY: `wrapped` is a `WithTsOut<MboMsg>`
        unsafe { encoder.encode_record_ref_ts_out(RecordRef::from(&wrapped), true) }.unwrap();
        encoder.close().unwrap();
    }
    let (_, batch) = F::read(&buf);
    assert_eq!(batch.num_rows(), 1);
    assert_eq!(column::<UInt32Array>(&batch, "instrument_id").value(0), 7);
    let ts_out = column::<TimestampNanosecondArray>(&batch, "ts_out");
    assert_eq!(ts_out.value(0) as u64, TS_OUT_VAL);
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_ts_out_false_via_record_ref<F: Format>(#[case] _format: F) {
    let mbo = make_mbo(11, 200);
    let mut buf = Vec::new();
    {
        let mut encoder = builder::<F>(&mut buf).build().unwrap();
        // SAFETY: `mbo` is a bare `MboMsg`
        unsafe { encoder.encode_record_ref_ts_out(RecordRef::from(&mbo), false) }.unwrap();
        encoder.close().unwrap();
    }
    let (schema, batch) = F::read(&buf);
    assert!(schema.field_with_name("ts_out").is_err());
    assert_eq!(column::<UInt32Array>(&batch, "instrument_id").value(0), 11);
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_empty_output_is_valid_file<F: Format>(#[case] _format: F) {
    let metadata = test_metadata();
    let mut buf = Vec::new();
    {
        let mut encoder = builder::<F>(&mut buf).metadata(&metadata).build().unwrap();
        encoder.close().unwrap();
    }
    let (schema, batch) = F::read(&buf);
    assert_eq!(batch.num_rows(), 0);
    assert!(schema.field_with_name("price").is_ok());
    assert_eq!(F::metadata(&buf)[METADATA_KEY], metadata_json(&metadata));
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_metadata_embedding<F: Format>(#[case] _format: F) {
    let metadata = test_metadata();
    let mut buf = Vec::new();
    {
        let mut encoder = builder::<F>(&mut buf).metadata(&metadata).build().unwrap();
        encoder.encode_record(&make_mbo(1, 100)).unwrap();
        encoder.close().unwrap();
    }
    let kv = F::metadata(&buf);
    assert_eq!(kv.len(), 1);
    let json = &kv[METADATA_KEY];
    assert_eq!(*json, metadata_json(&metadata));
    for expected in [
        "\"dataset\":\"GLBX.MDP3\"",
        "\"schema\":\"mbo\"",
        "\"stype_in\":\"raw_symbol\"",
        "\"end\":null",
        "\"symbols\":[\"ESM4\"]",
        "\"mappings\":[{\"raw_symbol\":\"ESM4\",\"intervals\":[{\"start_date\":20240318,\"end_date\":20240621,\"symbol\":\"12345\"}]}]",
    ] {
        assert!(json.contains(expected), "{expected} not in {json}");
    }
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_metadata_embedding_empty_lists<F: Format>(#[case] _format: F) {
    let metadata = Metadata {
        stype_in: None,
        symbols: vec![],
        mappings: vec![],
        ..test_metadata()
    };
    let mut buf = Vec::new();
    {
        let mut encoder = builder::<F>(&mut buf).metadata(&metadata).build().unwrap();
        encoder.encode_record(&make_mbo(1, 100)).unwrap();
        encoder.close().unwrap();
    }
    let json = &F::metadata(&buf)[METADATA_KEY];
    assert_eq!(*json, metadata_json(&metadata));
    for expected in ["\"stype_in\":null", "\"symbols\":[]", "\"mappings\":[]"] {
        assert!(json.contains(expected), "{expected} not in {json}");
    }
}

/// Both formats carry the DBN metadata on the Arrow schema, and Parquet also as
/// plain key-value pairs, so every reader sees the same keys and values.
#[cfg(feature = "parquet")]
#[test]
fn test_metadata_parity_across_formats() {
    let metadata = test_metadata();
    let mut arrow_buf = Vec::new();
    let mut parquet_buf = Vec::new();
    builder::<Arrow>(&mut arrow_buf)
        .metadata(&metadata)
        .build()
        .unwrap()
        .close()
        .unwrap();
    builder::<Parquet>(&mut parquet_buf)
        .metadata(&metadata)
        .build()
        .unwrap()
        .close()
        .unwrap();

    let arrow_schema_md = Arrow::metadata(&arrow_buf);
    let parquet_kv = Parquet::metadata(&parquet_buf);
    let parquet_schema_md: HashMap<String, String> =
        Parquet::read(&parquet_buf).0.metadata().clone().into();
    assert_eq!(arrow_schema_md.len(), 1);
    assert_eq!(arrow_schema_md, parquet_kv);
    assert_eq!(arrow_schema_md, parquet_schema_md);
    let reader = FileReader::try_new(io::Cursor::new(&arrow_buf), None).unwrap();
    assert!(
        reader.custom_metadata().is_empty(),
        "metadata belongs on the schema, not the IPC footer"
    );
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_symbol_column<F: Format>(#[case] _format: F) {
    let mut buf = Vec::new();
    {
        let mut encoder = builder::<F>(&mut buf).with_symbol(true).build().unwrap();
        encoder
            .encode_record_with_sym(&make_mbo(1, 100), Some("ESM4"))
            .unwrap();
        encoder
            .encode_record_with_sym(&make_mbo(2, 100), None)
            .unwrap();
        encoder.close().unwrap();
    }
    let (_, batch) = F::read(&buf);
    assert_eq!(batch.num_rows(), 2);
    let symbols = column::<StringArray>(&batch, "symbol");
    assert_eq!(symbols.value(0), "ESM4");
    assert!(symbols.is_null(1));
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_with_symbol_method_mismatch_errors<F: Format>(#[case] _format: F) {
    let mut buf = Vec::new();
    let mut encoder = builder::<F>(&mut buf).with_symbol(true).build().unwrap();
    let err = encoder.encode_record(&make_mbo(1, 100)).unwrap_err();
    assert!(err.to_string().contains("with_symbol"), "{err}");
    let err = encoder
        .encode_record_ref(RecordRef::from(&make_mbo(1, 100)))
        .unwrap_err();
    assert!(err.to_string().contains("with_symbol"), "{err}");

    let mut buf = Vec::new();
    let mut encoder = builder::<F>(&mut buf).build().unwrap();
    let err = encoder
        .encode_record_with_sym(&make_mbo(1, 100), Some("ESM4"))
        .unwrap_err();
    assert!(err.to_string().contains("with_symbol"), "{err}");
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_mixed_record_types_error<F: Format>(#[case] _format: F) {
    let mbo = make_mbo(1, 100);
    let trade = make_trade(2);
    let with_ts_out = WithTsOut::new(make_mbo(3, 100), 1);

    let mut buf = Vec::new();
    {
        let mut encoder = builder::<F>(&mut buf).build().unwrap();
        encoder.encode_record(&mbo).unwrap();
        let err = encoder.encode_record(&trade).unwrap_err();
        assert!(err.to_string().contains("Mixed schemas"), "{err}");
        let err = encoder
            .encode_record_ref(RecordRef::from(&trade))
            .unwrap_err();
        assert!(err.to_string().contains("Mixed schemas"), "{err}");
        let err = encoder.encode_record(&with_ts_out).unwrap_err();
        assert!(err.to_string().contains("Mixed schemas"), "{err}");
        encoder.close().unwrap();
    }
    assert_eq!(F::read(&buf).1.num_rows(), 1);

    let mut buf = Vec::new();
    {
        let mut encoder = builder::<F>(&mut buf).ts_out(true).build().unwrap();
        let err = encoder.encode_record(&mbo).unwrap_err();
        assert!(err.to_string().contains("Mixed schemas"), "{err}");
        encoder.encode_record(&with_ts_out).unwrap();
        encoder.close().unwrap();
    }
    assert_eq!(F::read(&buf).1.num_rows(), 1);
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_encode_decoded_mixed_schema_errors<F: Format>(#[case] _format: F) {
    let metadata = Metadata {
        schema: None,
        ..test_metadata()
    };
    let mut dbn = Vec::new();
    {
        let mut encoder = DbnEncoder::new(&mut dbn, &metadata).unwrap();
        encoder.encode_record(&make_mbo(1, 100)).unwrap();
        encoder.encode_record(&make_trade(2)).unwrap();
    }

    let mut buf = Vec::new();
    let err = EncoderBuilder::<F::File<&mut Vec<u8>>>::new(&mut buf)
        .metadata(&metadata)
        .build()
        .map(|_| ())
        .unwrap_err();
    assert!(err.to_string().contains("mixed schemas"), "{err}");

    let mut buf = Vec::new();
    let mut encoder = builder::<F>(&mut buf).build().unwrap();
    let err = encoder
        .encode_decoded(DbnDecoder::new(dbn.as_slice()).unwrap())
        .unwrap_err();
    assert!(err.to_string().contains("Mixed schemas"), "{err}");
}

#[cfg(feature = "zstd")]
#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_encode_decoded_record_layouts_changed_between_versions<F: Format>(
    #[case] _format: F,
    #[values("definition", "statistics")] schema: &str,
    #[values(1, 2, 3)] version: u8,
) {
    use crate::decode::{tests::TEST_DATA_PATH, DbnMetadata, DecodeRecordRef};
    use crate::VersionUpgradePolicy;

    let path = format!("{TEST_DATA_PATH}/test_data.{schema}.v{version}.dbn.zst");
    let decoder = || {
        let reader = zstd::stream::Decoder::new(std::fs::File::open(&path).unwrap()).unwrap();
        DbnDecoder::with_upgrade_policy(reader, VersionUpgradePolicy::AsIs).unwrap()
    };
    let mut record_count = 0;
    let mut counter = decoder();
    while counter.decode_record_ref().unwrap().is_some() {
        record_count += 1;
    }
    assert!(record_count > 0);

    let decoder = decoder();
    assert_eq!(decoder.metadata().version, version);
    let mut buf = Vec::new();
    EncoderBuilder::<F::File<&mut Vec<u8>>>::new(&mut buf)
        .metadata(decoder.metadata())
        .build()
        .unwrap()
        .encode_decoded(decoder)
        .unwrap();
    assert_eq!(F::read(&buf).1.num_rows(), record_count);
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_encode_after_close_errors<F: Format>(#[case] _format: F) {
    let mut buf = Vec::new();
    let mut encoder = builder::<F>(&mut buf).build().unwrap();
    encoder.close().unwrap();
    assert!(encoder.is_closed());
    assert!(encoder.encode_record(&make_mbo(1, 100)).is_err());
    assert!(encoder.flush().is_err());
    encoder.close().unwrap();
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_drop_finalizes_file<F: Format>(#[case] _format: F) {
    let mut buf = Vec::new();
    {
        let mut encoder = builder::<F>(&mut buf).build().unwrap();
        encoder.encode_record(&make_mbo(1, 100)).unwrap();
    }
    assert_eq!(F::read(&buf).1.num_rows(), 1);
}

struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("disk full"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("disk full"))
    }
}

#[rstest]
#[case::arrow(Arrow)]
#[cfg_attr(feature = "parquet", case::parquet(Parquet))]
fn test_failing_writer_errors<F: Format>(#[case] _format: F) {
    // Arrow IPC writes its header when built; Parquet writes nothing until close
    let Ok(mut encoder) = EncoderBuilder::<F::File<FailingWriter>>::new(FailingWriter)
        .schema(Schema::Mbo)
        .build()
    else {
        return;
    };
    encoder.encode_record(&make_mbo(1, 100)).unwrap();
    assert!(encoder.close().is_err());
}

#[cfg(feature = "zstd")]
#[rstest]
#[case::arrow(Arrow, crate::encode::OutputEncoding::Arrow)]
#[cfg_attr(
    feature = "parquet",
    case::parquet(Parquet, crate::encode::OutputEncoding::Parquet)
)]
fn test_dyn_encoder_roundtrip<F: Format>(
    #[case] _format: F,
    #[case] encoding: crate::encode::OutputEncoding,
) {
    let metadata = test_metadata();
    let mut buf = Vec::new();
    let mut encoder =
        crate::encode::DynEncoder::builder(&mut buf, encoding, Compression::None, &metadata)
            .build()
            .unwrap();
    for i in 0..3 {
        encoder.encode_record(&make_mbo(i, 100)).unwrap();
    }
    encoder.finish().unwrap();
    let (_, batch) = F::read(&buf);
    assert_eq!(batch.num_rows(), 3);
    let ids = column::<UInt32Array>(&batch, "instrument_id");
    for i in 0..3 {
        assert_eq!(ids.value(i), i as u32);
    }
}

#[cfg(feature = "parquet")]
#[test]
fn test_dyn_encoder_finish_surfaces_write_error() {
    use crate::encode::{DynEncoder, OutputEncoding};

    let metadata = test_metadata();
    let mut encoder = DynEncoder::builder(
        FailingWriter,
        OutputEncoding::Parquet,
        Compression::None,
        &metadata,
    )
    .build()
    .unwrap();
    encoder.encode_record(&make_mbo(1, 100)).unwrap();
    assert!(encoder.finish().is_err());
}
