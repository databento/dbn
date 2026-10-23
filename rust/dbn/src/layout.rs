//! Self-describing record layouts for DBN version 4 streams.
//!
//! [`StreamLayout`] is a table containing: one [`RecordLayout`] per rtype, plus the
//! nested [`StructLayout`]s and [`LabelDef`]s its fields reference.
//!
//! # Examples
//! Records whose layout matches a compiled struct cast to it, and any field the stream
//! declares reads through its [`LayoutIndex`]:
//!
//! ```
//! use dbn::{
//!     decode::{
//!         dbn::{fsm::DbnFsm, v4::Decoder},
//!         DbnMetadata,
//!     },
//!     encode::dbn::v4::Encoder,
//!     layout::DynFieldAccess,
//!     v4::{self, fields, layouts::default_layout, TradeMsg},
//!     Dataset, Metadata, SType, Schema,
//! };
//!
//! let metadata = Metadata::builder()
//!     .version(v4::DBN_VERSION)
//!     .dataset(Dataset::XnasItch)
//!     .schema(Some(Schema::Trades))
//!     .start(1_609_160_400_000_000_000)
//!     .stype_in(Some(SType::RawSymbol))
//!     .stype_out(SType::InstrumentId)
//!     .layout(default_layout().clone())
//!     .build();
//! let trade = TradeMsg {
//!     price: 4_215_500_000_000,
//!     size: 7,
//!     ..Default::default()
//! };
//! let mut encoded = Vec::new();
//! Encoder::new(&mut encoded, &metadata)?.encode(&trade)?;
//!
//! let fsm = DbnFsm::builder().header::<v4::RecordHeader>().build()?;
//! let mut decoder = Decoder::with_fsm(encoded.as_slice(), fsm)?;
//! let index = decoder.metadata().layout_index().unwrap();
//! let rec = decoder.decode_ref()?.unwrap();
//!
//! assert_eq!(rec.try_get::<TradeMsg>()?.size, 7);
//! assert_eq!(rec.field(&index, fields::PRICE).unwrap().raw, trade.price);
//! // This stream's trades declare no `sale_condition`
//! assert!(rec.field(&index, fields::SALE_CONDITION).is_none());
//! # Ok::<_, dbn::Error>(())
//! ```

use std::{collections::BTreeMap, marker::PhantomData};

use crate::{
    v4::{
        fields::{self, DecodeField, Field, FieldType},
        types::Symbol,
    },
    Record,
};

mod builders;
mod compare;
mod hash;
mod index;
mod pre_v4;
mod upgrade;
mod validate;
pub mod wire;

pub use builders::{RecordLayoutBuilder, StreamLayoutBuilder, StructLayoutBuilder};
pub use index::LayoutIndex;
pub(crate) use upgrade::{upgrade_layout, Upgrader};
#[cfg(test)]
pub(crate) use validate::validate;
pub(crate) use validate::validate_for_metadata;

/// The layout description of a record struct, used to build its [`RecordLayout`].
///
/// Usually implemented by [`dbn_record`](crate::macros::dbn_record).
pub trait RecordLayoutDesc {
    /// The display name of the record.
    const NAME: &'static str;

    /// Adds the fields the struct holds, after the header.
    fn add_fields(r: &mut RecordLayoutBuilder<'_>);

    /// The symbol fields every layout of this record declares but no struct holds.
    const REQUIRED_FIELDS: &'static [(Field<Symbol>, &'static str)] = &[];
}

/// The layout description of a struct nested in records, used to build its
/// [`StructLayout`].
///
/// Usually derived with [`StructLayoutDesc`](crate::macros::StructLayoutDesc).
pub trait StructLayoutDesc {
    /// The display name of the struct.
    const NAME: &'static str;

    /// Adds the fields of the struct.
    fn add_fields(s: &mut StructLayoutBuilder<'_>);
}

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
    /// whatever extensions the layout declares.
    pub fn record_size(&self) -> u16 {
        self.field_defs
            .iter()
            .map(|f| f.offset + f.size)
            .max()
            .unwrap_or(self.base_record_size)
            .max(self.base_record_size)
    }

    /// Returns `true` if the layout declares `f` with `f`'s type.
    pub fn has_field<T: FieldType>(&self, f: Field<T>) -> bool {
        self.field_defs
            .iter()
            .any(|def| def.field_id == f.id() && def.type_id == f.type_id())
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

    /// Whether this field is an extension, placed past `base_record_size`.
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

/// How records of one rtype in a stream can be read, decided once per stream by
/// comparing the stream's layout against the compiled one.
///
/// # Examples
/// ```
/// use dbn::{
///     layout::{LayoutIndex, StreamLayoutBuilder, Verdict},
///     v4::{fields, layouts::default_layout, rtype, TradeMsg},
///     Schema,
/// };
///
/// const RTYPE_CUSTOM_TRADES: u16 = 0x01A0;
///
/// let mut sb = StreamLayoutBuilder::new(1, dbn::SYMBOL_CSTR_LEN as u16);
/// let mut trades = sb.layout_of::<TradeMsg>(rtype::MBP_0);
/// trades.finish_base();
/// trades.field(fields::SALE_CONDITION, "sale_condition");
/// trades.finish();
/// let mut custom = sb.record(RTYPE_CUSTOM_TRADES, Schema::Trades as u16, "CustomTradeMsg");
/// custom.field(fields::PRICE, "price");
/// custom.finish();
/// let index = LayoutIndex::new(&sb.build()?, default_layout());
///
/// // Extension fields leave the base, and with it the cast, unchanged
/// assert_eq!(index.verdict(rtype::MBP_0), Some(Verdict::Compiled));
/// // No compiled struct exists for this rtype
/// assert_eq!(index.verdict(RTYPE_CUSTOM_TRADES), Some(Verdict::Dynamic));
/// // The stream carries no MBO records
/// assert_eq!(index.verdict(rtype::MBO), None);
/// # Ok::<_, dbn::Error>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Verdict {
    /// A compiled struct describes every base field, so records can be cast to it.
    /// Fields the compiled struct doesn't name require dynamic access.
    Compiled,
    /// No compiled struct matches, so every field needs dynamic access.
    Dynamic,
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

/// An untyped handle to a field resolved from a [`LayoutIndex`] by name or ID, for
/// reading fields without a compiled type, e.g. with
/// [`enum_at`](DynFieldAccess::enum_at).
///
/// # Correctness
/// Like a [`FieldOffset`], a handle is specific to the stream it was resolved from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DynField<'a> {
    rtype: u16,
    def: FieldDef,
    label_def: Option<&'a LabelDef>,
}

impl<'a> DynField<'a> {
    /// Returns the field's definition.
    pub fn def(&self) -> &FieldDef {
        &self.def
    }

    /// Returns the field's [`LabelDef`], or `None` if it isn't a labeled enum or bit
    /// set.
    pub fn label_def(&self) -> Option<&'a LabelDef> {
        self.label_def
    }

    fn bytes<'r>(&self, record: &'r [u8], rtype: u16) -> Option<&'r [u8]> {
        if rtype != self.rtype || self.def.is_array() {
            return None;
        }
        let start = usize::from(self.def.offset);
        record.get(start..start + usize::from(self.def.size))
    }
}

/// An enum value read through a [`DynField`], with the [`LabelDef`] naming it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumValue<'a> {
    raw: u16,
    label_def: Option<&'a LabelDef>,
}

impl<'a> EnumValue<'a> {
    /// Returns the raw value, e.g. `65` for `Side::Ask`.
    pub fn raw(&self) -> u16 {
        self.raw
    }

    /// Returns the value's label, e.g. `ASK`, or `None` if the field is unlabeled or
    /// the value has no label.
    pub fn label(&self) -> Option<&'a str> {
        self.label_def?.labels.get(&self.raw).map(String::as_str)
    }

    /// Returns the enum's name, e.g. `Side`, or `None` if the field is unlabeled.
    pub fn enum_name(&self) -> Option<&'a str> {
        self.label_def.map(|ld| ld.name.as_str())
    }
}

/// A bit set value read through a [`DynField`], with the [`LabelDef`] naming its bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitSetValue<'a> {
    raw: u64,
    label_def: Option<&'a LabelDef>,
}

impl<'a> BitSetValue<'a> {
    /// Returns the raw bits.
    pub fn raw(&self) -> u64 {
        self.raw
    }

    /// Returns the labels of the set bits in ascending bit order. Set bits without a
    /// label are skipped.
    pub fn labels(&self) -> impl Iterator<Item = &'a str> {
        let raw = self.raw;
        self.label_def
            .into_iter()
            .flat_map(|ld| &ld.labels)
            .filter(move |(&bit, _)| bit < u64::BITS as u16 && raw & (1 << bit) != 0)
            .map(|(_, label)| label.as_str())
    }
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

    /// Reads an enum `field` of any width. `None` if `field` was resolved for a
    /// different rtype, isn't an enum, or is beyond the record's length.
    ///
    /// # Examples
    /// ```
    /// use dbn::{
    ///     layout::{DynFieldAccess, LayoutIndex},
    ///     v4::{layouts::default_layout, rtype, MboMsg},
    ///     Side,
    /// };
    ///
    /// let index = LayoutIndex::new(default_layout(), default_layout());
    /// let side = index.field_by_name(rtype::MBO, "side").unwrap();
    /// let mbo = MboMsg {
    ///     side: Side::Ask as std::ffi::c_char,
    ///     ..Default::default()
    /// };
    ///
    /// let value = mbo.enum_at(&side).unwrap();
    /// assert_eq!(value.enum_name(), Some("Side"));
    /// assert_eq!(value.label(), Some("ASK"));
    /// ```
    fn enum_at<'a>(&self, field: &DynField<'a>) -> Option<EnumValue<'a>> {
        let bytes = field.bytes(self.as_ref(), self.raw_rtype())?;
        let raw = match field.def.type_id {
            fields::ENUM_CHAR_ID | fields::ENUM_8_ID => u16::from(u8::decode(bytes, 0)?),
            fields::ENUM_16_ID => u16::decode(bytes, 0)?,
            _ => return None,
        };
        Some(EnumValue {
            raw,
            label_def: field.label_def,
        })
    }

    /// Reads a bit set `field` of any width. `None` if `field` was resolved for a
    /// different rtype, isn't a bit set, or is beyond the record's length.
    fn bit_set_at<'a>(&self, field: &DynField<'a>) -> Option<BitSetValue<'a>> {
        let bytes = field.bytes(self.as_ref(), self.raw_rtype())?;
        let raw = match field.def.type_id {
            fields::BIT_SET_8_ID => u64::from(u8::decode(bytes, 0)?),
            fields::BIT_SET_16_ID => u64::from(u16::decode(bytes, 0)?),
            fields::BIT_SET_32_ID => u64::from(u32::decode(bytes, 0)?),
            fields::BIT_SET_64_ID => u64::decode(bytes, 0)?,
            _ => return None,
        };
        Some(BitSetValue {
            raw,
            label_def: field.label_def,
        })
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

    use rstest::*;

    use super::*;
    use crate::{
        flags,
        v4::{
            self,
            layouts::default_layout,
            rtype,
            types::{CStr, Char, Decimal, OpenEnum},
            MboMsg, RecordHeader, StatMsg, TradeMsg,
        },
        Action, FlagSet, Side, StatType, StatUpdateAction,
    };

    fn mbo_layout() -> StreamLayout {
        let mut sb = StreamLayoutBuilder::new(1, 8);
        sb.layout_of::<MboMsg>(rtype::MBO).finish();
        sb.build().unwrap()
    }

    fn mbo_index() -> LayoutIndex {
        LayoutIndex::new(&mbo_layout(), default_layout())
    }

    fn trades_layout() -> StreamLayout {
        let mut sb = StreamLayoutBuilder::new(1, 8);
        sb.layout_of::<TradeMsg>(rtype::MBP_0).finish();
        sb.build().unwrap()
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
        let mut sb = StreamLayoutBuilder::new(1, 8);
        {
            let mut r = sb.record(rtype::MBP_0, 0, "TradeMsg");
            r.field(sale_condition::<[Char; 4]>(), "sale_condition");
            r.finish();
        }
        let index = LayoutIndex::new(&sb.build().unwrap(), default_layout());

        assert!(index
            .offset_of(rtype::MBP_0, sale_condition::<[Char; 4]>())
            .is_some());
        assert!(index
            .offset_of(rtype::MBP_0, fields::SALE_CONDITION)
            .is_none());
        assert!(index
            .offset_of(rtype::MBP_0, sale_condition::<[Char; 8]>())
            .is_none());
    }

    #[test]
    fn reads_string_field() {
        let mut sb = StreamLayoutBuilder::new(1, 8);
        {
            let mut r = sb.record(rtype::MBP_1, 0, "SymRec");
            r.str_field(Field::<CStr>::new(30), "symbol", 8);
            r.finish();
        }
        let index = LayoutIndex::new(&sb.build().unwrap(), default_layout());

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
    fn an_rtype_the_stream_omits_has_no_verdict() {
        let index = mbo_index();
        assert_eq!(index.verdict(rtype::MBO), Some(Verdict::Compiled));
        assert!(index.verdict(rtype::MBP_0).is_none());
        assert!(index.offset_of(rtype::MBP_0, fields::PRICE).is_none());
    }

    fn trades_with_ext(declare: impl FnOnce(&mut RecordLayoutBuilder<'_>)) -> StreamLayout {
        let mut sb = StreamLayoutBuilder::new(1, 8);
        {
            let mut r = sb.record(rtype::MBP_0, 0, "TradeMsg");
            TradeMsg::add_fields(&mut r);
            r.finish_base();
            declare(&mut r);
            r.finish();
        }
        sb.build().unwrap()
    }

    fn trades_with_ts_out() -> StreamLayout {
        trades_with_ext(|r| {
            r.field(fields::TS_OUT, "ts_out");
        })
    }

    fn trades_opra_sale_condition() -> StreamLayout {
        trades_with_ext(|r| {
            r.field(sale_condition::<Char>(), "sale_condition");
        })
    }

    fn trades_sip_sale_condition() -> StreamLayout {
        trades_with_ext(|r| {
            r.field(sale_condition::<[Char; 4]>(), "sale_condition");
        })
    }

    fn sale_condition<T: FieldType>() -> Field<T> {
        Field::new(fields::SALE_CONDITION.id())
    }

    fn sample_trade() -> v4::TradeMsg {
        v4::TradeMsg {
            hd: RecordHeader::new::<v4::TradeMsg>(rtype::MBP_0, 1, 555, 111),
            price: 123_000_000_000,
            size: 7,
            action: b'T' as c_char,
            side: b'B' as c_char,
            flags: FlagSet::new(0b1010),
            _reserved: 0,
            ts_recv: 222,
            ts_in_delta: 9,
            sequence: 88,
        }
    }

    fn trade_with_ext<const TOTAL: usize>(ext: &[u8]) -> v4::RecordBuf<TOTAL> {
        let mut buf = v4::RecordBuf::<TOTAL>::from(sample_trade());
        let raw = buf.raw_buf_mut();
        raw[size_of::<v4::TradeMsg>()..][..ext.len()].copy_from_slice(ext);
        raw[..2].copy_from_slice(&((TOTAL - size_of::<RecordHeader>()) as u16).to_le_bytes());
        buf
    }

    fn trades_repacked() -> StreamLayout {
        let mut sb = StreamLayoutBuilder::new(1, 8);
        {
            let mut r = sb.record(rtype::MBP_0, 0, "TradeMsg");
            r.field(fields::PRICE, "price")
                .field(Field::<Decimal>::new(fields::SIZE.id()), "size")
                .field(fields::ACTION, "action")
                .field(fields::SIDE, "side")
                .field(fields::FLAGS, "flags")
                .field(fields::TS_RECV, "ts_recv")
                .field(fields::TS_IN_DELTA, "ts_in_delta")
                .field(fields::SEQUENCE, "sequence");
            r.finish();
        }
        sb.build().unwrap()
    }

    #[rstest]
    #[case::compiled_layout(trades_layout(), Verdict::Compiled)]
    #[case::appended_extension(trades_with_ts_out(), Verdict::Compiled)]
    #[case::repacked_size(trades_repacked(), Verdict::Dynamic)]
    fn verdict_authorizes_the_compiled_cast(
        #[case] stream: StreamLayout,
        #[case] expected: Verdict,
    ) {
        let index = LayoutIndex::new(&stream, default_layout());
        assert_eq!(index.verdict(rtype::MBP_0), Some(expected));
    }

    #[test]
    fn uncompiled_rtype_is_dynamic() {
        let uncompiled = 0x9999;
        let mut sb = StreamLayoutBuilder::new(1, 8);
        {
            let mut r = sb.record(uncompiled, 0, "Unknown");
            r.field(fields::PRICE, "price");
            r.finish();
        }
        let index = LayoutIndex::new(&sb.build().unwrap(), default_layout());
        assert_eq!(index.verdict(uncompiled), Some(Verdict::Dynamic));
    }

    #[test]
    fn a_matching_hash_is_trusted() {
        let mut stream = trades_repacked();
        stream.record_layouts[0].layout_hash = trades_layout().record_layouts[0].layout_hash;
        let index = LayoutIndex::new(&stream, default_layout());
        assert_eq!(index.verdict(rtype::MBP_0), Some(Verdict::Compiled));
    }

    #[test]
    fn ts_out_reads_from_the_extensions() {
        let index = LayoutIndex::new(&trades_with_ts_out(), default_layout());
        let rec = trade_with_ext::<64>(&777u64.to_le_bytes());

        assert_eq!(rec.record_size(), 64);
        assert_eq!(rec.get::<v4::TradeMsg>().unwrap().price, 123_000_000_000);
        assert_eq!(rec.field(&index, fields::TS_OUT).map(|t| t.0), Some(777));
    }

    #[test]
    fn absent_extension_reads_as_none() {
        let index = LayoutIndex::new(&trades_with_ts_out(), default_layout());
        let trade = sample_trade();
        let rec = crate::RecordRef::from(&trade);

        assert!(rec.field(&index, fields::TS_OUT).is_none());
        assert!(index.offset_of(rtype::MBP_0, fields::TS_OUT).is_some());
    }

    #[test]
    fn sale_condition_reads_at_each_datasets_type() {
        let opra_idx = LayoutIndex::new(&trades_opra_sale_condition(), default_layout());
        let rec = trade_with_ext::<64>(b"@");
        assert_eq!(
            rec.field(&opra_idx, sale_condition::<Char>()),
            Some(Char(b'@' as c_char))
        );

        let sip_idx = LayoutIndex::new(&trades_sip_sale_condition(), default_layout());
        let rec = trade_with_ext::<64>(b"@ TI");
        assert_eq!(
            rec.field(&sip_idx, sale_condition::<[Char; 4]>()),
            Some((*b"@ TI").map(|b| Char(b as c_char)))
        );
        assert!(rec.field(&sip_idx, sale_condition::<Char>()).is_none());
        assert!(rec
            .field(&opra_idx, sale_condition::<[Char; 4]>())
            .is_none());
    }

    #[rstest]
    #[case::opra(trades_opra_sale_condition(), 1)]
    #[case::sip(trades_sip_sale_condition(), 4)]
    fn sale_condition_layouts_stay_castable(#[case] stream: StreamLayout, #[case] size: u16) {
        let index = LayoutIndex::new(&stream, default_layout());
        assert_eq!(index.verdict(rtype::MBP_0), Some(Verdict::Compiled));

        let rec = stream.record_layout(rtype::MBP_0).unwrap();
        assert_eq!(usize::from(rec.base_record_size), size_of::<v4::TradeMsg>());
        let cond = rec
            .field_defs
            .iter()
            .find(|f| f.field_id == fields::SALE_CONDITION.id())
            .unwrap();
        assert_eq!(cond.type_id, <Char as FieldType>::TYPE_ID);
        assert_eq!(cond.size, size);
        assert!(cond.is_extension());
        assert_eq!(cond.is_array(), size > 1);
    }

    #[rstest]
    #[case::ts_out(trades_with_ts_out())]
    #[case::opra(trades_opra_sale_condition())]
    #[case::sip(trades_sip_sale_condition())]
    fn extensions_are_8_aligned(#[case] stream: StreamLayout) {
        let rec = stream.record_layout(rtype::MBP_0).unwrap();
        assert_eq!(
            rec.record_size() % 8,
            0,
            "extensions end at {}",
            rec.record_size()
        );
    }

    #[rstest]
    #[case::record(|l: &mut StreamLayout| {
        l.record_layouts[0].field_defs.reverse();
        l.record_layouts[0].field_names.reverse();
    })]
    #[case::header(|l: &mut StreamLayout| {
        l.struct_layouts[0].field_defs.reverse();
        l.struct_layouts[0].field_names.reverse();
    })]
    fn display_order_doesnt_affect_verdict(#[case] reorder: fn(&mut StreamLayout)) {
        let mut stream = mbo_layout();
        reorder(&mut stream);
        let rec = &mut stream.record_layouts[0];
        rec.layout_hash = rec.layout_hash(&stream.struct_layouts);
        assert_eq!(
            rec.layout_hash,
            default_layout()
                .record_layout(rtype::MBO)
                .unwrap()
                .layout_hash
        );
        let index = LayoutIndex::new(&stream, default_layout());
        assert_eq!(index.verdict(rtype::MBO), Some(Verdict::Compiled));
    }

    #[test]
    fn validate_accepts_builder_output() {
        assert!(validate::validate(&mbo_layout(), 8).is_ok());
    }

    #[rstest]
    #[case::extension_flag_within_base(|l: &mut StreamLayout| {
        l.record_layouts[0]
            .field_defs
            .iter_mut()
            .find(|f| f.field_id == fields::PRICE.id())
            .unwrap()
            .flags |= fields::FLAG_IS_EXTENSION;
    })]
    #[case::misaligned_offset(|l: &mut StreamLayout| {
        l.record_layouts[0]
            .field_defs
            .iter_mut()
            .find(|f| f.field_id == fields::PRICE.id())
            .unwrap()
            .offset = 33;
    })]
    #[case::field_past_base(
        |l: &mut StreamLayout| l.record_layouts[0].field_defs.last_mut().unwrap().offset += 8
    )]
    #[case::coverage_hole(|l: &mut StreamLayout| {
        l.record_layouts[0].field_defs.remove(0);
        l.record_layouts[0].field_names.remove(0);
    })]
    #[case::field_wider_than_its_type(|l: &mut StreamLayout| {
        l.record_layouts[0]
            .field_defs
            .iter_mut()
            .find(|f| f.field_id == fields::SIZE.id())
            .unwrap()
            .size = 8;
    })]
    #[case::struct_array_size_not_a_multiple(|l: &mut StreamLayout| {
        let f = l.record_layouts[0]
            .field_defs
            .iter_mut()
            .find(|f| f.field_id == fields::HD.id())
            .unwrap();
        f.flags |= fields::FLAG_IS_ARRAY;
        f.size = 5;
    })]
    #[case::duplicate_rtype(|l: &mut StreamLayout| {
        let dup = l.record_layouts[0].clone();
        l.record_layouts.push(dup);
    })]
    #[case::rtype_reserved_for_the_metadata_magic(|l: &mut StreamLayout| {
        l.record_layouts[0].rtype = 0x014E;
    })]
    #[case::duplicate_field_id(|l: &mut StreamLayout| {
        let first = l.record_layouts[0].field_defs[0].field_id;
        l.record_layouts[0].field_defs[1].field_id = first;
    })]
    #[case::name_count_mismatch(|l: &mut StreamLayout| {
        l.record_layouts[0].field_names.push("extra".to_owned());
    })]
    #[case::struct_def_index_out_of_range(|l: &mut StreamLayout| {
        l.record_layouts[0]
            .field_defs
            .iter_mut()
            .find(|f| f.field_id == fields::HD.id())
            .unwrap()
            .def_index = 999;
    })]
    #[case::label_def_index_out_of_range(|l: &mut StreamLayout| {
        l.record_layouts[0]
            .field_defs
            .iter_mut()
            .find(|f| fields::uses_label_def(f.type_id))
            .unwrap()
            .def_index = 999;
    })]
    #[case::struct_nested_in_struct(|l: &mut StreamLayout| {
        l.struct_layouts[0].field_defs[0].type_id = fields::STRUCT_ID;
    })]
    #[case::struct_size_not_a_multiple_of_its_alignment(|l: &mut StreamLayout| {
        l.struct_layouts[0].size += 1;
    })]
    #[case::symbol_wider_than_the_stream_declares(|l: &mut StreamLayout| {
        let rec = &mut l.record_layouts[0];
        rec.field_defs.push(FieldDef {
            field_id: fields::RAW_SYMBOL.id(),
            offset: rec.base_record_size,
            size: 16,
            type_id: fields::SYMBOL_ID,
            flags: 0,
            def_index: 0,
            scale: 0,
            _reserved: [0; 5],
        });
        rec.field_names.push("raw_symbol".to_owned());
        rec.base_record_size += 16;
    })]
    #[case::zero_size_struct_array(|l: &mut StreamLayout| {
        l.struct_layouts.push(StructLayout {
            size: 0,
            name: "Empty".to_owned(),
            field_defs: Vec::new(),
            field_names: Vec::new(),
        });
        let rec = &mut l.record_layouts[0];
        rec.field_defs.push(FieldDef {
            field_id: 0x0031,
            offset: rec.base_record_size,
            size: 0,
            type_id: fields::STRUCT_ID,
            flags: fields::FLAG_IS_ARRAY | fields::FLAG_IS_EXTENSION,
            def_index: (l.struct_layouts.len() - 1) as u16,
            scale: 0,
            _reserved: [0; 5],
        });
        rec.field_names.push("empties".to_owned());
    })]
    #[case::zero_size_string_array(|l: &mut StreamLayout| {
        let rec = &mut l.record_layouts[0];
        rec.field_defs.push(FieldDef {
            field_id: 0x0030,
            offset: rec.base_record_size,
            size: 0,
            type_id: fields::CSTR_ID,
            flags: fields::FLAG_IS_ARRAY | fields::FLAG_IS_EXTENSION,
            def_index: 0,
            scale: 0,
            _reserved: [0; 5],
        });
        rec.field_names.push("raw_symbols".to_owned());
    })]
    fn validate_rejects_corruption(#[case] corrupt: fn(&mut StreamLayout)) {
        let mut bad = mbo_layout();
        corrupt(&mut bad);
        assert!(validate::validate(&bad, 8).is_err());
    }

    #[test]
    fn offset_of_checks_enum_identity() {
        let index = mbo_index();
        assert!(index.offset_of(rtype::MBO, fields::SIDE).is_some());
        let side_as_action = Field::<OpenEnum<Action>>::new(fields::SIDE.id());
        assert!(index.offset_of(rtype::MBO, side_as_action).is_none());
    }

    #[test]
    fn field_by_name_and_id_agree() {
        let index = mbo_index();
        let side = index.field_by_name(rtype::MBO, "side").unwrap();
        assert_eq!(index.field_by_id(rtype::MBO, fields::SIDE.id()), Some(side));
        assert_eq!(side.label_def().unwrap().name, "Side");
        assert!(index.field_by_name(rtype::MBO, "stat_type").is_none());
        assert!(index.field_by_name(rtype::MBP_0, "side").is_none());
    }

    #[rstest]
    #[case::known(b'B', Some("BID"))]
    #[case::unknown(b'X', None)]
    fn enum_at_reads_a_char_enum(#[case] side: u8, #[case] label: Option<&str>) {
        let index = mbo_index();
        let mbo = MboMsg {
            side: side as c_char,
            ..sample_mbo()
        };
        let value = mbo
            .enum_at(&index.field_by_name(rtype::MBO, "side").unwrap())
            .unwrap();
        assert_eq!(value.raw(), u16::from(side));
        assert_eq!(value.label(), label);
        assert_eq!(value.enum_name(), Some("Side"));
    }

    #[rstest]
    #[case::u16("stat_type", "StatType", "OPEN_INTEREST")]
    #[case::u8("update_action", "StatUpdateAction", "DELETE")]
    fn enum_at_reads_numeric_enums(
        #[case] name: &str,
        #[case] enum_name: &str,
        #[case] label: &str,
    ) {
        let index = LayoutIndex::new(default_layout(), default_layout());
        let stat = StatMsg {
            stat_type: StatType::OpenInterest as u16,
            update_action: StatUpdateAction::Delete as u8,
            ..Default::default()
        };
        let value = stat
            .enum_at(&index.field_by_name(rtype::STATISTICS, name).unwrap())
            .unwrap();
        assert_eq!(value.enum_name(), Some(enum_name));
        assert_eq!(value.label(), Some(label));
    }

    #[test]
    fn bit_set_at_labels_set_bits() {
        let index = mbo_index();
        let mbo = MboMsg {
            flags: FlagSet::new(flags::LAST | flags::BAD_TS_RECV | 1),
            ..sample_mbo()
        };
        let value = mbo
            .bit_set_at(&index.field_by_name(rtype::MBO, "flags").unwrap())
            .unwrap();
        assert_eq!(value.raw(), 0x89);
        assert_eq!(value.labels().collect::<Vec<_>>(), ["BAD_TS_RECV", "LAST"]);
    }

    #[test]
    fn dyn_reads_reject_the_wrong_kind_and_rtype() {
        let index = LayoutIndex::new(default_layout(), default_layout());
        let mbo = sample_mbo();
        let side = index.field_by_name(rtype::MBO, "side").unwrap();
        let flags = index.field_by_name(rtype::MBO, "flags").unwrap();
        assert!(mbo.enum_at(&flags).is_none());
        assert!(mbo.bit_set_at(&side).is_none());
        let stat_type = index.field_by_name(rtype::STATISTICS, "stat_type").unwrap();
        assert!(mbo.enum_at(&stat_type).is_none());
    }
}
