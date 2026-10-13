use std::collections::HashMap;

use crate::{
    layout::{compare, FieldDef, FieldOffset, StreamLayout, Verdict},
    v4::fields::{Field, FieldType},
};

/// A field offset map built from a [`StreamLayout`].
#[derive(Debug, Clone)]
pub struct LayoutIndex {
    // Keyed on `rtype`, `field_id`
    fields: HashMap<(u16, u16), FieldDef>,
    verdicts: HashMap<u16, Verdict>,
}

impl LayoutIndex {
    /// Builds a field-offset map from `layout` and decides each rtype's [`Verdict`].
    pub fn new(layout: &StreamLayout, compiled: &StreamLayout) -> Self {
        let fields = layout
            .record_layouts
            .iter()
            .flat_map(|rec| {
                rec.field_defs
                    .iter()
                    .filter(|f| !f.is_padding())
                    .map(|f| ((rec.rtype, f.field_id), *f))
            })
            .collect();
        let verdicts = layout
            .record_layouts
            .iter()
            .map(|rec| (rec.rtype, compare::verdict(layout, rec, compiled)))
            .collect();
        Self { fields, verdicts }
    }

    /// A typed handle to `field` within `rtype`'s records. `None` if the field is
    /// absent for `rtype` or declared with a different type than `T`.
    pub fn offset_of<T: FieldType>(&self, rtype: u16, field: Field<T>) -> Option<FieldOffset<T>> {
        FieldOffset::from_def(rtype, *self.fields.get(&(rtype, field.id()))?)
    }

    /// Whether `rtype`'s records can be cast to a compiled struct. `None` if the stream
    /// declares no layout for `rtype`.
    pub fn verdict(&self, rtype: u16) -> Option<Verdict> {
        self.verdicts.get(&rtype).copied()
    }
}
