use std::collections::HashMap;

use crate::{
    layout::{FieldDef, FieldOffset, StreamLayout},
    v4::fields::{Field, FieldType},
};

/// A field offset map built from a [`StreamLayout`].
#[derive(Debug, Clone)]
pub struct LayoutIndex {
    // Keyed on `rtype`, `field_id`
    fields: HashMap<(u16, u16), FieldDef>,
}

impl LayoutIndex {
    /// Builds a field-offset map from `layout`.
    pub fn new(layout: &StreamLayout) -> Self {
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
        Self { fields }
    }

    /// A typed handle to `field` within `rtype`'s records. `None` if the field is
    /// absent for `rtype` or declared with a different type than `T`.
    pub fn offset_of<T: FieldType>(&self, rtype: u16, field: Field<T>) -> Option<FieldOffset<T>> {
        FieldOffset::from_def(rtype, *self.fields.get(&(rtype, field.id()))?)
    }
}
