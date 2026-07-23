use std::collections::HashSet;

use crate::{
    layout::{FieldDef, StreamLayout, StructLayout},
    v4::{fields, RecordHeader},
};

/// Validates a [`StreamLayout`] before any field access, rejecting a malformed table
/// as a decode error.
pub(crate) fn validate(layout: &StreamLayout, _symbol_cstr_len: u16) -> crate::Result<()> {
    let header_size = std::mem::size_of::<RecordHeader>() as u16;
    let mut rtypes = HashSet::new();
    for rec in &layout.record_layouts {
        if !rtypes.insert(rec.rtype) {
            return Err(err(
                &rec.name,
                &format!("duplicate rtype {:#06X}", rec.rtype),
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
            rec.base_record_size,
            true,
            layout,
        )?;
    }
    for sd in &layout.struct_layouts {
        // A struct body starts at offset 0 and has no extension fields.
        check_fields(&sd.name, &sd.field_defs, sd.size, false, layout)?;
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
    base: u16,
    allow_ext: bool,
    layout: &StreamLayout,
) -> crate::Result<()> {
    let mut base_spans: Vec<(u16, u16)> = Vec::new();
    let mut ext_spans: Vec<(u16, u16)> = Vec::new();
    for f in defs {
        // every offset lands on its type's natural alignment.
        let align = field_align(f, layout);
        if align > 1 && f.offset % align != 0 {
            return Err(err(
                ctx,
                &format!(
                    "field {} offset {} not {align}-aligned",
                    f.field_id, f.offset
                ),
            ));
        }
        check_size(ctx, f)?;
        let end = f.offset.checked_add(f.size).ok_or_else(|| {
            err(
                ctx,
                &format!("field {} offset+size overflows u16", f.field_id),
            )
        })?;
        // non-extension fields fit within the base; extensions start at or past it.
        if f.is_extension() {
            if !allow_ext {
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

fn check_size(ctx: &str, f: &FieldDef) -> crate::Result<()> {
    let Some(width) = fields::scalar_size(f.type_id) else {
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
fn field_align(f: &FieldDef, layout: &StreamLayout) -> u16 {
    if f.is_padding() {
        return 1;
    }
    if f.type_id == fields::STRUCT_ID {
        return layout
            .struct_layouts
            .get(f.def_index as usize)
            .map(StructLayout::align)
            .unwrap_or(1);
    }
    fields::scalar_size(f.type_id).unwrap_or(1)
}

fn err(ctx: &str, msg: &str) -> crate::Error {
    crate::Error::layout(format!("record `{ctx}`: {msg}"))
}
