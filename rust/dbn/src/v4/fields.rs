//! Field registry for encoding in the schema definitions and dynamic field access.

use std::{ffi::c_char, marker::PhantomData};

use crate::{
    v4::types::{CStr, Char, Decimal, EnumField, OpenEnum, TimestampNs},
    FlagSet,
};

// The field-ID registry constants (`PRICE`, `ACTION`, ...) are generated from the
// `[fields]` table in `dbn.toml`; re-exported so they live under `v4::fields`.
mod registry;
pub use registry::*;

/// Whether a `type_id` reads its `def_index` as a [`LabelDef`](crate::layout::LabelDef)
/// index. Every other type pins `def_index` to `0`, except `Struct`, which indexes
/// `struct_layouts`.
pub(crate) const fn uses_label_def(type_id: u8) -> bool {
    matches!(
        type_id,
        ENUM_CHAR_ID
            | ENUM_8_ID
            | ENUM_16_ID
            | BIT_SET_8_ID
            | BIT_SET_16_ID
            | BIT_SET_32_ID
            | BIT_SET_64_ID
    )
}

/// `FieldDef.flags` bit marking an array-typed field.
pub const FLAG_IS_ARRAY: u8 = 1 << 7;
/// `FieldDef.flags` bit marking an extension field (offset >= `base_record_size`).
pub const FLAG_IS_EXTENSION: u8 = 1 << 6;

/// Associated constants for a type DBN fields can have.
pub trait FieldType: Sized {
    /// The type ID of a field.
    const TYPE_ID: u8;
    /// Total wire size when this type is a fixed-count array, `None` otherwise. A
    /// resolver can only check a declared width against the descriptor when the type
    /// fixes it, which arrays do and a per-dataset [`CStr`] does not.
    const ARRAY_WIRE_SIZE: Option<u16> = None;
}

/// A [`FieldType`] whose wire width comes from the type rather than the dataset.
///
/// Not implemented for [`CStr`], whose width is declared per dataset.
///
/// ```compile_fail,E0599
/// use dbn::v4::{fields::Field, types::CStr};
///
/// // A `CStr`'s width is the dataset's to declare, so the type cannot report one.
/// fn bad() -> u16 {
///     Field::<CStr>::new(0x0030).size()
/// }
/// ```
pub trait FixedWidth: FieldType {
    /// The field's wire size in bytes, which is its type ID's width.
    const WIRE_SIZE: u16 = match scalar_size(Self::TYPE_ID) {
        Some(size) => size,
        None => panic!("FixedWidth type without a scalar width"),
    };
}

/// Decodes a field's little-endian wire bytes into the value dynamic access returns. A
/// companion to [`FieldType`] (the encode-side descriptor). The lifetime-generic
/// [`Value`](DecodeField::Value) unifies owned and borrowed reads under one accessor:
/// scalars return an owned value ignoring `'a`, while string, array, and struct fields
/// borrow the record's bytes (e.g. `CStr` returns `&'a str`).
pub trait DecodeField: FieldType {
    /// What decoding this field returns. Borrows the record for borrowed field types.
    type Value<'a>;
    /// Decodes from the field's `size`-byte window. `scale` is the `FieldDef.scale`
    /// (used by [`Decimal`], ignored otherwise). Returns `None` when the bytes can't
    /// form a valid value (a short window, or non-UTF-8 in a string field).
    fn decode(bytes: &[u8], scale: i8) -> Option<Self::Value<'_>>;
}

macro_rules! impl_decode_field_le {
    ($($t:ty),*) => {$(
        impl DecodeField for $t {
            type Value<'a> = $t;
            fn decode(bytes: &[u8], _scale: i8) -> Option<$t> {
                Some(<$t>::from_le_bytes(bytes.get(..std::mem::size_of::<$t>())?.try_into().ok()?))
            }
        }
    )*};
}
impl_decode_field_le!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

impl DecodeField for Decimal {
    type Value<'a> = Decimal;
    fn decode(bytes: &[u8], scale: i8) -> Option<Decimal> {
        Some(Decimal {
            raw: i64::decode(bytes, scale)?,
            scale,
        })
    }
}
impl DecodeField for TimestampNs {
    type Value<'a> = TimestampNs;
    fn decode(bytes: &[u8], scale: i8) -> Option<TimestampNs> {
        u64::decode(bytes, scale).map(TimestampNs)
    }
}
impl DecodeField for Char {
    type Value<'a> = Char;
    fn decode(bytes: &[u8], scale: i8) -> Option<Char> {
        u8::decode(bytes, scale).map(|c| Char(c as c_char))
    }
}
impl DecodeField for FlagSet {
    type Value<'a> = FlagSet;
    fn decode(bytes: &[u8], scale: i8) -> Option<FlagSet> {
        u8::decode(bytes, scale).map(FlagSet::new)
    }
}
impl DecodeField for CStr {
    // Borrows the record: the returned `&str` lives as long as the read window.
    type Value<'a> = &'a str;
    fn decode(bytes: &[u8], _scale: i8) -> Option<&str> {
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        std::str::from_utf8(&bytes[..end]).ok()
    }
}
impl<E> DecodeField for OpenEnum<E>
where
    E: EnumField,
    // The enum's raw representation reads as itself (an owned scalar).
    E::Repr: for<'a> DecodeField<Value<'a> = E::Repr>,
{
    type Value<'a> = OpenEnum<E>;
    fn decode(bytes: &[u8], scale: i8) -> Option<OpenEnum<E>> {
        Some(OpenEnum::from_raw(E::Repr::decode(bytes, scale)?))
    }
}
// Array (`[T; N]` -> `&'a [T]`) and `Struct` (-> `&'a S`) reads slot in the same way:
// further `DecodeField` impls with a borrowing `Value<'a>`.

/// A field constant.
#[derive(Debug, Clone, Copy)]
pub struct Field<T: FieldType> {
    id: u16,
    _type: PhantomData<T>,
}

impl<T: FieldType> Field<T> {
    /// Create a new field instance with the given field `id`.
    pub const fn new(id: u16) -> Self {
        Self {
            id,
            _type: PhantomData,
        }
    }

    /// The unique ID for the field.
    pub const fn id(&self) -> u16 {
        self.id
    }

    /// The type ID for the field.
    pub const fn type_id(&self) -> u8 {
        T::TYPE_ID
    }

    /// The [`FieldDef::flags`](crate::layout::FieldDef::flags) for this field.
    pub const fn flags(&self) -> u8 {
        (T::ARRAY_WIRE_SIZE.is_some() as u8) * FLAG_IS_ARRAY
    }
}

impl<T: FixedWidth> Field<T> {
    /// The wire size of the field. Absent for [`CStr`], whose width the dataset
    /// declares rather than the type.
    pub const fn size(&self) -> u16 {
        T::WIRE_SIZE
    }
}

/// Field ID for padding fields.
pub const PADDING: u16 = 0;

impl FieldType for u8 {
    const TYPE_ID: u8 = 0x01;
}
impl FieldType for u16 {
    const TYPE_ID: u8 = 0x02;
}
impl FieldType for u32 {
    const TYPE_ID: u8 = 0x03;
}
impl FieldType for u64 {
    const TYPE_ID: u8 = 0x04;
}
impl FieldType for i8 {
    const TYPE_ID: u8 = 0x05;
}
impl FieldType for i16 {
    const TYPE_ID: u8 = 0x06;
}
impl FieldType for i32 {
    const TYPE_ID: u8 = 0x07;
}
impl FieldType for i64 {
    const TYPE_ID: u8 = 0x08;
}
impl FieldType for f32 {
    const TYPE_ID: u8 = 0x09;
}
impl FieldType for f64 {
    const TYPE_ID: u8 = 0x0A;
}
impl FieldType for Decimal {
    const TYPE_ID: u8 = 0x0B;
}
impl FieldType for TimestampNs {
    const TYPE_ID: u8 = 0x0C;
}
impl FieldType for Char {
    const TYPE_ID: u8 = 0x0D;
}
impl FieldType for CStr {
    const TYPE_ID: u8 = CSTR_ID;
    // not an array of c_char; it's own semantic type
}
impl<T: FixedWidth, const N: usize> FieldType for [T; N] {
    const TYPE_ID: u8 = T::TYPE_ID;
    const ARRAY_WIRE_SIZE: Option<u16> = Some(T::WIRE_SIZE * N as u16);
}
impl<E> FieldType for OpenEnum<E>
where
    E: EnumField,
{
    // TODO: check enum identity through the field's `LabelDef`. Enums of the same
    // width share a type ID, so the tag alone can't tell `Side` from `Action`.
    const TYPE_ID: u8 = E::TYPE_ID;
}
/// Type ID for a C character enum.
pub const ENUM_CHAR_ID: u8 = 0x20;
/// Type ID for a 1-byte enum.
pub const ENUM_8_ID: u8 = 0x21;
/// Type ID for a 2-byte enum.
pub const ENUM_16_ID: u8 = 0x22;
/// Type ID for a 8-bit bit set.
pub const BIT_SET_8_ID: u8 = 0x30;
/// Type ID for a 16-bit bit set.
pub const BIT_SET_16_ID: u8 = 0x31;
/// Type ID for a 32-bit bit set.
pub const BIT_SET_32_ID: u8 = 0x32;
/// Type ID for a 64-bit bit set.
pub const BIT_SET_64_ID: u8 = 0x33;
/// Type ID for a C string.
pub const CSTR_ID: u8 = 0x0E;
/// Type ID for a nested struct.
pub const STRUCT_ID: u8 = 0x40;
/// Type ID for padding.
pub const PADDING_ID: u8 = 0xFF;

impl FieldType for FlagSet {
    const TYPE_ID: u8 = BIT_SET_8_ID;
}

macro_rules! impl_fixed_width {
    ($($t:ty),*) => {$(
        impl FixedWidth for $t {}
    )*};
}
impl_fixed_width!(
    Decimal,
    u8,
    u16,
    u32,
    u64,
    i8,
    i16,
    i32,
    i64,
    f32,
    f64,
    TimestampNs,
    Char,
    FlagSet
);

impl<T: FixedWidth, const N: usize> FixedWidth for [T; N] {
    const WIRE_SIZE: u16 = T::WIRE_SIZE * N as u16;
}
impl<E: EnumField> FixedWidth for OpenEnum<E> {}

/// The byte width and natural alignment of a scalar element of `type_id`, or
/// `None` for `CStr`, whose width the dataset declares, and for aggregate (`Struct`)
/// and `Padding` tags. For an array field the element width is returned, which is
/// also the field's alignment.
pub(crate) const fn scalar_size(type_id: u8) -> Option<u16> {
    Some(match type_id {
        0x01 | 0x05 | 0x0D | ENUM_CHAR_ID | ENUM_8_ID | BIT_SET_8_ID => 1,
        0x02 | 0x06 | ENUM_16_ID | BIT_SET_16_ID => 2,
        0x03 | 0x07 | 0x09 | BIT_SET_32_ID => 4,
        0x04 | 0x08 | 0x0A | 0x0B | 0x0C | BIT_SET_64_ID => 8,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn registry_field_ids_are_unique() {
        let ids = [
            PADDING,
            LENGTH.id(),
            RTYPE.id(),
            PUBLISHER_ID.id(),
            INSTRUMENT_ID.id(),
            TS_EVENT.id(),
            PRICE.id(),
            SIZE.id(),
            FLAGS.id(),
            ORDER_ID.id(),
            CHANNEL_ID.id(),
            HD,
        ];
        let mut seen = HashSet::new();
        for id in ids {
            assert!(seen.insert(id), "duplicate field ID {id:#06X}");
        }
    }
}
