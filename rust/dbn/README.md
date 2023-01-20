# dbn

[![build](https://github.com/databento/dbn/actions/workflows/build.yaml/badge.svg)](https://github.com/databento/dbn/actions/workflows/build.yaml)
![license](https://img.shields.io/github/license/databento/dbn?color=blue)
[![Current Crates.io Version](https://img.shields.io/crates/v/dbn.svg)](https://crates.io/crates/dbn)

The official library for working with the Databento Binary Encoding (DBN) format.

## Usage

The primary point for entrypoint for `dbn` is the `Dbn` object, which
represents the contents of one DBN file or byte stream.
To read a DBN file with MBO data and print each row:
```rust
use databento_defs::tick::TickMsg;
use dbn::Dbn;
use streaming_iterator::StreamingIterator;

let dbn = Dbn::from_file("20201228.dbn.zst")?.try_into_iter::<TickMsg>()?;
while let Some(tick) = dbn.next() {
    println!("{tick:?}");
}
```

## Documentation

See [the docs](https://docs.rs/dbn) for more detailed usage.

## Building

`dbn` is written in Rust, so you'll need to have [Rust installed](https://www.rust-lang.org/)
first.

To build, run the following commands:
```sh
git clone https://github.com/databento/dbn
cd dbn
cargo build --release
```

## Testing

Tests are run through `cargo test` and are located within each module.

## License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
