use dbn::decode::DecodeDbn;
use dbn::{decode::dbn::Decoder, record::MboMsg};
use polars::io::parquet::ZstdLevel;
use polars::prelude::IntoLazy;
use streaming_iterator::StreamingIterator;

fn main() -> Result<(), dbn::Error> {
    let dbn_stream = Decoder::from_zstd_file("../../tests/data/test_data.mbo.dbn.zst")?;
    let mut iter = dbn_stream.decode_stream::<MboMsg>();

    let mut stack = vec![];
    while let Some(mbo_msg) = iter.next() {
        println!("{mbo_msg:#?}");
        stack.push(mbo_msg.clone());
    }

    let fields = serde_arrow::arrow::serialize_into_fields(
        &stack,
        serde_arrow::schema::TracingOptions {
            allow_null_fields: true,
            map_as_struct: false,
            string_dictionary_encoding: false,
            coerce_numbers: false,
            try_parse_dates: true,
        },
    )
    .unwrap();
    println!("{fields:#?}");
    let arrays = serde_arrow::arrow::serialize_into_arrays(&fields, &stack).unwrap();
    let columns_func = |arrays| {
        fields
            .iter()
            .zip(arrays)
            .map(|(f, arr)| polars::series::Series::from_arrow_rs(f.name(), &arr).unwrap())
    };

    let df = polars::frame::DataFrame::new(columns_func(arrays.clone()).collect()).unwrap();
    println!("{df}");
    df.lazy()
        .sink_parquet(
            "./df.parquet".into(),
            polars::prelude::ParquetWriteOptions {
                compression: polars::io::parquet::ParquetCompression::Zstd(Some(ZstdLevel::try_new(15).unwrap())),
                statistics: true,
                row_group_size: None,
                data_pagesize_limit: None,
                maintain_order: false
            },
        )
        .unwrap();

    // flatten df

    let func = |i: polars::series::Series| match i.struct_() {
        Ok(i) => i
            .clone()
            .unnest()
            .iter()
            .map(|i| i.clone())
            .collect::<Vec<_>>(),
        _ => vec![i],
    };

    let df2 =
        polars::frame::DataFrame::new(columns_func(arrays.clone()).map(func).flatten().collect())
            .unwrap();
    println!("{df2}");

    df2.lazy()
        .sink_parquet(
            "./df2.parquet".into(),
            polars::prelude::ParquetWriteOptions {
                compression: polars::io::parquet::ParquetCompression::Zstd(Some(ZstdLevel::try_new(15).unwrap())),
                statistics: true,
                row_group_size: None,
                data_pagesize_limit: None,
                maintain_order: false
            },
        )
        .unwrap();

    Ok(())
}
