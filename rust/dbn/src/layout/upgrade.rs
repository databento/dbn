use super::pre_v4;
use crate::{
    layout::{FieldDef, RecordLayout, StreamLayout},
    v3,
    v4::{
        self,
        fields::{self, FieldDefault, FieldType},
    },
    Schema,
};

pub(crate) fn upgrade_layout(ts_out: bool) -> StreamLayout {
    let v3 = pre_v4::stream_layout(3, false);
    v4::layouts::compiled_layout(|rtype, r| {
        r.finish_base();
        for (f, name) in v3_only_fields(&v3.record_layouts, rtype) {
            r.copy_field(f, name, &v3.label_defs);
        }
        if ts_out {
            r.field(fields::TS_OUT, "ts_out");
        }
    })
}

fn v3_only_fields(v3: &[RecordLayout], rtype: u16) -> Vec<(&FieldDef, &str)> {
    let (Some(v3), Some(compiled)) = (
        v3.iter().find(|rec| rec.rtype == rtype),
        v4::layouts::default_layout().record_layout(rtype),
    ) else {
        return Vec::new();
    };
    let mut missing: Vec<_> = v3
        .field_defs
        .iter()
        .zip(v3.field_names.iter().map(String::as_str))
        .filter(|(f, _)| {
            !f.is_padding() && compiled.field_defs.iter().all(|c| c.field_id != f.field_id)
        })
        .collect();
    missing.sort_by_key(|(f, _)| f.offset);
    missing
}

pub struct Upgrader {
    plans: Vec<UpgradePlan>,
    unplanned_rtypes: Vec<u16>,
}

impl Upgrader {
    pub(crate) fn new(version: u8, dst: &StreamLayout, schema: Option<Schema>) -> Self {
        let schema_rec =
            schema.and_then(|s| dst.record_layouts.iter().find(|rec| rec.schema == s as u16));
        let ts_out = dst
            .record_layouts
            .iter()
            .any(|rec| rec.has_field(fields::TS_OUT));
        let mut plans = Vec::new();
        let mut unplanned_rtypes = Vec::new();
        for src in pre_v4::record_layouts(version, ts_out) {
            if let Some(to) = schema_rec
                .filter(|rec| rec.name == src.name)
                .or_else(|| dst.record_layout(src.rtype))
            {
                plans.push(UpgradePlan::new(&src, to))
            } else {
                unplanned_rtypes.push(src.rtype)
            }
        }
        Self {
            plans,
            unplanned_rtypes,
        }
    }

    // Returns the upgraded record's size, or `None` if `dst` is too short to hold it
    pub(crate) fn upgrade(&self, src: &[u8], dst: &mut [u8]) -> crate::Result<Option<usize>> {
        assert!(src.len() >= size_of::<v3::RecordHeader>());
        // SAFETY: `src` holds a header, and any bytes are a valid header of integers
        let hd = unsafe { src.as_ptr().cast::<v3::RecordHeader>().read_unaligned() };
        let src = &src[..hd.record_size()];
        let rtype = u16::from(hd.rtype);
        let plan = self.plans.iter().find(|plan| plan.src_rtype == rtype);
        if plan.is_none() && self.unplanned_rtypes.contains(&rtype) {
            return Err(crate::Error::decode(format!(
                "can't upgrade rtype {rtype:#04x} to DBN version 4 without the stream's schema"
            )));
        }
        let record_size = plan.map_or(
            size_of::<v4::RecordHeader>() + src.len() - size_of::<v3::RecordHeader>(),
            UpgradePlan::record_size,
        );
        let Some(dst) = dst.get_mut(..record_size) else {
            return Ok(None);
        };
        if let Some(plan) = plan {
            plan.apply(src, dst)
        } else {
            // Without a layout to upgrade from, keep the body as encoded
            dst[size_of::<v4::RecordHeader>()..]
                .copy_from_slice(&src[size_of::<v3::RecordHeader>()..])
        }
        let upgraded_hd = v4::RecordHeader::with_record_size(
            plan.map_or(rtype, |plan| plan.rtype),
            hd.publisher_id,
            u64::from(hd.instrument_id),
            hd.ts_event,
            record_size,
        )?;
        // SAFETY: `dst` holds `record_size` bytes, which includes the header
        unsafe {
            dst.as_mut_ptr()
                .cast::<v4::RecordHeader>()
                .write_unaligned(upgraded_hd)
        };
        Ok(Some(record_size))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Copy {
        src_offset: u16,
        dst_offset: u16,
        size: u16,
    },
    Int {
        src_offset: u16,
        dst_offset: u16,
        src_size: u16,
        dst_size: u16,
        signed: bool,
        has_undef_max: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct UpgradePlan {
    src_rtype: u16,
    rtype: u16,
    ops: Vec<Op>,
    defaults: Box<[u8]>,
}

impl UpgradePlan {
    pub fn new(src: &RecordLayout, dst: &RecordLayout) -> Self {
        let header_len = size_of::<v4::RecordHeader>();
        let mut defaults = vec![0; usize::from(dst.record_size()) - header_len].into_boxed_slice();
        let mut ops = Vec::new();
        for want in dst
            .field_defs
            .iter()
            .filter(|f| !f.is_padding() && f.field_id != fields::HD.id())
        {
            let start = usize::from(want.offset) - header_len;
            fields::default_of(want.field_id, want.type_id).write(
                want.type_id,
                &mut defaults[start..start + usize::from(want.size)],
            );
            if let Some(op) = src
                .field_defs
                .iter()
                .find(|have| have.field_id == want.field_id)
                .and_then(|have| op_for(have, want))
            {
                ops.push(op);
            }
        }
        ops.sort_unstable_by_key(Op::dst_offset);
        Self {
            src_rtype: src.rtype,
            rtype: dst.rtype,
            ops: coalesce(ops, src.base_record_size, dst.base_record_size),
            defaults,
        }
    }

    fn record_size(&self) -> usize {
        size_of::<v4::RecordHeader>() + self.defaults.len()
    }

    pub fn apply(&self, src: &[u8], dst: &mut [u8]) {
        dst[size_of::<v4::RecordHeader>()..].copy_from_slice(&self.defaults);
        for op in &self.ops {
            match *op {
                Op::Copy {
                    src_offset,
                    dst_offset,
                    size,
                } => {
                    let Some(from) = window(src, src_offset, size) else {
                        continue;
                    };
                    dst[usize::from(dst_offset)..][..from.len()].copy_from_slice(from);
                }
                Op::Int {
                    src_offset,
                    dst_offset,
                    src_size,
                    dst_size,
                    signed,
                    has_undef_max,
                } => {
                    let Some(from) = window(src, src_offset, src_size) else {
                        continue;
                    };
                    let to = &mut dst[usize::from(dst_offset)..][..usize::from(dst_size)];
                    let value = read_int(from, signed);
                    let (_, src_max) = int_range(src_size, signed);
                    let (dst_min, dst_max) = int_range(dst_size, signed);
                    let value = if has_undef_max
                        && (value == src_max || !(dst_min..=dst_max).contains(&value))
                    {
                        dst_max
                    } else {
                        value
                    };
                    to.copy_from_slice(&value.to_le_bytes()[..to.len()]);
                }
            }
        }
    }
}

impl Op {
    fn dst_offset(&self) -> u16 {
        match *self {
            Self::Copy { dst_offset, .. } | Self::Int { dst_offset, .. } => dst_offset,
        }
    }
}

fn op_for(have: &FieldDef, want: &FieldDef) -> Option<Op> {
    if have.size == want.size
        && have.type_id == want.type_id
        && have.scale == want.scale
        && have.is_array() == want.is_array()
    {
        return Some(Op::Copy {
            src_offset: have.offset,
            dst_offset: want.offset,
            size: want.size,
        });
    }
    if have.scale != want.scale || have.is_array() || want.is_array() {
        return None;
    }
    let is_str = |type_id| matches!(type_id, fields::CSTR_ID | fields::SYMBOL_ID);
    if is_str(have.type_id) && is_str(want.type_id) {
        return (have.size <= want.size).then_some(Op::Copy {
            src_offset: have.offset,
            dst_offset: want.offset,
            size: have.size,
        });
    }
    let signed = int_signedness(have.type_id)?;
    if int_signedness(want.type_id)? != signed {
        return None;
    }
    let undef_max = fields::default_of(want.field_id, want.type_id) == FieldDefault::Max;
    (want.size > have.size || undef_max).then_some(Op::Int {
        src_offset: have.offset,
        dst_offset: want.offset,
        src_size: have.size,
        dst_size: want.size,
        signed,
        has_undef_max: undef_max,
    })
}

fn int_signedness(type_id: u8) -> Option<bool> {
    if [u8::TYPE_ID, u16::TYPE_ID, u32::TYPE_ID, u64::TYPE_ID].contains(&type_id) {
        Some(false)
    } else if [i8::TYPE_ID, i16::TYPE_ID, i32::TYPE_ID, i64::TYPE_ID].contains(&type_id) {
        Some(true)
    } else {
        None
    }
}

fn read_int(bytes: &[u8], signed: bool) -> i128 {
    let mut buf = [0; 16];
    buf[..bytes.len()].copy_from_slice(bytes);
    let value = i128::from_le_bytes(buf);
    if signed {
        let shift = 128 - 8 * bytes.len() as u32;
        (value << shift) >> shift
    } else {
        value
    }
}

fn int_range(size: u16, signed: bool) -> (i128, i128) {
    let bits = 8 * u32::from(size);
    if signed {
        (-(1 << (bits - 1)), (1 << (bits - 1)) - 1)
    } else {
        (0, (1 << bits) - 1)
    }
}

fn coalesce(ops: Vec<Op>, src_base: u16, dst_base: u16) -> Vec<Op> {
    let in_base = |src_offset: u16, dst_offset: u16, size: u16| {
        src_offset + size <= src_base && dst_offset + size <= dst_base
    };
    let mut out: Vec<Op> = Vec::with_capacity(ops.len());
    for op in ops {
        if let (
            Some(Op::Copy {
                src_offset: prev_src,
                dst_offset: prev_dst,
                size: prev_size,
            }),
            Op::Copy {
                src_offset,
                dst_offset,
                size,
            },
        ) = (out.last_mut(), op)
        {
            if in_base(*prev_src, *prev_dst, *prev_size)
                && in_base(src_offset, dst_offset, size)
                && *prev_dst + *prev_size == dst_offset
                && *prev_src + *prev_size == src_offset
            {
                *prev_size += size;
                continue;
            }
        }
        out.push(op);
    }
    out
}

fn window(bytes: &[u8], offset: u16, size: u16) -> Option<&[u8]> {
    let start = offset as usize;
    bytes.get(start..start + size as usize)
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;
    use crate::{
        layout::{LayoutIndex, RecordLayoutBuilder, RecordLayoutDesc, StreamLayoutBuilder},
        v4::{
            self,
            fields::{self, Field, FixedWidth},
            rtype,
            types::Decimal,
        },
    };

    const HEADER_LEN: usize = size_of::<v4::RecordHeader>();

    impl UpgradePlan {
        pub(crate) fn writes_start_of(&self, field: &FieldDef) -> bool {
            self.ops.iter().any(|op| {
                let (offset, size) = match *op {
                    Op::Copy {
                        dst_offset, size, ..
                    } => (dst_offset, size),
                    Op::Int {
                        dst_offset,
                        dst_size,
                        ..
                    } => (dst_offset, dst_size),
                };
                offset <= field.offset && field.offset < offset + size
            })
        }
    }

    fn record_of(declare: impl FnOnce(&mut RecordLayoutBuilder<'_>)) -> RecordLayout {
        let mut sb = StreamLayoutBuilder::new(1, crate::compat::SYMBOL_CSTR_LEN_V3 as u16);
        {
            let mut r = sb.record(rtype::MBP_0, 0, "TradeMsg");
            declare(&mut r);
            r.finish();
        }
        sb.build()
            .unwrap()
            .record_layouts
            .pop()
            .expect("one record")
    }

    fn offset_of_field(layout: &RecordLayout, field_id: u16) -> usize {
        layout
            .field_defs
            .iter()
            .find(|f| f.field_id == field_id)
            .expect("declared field")
            .offset as usize
    }

    fn apply_to_fresh(plan: &UpgradePlan, src: &[u8], dst_size: usize) -> Vec<u8> {
        let mut dst = vec![0xAA; dst_size];
        plan.apply(src, &mut dst);
        dst
    }

    fn trades_with_channel_id() -> RecordLayout {
        record_of(|r| {
            v4::TradeMsg::add_fields(r);
            r.field(fields::CHANNEL_ID, "channel_id");
        })
    }

    #[test]
    fn append_only_growth_is_one_memcpy() {
        let src = record_of(v4::MboMsg::add_fields);
        let dst = record_of(|r| {
            v4::MboMsg::add_fields(r);
            r.field(fields::DEPTH, "depth");
        });
        let plan = UpgradePlan::new(&src, &dst);

        let body = HEADER_LEN as u16;
        assert_eq!(
            plan.ops,
            [Op::Copy {
                src_offset: body,
                dst_offset: body,
                size: src.base_record_size - body,
            }]
        );
    }

    #[test]
    fn a_field_the_source_lacks_keeps_the_destination_default() {
        let src = record_of(v4::TradeMsg::add_fields);
        let dst = trades_with_channel_id();
        let plan = UpgradePlan::new(&src, &dst);

        let price = 4_215_500_000_000i64.to_le_bytes();
        let mut bytes = vec![0u8; src.base_record_size as usize];
        let at = offset_of_field(&src, fields::PRICE.id());
        bytes[at..at + 8].copy_from_slice(&price);
        let out = apply_to_fresh(&plan, &bytes, dst.base_record_size as usize);

        let at = offset_of_field(&dst, fields::PRICE.id());
        assert_eq!(out[at..at + 8], price);
        assert_eq!(out[offset_of_field(&dst, fields::CHANNEL_ID.id())], u8::MAX);
    }

    #[rstest]
    #[case::narrowed_cstr(
        |r: &mut RecordLayoutBuilder<'_>| { r.str_field(Field::new(fields::SALE_CONDITION.id()), "symbol", 71); },
        |r: &mut RecordLayoutBuilder<'_>| { r.str_field(Field::new(fields::SALE_CONDITION.id()), "symbol", crate::compat::SYMBOL_CSTR_LEN_V1 as u16); },
    )]
    #[case::rescaled(
        |r: &mut RecordLayoutBuilder<'_>| { r.field(fields::SIZE, "size"); },
        |r: &mut RecordLayoutBuilder<'_>| { r.field(Field::<Decimal>::new(fields::SIZE.id()), "size"); },
    )]
    #[case::sign_changed(
        |r: &mut RecordLayoutBuilder<'_>| { r.field(Field::<i32>::new(fields::SEQUENCE.id()), "sequence"); },
        |r: &mut RecordLayoutBuilder<'_>| { r.field(fields::SEQUENCE, "sequence"); },
    )]
    #[case::narrowed_without_a_sentinel(
        |r: &mut RecordLayoutBuilder<'_>| { r.field(Field::<u64>::new(fields::SEQUENCE.id()), "sequence"); },
        |r: &mut RecordLayoutBuilder<'_>| { r.field(fields::SEQUENCE, "sequence"); },
    )]
    fn a_reshaped_field_yields_no_op(
        #[case] src: fn(&mut RecordLayoutBuilder<'_>),
        #[case] dst: fn(&mut RecordLayoutBuilder<'_>),
    ) {
        let plan = UpgradePlan::new(&record_of(src), &record_of(dst));
        assert!(plan.ops.is_empty());
    }

    fn resize<S: FixedWidth, D: FixedWidth>(src: Field<S>, dst: Field<D>, value: i128) -> i128 {
        let src = record_of(|r| {
            r.field(src, "f");
        });
        let dst = record_of(|r| {
            r.field(dst, "f");
        });
        let plan = UpgradePlan::new(&src, &dst);
        let (src_def, dst_def) = (&src.field_defs[1], &dst.field_defs[1]);
        let mut bytes = vec![0; src.base_record_size as usize];
        let at = src_def.offset as usize;
        bytes[at..at + src_def.size as usize]
            .copy_from_slice(&value.to_le_bytes()[..src_def.size as usize]);
        let out = apply_to_fresh(&plan, &bytes, dst.base_record_size as usize);
        let at = dst_def.offset as usize;
        read_int(
            &out[at..at + dst_def.size as usize],
            int_signedness(dst_def.type_id).unwrap(),
        )
    }

    #[rstest]
    #[case::value(5, 5)]
    #[case::negative(-5, -5)]
    #[case::sentinel(i32::MAX.into(), i64::MAX.into())]
    fn quantity_widens_with_its_sentinel(#[case] value: i128, #[case] expected: i128) {
        let src = Field::<i32>::new(fields::QUANTITY.id());
        assert_eq!(resize(src, fields::QUANTITY, value), expected);
    }

    #[test]
    fn raw_instrument_id_zero_extends() {
        let src = Field::<u32>::new(fields::RAW_INSTRUMENT_ID.id());
        let max = u32::MAX.into();
        assert_eq!(resize(src, fields::RAW_INSTRUMENT_ID, max), max);
    }

    #[rstest]
    #[case::value(3, 3)]
    #[case::sentinel(u16::MAX.into(), u8::MAX.into())]
    #[case::too_wide(300, u8::MAX.into())]
    fn channel_id_narrows_to_its_sentinel(#[case] value: i128, #[case] expected: i128) {
        assert_eq!(
            resize(fields::CHANNEL_ID_V1, fields::CHANNEL_ID, value),
            expected
        );
    }

    #[test]
    fn a_symbol_widens_with_nulls() {
        let src = record_of(|r| {
            r.str_field(
                fields::RAW_SYMBOL_V1,
                "raw_symbol",
                crate::compat::SYMBOL_CSTR_LEN_V1 as u16,
            );
        });
        let dst = record_of(|r| {
            r.symbol_field(fields::RAW_SYMBOL, "raw_symbol");
        });
        let plan = UpgradePlan::new(&src, &dst);
        let (src_def, dst_def) = (&src.field_defs[1], &dst.field_defs[1]);
        let mut bytes = vec![0; src.base_record_size as usize];
        let at = src_def.offset as usize;
        bytes[at..at + 4].copy_from_slice(b"ESZ5");
        let out = apply_to_fresh(&plan, &bytes, dst.base_record_size as usize);
        let at = dst_def.offset as usize;
        let mut expected = vec![0; dst_def.size as usize];
        expected[..4].copy_from_slice(b"ESZ5");
        assert_eq!(out[at..at + dst_def.size as usize], expected);
    }

    #[test]
    fn a_source_shorter_than_its_base_is_skipped() {
        let src = record_of(v4::TradeMsg::add_fields);
        let dst = trades_with_channel_id();
        let plan = UpgradePlan::new(&src, &dst);

        let out = apply_to_fresh(&plan, &[0x11; 32], dst.base_record_size as usize);

        assert!(out[..HEADER_LEN].iter().all(|&b| b == 0xAA));
        assert_eq!(out[HEADER_LEN..], *plan.defaults);
    }

    #[test]
    fn an_absent_extension_field_does_not_cost_the_base() {
        let with_ts_out = |r: &mut RecordLayoutBuilder<'_>| {
            v4::MboMsg::add_fields(r);
            r.finish_base();
            r.field(fields::TS_OUT, "ts_out");
        };
        let src = record_of(with_ts_out);
        let dst = record_of(with_ts_out);
        let plan = UpgradePlan::new(&src, &dst);
        assert_eq!(plan.ops.len(), 2);

        let base = src.base_record_size as usize;
        let out = apply_to_fresh(&plan, &vec![0x11; base], base + 8);

        assert!(out[HEADER_LEN..base].iter().all(|&b| b == 0x11));
        assert_eq!(out[base..], u64::MAX.to_le_bytes());
    }

    #[rstest]
    fn upgrader_plans_every_default_rtype(#[values(1, 2, 3)] version: u8) {
        let dst = v4::layouts::default_layout();
        let upgrader = Upgrader::new(version, dst, None);
        for rec in &dst.record_layouts {
            assert!(
                upgrader
                    .plans
                    .iter()
                    .any(|plan| plan.src_rtype == rec.rtype),
                "{}",
                rec.name
            );
        }
    }

    #[test]
    fn upgraded_records_fit_a_record_buf() {
        for rec in &upgrade_layout(true).record_layouts {
            assert!(usize::from(rec.record_size()) <= crate::MAX_RECORD_LEN);
        }
    }

    // `def_index` locates a label def in each stream's own table, so compare the label
    // def's name instead
    fn named_fields<'a>(
        layout: &'a StreamLayout,
        rec: &RecordLayout,
    ) -> Vec<(FieldDef, Option<&'a str>)> {
        let mut defs: Vec<_> = rec
            .field_defs
            .iter()
            .map(|&f| {
                if fields::uses_label_def(f.type_id) && f.has_label_def() {
                    let label = layout.label_defs[usize::from(f.def_index)].name.as_str();
                    (FieldDef { def_index: 0, ..f }, Some(label))
                } else {
                    (f, None)
                }
            })
            .collect();
        defs.sort_by_key(|(f, _)| f.offset);
        defs
    }

    #[test]
    fn the_upgrade_layout_only_adds_extensions() {
        let upgraded = upgrade_layout(false);
        let default = v4::layouts::default_layout();
        assert_eq!(upgraded.struct_layouts, default.struct_layouts);
        assert_eq!(upgraded.record_layouts.len(), default.record_layouts.len());
        let mut extended = Vec::new();
        for (u, d) in upgraded.record_layouts.iter().zip(&default.record_layouts) {
            assert_eq!(u.base_record_size, d.base_record_size);
            let (u_fields, d_fields) = (named_fields(&upgraded, u), named_fields(default, d));
            let is_base = |(f, _): &&(FieldDef, Option<&str>)| f.offset < d.base_record_size;
            assert!(u_fields
                .iter()
                .filter(is_base)
                .eq(d_fields.iter().filter(is_base)));
            if u_fields != d_fields {
                extended.push(u.rtype);
            }
        }
        assert_eq!(
            extended,
            [
                rtype::MBP_0,
                rtype::MBP_1,
                rtype::TBBO,
                rtype::INSTRUMENT_DEF,
                rtype::IMBALANCE
            ]
        );
    }

    #[test]
    fn the_upgrade_layout_labels_v3_only_fields() {
        let upgraded = upgrade_layout(false);
        let index = LayoutIndex::new(&upgraded, v4::layouts::default_layout());
        let field = index
            .field_by_name(rtype::INSTRUMENT_DEF, "user_defined_instrument")
            .unwrap();
        assert!(field.def().is_extension());
        assert_eq!(field.label_def().unwrap().name, "UserDefinedInstrument");
    }

    fn upgraded_rtype(version: u8, schema: Option<Schema>, src: &[u8]) -> crate::Result<u16> {
        let upgrader = Upgrader::new(version, v4::layouts::default_layout(), schema);
        let mut dst = [0; v4::MAX_RECORD_LEN];
        upgrader.upgrade(src, &mut dst)?.unwrap();
        // SAFETY: `upgrade()` wrote a header to the start of `dst`
        let hd = unsafe { dst.as_ptr().cast::<v4::RecordHeader>().read_unaligned() };
        Ok(hd.rtype)
    }

    fn ohlcv_deprecated() -> v3::OhlcvMsg {
        let mut rec = v3::OhlcvMsg::default_for_schema(Schema::Ohlcv1M);
        #[allow(deprecated)]
        {
            rec.hd.rtype = crate::rtype::OHLCV_DEPRECATED;
        }
        rec
    }

    #[rstest]
    #[case::tbbo(Some(Schema::Tbbo), rtype::TBBO)]
    #[case::mbp_1(Some(Schema::Mbp1), rtype::MBP_1)]
    #[case::no_schema(None, rtype::MBP_1)]
    fn an_mbp_1_record_takes_the_schemas_rtype(
        #[values(1, 2, 3)] version: u8,
        #[case] schema: Option<Schema>,
        #[case] expected: u16,
    ) {
        let src = v3::Mbp1Msg::default();
        assert_eq!(
            upgraded_rtype(version, schema, src.as_ref()).unwrap(),
            expected
        );
    }

    #[rstest]
    #[case::ohlcv_1s(Schema::Ohlcv1S, rtype::OHLCV_1S)]
    #[case::ohlcv_1m(Schema::Ohlcv1M, rtype::OHLCV_1M)]
    #[case::ohlcv_1h(Schema::Ohlcv1H, rtype::OHLCV_1H)]
    #[case::ohlcv_1d(Schema::Ohlcv1D, rtype::OHLCV_1D)]
    fn an_ohlcv_deprecated_record_takes_the_schemas_rtype(
        #[case] schema: Schema,
        #[case] expected: u16,
    ) {
        let src = ohlcv_deprecated();
        assert_eq!(
            upgraded_rtype(1, Some(schema), src.as_ref()).unwrap(),
            expected
        );
    }

    #[test]
    fn an_ohlcv_deprecated_record_needs_a_schema() {
        let err = upgraded_rtype(1, None, ohlcv_deprecated().as_ref()).unwrap_err();
        assert!(
            err.to_string().contains("without the stream's schema"),
            "{err}"
        );
    }

    #[test]
    fn the_plan_never_touches_the_header() {
        let src = record_of(v4::TradeMsg::add_fields);
        let dst = trades_with_channel_id();
        let plan = UpgradePlan::new(&src, &dst);

        let bytes = vec![0x11; src.base_record_size as usize];
        let out = apply_to_fresh(&plan, &bytes, dst.base_record_size as usize);
        assert!(out[..HEADER_LEN].iter().all(|&b| b == 0xAA));
    }
}
