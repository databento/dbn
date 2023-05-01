//! Encoding of DBN records into comma-separated values (CSV).
use std::io;

use anyhow::anyhow;
use streaming_iterator::StreamingIterator;

use super::EncodeDbn;
use crate::{
    decode::DecodeDbn,
    enums::{RType, Schema},
    record::{
        ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, StatMsg, StatusMsg,
        TbboMsg, TradeMsg,
    },
};

/// Type for encoding files and streams of DBN records in CSV.
///
/// Note that encoding [`Metadata`](crate::Metadata) in CSV is not supported.
pub struct Encoder<W>
where
    W: io::Write,
{
    writer: csv::Writer<W>,
    use_pretty_px: bool,
    use_pretty_ts: bool,
}

impl<W> Encoder<W>
where
    W: io::Write,
{
    /// Creates a new [`Encoder`] that will write to `writer`. If `use_pretty_px`
    /// is `true`, price fields will be serialized as a decimal. If `pretty_ts` is
    /// `true`, timestamp fields will be serialized in a ISO8601 datetime string.
    pub fn new(writer: W, use_pretty_px: bool, use_pretty_ts: bool) -> Self {
        let csv_writer = csv::WriterBuilder::new()
            .has_headers(false) // need to write our own custom header
            .from_writer(writer);
        Self {
            writer: csv_writer,
            use_pretty_px,
            use_pretty_ts,
        }
    }

    fn encode_header<R: super::DbnEncodable>(&mut self) -> anyhow::Result<()> {
        R::serialize_header(&mut self.writer)?;
        // end of line
        self.writer.write_record(None::<&[u8]>)?;
        Ok(())
    }
}

impl<W> EncodeDbn for Encoder<W>
where
    W: io::Write,
{
    fn encode_record<R: super::DbnEncodable>(&mut self, record: &R) -> anyhow::Result<bool> {
        let serialize_res = match (self.use_pretty_px, self.use_pretty_ts) {
            (true, true) => record.serialize_to::<_, true, true>(&mut self.writer),
            (true, false) => record.serialize_to::<_, true, false>(&mut self.writer),
            (false, true) => record.serialize_to::<_, false, true>(&mut self.writer),
            (false, false) => record.serialize_to::<_, false, false>(&mut self.writer),
        };
        match serialize_res
            // write new line
            .and_then(|_| self.writer.write_record(None::<&[u8]>))
        {
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
        self.encode_header::<R>()?;
        for record in records {
            if self.encode_record(record)? {
                break;
            }
        }
        self.flush()?;
        Ok(())
    }

    /// Encodes a stream of DBN records.
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying writer
    /// or there's a serialization error.
    fn encode_stream<R: super::DbnEncodable>(
        &mut self,
        mut stream: impl StreamingIterator<Item = R>,
    ) -> anyhow::Result<()> {
        self.encode_header::<R>()?;
        while let Some(record) = stream.next() {
            if self.encode_record(record)? {
                break;
            }
        }
        self.flush()?;
        Ok(())
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        Ok(self.writer.flush()?)
    }

    /// Encode DBN records directly from a DBN decoder. This implemented outside [`EncodeDbn`](super::EncodeDbn)
    /// because the CSV encoder has the additional constraint of only being able to encode a single schema in
    /// a stream.
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying writer
    /// or there's a serialization error.
    fn encode_decoded<D: DecodeDbn>(&mut self, mut decoder: D) -> anyhow::Result<()> {
        let ts_out = decoder.metadata().ts_out;
        if let Some(schema) = decoder.metadata().schema {
            match schema {
                Schema::Mbo => self.encode_header::<MboMsg>(),
                Schema::Mbp1 => self.encode_header::<Mbp1Msg>(),
                Schema::Mbp10 => self.encode_header::<Mbp10Msg>(),
                Schema::Tbbo => self.encode_header::<TbboMsg>(),
                Schema::Trades => self.encode_header::<TradeMsg>(),
                Schema::Ohlcv1S | Schema::Ohlcv1M | Schema::Ohlcv1H | Schema::Ohlcv1D => {
                    self.encode_header::<OhlcvMsg>()
                }
                Schema::Definition => self.encode_header::<InstrumentDefMsg>(),
                Schema::Statistics => self.encode_header::<StatMsg>(),
                Schema::Status => self.encode_header::<StatusMsg>(),
                Schema::Imbalance => self.encode_header::<ImbalanceMsg>(),
            }?;
            let rtype = RType::from(schema);
            while let Some(record) = decoder.decode_record_ref()? {
                if record.rtype().map_or(true, |r| r != rtype) {
                    return Err(anyhow!("Schema indicated {rtype:?}, but found record with rtype {:?}. Mixed schemas cannot be encoded in CSV.", record.rtype()));
                }
                // Safety: It's safe to cast to `WithTsOut` because we're passing in the `ts_out`
                // from the metadata header.
                if unsafe { self.encode_record_ref(record, ts_out)? } {
                    break;
                }
            }
            self.flush()?;
            Ok(())
        } else {
            Err(anyhow!("Can't encode a DBN with mixed schemas in CSV"))
        }
    }
}

pub(crate) mod serialize {
    use std::io;

    use csv::Writer;

    use crate::{
        encode::{format_px, format_ts},
        record::{
            BidAskPair, ErrorMsg, HasRType, ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg,
            Mbp1Msg, OhlcvMsg, RecordHeader, StatMsg, StatusMsg, SymbolMappingMsg, SystemMsg,
            TradeMsg, WithTsOut,
        },
        UNDEF_PRICE,
    };

    /// Because of the flat nature of CSVs, there are several limitations in the
    /// Rust CSV serde serialization library. This trait helps work around them.
    pub trait CsvSerialize {
        /// Encode the header to `csv_writer`.
        fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()>;

        /// Serialize the object to `csv_writer`. Allows custom behavior that would otherwise
        /// cause a runtime error, e.g. serializing a struct with array field.
        fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            csv_writer: &mut Writer<W>,
        ) -> csv::Result<()>;
    }

    impl CsvSerialize for RecordHeader {
        fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            ["rtype", "publisher_id", "instrument_id", "ts_event"]
                .iter()
                .try_for_each(|header| csv_writer.write_field(header))
        }

        fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            csv_writer: &mut Writer<W>,
        ) -> csv::Result<()> {
            csv_writer.write_field(self.rtype.to_string())?;
            csv_writer.write_field(self.publisher_id.to_string())?;
            csv_writer.write_field(self.instrument_id.to_string())?;
            write_ts_field::<W, PRETTY_TS>(csv_writer, self.ts_event)
        }
    }

    impl CsvSerialize for MboMsg {
        fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            RecordHeader::serialize_header(csv_writer)?;
            [
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

        fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            csv_writer: &mut Writer<W>,
        ) -> csv::Result<()> {
            self.hd
                .serialize_to::<W, PRETTY_PX, PRETTY_TS>(csv_writer)?;
            csv_writer.write_field(self.order_id.to_string())?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.price)?;
            csv_writer.write_field(self.size.to_string())?;
            csv_writer.write_field(self.flags.to_string())?;
            csv_writer.write_field(self.channel_id.to_string())?;
            csv_writer.write_field((self.action as u8 as char).to_string())?;
            csv_writer.write_field((self.side as u8 as char).to_string())?;
            write_ts_field::<W, PRETTY_TS>(csv_writer, self.ts_recv)?;
            csv_writer.write_field(self.ts_in_delta.to_string())?;
            csv_writer.write_field(self.sequence.to_string())
        }
    }

    impl CsvSerialize for Mbp1Msg {
        fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            RecordHeader::serialize_header(csv_writer)?;
            [
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

        fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            csv_writer: &mut Writer<W>,
        ) -> csv::Result<()> {
            self.hd
                .serialize_to::<W, PRETTY_PX, PRETTY_TS>(csv_writer)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.price)?;
            csv_writer.write_field(self.size.to_string())?;
            csv_writer.write_field((self.action as u8 as char).to_string())?;
            csv_writer.write_field((self.side as u8 as char).to_string())?;
            csv_writer.write_field(self.flags.to_string())?;
            csv_writer.write_field(self.depth.to_string())?;
            write_ts_field::<W, PRETTY_TS>(csv_writer, self.ts_recv)?;
            csv_writer.write_field(self.ts_in_delta.to_string())?;
            csv_writer.write_field(self.sequence.to_string())?;
            write_ba_pair::<W, PRETTY_PX>(csv_writer, &self.booklevel[0])
        }
    }

    impl CsvSerialize for Mbp10Msg {
        fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            RecordHeader::serialize_header(csv_writer)?;
            [
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

        fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            csv_writer: &mut Writer<W>,
        ) -> csv::Result<()> {
            self.hd
                .serialize_to::<W, PRETTY_PX, PRETTY_TS>(csv_writer)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.price)?;
            csv_writer.write_field(self.size.to_string())?;
            csv_writer.write_field((self.action as u8 as char).to_string())?;
            csv_writer.write_field((self.side as u8 as char).to_string())?;
            csv_writer.write_field(self.flags.to_string())?;
            csv_writer.write_field(self.depth.to_string())?;
            write_ts_field::<W, PRETTY_TS>(csv_writer, self.ts_recv)?;
            csv_writer.write_field(self.ts_in_delta.to_string())?;
            csv_writer.write_field(self.sequence.to_string())?;
            for level in self.booklevel.iter() {
                write_ba_pair::<W, PRETTY_PX>(csv_writer, level)?;
            }
            Ok(())
        }
    }

    impl CsvSerialize for TradeMsg {
        fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            RecordHeader::serialize_header(csv_writer)?;
            [
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

        fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            csv_writer: &mut Writer<W>,
        ) -> csv::Result<()> {
            self.hd
                .serialize_to::<W, PRETTY_PX, PRETTY_TS>(csv_writer)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.price)?;
            csv_writer.write_field(self.size.to_string())?;
            csv_writer.write_field((self.action as u8 as char).to_string())?;
            csv_writer.write_field((self.side as u8 as char).to_string())?;
            csv_writer.write_field(self.flags.to_string())?;
            csv_writer.write_field(self.depth.to_string())?;
            write_ts_field::<W, PRETTY_TS>(csv_writer, self.ts_recv)?;
            csv_writer.write_field(self.ts_in_delta.to_string())?;
            csv_writer.write_field(self.sequence.to_string())
        }
    }

    impl CsvSerialize for OhlcvMsg {
        fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            RecordHeader::serialize_header(csv_writer)?;
            ["open", "high", "low", "close", "volume"]
                .iter()
                .try_for_each(|header| csv_writer.write_field(header))
        }

        fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            csv_writer: &mut Writer<W>,
        ) -> csv::Result<()> {
            self.hd
                .serialize_to::<W, PRETTY_PX, PRETTY_TS>(csv_writer)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.open)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.high)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.low)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.close)?;
            csv_writer.write_field(self.volume.to_string())
        }
    }

    impl CsvSerialize for StatusMsg {
        fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            RecordHeader::serialize_header(csv_writer)?;
            [
                "ts_recv",
                "group",
                "trading_status",
                "halt_reason",
                "trading_event",
            ]
            .iter()
            .try_for_each(|header| csv_writer.write_field(header))
        }

        fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            csv_writer: &mut Writer<W>,
        ) -> csv::Result<()> {
            self.hd
                .serialize_to::<W, PRETTY_PX, PRETTY_TS>(csv_writer)?;
            write_ts_field::<W, PRETTY_TS>(csv_writer, self.ts_recv)?;
            csv_writer.write_field(self.group().unwrap_or_default())?;
            csv_writer.write_field(self.trading_status.to_string())?;
            csv_writer.write_field(self.halt_reason.to_string())?;
            csv_writer.write_field(self.trading_event.to_string())
        }
    }

    impl CsvSerialize for InstrumentDefMsg {
        fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            RecordHeader::serialize_header(csv_writer)?;
            [
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
                "trading_reference_date",
                "appl_id",
                "maturity_year",
                "decay_start_date",
                "channel_id",
                "currency",
                "settl_currency",
                "secsubtype",
                "raw_symbol",
                "group",
                "exchange",
                "asset",
                "cfi",
                "security_type",
                "unit_of_measure",
                "underlying",
                "strike_price_currency",
                "instrument_class",
                "strike_price",
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

        fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            csv_writer: &mut Writer<W>,
        ) -> csv::Result<()> {
            self.hd
                .serialize_to::<W, PRETTY_PX, PRETTY_TS>(csv_writer)?;
            write_ts_field::<W, PRETTY_TS>(csv_writer, self.ts_recv)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.min_price_increment)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.display_factor)?;
            write_ts_field::<W, PRETTY_TS>(csv_writer, self.expiration)?;
            write_ts_field::<W, PRETTY_TS>(csv_writer, self.activation)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.high_limit_price)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.low_limit_price)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.max_price_variation)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.trading_reference_price)?;
            csv_writer.write_field(self.unit_of_measure_qty.to_string())?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.min_price_increment_amount)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.price_ratio)?;
            csv_writer.write_field(self.inst_attrib_value.to_string())?;
            csv_writer.write_field(self.underlying_id.to_string())?;
            csv_writer.write_field(self.cleared_volume.to_string())?;
            csv_writer.write_field(self.market_depth_implied.to_string())?;
            csv_writer.write_field(self.market_depth.to_string())?;
            csv_writer.write_field(self.market_segment_id.to_string())?;
            csv_writer.write_field(self.max_trade_vol.to_string())?;
            csv_writer.write_field(self.min_lot_size.to_string())?;
            csv_writer.write_field(self.min_lot_size_block.to_string())?;
            csv_writer.write_field(self.min_lot_size_round_lot.to_string())?;
            csv_writer.write_field(self.min_trade_vol.to_string())?;
            csv_writer.write_field(self.open_interest_qty.to_string())?;
            csv_writer.write_field(self.contract_multiplier.to_string())?;
            csv_writer.write_field(self.decay_quantity.to_string())?;
            csv_writer.write_field(self.original_contract_size.to_string())?;
            csv_writer.write_field(self.trading_reference_date.to_string())?;
            csv_writer.write_field(self.appl_id.to_string())?;
            csv_writer.write_field(self.maturity_year.to_string())?;
            csv_writer.write_field(self.decay_start_date.to_string())?;
            csv_writer.write_field(self.channel_id.to_string())?;
            csv_writer.write_field(self.currency().unwrap_or_default())?;
            csv_writer.write_field(self.settl_currency().unwrap_or_default())?;
            csv_writer.write_field(self.secsubtype().unwrap_or_default())?;
            csv_writer.write_field(self.raw_symbol().unwrap_or_default())?;
            csv_writer.write_field(self.group().unwrap_or_default())?;
            csv_writer.write_field(self.exchange().unwrap_or_default())?;
            csv_writer.write_field(self.asset().unwrap_or_default())?;
            csv_writer.write_field(self.cfi().unwrap_or_default())?;
            csv_writer.write_field(self.security_type().unwrap_or_default())?;
            csv_writer.write_field(self.unit_of_measure().unwrap_or_default())?;
            csv_writer.write_field(self.underlying().unwrap_or_default())?;
            csv_writer.write_field(self.strike_price_currency().unwrap_or_default())?;
            csv_writer.write_field((self.instrument_class as u8 as char).to_string())?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.strike_price)?;
            csv_writer.write_field((self.match_algorithm as u8 as char).to_string())?;
            csv_writer.write_field(self.md_security_trading_status.to_string())?;
            csv_writer.write_field(self.main_fraction.to_string())?;
            csv_writer.write_field(self.price_display_format.to_string())?;
            csv_writer.write_field(self.settl_price_type.to_string())?;
            csv_writer.write_field(self.sub_fraction.to_string())?;
            csv_writer.write_field(self.underlying_product.to_string())?;
            csv_writer.write_field((self.security_update_action as u8 as char).to_string())?;
            csv_writer.write_field(self.maturity_month.to_string())?;
            csv_writer.write_field(self.maturity_day.to_string())?;
            csv_writer.write_field(self.maturity_week.to_string())?;
            csv_writer.write_field((self.user_defined_instrument as u8 as char).to_string())?;
            csv_writer.write_field(self.contract_multiplier_unit.to_string())?;
            csv_writer.write_field(self.flow_schedule_type.to_string())?;
            csv_writer.write_field(self.tick_rule.to_string())
        }
    }

    impl CsvSerialize for ImbalanceMsg {
        fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            RecordHeader::serialize_header(csv_writer)?;
            [
                "ts_recv",
                "ref_price",
                "auction_time",
                "cont_book_clr_price",
                "auct_interest_clr_price",
                "ssr_filling_price",
                "ind_match_price",
                "upper_collar",
                "lower_collar",
                "paired_qty",
                "total_imbalance_qty",
                "market_imbalance_qty",
                "unpaired_qty",
                "auction_type",
                "side",
                "auction_status",
                "freeze_status",
                "num_extension",
                "unpaired_side",
                "significant_imbalance",
            ]
            .iter()
            .try_for_each(|header| csv_writer.write_field(header))
        }

        fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            csv_writer: &mut Writer<W>,
        ) -> csv::Result<()> {
            self.hd
                .serialize_to::<W, PRETTY_PX, PRETTY_TS>(csv_writer)?;
            write_ts_field::<W, PRETTY_TS>(csv_writer, self.ts_recv)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.ref_price)?;
            csv_writer.write_field(self.auction_time.to_string())?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.cont_book_clr_price)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.auct_interest_clr_price)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.ssr_filling_price)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.ind_match_price)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.upper_collar)?;
            write_px_field::<W, PRETTY_PX>(csv_writer, self.lower_collar)?;
            csv_writer.write_field(self.paired_qty.to_string())?;
            csv_writer.write_field(self.total_imbalance_qty.to_string())?;
            csv_writer.write_field(self.market_imbalance_qty.to_string())?;
            csv_writer.write_field(self.unpaired_qty.to_string())?;
            csv_writer.write_field((self.auction_type as u8 as char).to_string())?;
            csv_writer.write_field((self.side as u8 as char).to_string())?;
            csv_writer.write_field(self.auction_status.to_string())?;
            csv_writer.write_field(self.freeze_status.to_string())?;
            csv_writer.write_field(self.num_extensions.to_string())?;
            csv_writer.write_field((self.unpaired_side as u8 as char).to_string())?;
            csv_writer.write_field((self.significant_imbalance as u8 as char).to_string())
        }
    }

    impl CsvSerialize for StatMsg {
        fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            RecordHeader::serialize_header(csv_writer)?;
            [
                "ts_recv",
                "ts_ref",
                "price",
                "quantity",
                "sequence",
                "ts_in_delta",
                "stat_type",
                "channel_id",
                "update_action",
                "stat_flags",
            ]
            .iter()
            .try_for_each(|header| csv_writer.write_field(header))
        }

        fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            csv_writer: &mut Writer<W>,
        ) -> csv::Result<()> {
            self.hd
                .serialize_to::<W, PRETTY_PX, PRETTY_TS>(csv_writer)?;
            write_ts_field::<W, PRETTY_TS>(csv_writer, self.ts_recv)?;
            write_ts_field::<W, PRETTY_TS>(csv_writer, self.ts_ref)?;
            write_px_field::<W, PRETTY_TS>(csv_writer, self.price)?;
            csv_writer.write_field(self.quantity.to_string())?;
            csv_writer.write_field(self.sequence.to_string())?;
            csv_writer.write_field(self.ts_in_delta.to_string())?;
            csv_writer.write_field(self.stat_type.to_string())?;
            csv_writer.write_field(self.channel_id.to_string())?;
            csv_writer.write_field(self.update_action.to_string())?;
            csv_writer.write_field(self.stat_flags.to_string())
        }
    }

    impl CsvSerialize for ErrorMsg {
        fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            RecordHeader::serialize_header(csv_writer)?;
            csv_writer.write_field("err")
        }

        fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            csv_writer: &mut Writer<W>,
        ) -> csv::Result<()> {
            self.hd
                .serialize_to::<W, PRETTY_PX, PRETTY_TS>(csv_writer)?;
            csv_writer.write_field(self.err().unwrap_or_default())
        }
    }

    impl CsvSerialize for SystemMsg {
        fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            RecordHeader::serialize_header(csv_writer)?;
            csv_writer.write_field("msg")
        }

        fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            csv_writer: &mut Writer<W>,
        ) -> csv::Result<()> {
            self.hd
                .serialize_to::<W, PRETTY_PX, PRETTY_TS>(csv_writer)?;
            csv_writer.write_field(self.msg().unwrap_or_default())
        }
    }

    impl CsvSerialize for SymbolMappingMsg {
        fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            RecordHeader::serialize_header(csv_writer)?;
            ["stype_in_symbol", "stype_out_symbol", "start_ts", "end_ts"]
                .iter()
                .try_for_each(|header| csv_writer.write_field(header))
        }

        fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            csv_writer: &mut Writer<W>,
        ) -> csv::Result<()> {
            self.hd
                .serialize_to::<W, PRETTY_PX, PRETTY_TS>(csv_writer)?;
            csv_writer.write_field(self.stype_in_symbol().unwrap_or_default())?;
            csv_writer.write_field(self.stype_out_symbol().unwrap_or_default())?;
            write_ts_field::<W, PRETTY_TS>(csv_writer, self.start_ts)?;
            write_ts_field::<W, PRETTY_TS>(csv_writer, self.end_ts)
        }
    }

    impl<T: HasRType + CsvSerialize> CsvSerialize for WithTsOut<T> {
        fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
            T::serialize_header(csv_writer)?;
            csv_writer.write_field("ts_out")
        }

        fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
            &self,
            csv_writer: &mut Writer<W>,
        ) -> csv::Result<()> {
            self.rec
                .serialize_to::<W, PRETTY_PX, PRETTY_TS>(csv_writer)?;
            write_ts_field::<W, PRETTY_TS>(csv_writer, self.ts_out)
        }
    }

    fn write_px_field<W: io::Write, const PRETTY_PX: bool>(
        csv_writer: &mut Writer<W>,
        px: i64,
    ) -> csv::Result<()> {
        if PRETTY_PX {
            if px == UNDEF_PRICE {
                csv_writer.write_field("")
            } else {
                csv_writer.write_field(format_px(px))
            }
        } else {
            csv_writer.write_field(px.to_string())
        }
    }

    fn write_ts_field<W: io::Write, const PRETTY_TS: bool>(
        csv_writer: &mut Writer<W>,
        ts: u64,
    ) -> csv::Result<()> {
        if PRETTY_TS {
            if ts == 0 {
                csv_writer.write_field("")
            } else {
                csv_writer.write_field(format_ts(ts))
            }
        } else {
            csv_writer.write_field(ts.to_string())
        }
    }

    fn write_ba_pair<W: io::Write, const PRETTY_PX: bool>(
        csv_writer: &mut Writer<W>,
        ba_pair: &BidAskPair,
    ) -> csv::Result<()> {
        write_px_field::<W, PRETTY_PX>(csv_writer, ba_pair.bid_px)?;
        write_px_field::<W, PRETTY_PX>(csv_writer, ba_pair.ask_px)?;
        csv_writer.write_field(ba_pair.bid_sz.to_string())?;
        csv_writer.write_field(ba_pair.ask_sz.to_string())?;
        csv_writer.write_field(ba_pair.bid_ct.to_string())?;
        csv_writer.write_field(ba_pair.ask_ct.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::{array, io::BufWriter, os::raw::c_char};

    use super::*;
    use crate::{
        encode::test_data::{VecStream, BID_ASK, RECORD_HEADER},
        enums::{
            InstrumentClass, SecurityUpdateAction, StatType, StatUpdateAction,
            UserDefinedInstrument,
        },
        record::{
            str_to_c_chars, ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg,
            StatMsg, StatusMsg, TradeMsg, WithTsOut,
        },
    };

    const HEADER_CSV: &str = "4,1,323,1658441851000000000";

    const BID_ASK_CSV: &str = "372000000000000,372500000000000,10,5,5,2";

    fn extract_2nd_line(buffer: Vec<u8>) -> String {
        let output = String::from_utf8(buffer).expect("valid UTF-8");
        let (first, second) = output.split_once('\n').expect("two lines");
        assert!(!first.trim().is_empty());
        second
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
        Encoder::new(writer, false, false)
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
        Encoder::new(writer, false, false)
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
        Encoder::new(writer, false, false)
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
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer, false, false)
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
        Encoder::new(writer, false, false)
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
        Encoder::new(writer, false, false)
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
            reserved1: Default::default(),
            trading_reference_date: 0,
            appl_id: 0,
            maturity_year: 0,
            decay_start_date: 0,
            channel_id: 4,
            currency: [0; 4],
            settl_currency: str_to_c_chars("USD").unwrap(),
            secsubtype: [0; 6],
            raw_symbol: [0; 22],
            group: [0; 21],
            exchange: [0; 5],
            asset: [0; 7],
            cfi: [0; 7],
            security_type: [0; 7],
            unit_of_measure: [0; 31],
            underlying: [0; 21],
            strike_price_currency: Default::default(),
            instrument_class: InstrumentClass::Future as u8 as c_char,
            reserved2: Default::default(),
            strike_price: 0,
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
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer, false, false)
            .encode_stream(VecStream::new(data))
            .unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(line, format!("{HEADER_CSV},1658441891000000000,100,1000,1698450000000000000,1697350000000000000,1000000,-1000000,0,500000,5,5,10,10,256785,0,0,13,0,10000,1,1000,100,1,0,0,0,0,0,0,0,0,4,,USD,,,,,,,,,,,F,0,F,2,4,8,9,23,10,A,8,9,11,N,0,5,0"));
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
            },
            ts_out: 1678480044000000000,
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer, false, false)
            .encode_records(data.as_slice())
            .unwrap();
        let lines = String::from_utf8(buffer).expect("valid UTF-8");
        assert_eq!(
            lines,
            format!("rtype,publisher_id,instrument_id,ts_event,price,size,action,side,flags,depth,ts_recv,ts_in_delta,sequence,ts_out\n{HEADER_CSV},5500,3,T,A,128,9,1658441891000000000,22000,1002375,1678480044000000000\n")
        );
    }

    #[test]
    fn test_imbalance_encode_records() {
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
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer, false, false)
            .encode_records(data.as_slice())
            .unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(
            line,
            format!("{HEADER_CSV},1,2,3,4,5,6,7,8,9,10,11,12,13,B,A,14,15,16,A,N")
        );
    }

    #[test]
    fn test_stat_encode_stream() {
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
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::new(writer, false, false)
            .encode_stream(VecStream::new(data))
            .unwrap();
        let line = extract_2nd_line(buffer);
        assert_eq!(line, format!("{HEADER_CSV},1,2,3,0,4,5,1,7,1,0"));
    }
}
