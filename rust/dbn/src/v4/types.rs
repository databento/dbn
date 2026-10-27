//! Types for fields with no direct Rust primitive, used by the v4 field registry.

use std::{ffi::c_char, marker::PhantomData};

/// Nanosecond-precision UNIX epoch timestamp.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimestampNs(pub u64);

/// The dynamic-access carrier for a fixed-point value: a raw integer and its
/// scale. `Fixed` is the wire `type_id` (`FieldDef.scale` gives the exponent);
/// `Decimal` is what typed dynamic access returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal {
    /// The unscaled integer value.
    pub raw: i64,
    /// The base-10 exponent applied to `raw`.
    pub scale: i8,
}

/// The name and labels of an enum or bit set type, the source of its
/// [`LabelDef`](crate::layout::LabelDef).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Labels {
    /// The type's name, e.g. `Side`.
    pub name: &'static str,
    /// Each enum value or bit position and its label, e.g. `(65, "ASK")`.
    pub values: &'static [(u16, &'static str)],
}

/// An enum that can be stored in a record field.
pub trait EnumField: Copy {
    /// The integer type the enum's raw value is read as.
    type Repr: Copy;
    /// The type a compiled record struct stores the enum as, which is `c_char` for a
    /// character enum.
    type Storage: Copy;
    /// The field registry type ID for this enum.
    const TYPE_ID: u8;
    /// The enum's name in its [`LabelDef`](crate::layout::LabelDef), e.g. `Side`.
    const NAME: &'static str;
    /// Each variant's raw value and label, e.g. `(65, "ASK")`.
    const LABELS: &'static [(u16, &'static str)];
}

/// An enum wrapper that allows carrying non-variant values. Models how
/// DBN handles encoding enums.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpenEnum<E: EnumField> {
    raw: E::Repr,
    _marker: PhantomData<E>,
}

impl<E: EnumField> OpenEnum<E> {
    /// Wraps a raw representation, keeping non-variant values (the open-enum contract).
    pub const fn from_raw(raw: E::Repr) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    /// The underlying raw representation.
    pub const fn raw(&self) -> E::Repr {
        self.raw
    }
}

impl<E> OpenEnum<E>
where
    E: EnumField + TryFrom<E::Repr>,
    E::Repr: std::fmt::Display,
{
    /// The known variant this wraps, converting via the enum's `TryFrom<Repr>`.
    ///
    /// # Errors
    /// Returns a conversion error if the raw value is not a known variant.
    pub fn get(&self) -> crate::Result<E> {
        E::try_from(self.raw).map_err(|_| crate::Error::conversion::<E>(self.raw))
    }
}

/// New type wrapper to differentiate between C char and the numeric type it's aliased
/// to, e.g. `i8` on Linux.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Char(pub c_char);

/// Marker for null-terminated string fields. Uninhabited as it's only
/// used as a type parameter. Width is declared in the `FieldDef`.
#[derive(Clone, Copy)]
pub enum CStr {}

/// Marker for null-terminated symbol fields. Uninhabited as it's only used as a type
/// parameter. Every symbol field in a stream has the stream's symbol width.
#[derive(Clone, Copy)]
pub enum Symbol {}
