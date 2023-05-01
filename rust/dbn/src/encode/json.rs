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

type JsonObject = serde_json::Map<String, serde_json::Value>;

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
        let mut json = to_json_string(metadata, self.use_pretty_px, self.use_pretty_ts);
        if self.should_pretty_print {
            json = serde_json::to_string_pretty(&serde_json::from_str::<JsonObject>(&json)?)?;
        }
        self.writer.write_all(json.as_bytes())?;
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
}

impl<W> EncodeDbn for Encoder<W>
where
    W: io::Write,
{
    fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> anyhow::Result<bool> {
        let mut json = to_json_string(record, self.use_pretty_px, self.use_pretty_ts);
        if self.should_pretty_print {
            json = serde_json::to_string_pretty(&serde_json::from_str::<JsonObject>(&json)?)?;
        }
        match self.writer.write_all(json.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) if matches!(e.kind(), io::ErrorKind::BrokenPipe) => return Ok(true),
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

    fn flush(&mut self) -> anyhow::Result<()> {
        Ok(self.writer.flush()?)
    }
}

pub(crate) mod serialize {
    use json_writer::{JSONArrayWriter, JSONObjectWriter, NULL};
    use time::format_description::FormatItem;

    use crate::{
        encode::{format_px, format_ts},
        record::{
            BidAskPair, ErrorMsg, HasRType, ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg,
            Mbp1Msg, OhlcvMsg, RecordHeader, StatMsg, StatusMsg, SymbolMappingMsg, SystemMsg,
            TradeMsg, WithTsOut,
        },
        Metadata, UNDEF_PRICE,
    };

    pub fn to_json_string<T: JsonSerialize>(
        obj: &T,
        use_pretty_px: bool,
        use_pretty_ts: bool,
    ) -> String {
        let mut res = String::new();
        {
            let mut writer = JSONObjectWriter::new(&mut res);
            match (use_pretty_px, use_pretty_ts) {
                (true, true) => obj.to_json::<true, true>(&mut writer),
                (true, false) => obj.to_json::<true, false>(&mut writer),
                (false, true) => obj.to_json::<false, true>(&mut writer),
                (false, false) => obj.to_json::<false, false>(&mut writer),
            };
        }
        res
    }

    pub trait JsonSerialize {
        fn to_json<const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            writer: &mut JSONObjectWriter,
        );
    }

    impl JsonSerialize for MboMsg {
        fn to_json<const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            writer: &mut JSONObjectWriter,
        ) {
            write_header::<PRETTY_TS>(writer, &self.hd);
            writer.value("order_id", &self.order_id.to_string());
            write_px_field::<PRETTY_PX>(writer, "price", self.price);
            writer.value("size", self.size);
            writer.value("flags", self.flags);
            writer.value("channel_id", self.channel_id);
            writer.value("action", &(self.action as u8 as char).to_string());
            writer.value("side", &(self.side as u8 as char).to_string());
            write_ts_field::<PRETTY_TS>(writer, "ts_recv", self.ts_recv);
            writer.value("ts_in_delta", self.ts_in_delta);
            writer.value("sequence", self.sequence);
        }
    }

    impl JsonSerialize for Mbp1Msg {
        fn to_json<const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            writer: &mut JSONObjectWriter,
        ) {
            write_header::<PRETTY_TS>(writer, &self.hd);
            write_px_field::<PRETTY_PX>(writer, "price", self.price);
            writer.value("size", self.size);
            writer.value("action", &(self.action as u8 as char).to_string());
            writer.value("side", &(self.side as u8 as char).to_string());
            writer.value("flags", self.flags);
            writer.value("depth", self.depth);
            write_ts_field::<PRETTY_TS>(writer, "ts_recv", self.ts_recv);
            writer.value("ts_in_delta", self.ts_in_delta);
            writer.value("sequence", self.sequence);
            write_ba_pair::<PRETTY_PX>(&mut writer.array("booklevel"), &self.booklevel[0]);
        }
    }

    impl JsonSerialize for Mbp10Msg {
        fn to_json<const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            writer: &mut JSONObjectWriter,
        ) {
            write_header::<PRETTY_TS>(writer, &self.hd);
            write_px_field::<PRETTY_PX>(writer, "price", self.price);
            writer.value("size", self.size);
            writer.value("action", &(self.action as u8 as char).to_string());
            writer.value("side", &(self.side as u8 as char).to_string());
            writer.value("flags", self.flags);
            writer.value("depth", self.depth);
            write_ts_field::<PRETTY_TS>(writer, "ts_recv", self.ts_recv);
            writer.value("ts_in_delta", self.ts_in_delta);
            writer.value("sequence", self.sequence);
            let mut arr_writer = writer.array("booklevel");
            for level in self.booklevel.iter() {
                write_ba_pair::<PRETTY_PX>(&mut arr_writer, level);
            }
        }
    }

    impl JsonSerialize for TradeMsg {
        fn to_json<const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            writer: &mut JSONObjectWriter,
        ) {
            write_header::<PRETTY_TS>(writer, &self.hd);
            write_px_field::<PRETTY_PX>(writer, "price", self.price);
            writer.value("size", self.size);
            writer.value("action", &(self.action as u8 as char).to_string());
            writer.value("side", &(self.side as u8 as char).to_string());
            writer.value("flags", self.flags);
            writer.value("depth", self.depth);
            write_ts_field::<PRETTY_TS>(writer, "ts_recv", self.ts_recv);
            writer.value("ts_in_delta", self.ts_in_delta);
            writer.value("sequence", self.sequence);
        }
    }

    impl JsonSerialize for OhlcvMsg {
        fn to_json<const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            writer: &mut JSONObjectWriter,
        ) {
            write_header::<PRETTY_TS>(writer, &self.hd);
            write_px_field::<PRETTY_PX>(writer, "open", self.open);
            write_px_field::<PRETTY_PX>(writer, "high", self.high);
            write_px_field::<PRETTY_PX>(writer, "low", self.low);
            write_px_field::<PRETTY_PX>(writer, "close", self.close);
            writer.value("volume", &self.volume.to_string());
        }
    }

    impl JsonSerialize for StatusMsg {
        fn to_json<const _PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            writer: &mut JSONObjectWriter,
        ) {
            write_header::<PRETTY_TS>(writer, &self.hd);
            write_ts_field::<PRETTY_TS>(writer, "ts_recv", self.ts_recv);
            writer.value("group", self.group().unwrap_or_default());
            writer.value("trading_status", self.trading_status);
            writer.value("halt_reason", self.halt_reason);
            writer.value("trading_event", self.trading_event);
        }
    }

    impl JsonSerialize for InstrumentDefMsg {
        fn to_json<const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            writer: &mut JSONObjectWriter,
        ) {
            write_header::<PRETTY_TS>(writer, &self.hd);
            write_ts_field::<PRETTY_TS>(writer, "ts_recv", self.ts_recv);
            write_px_field::<PRETTY_PX>(writer, "min_price_increment", self.min_price_increment);
            write_px_field::<PRETTY_PX>(writer, "display_factor", self.display_factor);
            write_ts_field::<PRETTY_TS>(writer, "expiration", self.expiration);
            write_ts_field::<PRETTY_TS>(writer, "activation", self.activation);
            write_px_field::<PRETTY_PX>(writer, "high_limit_price", self.high_limit_price);
            write_px_field::<PRETTY_PX>(writer, "low_limit_price", self.low_limit_price);
            write_px_field::<PRETTY_PX>(writer, "max_price_variation", self.max_price_variation);
            write_px_field::<PRETTY_PX>(
                writer,
                "trading_reference_price",
                self.trading_reference_price,
            );
            writer.value("unit_of_measure_qty", &self.unit_of_measure_qty.to_string());
            write_px_field::<PRETTY_PX>(
                writer,
                "min_price_increment_amount",
                self.min_price_increment_amount,
            );
            write_px_field::<PRETTY_PX>(writer, "price_ratio", self.price_ratio);
            writer.value("inst_attrib_value", self.inst_attrib_value);
            writer.value("underlying_id", self.underlying_id);
            writer.value("cleared_volume", self.cleared_volume);
            writer.value("market_depth_implied", self.market_depth_implied);
            writer.value("market_depth", self.market_depth);
            writer.value("market_segment_id", self.market_segment_id);
            writer.value("max_trade_vol", self.max_trade_vol);
            writer.value("min_lot_size", self.min_lot_size);
            writer.value("min_lot_size_block", self.min_lot_size_block);
            writer.value("min_lot_size_round_lot", self.min_lot_size_round_lot);
            writer.value("min_trade_vol", self.min_trade_vol);
            writer.value("open_interest_qty", self.open_interest_qty);
            writer.value("contract_multiplier", self.contract_multiplier);
            writer.value("decay_quantity", self.decay_quantity);
            writer.value("original_contract_size", self.original_contract_size);
            writer.value("trading_reference_date", self.trading_reference_date);
            writer.value("appl_id", self.appl_id);
            writer.value("maturity_year", self.maturity_year);
            writer.value("decay_start_date", self.decay_start_date);
            writer.value("channel_id", self.channel_id);
            writer.value("currency", self.currency().unwrap_or_default());
            writer.value("settl_currency", self.settl_currency().unwrap_or_default());
            writer.value("secsubtype", self.secsubtype().unwrap_or_default());
            writer.value("raw_symbol", self.raw_symbol().unwrap_or_default());
            writer.value("group", self.group().unwrap_or_default());
            writer.value("exchange", self.exchange().unwrap_or_default());
            writer.value("asset", self.asset().unwrap_or_default());
            writer.value("cfi", self.cfi().unwrap_or_default());
            writer.value("security_type", self.security_type().unwrap_or_default());
            writer.value(
                "unit_of_measure",
                self.unit_of_measure().unwrap_or_default(),
            );
            writer.value("underlying", self.underlying().unwrap_or_default());
            writer.value(
                "strike_price_currency",
                self.strike_price_currency().unwrap_or_default(),
            );
            writer.value(
                "instrument_class",
                &(self.instrument_class as u8 as char).to_string(),
            );
            write_px_field::<PRETTY_PX>(writer, "strike_price", self.strike_price);
            writer.value(
                "match_algorithm",
                &(self.match_algorithm as u8 as char).to_string(),
            );
            writer.value(
                "md_security_trading_status",
                self.md_security_trading_status,
            );
            writer.value("main_fraction", self.main_fraction);
            writer.value("price_display_format", self.price_display_format);
            writer.value("settl_price_type", self.settl_price_type);
            writer.value("sub_fraction", self.sub_fraction);
            writer.value("underlying_product", self.underlying_product);
            writer.value(
                "security_update_action",
                &(self.security_update_action as u8 as char).to_string(),
            );
            writer.value("maturity_month", self.maturity_month);
            writer.value("maturity_day", self.maturity_day);
            writer.value("maturity_week", self.maturity_week);
            writer.value(
                "user_defined_instrument",
                &(self.user_defined_instrument as u8 as char).to_string(),
            );
            writer.value("contract_multiplier_unit", self.contract_multiplier_unit);
            writer.value("flow_schedule_type", self.flow_schedule_type);
            writer.value("tick_rule", self.tick_rule);
        }
    }

    impl JsonSerialize for ImbalanceMsg {
        fn to_json<const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            writer: &mut JSONObjectWriter,
        ) {
            write_header::<PRETTY_TS>(writer, &self.hd);
            write_ts_field::<PRETTY_TS>(writer, "ts_recv", self.ts_recv);
            write_px_field::<PRETTY_PX>(writer, "ref_price", self.ref_price);
            writer.value("auction_time", &self.auction_time.to_string());
            write_px_field::<PRETTY_PX>(writer, "cont_book_clr_price", self.cont_book_clr_price);
            write_px_field::<PRETTY_PX>(
                writer,
                "auct_interest_clr_price",
                self.auct_interest_clr_price,
            );
            write_px_field::<PRETTY_PX>(writer, "ssr_filling_price", self.ssr_filling_price);
            write_px_field::<PRETTY_PX>(writer, "ind_match_price", self.ind_match_price);
            write_px_field::<PRETTY_PX>(writer, "upper_collar", self.upper_collar);
            write_px_field::<PRETTY_PX>(writer, "lower_collar", self.lower_collar);
            writer.value("paired_qty", self.paired_qty);
            writer.value("total_imbalance_qty", self.total_imbalance_qty);
            writer.value("market_imbalance_qty", self.market_imbalance_qty);
            writer.value("unpaired_qty", self.unpaired_qty);
            writer.value(
                "auction_type",
                &(self.auction_type as u8 as char).to_string(),
            );
            writer.value("side", &(self.side as u8 as char).to_string());
            writer.value("auction_status", self.auction_status);
            writer.value("freeze_status", self.freeze_status);
            writer.value("num_extensions", self.num_extensions);
            writer.value(
                "unpaired_side",
                &(self.unpaired_side as u8 as char).to_string(),
            );
            writer.value(
                "significant_imbalance",
                &(self.significant_imbalance as u8 as char).to_string(),
            );
        }
    }

    impl JsonSerialize for StatMsg {
        fn to_json<const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            writer: &mut JSONObjectWriter,
        ) {
            write_header::<PRETTY_TS>(writer, &self.hd);
            write_ts_field::<PRETTY_TS>(writer, "ts_recv", self.ts_recv);
            write_ts_field::<PRETTY_TS>(writer, "ts_ref", self.ts_ref);
            write_px_field::<PRETTY_PX>(writer, "price", self.price);
            writer.value("quantity", self.quantity);
            writer.value("sequence", self.sequence);
            writer.value("ts_in_delta", self.ts_in_delta);
            writer.value("stat_type", self.stat_type);
            writer.value("channel_id", self.channel_id);
            writer.value("update_action", self.update_action);
            writer.value("stat_flags", self.stat_flags);
        }
    }

    impl JsonSerialize for ErrorMsg {
        fn to_json<const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            writer: &mut JSONObjectWriter,
        ) {
            write_header::<PRETTY_TS>(writer, &self.hd);
            writer.value("err", self.err().unwrap_or_default());
        }
    }

    impl JsonSerialize for SystemMsg {
        fn to_json<const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            writer: &mut JSONObjectWriter,
        ) {
            write_header::<PRETTY_TS>(writer, &self.hd);
            writer.value("msg", self.msg().unwrap_or_default());
        }
    }

    impl JsonSerialize for SymbolMappingMsg {
        fn to_json<const _PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            writer: &mut JSONObjectWriter,
        ) {
            write_header::<PRETTY_TS>(writer, &self.hd);
            writer.value(
                "stype_in_symbol",
                self.stype_in_symbol().unwrap_or_default(),
            );
            writer.value(
                "stype_out_symbol",
                self.stype_out_symbol().unwrap_or_default(),
            );
            write_ts_field::<PRETTY_TS>(writer, "start_ts", self.start_ts);
            write_ts_field::<PRETTY_TS>(writer, "end_ts", self.end_ts);
        }
    }

    impl<T: HasRType + JsonSerialize> JsonSerialize for WithTsOut<T> {
        fn to_json<const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            writer: &mut JSONObjectWriter,
        ) {
            self.rec.to_json::<PRETTY_PX, PRETTY_TS>(writer);
            write_ts_field::<PRETTY_TS>(writer, "ts_out", self.ts_out);
        }
    }

    impl JsonSerialize for Metadata {
        fn to_json<const _PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            writer: &mut JSONObjectWriter,
        ) {
            writer.value("version", self.version);
            writer.value("dataset", &self.dataset);
            writer.value("schema", self.schema.map(|s| s.as_str()));
            write_ts_field::<PRETTY_TS>(writer, "start", self.start);
            if let Some(end) = self.end {
                write_ts_field::<PRETTY_TS>(writer, "end", end.get());
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
                    write_date_field::<PRETTY_TS>(
                        &mut interval_writer,
                        "start_date",
                        &interval.start_date,
                    );
                    write_date_field::<PRETTY_TS>(
                        &mut interval_writer,
                        "end_date",
                        &interval.end_date,
                    );
                    interval_writer.value("symbol", &interval.symbol);
                }
            }
        }
    }

    fn write_header<const PRETTY_TS: bool>(writer: &mut JSONObjectWriter, hd: &RecordHeader) {
        let mut hd_writer = writer.object("hd");
        hd_writer.value("rtype", hd.rtype);
        hd_writer.value("publisher_id", hd.publisher_id);
        hd_writer.value("instrument_id", hd.instrument_id);
        write_ts_field::<PRETTY_TS>(&mut hd_writer, "ts_event", hd.ts_event);
    }

    fn write_ba_pair<const PRETTY_PX: bool>(
        arr_writer: &mut JSONArrayWriter,
        ba_pair: &BidAskPair,
    ) {
        let mut writer = arr_writer.object();
        write_px_field::<PRETTY_PX>(&mut writer, "bid_px", ba_pair.bid_px);
        write_px_field::<PRETTY_PX>(&mut writer, "ask_px", ba_pair.ask_px);
        writer.value("bid_sz", ba_pair.bid_sz);
        writer.value("ask_sz", ba_pair.ask_sz);
        writer.value("bid_ct", ba_pair.bid_ct);
        writer.value("ask_ct", ba_pair.ask_ct);
    }

    fn write_px_field<const PRETTY_PX: bool>(writer: &mut JSONObjectWriter, key: &str, px: i64) {
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

    fn write_ts_field<const PRETTY_TS: bool>(writer: &mut JSONObjectWriter, key: &str, ts: u64) {
        if PRETTY_TS {
            if ts == 0 {
                writer.value(key, NULL);
            } else {
                writer.value(key, &format_ts(ts));
            }
        } else {
            // Convert to string to avoid a loss of precision
            writer.value(key, &ts.to_string());
        }
    }

    fn write_date_field<const PRETTY_TS: bool>(
        writer: &mut JSONObjectWriter,
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

    const HEADER_JSON: &str =
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
            booklevel: [BID_ASK],
        }];
        let slice_res = write_json_to_string(data.as_slice(), false, true, true);
        let stream_res = write_json_stream_to_string(data, false, true, true);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{},{},{}}}\n",
                r#""hd":{"rtype":4,"publisher_id":1,"instrument_id":323,"ts_event":"2022-07-21T22:17:31.000000000"}"#,
                r#""price":"0.000005500","size":3,"action":"B","side":"B","flags":128,"depth":9,"ts_recv":"2022-07-21T22:18:11.000000000","ts_in_delta":22000,"sequence":1002375"#,
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
        let slice_res = write_json_to_string(data.as_slice(), false, true, true);
        let stream_res = write_json_stream_to_string(data, false, true, true);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{},{},{}}}\n",
                r#""hd":{"rtype":4,"publisher_id":1,"instrument_id":323,"ts_event":"2022-07-21T22:17:31.000000000"}"#,
                r#""price":"0.000005500","size":3,"action":"T","side":"N","flags":128,"depth":9,"ts_recv":"2022-07-21T22:18:11.000000000","ts_in_delta":22000,"sequence":1002375"#,
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
                r#""hd":{"rtype":4,"publisher_id":1,"instrument_id":323,"ts_event":"2022-07-21T22:17:31.000000000"}"#,
                r#""ts_recv":"2022-07-21T22:18:11.000000000","group":"group","trading_status":3,"halt_reason":4,"trading_event":6"#,
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
            reserved1: Default::default(),
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
            reserved2: Default::default(),
            strike_price: 4_100_000_000_000,
            reserved3: Default::default(),
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
            _dummy: [0; 3],
        }];
        let slice_res = write_json_to_string(data.as_slice(), false, true, true);
        let stream_res = write_json_stream_to_string(data, false, true, true);

        assert_eq!(slice_res, stream_res);
        assert_eq!(
            slice_res,
            format!(
                "{{{},{}}}\n",
                r#""hd":{"rtype":4,"publisher_id":1,"instrument_id":323,"ts_event":"2022-07-21T22:17:31.000000000"}"#,
                concat!(
                    r#""ts_recv":"2022-07-21T22:18:11.000000000","min_price_increment":"0.000000100","display_factor":"0.000001000","expiration":"2023-10-27T23:40:00.000000000","activation":"2023-10-15T06:06:40.000000000","#,
                    r#""high_limit_price":"0.001000000","low_limit_price":"-0.001000000","max_price_variation":"0.000000000","trading_reference_price":"0.000500000","unit_of_measure_qty":"5","#,
                    r#""min_price_increment_amount":"0.000000005","price_ratio":"0.000000010","inst_attrib_value":10,"underlying_id":256785,"cleared_volume":0,"market_depth_implied":0,"#,
                    r#""market_depth":13,"market_segment_id":0,"max_trade_vol":10000,"min_lot_size":1,"min_lot_size_block":1000,"min_lot_size_round_lot":100,"min_trade_vol":1,"#,
                    r#""open_interest_qty":0,"contract_multiplier":0,"decay_quantity":0,"original_contract_size":0,"trading_reference_date":0,"appl_id":0,"#,
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
            dataset: "GLBX.MDP3".to_owned(),
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
            :\"2022-09-09T14:45:05.128748281\",\"end\":\"2022-09-09T14:45:20.914876944\",\"limit\":null,\
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
                r#""hd":{"rtype":4,"publisher_id":1,"instrument_id":323,"ts_event":"2022-07-21T22:17:31.000000000"}"#,
                r#""open":"5000","high":"8000","low":"3000","close":"6000","volume":"55000","ts_out":"2023-03-10T20:57:49.000000000""#,
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
