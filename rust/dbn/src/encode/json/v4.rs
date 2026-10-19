//! Encoding DBN version 4 records into JSON lines.

use std::io;

use super::serialize::{to_json_in_buf, write_c_char, write_px, write_ts, JsonSerialize};
use crate::{
    encode::resolved_layout::{ResolvedField, ResolvedStreamLayout, ResolvedValue, ScalarKind},
    json_writer::{JsonArrayWriter, JsonObjectWriter, JsonWriter, JsonWriterValue, Key, Slot},
    layout::validate_for_metadata,
    record::HasRType,
    v4::{
        self,
        fields::DecodeField,
        types::{CStr, Char, Decimal, TimestampNs},
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

impl<W> Encoder<W>
where
    W: io::Write,
{
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
        let layout = metadata
            .layout
            .as_ref()
            .ok_or_else(|| Error::encode("metadata has no stream layout"))?;
        validate_for_metadata(layout, metadata.symbol_cstr_len as u16, metadata.ts_out)?;
        let layout = ResolvedStreamLayout::new(layout).ok_or_else(|| {
            Error::encode("stream layout has a field type the JSON encoder doesn't support")
        })?;
        Ok(Self {
            writer,
            buf: String::new(),
            layout,
            should_pretty_print,
            use_pretty_px,
            use_pretty_ts,
        })
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
        let rtype = record.header().rtype;
        let layout = self
            .layout
            .get(rtype)
            .ok_or_else(|| Error::encode(format!("no layout for rtype {rtype:#06X}")))?;
        let bytes = record.as_ref();
        if bytes.len() < usize::from(layout.record_size) {
            return Err(Error::encode("record is shorter than its layout"));
        }
        to_json_in_buf(
            &mut self.buf,
            &LayoutRecord {
                fields: &layout.fields,
                bytes,
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
        ResolvedValue::Scalar { kind, offset, size } => {
            write_scalar::<_, PRETTY_PX, PRETTY_TS>(slot, *kind, field_bytes(bytes, *offset, *size))
        }
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

/// The caller checks the record holds its layout's `record_size`, which bounds every
/// resolved field.
fn field_bytes(bytes: &[u8], offset: u16, size: u16) -> &[u8] {
    let start = usize::from(offset);
    &bytes[start..start + usize::from(size)]
}

/// Matches the v1-3 JSON encoding: 64-bit integers are strings to avoid a loss of
/// precision in JSON parsers, except `instrument_id`, which v1-3 encodes as a number.
fn write_scalar<S: Slot, const PRETTY_PX: bool, const PRETTY_TS: bool>(
    slot: &mut S,
    kind: ScalarKind,
    bytes: &[u8],
) {
    // Every resolved scalar's window is exactly its type's width, so only a string
    // can fail to decode. The scale is always -9, which only `Decimal` reads, and the
    // raw value suffices
    fn read<T: for<'a> DecodeField<Value<'a> = T>>(bytes: &[u8]) -> T {
        T::decode(bytes, 0).expect("scalar window matches its type width")
    }
    match kind {
        ScalarKind::U8 => read::<u8>(bytes).write_json(slot.begin()),
        ScalarKind::U16 => read::<u16>(bytes).write_json(slot.begin()),
        ScalarKind::U32 => read::<u32>(bytes).write_json(slot.begin()),
        ScalarKind::U64 => itoa::Buffer::new()
            .format(read::<u64>(bytes))
            .write_json(slot.begin()),
        ScalarKind::InstrumentId => read::<u64>(bytes).write_json(slot.begin()),
        ScalarKind::I8 => read::<i8>(bytes).write_json(slot.begin()),
        ScalarKind::I16 => read::<i16>(bytes).write_json(slot.begin()),
        ScalarKind::I32 => read::<i32>(bytes).write_json(slot.begin()),
        ScalarKind::I64 => itoa::Buffer::new()
            .format(read::<i64>(bytes))
            .write_json(slot.begin()),
        ScalarKind::F32 => read::<f32>(bytes).write_json(slot.begin()),
        ScalarKind::F64 => read::<f64>(bytes).write_json(slot.begin()),
        ScalarKind::Decimal => write_px::<_, PRETTY_PX>(slot, read::<Decimal>(bytes).raw),
        ScalarKind::Timestamp => write_ts::<_, PRETTY_TS>(slot, read::<TimestampNs>(bytes).0),
        ScalarKind::Char => write_c_char(slot, read::<Char>(bytes).0),
        ScalarKind::Str => CStr::decode(bytes, 0)
            .unwrap_or_default()
            .write_json(slot.begin()),
    }
}

#[cfg(test)]
mod tests {
    use std::os::raw::c_char;

    use rstest::rstest;

    use super::*;
    use crate::{
        decode::dbn::tests::{metadata_v4, trade_v4},
        encode::{json, EncodeRecord},
        layout::{RecordLayoutDesc, StreamLayoutBuilder},
        test_utils::{mbo_v3, mbo_v4},
        v4::{fields, rtype, types::Char},
        BidAskPair, FlagSet, SType, Schema,
    };

    const SYMBOL_LEN: usize = crate::compat::SYMBOL_CSTR_LEN_V3;

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

    /// The record's bytes in `u64`s so a `RecordRef` to them is aligned for the header.
    fn aligned(bytes: &[u8]) -> Vec<u64> {
        bytes
            .chunks(8)
            .map(|chunk| u64::from_le_bytes(chunk.try_into().unwrap()))
            .collect()
    }

    fn as_ref(buf: &[u64]) -> RecordRef<'_, v4::RecordHeader> {
        // SAFETY: `buf` holds a whole record starting with an aligned v4 header whose
        // `length` covers the rest of `buf`.
        unsafe { RecordRef::unchecked_from_header(buf.as_ptr().cast()) }
    }

    fn levels() -> [BidAskPair; 10] {
        std::array::from_fn(|i| BidAskPair {
            bid_px: 5_000_000_000 - i as i64 * 250_000_000,
            ask_px: if i == 9 {
                crate::UNDEF_PRICE
            } else {
                5_250_000_000 + i as i64 * 250_000_000
            },
            bid_sz: 10 + i as u32,
            ask_sz: 20 + i as u32,
            bid_ct: i as u32,
            ask_ct: 2 * i as u32,
        })
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
        let v4 = v4::Mbp10Msg {
            hd: v4::RecordHeader::new::<v4::Mbp10Msg>(
                rtype::MBP_10,
                1,
                5482,
                1_697_240_529_000_000_000,
            ),
            price: crate::UNDEF_PRICE,
            size: 3,
            action: b'A' as c_char,
            side: 0,
            flags: FlagSet::empty().set_last(),
            depth: 4,
            ts_recv: crate::UNDEF_TIMESTAMP,
            ts_in_delta: -5,
            sequence: 12,
            levels: levels(),
        };
        let v3 = crate::Mbp10Msg {
            hd: crate::RecordHeader::new::<crate::Mbp10Msg>(
                crate::rtype::MBP_10,
                1,
                5482,
                1_697_240_529_000_000_000,
            ),
            price: v4.price,
            size: v4.size,
            action: v4.action,
            side: v4.side,
            flags: v4.flags,
            depth: v4.depth,
            ts_recv: v4.ts_recv,
            ts_in_delta: v4.ts_in_delta,
            sequence: v4.sequence,
            levels: levels(),
        };
        assert_eq!(
            encode_v4(&metadata_v4(Schema::Mbp10), RecordRef::from(&v4), flags).unwrap(),
            encode_v3(&v3, flags)
        );
    }

    #[rstest]
    fn symbol_mapping_with_required_symbols_matches_v3(
        #[values(false, true)] pretty_print: bool,
        #[values(false, true)] pretty_ts: bool,
    ) {
        let flags = (pretty_print, false, pretty_ts);
        let mapping = v4::SymbolMappingMsg {
            hd: v4::RecordHeader::new::<v4::SymbolMappingMsg>(rtype::SYMBOL_MAPPING, 0, 5482, 7),
            start_ts: 1_697_155_200_000_000_000,
            end_ts: 1_697_241_600_000_000_000,
            stype_in: SType::RawSymbol as u8,
            stype_out: SType::InstrumentId as u8,
            _reserved: [0; 6],
        };
        let layout = v4::layouts::default_layout();
        let rec = layout.record_layout(rtype::SYMBOL_MAPPING).unwrap();
        let mut bytes = vec![0; usize::from(rec.record_size())];
        bytes[..size_of::<v4::SymbolMappingMsg>()]
            .copy_from_slice(RecordRef::from(&mapping).as_ref());
        // v4 `length` counts the bytes after the header
        let length = u16::try_from(bytes.len() - size_of::<v4::RecordHeader>()).unwrap();
        bytes[..2].copy_from_slice(&length.to_le_bytes());
        for (field, symbol) in [
            (fields::STYPE_IN_SYMBOL, "ESZ6"),
            (fields::STYPE_OUT_SYMBOL, "5482"),
        ] {
            let def = rec
                .field_defs
                .iter()
                .find(|f| f.field_id == field.id())
                .unwrap();
            let start = usize::from(def.offset);
            bytes[start..start + symbol.len()].copy_from_slice(symbol.as_bytes());
        }
        let buf = aligned(&bytes);
        let v3 = crate::SymbolMappingMsg::new(
            5482,
            7,
            SType::RawSymbol,
            "ESZ6",
            SType::InstrumentId,
            "5482",
            mapping.start_ts,
            mapping.end_ts,
        )
        .unwrap();
        assert_eq!(
            encode_v4(&metadata_v4(Schema::Mbo), as_ref(&buf), flags).unwrap(),
            encode_v3(&v3, flags)
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

    fn trades_with_conditions<T: fields::FixedWidth>() -> Metadata {
        let mut sb = StreamLayoutBuilder::new(1, SYMBOL_LEN as u16);
        {
            let mut r = sb.record(rtype::MBP_0, Schema::Trades as u16, v4::TradeMsg::NAME);
            v4::TradeMsg::add_fields(&mut r);
            r.ext_field(
                fields::Field::<T>::new(fields::CONDITIONS.id()),
                "conditions",
            );
            r.finish();
        }
        let mut metadata = metadata_v4(Schema::Trades);
        metadata.layout = Some(sb.build().unwrap());
        metadata
    }

    #[rstest]
    #[case::sip(trades_with_conditions::<[Char; 4]>(), b"@\0T\t", r#"["@",null,"T","\t"]"#)]
    #[case::opra(trades_with_conditions::<Char>(), b"I", r#""I""#)]
    fn conditions_follow_compiled_fields(
        #[case] metadata: Metadata,
        #[case] conditions: &[u8],
        #[case] expected: &str,
    ) {
        let rec = metadata.layout.as_ref().unwrap().record_layouts[0].clone();
        let trade = trade_v4();
        let mut bytes = vec![0; usize::from(rec.record_size())];
        bytes[..size_of::<v4::TradeMsg>()].copy_from_slice(RecordRef::from(&trade).as_ref());
        let length = u16::try_from(bytes.len() - size_of::<v4::RecordHeader>()).unwrap();
        bytes[..2].copy_from_slice(&length.to_le_bytes());
        let start = size_of::<v4::TradeMsg>();
        bytes[start..start + conditions.len()].copy_from_slice(conditions);
        let buf = aligned(&bytes);

        let json = encode_v4(&metadata, as_ref(&buf), (false, false, false)).unwrap();
        assert!(
            json.ends_with(&format!("\"sequence\":88,\"conditions\":{expected}}}\n")),
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
            &trades_with_conditions::<[Char; 4]>(),
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
            Err(Error::Encode(_))
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
