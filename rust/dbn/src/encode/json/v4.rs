//! Encoding DBN version 4 records into JSON lines.

use std::io;

use super::serialize::{to_json_in_buf, write_c_char, write_px, write_ts, JsonSerialize};
use crate::{
    encode::resolved_layout::{ResolvedField, ResolvedStreamLayout, ResolvedValue},
    json_writer::{JsonArrayWriter, JsonObjectWriter, JsonWriter, JsonWriterValue, Key, Slot},
    record::HasRType,
    v4::{
        self,
        fields::{ScalarValue, INSTRUMENT_ID},
    },
    Error, Metadata, RecordRef, Result,
};

/// Encodes DBN version 4 records into JSON lines, reading each record's fields from the
/// stream layout.
pub struct Encoder<W>
where
    W: io::Write,
{
    writer: W,
    buf: String,
    layout: ResolvedStreamLayout,
    should_pretty_print: bool,
    use_pretty_px: bool,
    use_pretty_ts: bool,
}

/// Helper for constructing a DBN version 4 JSON [`Encoder`].
pub struct EncoderBuilder<'a, W>
where
    W: io::Write,
{
    writer: W,
    metadata: &'a Metadata,
    should_pretty_print: bool,
    use_pretty_px: bool,
    use_pretty_ts: bool,
}

impl<'a, W> EncoderBuilder<'a, W>
where
    W: io::Write,
{
    /// Creates a new JSON encoder builder for records described by `metadata`'s layout.
    pub fn new(writer: W, metadata: &'a Metadata) -> Self {
        Self {
            writer,
            metadata,
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

    /// Creates the new encoder with the previously specified settings.
    ///
    /// # Errors
    /// This function returns an error if the metadata carries no stream layout, the
    /// layout is invalid, or it contains a field the encoder doesn't support.
    pub fn build(self) -> Result<Encoder<W>> {
        Ok(Encoder {
            writer: self.writer,
            buf: String::new(),
            layout: ResolvedStreamLayout::from_metadata(self.metadata)?,
            should_pretty_print: self.should_pretty_print,
            use_pretty_px: self.use_pretty_px,
            use_pretty_ts: self.use_pretty_ts,
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
    /// layout. If `should_pretty_print` is `true`, each JSON object will be nicely
    /// formatted and indented, instead of the default compact output with no whitespace
    /// between key-value pairs.
    ///
    /// # Errors
    /// This function returns an error if `metadata` carries no stream layout, the
    /// layout is invalid, or it contains a field the encoder doesn't support.
    pub fn new(
        writer: W,
        metadata: &Metadata,
        should_pretty_print: bool,
        use_pretty_px: bool,
        use_pretty_ts: bool,
    ) -> Result<Self> {
        Self::builder(writer, metadata)
            .should_pretty_print(should_pretty_print)
            .use_pretty_px(use_pretty_px)
            .use_pretty_ts(use_pretty_ts)
            .build()
    }

    /// Encodes `metadata` into JSON.
    ///
    /// # Errors
    /// This function returns an error if there's an error writing to `writer`.
    pub fn encode_metadata(&mut self, metadata: &Metadata) -> Result<()> {
        to_json_in_buf(
            &mut self.buf,
            metadata,
            self.should_pretty_print,
            self.use_pretty_px,
            self.use_pretty_ts,
        );
        let io_err = |e| Error::io(e, "writing metadata");
        self.write_buf(io_err)?;
        self.writer.flush().map_err(io_err)?;
        Ok(())
    }

    /// Encodes a single record.
    ///
    /// # Errors
    /// This function returns an error if the stream layout doesn't describe the
    /// record's rtype, the record is shorter than its layout, or it fails to write to
    /// the underlying writer.
    pub fn encode_record<R>(&mut self, record: &R) -> Result<()>
    where
        R: HasRType<Header = v4::RecordHeader>,
    {
        self.encode_record_ref(RecordRef::from(record))
    }

    /// Encodes a single record.
    ///
    /// # Errors
    /// This function returns an error if the stream layout doesn't describe the
    /// record's rtype, the record is shorter than its layout, or it fails to write to
    /// the underlying writer.
    pub fn encode_record_ref(&mut self, record: RecordRef<'_, v4::RecordHeader>) -> Result<()> {
        self.encode_to_buf(record)?;
        self.write_buf(|e| Error::io(e, "writing record"))
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
        &self.writer
    }

    /// Returns a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Writes to `self.buf`, but not the writer.
    fn encode_to_buf(&mut self, record: RecordRef<'_, v4::RecordHeader>) -> Result<()> {
        let layout = self.layout.layout_for(record)?;
        to_json_in_buf(
            &mut self.buf,
            &LayoutRecord {
                fields: &layout.fields,
                bytes: record.as_ref(),
            },
            self.should_pretty_print,
            self.use_pretty_px,
            self.use_pretty_ts,
        );
        Ok(())
    }

    fn write_buf<F>(&mut self, handle_err: F) -> Result<()>
    where
        F: FnOnce(io::Error) -> Error,
    {
        let res = self
            .writer
            .write_all(self.buf.as_bytes())
            .map_err(handle_err);
        // Always clear `buf`
        self.buf.clear();
        res
    }
}

/// A record's bytes paired with the resolved fields of its layout.
struct LayoutRecord<'a> {
    fields: &'a [ResolvedField],
    bytes: &'a [u8],
}

impl JsonSerialize for LayoutRecord<'_> {
    fn to_json<J: JsonWriter, const PRETTY_PX: bool, const PRETTY_TS: bool>(
        &self,
        writer: &mut JsonObjectWriter<J>,
    ) {
        write_fields::<J, PRETTY_PX, PRETTY_TS>(writer, self.fields, self.bytes);
    }
}

fn write_fields<J: JsonWriter, const PRETTY_PX: bool, const PRETTY_TS: bool>(
    object: &mut JsonObjectWriter<J>,
    fields: &[ResolvedField],
    bytes: &[u8],
) {
    for field in fields {
        write_value::<_, PRETTY_PX, PRETTY_TS>(
            &mut Key {
                object,
                key: &field.name,
            },
            &field.value,
            bytes,
        );
    }
}

fn write_value<S: Slot, const PRETTY_PX: bool, const PRETTY_TS: bool>(
    slot: &mut S,
    value: &ResolvedValue,
    bytes: &[u8],
) {
    match value {
        ResolvedValue::Scalar(scalar) => match scalar.read(bytes) {
            Some(value) => write_scalar::<_, PRETTY_PX, PRETTY_TS>(slot, scalar.field_id, value),
            None => "".write_json(slot.begin()),
        },
        ResolvedValue::Struct(fields) => write_fields::<_, PRETTY_PX, PRETTY_TS>(
            &mut JsonObjectWriter::new(slot.begin()),
            fields,
            bytes,
        ),
        ResolvedValue::Array(elems) => {
            let mut array = JsonArrayWriter::new(slot.begin());
            for elem in elems {
                write_value::<_, PRETTY_PX, PRETTY_TS>(&mut array, elem, bytes);
            }
        }
    }
}

/// Matches the v1-3 JSON encoding: 64-bit integers are strings to avoid a loss of
/// precision in JSON parsers, except `instrument_id`, which v1-3 encodes as a number.
/// Parsers that read numbers as `f64` lose precision only past 2^53.
fn write_scalar<S: Slot, const PRETTY_PX: bool, const PRETTY_TS: bool>(
    slot: &mut S,
    field_id: u16,
    value: ScalarValue<'_>,
) {
    match value {
        ScalarValue::U8(v) => v.write_json(slot.begin()),
        ScalarValue::U16(v) => v.write_json(slot.begin()),
        ScalarValue::U32(v) => v.write_json(slot.begin()),
        ScalarValue::U64(v) if field_id == INSTRUMENT_ID.id() => v.write_json(slot.begin()),
        ScalarValue::U64(v) => itoa::Buffer::new().format(v).write_json(slot.begin()),
        ScalarValue::I8(v) => v.write_json(slot.begin()),
        ScalarValue::I16(v) => v.write_json(slot.begin()),
        ScalarValue::I32(v) => v.write_json(slot.begin()),
        ScalarValue::I64(v) => itoa::Buffer::new().format(v).write_json(slot.begin()),
        ScalarValue::F32(v) => v.write_json(slot.begin()),
        ScalarValue::F64(v) => v.write_json(slot.begin()),
        ScalarValue::Decimal(px) => write_px::<_, PRETTY_PX>(slot, px.raw),
        ScalarValue::Timestamp(ts) => write_ts::<_, PRETTY_TS>(slot, ts.0),
        ScalarValue::Char(c) => write_c_char(slot, c.0),
        ScalarValue::Str(s) => s.write_json(slot.begin()),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::{
        decode::dbn::tests::{metadata_v4, trade_v4},
        encode::{json, EncodeRecord},
        test_utils::{
            as_ref, mbo_v3, mbo_v4, mbp10_v3, mbp10_v4, symbol_mapping_v3, symbol_mapping_v4,
            trade_v4_with_sale_condition, trades_with_sale_condition,
        },
        v4::{fields, rtype, types::Char},
        Schema,
    };

    fn encode_v4(
        metadata: &Metadata,
        record: RecordRef<'_, v4::RecordHeader>,
        flags: (bool, bool, bool),
    ) -> Result<String> {
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf, metadata, flags.0, flags.1, flags.2)?;
        encoder.encode_record_ref(record)?;
        Ok(String::from_utf8(buf).unwrap())
    }

    fn encode_v3<R: crate::encode::DbnEncodable>(record: &R, flags: (bool, bool, bool)) -> String {
        let mut buf = Vec::new();
        json::Encoder::new(&mut buf, flags.0, flags.1, flags.2)
            .encode_record(record)
            .unwrap();
        String::from_utf8(buf).unwrap()
    }

    fn parse(json: &str) -> serde_json::Value {
        serde_json::from_str(json).unwrap()
    }

    #[rstest]
    fn mbo_matches_v3(
        #[values(false, true)] pretty_print: bool,
        #[values(false, true)] pretty_px: bool,
        #[values(false, true)] pretty_ts: bool,
    ) {
        let flags = (pretty_print, pretty_px, pretty_ts);
        let v4 = mbo_v4();
        assert_eq!(
            encode_v4(&metadata_v4(Schema::Mbo), RecordRef::from(&v4), flags).unwrap(),
            encode_v3(&mbo_v3(), flags)
        );
    }

    #[rstest]
    fn mbp10_matches_v3(
        #[values(false, true)] pretty_print: bool,
        #[values(false, true)] pretty_px: bool,
        #[values(false, true)] pretty_ts: bool,
    ) {
        let flags = (pretty_print, pretty_px, pretty_ts);
        assert_eq!(
            encode_v4(
                &metadata_v4(Schema::Mbp10),
                RecordRef::from(&mbp10_v4()),
                flags
            )
            .unwrap(),
            encode_v3(&mbp10_v3(), flags)
        );
    }

    #[rstest]
    fn symbol_mapping_with_required_symbols_matches_v3(
        #[values(false, true)] pretty_print: bool,
        #[values(false, true)] pretty_ts: bool,
    ) {
        let flags = (pretty_print, false, pretty_ts);
        let buf = symbol_mapping_v4();
        assert_eq!(
            encode_v4(&metadata_v4(Schema::Mbo), as_ref(&buf), flags).unwrap(),
            encode_v3(&symbol_mapping_v3(), flags)
        );
    }

    #[rstest]
    fn trade_matches_v3_without_depth(
        #[values(false, true)] pretty_px: bool,
        #[values(false, true)] pretty_ts: bool,
    ) {
        let flags = (false, pretty_px, pretty_ts);
        let v4 = trade_v4();
        let v3 = crate::TradeMsg {
            hd: crate::RecordHeader::new::<crate::TradeMsg>(
                crate::rtype::MBP_0,
                v4.hd.publisher_id,
                v4.hd.instrument_id as u32,
                v4.hd.ts_event,
            ),
            price: v4.price,
            size: v4.size,
            action: v4.action,
            side: v4.side,
            flags: v4.flags,
            depth: 0,
            ts_recv: v4.ts_recv,
            ts_in_delta: v4.ts_in_delta,
            sequence: v4.sequence,
        };
        let v4_json =
            parse(&encode_v4(&metadata_v4(Schema::Trades), RecordRef::from(&v4), flags).unwrap());
        let mut v3_json = parse(&encode_v3(&v3, flags));
        // v4 trades drop `depth`, which is always 0
        v3_json.as_object_mut().unwrap().remove("depth").unwrap();
        assert_eq!(v4_json, v3_json);
    }

    #[rstest]
    #[case::sip(trades_with_sale_condition::<[Char; 4]>(), b"@\0T\t", r#"["@",null,"T","\t"]"#)]
    #[case::opra(trades_with_sale_condition::<Char>(), b"I", r#""I""#)]
    fn sale_condition_follows_compiled_fields(
        #[case] metadata: Metadata,
        #[case] sale_condition: &[u8],
        #[case] expected: &str,
    ) {
        let buf = trade_v4_with_sale_condition(&metadata, sale_condition);
        let json = encode_v4(&metadata, as_ref(&buf), (false, false, false)).unwrap();
        assert!(
            json.ends_with(&format!(
                "\"sequence\":88,\"sale_condition\":{expected}}}\n"
            )),
            "{json}"
        );
        assert!(
            json.starts_with(r#"{"ts_recv":"1697240529000000100","hd":{"#),
            "{json}"
        );
    }

    #[test]
    fn short_record_errors() {
        let trade = trade_v4();
        let res = encode_v4(
            &trades_with_sale_condition::<[Char; 4]>(),
            RecordRef::from(&trade),
            (false, false, false),
        );
        assert!(
            matches!(&res, Err(Error::Encode(msg)) if msg.contains("shorter")),
            "{res:?}"
        );
    }

    #[test]
    fn undeclared_rtype_errors() {
        let mbo = mbo_v4();
        // The default layout declares every compiled rtype, so drop MBO from it
        let mut metadata = metadata_v4(Schema::Mbo);
        metadata
            .layout
            .as_mut()
            .unwrap()
            .record_layouts
            .retain(|rec| rec.rtype != rtype::MBO);
        let missing = encode_v4(&metadata, RecordRef::from(&mbo), (false, false, false));
        assert!(
            matches!(&missing, Err(Error::Encode(msg)) if msg.contains("no layout")),
            "{missing:?}"
        );
    }

    #[test]
    fn metadata_without_layout_errors() {
        let mut metadata = metadata_v4(Schema::Mbo);
        metadata.layout = None;
        assert!(Encoder::new(Vec::new(), &metadata, false, false, false).is_err());
    }

    #[test]
    fn invalid_layout_errors() {
        let mut metadata = metadata_v4(Schema::Mbo);
        let rec = &mut metadata.layout.as_mut().unwrap().record_layouts[0];
        rec.field_defs.remove(1);
        rec.field_names.remove(1);
        assert!(matches!(
            Encoder::new(Vec::new(), &metadata, false, false, false),
            Err(Error::Layout(_))
        ));
    }

    #[test]
    fn unsupported_field_errors() {
        let mut metadata = metadata_v4(Schema::Mbo);
        let price = metadata.layout.as_mut().unwrap().record_layouts[0]
            .field_defs
            .iter_mut()
            .find(|f| f.field_id == fields::PRICE.id())
            .unwrap();
        price.scale = -4;
        assert!(matches!(
            Encoder::new(Vec::new(), &metadata, false, false, false),
            Err(Error::Layout(_))
        ));
    }

    #[test]
    fn encodes_v4_metadata() {
        let metadata = metadata_v4(Schema::Mbo);
        let mut buf = Vec::new();
        Encoder::new(&mut buf, &metadata, false, false, false)
            .unwrap()
            .encode_metadata(&metadata)
            .unwrap();
        assert!(String::from_utf8(buf)
            .unwrap()
            .contains(r#""dataset":"GLBX.MDP3""#));
    }
}
