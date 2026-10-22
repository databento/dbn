//! Field registry for encoding in the schema definitions and dynamic field access.

use std::{ffi::c_char, marker::PhantomData};

use crate::{
    flags,
    v4::{
        types::{CStr, Char, Decimal, EnumField, Labels, OpenEnum, Symbol, TimestampNs},
        RecordHeader,
    },
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
    /// The labels of an enum or bit set type, `None` for every other type.
    const LABELS: Option<Labels> = None;
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
///
/// ```compile_fail,E0277
/// use dbn::{layout::RecordLayoutBuilder, v4::{fields::Field, types::CStr}};
/// fn bad(b: &mut RecordLayoutBuilder) {
///     b.field(Field::<CStr>::new(0x0030), "raw_symbol");
/// }
/// ```
pub trait FixedWidth: FieldType {
    /// The field's wire size in bytes, which is its type ID's width.
    const WIRE_SIZE: u16 = match scalar_size(Self::TYPE_ID) {
        Some(size) => size,
        None => panic!("FixedWidth type without a scalar width"),
    };
    /// The type a compiled record struct stores the field as.
    type Storage;
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
impl DecodeField for Symbol {
    type Value<'a> = &'a str;
    fn decode(bytes: &[u8], scale: i8) -> Option<&str> {
        CStr::decode(bytes, scale)
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
impl<T, const N: usize> DecodeField for [T; N]
where
    T: FixedWidth + for<'x> DecodeField<Value<'x> = T>,
{
    type Value<'a> = [T; N];
    fn decode(bytes: &[u8], scale: i8) -> Option<[T; N]> {
        let width = usize::from(T::WIRE_SIZE);
        let mut out: [Option<T>; N] = [const { None }; N];
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = Some(T::decode(bytes.get(i * width..(i + 1) * width)?, scale)?);
        }
        Some(out.map(Option::unwrap))
    }
}

/// A scalar field's type read from its `type_id` at runtime. Enums and bit sets read
/// as their raw unsigned value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScalarType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Decimal,
    Timestamp,
    Char,
    Str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ScalarValue<'a> {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Decimal(Decimal),
    Timestamp(TimestampNs),
    Char(Char),
    Str(&'a str),
}

impl ScalarType {
    /// `None` for a `Struct` or `Padding` tag or an unknown type ID.
    pub(crate) fn new(type_id: u8) -> Option<Self> {
        Some(match type_id {
            <u8 as FieldType>::TYPE_ID | ENUM_8_ID | BIT_SET_8_ID => Self::U8,
            <u16 as FieldType>::TYPE_ID | ENUM_16_ID | BIT_SET_16_ID => Self::U16,
            <u32 as FieldType>::TYPE_ID | BIT_SET_32_ID => Self::U32,
            <u64 as FieldType>::TYPE_ID | BIT_SET_64_ID => Self::U64,
            <i8 as FieldType>::TYPE_ID => Self::I8,
            <i16 as FieldType>::TYPE_ID => Self::I16,
            <i32 as FieldType>::TYPE_ID => Self::I32,
            <i64 as FieldType>::TYPE_ID => Self::I64,
            <f32 as FieldType>::TYPE_ID => Self::F32,
            <f64 as FieldType>::TYPE_ID => Self::F64,
            <Decimal as FieldType>::TYPE_ID => Self::Decimal,
            <TimestampNs as FieldType>::TYPE_ID => Self::Timestamp,
            <Char as FieldType>::TYPE_ID | ENUM_CHAR_ID => Self::Char,
            CSTR_ID | SYMBOL_ID => Self::Str,
            _ => return None,
        })
    }

    /// Decodes like [`DecodeField::decode`]: `None` for a short window or non-UTF-8 in
    /// a string.
    pub(crate) fn decode(self, bytes: &[u8], scale: i8) -> Option<ScalarValue<'_>> {
        Some(match self {
            Self::U8 => ScalarValue::U8(u8::decode(bytes, scale)?),
            Self::U16 => ScalarValue::U16(u16::decode(bytes, scale)?),
            Self::U32 => ScalarValue::U32(u32::decode(bytes, scale)?),
            Self::U64 => ScalarValue::U64(u64::decode(bytes, scale)?),
            Self::I8 => ScalarValue::I8(i8::decode(bytes, scale)?),
            Self::I16 => ScalarValue::I16(i16::decode(bytes, scale)?),
            Self::I32 => ScalarValue::I32(i32::decode(bytes, scale)?),
            Self::I64 => ScalarValue::I64(i64::decode(bytes, scale)?),
            Self::F32 => ScalarValue::F32(f32::decode(bytes, scale)?),
            Self::F64 => ScalarValue::F64(f64::decode(bytes, scale)?),
            Self::Decimal => ScalarValue::Decimal(Decimal::decode(bytes, scale)?),
            Self::Timestamp => ScalarValue::Timestamp(TimestampNs::decode(bytes, scale)?),
            Self::Char => ScalarValue::Char(Char::decode(bytes, scale)?),
            Self::Str => ScalarValue::Str(CStr::decode(bytes, scale)?),
        })
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldDefault {
    Max,
    Value(i64),
}

pub(crate) fn default_of(field_id: u16, type_id: u8) -> FieldDefault {
    registry_default(field_id).unwrap_or(
        if type_id == Decimal::TYPE_ID || type_id == TimestampNs::TYPE_ID {
            FieldDefault::Max
        } else {
            FieldDefault::Value(0)
        },
    )
}

impl FieldDefault {
    pub(crate) fn write(self, type_id: u8, bytes: &mut [u8]) {
        match self {
            Self::Max => {
                bytes.fill(u8::MAX);
                let signed = [
                    i8::TYPE_ID,
                    i16::TYPE_ID,
                    i32::TYPE_ID,
                    i64::TYPE_ID,
                    Decimal::TYPE_ID,
                ];
                if signed.contains(&type_id) {
                    if let Some(last) = bytes.last_mut() {
                        *last = i8::MAX as u8;
                    }
                }
            }
            Self::Value(value) => {
                let value = value.to_le_bytes();
                let (head, tail) = bytes.split_at_mut(bytes.len().min(value.len()));
                head.copy_from_slice(&value[..head.len()]);
                tail.fill(0);
            }
        }
    }
}

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
impl FieldType for Symbol {
    const TYPE_ID: u8 = SYMBOL_ID;
}
impl FieldType for RecordHeader {
    const TYPE_ID: u8 = STRUCT_ID;
}
impl<T: FixedWidth, const N: usize> FieldType for [T; N] {
    const TYPE_ID: u8 = T::TYPE_ID;
    const ARRAY_WIRE_SIZE: Option<u16> = Some(T::WIRE_SIZE * N as u16);
    const LABELS: Option<Labels> = T::LABELS;
}
impl<E> FieldType for OpenEnum<E>
where
    E: EnumField,
{
    const TYPE_ID: u8 = E::TYPE_ID;
    const LABELS: Option<Labels> = Some(Labels {
        name: E::NAME,
        values: E::LABELS,
    });
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
/// Type ID for a symbol, a C string with the stream's symbol width.
pub const SYMBOL_ID: u8 = 0x0F;
/// Type ID for a nested struct.
pub const STRUCT_ID: u8 = 0x40;
/// Type ID for padding.
pub const PADDING_ID: u8 = 0xFF;

impl FieldType for FlagSet {
    const TYPE_ID: u8 = BIT_SET_8_ID;
    const LABELS: Option<Labels> = Some(Labels {
        name: "FlagSet",
        values: &[
            (flags::LAST.trailing_zeros() as u16, "LAST"),
            (flags::TOB.trailing_zeros() as u16, "TOB"),
            (flags::SNAPSHOT.trailing_zeros() as u16, "SNAPSHOT"),
            (flags::MBP.trailing_zeros() as u16, "MBP"),
            (flags::BAD_TS_RECV.trailing_zeros() as u16, "BAD_TS_RECV"),
            (
                flags::MAYBE_BAD_BOOK.trailing_zeros() as u16,
                "MAYBE_BAD_BOOK",
            ),
            (
                flags::PUBLISHER_SPECIFIC.trailing_zeros() as u16,
                "PUBLISHER_SPECIFIC",
            ),
        ],
    });
}

macro_rules! impl_fixed_width {
    ($($t:ty => $storage:ty),*) => {$(
        impl FixedWidth for $t {
            type Storage = $storage;
        }
    )*};
}
impl_fixed_width!(
    Decimal => i64,
    u8 => u8,
    u16 => u16,
    u32 => u32,
    u64 => u64,
    i8 => i8,
    i16 => i16,
    i32 => i32,
    i64 => i64,
    f32 => f32,
    f64 => f64,
    TimestampNs => u64,
    Char => c_char,
    FlagSet => FlagSet
);

impl<T: FixedWidth, const N: usize> FixedWidth for [T; N] {
    const WIRE_SIZE: u16 = T::WIRE_SIZE * N as u16;
    type Storage = [T::Storage; N];
}
impl<E: EnumField> FixedWidth for OpenEnum<E> {
    type Storage = E::Storage;
}

/// The byte width and natural alignment of a scalar element of `type_id`, or
/// `None` for `CStr` and `Symbol`, whose width the dataset declares, and for aggregate
/// (`Struct`) and `Padding` tags. For an array field the element width is returned,
/// which is also the field's alignment.
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

    use rstest::rstest;

    use super::*;
    use crate::{
        layout::{RecordLayoutDesc, StreamLayoutBuilder},
        HasRType,
    };

    #[rstest]
    #[case::enum_char(ENUM_CHAR_ID, b"B", Some(ScalarValue::Char(Char(b'B' as c_char))))]
    #[case::bit_set_16(BIT_SET_16_ID, &[0x01, 0x02], Some(ScalarValue::U16(0x0201)))]
    #[case::price(
        <Decimal as FieldType>::TYPE_ID,
        &5_500_000_000_i64.to_le_bytes(),
        Some(ScalarValue::Decimal(Decimal { raw: 5_500_000_000, scale: -9 }))
    )]
    #[case::symbol(SYMBOL_ID, b"ESZ6\0\0\0\0", Some(ScalarValue::Str("ESZ6")))]
    #[case::non_utf8(CSTR_ID, b"\xFFZ6\0", None)]
    #[case::short(<u32 as FieldType>::TYPE_ID, &[0x01, 0x02], None)]
    fn scalar_type_decodes(
        #[case] type_id: u8,
        #[case] bytes: &[u8],
        #[case] expected: Option<ScalarValue<'static>>,
    ) {
        assert_eq!(
            ScalarType::new(type_id).unwrap().decode(bytes, -9),
            expected
        );
    }

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
            HD.id(),
        ];
        let mut seen = HashSet::new();
        for id in ids {
            assert!(seen.insert(id), "duplicate field ID {id:#06X}");
        }
    }

    fn non_registry_defaults<R: RecordLayoutDesc + HasRType>(
        rec: R,
        mut sb: StreamLayoutBuilder,
    ) -> Vec<String> {
        sb.layout_of::<R>(rec.raw_rtype()).finish();
        let layout = sb.into_unvalidated().record_layouts.pop().unwrap();
        let bytes = rec.as_ref();
        layout
            .field_defs
            .iter()
            .zip(&layout.field_names)
            .filter(|(f, _)| !f.is_padding() && f.field_id != HD.id() && !f.is_array())
            .filter(|(f, _)| usize::from(f.offset + f.size) <= bytes.len())
            .filter(|(f, _)| {
                let mut expected = vec![0; usize::from(f.size)];
                default_of(f.field_id, f.type_id).write(f.type_id, &mut expected);
                let start = usize::from(f.offset);
                bytes[start..start + expected.len()] != expected
            })
            .map(|(_, name)| format!("{}.{name}", R::NAME))
            .collect()
    }

    macro_rules! non_registry_defaults {
        ($v:ident, $sb:expr, $mbp1:expr) => {{
            use crate::$v::*;
            use crate::Schema;
            [
                non_registry_defaults(MboMsg::default(), $sb),
                non_registry_defaults(TradeMsg::default(), $sb),
                non_registry_defaults($mbp1, $sb),
                non_registry_defaults(Mbp10Msg::default(), $sb),
                non_registry_defaults(BboMsg::default_for_schema(Schema::Bbo1S), $sb),
                non_registry_defaults(Cmbp1Msg::default_for_schema(Schema::Cmbp1), $sb),
                non_registry_defaults(CbboMsg::default_for_schema(Schema::Cbbo1S), $sb),
                non_registry_defaults(OhlcvMsg::default_for_schema(Schema::Ohlcv1S), $sb),
                non_registry_defaults(StatusMsg::default(), $sb),
                non_registry_defaults(InstrumentDefMsg::default(), $sb),
                non_registry_defaults(ImbalanceMsg::default(), $sb),
                non_registry_defaults(StatMsg::default(), $sb),
                non_registry_defaults(ErrorMsg::default(), $sb),
                non_registry_defaults(SymbolMappingMsg::default(), $sb),
                non_registry_defaults(SystemMsg::default(), $sb),
            ]
            .concat()
        }};
    }

    #[test]
    fn record_defaults_match_the_registry() {
        let mismatched = [
            non_registry_defaults!(
                v1,
                StreamLayoutBuilder::pre_v4(crate::v1::SYMBOL_CSTR_LEN as u16),
                Mbp1Msg::default()
            ),
            non_registry_defaults!(
                v2,
                StreamLayoutBuilder::pre_v4(crate::v2::SYMBOL_CSTR_LEN as u16),
                Mbp1Msg::default()
            ),
            non_registry_defaults!(
                v3,
                StreamLayoutBuilder::pre_v4(crate::v3::SYMBOL_CSTR_LEN as u16),
                Mbp1Msg::default()
            ),
            non_registry_defaults!(
                v4,
                StreamLayoutBuilder::new(DBN_VERSION.into(), crate::SYMBOL_CSTR_LEN as u16),
                Mbp1Msg::default_for_schema(Schema::Mbp1)
            ),
        ]
        .concat();
        assert!(mismatched.is_empty(), "{mismatched:?}");
    }
}
