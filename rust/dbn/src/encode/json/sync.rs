use std::io;

use super::serialize::{to_json_string, to_json_string_with_sym};
use crate::{
    encode::{DbnEncodable, EncodeDbn, EncodeRecord, EncodeRecordRef, EncodeRecordTextExt},
    rtype_method_dispatch, rtype_ts_out_method_dispatch, Error, Metadata, Result,
};

/// Type for encoding files and streams of DBN records in JSON lines.
pub struct Encoder<W>
where
    W: io::Write,
{
    writer: W,
    should_pretty_print: bool,
    use_pretty_px: bool,
    use_pretty_ts: bool,
}

/// Helper for constructing a JSON [`Encoder`].
///
/// No fields are required.
pub struct EncoderBuilder<W>
where
    W: io::Write,
{
    writer: W,
    should_pretty_print: bool,
    use_pretty_px: bool,
    use_pretty_ts: bool,
}

impl<W> EncoderBuilder<W>
where
    W: io::Write,
{
    /// Creates a new JSON encoder builder.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            should_pretty_print: false,
            use_pretty_px: false,
            use_pretty_ts: false,
        }
    }

    /// Sets whether the JSON encoder should encode nicely-formatted JSON objects
    /// with indentation. Defaults to `false` where each JSON object is compact with
    /// no spacing.
    pub fn should_pretty_print(mut self, should_pretty_print: bool) -> Self {
        self.should_pretty_print = should_pretty_print;
        self
    }

    /// Sets whether the JSON encoder will serialize price fields as a decimal. Defaults
    /// to `false`.
    pub fn use_pretty_px(mut self, use_pretty_px: bool) -> Self {
        self.use_pretty_px = use_pretty_px;
        self
    }

    /// Sets whether the JSON encoder will serialize timestamp fields as ISO8601
    /// datetime strings. Defaults to `false`.
    pub fn use_pretty_ts(mut self, use_pretty_ts: bool) -> Self {
        self.use_pretty_ts = use_pretty_ts;
        self
    }

    /// Creates the new encoder with the previously specified settings and if
    /// `write_header` is `true`, encodes the header row.
    pub fn build(self) -> Encoder<W> {
        Encoder::new(
            self.writer,
            self.should_pretty_print,
            self.use_pretty_px,
            self.use_pretty_ts,
        )
    }
}

impl<W> Encoder<W>
where
    W: io::Write,
{
    /// Creates a new instance of [`Encoder`]. If `should_pretty_print` is `true`,
    /// each JSON object will be nicely formatted and indented, instead of the default
    /// compact output with no whitespace between key-value pairs.
    pub fn new(
        writer: W,
        should_pretty_print: bool,
        use_pretty_px: bool,
        use_pretty_ts: bool,
    ) -> Self {
        Self {
            writer,
            should_pretty_print,
            use_pretty_px,
            use_pretty_ts,
        }
    }

    /// Creates a builder for configuring an `Encoder` object.
    pub fn builder(writer: W) -> EncoderBuilder<W> {
        EncoderBuilder::new(writer)
    }

    /// Encodes `metadata` into JSON.
    ///
    /// # Errors
    /// This function returns an error if there's an error writing to `writer`.
    pub fn encode_metadata(&mut self, metadata: &Metadata) -> Result<()> {
        let json = to_json_string(
            metadata,
            self.should_pretty_print,
            self.use_pretty_px,
            self.use_pretty_ts,
        );
        let io_err = |e| Error::io(e, "writing metadata");
        self.writer.write_all(json.as_bytes()).map_err(io_err)?;
        self.writer.flush().map_err(io_err)?;
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
}

impl<W> EncodeRecord for Encoder<W>
where
    W: io::Write,
{
    fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> Result<()> {
        let json = to_json_string(
            record,
            self.should_pretty_print,
            self.use_pretty_px,
            self.use_pretty_ts,
        );
        match self.writer.write_all(json.as_bytes()) {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::io(e, "writing record")),
        }
    }

    fn flush(&mut self) -> Result<()> {
        self.writer
            .flush()
            .map_err(|e| Error::io(e, "flushing output"))
    }
}

impl<W> EncodeRecordRef for Encoder<W>
where
    W: io::Write,
{
    fn encode_record_ref(&mut self, record: crate::RecordRef) -> Result<()> {
        rtype_method_dispatch!(record, self, encode_record)?
    }

    unsafe fn encode_record_ref_ts_out(
        &mut self,
        record: crate::RecordRef,
        ts_out: bool,
    ) -> Result<()> {
        rtype_ts_out_method_dispatch!(record, ts_out, self, encode_record)?
    }
}

impl<W> EncodeDbn for Encoder<W> where W: io::Write {}

impl<W> EncodeRecordTextExt for Encoder<W>
where
    W: io::Write,
{
    fn encode_record_with_sym<R: DbnEncodable>(
        &mut self,
        record: &R,
        symbol: Option<&str>,
    ) -> Result<()> {
        let json = to_json_string_with_sym(
            record,
            self.should_pretty_print,
            self.use_pretty_px,
            self.use_pretty_ts,
            symbol,
        );
        match self.writer.write_all(json.as_bytes()) {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::io(e, "writing record")),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::clone_on_copy)]

    use std::{array, io::BufWriter, num::NonZeroU64, os::raw::c_char};

    use super::*;
    use crate::{
        compat::SYMBOL_CSTR_LEN_V1,
        encode::test_data::{BID_ASK, RECORD_HEADER},
        enums::{
            rtype, InstrumentClass, SType, Schema, SecurityUpdateAction, StatType,
            StatUpdateAction, UserDefinedInstrument,
        },
        record::{
            str_to_c_chars, ErrorMsg, ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg,
            OhlcvMsg, RecordHeader, StatMsg, StatusMsg, TradeMsg, WithTsOut,
        },
        test_utils::VecStream,
        Dataset, MappingInterval, RecordRef, SymbolMapping, FIXED_PRICE_SCALE,
    };

    fn write_json_to_string<R>(
        records: &[R],
        should_pretty_print: bool,
        use_pretty_px: bool,
        use_pretty_ts: bool,
    ) -> String
    where
        R: DbnEncodable,
    {
        let mut buffer = Vec::new();
        Encoder::new(
            &mut buffer,
            should_pretty_print,
            use_pretty_px,
            use_pretty_ts,
        )
        .encode_records(records)
        .unwrap();
        String::from_utf8(buffer).expect("valid UTF-8")
    }

    fn write_json_stream_to_string<R>(
        records: Vec<R>,
        should_pretty_print: bool,
        use_pretty_px: bool,
        use_pretty_ts: bool,
    ) -> String
    where
        R: DbnEncodable,
    {
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer, should_pretty_print, use_pretty_px, use_pretty_ts)
            .encode_stream(VecStream::new(records))
            .unwrap();
        String::from_utf8(buffer).expect("valid UTF-8")
    }

    fn write_json_metadata_to_string(metadata: &Metadata, should_pretty_print: bool) -> String {
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer, should_pretty_print, true, true)
            .encode_metadata(metadata)
            .unwrap();
        String::from_utf8(buffer).expect("valid UTF-8")
    }

    const HEADER_JSON: &str =
        r#""hd":{"ts_event":"1658441851000000000","rtype":4,"publisher_id":1,"instrument_id":323}"#;
    const BID_ASK_JSON: &str = r#"{"bid_px":"372000.000000000","ask_px":"372500.000000000","bid_sz":10,"ask_sz":5,"bid_ct":5,"ask_ct":2}"#;

    #[test]
    fn test_mbo_write_json() {
        let data = vec![MboMsg {
            hd: RECORD_HEADER,
            order_id: 16,
            price: 5500,
            size: 3,
            flags: 128.into(),
            channel_id: 14,
            action: 'R' as c_char,
            side: 'N' as c_char,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
        }];
        let slice_res = write_json_to_string(data.as_slice(), false, true, false);
        let stream_res = write_json_stream_to_string(data, false, true, false);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{},{HEADER_JSON},{}}}\n",
                r#""ts_recv":"1658441891000000000""#,
                r#""action":"R","side":"N","price":"0.000005500","size":3,"channel_id":14,"order_id":"16","flags":128,"ts_in_delta":22000,"sequence":1002375"#
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
            flags: 128.into(),
            depth: 9,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
            levels: [BID_ASK],
        }];
        let slice_res = write_json_to_string(data.as_slice(), false, true, true);
        let stream_res = write_json_stream_to_string(data, false, true, true);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{},{},{},{}}}\n",
                r#""ts_recv":"2022-07-21T22:18:11.000000000Z""#,
                r#""hd":{"ts_event":"2022-07-21T22:17:31.000000000Z","rtype":4,"publisher_id":1,"instrument_id":323}"#,
                r#""action":"B","side":"B","depth":9,"price":"0.000005500","size":3,"flags":128,"ts_in_delta":22000,"sequence":1002375"#,
                format_args!("\"levels\":[{BID_ASK_JSON}]")
            )
        );
    }

    #[test]
    fn test_mbp10_write_json() {
        let data = vec![Mbp10Msg {
            hd: RECORD_HEADER,
            price: 5500,
            size: 3,
            action: 'T' as c_char,
            side: 'N' as c_char,
            flags: 128.into(),
            depth: 9,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
            levels: array::from_fn(|_| BID_ASK.clone()),
        }];
        let slice_res = write_json_to_string(data.as_slice(), false, true, true);
        let stream_res = write_json_stream_to_string(data, false, true, true);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{},{},{},{}}}\n",
                r#""ts_recv":"2022-07-21T22:18:11.000000000Z""#,
                r#""hd":{"ts_event":"2022-07-21T22:17:31.000000000Z","rtype":4,"publisher_id":1,"instrument_id":323}"#,
                r#""action":"T","side":"N","depth":9,"price":"0.000005500","size":3,"flags":128,"ts_in_delta":22000,"sequence":1002375"#,
                format_args!("\"levels\":[{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON}]")
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
            flags: 128.into(),
            depth: 9,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
        }];
        let slice_res = write_json_to_string(data.as_slice(), false, false, false);
        let stream_res = write_json_stream_to_string(data, false, false, false);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{},{HEADER_JSON},{}}}\n",
                r#""ts_recv":"1658441891000000000""#,
                r#""action":"C","side":"B","depth":9,"price":"5500","size":3,"flags":128,"ts_in_delta":22000,"sequence":1002375"#,
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
        let slice_res = write_json_to_string(data.as_slice(), false, true, false);
        let stream_res = write_json_stream_to_string(data, false, true, false);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{HEADER_JSON},{}}}\n",
                r#""open":"0.000005000","high":"0.000008000","low":"0.000003000","close":"0.000006000","volume":"55000""#,
            )
        );
    }

    #[test]
    fn test_status_write_json() {
        let data = vec![StatusMsg {
            hd: RECORD_HEADER,
            ts_recv: 1658441891000000000,
            action: 1,
            reason: 2,
            trading_event: 3,
            is_trading: b'Y' as c_char,
            is_quoting: b'Y' as c_char,
            is_short_sell_restricted: b'~' as c_char,
            _reserved: Default::default(),
        }];
        let slice_res = write_json_to_string(data.as_slice(), false, false, true);
        let stream_res = write_json_stream_to_string(data, false, false, true);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{},{},{}}}\n",
                r#""ts_recv":"2022-07-21T22:18:11.000000000Z""#,
                r#""hd":{"ts_event":"2022-07-21T22:17:31.000000000Z","rtype":4,"publisher_id":1,"instrument_id":323}"#,
                r#""action":1,"reason":2,"trading_event":3,"is_trading":"Y","is_quoting":"Y","is_short_sell_restricted":"~""#,
            )
        );
    }

    #[test]
    fn test_instrument_def_write_json() {
        let data = vec![InstrumentDefMsg {
            hd: RECORD_HEADER,
            ts_recv: 1658441891000000000,
            min_price_increment: 100,
            display_factor: 1_000_000_000,
            expiration: 1698450000000000000,
            activation: 1697350000000000000,
            high_limit_price: 1_000_000,
            low_limit_price: -1_000_000,
            max_price_variation: 0,
            trading_reference_price: 500_000,
            unit_of_measure_qty: 5_000_000_000,
            min_price_increment_amount: 5,
            price_ratio: 10,
            inst_attrib_value: 10,
            underlying_id: 256785,
            raw_instrument_id: RECORD_HEADER.instrument_id,
            market_depth_implied: 0,
            market_depth: 13,
            market_segment_id: 0,
            max_trade_vol: 10_000,
            min_lot_size: 1,
            min_lot_size_block: 1000,
            min_lot_size_round_lot: 100,
            min_trade_vol: 1,
            contract_multiplier: 0,
            decay_quantity: 0,
            original_contract_size: 0,
            trading_reference_date: 0,
            appl_id: 0,
            maturity_year: 0,
            decay_start_date: 0,
            channel_id: 4,
            currency: str_to_c_chars("USD").unwrap(),
            settl_currency: str_to_c_chars("USD").unwrap(),
            secsubtype: Default::default(),
            raw_symbol: str_to_c_chars("ESZ4 C4100").unwrap(),
            group: str_to_c_chars("EW").unwrap(),
            exchange: str_to_c_chars("XCME").unwrap(),
            asset: str_to_c_chars("ES").unwrap(),
            cfi: str_to_c_chars("OCAFPS").unwrap(),
            security_type: str_to_c_chars("OOF").unwrap(),
            unit_of_measure: str_to_c_chars("IPNT").unwrap(),
            underlying: str_to_c_chars("ESZ4").unwrap(),
            strike_price_currency: str_to_c_chars("USD").unwrap(),
            instrument_class: InstrumentClass::Call as c_char,
            strike_price: 4_100_000_000_000,
            match_algorithm: 'F' as c_char,
            md_security_trading_status: 2,
            main_fraction: 4,
            price_display_format: 8,
            settl_price_type: 9,
            sub_fraction: 23,
            underlying_product: 10,
            security_update_action: SecurityUpdateAction::Add as c_char,
            maturity_month: 8,
            maturity_day: 9,
            maturity_week: 11,
            user_defined_instrument: UserDefinedInstrument::No,
            contract_multiplier_unit: 0,
            flow_schedule_type: 5,
            tick_rule: 0,
            _reserved: Default::default(),
        }];
        let slice_res = write_json_to_string(data.as_slice(), false, true, true);
        let stream_res = write_json_stream_to_string(data, false, true, true);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{},{},{}}}\n",
                r#""ts_recv":"2022-07-21T22:18:11.000000000Z""#,
                r#""hd":{"ts_event":"2022-07-21T22:17:31.000000000Z","rtype":4,"publisher_id":1,"instrument_id":323}"#,
                concat!(
                    r#""raw_symbol":"ESZ4 C4100","security_update_action":"A","instrument_class":"C","min_price_increment":"0.000000100","display_factor":"1.000000000","expiration":"2023-10-27T23:40:00.000000000Z","activation":"2023-10-15T06:06:40.000000000Z","#,
                    r#""high_limit_price":"0.001000000","low_limit_price":"-0.001000000","max_price_variation":"0.000000000","trading_reference_price":"0.000500000","unit_of_measure_qty":"5.000000000","#,
                    r#""min_price_increment_amount":"0.000000005","price_ratio":"0.000000010","inst_attrib_value":10,"underlying_id":256785,"raw_instrument_id":323,"market_depth_implied":0,"#,
                    r#""market_depth":13,"market_segment_id":0,"max_trade_vol":10000,"min_lot_size":1,"min_lot_size_block":1000,"min_lot_size_round_lot":100,"min_trade_vol":1,"#,
                    r#""contract_multiplier":0,"decay_quantity":0,"original_contract_size":0,"trading_reference_date":0,"appl_id":0,"#,
                    r#""maturity_year":0,"decay_start_date":0,"channel_id":4,"currency":"USD","settl_currency":"USD","secsubtype":"","group":"EW","exchange":"XCME","asset":"ES","cfi":"OCAFPS","#,
                    r#""security_type":"OOF","unit_of_measure":"IPNT","underlying":"ESZ4","strike_price_currency":"USD","strike_price":"4100.000000000","match_algorithm":"F","md_security_trading_status":2,"main_fraction":4,"price_display_format":8,"#,
                    r#""settl_price_type":9,"sub_fraction":23,"underlying_product":10,"maturity_month":8,"maturity_day":9,"maturity_week":11,"#,
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
            _reserved: [0],
        }];
        let slice_res = write_json_to_string(data.as_slice(), false, false, false);
        let stream_res = write_json_stream_to_string(data, false, false, false);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{},{HEADER_JSON},{}}}\n",
                r#""ts_recv":"1""#,
                concat!(
                    r#""ref_price":"2","auction_time":"3","cont_book_clr_price":"4","auct_interest_clr_price":"5","#,
                    r#""ssr_filling_price":"6","ind_match_price":"7","upper_collar":"8","lower_collar":"9","paired_qty":10,"#,
                    r#""total_imbalance_qty":11,"market_imbalance_qty":12,"unpaired_qty":13,"auction_type":"B","side":"A","#,
                    r#""auction_status":14,"freeze_status":15,"num_extensions":16,"unpaired_side":"A","significant_imbalance":"N""#,
                )
            )
        );
    }

    #[test]
    fn test_stat_write_json() {
        let data = vec![StatMsg {
            hd: RECORD_HEADER,
            ts_recv: 1,
            ts_ref: 2,
            price: 3,
            quantity: 0,
            sequence: 4,
            ts_in_delta: 5,
            stat_type: StatType::OpeningPrice as u16,
            channel_id: 7,
            update_action: StatUpdateAction::New as u8,
            stat_flags: 0,
            _reserved: Default::default(),
        }];
        let slice_res = write_json_to_string(data.as_slice(), false, true, false);
        let stream_res = write_json_stream_to_string(data, false, true, false);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{},{HEADER_JSON},{}}}\n",
                r#""ts_recv":"1""#,
                concat!(
                    r#""ts_ref":"2","price":"0.000000003","quantity":0,"sequence":4,"#,
                    r#""ts_in_delta":5,"stat_type":1,"channel_id":7,"update_action":1,"stat_flags":0"#,
                )
            )
        );
    }

    #[test]
    fn test_metadata_write_json() {
        let metadata = Metadata {
            version: 1,
            dataset: Dataset::GlbxMdp3.to_string(),
            schema: Some(Schema::Ohlcv1H),
            start: 1662734705128748281,
            end: NonZeroU64::new(1662734720914876944),
            limit: None,
            stype_in: Some(SType::InstrumentId),
            stype_out: SType::RawSymbol,
            ts_out: false,
            symbol_cstr_len: SYMBOL_CSTR_LEN_V1,
            symbols: vec!["ESZ2".to_owned()],
            partial: Vec::new(),
            not_found: Vec::new(),
            mappings: vec![SymbolMapping {
                raw_symbol: "ESZ2".to_owned(),
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
            :\"2022-09-09T14:45:05.128748281Z\",\"end\":\"2022-09-09T14:45:20.914876944Z\",\"limit\":null,\
            \"stype_in\":\"instrument_id\",\"stype_out\":\"raw_symbol\",\"ts_out\":false,\"symbol_cstr_len\":22,\"symbols\"\
            :[\"ESZ2\"],\"partial\":[],\"not_found\":[],\"mappings\":[{\"raw_symbol\":\"ESZ2\",\
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
        let res = write_json_to_string(records.as_slice(), false, false, true);
        assert_eq!(
            res,
            format!(
                "{{{},{}}}\n",
                r#""hd":{"ts_event":"2022-07-21T22:17:31.000000000Z","rtype":4,"publisher_id":1,"instrument_id":323}"#,
                r#""open":"5000","high":"8000","low":"3000","close":"6000","volume":"55000","ts_out":"2023-03-10T20:57:49.000000000Z""#,
            )
        );
    }

    #[test]
    fn test_serialize_quoted_str_to_json() {
        let json = write_json_to_string(
            vec![ErrorMsg::new(0, "\"A test", true)].as_slice(),
            false,
            true,
            true,
        );
        assert_eq!(
            json,
            r#"{"hd":{"ts_event":null,"rtype":21,"publisher_id":0,"instrument_id":0},"err":"\"A test","code":255,"is_last":1}
"#
        );
    }

    #[test]
    fn test_encode_ref_with_sym() {
        let mut buffer = Vec::new();
        const BAR: OhlcvMsg = OhlcvMsg {
            hd: RecordHeader::new::<OhlcvMsg>(rtype::OHLCV_1H, 10, 9, 0),
            open: 175 * FIXED_PRICE_SCALE,
            high: 177 * FIXED_PRICE_SCALE,
            low: 174 * FIXED_PRICE_SCALE,
            close: 175 * FIXED_PRICE_SCALE,
            volume: 4033445,
        };
        let rec_ref = RecordRef::from(&BAR);
        let mut encoder = Encoder::new(&mut buffer, false, false, false);
        encoder.encode_ref_with_sym(rec_ref, None).unwrap();
        encoder.encode_ref_with_sym(rec_ref, Some("AAPL")).unwrap();
        let res = String::from_utf8(buffer).unwrap();
        assert_eq!(
            res,
            "{\"hd\":{\"ts_event\":\"0\",\"rtype\":34,\"publisher_id\":10,\"instrument_id\":9},\"open\":\"175000000000\",\"high\":\"177000000000\",\"low\":\"174000000000\",\"close\":\"175000000000\",\"volume\":\"4033445\",\"symbol\":null}\n\
            {\"hd\":{\"ts_event\":\"0\",\"rtype\":34,\"publisher_id\":10,\"instrument_id\":9},\"open\":\"175000000000\",\"high\":\"177000000000\",\"low\":\"174000000000\",\"close\":\"175000000000\",\"volume\":\"4033445\",\"symbol\":\"AAPL\"}\n",
        );
    }
}
