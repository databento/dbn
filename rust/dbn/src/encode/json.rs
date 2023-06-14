//! Encoding of DBN records into newline-delimited JSON (ndjson).
use std::io;

use streaming_iterator::StreamingIterator;

use self::serialize::to_json_string;
use super::{DbnEncodable, EncodeDbn};
use crate::Metadata;

/// Type for encoding files and streams of DBN records in newline-delimited JSON (ndjson).
pub struct Encoder<W>
where
    W: io::Write,
{
    writer: W,
    should_pretty_print: bool,
    use_pretty_px: bool,
    use_pretty_ts: bool,
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

    /// Encodes `metadata` into JSON.
    ///
    /// # Errors
    /// This function returns an error if there's an error writing to `writer`.
    pub fn encode_metadata(&mut self, metadata: &Metadata) -> anyhow::Result<()> {
        let json = to_json_string(
            metadata,
            self.should_pretty_print,
            self.use_pretty_px,
            self.use_pretty_ts,
        );
        self.writer.write_all(json.as_bytes())?;
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
}

impl<W> EncodeDbn for Encoder<W>
where
    W: io::Write,
{
    fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> anyhow::Result<bool> {
        let json = to_json_string(
            record,
            self.should_pretty_print,
            self.use_pretty_px,
            self.use_pretty_ts,
        );
        match self.writer.write_all(json.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) if matches!(e.kind(), io::ErrorKind::BrokenPipe) => return Ok(true),
            Err(e) => {
                Err(anyhow::Error::new(e).context(format!("Failed to serialize {record:#?}")))
            }
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

    fn flush(&mut self) -> anyhow::Result<()> {
        Ok(self.writer.flush()?)
    }
}

pub(crate) mod serialize {
    use std::ffi::c_char;

    use time::format_description::FormatItem;

    use crate::json_writer::{JsonObjectWriter, NULL};
    use crate::UNDEF_TIMESTAMP;
    use crate::{
        encode::{format_px, format_ts},
        enums::{SecurityUpdateAction, UserDefinedInstrument},
        record::{c_chars_to_str, BidAskPair, HasRType, RecordHeader, WithTsOut},
        Metadata, UNDEF_PRICE,
    };

    /// Serializes `obj` to a JSON string.
    pub fn to_json_string<T: JsonSerialize>(
        obj: &T,
        should_pretty_print: bool,
        use_pretty_px: bool,
        use_pretty_ts: bool,
    ) -> String {
        let mut res = String::new();
        if should_pretty_print {
            let mut pretty = pretty_writer(&mut res);
            let mut writer = JsonObjectWriter::new(&mut pretty);
            to_json_with_writer(obj, &mut writer, use_pretty_px, use_pretty_ts);
        } else {
            let mut writer = JsonObjectWriter::new(&mut res);
            to_json_with_writer(obj, &mut writer, use_pretty_px, use_pretty_ts);
        }
        res.push('\n');
        res
    }

    fn to_json_with_writer<T: JsonSerialize, J: crate::json_writer::JsonWriter>(
        obj: &T,
        writer: &mut JsonObjectWriter<J>,
        use_pretty_px: bool,
        use_pretty_ts: bool,
    ) {
        match (use_pretty_px, use_pretty_ts) {
            (true, true) => obj.to_json::<J, true, true>(writer),
            (true, false) => obj.to_json::<J, true, false>(writer),
            (false, true) => obj.to_json::<J, false, true>(writer),
            (false, false) => obj.to_json::<J, false, false>(writer),
        };
    }

    fn pretty_writer(buffer: &mut String) -> crate::json_writer::PrettyJsonWriter<'_> {
        crate::json_writer::PrettyJsonWriter::with_indent(buffer, "    ")
    }

    pub trait JsonSerialize {
        fn to_json<
            J: crate::json_writer::JsonWriter,
            const PRETTY_PX: bool,
            const PRETTY_TS: bool,
        >(
            &self,
            writer: &mut JsonObjectWriter<J>,
        );
    }

    impl<T: HasRType + JsonSerialize> JsonSerialize for WithTsOut<T> {
        fn to_json<
            J: crate::json_writer::JsonWriter,
            const PRETTY_PX: bool,
            const PRETTY_TS: bool,
        >(
            &self,
            writer: &mut JsonObjectWriter<J>,
        ) {
            self.rec.to_json::<J, PRETTY_PX, PRETTY_TS>(writer);
            write_ts_field::<J, PRETTY_TS>(writer, "ts_out", self.ts_out);
        }
    }

    impl JsonSerialize for Metadata {
        fn to_json<
            J: crate::json_writer::JsonWriter,
            const _PRETTY_PX: bool,
            const PRETTY_TS: bool,
        >(
            &self,
            writer: &mut JsonObjectWriter<J>,
        ) {
            writer.value("version", self.version);
            writer.value("dataset", &self.dataset);
            writer.value("schema", self.schema.map(|s| s.as_str()));
            write_ts_field::<J, PRETTY_TS>(writer, "start", self.start);
            if let Some(end) = self.end {
                write_ts_field::<J, PRETTY_TS>(writer, "end", end.get());
            } else {
                writer.value("end", NULL);
            }
            writer.value("limit", self.limit.map(|l| l.to_string()).as_ref());
            writer.value("stype_in", self.stype_in.map(|s| s.as_str()));
            writer.value("stype_out", self.stype_out.as_str());
            writer.value("ts_out", self.ts_out);
            for (key, sym_list) in [
                ("symbols", &self.symbols),
                ("partial", &self.partial),
                ("not_found", &self.not_found),
            ] {
                let mut writer = writer.array(key);
                for symbol in sym_list {
                    writer.value(symbol);
                }
            }
            let mut mappings_writer = writer.array("mappings");
            for mapping in self.mappings.iter() {
                let mut item_writer = mappings_writer.object();
                item_writer.value("raw_symbol", &mapping.raw_symbol);
                let mut interval_arr_writer = item_writer.array("intervals");
                for interval in mapping.intervals.iter() {
                    let mut interval_writer = interval_arr_writer.object();
                    write_date_field::<J, PRETTY_TS>(
                        &mut interval_writer,
                        "start_date",
                        &interval.start_date,
                    );
                    write_date_field::<J, PRETTY_TS>(
                        &mut interval_writer,
                        "end_date",
                        &interval.end_date,
                    );
                    interval_writer.value("symbol", &interval.symbol);
                }
            }
        }
    }

    pub trait WriteField {
        fn write_field<
            J: crate::json_writer::JsonWriter,
            const PRETTY_PX: bool,
            const PRETTY_TS: bool,
        >(
            &self,
            writer: &mut JsonObjectWriter<J>,
            name: &str,
        );
    }

    impl WriteField for RecordHeader {
        fn write_field<
            J: crate::json_writer::JsonWriter,
            const PRETTY_PX: bool,
            const PRETTY_TS: bool,
        >(
            &self,
            writer: &mut JsonObjectWriter<J>,
            name: &str,
        ) {
            let mut hd_writer = writer.object(name);
            hd_writer.value("rtype", self.rtype);
            hd_writer.value("publisher_id", self.publisher_id);
            hd_writer.value("instrument_id", self.instrument_id);
            write_ts_field::<J, PRETTY_TS>(&mut hd_writer, "ts_event", self.ts_event);
        }
    }

    impl<const N: usize> WriteField for [BidAskPair; N] {
        fn write_field<
            J: crate::json_writer::JsonWriter,
            const PRETTY_PX: bool,
            const PRETTY_TS: bool,
        >(
            &self,
            writer: &mut JsonObjectWriter<J>,
            name: &str,
        ) {
            let mut arr_writer = writer.array(name);
            for level in self.iter() {
                let mut item_writer = arr_writer.object();
                write_px_field::<J, PRETTY_PX>(&mut item_writer, "bid_px", level.bid_px);
                write_px_field::<J, PRETTY_PX>(&mut item_writer, "ask_px", level.ask_px);
                item_writer.value("bid_sz", level.bid_sz);
                item_writer.value("ask_sz", level.ask_sz);
                item_writer.value("bid_ct", level.bid_ct);
                item_writer.value("ask_ct", level.ask_ct);
            }
        }
    }

    impl WriteField for i64 {
        fn write_field<
            J: crate::json_writer::JsonWriter,
            const PRETTY_PX: bool,
            const PRETTY_TS: bool,
        >(
            &self,
            writer: &mut JsonObjectWriter<J>,
            name: &str,
        ) {
            writer.value(name, &self.to_string());
        }
    }

    impl WriteField for u64 {
        fn write_field<
            J: crate::json_writer::JsonWriter,
            const PRETTY_PX: bool,
            const PRETTY_TS: bool,
        >(
            &self,
            writer: &mut JsonObjectWriter<J>,
            name: &str,
        ) {
            writer.value(name, &self.to_string());
        }
    }

    macro_rules! impl_write_field_for {
        ($($ty:ident),+) => {
            $(
                impl WriteField for $ty {
                    fn write_field<J: crate::json_writer::JsonWriter, const PRETTY_PX: bool, const PRETTY_TS: bool>(
                        &self,
                        writer: &mut JsonObjectWriter<J>,
                        name: &str,
                    ) {
                        writer.value(name, self);
                    }
                }
            )*
        };
    }

    impl_write_field_for! {i32, u32, i16, u16, i8, u8, bool}

    impl WriteField for SecurityUpdateAction {
        fn write_field<
            J: crate::json_writer::JsonWriter,
            const _PRETTY_PX: bool,
            const _PRETTY_TS: bool,
        >(
            &self,
            writer: &mut JsonObjectWriter<J>,
            name: &str,
        ) {
            writer.value(name, &(*self as u8 as char).to_string());
        }
    }

    impl WriteField for UserDefinedInstrument {
        fn write_field<
            J: crate::json_writer::JsonWriter,
            const _PRETTY_PX: bool,
            const _PRETTY_TS: bool,
        >(
            &self,
            writer: &mut JsonObjectWriter<J>,
            name: &str,
        ) {
            writer.value(name, &(*self as u8 as char).to_string());
        }
    }

    impl<const N: usize> WriteField for [c_char; N] {
        fn write_field<
            J: crate::json_writer::JsonWriter,
            const PRETTY_PX: bool,
            const PRETTY_TS: bool,
        >(
            &self,
            writer: &mut JsonObjectWriter<J>,
            name: &str,
        ) {
            writer.value(name, c_chars_to_str(self).unwrap_or_default());
        }
    }

    pub fn write_c_char_field<J: crate::json_writer::JsonWriter>(
        writer: &mut JsonObjectWriter<J>,
        name: &str,
        c_char: c_char,
    ) {
        writer.value(name, &(c_char as u8 as char).to_string());
    }

    pub fn write_px_field<J: crate::json_writer::JsonWriter, const PRETTY_PX: bool>(
        writer: &mut JsonObjectWriter<J>,
        key: &str,
        px: i64,
    ) {
        if PRETTY_PX {
            if px == UNDEF_PRICE {
                writer.value(key, NULL);
            } else {
                writer.value(key, &format_px(px));
            }
        } else {
            // Convert to string to avoid a loss of precision
            writer.value(key, &px.to_string())
        }
    }

    pub fn write_ts_field<J: crate::json_writer::JsonWriter, const PRETTY_TS: bool>(
        writer: &mut JsonObjectWriter<J>,
        key: &str,
        ts: u64,
    ) {
        if PRETTY_TS {
            match ts {
                0 | UNDEF_TIMESTAMP => writer.value(key, NULL),
                ts => writer.value(key, &format_ts(ts)),
            };
        } else {
            // Convert to string to avoid a loss of precision
            writer.value(key, &ts.to_string());
        }
    }

    fn write_date_field<J: crate::json_writer::JsonWriter, const PRETTY_TS: bool>(
        writer: &mut JsonObjectWriter<J>,
        key: &str,
        date: &time::Date,
    ) {
        const DATE_FORMAT: &[FormatItem<'static>] =
            time::macros::format_description!("[year]-[month]-[day]");
        if PRETTY_TS {
            writer.value(key, &date.format(DATE_FORMAT).unwrap_or_default());
        } else {
            let mut date_int = date.year() as u32 * 10_000;
            date_int += date.month() as u32 * 100;
            date_int += date.day() as u32;
            writer.value(key, date_int);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{array, io::BufWriter, num::NonZeroU64, os::raw::c_char};

    use super::*;
    use crate::{
        datasets::GLBX_MDP3,
        encode::test_data::{VecStream, BID_ASK, RECORD_HEADER},
        enums::{
            InstrumentClass, SType, Schema, SecurityUpdateAction, StatType, StatUpdateAction,
            UserDefinedInstrument,
        },
        record::{
            str_to_c_chars, ErrorMsg, ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg,
            OhlcvMsg, StatMsg, StatusMsg, TradeMsg, WithTsOut,
        },
        MappingInterval, SymbolMapping,
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
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer, should_pretty_print, use_pretty_px, use_pretty_ts)
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

    pub const HEADER_JSON: &str =
        r#""hd":{"rtype":4,"publisher_id":1,"instrument_id":323,"ts_event":"1658441851000000000"}"#;
    const BID_ASK_JSON: &str = r#"{"bid_px":"372000.000000000","ask_px":"372500.000000000","bid_sz":10,"ask_sz":5,"bid_ct":5,"ask_ct":2}"#;

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
        let slice_res = write_json_to_string(data.as_slice(), false, true, false);
        let stream_res = write_json_stream_to_string(data, false, true, false);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{HEADER_JSON},{}}}\n",
                r#""order_id":"16","price":"0.000005500","size":3,"flags":128,"channel_id":14,"action":"R","side":"N","ts_recv":"1658441891000000000","ts_in_delta":22000,"sequence":1002375"#
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
            levels: [BID_ASK],
        }];
        let slice_res = write_json_to_string(data.as_slice(), false, true, true);
        let stream_res = write_json_stream_to_string(data, false, true, true);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{},{},{}}}\n",
                r#""hd":{"rtype":4,"publisher_id":1,"instrument_id":323,"ts_event":"2022-07-21T22:17:31.000000000Z"}"#,
                r#""price":"0.000005500","size":3,"action":"B","side":"B","flags":128,"depth":9,"ts_recv":"2022-07-21T22:18:11.000000000Z","ts_in_delta":22000,"sequence":1002375"#,
                format_args!("\"levels\":[{BID_ASK_JSON}]")
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
            levels: array::from_fn(|_| BID_ASK.clone()),
        }];
        let slice_res = write_json_to_string(data.as_slice(), false, true, true);
        let stream_res = write_json_stream_to_string(data, false, true, true);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{},{},{}}}\n",
                r#""hd":{"rtype":4,"publisher_id":1,"instrument_id":323,"ts_event":"2022-07-21T22:17:31.000000000Z"}"#,
                r#""price":"0.000005500","size":3,"action":"T","side":"N","flags":128,"depth":9,"ts_recv":"2022-07-21T22:18:11.000000000Z","ts_in_delta":22000,"sequence":1002375"#,
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
            flags: 128,
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
                "{{{HEADER_JSON},{}}}\n",
                r#""price":"5500","size":3,"action":"C","side":"B","flags":128,"depth":9,"ts_recv":"1658441891000000000","ts_in_delta":22000,"sequence":1002375"#,
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
            group: str_to_c_chars("group").unwrap(),
            trading_status: 3,
            halt_reason: 4,
            trading_event: 6,
        }];
        let slice_res = write_json_to_string(data.as_slice(), false, false, true);
        let stream_res = write_json_stream_to_string(data, false, false, true);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{},{}}}\n",
                r#""hd":{"rtype":4,"publisher_id":1,"instrument_id":323,"ts_event":"2022-07-21T22:17:31.000000000Z"}"#,
                r#""ts_recv":"2022-07-21T22:18:11.000000000Z","group":"group","trading_status":3,"halt_reason":4,"trading_event":6"#,
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
            instrument_class: InstrumentClass::Call as u8 as c_char,
            strike_price: 4_100_000_000_000,
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
            user_defined_instrument: UserDefinedInstrument::No,
            contract_multiplier_unit: 0,
            flow_schedule_type: 5,
            tick_rule: 0,
            _reserved1: Default::default(),
            _reserved2: Default::default(),
            _reserved3: Default::default(),
            _reserved4: Default::default(),
            _reserved5: Default::default(),
            _dummy: [0; 3],
        }];
        let slice_res = write_json_to_string(data.as_slice(), false, true, true);
        let stream_res = write_json_stream_to_string(data, false, true, true);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{},{}}}\n",
                r#""hd":{"rtype":4,"publisher_id":1,"instrument_id":323,"ts_event":"2022-07-21T22:17:31.000000000Z"}"#,
                concat!(
                    r#""ts_recv":"2022-07-21T22:18:11.000000000Z","min_price_increment":"0.000000100","display_factor":"1000","expiration":"2023-10-27T23:40:00.000000000Z","activation":"2023-10-15T06:06:40.000000000Z","#,
                    r#""high_limit_price":"0.001000000","low_limit_price":"-0.001000000","max_price_variation":"0.000000000","trading_reference_price":"0.000500000","unit_of_measure_qty":"5","#,
                    r#""min_price_increment_amount":"0.000000005","price_ratio":"0.000000010","inst_attrib_value":10,"underlying_id":256785,"market_depth_implied":0,"#,
                    r#""market_depth":13,"market_segment_id":0,"max_trade_vol":10000,"min_lot_size":1,"min_lot_size_block":1000,"min_lot_size_round_lot":100,"min_trade_vol":1,"#,
                    r#""contract_multiplier":0,"decay_quantity":0,"original_contract_size":0,"trading_reference_date":0,"appl_id":0,"#,
                    r#""maturity_year":0,"decay_start_date":0,"channel_id":4,"currency":"USD","settl_currency":"USD","secsubtype":"","raw_symbol":"ESZ4 C4100","group":"EW","exchange":"XCME","asset":"ES","cfi":"OCAFPS","#,
                    r#""security_type":"OOF","unit_of_measure":"IPNT","underlying":"ESZ4","strike_price_currency":"USD","instrument_class":"C","strike_price":"4100.000000000","match_algorithm":"F","md_security_trading_status":2,"main_fraction":4,"price_display_format":8,"#,
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
        let slice_res = write_json_to_string(data.as_slice(), false, false, false);
        let stream_res = write_json_stream_to_string(data, false, false, false);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{HEADER_JSON},{}}}\n",
                concat!(
                    r#""ts_recv":"1","ref_price":"2","auction_time":"3","cont_book_clr_price":"4","auct_interest_clr_price":"5","#,
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
            _dummy: Default::default(),
        }];
        let slice_res = write_json_to_string(data.as_slice(), false, true, false);
        let stream_res = write_json_stream_to_string(data, false, true, false);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{HEADER_JSON},{}}}\n",
                concat!(
                    r#""ts_recv":"1","ts_ref":"2","price":"0.000000003","quantity":0,"sequence":4,"#,
                    r#""ts_in_delta":5,"stat_type":1,"channel_id":7,"update_action":1,"stat_flags":0"#,
                )
            )
        );
    }

    #[test]
    fn test_metadata_write_json() {
        let metadata = Metadata {
            version: 1,
            dataset: GLBX_MDP3.to_owned(),
            schema: Some(Schema::Ohlcv1H),
            start: 1662734705128748281,
            end: NonZeroU64::new(1662734720914876944),
            limit: None,
            stype_in: Some(SType::InstrumentId),
            stype_out: SType::RawSymbol,
            ts_out: false,
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
            \"stype_in\":\"instrument_id\",\"stype_out\":\"raw_symbol\",\"ts_out\":false,\"symbols\"\
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
                r#""hd":{"rtype":4,"publisher_id":1,"instrument_id":323,"ts_event":"2022-07-21T22:17:31.000000000Z"}"#,
                r#""open":"5000","high":"8000","low":"3000","close":"6000","volume":"55000","ts_out":"2023-03-10T20:57:49.000000000Z""#,
            )
        );
    }

    #[test]
    fn test_serialize_quoted_str_to_json() {
        let json = write_json_to_string(
            vec![ErrorMsg::new(0, "\"A test")].as_slice(),
            false,
            true,
            true,
        );
        assert_eq!(
            json,
            r#"{"hd":{"rtype":21,"publisher_id":0,"instrument_id":0,"ts_event":null},"err":"\"A test"}
"#
        );
    }
}

#[cfg(feature = "async")]
pub use r#async::Encoder as AsyncEncoder;

#[cfg(feature = "async")]
mod r#async {
    use tokio::io;

    use crate::{
        encode::DbnEncodable, record_ref::RecordRef, rtype_ts_out_async_dispatch, Metadata,
    };

    /// Type for encoding files and streams of DBN records in newline-delimited JSON (ndjson).
    pub struct Encoder<W>
    where
        W: io::AsyncWriteExt + Unpin,
    {
        writer: W,
        should_pretty_print: bool,
        use_pretty_px: bool,
        use_pretty_ts: bool,
    }

    impl<W> Encoder<W>
    where
        W: io::AsyncWriteExt + Unpin,
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

        /// Encodes `metadata` into JSON.
        ///
        /// # Errors
        /// This function returns an error if there's an error writing to `writer`.
        pub async fn encode_metadata(&mut self, metadata: &Metadata) -> anyhow::Result<()> {
            let json = super::to_json_string(
                metadata,
                self.should_pretty_print,
                self.use_pretty_px,
                self.use_pretty_ts,
            );
            self.writer.write_all(json.as_bytes()).await?;
            self.writer.flush().await?;
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

        /// Encode a single DBN record of type `R`.
        ///
        /// Returns `true`if the pipe was closed.
        ///
        /// # Errors
        /// This function returns an error if it's unable to write to the underlying
        /// writer.
        pub async fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> anyhow::Result<bool> {
            let json = super::to_json_string(
                record,
                self.should_pretty_print,
                self.use_pretty_px,
                self.use_pretty_ts,
            );
            match self.writer.write_all(json.as_bytes()).await {
                Ok(_) => Ok(()),
                Err(e) if matches!(e.kind(), io::ErrorKind::BrokenPipe) => return Ok(true),
                Err(e) => {
                    Err(anyhow::Error::new(e).context(format!("Failed to serialize {record:#?}")))
                }
            }?;
            Ok(false)
        }

        /// Encodes a single DBN record.
        ///
        /// Returns `true`if the pipe was closed.
        ///
        /// # Safety
        /// `ts_out` must be `false` if `record` does not have an appended `ts_out
        ///
        /// # Errors
        /// This function returns an error if it's unable to write to the underlying writer
        /// or there's a serialization error.
        pub async unsafe fn encode_record_ref(
            &mut self,
            record_ref: RecordRef<'_>,
            ts_out: bool,
        ) -> anyhow::Result<bool> {
            rtype_ts_out_async_dispatch!(record_ref, ts_out, |rec| async move {
                self.encode_record(rec).await
            })?
        }
    }

    #[cfg(test)]
    mod tests {
        use std::ffi::c_char;

        use tokio::io::{AsyncWriteExt, BufWriter};

        use crate::{
            encode::test_data::RECORD_HEADER,
            enums::rtype,
            record::{HasRType, MboMsg, RecordHeader},
        };

        use super::*;

        async fn write_to_json_string<R>(
            record: &R,
            should_pretty_print: bool,
            use_pretty_px: bool,
            use_pretty_ts: bool,
        ) -> String
        where
            R: DbnEncodable,
        {
            let mut buffer = Vec::new();
            let mut writer = BufWriter::new(&mut buffer);
            Encoder::new(
                &mut writer,
                should_pretty_print,
                use_pretty_px,
                use_pretty_ts,
            )
            .encode_record(record)
            .await
            .unwrap();
            writer.flush().await.unwrap();
            String::from_utf8(buffer).expect("valid UTF-8")
        }

        async fn write_ref_to_json_string(
            record: RecordRef<'_>,
            should_pretty_print: bool,
            use_pretty_px: bool,
            use_pretty_ts: bool,
        ) -> String {
            let mut buffer = Vec::new();
            let mut writer = BufWriter::new(&mut buffer);
            unsafe {
                Encoder::new(
                    &mut writer,
                    should_pretty_print,
                    use_pretty_px,
                    use_pretty_ts,
                )
                .encode_record_ref(record, false)
            }
            .await
            .unwrap();
            writer.flush().await.unwrap();
            String::from_utf8(buffer).expect("valid UTF-8")
        }

        #[tokio::test]
        async fn test_mbo_write_json() {
            let record = MboMsg {
                hd: RecordHeader::new::<MboMsg>(
                    rtype::MBO,
                    RECORD_HEADER.publisher_id,
                    RECORD_HEADER.instrument_id,
                    RECORD_HEADER.ts_event,
                ),
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
            };
            let res = write_to_json_string(&record, false, true, false).await;
            let ref_res = write_ref_to_json_string(
                unsafe { RecordRef::unchecked_from_header(record.header()) },
                false,
                true,
                false,
            )
            .await;

            assert_eq!(res, ref_res);
            assert_eq!(
                ref_res,
                format!(
                    "{{{},{}}}\n",
                    r#""hd":{"rtype":160,"publisher_id":1,"instrument_id":323,"ts_event":"1658441851000000000"}"#,
                    r#""order_id":"16","price":"0.000005500","size":3,"flags":128,"channel_id":14,"action":"R","side":"N","ts_recv":"1658441891000000000","ts_in_delta":22000,"sequence":1002375"#
                )
            );
        }
    }
}
