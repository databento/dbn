//! Self-describing record layouts for DBN version 4 streams.
//!
//! [`StreamLayout`] is a table containing: one [`RecordLayout`] per rtype, plus the
//! nested [`StructLayout`]s and [`LabelDef`]s its fields reference.

use std::collections::BTreeMap;

use crate::v4::fields;

mod builders;
mod hash;
mod validate;

/// The layout descriptions for all records in a DBN stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamLayout {
    /// Advisory monotonic revision number.
    pub stream_layout_version: u16,
    /// The record layouts of this stream.
    pub record_layouts: Vec<RecordLayout>,
    /// Struct layouts for struct fields in the records.
    pub struct_layouts: Vec<StructLayout>,
    /// Enum and bit flag definitions.
    pub label_defs: Vec<LabelDef>,
}

/// The layout description of a particular record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordLayout {
    /// The sentinel record type ID [`crate::v4::rtype`].
    pub rtype: u16,
    /// The schema (see [`Schema`](crate::Schema)).
    pub schema: u16,
    /// The size in bytes of the record, excluding extension fields.
    pub base_record_size: u16,
    /// The hash of the records layout and its component fields.
    pub layout_hash: u64,
    /// The display name of record.
    pub name: String,
    /// Definitions of the records fields.
    pub field_defs: Vec<FieldDef>,
    /// The display names of the fields.
    pub field_names: Vec<String>,
}

impl RecordLayout {
    /// The only legal size for records of this rtype in this stream: the base plus
    /// whatever extension tail the layout declares.
    pub fn record_size(&self) -> u16 {
        self.field_defs
            .iter()
            .map(|f| f.offset + f.size)
            .max()
            .unwrap_or(self.base_record_size)
            .max(self.base_record_size)
    }
}

/// The encoded/decoded definition of a field in a particular record type and schema.
/// Similar to [`crate::v4::fields::Field`], but not generic.
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct FieldDef {
    /// See [`crate::v4::fields`].
    pub field_id: u16,
    /// The byte offset of the field from the start of the record.
    pub offset: u16,
    /// The total span of the field.
    ///
    /// For arrays, it includes all elements, therefore element count =  `size` / element_size.
    pub size: u16,
    /// The type tag. See [`crate::v4::fields`].
    pub type_id: u8,
    /// Bit flags
    /// - 7: `is_array`
    /// - 6: `is_extension`
    /// - 0-5: reserved
    pub flags: u8,
    /// Index into [`StreamLayout::label_defs`] for enums and bit sets, or
    /// [`UNDEF_LABEL_DEF`](FieldDef::UNDEF_LABEL_DEF) when unlabeled. Index into
    /// [`StreamLayout::struct_layouts`] for a `Struct`, and 0 for every other type.
    pub def_index: u16,
    /// Fixed decimal exponent; 0 otherwise.
    pub scale: i8,
    #[doc(hidden)]
    pub(crate) _reserved: [u8; 5],
}
const _: () = assert!(
    std::mem::size_of::<FieldDef>() == 16,
    "FieldDef does not match expected size"
);

impl FieldDef {
    /// `def_index` when the field references no [`LabelDef`].
    pub const UNDEF_LABEL_DEF: u16 = u16::MAX;

    /// Whether this field holds an array of its element type.
    pub const fn is_array(&self) -> bool {
        self.flags & fields::FLAG_IS_ARRAY != 0
    }

    /// Whether this field lives past `base_record_size` in the extension tail.
    pub const fn is_extension(&self) -> bool {
        self.flags & fields::FLAG_IS_EXTENSION != 0
    }

    /// Whether this field is explicit padding rather than a data field.
    pub const fn is_padding(&self) -> bool {
        self.type_id == fields::PADDING_ID
    }

    /// Whether this field names a [`LabelDef`] for its enum or bit-set labels.
    pub const fn has_label_def(&self) -> bool {
        self.def_index != Self::UNDEF_LABEL_DEF
    }
}

/// The layout description of a struct within one or more record types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructLayout {
    /// The size in bytes of the struct.
    pub size: u16,
    /// The display name of the struct.
    pub name: String,
    /// Definitions of the records fields.
    pub field_defs: Vec<FieldDef>,
    /// The display names of the fields.
    pub field_names: Vec<String>,
}

/// The description of a bit set or enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelDef {
    /// The display name of the bit set or enum.
    pub name: String,
    /// Map of enum value (e.g. `'A'`) or bit position for a bit set to name (e.g.
    /// `"ASK"`).
    // `BTreeMap` to preserve order while facilitating lookup by ID.
    pub labels: BTreeMap<u16, String>,
}
