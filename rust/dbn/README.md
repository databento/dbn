# dbn

[![build](https://github.com/databento/dbn/actions/workflows/build.yaml/badge.svg)](https://github.com/databento/dbn/actions/workflows/build.yaml)
![license](https://img.shields.io/github/license/databento/dbn?color=blue)
[![Current Crates.io Version](https://img.shields.io/crates/v/dbn.svg)](https://crates.io/crates/dbn)

The official library for working with the Databento Binary Encoding (DBN, formerly DBZ).

## Usage

To read a DBN file with MBO data and print each row:
```rust
use dbn::{
    decode::dbn::Decoder,
    records::MboMsg,
};
use streaming_iterator::StreamingIterator;

let mut dbn_stream = Decoder::from_zstd_file("20201228.dbn.zst")?.decode_stream::<MboMsg>()?;
while let Some(mbo_msg) = dbn_stream.next() {
    println!("{mbo_msg:?}");
}
```

## Documentation

See [the docs](https://docs.rs/dbn) for more detailed usage.

## License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
