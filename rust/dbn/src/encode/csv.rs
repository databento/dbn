//! Encoding of DBN records into comma-separated values (CSV).
use std::io;

use streaming_iterator::StreamingIterator;

use super::EncodeDbn;

/// Type for encoding files and streams of DBN records in CSV.
///
/// Note that encoding [`Metadata`](crate::Metadata) in CSV is not supported.
pub struct Encoder<W>
where
    W: io::Write,
{
    writer: csv::Writer<W>,
}

impl<W> Encoder<W>
where
    W: io::Write,
{
    /// Creates a new [`Encoder`] that will write to `writer`.
    pub fn new(writer: W) -> Self {
        let csv_writer = csv::WriterBuilder::new()
            .has_headers(false) // need to write our own custom header
            .from_writer(writer);
        Self { writer: csv_writer }
    }
}

impl<W> EncodeDbn for Encoder<W>
where
    W: io::Write,
{
    fn encode_record<R: super::DbnEncodable>(&mut self, record: &R) -> anyhow::Result<bool> {
        match record.serialize_to(&mut self.writer) {
            Ok(_) => Ok(false),
            Err(e) => {
                if matches!(e.kind(), csv::ErrorKind::Io(io_err) if io_err.kind() == io::ErrorKind::BrokenPipe)
                {
                    // closed pipe, should stop writing output
                    Ok(true)
                } else {
                    Err(anyhow::Error::new(e).context(format!("Failed to serialize {record:#?}")))
                }
            }
        }
    }

    fn encode_records<R: super::DbnEncodable>(&mut self, records: &[R]) -> anyhow::Result<()> {
        R::serialize_headers(&mut self.writer)?;
        // end of line
        self.writer.write_record(None::<&[u8]>)?;
        for record in records {
            if self.encode_record(record)? {
                return Ok(());
            }
        }
        self.writer.flush()?;
        Ok(())
    }

    fn encode_stream<R: super::DbnEncodable>(
        &mut self,
        mut stream: impl StreamingIterator<Item = R>,
    ) -> anyhow::Result<()> {
        R::serialize_headers(&mut self.writer)?;
        // end of line
        self.writer.write_record(None::<&[u8]>)?;
        while let Some(record) = stream.next() {
            if self.encode_record(record)? {
                return Ok(());
            }
        }
        self.writer.flush()?;
        Ok(())
    }
}

pub(crate) mod serialize {
    use std::{fmt, io};

    use csv::Writer;
    use serde::Serialize;

    use crate::record::{
        ErrorMsg, HasRType, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, StatusMsg,
        SymbolMappingMsg, SystemMsg, TradeMsg, WithTsOut,
    };

    /// Because of the flat nature of CSVs, there are several limitations in the
    /// Rust CSV serde serialization library. This trait helps work around them.
    pub trait CsvSerialize: Serialize + fmt::Debug {
        /// Write the headers to `csv_writer`.
        fn serialize_headers<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()>;

        /// Serialize the object to `csv_writer`. Allows custom behavior that would otherwise
        /// cause a runtime error, e.g. serializing a struct with array field.
        fn serialize_to<W: io::Write>(&self, csv_writer: &mut Writer<W>) -> csv::Result<()> {
            csv_writer.serialize(self)
        }
    }

    impl CsvSerialize for MboMsg {
        fn serialize_headers<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            [
                "rtype",
                "publisher_id",
                "product_id",
                "ts_event",
                "order_id",
                "price",
                "size",
                "flags",
                "channel_id",
                "action",
                "side",
                "ts_recv",
                "ts_in_delta",
                "sequence",
            ]
            .iter()
            .try_for_each(|header| csv_writer.write_field(header))
        }
    }

    impl CsvSerialize for Mbp1Msg {
        fn serialize_headers<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            [
                "rtype",
                "publisher_id",
                "product_id",
                "ts_event",
                "price",
                "size",
                "action",
                "side",
                "flags",
                "depth",
                "ts_recv",
                "ts_in_delta",
                "sequence",
                "bid_px_00",
                "ask_px_00",
                "bid_sz_00",
                "ask_sz_00",
                "bid_ct_00",
                "ask_ct_00",
            ]
            .iter()
            .try_for_each(|header| csv_writer.write_field(header))
        }
    }

    impl CsvSerialize for Mbp10Msg {
        fn serialize_headers<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            [
                "rtype",
                "publisher_id",
                "product_id",
                "ts_event",
                "price",
                "size",
                "action",
                "side",
                "flags",
                "depth",
                "ts_recv",
                "ts_in_delta",
                "sequence",
                "bid_px_00",
                "ask_px_00",
                "bid_sz_00",
                "ask_sz_00",
                "bid_ct_00",
                "ask_ct_00",
                "bid_px_01",
                "ask_px_01",
                "bid_sz_01",
                "ask_sz_01",
                "bid_ct_01",
                "ask_ct_01",
                "bid_px_02",
                "ask_px_02",
                "bid_sz_02",
                "ask_sz_02",
                "bid_ct_02",
                "ask_ct_02",
                "bid_px_03",
                "ask_px_03",
                "bid_sz_03",
                "ask_sz_03",
                "bid_ct_03",
                "ask_ct_03",
                "bid_px_04",
                "ask_px_04",
                "bid_sz_04",
                "ask_sz_04",
                "bid_ct_04",
                "ask_ct_04",
                "bid_px_05",
                "ask_px_05",
                "bid_sz_05",
                "ask_sz_05",
                "bid_ct_05",
                "ask_ct_05",
                "bid_px_06",
                "ask_px_06",
                "bid_sz_06",
                "ask_sz_06",
                "bid_ct_06",
                "ask_ct_06",
                "bid_px_07",
                "ask_px_07",
                "bid_sz_07",
                "ask_sz_07",
                "bid_ct_07",
                "ask_ct_07",
                "bid_px_08",
                "ask_px_08",
                "bid_sz_08",
                "ask_sz_08",
                "bid_ct_08",
                "ask_ct_08",
                "bid_px_09",
                "ask_px_09",
                "bid_sz_09",
                "ask_sz_09",
                "bid_ct_09",
                "ask_ct_09",
            ]
            .iter()
            .try_for_each(|header| csv_writer.write_field(header))
        }

        fn serialize_to<W: io::Write>(&self, csv_writer: &mut Writer<W>) -> csv::Result<()> {
            csv_writer.write_field(self.hd.rtype.to_string())?;
            csv_writer.write_field(self.hd.publisher_id.to_string())?;
            csv_writer.write_field(self.hd.product_id.to_string())?;
            csv_writer.write_field(self.hd.ts_event.to_string())?;
            csv_writer.write_field(self.price.to_string())?;
            csv_writer.write_field(self.size.to_string())?;
            csv_writer.write_field((self.action as u8 as char).to_string())?;
            csv_writer.write_field((self.side as u8 as char).to_string())?;
            csv_writer.write_field(self.flags.to_string())?;
            csv_writer.write_field(self.depth.to_string())?;
            csv_writer.write_field(self.ts_recv.to_string())?;
            csv_writer.write_field(self.ts_in_delta.to_string())?;
            csv_writer.write_field(self.sequence.to_string())?;
            for level in self.booklevel.iter() {
                csv_writer.write_field(level.bid_px.to_string())?;
                csv_writer.write_field(level.ask_px.to_string())?;
                csv_writer.write_field(level.bid_sz.to_string())?;
                csv_writer.write_field(level.ask_sz.to_string())?;
                csv_writer.write_field(level.bid_ct.to_string())?;
                csv_writer.write_field(level.ask_ct.to_string())?;
            }
            // end of line
            csv_writer.write_record(None::<&[u8]>)?;
            Ok(())
        }
    }

    impl CsvSerialize for TradeMsg {
        fn serialize_headers<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            [
                "rtype",
                "publisher_id",
                "product_id",
                "ts_event",
                "price",
                "size",
                "action",
                "side",
                "flags",
                "depth",
                "ts_recv",
                "ts_in_delta",
                "sequence",
            ]
            .iter()
            .try_for_each(|header| csv_writer.write_field(header))
        }
    }

    impl CsvSerialize for OhlcvMsg {
        fn serialize_headers<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            [
                "rtype",
                "publisher_id",
                "product_id",
                "ts_event",
                "open",
                "high",
                "low",
                "close",
                "volume",
            ]
            .iter()
            .try_for_each(|header| csv_writer.write_field(header))
        }
    }

    impl CsvSerialize for StatusMsg {
        fn serialize_headers<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            [
                "rtype",
                "publisher_id",
                "product_id",
                "ts_event",
                "ts_recv",
                "group",
                "trading_status",
                "halt_reason",
                "trading_event",
            ]
            .iter()
            .try_for_each(|header| csv_writer.write_field(header))
        }
    }

    impl CsvSerialize for InstrumentDefMsg {
        fn serialize_headers<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            [
                "rtype",
                "publisher_id",
                "product_id",
                "ts_event",
                "ts_recv",
                "min_price_increment",
                "display_factor",
                "expiration",
                "activation",
                "high_limit_price",
                "low_limit_price",
                "max_price_variation",
                "trading_reference_price",
                "unit_of_measure_qty",
                "min_price_increment_amount",
                "price_ratio",
                "inst_attrib_value",
                "underlying_id",
                "cleared_volume",
                "market_depth_implied",
                "market_depth",
                "market_segment_id",
                "max_trade_vol",
                "min_lot_size",
                "min_lot_size_block",
                "min_lot_size_round_lot",
                "min_trade_vol",
                "open_interest_qty",
                "contract_multiplier",
                "decay_quantity",
                "original_contract_size",
                "related_security_id",
                "trading_reference_date",
                "appl_id",
                "maturity_year",
                "decay_start_date",
                "channel_id",
                "currency",
                "settl_currency",
                "secsubtype",
                "symbol",
                "group",
                "exchange",
                "asset",
                "cfi",
                "security_type",
                "unit_of_measure",
                "underlying",
                "related",
                "match_algorithm",
                "md_security_trading_status",
                "main_fraction",
                "price_display_format",
                "settl_price_type",
                "sub_fraction",
                "underlying_product",
                "security_update_action",
                "maturity_month",
                "maturity_day",
                "maturity_week",
                "user_defined_instrument",
                "contract_multiplier_unit",
                "flow_schedule_type",
                "tick_rule",
            ]
            .iter()
            .try_for_each(|header| csv_writer.write_field(header))
        }
    }

    impl CsvSerialize for ErrorMsg {
        fn serialize_headers<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            ["rtype", "publisher_id", "product_id", "ts_event", "err"]
                .iter()
                .try_for_each(|header| csv_writer.write_field(header))
        }
    }

    impl CsvSerialize for SystemMsg {
        fn serialize_headers<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            ["rtype", "publisher_id", "product_id", "ts_event", "msg"]
                .iter()
                .try_for_each(|header| csv_writer.write_field(header))
        }
    }

    impl CsvSerialize for SymbolMappingMsg {
        fn serialize_headers<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            [
                "rtype",
                "publisher_id",
                "product_id",
                "ts_event",
                "stype_in_symbol",
                "stype_out_symbol",
                "start_ts",
                "end_ts",
            ]
            .iter()
            .try_for_each(|header| csv_writer.write_field(header))
        }
    }

    impl<T: HasRType + CsvSerialize> CsvSerialize for WithTsOut<T> {
        fn serialize_headers<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            csv_writer.write_field("ts_out")?;
            T::serialize_headers(csv_writer)
        }

        fn serialize_to<W: io::Write>(&self, csv_writer: &mut Writer<W>) -> csv::Result<()> {
            csv_writer.write_field(self.ts_out.to_string())?;
            csv_writer.serialize(&self.rec)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{array, io::BufWriter, os::raw::c_char};

    use super::*;
    use crate::{
        encode::test_data::{VecStream, BID_ASK, RECORD_HEADER},
        enums::SecurityUpdateAction,
        record::{
            str_to_c_chars, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, StatusMsg,
            TradeMsg, WithTsOut,
        },
    };

    const HEADER_CSV: &str = "4,1,323,1658441851000000000";

    const BID_ASK_CSV: &str = "372000000000000,372500000000000,10,5,5,2";

    fn extract_2nd_line(buffer: Vec<u8>) -> String {
        let output = String::from_utf8(buffer).expect("valid UTF-8");
        output
            .split_once('\n')
            .expect("two lines")
            .1
            .trim_end() // remove newline
            .to_owned()
    }

    #[test]
    fn test_mbo_encode_stream() {
        let data = vec![MboMsg {
            hd: RECORD_HEADER,
            order_id: 16,
            price: 5500,
            size: 3,
            flags: 128,
            channel_id: 14,
            action: 'B' as c_char,
            side: 'B' as c_char,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer)
            .encode_stream(VecStream::new(data))
            .unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(
            line,
            format!("{HEADER_CSV},16,5500,3,128,14,B,B,1658441891000000000,22000,1002375")
        );
    }

    #[test]
    fn test_mbp1_encode_records() {
        let data = vec![Mbp1Msg {
            hd: RECORD_HEADER,
            price: 5500,
            size: 3,
            action: 'M' as c_char,
            side: 'A' as c_char,
            flags: 128,
            depth: 9,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
            booklevel: [BID_ASK],
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer)
            .encode_records(data.as_slice())
            .unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(
            line,
            format!(
                "{HEADER_CSV},5500,3,M,A,128,9,1658441891000000000,22000,1002375,{BID_ASK_CSV}"
            )
        );
    }

    #[test]
    fn test_mbo10_encode_stream() {
        let data = vec![Mbp10Msg {
            hd: RECORD_HEADER,
            price: 5500,
            size: 3,
            action: 'B' as c_char,
            side: 'A' as c_char,
            flags: 128,
            depth: 9,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
            booklevel: array::from_fn(|_| BID_ASK.clone()),
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer)
            .encode_stream(VecStream::new(data))
            .unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(
            line,
            format!("{HEADER_CSV},5500,3,B,A,128,9,1658441891000000000,22000,1002375,{BID_ASK_CSV},{BID_ASK_CSV},{BID_ASK_CSV},{BID_ASK_CSV},{BID_ASK_CSV},{BID_ASK_CSV},{BID_ASK_CSV},{BID_ASK_CSV},{BID_ASK_CSV},{BID_ASK_CSV}")
        );
    }

    #[test]
    fn test_trade_encode_records() {
        let data = vec![TradeMsg {
            hd: RECORD_HEADER,
            price: 5500,
            size: 3,
            action: 'B' as c_char,
            side: 'B' as c_char,
            flags: 128,
            depth: 9,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
            booklevel: [],
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer)
            .encode_records(data.as_slice())
            .unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(
            line,
            format!("{HEADER_CSV},5500,3,B,B,128,9,1658441891000000000,22000,1002375")
        );
    }

    #[test]
    fn test_ohlcv_encode_stream() {
        let data = vec![OhlcvMsg {
            hd: RECORD_HEADER,
            open: 5000,
            high: 8000,
            low: 3000,
            close: 6000,
            volume: 55_000,
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer)
            .encode_stream(VecStream::new(data))
            .unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(line, format!("{HEADER_CSV},5000,8000,3000,6000,55000"));
    }

    #[test]
    fn test_status_encode_records() {
        let mut group = [0; 21];
        for (i, c) in "group".chars().enumerate() {
            group[i] = c as c_char;
        }
        let data = vec![StatusMsg {
            hd: RECORD_HEADER,
            ts_recv: 1658441891000000000,
            group,
            trading_status: 3,
            halt_reason: 4,
            trading_event: 6,
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer)
            .encode_records(data.as_slice())
            .unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(
            line,
            format!("{HEADER_CSV},1658441891000000000,group,3,4,6")
        );
    }

    #[test]
    fn test_instrument_def_encode_stream() {
        let data = vec![InstrumentDefMsg {
            hd: RECORD_HEADER,
            ts_recv: 1658441891000000000,
            min_price_increment: 100,
            display_factor: 1000,
            expiration: 1698450000000000000,
            activation: 1697350000000000000,
            high_limit_price: 1_000_000,
            low_limit_price: -1_000_000,
            max_price_variation: 0,
            trading_reference_price: 500_000,
            unit_of_measure_qty: 5,
            min_price_increment_amount: 5,
            price_ratio: 10,
            inst_attrib_value: 10,
            underlying_id: 256785,
            cleared_volume: 0,
            market_depth_implied: 0,
            market_depth: 13,
            market_segment_id: 0,
            max_trade_vol: 10_000,
            min_lot_size: 1,
            min_lot_size_block: 1000,
            min_lot_size_round_lot: 100,
            min_trade_vol: 1,
            open_interest_qty: 0,
            contract_multiplier: 0,
            decay_quantity: 0,
            original_contract_size: 0,
            related_security_id: 0,
            trading_reference_date: 0,
            appl_id: 0,
            maturity_year: 0,
            decay_start_date: 0,
            channel_id: 4,
            currency: [0; 4],
            settl_currency: str_to_c_chars("USD").unwrap(),
            secsubtype: [0; 6],
            symbol: [0; 22],
            group: [0; 21],
            exchange: [0; 5],
            asset: [0; 7],
            cfi: [0; 7],
            security_type: [0; 7],
            unit_of_measure: [0; 31],
            underlying: [0; 21],
            related: [0; 21],
            match_algorithm: 'F' as c_char,
            md_security_trading_status: 2,
            main_fraction: 4,
            price_display_format: 8,
            settl_price_type: 9,
            sub_fraction: 23,
            underlying_product: 10,
            security_update_action: SecurityUpdateAction::Add,
            maturity_month: 8,
            maturity_day: 9,
            maturity_week: 11,
            user_defined_instrument: 'N' as c_char,
            contract_multiplier_unit: 0,
            flow_schedule_type: 5,
            tick_rule: 0,
            _dummy: [0; 3],
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer)
            .encode_stream(VecStream::new(data))
            .unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(line, format!("{HEADER_CSV},1658441891000000000,100,1000,1698450000000000000,1697350000000000000,1000000,-1000000,0,500000,5,5,10,10,256785,0,0,13,0,10000,1,1000,100,1,0,0,0,0,0,0,0,0,0,4,,USD,,,,,,,,,,,F,2,4,8,9,23,10,A,8,9,11,N,0,5,0"));
    }

    #[test]
    fn test_encode_with_ts_out() {
        let data = vec![WithTsOut {
            rec: TradeMsg {
                hd: RECORD_HEADER,
                price: 5500,
                size: 3,
                action: 'T' as c_char,
                side: 'A' as c_char,
                flags: 128,
                depth: 9,
                ts_recv: 1658441891000000000,
                ts_in_delta: 22_000,
                sequence: 1_002_375,
                booklevel: [],
            },
            ts_out: 1678480044000000000,
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer)
            .encode_records(data.as_slice())
            .unwrap();
        let lines = String::from_utf8(buffer).expect("valid UTF-8");
        assert_eq!(
            lines,
            format!("ts_out,rtype,publisher_id,product_id,ts_event,price,size,action,side,flags,depth,ts_recv,ts_in_delta,sequence\n1678480044000000000,{HEADER_CSV},5500,3,T,A,128,9,1658441891000000000,22000,1002375\n")
        );
    }
}
