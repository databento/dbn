//! Field registry for encoding in the schema definitions and dynamic field access.

use std::marker::PhantomData;

use crate::{
    v4::types::{CStr, Char, Decimal, EnumField, OpenEnum, TimestampNs},
    FlagSet,
};

/// Whether a `type_id` reads its `def_index` as a [`LabelDef`](crate::layout::LabelDef)
/// index. Every other type pins `def_index` to `0`, except `Struct`, which indexes
/// `struct_layouts`.
pub const fn uses_label_def(type_id: u8) -> bool {
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
    /// `true` if the field is an array.
    const IS_ARRAY: bool = false;
    /// The `FieldDef.scale` a field of this type defaults to (`-9` for fixed-point
    /// prices, `0` otherwise).
    const DEFAULT_SCALE: i8 = 0;
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
    /// The field's wire size in bytes. Usually the Rust size; differs where the
    /// dynamic-access type is wider than the wire type (e.g. `Decimal`).
    const WIRE_SIZE: u16;
}

/// A field constant.
#[derive(Debug, Clone, Copy)]
pub struct Field<T: FieldType> {
    id: u16,
    is_extension: bool,
    _type: PhantomData<T>,
}

impl<T: FieldType> Field<T> {
    /// Create a new field instance with the given field `id`.
    pub const fn new(id: u16) -> Self {
        Self {
            id,
            is_extension: false,
            _type: PhantomData,
        }
    }

    /// Mark this field as an extension field, i.e. not part of the base struct.
    pub const fn as_extension(mut self) -> Self {
        self.is_extension = true;
        self
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
        ((T::IS_ARRAY as u8) * FLAG_IS_ARRAY) | ((self.is_extension as u8) * FLAG_IS_EXTENSION)
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
/// Field ID for the record `length`.
pub const LENGTH: Field<u16> = Field::new(0x01);
/// Field ID for the record type (`rtype`).
pub const RTYPE: Field<u16> = Field::new(0x02);
/// Field ID for `publisher_id`.
pub const PUBLISHER_ID: Field<u16> = Field::new(0x03);
/// Field ID for `instrument_id`.
pub const INSTRUMENT_ID: Field<u64> = Field::new(0x04);
/// Field ID for `ts_event`.
pub const TS_EVENT: Field<TimestampNs> = Field::new(0x05);
/// Field ID for `price`.
pub const PRICE: Field<Decimal> = Field::new(0x10);
/// Field ID for `size`.
pub const SIZE: Field<u32> = Field::new(0x11);
/// Field ID for `flags`.
pub const FLAGS: Field<FlagSet> = Field::new(0x14);
/// Field ID for `order_id`.
pub const ORDER_ID: Field<u64> = Field::new(0x19);
/// Field ID for `channel_id`.
pub const CHANNEL_ID: Field<u8> = Field::new(0x1B);
/// Field ID for `hd`, the record header.
pub const HD: u16 = 0x1000;

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
    const DEFAULT_SCALE: i8 = -9;
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
    const IS_ARRAY: bool = true;
    const DEFAULT_SCALE: i8 = T::DEFAULT_SCALE;
    // Element wire size times the count, not `size_of::<[T; N]>()`, which would use
    // T's Rust size and ignore a wider-than-wire override like `Decimal`.
    const ARRAY_WIRE_SIZE: Option<u16> = Some(T::WIRE_SIZE * N as u16);
}
impl<E> FieldType for OpenEnum<E>
where
    E: EnumField,
{
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

macro_rules! impl_fixed_width_rust_size {
    ($($t:ty),*) => {$(
        impl FixedWidth for $t {
            const WIRE_SIZE: u16 = std::mem::size_of::<$t>() as u16;
        }
    )*};
}
impl_fixed_width_rust_size!(
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

impl FixedWidth for Decimal {
    // Wire `Fixed` is an i64; `Decimal` also carries the scale, so its Rust size is
    // wider than the 8 wire bytes.
    const WIRE_SIZE: u16 = 8;
}
impl<T: FixedWidth, const N: usize> FixedWidth for [T; N] {
    const WIRE_SIZE: u16 = T::WIRE_SIZE * N as u16;
}
impl<E> FixedWidth for OpenEnum<E>
where
    E: EnumField,
{
    const WIRE_SIZE: u16 = std::mem::size_of::<Self>() as u16;
}

/// The byte width and natural alignment of a scalar element of `type_id`, or
/// `None` for aggregate (`Struct`) and `Padding` tags. For an array field the
/// element width is returned, which is also the field's alignment.
pub const fn scalar_size(type_id: u8) -> Option<u16> {
    Some(match type_id {
        0x01 | 0x05 | 0x0D | 0x0E | ENUM_CHAR_ID | ENUM_8_ID | BIT_SET_8_ID => 1,
        0x02 | 0x06 | ENUM_16_ID | BIT_SET_16_ID => 2,
        0x03 | 0x07 | 0x09 | BIT_SET_32_ID => 4,
        0x04 | 0x08 | 0x0A | 0x0B | 0x0C | BIT_SET_64_ID => 8,
        _ => return None,
    })
}
