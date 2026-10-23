use crate::{
    layout::{FieldDef, RecordLayout, StreamLayout},
    v4::{
        fields::{self, FieldType},
        types::{Char, Decimal, TimestampNs},
    },
};

/// A [`StreamLayout`] resolved for encoding records of any of its rtypes.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedStreamLayout {
    // A stream declares < 25 rtypes at most, so linear scan beats hashing
    records: Vec<ResolvedLayout>,
}

/// A [`RecordLayout`] with nested struct offsets and array strides resolved, so each
/// scalar's offset is from the start of the record.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedLayout {
    pub rtype: u16,
    pub record_size: u16,
    /// In display order
    pub fields: Vec<ResolvedField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedField {
    pub name: String,
    pub value: ResolvedValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedValue {
    Scalar {
        kind: ScalarKind,
        offset: u16,
        size: u16,
    },
    Struct(Vec<ResolvedField>),
    Array(Vec<ResolvedValue>),
}

/// How a scalar is read and formatted. Enums and bit sets are their raw unsigned value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScalarKind {
    U8,
    U16,
    U32,
    U64,
    /// The `u64` instrument ID. JSON writes other 64-bit integers as strings, but writes
    /// this one as a number to match the `u32` instrument ID of v1-3. Parsers that read
    /// numbers as `f64` lose precision only past 2^53.
    InstrumentId,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Decimal,
    Timestamp,
    Char,
    Str,
}

impl ResolvedStreamLayout {
    pub fn new(layout: &StreamLayout) -> Option<Self> {
        let records = layout
            .record_layouts
            .iter()
            .map(|rec| ResolvedLayout::new(layout, rec))
            .collect::<Option<_>>()?;
        Some(Self { records })
    }

    pub fn get(&self, rtype: u16) -> Option<&ResolvedLayout> {
        self.records.iter().find(|rec| rec.rtype == rtype)
    }
}

impl ResolvedLayout {
    /// `layout` must be valid, e.g. through `validate_for_metadata`. Returns `None` if
    /// `rec` contains a field the encoders don't support.
    pub fn new(layout: &StreamLayout, rec: &RecordLayout) -> Option<Self> {
        Some(Self {
            rtype: rec.rtype,
            record_size: rec.record_size(),
            fields: resolve_fields(layout, &rec.field_defs, &rec.field_names, 0)?,
        })
    }
}

impl ScalarKind {
    /// `None` for a type the encoders don't support, including fixed-point scales other
    /// than -9.
    fn new(def: &FieldDef) -> Option<Self> {
        Some(match def.type_id {
            <u8 as FieldType>::TYPE_ID | fields::ENUM_8_ID | fields::BIT_SET_8_ID => Self::U8,
            <u16 as FieldType>::TYPE_ID | fields::ENUM_16_ID | fields::BIT_SET_16_ID => Self::U16,
            <u32 as FieldType>::TYPE_ID | fields::BIT_SET_32_ID => Self::U32,
            // Must precede the `u64` arm below
            <u64 as FieldType>::TYPE_ID if def.field_id == fields::INSTRUMENT_ID.id() => {
                Self::InstrumentId
            }
            <u64 as FieldType>::TYPE_ID | fields::BIT_SET_64_ID => Self::U64,
            <i8 as FieldType>::TYPE_ID => Self::I8,
            <i16 as FieldType>::TYPE_ID => Self::I16,
            <i32 as FieldType>::TYPE_ID => Self::I32,
            <i64 as FieldType>::TYPE_ID => Self::I64,
            <f32 as FieldType>::TYPE_ID => Self::F32,
            <f64 as FieldType>::TYPE_ID => Self::F64,
            <Decimal as FieldType>::TYPE_ID if def.scale == -9 => Self::Decimal,
            <TimestampNs as FieldType>::TYPE_ID => Self::Timestamp,
            <Char as FieldType>::TYPE_ID | fields::ENUM_CHAR_ID => Self::Char,
            fields::CSTR_ID | fields::SYMBOL_ID => Self::Str,
            _ => return None,
        })
    }
}

fn resolve_fields(
    layout: &StreamLayout,
    defs: &[FieldDef],
    names: &[String],
    base: u16,
) -> Option<Vec<ResolvedField>> {
    defs.iter()
        .zip(names)
        .filter(|(def, name)| !def.is_padding() && !name.is_empty())
        .map(|(def, name)| resolve_field(layout, def, name, base))
        .collect()
}

fn resolve_field(
    layout: &StreamLayout,
    def: &FieldDef,
    name: &str,
    base: u16,
) -> Option<ResolvedField> {
    let offset = base + def.offset;
    let nested = (def.type_id == fields::STRUCT_ID)
        .then(|| &layout.struct_layouts[usize::from(def.def_index)]);
    let elem_size = match nested {
        Some(sd) => sd.size,
        // Strings have no fixed width
        None => fields::scalar_size(def.type_id).unwrap_or(def.size),
    };
    let resolve_elem = |offset: u16| -> Option<ResolvedValue> {
        Some(match nested {
            Some(sd) => ResolvedValue::Struct(resolve_fields(
                layout,
                &sd.field_defs,
                &sd.field_names,
                offset,
            )?),
            None => ResolvedValue::Scalar {
                kind: ScalarKind::new(def)?,
                offset,
                size: elem_size,
            },
        })
    };
    let value = if def.is_array() {
        ResolvedValue::Array(
            (0..def.size / elem_size)
                .map(|i| resolve_elem(offset + i * elem_size))
                .collect::<Option<_>>()?,
        )
    } else {
        resolve_elem(offset)?
    };
    Some(ResolvedField {
        name: name.to_owned(),
        value,
    })
}

#[cfg(test)]
mod tests {
    use std::mem::{offset_of, size_of};

    use rstest::rstest;

    use super::*;
    use crate::{
        layout::StreamLayoutBuilder,
        v4::{self, layouts::default_layout, rtype},
        BidAskPair,
    };

    fn resolved(rtype: u16) -> ResolvedLayout {
        ResolvedStreamLayout::new(default_layout())
            .unwrap()
            .get(rtype)
            .unwrap()
            .clone()
    }

    fn named<'a>(fields: &'a [ResolvedField], target: &str) -> &'a ResolvedValue {
        &fields
            .iter()
            .find(|f| f.name == target)
            .unwrap_or_else(|| panic!("no field `{target}` in {fields:?}"))
            .value
    }

    fn scalar(value: &ResolvedValue) -> (ScalarKind, usize, usize) {
        let ResolvedValue::Scalar { kind, offset, size } = value else {
            panic!("{value:?} isn't a scalar");
        };
        (*kind, usize::from(*offset), usize::from(*size))
    }

    #[test]
    fn every_default_rtype_resolves() {
        let stream = ResolvedStreamLayout::new(default_layout()).unwrap();
        for rec in &default_layout().record_layouts {
            let resolved = stream.get(rec.rtype).unwrap();
            assert_eq!(resolved.record_size, rec.record_size());
        }
        assert!(stream.get(0xFFFF).is_none());
    }

    #[test]
    fn header_resolves_without_length() {
        let mbo = resolved(rtype::MBO);
        let ResolvedValue::Struct(fields) = named(&mbo.fields, "hd") else {
            panic!("`hd` isn't a struct");
        };
        let names: Vec<_> = fields.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(
            names,
            ["ts_event", "rtype", "publisher_id", "instrument_id"]
        );
        assert_eq!(
            scalar(named(fields, "ts_event")),
            (
                ScalarKind::Timestamp,
                offset_of!(v4::RecordHeader, ts_event),
                8,
            )
        );
        assert_eq!(
            scalar(named(fields, "instrument_id")).0,
            ScalarKind::InstrumentId
        );
    }

    #[rstest]
    #[case::price("price", ScalarKind::Decimal, offset_of!(v4::MboMsg, price), 8)]
    #[case::side("side", ScalarKind::Char, offset_of!(v4::MboMsg, side), 1)]
    #[case::flags("flags", ScalarKind::U8, offset_of!(v4::MboMsg, flags), 1)]
    #[case::ts_recv("ts_recv", ScalarKind::Timestamp, offset_of!(v4::MboMsg, ts_recv), 8)]
    fn scalars_resolve_to_struct_offsets(
        #[case] name: &str,
        #[case] kind: ScalarKind,
        #[case] offset: usize,
        #[case] size: usize,
    ) {
        let mbo = resolved(rtype::MBO);
        assert_eq!(scalar(named(&mbo.fields, name)), (kind, offset, size));
    }

    #[test]
    fn struct_array_elements_resolve_to_absolute_offsets() {
        let mbp10 = resolved(rtype::MBP_10);
        let ResolvedValue::Array(elems) = named(&mbp10.fields, "levels") else {
            panic!("`levels` isn't an array");
        };
        assert_eq!(elems.len(), 10);
        for (i, elem) in elems.iter().enumerate() {
            let ResolvedValue::Struct(fields) = elem else {
                panic!("{elem:?} isn't a struct");
            };
            let level = offset_of!(v4::Mbp10Msg, levels) + i * size_of::<BidAskPair>();
            assert_eq!(
                scalar(named(fields, "ask_sz")).1,
                level + offset_of!(BidAskPair, ask_sz)
            );
        }
    }

    #[test]
    fn required_symbol_resolves_past_the_base() {
        let layout = default_layout()
            .record_layout(rtype::INSTRUMENT_DEF)
            .unwrap();
        let def = resolved(rtype::INSTRUMENT_DEF);
        let (kind, offset, size) = scalar(named(&def.fields, "raw_symbol"));
        assert_eq!(kind, ScalarKind::Str);
        assert_eq!(size, crate::compat::SYMBOL_CSTR_LEN_V3);
        assert!(offset + size <= usize::from(layout.record_size()));
    }

    #[test]
    fn scalar_arrays_resolve_per_element() {
        let mut sb = StreamLayoutBuilder::new(1, 8);
        {
            let mut r = sb.record(rtype::MBP_0, 0, "TradeMsg");
            r.field(fields::PRICE, "price");
            r.finish_base();
            r.field(
                fields::Field::<[Char; 4]>::new(fields::SALE_CONDITION.id()),
                "sale_condition",
            );
            r.finish();
        }
        let layout = sb.build().unwrap();
        let rec = ResolvedStreamLayout::new(&layout).unwrap();
        let rec = rec.get(rtype::MBP_0).unwrap();
        let ResolvedValue::Array(elems) = named(&rec.fields, "sale_condition") else {
            panic!("`sale_condition` isn't an array");
        };
        let offsets: Vec<_> = elems.iter().map(|e| scalar(e).1).collect();
        let first = offsets[0];
        assert_eq!(offsets, [first, first + 1, first + 2, first + 3]);
        assert!(elems.iter().all(|e| scalar(e).0 == ScalarKind::Char));
    }

    #[test]
    fn unsupported_decimal_scale_errors() {
        let mut layout = default_layout().clone();
        let price = layout.record_layouts[0]
            .field_defs
            .iter_mut()
            .find(|f| f.type_id == <Decimal as FieldType>::TYPE_ID)
            .unwrap();
        price.scale = -4;
        assert!(ResolvedStreamLayout::new(&layout).is_none());
    }

    #[test]
    fn unsupported_type_id_errors() {
        let mut layout = default_layout().clone();
        let rec = &mut layout.record_layouts[0];
        let def = rec
            .field_defs
            .iter_mut()
            .find(|f| f.type_id == <u64 as FieldType>::TYPE_ID)
            .unwrap();
        def.type_id = 0x3F;
        assert!(ResolvedStreamLayout::new(&layout).is_none());
    }
}
