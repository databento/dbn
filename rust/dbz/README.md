# dbz

[![build](https://github.com/databento/dbz/actions/workflows/build.yml/badge.svg)](https://github.com/databento/dbz/actions/workflows/build.yml)
![license](https://img.shields.io/github/license/databento/dbz?color=blue)
[![Current Crates.io Version](https://img.shields.io/crates/v/dbz.svg)](https://crates.io/crates/dbz)

The official library for working with the Databento Binary Encoding (DBZ) format.

## Usage

The primary point for entrypoint for `dbz` is the `Dbz` object, which
represents the contents of one DBZ file or byte stream.
To read a DBZ file with MBO data and print each row:
```rust
use databento_defs::tick::TickMsg;
use dbz::Dbz;
use streaming_iterator::StreamingIterator;

let dbz = Dbz::from_file("20201228.dbz")?.try_into_iter::<TickMsg>()?;
while let Some(tick) = dbz.next() {
    println!("{tick:?}");
}
```

## Documentation

See [the docs](https://docs.rs/dbz) for more detailed usage.

## Building

`dbz` is written in Rust, so you'll need to have [Rust installed](https://www.rust-lang.org/)
first.

To build, run the following commands:
```sh
git clone https://github.com/databento/dbz
cd dbz
cargo build --release
```

## Testing

Tests are run through `cargo test` and are located within each module.

## License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
