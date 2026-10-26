//! The wire form of a [`StreamLayout`].

use std::collections::BTreeMap;

use super::{
    builders::FieldDefBuilder,
    validate::{check_compiled_rtypes, validate},
};
use crate::{
    layout::{FieldDef, LabelDef, RecordLayout, StreamLayout, StructLayout},
    Error,
};

const FIELD_DEF_LEN: usize = 16;
const _: () = assert!(FIELD_DEF_LEN == std::mem::size_of::<FieldDef>());

/// Serializes `layout` onto the end of `out`.
///
/// # Errors
/// Returns [`Error::Layout`] if a count exceeds `u16::MAX`, a section exceeds
/// `u16::MAX` bytes, or a field-def and field-name list disagree in length, and
/// [`Error::Encode`] if a name exceeds 255 bytes.
pub fn encode(out: &mut Vec<u8>, layout: &StreamLayout) -> crate::Result<()> {
    out.extend_from_slice(&layout.stream_layout_version.to_le_bytes());
    encode_count(out, "record layouts", layout.record_layouts.len())?;
    for rec in &layout.record_layouts {
        encode_record_layout(out, rec)?;
    }
    encode_count(out, "struct layouts", layout.struct_layouts.len())?;
    for sd in &layout.struct_layouts {
        encode_struct_layout(out, sd)?;
    }
    encode_count(out, "label defs", layout.label_defs.len())?;
    for ld in &layout.label_defs {
        encode_label_def(out, ld)?;
    }
    Ok(())
}

pub(crate) fn encoded_len(layout: &StreamLayout) -> usize {
    const SECTION_LEN: usize = size_of::<u16>();
    const COUNT_LEN: usize = size_of::<u16>();
    let fields_len = |defs: &[FieldDef], names: &[String]| {
        COUNT_LEN + defs.len() * FIELD_DEF_LEN + strs_len(names)
    };
    let records = layout.record_layouts.iter().map(|rec| {
        SECTION_LEN
            + size_of_val(&rec.rtype)
            + size_of_val(&rec.schema)
            + size_of_val(&rec.base_record_size)
            + size_of_val(&rec.layout_hash)
            + str_len(&rec.name)
            + fields_len(&rec.field_defs, &rec.field_names)
    });
    let structs = layout.struct_layouts.iter().map(|sd| {
        SECTION_LEN
            + size_of_val(&sd.size)
            + str_len(&sd.name)
            + fields_len(&sd.field_defs, &sd.field_names)
    });
    let labels = layout.label_defs.iter().map(|ld| {
        SECTION_LEN
            + str_len(&ld.name)
            + COUNT_LEN
            + ld.labels
                .iter()
                .map(|(code, name)| size_of_val(code) + str_len(name))
                .sum::<usize>()
    });
    size_of_val(&layout.stream_layout_version)
        + 3 * COUNT_LEN
        + records.sum::<usize>()
        + structs.sum::<usize>()
        + labels.sum::<usize>()
}

/// Parses and validates a [`StreamLayout`] from `bytes`.
///
/// Bytes past the last label def are ignored.
///
/// # Errors
/// Returns [`Error::Layout`] on a malformed table: a length or count running past its
/// enclosing section, a duplicate label code, a repacked base under a compiled record's
/// rtype, or any other validation error. Returns [`Error::Decode`] on truncated input or
/// a name that isn't NUL-terminated UTF-8.
pub fn decode(bytes: &[u8], symbol_cstr_len: u16) -> crate::Result<StreamLayout> {
    let mut cur = ByteReader::new(bytes);
    let stream_layout_version = cur.u16("stream layout")?;
    let mut record_layouts = Vec::new();
    for _ in 0..cur.u16("stream layout")? {
        record_layouts.push(decode_record_layout(&mut cur)?);
    }
    let mut struct_layouts = Vec::new();
    for _ in 0..cur.u16("stream layout")? {
        struct_layouts.push(decode_struct_layout(&mut cur)?);
    }
    let mut label_defs = Vec::new();
    for _ in 0..cur.u16("stream layout")? {
        label_defs.push(decode_label_def(&mut cur)?);
    }
    let layout = StreamLayout {
        stream_layout_version,
        record_layouts,
        struct_layouts,
        label_defs,
    };
    validate(&layout, symbol_cstr_len)?;
    check_layout_hashes(&layout)?;
    check_compiled_rtypes(&layout)?;
    Ok(layout)
}

fn check_layout_hashes(layout: &StreamLayout) -> crate::Result<()> {
    for rec in &layout.record_layouts {
        let computed = rec.layout_hash(&layout.struct_layouts);
        if rec.layout_hash != computed {
            return Err(Error::layout(format!(
                "record `{}`: declared layout_hash {:#018x} does not match the field table ({computed:#018x})",
                rec.name, rec.layout_hash
            )));
        }
    }
    Ok(())
}

fn encode_record_layout(out: &mut Vec<u8>, rec: &RecordLayout) -> crate::Result<()> {
    section(out, "record layout", |out| {
        out.extend_from_slice(&rec.rtype.to_le_bytes());
        out.extend_from_slice(&rec.schema.to_le_bytes());
        out.extend_from_slice(&rec.base_record_size.to_le_bytes());
        out.extend_from_slice(&rec.layout_hash.to_le_bytes());
        encode_str(out, "record layout", &rec.name)?;
        encode_fields(out, "record layout", &rec.field_defs, &rec.field_names)
    })
}

fn decode_record_layout(cur: &mut ByteReader<'_>) -> crate::Result<RecordLayout> {
    const CTX: &str = "record layout";
    let mut sec = cur.section(CTX)?;
    let rtype = sec.u16(CTX)?;
    let schema = sec.u16(CTX)?;
    let base_record_size = sec.u16(CTX)?;
    let layout_hash = sec.u64(CTX)?;
    let name = sec.str(CTX)?;
    let (field_defs, field_names) = decode_fields(&mut sec, CTX)?;
    Ok(RecordLayout {
        rtype,
        schema,
        base_record_size,
        layout_hash,
        name,
        field_defs,
        field_names,
    })
}

fn encode_struct_layout(out: &mut Vec<u8>, sd: &StructLayout) -> crate::Result<()> {
    section(out, "struct layout", |out| {
        out.extend_from_slice(&sd.size.to_le_bytes());
        encode_str(out, "struct layout", &sd.name)?;
        encode_fields(out, "struct layout", &sd.field_defs, &sd.field_names)
    })
}

fn decode_struct_layout(cur: &mut ByteReader<'_>) -> crate::Result<StructLayout> {
    const CTX: &str = "struct layout";
    let mut sec = cur.section(CTX)?;
    let struct_size = sec.u16(CTX)?;
    let name = sec.str(CTX)?;
    let (fields, field_names) = decode_fields(&mut sec, CTX)?;
    Ok(StructLayout {
        size: struct_size,
        name,
        field_defs: fields,
        field_names,
    })
}

fn encode_label_def(out: &mut Vec<u8>, ld: &LabelDef) -> crate::Result<()> {
    section(out, "label def", |out| {
        encode_str(out, "label def", &ld.name)?;
        encode_count(out, "labels", ld.labels.len())?;
        for (code, name) in &ld.labels {
            out.extend_from_slice(&code.to_le_bytes());
            encode_str(out, "label", name)?;
        }
        Ok(())
    })
}

fn decode_label_def(cur: &mut ByteReader<'_>) -> crate::Result<LabelDef> {
    const CTX: &str = "label def";
    let mut sec = cur.section(CTX)?;
    let name = sec.str(CTX)?;
    let mut labels = BTreeMap::new();
    for _ in 0..sec.u16(CTX)? {
        let code = sec.u16(CTX)?;
        if labels.insert(code, sec.str(CTX)?).is_some() {
            return Err(Error::layout(format!(
                "label def `{name}`: duplicate label code {code}"
            )));
        }
    }
    Ok(LabelDef { name, labels })
}

fn encode_fields(
    out: &mut Vec<u8>,
    ctx: &str,
    defs: &[FieldDef],
    names: &[String],
) -> crate::Result<()> {
    if defs.len() != names.len() {
        return Err(Error::layout(format!(
            "{ctx}: {} field defs but {} field names",
            defs.len(),
            names.len()
        )));
    }
    encode_count(out, ctx, defs.len())?;
    for f in defs {
        out.extend_from_slice(&f.field_id.to_le_bytes());
        out.extend_from_slice(&f.offset.to_le_bytes());
        out.extend_from_slice(&f.size.to_le_bytes());
        out.push(f.type_id);
        out.push(f.flags);
        out.extend_from_slice(&f.def_index.to_le_bytes());
        out.push(f.scale as u8);
        out.extend_from_slice(&f._reserved);
    }
    for name in names {
        encode_str(out, ctx, name)?;
    }
    Ok(())
}

fn decode_fields(
    cur: &mut ByteReader<'_>,
    ctx: &str,
) -> crate::Result<(Vec<FieldDef>, Vec<String>)> {
    let count = cur.u16(ctx)?;
    let mut defs = Vec::new();
    for _ in 0..count {
        let mut f = ByteReader::new(cur.take(ctx, FIELD_DEF_LEN)?);
        let field_id = f.u16(ctx)?;
        let offset = f.u16(ctx)?;
        let size = f.u16(ctx)?;
        let type_id = f.u8(ctx)?;
        let flags = f.u8(ctx)?;
        let def_index = f.u16(ctx)?;
        let scale = f.i8(ctx)?;
        defs.push(
            FieldDefBuilder::new(field_id, type_id)
                .offset(offset)
                .size(size)
                .flags(flags)
                .def_index(def_index)
                .scale(scale)
                .build(),
        );
    }
    let mut names = Vec::new();
    for _ in 0..count {
        names.push(cur.str(ctx)?);
    }
    Ok((defs, names))
}

fn section(
    out: &mut Vec<u8>,
    ctx: &str,
    body: impl FnOnce(&mut Vec<u8>) -> crate::Result<()>,
) -> crate::Result<()> {
    let start = out.len();
    out.extend_from_slice(&0u16.to_le_bytes());
    body(out)?;
    let len = u16::try_from(out.len() - start)
        .map_err(|_| Error::layout(format!("{ctx} exceeds {} bytes", u16::MAX)))?;
    out[start..start + 2].copy_from_slice(&len.to_le_bytes());
    Ok(())
}

/// Appends `s` as a length-prefixed string: a `u8` byte length, the UTF-8 bytes, and a
/// NUL the length doesn't count.
pub(crate) fn encode_str(out: &mut Vec<u8>, ctx: &str, s: &str) -> crate::Result<()> {
    let len = u8::try_from(s.len())
        .map_err(|_| Error::encode(format!("{ctx}: `{s}` exceeds {} bytes", u8::MAX)))?;
    out.push(len);
    out.extend_from_slice(s.as_bytes());
    out.push(0);
    Ok(())
}

/// The encoded length of `s` as a length-prefixed string.
pub(crate) fn str_len(s: &str) -> usize {
    let (len_byte, nul) = (1, 1);
    len_byte + s.len() + nul
}

pub(crate) fn strs_len(strs: &[String]) -> usize {
    strs.iter().map(|s| str_len(s)).sum()
}

fn encode_count(out: &mut Vec<u8>, ctx: &str, n: usize) -> crate::Result<()> {
    let n = u16::try_from(n).map_err(|_| Error::layout(format!("more than {} {ctx}", u16::MAX)))?;
    out.extend_from_slice(&n.to_le_bytes());
    Ok(())
}

pub(crate) struct ByteReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub fn take(&mut self, ctx: &str, n: usize) -> crate::Result<&'a [u8]> {
        let end = self.pos.checked_add(n).ok_or_else(|| truncated(ctx))?;
        let out = self.buf.get(self.pos..end).ok_or_else(|| truncated(ctx))?;
        self.pos = end;
        Ok(out)
    }

    pub fn remaining(&self) -> &'a [u8] {
        &self.buf[self.pos..]
    }

    pub fn u8(&mut self, ctx: &str) -> crate::Result<u8> {
        Ok(self.take(ctx, 1)?[0])
    }

    pub fn i8(&mut self, ctx: &str) -> crate::Result<i8> {
        Ok(self.u8(ctx)? as i8)
    }

    pub fn u16(&mut self, ctx: &str) -> crate::Result<u16> {
        let mut bytes = [0; 2];
        bytes.copy_from_slice(self.take(ctx, 2)?);
        Ok(u16::from_le_bytes(bytes))
    }

    pub fn u32(&mut self, ctx: &str) -> crate::Result<u32> {
        let mut bytes = [0; 4];
        bytes.copy_from_slice(self.take(ctx, 4)?);
        Ok(u32::from_le_bytes(bytes))
    }

    pub fn u64(&mut self, ctx: &str) -> crate::Result<u64> {
        let mut bytes = [0; 8];
        bytes.copy_from_slice(self.take(ctx, 8)?);
        Ok(u64::from_le_bytes(bytes))
    }

    /// Reads a length-prefixed string written by [`encode_str`].
    pub fn str(&mut self, ctx: &str) -> crate::Result<String> {
        let len = self.u8(ctx)? as usize;
        let bytes = self.take(ctx, len)?;
        if bytes.contains(&0) {
            return Err(Error::decode(format!("{ctx}: string contains a NUL byte")));
        }
        let s = std::str::from_utf8(bytes)
            .map_err(|_| Error::decode(format!("{ctx}: string is not UTF-8")))?
            .to_owned();
        if self.u8(ctx)? != 0 {
            return Err(Error::decode(format!(
                "{ctx}: string `{s}` is not NUL-terminated"
            )));
        }
        Ok(s)
    }

    fn section(&mut self, ctx: &str) -> crate::Result<Self> {
        let len = self.u16(ctx)?;
        let body = len.checked_sub(2).ok_or_else(|| {
            Error::layout(format!(
                "{ctx}: length {len} is shorter than its own prefix"
            ))
        })?;
        Ok(ByteReader::new(self.take(ctx, body.into())?))
    }
}

fn truncated(ctx: &str) -> crate::Error {
    crate::Error::decode(format!("{ctx}: ended mid-field"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        layout::{RecordLayoutDesc, StreamLayoutBuilder},
        v4::{
            self,
            fields::{self, Field},
            layouts::default_layout,
            rtype,
            types::Decimal,
        },
        Schema,
    };

    const SYM_LEN: u16 = crate::compat::SYMBOL_CSTR_LEN_V3 as u16;
    const FIRST_SECTION: usize = 4;
    const FIRST_NAME: usize = FIRST_SECTION + 2 + 2 + 2 + 2 + 8;
    const FIRST_HASH: usize = FIRST_NAME - 8;

    fn round_trip(layout: &StreamLayout) -> StreamLayout {
        let mut buf = Vec::new();
        encode(&mut buf, layout).unwrap();
        decode(&buf, SYM_LEN).unwrap()
    }

    fn encoded_default() -> Vec<u8> {
        let mut buf = Vec::new();
        encode(&mut buf, default_layout()).unwrap();
        buf
    }

    fn dataset_layout() -> StreamLayout {
        let mut sb = StreamLayoutBuilder::new(7, SYM_LEN);
        sb.label(LabelDef {
            name: "Side".to_owned(),
            labels: BTreeMap::from([
                (u16::from(b'A'), "ASK".to_owned()),
                (u16::from(b'B'), "BID".to_owned()),
            ]),
        });
        sb.layout_of::<v4::MboMsg>(rtype::MBO).finish();
        {
            let mut r = sb.record(rtype::MBP_0, Schema::Trades as u16, v4::TradeMsg::NAME);
            v4::TradeMsg::add_fields(&mut r);
            r.finish_base();
            r.field(fields::TS_OUT, "ts_out");
            r.finish();
        }
        sb.build().unwrap()
    }

    #[rstest::rstest]
    #[case::default(default_layout().clone())]
    #[case::dataset(dataset_layout())]
    fn encoded_len_matches_encode(#[case] layout: StreamLayout) {
        let mut buf = Vec::new();
        encode(&mut buf, &layout).unwrap();
        assert_eq!(encoded_len(&layout), buf.len());
    }

    #[test]
    fn default_layout_round_trips() {
        assert_eq!(&round_trip(default_layout()), default_layout());
    }

    #[test]
    fn dataset_layout_round_trips() {
        let layout = dataset_layout();
        assert_eq!(round_trip(&layout), layout);
    }

    #[test]
    fn encode_appends_to_the_buffer() {
        let mut buf = vec![0xFF; 3];
        encode(&mut buf, default_layout()).unwrap();
        assert_eq!(&buf[..3], &[0xFF; 3]);
        assert_eq!(&decode(&buf[3..], SYM_LEN).unwrap(), default_layout());
    }

    #[test]
    fn every_truncation_errors() {
        let buf = encoded_default();
        for len in 0..buf.len() {
            assert!(
                decode(&buf[..len], SYM_LEN).is_err(),
                "{len} of {} bytes decoded",
                buf.len()
            );
        }
    }

    #[test]
    fn unknown_trailing_section_bytes_are_skipped() {
        let mut buf = encoded_default();
        let len = u16::from_le_bytes([buf[FIRST_SECTION], buf[FIRST_SECTION + 1]]) as usize;
        let extra = [0xAAu8; 6];
        buf.splice(
            FIRST_SECTION + len..FIRST_SECTION + len,
            extra.iter().copied(),
        );
        let grown = u16::try_from(len + extra.len()).unwrap();
        buf[FIRST_SECTION..FIRST_SECTION + 2].copy_from_slice(&grown.to_le_bytes());

        assert_eq!(&decode(&buf, SYM_LEN).unwrap(), default_layout());
    }

    #[test]
    fn a_section_shorter_than_its_contents_errors() {
        let mut buf = encoded_default();
        let len = u16::from_le_bytes([buf[FIRST_SECTION], buf[FIRST_SECTION + 1]]);
        buf[FIRST_SECTION..FIRST_SECTION + 2].copy_from_slice(&(len - 8).to_le_bytes());
        assert!(decode(&buf, SYM_LEN).is_err());
    }

    #[test]
    fn an_overstated_count_errors() {
        let mut buf = encoded_default();
        buf[2..4].copy_from_slice(&u16::MAX.to_le_bytes());
        assert!(decode(&buf, SYM_LEN).is_err());
    }

    #[test]
    fn an_overstated_name_length_errors() {
        let mut buf = encoded_default();
        buf[FIRST_NAME] = u8::MAX;
        assert!(decode(&buf, SYM_LEN).is_err());
    }

    #[test]
    fn a_name_without_its_nul_errors() {
        let mut buf = encoded_default();
        let len = buf[FIRST_NAME] as usize;
        buf[FIRST_NAME + 1 + len] = b'!';
        assert!(decode(&buf, SYM_LEN).is_err());
    }

    #[test]
    fn a_name_with_an_interior_nul_errors() {
        let mut buf = encoded_default();
        buf[FIRST_NAME + 1] = 0;
        assert!(decode(&buf, SYM_LEN).is_err());
    }

    #[test]
    fn a_declared_hash_the_field_table_contradicts_errors() {
        let mut buf = encoded_default();
        buf[FIRST_HASH] ^= 1;
        assert!(decode(&buf, SYM_LEN).is_err());
    }

    #[test]
    fn a_repacked_compiled_rtype_errors() {
        let mut sb = StreamLayoutBuilder::new(1, SYM_LEN);
        let mut r = sb.record(rtype::MBP_0, Schema::Trades as u16, "TradeMsg");
        r.field(fields::PRICE, "price")
            .field(Field::<Decimal>::new(fields::SIZE.id()), "size")
            .field(fields::TS_RECV, "ts_recv")
            .field(fields::TS_IN_DELTA, "ts_in_delta")
            .field(fields::SEQUENCE, "sequence");
        r.finish();
        let mut encoded = Vec::new();
        encode(&mut encoded, &sb.build().unwrap()).unwrap();

        let err = decode(&encoded, SYM_LEN).unwrap_err();
        assert!(err.to_string().contains("repacked"), "{err}");
    }

    #[test]
    fn a_duplicate_label_code_errors() {
        let mut buf = Vec::new();
        encode(&mut buf, &dataset_layout()).unwrap();
        // `B` followed by its encoded `BID` label
        let bid = [b'B', 0, 3, b'B', b'I', b'D', 0];
        let second = buf.windows(bid.len()).position(|w| w == bid).unwrap();
        buf[second..second + 2].copy_from_slice(&u16::from(b'A').to_le_bytes());
        assert!(decode(&buf, SYM_LEN).is_err());
    }

    #[test]
    fn a_parseable_but_invalid_table_errors() {
        let mut layout = default_layout().clone();
        layout.record_layouts[1].rtype = layout.record_layouts[0].rtype;
        let mut buf = Vec::new();
        encode(&mut buf, &layout).unwrap();
        assert!(decode(&buf, SYM_LEN).is_err());
    }

    #[rstest::rstest]
    #[case::empty("")]
    #[case::ascii("SPY   260116C00500000")]
    #[case::multibyte("ÄÖÜ")]
    #[case::max_len(&"x".repeat(255))]
    fn a_str_round_trips(#[case] s: &str) {
        let mut buf = Vec::new();
        encode_str(&mut buf, "test", s).unwrap();
        assert_eq!(buf.len(), str_len(s));
        let mut cur = ByteReader::new(&buf);
        assert_eq!(cur.str("test").unwrap(), s);
        assert!(cur.remaining().is_empty());
    }

    #[test]
    fn a_name_over_255_bytes_is_an_encode_error() {
        let mut layout = default_layout().clone();
        layout.record_layouts[0].name = "x".repeat(256);
        assert!(encode(&mut Vec::new(), &layout).is_err());
    }

    #[test]
    fn mismatched_field_def_and_name_counts_are_an_encode_error() {
        let mut layout = default_layout().clone();
        layout.record_layouts[0].field_names.pop();
        assert!(encode(&mut Vec::new(), &layout).is_err());
    }
}
