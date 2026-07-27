use std::collections::HashSet;

use crate::{
    layout::{FieldDef, StreamLayout, StructLayout},
    v4::{fields, RecordHeader},
};

/// Validates a [`StreamLayout`] before any field access, rejecting a malformed table
/// as a decode error.
pub(crate) fn validate(layout: &StreamLayout, _symbol_cstr_len: u16) -> crate::Result<()> {
    for sd in &layout.struct_layouts {
        let align = sd.align();
        if !sd.size.is_multiple_of(align) {
            return Err(err(
                &sd.name,
                &format!(
                    "size {} is not a multiple of its {align}-byte alignment",
                    sd.size
                ),
            ));
        }
        check_fields(
            &sd.name,
            &sd.field_defs,
            &sd.field_names,
            sd.size,
            false,
            layout,
        )?;
    }
    let header_size = std::mem::size_of::<RecordHeader>() as u16;
    let mut rtypes = HashSet::new();
    for rec in &layout.record_layouts {
        if !rtypes.insert(rec.rtype) {
            return Err(err(
                &rec.name,
                &format!("duplicate rtype {:#06X}", rec.rtype),
            ));
        }
        if rec.rtype & 0x00FF == u16::from(b'N') {
            return Err(err(
                &rec.name,
                &format!(
                    "rtype {:#06X} is reserved: its low byte matches the metadata magic",
                    rec.rtype
                ),
            ));
        }
        if rec.base_record_size < header_size {
            return Err(err(&rec.name, "base_record_size smaller than the header"));
        }
        if rec.base_record_size % 8 != 0 {
            return Err(err(&rec.name, "base_record_size not a multiple of 8"));
        }
        // The header is `field_defs[0]`, so a record's coverage starts at 0 like a
        // struct's rather than past the header.
        check_fields(
            &rec.name,
            &rec.field_defs,
            &rec.field_names,
            rec.base_record_size,
            true,
            layout,
        )?;
    }
    Ok(())
}

/// Checks alignment, bounds, and coverage for one field sequence.
///
/// The non-extension fields (including padding) must tile `[0, base)` with no gap
/// or overlap; extension fields must be contiguous and ascending from `base`.
fn check_fields(
    ctx: &str,
    defs: &[FieldDef],
    names: &[String],
    base: u16,
    is_record: bool,
    layout: &StreamLayout,
) -> crate::Result<()> {
    if defs.len() != names.len() {
        return Err(err(
            ctx,
            &format!("{} fields but {} field names", defs.len(), names.len()),
        ));
    }
    let mut base_spans: Vec<(u16, u16)> = Vec::new();
    let mut ext_spans: Vec<(u16, u16)> = Vec::new();
    let mut field_ids = HashSet::new();
    for f in defs {
        if !f.is_padding() && !field_ids.insert(f.field_id) {
            return Err(err(ctx, &format!("duplicate field id {}", f.field_id)));
        }
        let nested = check_def_index(ctx, f, is_record, layout)?;
        // every offset lands on its type's natural alignment.
        let align = field_align(f, nested);
        if align > 1 && f.offset % align != 0 {
            return Err(err(
                ctx,
                &format!(
                    "field {} offset {} not {align}-aligned",
                    f.field_id, f.offset
                ),
            ));
        }
        check_size(ctx, f, nested)?;
        let end = f.offset.checked_add(f.size).ok_or_else(|| {
            err(
                ctx,
                &format!("field {} offset+size overflows u16", f.field_id),
            )
        })?;
        // non-extension fields fit within the base; extensions start at or past it.
        if f.is_extension() {
            if !is_record {
                return Err(err(
                    ctx,
                    &format!("field {} may not be an extension", f.field_id),
                ));
            }
            if f.offset < base {
                return Err(err(
                    ctx,
                    &format!(
                        "extension field {} at {} precedes base {base}",
                        f.field_id, f.offset
                    ),
                ));
            }
            ext_spans.push((f.offset, end));
        } else {
            if end > base {
                return Err(err(
                    ctx,
                    &format!("field {} ends at {end}, past base {base}", f.field_id),
                ));
            }
            base_spans.push((f.offset, end));
        }
    }
    // the base region is tiled exactly.
    base_spans.sort_unstable();
    let mut cursor = 0;
    for (offset, end) in base_spans {
        if offset != cursor {
            return Err(err(
                ctx,
                &format!("base coverage gap or overlap: expected {cursor}, found {offset}"),
            ));
        }
        cursor = end;
    }
    if cursor != base {
        return Err(err(
            ctx,
            &format!("base region [0, {base}) not fully covered (reached {cursor})"),
        ));
    }
    // extensions are contiguous and ascending from the base.
    ext_spans.sort_unstable();
    let mut ext_cursor = base;
    for (offset, end) in ext_spans {
        if offset != ext_cursor {
            return Err(err(
                ctx,
                &format!("extension gap: expected {ext_cursor}, found {offset}"),
            ));
        }
        ext_cursor = end;
    }
    Ok(())
}

/// Returns the struct a `Struct` field references.
fn check_def_index<'a>(
    ctx: &str,
    f: &FieldDef,
    is_record: bool,
    layout: &'a StreamLayout,
) -> crate::Result<Option<&'a StructLayout>> {
    let (len, table) = if f.type_id == fields::STRUCT_ID {
        if !is_record {
            return Err(err(
                ctx,
                &format!("field {} nests a struct within a struct", f.field_id),
            ));
        }
        if let Some(sd) = layout.struct_layouts.get(usize::from(f.def_index)) {
            return Ok(Some(sd));
        }
        (layout.struct_layouts.len(), "struct_layouts")
    } else if fields::uses_label_def(f.type_id) && f.has_label_def() {
        if usize::from(f.def_index) < layout.label_defs.len() {
            return Ok(None);
        }
        (layout.label_defs.len(), "label_defs")
    } else {
        return Ok(None);
    };
    Err(err(
        ctx,
        &format!(
            "field {} def_index {} out of range for {table} of length {len}",
            f.field_id, f.def_index
        ),
    ))
}

fn check_size(ctx: &str, f: &FieldDef, nested: Option<&StructLayout>) -> crate::Result<()> {
    let width = if let Some(sd) = nested {
        sd.size
    } else if let Some(width) = fields::scalar_size(f.type_id) {
        width
    } else {
        return Ok(());
    };
    if f.size == width || (f.is_array() && f.size.is_multiple_of(width)) {
        return Ok(());
    }
    Err(err(
        ctx,
        &format!(
            "field {} size {} doesn't fit its {width}-byte type",
            f.field_id, f.size
        ),
    ))
}

/// The natural alignment of a field: the scalar type's width, a nested struct's
/// alignment, or 1 for padding. Computed independently of the builder so the validator
/// checks rather than trusts the layout's origin.
fn field_align(f: &FieldDef, nested: Option<&StructLayout>) -> u16 {
    if f.is_padding() {
        return 1;
    }
    if let Some(sd) = nested {
        return sd.align();
    }
    fields::scalar_size(f.type_id).unwrap_or(1)
}

fn err(ctx: &str, msg: &str) -> crate::Error {
    crate::Error::layout(format!("record `{ctx}`: {msg}"))
}
