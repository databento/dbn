use super::{FieldDef, StreamLayout};
use crate::{v4::fields, Error};

impl StreamLayout {
    /// Combines this layout with `other` into one describing the records of all of them.
    /// Records share identical struct layouts and label definitions.
    ///
    /// # Errors
    /// This function returns an error if two layouts describe the same rtype
    /// differently or a field references a struct layout or label definition its layout
    /// lacks.
    pub fn merge(mut self, other: impl IntoIterator<Item = StreamLayout>) -> crate::Result<Self> {
        for other in other {
            self.merge_one(other)?;
        }
        Ok(self)
    }

    fn merge_one(&mut self, other: StreamLayout) -> crate::Result<()> {
        self.stream_layout_version = self.stream_layout_version.max(other.stream_layout_version);
        let label_indices = other
            .label_defs
            .into_iter()
            .map(|label_def| find_or_push(&mut self.label_defs, label_def))
            .collect::<crate::Result<Vec<_>>>()?;
        let struct_indices = other
            .struct_layouts
            .into_iter()
            .map(|mut struct_layout| {
                remap(&mut struct_layout.field_defs, &label_indices, &[])?;
                find_or_push(&mut self.struct_layouts, struct_layout)
            })
            .collect::<crate::Result<Vec<_>>>()?;
        for mut record_layout in other.record_layouts {
            remap(
                &mut record_layout.field_defs,
                &label_indices,
                &struct_indices,
            )?;
            if let Some(existing) = self
                .record_layouts
                .iter()
                .find(|existing| existing.rtype == record_layout.rtype)
            {
                if *existing != record_layout {
                    return Err(bad_arg(format!(
                        "conflicting layouts for rtype {:#06X} ({})",
                        record_layout.rtype, record_layout.name
                    )));
                }
            } else {
                self.record_layouts.push(record_layout);
            }
        }
        Ok(())
    }
}

fn find_or_push<T: PartialEq>(table: &mut Vec<T>, item: T) -> crate::Result<u16> {
    let index = if let Some(index) = table.iter().position(|existing| *existing == item) {
        index
    } else {
        table.push(item);
        table.len() - 1
    };
    u16::try_from(index).map_err(|_| bad_arg("too many struct layouts or label definitions"))
}

fn remap(
    field_defs: &mut [FieldDef],
    label_indices: &[u16],
    struct_indices: &[u16],
) -> crate::Result<()> {
    for field_def in field_defs {
        let indices = if field_def.type_id == fields::STRUCT_ID {
            struct_indices
        } else if fields::uses_label_def(field_def.type_id) && field_def.has_label_def() {
            label_indices
        } else {
            continue;
        };
        field_def.def_index = *indices
            .get(usize::from(field_def.def_index))
            .ok_or_else(|| {
                bad_arg(format!(
                    "field {:#06X} references missing definition {}",
                    field_def.field_id, field_def.def_index
                ))
            })?;
    }
    Ok(())
}

fn bad_arg(desc: impl Into<String>) -> Error {
    Error::BadArgument {
        param_name: "other".to_owned(),
        desc: desc.into(),
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use crate::{
        layout::{
            tests::{mbo_layout, trades_layout},
            upgrade_layout, LayoutIndex,
        },
        v4::{layouts::default_layout, rtype},
        Schema,
    };

    #[rstest]
    fn merging_remaps_label_defs(
        #[values(rtype::MBP_0, rtype::MBO)] rtype: u16,
        #[values("action", "side", "flags")] name: &str,
    ) {
        let merged = trades_layout().merge([mbo_layout()]).unwrap();
        let merged_index = LayoutIndex::new(&merged, default_layout());
        let source = if rtype == rtype::MBO {
            LayoutIndex::new(&mbo_layout(), default_layout())
        } else {
            LayoutIndex::new(&trades_layout(), default_layout())
        };
        let label_def = source.field_by_name(rtype, name).unwrap().label_def();
        assert!(label_def.is_some());
        assert_eq!(
            merged_index.field_by_name(rtype, name).unwrap().label_def(),
            label_def
        );
        assert_eq!(merged.label_defs.len(), trades_layout().label_defs.len());
    }

    #[test]
    fn merging_a_layout_with_itself_changes_nothing() {
        let layout = upgrade_layout(false, Some(Schema::Statistics));
        assert_eq!(layout.clone().merge([layout.clone()]).unwrap(), layout);
    }

    #[test]
    fn merging_conflicting_rtype_layouts_errors() {
        let without_ts_out = upgrade_layout(false, Some(Schema::Mbo));
        let with_ts_out = upgrade_layout(true, Some(Schema::Mbo));
        assert!(matches!(
            without_ts_out.merge([with_ts_out]),
            Err(crate::Error::BadArgument { desc, .. }) if desc.starts_with("conflicting layouts")
        ));
    }

    #[test]
    fn merging_a_missing_definition_errors() {
        let mut bad = upgrade_layout(false, Some(Schema::Mbo));
        bad.struct_layouts.clear();
        assert!(upgrade_layout(false, Some(Schema::Trades))
            .merge([bad])
            .is_err());
    }
}
