use std::{fmt, io};

use anyhow::Context;
use serde::Serialize;
use streaming_iterator::StreamingIterator;

use databento_defs::record::ConstTypeId;

/// Incrementally serializes the contents of `iter` into CSV to `writer` so the
/// contents of `iter` are not all buffered into memory at once.
pub fn write_csv<T>(
    writer: impl io::Write,
    mut iter: impl StreamingIterator<Item = T>,
) -> anyhow::Result<()>
where
    T: ConstTypeId + serialize::CsvSerialize + Serialize + fmt::Debug,
{
    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(false) // need to write our own custom header
        .from_writer(writer);
    csv_writer.write_record(T::HEADERS)?;
    while let Some(record) = iter.next() {
        match record.serialize_to(&mut csv_writer) {
            Err(e) => {
                if matches!(e.kind(), csv::ErrorKind::Io(io_err) if io_err.kind() == io::ErrorKind::BrokenPipe) {
                    // closed pipe, should stop writing output
                    return Ok(())
                 } else {
                    Err(e)
                 }
            }
            r => r,
        }
        .with_context(|| format!("Failed to serialize {record:#?}"))?;
    }
    csv_writer.flush()?;
    Ok(())
}

pub mod serialize {
    use csv::Writer;
    use databento_defs::record::{
        InstrumentDefMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, StatusMsg, TickMsg, TradeMsg,
    };
    use serde::Serialize;
    use std::{fmt, io};

    /// Because of the flat nature of CSVs, there are several limitations in the
    /// Rust CSV serde serialization library. This trait helps work around them.
    pub trait CsvSerialize: Serialize + fmt::Debug {
        /// The CSV header needs to be defined in a flat struct (no nested structs)
        /// in order to work correctly and the library doesn't support `#[serde(flatten)]`.
        const HEADERS: &'static [&'static str];

        /// Serialize the object to `csv_writer`. Allows custom behavior that would otherwise
        /// cause a runtime error, e.g. serializing a struct with array field.
        fn serialize_to<W: io::Write>(&self, csv_writer: &mut Writer<W>) -> csv::Result<()> {
            csv_writer.serialize(self)
        }
    }

    impl CsvSerialize for TickMsg {
        const HEADERS: &'static [&'static str] = &[
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
        ];
    }

    impl CsvSerialize for Mbp1Msg {
        const HEADERS: &'static [&'static str] = &[
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
        ];
    }

    impl CsvSerialize for Mbp10Msg {
        const HEADERS: &'static [&'static str] = &[
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
        ];

        fn serialize_to<W: io::Write>(&self, csv_writer: &mut Writer<W>) -> csv::Result<()> {
            csv_writer.write_field(self.hd.rtype.to_string())?;
            csv_writer.write_field(self.hd.publisher_id.to_string())?;
            csv_writer.write_field(self.hd.product_id.to_string())?;
            csv_writer.write_field(self.hd.ts_event.to_string())?;
            csv_writer.write_field(self.price.to_string())?;
            csv_writer.write_field(self.size.to_string())?;
            csv_writer.write_field(self.action.to_string())?;
            csv_writer.write_field(self.side.to_string())?;
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
        const HEADERS: &'static [&'static str] = &[
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
        ];
    }

    impl CsvSerialize for OhlcvMsg {
        const HEADERS: &'static [&'static str] = &[
            "rtype",
            "publisher_id",
            "product_id",
            "ts_event",
            "open",
            "high",
            "low",
            "close",
            "volume",
        ];
    }

    impl CsvSerialize for StatusMsg {
        const HEADERS: &'static [&'static str] = &[
            "rtype",
            "publisher_id",
            "product_id",
            "ts_event",
            "ts_recv",
            "group",
            "trading_status",
            "halt_reason",
            "trading_event",
        ];
    }

    impl CsvSerialize for InstrumentDefMsg {
        const HEADERS: &'static [&'static str] = &[
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
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::write::test_data::{VecStream, BID_ASK, RECORD_HEADER};
    use databento_defs::record::{
        InstrumentDefMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, SecurityUpdateAction, StatusMsg, TickMsg,
        TradeMsg,
    };
    use std::{io::BufWriter, os::raw::c_char};

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
    fn test_tick_write_csv() {
        let data = vec![TickMsg {
            hd: RECORD_HEADER,
            order_id: 16,
            price: 5500,
            size: 3,
            flags: 128,
            channel_id: 14,
            action: 'B' as i8,
            side: 67,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        write_csv(writer, VecStream::new(data)).unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(
            line,
            format!("{HEADER_CSV},16,5500,3,128,14,66,67,1658441891000000000,22000,1002375")
        );
    }

    #[test]
    fn test_mbo1_write_csv() {
        let data = vec![Mbp1Msg {
            hd: RECORD_HEADER,
            price: 5500,
            size: 3,
            action: 'B' as i8,
            side: 67,
            flags: 128,
            depth: 9,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
            booklevel: [BID_ASK; 1],
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        write_csv(writer, VecStream::new(data)).unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(
            line,
            format!(
                "{HEADER_CSV},5500,3,66,67,128,9,1658441891000000000,22000,1002375,{BID_ASK_CSV}"
            )
        );
    }

    #[test]
    fn test_mbo10_write_csv() {
        let data = vec![Mbp10Msg {
            hd: RECORD_HEADER,
            price: 5500,
            size: 3,
            action: 'B' as i8,
            side: 67,
            flags: 128,
            depth: 9,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
            booklevel: [BID_ASK; 10],
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        write_csv(writer, VecStream::new(data)).unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(
            line,
            format!("{HEADER_CSV},5500,3,66,67,128,9,1658441891000000000,22000,1002375,{BID_ASK_CSV},{BID_ASK_CSV},{BID_ASK_CSV},{BID_ASK_CSV},{BID_ASK_CSV},{BID_ASK_CSV},{BID_ASK_CSV},{BID_ASK_CSV},{BID_ASK_CSV},{BID_ASK_CSV}")
        );
    }

    #[test]
    fn test_trade_write_csv() {
        let data = vec![TradeMsg {
            hd: RECORD_HEADER,
            price: 5500,
            size: 3,
            action: 'B' as i8,
            side: 67,
            flags: 128,
            depth: 9,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
            booklevel: [],
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        write_csv(writer, VecStream::new(data)).unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(
            line,
            format!("{HEADER_CSV},5500,3,66,67,128,9,1658441891000000000,22000,1002375")
        );
    }

    #[test]
    fn test_ohlcv_write_csv() {
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
        write_csv(writer, VecStream::new(data)).unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(line, format!("{HEADER_CSV},5000,8000,3000,6000,55000"));
    }

    #[test]
    fn test_status_write_csv() {
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
        write_csv(writer, VecStream::new(data)).unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(
            line,
            format!("{HEADER_CSV},1658441891000000000,group,3,4,6")
        );
    }

    #[test]
    fn test_instrument_def_write_csv() {
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
            settl_currency: ['U' as c_char, 'S' as c_char, 'D' as c_char, 0],
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
            match_algorithm: 1,
            md_security_trading_status: 2,
            main_fraction: 4,
            price_display_format: 8,
            settl_price_type: 9,
            sub_fraction: 23,
            underlying_product: 10,
            security_update_action: SecurityUpdateAction::Invalid,
            maturity_month: 8,
            maturity_day: 9,
            maturity_week: 11,
            user_defined_instrument: 1,
            contract_multiplier_unit: 0,
            flow_schedule_type: 5,
            tick_rule: 0,
            _dummy: [0; 3],
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        write_csv(writer, VecStream::new(data)).unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(line, format!("{HEADER_CSV},1658441891000000000,100,1000,1698450000000000000,1697350000000000000,1000000,-1000000,0,500000,5,5,10,10,256785,0,0,13,0,10000,1,1000,100,1,0,0,0,0,0,0,0,0,0,4,,USD,,,,,,,,,,,1,2,4,8,9,23,10,Invalid,8,9,11,1,0,5,0"));
    }
}
