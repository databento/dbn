//! Self-describing record layouts for DBN version 4 streams.
//!
//! [`StreamLayout`] is a table containing: one [`RecordLayout`] per rtype, plus the
//! nested [`StructLayout`]s and [`LabelDef`]s its fields reference.

use std::{collections::BTreeMap, marker::PhantomData};

use crate::{
    v4::fields::{self, DecodeField, Field, FieldType},
    Record,
};

mod builders;
mod hash;
mod index;
mod validate;

pub use index::LayoutIndex;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Similar to a pointer-to-member in C++. Contains the offset to a field in a specific
/// record based on the stream's layout description.
///
/// # Correctness
/// An offset is specific to the stream it was resolved from. Records of the same rtype
/// in another stream can have different extension fields, so reusing offsets across
/// streams can return a wrong value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldOffset<T> {
    rtype: u16,
    def: FieldDef,
    _marker: PhantomData<T>,
}

impl StreamLayout {
    /// The layout for records of `rtype`, or `None` if the stream carries no such
    /// record type.
    pub fn record_layout(&self, rtype: u16) -> Option<&RecordLayout> {
        self.record_layouts.iter().find(|r| r.rtype == rtype)
    }
}

impl<T: FieldType> FieldOffset<T> {
    pub(crate) fn from_def(rtype: u16, def: FieldDef) -> Option<Self> {
        (def.type_id == T::TYPE_ID
            && def.is_array() == T::ARRAY_WIRE_SIZE.is_some()
            && T::ARRAY_WIRE_SIZE.is_none_or(|size| size == def.size))
        .then_some(Self {
            rtype,
            def,
            _marker: PhantomData,
        })
    }
}

impl<T> FieldOffset<T> {
    /// Returns the associated record type.
    pub fn rtype(&self) -> u16 {
        self.rtype
    }

    /// Returns the byte offset of the field.
    pub fn offset(&self) -> u16 {
        self.def.offset
    }

    /// Returns the size of the field.
    pub fn size(&self) -> u16 {
        self.def.size
    }

    /// Returns `true` if the field is an extension field.
    pub fn is_extension(&self) -> bool {
        self.def.is_extension()
    }
}

impl<T: DecodeField> FieldOffset<T> {
    /// Decodes this field from a record's `bytes`. `rtype` is the record's own rtype; a
    /// mismatch with the rtype this offset was resolved for yields `None`, as does a
    /// record too short to contain the field or bytes that can't form a valid value.
    ///
    /// # Correctness
    /// Sound for any `bytes`: the read is a checked slice, so a stale or foreign offset
    /// can only misread within `bytes` or return `None`.
    pub fn decode<'a>(&self, bytes: &'a [u8], rtype: u16) -> Option<T::Value<'a>> {
        if rtype != self.rtype {
            return None;
        }
        let start = self.def.offset as usize;
        T::decode(
            bytes.get(start..start + self.def.size as usize)?,
            self.def.scale,
        )
    }
}

/// Layout-driven dynamic field access for a v4 record holder:
/// [`RecordRef`](crate::RecordRef), [`RecordBuf`](crate::RecordBuf), and concrete
/// records like [`MboMsg`](crate::v4::MboMsg).
///
/// # Correctness
/// Threading an `index` that does not describe this record's stream is a correctness
/// not a soundness issue. Reads are bounds-checked against the record's own
/// length, so a mismatched index returns a wrong value or `None`, not an
/// out-of-bounds read.
//
// Implementation:
// A record carries no layout, so the
// stream's [`LayoutIndex`] is threaded in and the record's own rtype selects its
// fields. Scalar fields return an owned value; borrowed field types (string,
// array, struct) return a reference tied to `&self`.
pub trait DynFieldAccess: Record<Header = crate::v4::RecordHeader> {
    /// Reads `field` via `index`. `None` if the field is absent for the record's
    /// rtype, typed differently than `T`, or beyond the record's length.
    fn field<T: DecodeField>(&self, index: &LayoutIndex, field: Field<T>) -> Option<T::Value<'_>> {
        self.field_at(index.offset_of(self.raw_rtype(), field)?)
    }

    /// Reads a field from a [`FieldOffset`] resolved once against the index, skipping
    /// the per-call lookup. `None` if the offset was resolved for a different rtype or
    /// the record is too short.
    fn field_at<T: DecodeField>(&self, offset: FieldOffset<T>) -> Option<T::Value<'_>> {
        offset.decode(self.as_ref(), self.raw_rtype())
    }
}

impl<R: Record<Header = crate::v4::RecordHeader>> DynFieldAccess for R {}

impl StructLayout {
    pub(crate) fn align(&self) -> u16 {
        self.field_defs
            .iter()
            .filter_map(|f| fields::scalar_size(f.type_id))
            .max()
            .unwrap_or(1)
    }
}

#[cfg(test)]
mod tests {
    use std::os::raw::c_char;

    use super::{builders::StreamLayoutBuilder, *};
    use crate::{
        v4::{
            self, rtype,
            types::{CStr, Char, Decimal},
            MboMsg, RecordHeader,
        },
        Action, FlagSet, Side,
    };

    /// A layout whose single record mirrors `v4::MboMsg`'s field offsets exactly.
    fn mbo_layout() -> StreamLayout {
        let mut sb = StreamLayoutBuilder::new(1);
        {
            let mut r = sb.record(rtype::MBO, 0, "MboMsg");
            r.field(fields::ORDER_ID, "order_id")
                .field(fields::PRICE, "price")
                .field(fields::SIZE, "size")
                .field(fields::FLAGS, "flags")
                .field(fields::CHANNEL_ID, "channel_id")
                .field(fields::ACTION, "action")
                .field(fields::SIDE, "side")
                .field(fields::TS_RECV, "ts_recv")
                .field(fields::TS_IN_DELTA, "ts_in_delta")
                .field(fields::SEQUENCE, "sequence");
            r.finish();
        }
        sb.build(8).unwrap()
    }

    fn mbo_index() -> LayoutIndex {
        LayoutIndex::new(&mbo_layout())
    }

    fn sample_mbo() -> MboMsg {
        MboMsg {
            hd: RecordHeader::new::<MboMsg>(rtype::MBO, 1, 555, 111),
            order_id: 42,
            price: 123_000_000_000,
            size: 7,
            flags: FlagSet::new(0b1010),
            channel_id: 3,
            action: b'A' as c_char,
            side: b'B' as c_char,
            ts_recv: 222,
            ts_in_delta: 9,
            sequence: 88,
        }
    }

    #[test]
    fn layout_base_matches_struct() {
        let base = mbo_layout()
            .record_layout(rtype::MBO)
            .unwrap()
            .base_record_size;
        assert_eq!(base as usize, std::mem::size_of::<MboMsg>());
    }

    #[test]
    fn offset_of_resolves_and_rejects() {
        let index = mbo_index();
        assert_eq!(
            index
                .offset_of(rtype::MBO, fields::ORDER_ID)
                .unwrap()
                .offset(),
            24
        );
        // absent field id
        assert!(index
            .offset_of(rtype::MBO, Field::<u64>::new(9999))
            .is_none());
        // rtype with no record layout
        assert!(index.offset_of(rtype::MBP_1, fields::ORDER_ID).is_none());
        // type mismatch: PRICE is a Decimal (Fixed), not a u32
        assert!(index
            .offset_of(rtype::MBO, Field::<u32>::new(fields::PRICE.id()))
            .is_none());
    }

    #[test]
    fn mbo_round_trips_through_field_table() {
        let index = mbo_index();
        let mbo = sample_mbo();
        let rec = crate::RecordRef::from(&mbo);

        assert_eq!(rec.field(&index, fields::ORDER_ID), Some(42u64));
        assert_eq!(
            rec.field(&index, fields::PRICE),
            Some(Decimal {
                raw: 123_000_000_000,
                scale: -9,
            })
        );
        assert_eq!(rec.field(&index, fields::SIZE), Some(7u32));
        assert_eq!(rec.field(&index, fields::CHANNEL_ID), Some(3u8));
        assert_eq!(
            rec.field(&index, fields::FLAGS).map(|f| f.raw()),
            Some(0b1010)
        );
        assert_eq!(rec.field(&index, fields::TS_RECV).map(|t| t.0), Some(222));
        assert_eq!(rec.field(&index, fields::SEQUENCE), Some(88u32));
        assert_eq!(rec.field(&index, fields::TS_IN_DELTA), Some(9i32));
        assert!(rec.field(&index, fields::DEPTH).is_none());
        // OpenEnum: raw byte preserved, and it resolves to the known variant.
        let action = rec.field(&index, fields::ACTION).unwrap();
        assert_eq!(action.get().unwrap(), Action::Add);
        assert_eq!(
            rec.field(&index, fields::SIDE).unwrap().get().unwrap(),
            Side::Bid
        );
    }

    #[test]
    fn field_at_matches_field_and_guards_rtype() {
        let index = mbo_index();
        let mbo = sample_mbo();
        let rec = crate::RecordRef::from(&mbo);

        let off = index.offset_of(rtype::MBO, fields::ORDER_ID).unwrap();
        assert_eq!(rec.field_at(off), Some(42u64));

        // An offset resolved for another rtype must not read this record.
        let bad = FieldOffset {
            rtype: 0x9999,
            ..off
        };
        assert!(rec.field_at(bad).is_none());
    }

    #[test]
    fn offset_of_rejects_array_shape_mismatch() {
        let conditions = Field::<[Char; 4]>::new(0x1A);
        let mut sb = StreamLayoutBuilder::new(1);
        {
            let mut r = sb.record(rtype::MBP_0, 0, "WithConditions");
            r.field(conditions, "conditions");
            r.finish();
        }
        let index = LayoutIndex::new(&sb.build(8).unwrap());

        assert!(index.offset_of(rtype::MBP_0, conditions).is_some());
        assert!(index
            .offset_of(rtype::MBP_0, Field::<Char>::new(conditions.id()))
            .is_none());
        assert!(index
            .offset_of(rtype::MBP_0, Field::<[Char; 8]>::new(conditions.id()))
            .is_none());
    }

    #[test]
    fn reads_string_field() {
        let mut sb = StreamLayoutBuilder::new(1);
        {
            let mut r = sb.record(rtype::MBP_1, 0, "SymRec");
            r.str_field(Field::<CStr>::new(30), "symbol", 8);
            r.finish();
        }
        let index = LayoutIndex::new(&sb.build(8).unwrap());

        #[repr(C, align(8))]
        struct Aligned([u8; 32]);
        let mut buf = Aligned([0u8; 32]);
        buf.0[0..2].copy_from_slice(&8u16.to_le_bytes()); // length: 8 bytes after header
        buf.0[2..4].copy_from_slice(&rtype::MBP_1.to_le_bytes());
        buf.0[24..26].copy_from_slice(b"ES");

        let rec = unsafe { v4::RecordRef::new(&buf.0) };
        assert_eq!(rec.field(&index, Field::<CStr>::new(30)), Some("ES"));
    }

    #[test]
    fn validate_rejects_corruption() {
        // misaligned offset (price is a Decimal, 8-aligned)
        let mut bad = mbo_layout();
        bad.record_layouts[0].field_defs[1].offset = 33;
        assert!(super::validate::validate(&bad, 8).is_err());

        // a field running past base_record_size
        let mut bad = mbo_layout();
        bad.record_layouts[0].field_defs.last_mut().unwrap().size += 8;
        assert!(super::validate::validate(&bad, 8).is_err());

        let mut bad = mbo_layout();
        bad.record_layouts[0]
            .field_defs
            .iter_mut()
            .find(|f| f.field_id == fields::SIZE.id())
            .unwrap()
            .size = 8;
        let err = super::validate::validate(&bad, 8).unwrap_err().to_string();
        assert!(err.contains("doesn't fit its 4-byte type"), "{err}");

        let mut bad = mbo_layout();
        bad.record_layouts.push(bad.record_layouts[0].clone());
        let err = super::validate::validate(&bad, 8).unwrap_err().to_string();
        assert!(err.contains("duplicate rtype"), "{err}");

        // a hole in the base coverage
        let mut bad = mbo_layout();
        bad.record_layouts[0].field_defs.remove(0);
        assert!(super::validate::validate(&bad, 8).is_err());
    }
}
