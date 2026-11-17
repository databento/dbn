use crate::{
    layout::{hash::by_offset, FieldDef, RecordLayout, StreamLayout, StructLayout, Verdict},
    v4::fields,
};

pub(crate) fn verdict(
    stream: &StreamLayout,
    stream_rec: &RecordLayout,
    compiled: &StreamLayout,
) -> Verdict {
    let Some(compiled_rec) = compiled.record_layout(stream_rec.rtype) else {
        return Verdict::Dynamic;
    };
    if stream_rec.layout_hash == compiled_rec.layout_hash
        || is_base_compatible(stream, stream_rec, compiled, compiled_rec)
    {
        Verdict::Compiled
    } else {
        Verdict::Dynamic
    }
}

fn is_base_compatible(
    stream: &StreamLayout,
    stream_rec: &RecordLayout,
    compiled: &StreamLayout,
    compiled_rec: &RecordLayout,
) -> bool {
    base_fields(compiled_rec).all(|want| {
        stream_rec
            .field_defs
            .iter()
            .any(|have| field_eq(have, stream, want, compiled))
    })
}

pub(crate) fn is_repacked(
    stream: &StreamLayout,
    stream_rec: &RecordLayout,
    compiled: &StreamLayout,
    compiled_rec: &RecordLayout,
) -> bool {
    base_fields(compiled_rec).any(|want| {
        stream_rec
            .field_defs
            .iter()
            .find(|have| have.field_id == want.field_id)
            .is_some_and(|have| !field_eq(have, stream, want, compiled))
    })
}

fn base_fields(compiled_rec: &RecordLayout) -> impl Iterator<Item = &FieldDef> {
    // Required fields aren't part of the struct, and their width varies by stream
    let required = crate::v4::layouts::required_fields(compiled_rec.rtype);
    compiled_rec
        .field_defs
        .iter()
        .filter(|f| !f.is_padding() && !required.iter().any(|(r, _)| r.id() == f.field_id))
}

fn field_eq(a: &FieldDef, a_layout: &StreamLayout, b: &FieldDef, b_layout: &StreamLayout) -> bool {
    if !eq_except_def_index(a, b) {
        return false;
    }
    if a.type_id != fields::STRUCT_ID {
        return true;
    }
    match (
        a_layout.struct_layouts.get(a.def_index as usize),
        b_layout.struct_layouts.get(b.def_index as usize),
    ) {
        (Some(x), Some(y)) => struct_eq(x, y),
        _ => false,
    }
}

fn eq_except_def_index(a: &FieldDef, b: &FieldDef) -> bool {
    a.field_id == b.field_id
        && a.offset == b.offset
        && a.size == b.size
        && a.type_id == b.type_id
        && a.flags == b.flags
        && a.scale == b.scale
}

fn struct_eq(a: &StructLayout, b: &StructLayout) -> bool {
    a.size == b.size
        && a.field_defs.len() == b.field_defs.len()
        && by_offset(&a.field_defs)
            .into_iter()
            .zip(by_offset(&b.field_defs))
            .all(|(x, y)| eq_except_def_index(x, y))
}
