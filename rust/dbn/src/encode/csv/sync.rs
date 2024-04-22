use std::{io, num::NonZeroU64};

use streaming_iterator::StreamingIterator;

use crate::{
    decode::{DbnMetadata, DecodeRecordRef},
    encode::{DbnEncodable, EncodeDbn, EncodeRecord, EncodeRecordRef, EncodeRecordTextExt},
    rtype_method_dispatch, rtype_ts_out_method_dispatch, schema_method_dispatch,
    schema_ts_out_method_dispatch, Error, RType, Record, Result, Schema,
};

/// Type for encoding files and streams of DBN records in CSV or other text-delimited
/// tabular file formats including TSV (tab-separated values).
///
/// Note that encoding [`Metadata`](crate::Metadata) in CSV is not supported.
pub struct Encoder<W>
where
    W: io::Write,
{
    writer: csv::Writer<W>,
    // Prevent writing header twice
    has_written_header: bool,
    use_pretty_px: bool,
    use_pretty_ts: bool,
}

/// Helper for constructing a CSV [`Encoder`].
///
/// If writing a CSV header (`write_header`), which is enabled by default,
/// `schema` is required, otherwise no fields are required.
pub struct EncoderBuilder<W>
where
    W: io::Write,
{
    writer: W,
    use_pretty_px: bool,
    use_pretty_ts: bool,
    write_header: bool,
    schema: Option<Schema>,
    ts_out: bool,
    with_symbol: bool,
    delimiter: u8,
}

impl<W> EncoderBuilder<W>
where
    W: io::Write,
{
    /// Creates a new CSV encoder builder.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            use_pretty_px: false,
            use_pretty_ts: false,
            write_header: true,
            schema: None,
            ts_out: false,
            with_symbol: false,
            delimiter: b',',
        }
    }

    /// Sets whether the CSV encoder will serialize price fields as a decimal. Defaults
    /// to `false`.
    pub fn use_pretty_px(mut self, use_pretty_px: bool) -> Self {
        self.use_pretty_px = use_pretty_px;
        self
    }

    /// Sets whether the CSV encoder will serialize timestamp fields as ISO8601 datetime
    /// strings. Defaults to `false`.
    pub fn use_pretty_ts(mut self, use_pretty_ts: bool) -> Self {
        self.use_pretty_ts = use_pretty_ts;
        self
    }

    /// Sets whether the CSV encoder will write a header row when it's created.
    /// Defaults to `true`. If `false`, a header row can still be written with
    /// [`Encoder::encode_header()`] or [`Encoder::encode_header_for_schema()`].
    pub fn write_header(mut self, write_header: bool) -> Self {
        self.write_header = write_header;
        self
    }

    /// Sets the schema that will be encoded. This is required if writing a header row.
    ///
    /// # Errors
    /// This function returns an error if `schema` is `None`. It accepts to an `Option` to
    /// more easily work with [`Metadata::schema`](crate::Metadata::schema).
    pub fn schema(mut self, schema: Option<Schema>) -> crate::Result<Self> {
        if schema.is_none() {
            return Err(Error::encode("can't encode a CSV with mixed schemas"));
        };
        self.schema = schema;
        Ok(self)
    }

    /// Sets whether to add a header field "ts_out". Defaults to `false`.
    pub fn ts_out(mut self, ts_out: bool) -> Self {
        self.ts_out = ts_out;
        self
    }

    /// Sets whether to add a header field "symbol". Defaults to `false`.
    pub fn with_symbol(mut self, with_symbol: bool) -> Self {
        self.with_symbol = with_symbol;
        self
    }

    /// Sets the field delimiter. Defaults to `b','` for comma-separated values (CSV).
    pub fn delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Creates the new encoder with the previously specified settings and if
    /// `write_header` is `true`, encodes the header row.
    ///
    /// # Errors
    /// This function returns an error if it fails to write the header row.
    pub fn build(self) -> crate::Result<Encoder<W>> {
        let mut encoder = Encoder {
            writer: csv::WriterBuilder::new()
                .has_headers(false)
                .delimiter(self.delimiter)
                .from_writer(self.writer),
            has_written_header: false,
            use_pretty_px: self.use_pretty_px,
            use_pretty_ts: self.use_pretty_ts,
        };
        if self.write_header {
            let Some(schema) = self.schema else {
                return Err(Error::BadArgument {
                    param_name: "schema".to_owned(),
                    desc: "need to specify schema in order to write header".to_owned(),
                });
            };
            encoder.encode_header_for_schema(schema, self.ts_out, self.with_symbol)?;
        }
        Ok(encoder)
    }
}

impl<W> Encoder<W>
where
    W: io::Write,
{
    /// Creates a builder for configuring an `Encoder` object.
    pub fn builder(writer: W) -> EncoderBuilder<W> {
        EncoderBuilder::new(writer)
    }

    /// Creates a new [`Encoder`] that will write to `writer`. If `use_pretty_px`
    /// is `true`, price fields will be serialized as a decimal. If `pretty_ts` is
    /// `true`, timestamp fields will be serialized in a ISO8601 datetime string.
    pub fn new(writer: W, use_pretty_px: bool, use_pretty_ts: bool) -> Self {
        Self::builder(writer)
            .write_header(false)
            .use_pretty_px(use_pretty_px)
            .use_pretty_ts(use_pretty_ts)
            .build()
            // Not setting `schema` or enabling `write_header`
            .unwrap()
    }

    /// Returns a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        self.writer.get_ref()
    }

    /// Encodes the CSV header for the record type `R`, i.e. the names of each of the
    /// fields to the output.
    ///
    /// If `with_symbol` is `true`, will add a header field for "symbol". This should
    /// only be used with  [`Self::encode_record_with_sym()`] and
    /// [`Self::encode_ref_with_sym()`], otherwise there will be a mismatch between the
    /// number of fields in the header and the body.
    ///
    /// # Errors
    /// This function returns an error if there's an error writing to `writer`.
    pub fn encode_header<R: DbnEncodable>(&mut self, with_symbol: bool) -> Result<()> {
        R::serialize_header(&mut self.writer)?;
        if with_symbol {
            self.writer.write_field("symbol")?;
        }
        // end of line
        self.writer.write_record(None::<&[u8]>)?;
        self.has_written_header = true;
        Ok(())
    }

    /// Encodes the CSV header for `schema`, i.e. the names of each of the fields to
    /// the output.
    ///
    /// If `ts_out` is `true`, it will add a header field "ts_out".
    ///
    /// If `with_symbol` is `true`, it will add a header field for "symbol". This should
    /// only be used with  [`Self::encode_record_with_sym()`] and
    /// [`Self::encode_ref_with_sym()`], otherwise there will be a mismatch between the
    /// number of fields in the header and the body.
    ///
    /// # Errors
    /// This function returns an error if there's an error writing to `writer`.
    pub fn encode_header_for_schema(
        &mut self,
        schema: Schema,
        ts_out: bool,
        with_symbol: bool,
    ) -> Result<()> {
        schema_ts_out_method_dispatch!(schema, ts_out, self, encode_header, with_symbol)?;
        self.has_written_header = true;
        Ok(())
    }

    fn encode_record_impl<R: DbnEncodable>(&mut self, record: &R) -> csv::Result<()> {
        match (self.use_pretty_px, self.use_pretty_ts) {
            (true, true) => record.serialize_to::<_, true, true>(&mut self.writer),
            (true, false) => record.serialize_to::<_, true, false>(&mut self.writer),
            (false, true) => record.serialize_to::<_, false, true>(&mut self.writer),
            (false, false) => record.serialize_to::<_, false, false>(&mut self.writer),
        }
    }

    fn encode_symbol(&mut self, symbol: Option<&str>) -> csv::Result<()> {
        self.writer.write_field(symbol.unwrap_or_default())
    }
}

impl<W> EncodeRecord for Encoder<W>
where
    W: io::Write,
{
    fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> Result<()> {
        match self
            .encode_record_impl(record)
            // write new line
            .and_then(|_| self.writer.write_record(None::<&[u8]>))
        {
            Ok(()) => Ok(()),
            Err(e) => Err(match e.into_kind() {
                csv::ErrorKind::Io(err) => Error::io(err, format!("serializing {record:?}")),
                e => Error::encode(format!("failed to serialize {record:?}: {e:?}")),
            }),
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

impl<W> EncodeDbn for Encoder<W>
where
    W: io::Write,
{
    fn encode_records<R: DbnEncodable>(&mut self, records: &[R]) -> Result<()> {
        if !self.has_written_header {
            self.encode_header::<R>(false)?;
        }
        for record in records {
            self.encode_record(record)?;
        }
        self.flush()?;
        Ok(())
    }

    /// Encodes a stream of DBN records.
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying writer
    /// or there's a serialization error.
    fn encode_stream<R: DbnEncodable>(
        &mut self,
        mut stream: impl StreamingIterator<Item = R>,
    ) -> Result<()> {
        if !self.has_written_header {
            self.encode_header::<R>(false)?;
        }
        while let Some(record) = stream.next() {
            self.encode_record(record)?;
        }
        self.flush()?;
        Ok(())
    }

    /// Encode DBN records directly from a DBN decoder. This implemented outside
    /// [`EncodeDbn`] because the CSV encoder has the additional constraint of only
    /// being able to encode a single schema in a stream.
    ///
    /// # Errors
    /// This function returns an error if it's unable to write to the underlying writer
    /// or there's a serialization error.
    fn encode_decoded<D: DecodeRecordRef + DbnMetadata>(&mut self, mut decoder: D) -> Result<()> {
        let ts_out = decoder.metadata().ts_out;
        if let Some(schema) = decoder.metadata().schema {
            if !self.has_written_header {
                self.encode_header_for_schema(schema, ts_out, false)?;
            }
            let rtype = RType::from(schema);
            while let Some(record) = decoder.decode_record_ref()? {
                if record.rtype().map_or(true, |r| r != rtype) {
                    return Err(Error::encode(format!("schema indicated {rtype:?}, but found record with rtype {:?}. Mixed schemas cannot be encoded in CSV.", record.rtype())));
                }
                // Safety: It's safe to cast to `WithTsOut` because we're passing in the `ts_out`
                // from the metadata header.
                unsafe { self.encode_record_ref_ts_out(record, ts_out) }?;
            }
            self.flush()?;
            Ok(())
        } else {
            Err(Error::encode("can't encode a CSV with mixed schemas"))
        }
    }

    fn encode_decoded_with_limit<D: DecodeRecordRef + DbnMetadata>(
        &mut self,
        mut decoder: D,
        limit: NonZeroU64,
    ) -> Result<()> {
        let ts_out = decoder.metadata().ts_out;
        if let Some(schema) = decoder.metadata().schema {
            schema_method_dispatch!(schema, self, encode_header, false)?;
            let rtype = RType::from(schema);
            let mut i = 0;
            while let Some(record) = decoder.decode_record_ref()? {
                if record.rtype().map_or(true, |r| r != rtype) {
                    return Err(Error::encode(format!("schema indicated {rtype:?}, but found record with rtype {:?}. Mixed schemas cannot be encoded in CSV.", record.rtype())));
                }
                // Safety: It's safe to cast to `WithTsOut` because we're passing in the `ts_out`
                // from the metadata header.
                unsafe { self.encode_record_ref_ts_out(record, ts_out) }?;
                i += 1;
                if i == limit.get() {
                    break;
                }
            }
            self.flush()?;
            Ok(())
        } else {
            Err(Error::encode("can't encode a CSV with mixed schemas"))
        }
    }
}

impl<W> EncodeRecordTextExt for Encoder<W>
where
    W: io::Write,
{
    fn encode_record_with_sym<R: DbnEncodable>(
        &mut self,
        record: &R,
        symbol: Option<&str>,
    ) -> Result<()> {
        match self
            .encode_record_impl(record)
            .and_then(|_| self.encode_symbol(symbol))
            // write new line
            .and_then(|_| self.writer.write_record(None::<&[u8]>))
        {
            Ok(()) => Ok(()),
            Err(e) => Err(match e.into_kind() {
                csv::ErrorKind::Io(err) => Error::io(err, format!("serializing {record:?}")),
                e => Error::encode(format!("failed to serialize {record:?}: {e:?}")),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{array, io::BufWriter, os::raw::c_char};

    use rstest::*;

    use super::*;
    use crate::{
        encode::test_data::{VecStream, BID_ASK, RECORD_HEADER},
        enums::{
            rtype, InstrumentClass, SecurityUpdateAction, StatType, StatUpdateAction,
            UserDefinedInstrument,
        },
        record::{
            str_to_c_chars, ImbalanceMsg, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg,
            RecordHeader, StatMsg, StatusMsg, TradeMsg, WithTsOut,
        },
        RecordRef, FIXED_PRICE_SCALE,
    };

    fn header(sep: char) -> String {
        format!("1658441851000000000{sep}4{sep}1{sep}323")
    }

    fn bid_ask(sep: char) -> String {
        format!("372000000000000{sep}372500000000000{sep}10{sep}5{sep}5{sep}2")
    }

    fn extract_2nd_line(buffer: Vec<u8>) -> String {
        let output = String::from_utf8(buffer).expect("valid UTF-8");
        let (first, second) = output.split_once('\n').expect("two lines");
        assert!(!first.trim().is_empty());
        second
            .trim_end() // remove newline
            .to_owned()
    }

    #[rstest]
    #[case::csv(b',')]
    #[case::tsv(b'\t')]
    fn test_mbo_encode_stream(#[case] sep: u8) {
        let data = vec![MboMsg {
            hd: RECORD_HEADER,
            order_id: 16,
            price: 5500,
            size: 3,
            flags: 128.into(),
            channel_id: 14,
            action: 'B' as c_char,
            side: 'B' as c_char,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::builder(writer)
            .delimiter(sep)
            .write_header(false)
            .build()
            .unwrap()
            .encode_stream(VecStream::new(data))
            .unwrap();
        let line = extract_2nd_line(buffer);
        let sep = sep as char;
        assert_eq!(
            line,
            format!(
                "1658441891000000000{sep}{}{sep}B{sep}B{sep}5500{sep}3{sep}14{sep}16{sep}128{sep}22000{sep}1002375",
                header(sep)
            )
        );
    }

    #[rstest]
    #[case::csv(b',')]
    #[case::tsv(b'\t')]
    fn test_mbp1_encode_records(#[case] sep: u8) {
        let data = vec![Mbp1Msg {
            hd: RECORD_HEADER,
            price: 5500,
            size: 3,
            action: 'M' as c_char,
            side: 'A' as c_char,
            flags: 128.into(),
            depth: 9,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
            levels: [BID_ASK],
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::builder(writer)
            .delimiter(sep)
            .write_header(false)
            .build()
            .unwrap()
            .encode_records(data.as_slice())
            .unwrap();
        let line = extract_2nd_line(buffer);
        let sep = sep as char;
        assert_eq!(
            line,
            format!(
                "1658441891000000000{sep}{}{sep}M{sep}A{sep}9{sep}5500{sep}3{sep}128{sep}22000{sep}1002375{sep}{}",
                header(sep),
                bid_ask(sep)
            )
        );
    }

    #[rstest]
    #[case::csv(b',')]
    #[case::tsv(b'\t')]
    fn test_mbp10_encode_stream(#[case] sep: u8) {
        let data = vec![Mbp10Msg {
            hd: RECORD_HEADER,
            price: 5500,
            size: 3,
            action: 'B' as c_char,
            side: 'A' as c_char,
            flags: 128.into(),
            depth: 9,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
            levels: array::from_fn(|_| BID_ASK.clone()),
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::builder(writer)
            .delimiter(sep)
            .write_header(false)
            .build()
            .unwrap()
            .encode_stream(VecStream::new(data))
            .unwrap();
        let line = extract_2nd_line(buffer);
        let sep = sep as char;
        assert_eq!(
            line,
            format!("1658441891000000000{sep}{}{sep}B{sep}A{sep}9{sep}5500{sep}3{sep}128{sep}22000\
                     {sep}1002375{sep}{bid_ask}{sep}{bid_ask}{sep}{bid_ask}{sep}{bid_ask}{sep}\
                     {bid_ask}{sep}{bid_ask}{sep}{bid_ask}{sep}{bid_ask}{sep}{bid_ask}{sep}{bid_ask}",
                     header(sep), bid_ask = bid_ask(sep))
        );
    }
    #[rstest]
    #[case::csv(b',')]
    #[case::tsv(b'\t')]
    fn test_trade_encode_records(#[case] sep: u8) {
        let data = vec![TradeMsg {
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
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::builder(writer)
            .delimiter(sep)
            .write_header(false)
            .build()
            .unwrap()
            .encode_records(data.as_slice())
            .unwrap();
        let line = extract_2nd_line(buffer);
        let sep = sep as char;
        assert_eq!(
            line,
            format!(
                "1658441891000000000{sep}{}{sep}B{sep}B{sep}9{sep}5500{sep}3{sep}128{sep}22000{sep}1002375",
                header(sep)
            )
        );
    }

    #[rstest]
    #[case::csv(b',')]
    #[case::tsv(b'\t')]
    fn test_ohlcv_encode_stream(#[case] sep: u8) {
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
        Encoder::builder(writer)
            .delimiter(sep)
            .write_header(false)
            .build()
            .unwrap()
            .encode_stream(VecStream::new(data))
            .unwrap();
        let line = extract_2nd_line(buffer);
        let sep = sep as char;
        assert_eq!(
            line,
            format!(
                "{}{sep}5000{sep}8000{sep}3000{sep}6000{sep}55000",
                header(sep)
            )
        );
    }

    #[rstest]
    #[case::csv(b',')]
    #[case::tsv(b'\t')]
    fn test_status_encode_records(#[case] sep: u8) {
        let mut group = [0; 21];
        for (i, c) in "group".chars().enumerate() {
            group[i] = c as c_char;
        }
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
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::builder(writer)
            .delimiter(sep)
            .write_header(false)
            .build()
            .unwrap()
            .encode_records(data.as_slice())
            .unwrap();
        let line = extract_2nd_line(buffer);
        let sep = sep as char;
        assert_eq!(
            line,
            format!(
                "1658441891000000000{sep}{}{sep}1{sep}2{sep}3{sep}Y{sep}Y{sep}~",
                header(sep)
            )
        );
    }

    #[rstest]
    #[case::csv(b',')]
    #[case::tsv(b'\t')]
    fn test_instrument_def_encode_stream(#[case] sep: u8) {
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
            instrument_class: InstrumentClass::Call as u8 as c_char,
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
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::builder(writer)
            .delimiter(sep)
            .write_header(false)
            .build()
            .unwrap()
            .encode_stream(VecStream::new(data))
            .unwrap();
        let line = extract_2nd_line(buffer);
        let sep = sep as char;
        assert_eq!(line, format!("1658441891000000000{sep}{}{sep}ESZ4 C4100{sep}A{sep}C{sep}100{sep}\
                                 1000{sep}1698450000000000000{sep}1697350000000000000{sep}1000000\
                                 {sep}-1000000{sep}0{sep}500000{sep}5{sep}5{sep}10{sep}10{sep}256785\
                                 {sep}323{sep}0{sep}13{sep}0{sep}10000{sep}1{sep}1000{sep}100{sep}1\
                                 {sep}0{sep}0{sep}0{sep}0{sep}0{sep}0{sep}0{sep}4{sep}USD{sep}USD\
                                 {sep}{sep}EW{sep}XCME{sep}ES{sep}OCAFPS{sep}OOF{sep}IPNT{sep}ESZ4{sep}\
                                 USD{sep}4100000000000{sep}F{sep}2{sep}4{sep}8{sep}9{sep}23{sep}10\
                                 {sep}8{sep}9{sep}11{sep}N{sep}0{sep}5{sep}0", header(sep)));
    }

    #[rstest]
    #[case::csv(b',')]
    #[case::tsv(b'\t')]
    fn test_encode_with_ts_out(#[case] sep: u8) {
        let data = vec![WithTsOut {
            rec: TradeMsg {
                hd: RECORD_HEADER,
                price: 5500,
                size: 3,
                action: 'T' as c_char,
                side: 'A' as c_char,
                flags: 128.into(),
                depth: 9,
                ts_recv: 1658441891000000000,
                ts_in_delta: 22_000,
                sequence: 1_002_375,
            },
            ts_out: 1678480044000000000,
        }];
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::builder(writer)
            .delimiter(sep)
            .write_header(false)
            .build()
            .unwrap()
            .encode_records(data.as_slice())
            .unwrap();
        let lines = String::from_utf8(buffer).expect("valid UTF-8");
        let sep = sep as char;
        assert_eq!(
            lines,
            format!("ts_recv{sep}ts_event{sep}rtype{sep}publisher_id{sep}instrument_id{sep}action\
                    {sep}side{sep}depth{sep}price{sep}size{sep}flags{sep}ts_in_delta{sep}sequence\
                    {sep}ts_out\n1658441891000000000{sep}{}{sep}T{sep}A{sep}9{sep}5500{sep}3{sep}128\
                    {sep}22000{sep}1002375{sep}1678480044000000000\n", header(sep))
        );
    }

    #[rstest]
    #[case::csv(b',')]
    #[case::tsv(b'\t')]
    fn test_imbalance_encode_records(#[case] sep: u8) {
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
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::builder(writer)
            .delimiter(sep)
            .write_header(false)
            .build()
            .unwrap()
            .encode_records(data.as_slice())
            .unwrap();
        let line = extract_2nd_line(buffer);
        let sep = sep as char;
        assert_eq!(
            line,
            format!(
                "1{sep}{}{sep}2{sep}3{sep}4{sep}5{sep}6{sep}7{sep}8{sep}9{sep}10{sep}11{sep}12{sep}\
                13{sep}B{sep}A{sep}14{sep}15{sep}16{sep}A{sep}N",
                header(sep)
            )
        );
    }

    #[rstest]
    #[case::csv(b',')]
    #[case::tsv(b'\t')]
    fn test_stat_encode_stream(#[case] sep: u8) {
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
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        Encoder::builder(writer)
            .delimiter(sep)
            .write_header(false)
            .build()
            .unwrap()
            .encode_stream(VecStream::new(data))
            .unwrap();
        let line = extract_2nd_line(buffer);
        let sep = sep as char;
        assert_eq!(
            line,
            format!(
                "1{sep}{}{sep}2{sep}3{sep}0{sep}4{sep}5{sep}1{sep}7{sep}1{sep}0",
                header(sep)
            )
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
        let mut encoder = Encoder::new(&mut buffer, false, false);
        encoder.encode_ref_with_sym(rec_ref, None).unwrap();
        encoder.encode_ref_with_sym(rec_ref, Some("AAPL")).unwrap();
        drop(encoder);
        let res = String::from_utf8(buffer).unwrap();
        assert_eq!(
            res,
            "0,34,10,9,175000000000,177000000000,174000000000,175000000000,4033445,\n\
            0,34,10,9,175000000000,177000000000,174000000000,175000000000,4033445,AAPL\n"
        );
    }

    #[test]
    fn test_encode_header_for_schema() {
        let mut buffer = Vec::new();
        {
            let mut encoder = Encoder::new(&mut buffer, false, false);
            encoder
                .encode_header_for_schema(Schema::Statistics, false, false)
                .unwrap();
        }
        {
            let mut encoder = Encoder::new(&mut buffer, false, false);
            encoder
                .encode_header_for_schema(Schema::Statistics, true, true)
                .unwrap();
        }

        let res = String::from_utf8(buffer).unwrap();
        let (fst_line, snd_line) = res.split_once('\n').unwrap();
        assert!(snd_line.ends_with(",symbol\n"));
        let orig_header = snd_line.split_once(",ts_out,symbol").unwrap().0;
        assert_eq!(fst_line, orig_header);
    }
}
