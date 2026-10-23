use std::collections::HashMap;

use crate::{
    layout::{
        compare, DynField, FieldDef, FieldOffset, LabelDef, RecordLayout, StreamLayout, Verdict,
    },
    v4::fields::{self, Field, FieldType},
};

/// A field offset map built from a [`StreamLayout`].
#[derive(Debug, Clone)]
pub struct LayoutIndex {
    // Keyed on `rtype`, `field_id`
    fields: HashMap<(u16, u16), FieldDef>,
    verdicts: HashMap<u16, Verdict>,
    record_layouts: Vec<RecordLayout>,
    label_defs: Vec<LabelDef>,
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
        Self {
            fields,
            verdicts,
            record_layouts: layout.record_layouts.clone(),
            label_defs: layout.label_defs.clone(),
        }
    }

    /// A typed handle to `field` within `rtype`'s records. `None` if the field is
    /// absent for `rtype` or declared with a different type than `T`, including a
    /// different enum or bit set.
    ///
    /// # Examples
    /// Resolve a field once, then read it from each record with
    /// [`field_at`](crate::layout::DynFieldAccess::field_at):
    ///
    /// ```
    /// use dbn::{
    ///     layout::{DynFieldAccess, LayoutIndex},
    ///     v4::{fields, layouts::default_layout, rtype, MboMsg, TradeMsg},
    /// };
    ///
    /// let index = LayoutIndex::new(default_layout(), default_layout());
    /// let price = index.offset_of(rtype::MBP_0, fields::PRICE).unwrap();
    /// for raw in [4_215_500_000_000, 4_215_750_000_000] {
    ///     let trade = TradeMsg {
    ///         price: raw,
    ///         ..Default::default()
    ///     };
    ///     assert_eq!(trade.field_at(price).unwrap().raw, raw);
    /// }
    ///
    /// // `MboMsg` holds its price at a different offset, and the handle's rtype keeps it
    /// // from misreading one
    /// let mbo = MboMsg::default();
    /// assert_ne!(index.offset_of(rtype::MBO, fields::PRICE).unwrap().offset(), price.offset());
    /// assert!(mbo.field_at(price).is_none());
    /// ```
    pub fn offset_of<T: FieldType>(&self, rtype: u16, field: Field<T>) -> Option<FieldOffset<T>> {
        let def = *self.fields.get(&(rtype, field.id()))?;
        if let Some(labels) = T::LABELS {
            if self
                .label_def(&def)
                .is_some_and(|ld| ld.name != labels.name)
            {
                return None;
            }
        }
        FieldOffset::from_def(rtype, def)
    }

    /// An untyped handle to the field named `name` within `rtype`'s records, or `None`
    /// if `rtype` has no such field.
    pub fn field_by_name(&self, rtype: u16, name: &str) -> Option<DynField<'_>> {
        let rec = self.record_layouts.iter().find(|rec| rec.rtype == rtype)?;
        let (_, def) = rec
            .field_names
            .iter()
            .zip(&rec.field_defs)
            .find(|(field_name, def)| *field_name == name && !def.is_padding())?;
        self.field_by_id(rtype, def.field_id)
    }

    /// An untyped handle to the field with ID `field_id` within `rtype`'s records, or
    /// `None` if `rtype` has no such field.
    pub fn field_by_id(&self, rtype: u16, field_id: u16) -> Option<DynField<'_>> {
        let def = *self.fields.get(&(rtype, field_id))?;
        Some(DynField {
            rtype,
            def,
            label_def: self.label_def(&def),
        })
    }

    /// Whether `rtype`'s records can be cast to a compiled struct. `None` if the stream
    /// declares no layout for `rtype`.
    pub fn verdict(&self, rtype: u16) -> Option<Verdict> {
        self.verdicts.get(&rtype).copied()
    }

    fn label_def(&self, def: &FieldDef) -> Option<&LabelDef> {
        if fields::uses_label_def(def.type_id) && def.has_label_def() {
            self.label_defs.get(usize::from(def.def_index))
        } else {
            None
        }
    }
}
