//! Encoding of DBN records into newline-delimited JSON (ndjson).
use std::{fmt, io};

use anyhow::Context;
use serde::Serialize;
use serde_json::ser::PrettyFormatter;
use streaming_iterator::StreamingIterator;

use crate::Metadata;

use super::{DbnEncodable, EncodeDbn};

/// Type for encoding files and streams of DBN records in newline-delimited JSON (ndjson).
pub struct Encoder<W>
where
    W: io::Write,
{
    writer: W,
    should_pretty_print: bool,
}

impl<W> Encoder<W>
where
    W: io::Write,
{
    /// Creates a new instance of [`Encoder`]. If `should_pretty_print` is `true`,
    /// each JSON object will be nicely formatted and indented, instead of the default
    /// compact output with no whitespace between key-value pairs.
    pub fn new(writer: W, should_pretty_print: bool) -> Self {
        Self {
            writer,
            should_pretty_print,
        }
    }

    /// Encodes `metadata` into JSON.
    ///
    /// # Errors
    /// This function returns an error if `metadata` is not serializable or if there's
    /// an error writing to `writer`.
    pub fn encode_metadata(&mut self, metadata: &Metadata) -> anyhow::Result<()> {
        self.serialize(metadata)
            .with_context(|| format!("Failed to serialize {metadata:#?}"))?;
        // newline at EOF
        self.writer.write_all(b"\n")?;
        self.writer.flush()?;
        Ok(())
    }

    /// Returns a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        &self.writer
    }

    /// Returns a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    fn serialize<T: fmt::Debug + Serialize>(&mut self, obj: &T) -> serde_json::Result<()> {
        if self.should_pretty_print {
            obj.serialize(&mut serde_json::Serializer::with_formatter(
                &mut self.writer,
                Self::pretty_formatter(),
            ))
        } else {
            obj.serialize(&mut serde_json::Serializer::new(&mut self.writer))
        }
    }

    fn pretty_formatter() -> PrettyFormatter<'static> {
        // `PrettyFormatter::with_indent` should be `const`
        PrettyFormatter::with_indent(b"    ")
    }
}

impl<W> EncodeDbn for Encoder<W>
where
    W: io::Write,
{
    fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> anyhow::Result<bool> {
        match self.serialize(record) {
            // broken output, likely a closed pipe
            Ok(_) => Ok(()),
            Err(e) if e.is_io() => return Ok(true),
            Err(e) => {
                Err(anyhow::Error::new(e).context(format!("Failed to serialize {record:#?}")))
            }
        }?;
        match self.writer.write_all(b"\n") {
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => return Ok(true),
            r => r,
        }?;
        Ok(false)
    }

    fn encode_records<R: DbnEncodable>(&mut self, records: &[R]) -> anyhow::Result<()> {
        for record in records {
            if self.encode_record(record)? {
                return Ok(());
            }
        }
        self.writer.flush()?;
        Ok(())
    }

    fn encode_stream<R: DbnEncodable>(
        &mut self,
        mut stream: impl StreamingIterator<Item = R>,
    ) -> anyhow::Result<()> {
        while let Some(record) = stream.next() {
            if self.encode_record(record)? {
                return Ok(());
            }
        }
        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{array, io::BufWriter, num::NonZeroU64, os::raw::c_char};

    use super::*;
    use crate::{
        encode::test_data::{VecStream, BID_ASK, RECORD_HEADER},
        enums::{SType, Schema, SecurityUpdateAction},
        record::{
            str_to_c_chars, ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg,
            StatusMsg, TradeMsg, WithTsOut,
        },
        MappingInterval, SymbolMapping,
    };

    fn write_json_to_string<R>(records: &[R], should_pretty_print: bool) -> String
    where
        R: DbnEncodable,
    {
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer, should_pretty_print)
            .encode_records(records)
            .unwrap();
        String::from_utf8(buffer).expect("valid UTF-8")
    }

    fn write_json_stream_to_string<R>(records: Vec<R>, should_pretty_print: bool) -> String
    where
        R: DbnEncodable,
    {
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer, should_pretty_print)
            .encode_stream(VecStream::new(records))
            .unwrap();
        String::from_utf8(buffer).expect("valid UTF-8")
    }

    fn write_json_metadata_to_string(metadata: &Metadata, should_pretty_print: bool) -> String {
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer, should_pretty_print)
            .encode_metadata(metadata)
            .unwrap();
        String::from_utf8(buffer).expect("valid UTF-8")
    }

    const HEADER_JSON: &str =
        r#""hd":{"rtype":4,"publisher_id":1,"product_id":323,"ts_event":"1658441851000000000"}"#;
    const BID_ASK_JSON: &str = r#"{"bid_px":372000000000000,"ask_px":372500000000000,"bid_sz":10,"ask_sz":5,"bid_ct":5,"ask_ct":2}"#;

    #[test]
    fn test_mbo_write_json() {
        let data = vec![MboMsg {
            hd: RECORD_HEADER,
            order_id: 16,
            price: 5500,
            size: 3,
            flags: 128,
            channel_id: 14,
            action: 'R' as c_char,
            side: 'N' as c_char,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
        }];
        let slice_res = write_json_to_string(data.as_slice(), false);
        let stream_res = write_json_stream_to_string(data, false);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{HEADER_JSON},{}}}\n",
                r#""order_id":"16","price":5500,"size":3,"flags":128,"channel_id":14,"action":"R","side":"N","ts_recv":"1658441891000000000","ts_in_delta":22000,"sequence":1002375"#
            )
        );
    }

    #[test]
    fn test_mbp1_write_json() {
        let data = vec![Mbp1Msg {
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
            booklevel: [BID_ASK],
        }];
        let slice_res = write_json_to_string(data.as_slice(), false);
        let stream_res = write_json_stream_to_string(data, false);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{HEADER_JSON},{},{}}}\n",
                r#""price":5500,"size":3,"action":"B","side":"B","flags":128,"depth":9,"ts_recv":"1658441891000000000","ts_in_delta":22000,"sequence":1002375"#,
                format_args!("\"booklevel\":[{BID_ASK_JSON}]")
            )
        );
    }

    #[test]
    fn test_mbo10_write_json() {
        let data = vec![Mbp10Msg {
            hd: RECORD_HEADER,
            price: 5500,
            size: 3,
            action: 'T' as c_char,
            side: 'N' as c_char,
            flags: 128,
            depth: 9,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
            booklevel: array::from_fn(|_| BID_ASK.clone()),
        }];
        let slice_res = write_json_to_string(data.as_slice(), false);
        let stream_res = write_json_stream_to_string(data, false);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{HEADER_JSON},{},{}}}\n",
                r#""price":5500,"size":3,"action":"T","side":"N","flags":128,"depth":9,"ts_recv":"1658441891000000000","ts_in_delta":22000,"sequence":1002375"#,
                format_args!("\"booklevel\":[{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON}]")
            )
        );
    }

    #[test]
    fn test_trade_write_json() {
        let data = vec![TradeMsg {
            hd: RECORD_HEADER,
            price: 5500,
            size: 3,
            action: 'C' as c_char,
            side: 'B' as c_char,
            flags: 128,
            depth: 9,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
            booklevel: [],
        }];
        let slice_res = write_json_to_string(data.as_slice(), false);
        let stream_res = write_json_stream_to_string(data, false);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{HEADER_JSON},{}}}\n",
                r#""price":5500,"size":3,"action":"C","side":"B","flags":128,"depth":9,"ts_recv":"1658441891000000000","ts_in_delta":22000,"sequence":1002375"#,
            )
        );
    }

    #[test]
    fn test_ohlcv_write_json() {
        let data = vec![OhlcvMsg {
            hd: RECORD_HEADER,
            open: 5000,
            high: 8000,
            low: 3000,
            close: 6000,
            volume: 55_000,
        }];
        let slice_res = write_json_to_string(data.as_slice(), false);
        let stream_res = write_json_stream_to_string(data, false);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{HEADER_JSON},{}}}\n",
                r#""open":5000,"high":8000,"low":3000,"close":6000,"volume":55000"#,
            )
        );
    }

    #[test]
    fn test_status_write_json() {
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
        let slice_res = write_json_to_string(data.as_slice(), false);
        let stream_res = write_json_stream_to_string(data, false);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{HEADER_JSON},{}}}\n",
                r#""ts_recv":"1658441891000000000","group":"group","trading_status":3,"halt_reason":4,"trading_event":6"#,
            )
        );
    }

    #[test]
    fn test_instrument_def_write_json() {
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
        let slice_res = write_json_to_string(data.as_slice(), false);
        let stream_res = write_json_stream_to_string(data, false);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{HEADER_JSON},{}}}\n",
                concat!(
                    r#""ts_recv":"1658441891000000000","min_price_increment":100,"display_factor":1000,"expiration":"1698450000000000000","activation":"1697350000000000000","#,
                    r#""high_limit_price":1000000,"low_limit_price":-1000000,"max_price_variation":0,"trading_reference_price":500000,"unit_of_measure_qty":5,"#,
                    r#""min_price_increment_amount":5,"price_ratio":10,"inst_attrib_value":10,"underlying_id":256785,"cleared_volume":0,"market_depth_implied":0,"#,
                    r#""market_depth":13,"market_segment_id":0,"max_trade_vol":10000,"min_lot_size":1,"min_lot_size_block":1000,"min_lot_size_round_lot":100,"min_trade_vol":1,"#,
                    r#""open_interest_qty":0,"contract_multiplier":0,"decay_quantity":0,"original_contract_size":0,"trading_reference_date":0,"appl_id":0,"#,
                    r#""maturity_year":0,"decay_start_date":0,"channel_id":4,"currency":"","settl_currency":"USD","secsubtype":"","symbol":"","group":"","exchange":"","asset":"","cfi":"","#,
                    r#""security_type":"","unit_of_measure":"","underlying":"","match_algorithm":"F","md_security_trading_status":2,"main_fraction":4,"price_display_format":8,"#,
                    r#""settl_price_type":9,"sub_fraction":23,"underlying_product":10,"security_update_action":"A","maturity_month":8,"maturity_day":9,"maturity_week":11,"#,
                    r#""user_defined_instrument":"N","contract_multiplier_unit":0,"flow_schedule_type":5,"tick_rule":0"#
                )
            )
        );
    }

    #[test]
    fn test_imbalance_write_json() {
        let data = vec![ImbalanceMsg {
            hd: RECORD_HEADER,
            ts_recv: 1,
            ref_price: 2,
            auction_time: 3,
            cont_book_clr_price: 4,
            auct_interest_clr_price: 5,
            ssr_filling_price: 6,
            ind_match_price: 7,
            upper_collar: 8,
            lower_collar: 9,
            paired_qty: 10,
            total_imbalance_qty: 11,
            market_imbalance_qty: 12,
            unpaired_qty: 13,
            auction_type: 'B' as c_char,
            side: 'A' as c_char,
            auction_status: 14,
            freeze_status: 15,
            num_extensions: 16,
            unpaired_side: 'A' as c_char,
            significant_imbalance: 'N' as c_char,
            _dummy: [0],
        }];
        let slice_res = write_json_to_string(data.as_slice(), false);
        let stream_res = write_json_stream_to_string(data, false);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{HEADER_JSON},{}}}\n",
                concat!(
                    r#""ts_recv":"1","ref_price":2,"auction_time":"3","cont_book_clr_price":4,"auct_interest_clr_price":5,"#,
                    r#""ssr_filling_price":6,"ind_match_price":7,"upper_collar":8,"lower_collar":9,"paired_qty":10,"#,
                    r#""total_imbalance_qty":11,"market_imbalance_qty":12,"unpaired_qty":13,"auction_type":"B","side":"A","#,
                    r#""auction_status":14,"freeze_status":15,"num_extensions":16,"unpaired_side":"A","significant_imbalance":"N""#,
                )
            )
        );
    }

    #[test]
    fn test_metadata_write_json() {
        let metadata = Metadata {
            version: 1,
            dataset: "GLBX.MDP3".to_owned(),
            schema: Schema::Ohlcv1H,
            start: 1662734705128748281,
            end: NonZeroU64::new(1662734720914876944),
            limit: None,
            stype_in: SType::ProductId,
            stype_out: SType::Native,
            ts_out: false,
            symbols: vec!["ESZ2".to_owned()],
            partial: Vec::new(),
            not_found: Vec::new(),
            mappings: vec![SymbolMapping {
                native_symbol: "ESZ2".to_owned(),
                intervals: vec![MappingInterval {
                    start_date: time::Date::from_calendar_date(2022, time::Month::September, 9)
                        .unwrap(),
                    end_date: time::Date::from_calendar_date(2022, time::Month::September, 10)
                        .unwrap(),
                    symbol: "ESH2".to_owned(),
                }],
            }],
        };
        let res = write_json_metadata_to_string(&metadata, false);
        assert_eq!(
            res,
            "{\"version\":1,\"dataset\":\"GLBX.MDP3\",\"schema\":\"ohlcv-1h\",\"start\"\
            :1662734705128748281,\"end\":1662734720914876944,\"limit\":0,\
            \"stype_in\":\"product_id\",\"stype_out\":\"native\",\"ts_out\":false,\"symbols\"\
            :[\"ESZ2\"],\"partial\":[],\"not_found\":[],\"mappings\":[{\"native_symbol\":\"ESZ2\",\
            \"intervals\":[{\"start_date\":\"2022-09-09\",\"end_date\":\"2022-09-10\",\"symbol\":\
            \"ESH2\"}]}]}\n"
        );
    }

    #[test]
    fn test_encode_with_ts_out() {
        let records = vec![WithTsOut {
            rec: OhlcvMsg {
                hd: RECORD_HEADER,
                open: 5000,
                high: 8000,
                low: 3000,
                close: 6000,
                volume: 55_000,
            },
            ts_out: 1678481869000000000,
        }];
        let res = write_json_to_string(records.as_slice(), false);
        assert_eq!(
            res,
            format!(
                "{{{HEADER_JSON},{}}}\n",
                r#""open":5000,"high":8000,"low":3000,"close":6000,"volume":55000,"ts_out":"1678481869000000000""#,
            )
        );
    }
}
