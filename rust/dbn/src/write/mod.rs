mod csv;
pub(crate) mod dbn;
mod json;

use std::{fmt, io};

use anyhow::anyhow;
use serde_json::ser::CompactFormatter;

use self::{
    csv::{serialize::CsvSerialize, write_csv},
    json::{pretty_formatter, write_json, write_json_metadata},
};
use crate::{
    enums::Schema,
    record::{
        ConstTypeId, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, StatusMsg, TbboMsg,
        TradeMsg,
    },
    Dbn, Metadata,
};

/// An encoding that DBNs can be translated to.
#[derive(Clone, Copy, Debug)]
pub enum OutputEncoding {
    /// Comma-separate values.
    Csv,
    /// JavaScript object notation.
    Json { should_pretty_print: bool },
}

impl<R: io::BufRead> Dbn<R> {
    /// Streams the contents of the [`Dbn`] to `writer` encoding it using `encoding`. Consumes the
    /// [`Dbn`] object.
    ///
    /// # Errors
    /// This function returns an error if [`Dbn::schema()`] is
    /// [`Schema::Statistics`](crate::enums::Schema::Statistics). It will also
    /// return an error if there's an issue writing the output to `writer`.
    pub fn write_to(self, writer: impl io::Write, encoding: OutputEncoding) -> anyhow::Result<()> {
        match self.schema() {
            Schema::Mbo => self.write_with_tick_to::<MboMsg, _>(writer, encoding),
            Schema::Mbp1 => self.write_with_tick_to::<Mbp1Msg, _>(writer, encoding),
            Schema::Mbp10 => self.write_with_tick_to::<Mbp10Msg, _>(writer, encoding),
            Schema::Tbbo => self.write_with_tick_to::<TbboMsg, _>(writer, encoding),
            Schema::Trades => self.write_with_tick_to::<TradeMsg, _>(writer, encoding),
            Schema::Ohlcv1S | Schema::Ohlcv1M | Schema::Ohlcv1H | Schema::Ohlcv1D => {
                self.write_with_tick_to::<OhlcvMsg, _>(writer, encoding)
            }
            Schema::Definition => self.write_with_tick_to::<InstrumentDefMsg, _>(writer, encoding),
            Schema::Statistics => Err(anyhow!("Not implemented for schema Statistics")),
            Schema::Status => self.write_with_tick_to::<StatusMsg, _>(writer, encoding),
        }
    }

    fn write_with_tick_to<T, W>(self, writer: W, encoding: OutputEncoding) -> anyhow::Result<()>
    where
        T: ConstTypeId + CsvSerialize + fmt::Debug,
        W: io::Write,
    {
        let iter = self.try_into_iter::<T>()?;
        match encoding {
            OutputEncoding::Csv => write_csv(writer, iter),
            OutputEncoding::Json {
                should_pretty_print,
            } => {
                if should_pretty_print {
                    write_json(writer, pretty_formatter(), iter)
                } else {
                    write_json(writer, CompactFormatter, iter)
                }
            }
        }
    }
}

impl Metadata {
    /// Writes the metadata to `writer` encoding it using `encoding`, if supported.
    ///
    /// # Note
    /// Encoding Metadata as CSV is unsupported.
    ///
    /// # Errors
    /// This function returns an error if [`Dbn::schema()`] is
    /// [`Schema::Statistics`](crate::enums::Schema::Statistics). It will also
    /// return an error if there's an issue writing the output to `writer`.
    pub fn write_to(&self, writer: impl io::Write, encoding: OutputEncoding) -> anyhow::Result<()> {
        match encoding {
            OutputEncoding::Csv => Err(anyhow!(
                "Encode metadata as a CSV is unsupported because it isn't tabular"
            )),
            OutputEncoding::Json {
                should_pretty_print,
            } => {
                if should_pretty_print {
                    write_json_metadata(writer, pretty_formatter(), self)
                } else {
                    write_json_metadata(writer, CompactFormatter, self)
                }
            }
        }
    }
}

#[cfg(test)]
mod test_data {
    use streaming_iterator::StreamingIterator;

    use crate::record::{BidAskPair, RecordHeader};

    // Common data used in multiple tests
    pub const RECORD_HEADER: RecordHeader = RecordHeader {
        length: 30,
        rtype: 4,
        publisher_id: 1,
        product_id: 323,
        ts_event: 1658441851000000000,
    };

    pub const BID_ASK: BidAskPair = BidAskPair {
        bid_px: 372000000000000,
        ask_px: 372500000000000,
        bid_sz: 10,
        ask_sz: 5,
        bid_ct: 5,
        ask_ct: 2,
    };

    /// A testing shim to get a streaming iterator from a [`Vec`].
    pub struct VecStream<T> {
        vec: Vec<T>,
        idx: isize,
    }

    impl<T> VecStream<T> {
        pub fn new(vec: Vec<T>) -> Self {
            // initialize at -1 because `advance()` is always called before
            // `get()`.
            Self { vec, idx: -1 }
        }
    }

    impl<T> StreamingIterator for VecStream<T> {
        type Item = T;

        fn advance(&mut self) {
            self.idx += 1;
        }

        fn get(&self) -> Option<&Self::Item> {
            self.vec.get(self.idx as usize)
        }
    }
}
