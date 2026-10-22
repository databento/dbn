//! Encoding DBN version 4 records into comma-separated values (CSV).

use std::io;

use csv::Writer;

use super::serialize::{write_c_char_field, write_px_field, write_ts_field, WriteField};
use crate::{
    encode::resolved_layout::{ResolvedField, ResolvedStreamLayout, ResolvedValue},
    record::HasRType,
    v4::{self, fields::ScalarValue},
    Error, Metadata, RecordRef, Result,
};

/// Encodes DBN version 4 records into CSV or other text-delimited tabular formats,
/// reading each record's fields from the stream layout.
///
/// Every record must share the first record's rtype.
pub struct Encoder<W>
where
    W: io::Write,
{
    writer: Writer<W>,
    layout: ResolvedStreamLayout,
    rtype: Option<u16>,
    write_header: bool,
    use_pretty_px: bool,
    use_pretty_ts: bool,
    with_symbol: bool,
}

/// Helper for constructing a DBN version 4 CSV [`Encoder`].
pub struct EncoderBuilder<'a, W>
where
    W: io::Write,
{
    writer: W,
    metadata: &'a Metadata,
    use_pretty_px: bool,
    use_pretty_ts: bool,
    write_header: bool,
    with_symbol: bool,
    delimiter: u8,
}

impl<'a, W> EncoderBuilder<'a, W>
where
    W: io::Write,
{
    /// Creates a new CSV encoder builder for records described by `metadata`'s layout.
    pub fn new(writer: W, metadata: &'a Metadata) -> Self {
        Self {
            writer,
            metadata,
            use_pretty_px: false,
            use_pretty_ts: false,
            write_header: true,
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

    /// Sets whether the CSV encoder will write a header row before the first record.
    /// Defaults to `true`.
    ///
    /// If `false`, a header row can still be written with [`Encoder::encode_header()`].
    pub fn write_header(mut self, write_header: bool) -> Self {
        self.write_header = write_header;
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

    /// Creates the new encoder with the previously specified settings.
    ///
    /// # Errors
    /// This function returns an error if the metadata carries no stream layout, the
    /// layout is invalid, or it contains a field the encoder doesn't support.
    pub fn build(self) -> Result<Encoder<W>> {
        Ok(Encoder {
            writer: csv::WriterBuilder::new()
                .has_headers(false)
                .delimiter(self.delimiter)
                .from_writer(self.writer),
            layout: ResolvedStreamLayout::from_metadata(self.metadata)?,
            rtype: None,
            write_header: self.write_header,
            use_pretty_px: self.use_pretty_px,
            use_pretty_ts: self.use_pretty_ts,
            with_symbol: self.with_symbol,
        })
    }
}

impl<W> Encoder<W>
where
    W: io::Write,
{
    /// Creates a builder for configuring an `Encoder` object.
    pub fn builder(writer: W, metadata: &Metadata) -> EncoderBuilder<'_, W> {
        EncoderBuilder::new(writer, metadata)
    }

    /// Creates a new instance of [`Encoder`] for records described by `metadata`'s
    /// layout. If `use_pretty_px` is `true`, price fields will be serialized as a
    /// decimal. If `use_pretty_ts` is `true`, timestamp fields will be serialized as
    /// ISO8601 datetime strings. The encoder writes a header row before the first
    /// record.
    ///
    /// # Errors
    /// This function returns an error if `metadata` carries no stream layout, the
    /// layout is invalid, or it contains a field the encoder doesn't support.
    pub fn new(
        writer: W,
        metadata: &Metadata,
        use_pretty_px: bool,
        use_pretty_ts: bool,
    ) -> Result<Self> {
        Self::builder(writer, metadata)
            .use_pretty_px(use_pretty_px)
            .use_pretty_ts(use_pretty_ts)
            .build()
    }

    /// Encodes the CSV header for records of `rtype`, i.e. the names of each of the
    /// fields to the output.
    ///
    /// # Errors
    /// This function returns an error if the stream layout doesn't describe `rtype` or
    /// there's an error writing to `writer`.
    pub fn encode_header(&mut self, rtype: u16) -> Result<()> {
        let layout = self.layout.get(rtype)?;
        write_header(&mut self.writer, &layout.fields, "")?;
        if self.with_symbol {
            self.writer.write_field("symbol")?;
        }
        self.writer.write_record(None::<&[u8]>)?;
        self.rtype = Some(rtype);
        Ok(())
    }

    /// Encodes a single record, preceded by the header row if it's the first.
    ///
    /// # Errors
    /// This function returns an error if the stream layout doesn't describe the
    /// record's rtype, the record is shorter than its layout, its rtype differs from
    /// the first record's, or it fails to write to the underlying writer.
    pub fn encode_record<R>(&mut self, record: &R) -> Result<()>
    where
        R: HasRType<Header = v4::RecordHeader>,
    {
        self.encode_record_ref(RecordRef::from(record))
    }

    /// Encodes a single record, preceded by the header row if it's the first.
    ///
    /// # Errors
    /// This function returns an error if the stream layout doesn't describe the
    /// record's rtype, the record is shorter than its layout, its rtype differs from
    /// the first record's, or it fails to write to the underlying writer.
    pub fn encode_record_ref(&mut self, record: RecordRef<'_, v4::RecordHeader>) -> Result<()> {
        if self.with_symbol {
            return Err(Error::encode(
                "Encoder configured with with_symbol(true); use encode_ref_with_sym",
            ));
        }
        self.encode_record_impl(record, None)
    }

    /// Encodes a single record along with the record's text symbol.
    ///
    /// # Errors
    /// This function returns an error if the stream layout doesn't describe the
    /// record's rtype, the record is shorter than its layout, its rtype differs from
    /// the first record's, or it fails to write to the underlying writer.
    pub fn encode_record_with_sym<R>(&mut self, record: &R, symbol: Option<&str>) -> Result<()>
    where
        R: HasRType<Header = v4::RecordHeader>,
    {
        self.encode_ref_with_sym(RecordRef::from(record), symbol)
    }

    /// Encodes a single record along with the record's text symbol.
    ///
    /// # Errors
    /// This function returns an error if the stream layout doesn't describe the
    /// record's rtype, the record is shorter than its layout, its rtype differs from
    /// the first record's, or it fails to write to the underlying writer.
    pub fn encode_ref_with_sym(
        &mut self,
        record: RecordRef<'_, v4::RecordHeader>,
        symbol: Option<&str>,
    ) -> Result<()> {
        if !self.with_symbol {
            return Err(Error::encode(
                "Encoder not configured with with_symbol(true); use encode_record_ref",
            ));
        }
        self.encode_record_impl(record, symbol)
    }

    /// Flushes any buffered content to the underlying writer.
    ///
    /// # Errors
    /// This function returns an error if it fails to flush the underlying writer.
    pub fn flush(&mut self) -> Result<()> {
        self.writer
            .flush()
            .map_err(|e| Error::io(e, "flushing output"))
    }

    /// Returns a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        self.writer.get_ref()
    }

    /// Flushes any buffered data and returns the underlying writer.
    ///
    /// # Errors
    /// This function returns an error if it fails to flush to the underlying writer.
    pub fn into_inner(self) -> Result<W> {
        self.writer
            .into_inner()
            .map_err(|e| Error::io(e.into_error(), "flushing CSV writer"))
    }

    fn encode_record_impl(
        &mut self,
        record: RecordRef<'_, v4::RecordHeader>,
        symbol: Option<&str>,
    ) -> Result<()> {
        let rtype = record.header().rtype;
        if self.rtype.is_none() && self.write_header {
            self.encode_header(rtype)?;
        }
        let layout = self.layout.layout_for(record)?;
        let first_rtype = *self.rtype.get_or_insert(rtype);
        if first_rtype != rtype {
            return Err(Error::encode(format!(
                "first record has rtype {first_rtype:#06X}, but found record with rtype {rtype:#06X}. Mixed rtypes cannot be encoded in CSV."
            )));
        }
        let writer = &mut self.writer;
        let bytes = record.as_ref();
        match (self.use_pretty_px, self.use_pretty_ts) {
            (true, true) => write_fields::<_, true, true>(writer, &layout.fields, bytes),
            (true, false) => write_fields::<_, true, false>(writer, &layout.fields, bytes),
            (false, true) => write_fields::<_, false, true>(writer, &layout.fields, bytes),
            (false, false) => write_fields::<_, false, false>(writer, &layout.fields, bytes),
        }
        .and_then(|()| {
            if self.with_symbol {
                writer.write_field(symbol.unwrap_or_default())
            } else {
                Ok(())
            }
        })
        .and_then(|()| writer.write_record(None::<&[u8]>))
        .map_err(|e| match e.into_kind() {
            csv::ErrorKind::Io(err) => Error::io(err, format!("serializing {:?}", record.header())),
            e => Error::encode(format!("failed to serialize {:?}: {e:?}", record.header())),
        })
    }
}

fn write_header<W: io::Write>(
    writer: &mut Writer<W>,
    fields: &[ResolvedField],
    suffix: &str,
) -> csv::Result<()> {
    for field in fields {
        write_value_header(writer, &field.name, &field.value, suffix)?;
    }
    Ok(())
}

fn write_value_header<W: io::Write>(
    writer: &mut Writer<W>,
    name: &str,
    value: &ResolvedValue,
    suffix: &str,
) -> csv::Result<()> {
    match value {
        ResolvedValue::Scalar { .. } => writer.write_field(format!("{name}{suffix}")),
        ResolvedValue::Struct(fields) => write_header(writer, fields, suffix),
        ResolvedValue::Array(elems) => {
            for (i, elem) in elems.iter().enumerate() {
                write_value_header(writer, name, elem, &format!("{suffix}_{i:02}"))?;
            }
            Ok(())
        }
    }
}

fn write_fields<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
    writer: &mut Writer<W>,
    fields: &[ResolvedField],
    bytes: &[u8],
) -> csv::Result<()> {
    for field in fields {
        write_value::<_, PRETTY_PX, PRETTY_TS>(writer, &field.value, bytes)?;
    }
    Ok(())
}

fn write_value<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
    writer: &mut Writer<W>,
    value: &ResolvedValue,
    bytes: &[u8],
) -> csv::Result<()> {
    match value {
        ResolvedValue::Scalar(scalar) => match scalar.read(bytes) {
            Some(value) => write_scalar::<_, PRETTY_PX, PRETTY_TS>(writer, value),
            None => writer.write_field(""),
        },
        ResolvedValue::Struct(fields) => {
            write_fields::<_, PRETTY_PX, PRETTY_TS>(writer, fields, bytes)
        }
        ResolvedValue::Array(elems) => {
            for elem in elems {
                write_value::<_, PRETTY_PX, PRETTY_TS>(writer, elem, bytes)?;
            }
            Ok(())
        }
    }
}

fn write_scalar<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
    writer: &mut Writer<W>,
    value: ScalarValue<'_>,
) -> csv::Result<()> {
    match value {
        ScalarValue::U8(v) => v.write_field::<W, false, false>(writer),
        ScalarValue::U16(v) => v.write_field::<W, false, false>(writer),
        ScalarValue::U32(v) => v.write_field::<W, false, false>(writer),
        ScalarValue::U64(v) => v.write_field::<W, false, false>(writer),
        ScalarValue::I8(v) => v.write_field::<W, false, false>(writer),
        ScalarValue::I16(v) => v.write_field::<W, false, false>(writer),
        ScalarValue::I32(v) => v.write_field::<W, false, false>(writer),
        ScalarValue::I64(v) => v.write_field::<W, false, false>(writer),
        ScalarValue::F32(v) => writer.write_field(v.to_string()),
        ScalarValue::F64(v) => writer.write_field(v.to_string()),
        ScalarValue::Decimal(px) => write_px_field::<_, PRETTY_PX>(writer, px.raw),
        ScalarValue::Timestamp(ts) => write_ts_field::<_, PRETTY_TS>(writer, ts.0),
        ScalarValue::Char(c) => write_c_char_field(writer, c.0),
        ScalarValue::Str(s) => writer.write_field(s),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::{
        decode::dbn::tests::{metadata_v4, trade_v4},
        encode::{csv, DbnEncodable, EncodeRecord, EncodeRecordTextExt},
        test_utils::{
            as_ref, mbo_v3, mbo_v4, mbp10_v3, mbp10_v4, symbol_mapping_v3, symbol_mapping_v4,
            trade_v4_with_sale_condition, trades_with_sale_condition,
        },
        v4::types::Char,
        Schema,
    };

    fn encode_v4<'a>(
        metadata: &Metadata,
        records: impl IntoIterator<Item = RecordRef<'a, v4::RecordHeader>>,
        delimiter: u8,
        pretty_px: bool,
        pretty_ts: bool,
    ) -> Result<String> {
        let mut encoder = Encoder::builder(Vec::new(), metadata)
            .delimiter(delimiter)
            .use_pretty_px(pretty_px)
            .use_pretty_ts(pretty_ts)
            .build()?;
        for record in records {
            encoder.encode_record_ref(record)?;
        }
        Ok(String::from_utf8(encoder.into_inner()?).unwrap())
    }

    fn encode_v3<R: DbnEncodable>(
        record: &R,
        delimiter: u8,
        pretty_px: bool,
        pretty_ts: bool,
    ) -> String {
        let mut encoder = csv::Encoder::builder(Vec::new())
            .delimiter(delimiter)
            .use_pretty_px(pretty_px)
            .use_pretty_ts(pretty_ts)
            .build()
            .unwrap();
        encoder.encode_record(record).unwrap();
        String::from_utf8(encoder.into_inner().unwrap()).unwrap()
    }

    #[rstest]
    fn mbo_matches_v3(
        #[values(b',', b'\t')] delimiter: u8,
        #[values(false, true)] pretty_px: bool,
        #[values(false, true)] pretty_ts: bool,
    ) {
        let v4 = mbo_v4();
        assert_eq!(
            encode_v4(
                &metadata_v4(Schema::Mbo),
                [RecordRef::from(&v4)],
                delimiter,
                pretty_px,
                pretty_ts
            )
            .unwrap(),
            encode_v3(&mbo_v3(), delimiter, pretty_px, pretty_ts)
        );
    }

    #[rstest]
    fn mbp10_matches_v3(
        #[values(b',', b'\t')] delimiter: u8,
        #[values(false, true)] pretty_px: bool,
        #[values(false, true)] pretty_ts: bool,
    ) {
        let v4 = mbp10_v4();
        assert_eq!(
            encode_v4(
                &metadata_v4(Schema::Mbp10),
                [RecordRef::from(&v4)],
                delimiter,
                pretty_px,
                pretty_ts
            )
            .unwrap(),
            encode_v3(&mbp10_v3(), delimiter, pretty_px, pretty_ts)
        );
    }

    #[rstest]
    fn symbol_mapping_with_required_symbols_matches_v3(#[values(false, true)] pretty_ts: bool) {
        let buf = symbol_mapping_v4();
        assert_eq!(
            encode_v4(
                &metadata_v4(Schema::Mbo),
                [as_ref(&buf)],
                b',',
                false,
                pretty_ts
            )
            .unwrap(),
            encode_v3(&symbol_mapping_v3(), b',', false, pretty_ts)
        );
    }

    #[rstest]
    #[case::sip(
        trades_with_sale_condition::<[Char; 4]>(),
        b"@\0T\t",
        "sale_condition_00,sale_condition_01,sale_condition_02,sale_condition_03",
        "@,,T,\\t"
    )]
    #[case::opra(trades_with_sale_condition::<Char>(), b"I", "sale_condition", "I")]
    fn sale_condition_follows_compiled_fields(
        #[case] metadata: Metadata,
        #[case] sale_condition: &[u8],
        #[case] expected_header: &str,
        #[case] expected: &str,
    ) {
        let buf = trade_v4_with_sale_condition(&metadata, sale_condition);
        let csv = encode_v4(&metadata, [as_ref(&buf)], b',', false, false).unwrap();
        let (header, row) = csv.split_once('\n').unwrap();
        assert!(
            header.starts_with("ts_recv,ts_event,rtype,publisher_id,instrument_id,"),
            "{header}"
        );
        assert!(
            header.ends_with(&format!(",sequence,{expected_header}")),
            "{header}"
        );
        assert!(row.ends_with(&format!(",88,{expected}\n")), "{row}");
    }

    #[test]
    fn omits_header() {
        let mbo = mbo_v4();
        let mut encoder = Encoder::builder(Vec::new(), &metadata_v4(Schema::Mbo))
            .write_header(false)
            .build()
            .unwrap();
        encoder.encode_record(&mbo).unwrap();
        let csv = String::from_utf8(encoder.into_inner().unwrap()).unwrap();
        let v3 = encode_v3(&mbo_v3(), b',', false, false);
        assert_eq!(csv, v3.split_once('\n').unwrap().1);
    }

    #[test]
    fn mixed_rtypes_error() {
        let mbo = mbo_v4();
        let trade = trade_v4();
        let res = encode_v4(
            &metadata_v4(Schema::Mbo),
            [RecordRef::from(&mbo), RecordRef::from(&trade)],
            b',',
            false,
            false,
        );
        assert!(
            matches!(&res, Err(Error::Encode(msg)) if msg.contains("Mixed rtypes")),
            "{res:?}"
        );
    }

    #[rstest]
    fn with_symbol_matches_v3(#[values(None, Some("ESZ6"))] symbol: Option<&str>) {
        let mut encoder = Encoder::builder(Vec::new(), &metadata_v4(Schema::Mbo))
            .with_symbol(true)
            .build()
            .unwrap();
        encoder.encode_record_with_sym(&mbo_v4(), symbol).unwrap();
        let mut v3_encoder = csv::Encoder::builder(Vec::new())
            .with_symbol(true)
            .build()
            .unwrap();
        v3_encoder
            .encode_record_with_sym(&mbo_v3(), symbol)
            .unwrap();
        assert_eq!(
            String::from_utf8(encoder.into_inner().unwrap()).unwrap(),
            String::from_utf8(v3_encoder.into_inner().unwrap()).unwrap()
        );
    }

    #[rstest]
    fn mismatched_symbol_method_errors(#[values(false, true)] with_symbol: bool) {
        let mbo = mbo_v4();
        let mut encoder = Encoder::builder(Vec::new(), &metadata_v4(Schema::Mbo))
            .with_symbol(with_symbol)
            .build()
            .unwrap();
        let res = if with_symbol {
            encoder.encode_record(&mbo)
        } else {
            encoder.encode_record_with_sym(&mbo, None)
        };
        assert!(
            matches!(&res, Err(Error::Encode(msg)) if msg.contains("with_symbol(true)")),
            "{res:?}"
        );
    }

    #[test]
    fn encode_header_matches_automatic_header() {
        let mbo = mbo_v4();
        let metadata = metadata_v4(Schema::Mbo);
        let mut encoder = Encoder::builder(Vec::new(), &metadata)
            .write_header(false)
            .build()
            .unwrap();
        encoder.encode_header(v4::rtype::MBO).unwrap();
        encoder.encode_record(&mbo).unwrap();
        assert_eq!(
            String::from_utf8(encoder.into_inner().unwrap()).unwrap(),
            encode_v4(&metadata, [RecordRef::from(&mbo)], b',', false, false).unwrap()
        );
    }

    #[test]
    fn encode_header_unknown_rtype_errors() {
        let mut encoder =
            Encoder::new(Vec::new(), &metadata_v4(Schema::Mbo), false, false).unwrap();
        let res = encoder.encode_header(0xFFFF);
        assert!(
            matches!(&res, Err(Error::Encode(msg)) if msg.contains("no layout")),
            "{res:?}"
        );
    }
}
