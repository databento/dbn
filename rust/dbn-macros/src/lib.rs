use proc_macro::TokenStream;

mod arrow;
mod dbn_attr;
mod debug;
mod enum_field;
mod has_rtype;
mod layout_desc;
mod py_field_desc;
mod py_repr;
mod serialize;
mod utils;

/// Dummy derive macro to get around `cfg_attr` incompatibility of several
/// of pyo3's attribute macros. See <https://github.com/PyO3/pyo3/issues/780>.
///
/// `MockPyo3` is an invented trait.
#[proc_macro_derive(MockPyo3, attributes(pyo3))]
pub fn derive_mock_pyo3(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}

/// Dummy derive macro to enable the `dbn` helper attribute for record types
/// using the `dbn_record` proc macro but neither `CsvSerialize` nor `JsonSerialize` as
/// helper attributes aren't supported for proc macros alone. See
/// <https://github.com/rust-lang/rust/issues/65823>.
#[proc_macro_derive(DbnAttr, attributes(dbn))]
pub fn dbn_attr(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}

/// Derive macro for the `EnumField` trait used in DBN v4 dynamic field access.
///
/// Emits the `Repr` associated type and `TYPE_ID` const from the enum's
/// `#[repr(u8)]` / `#[repr(u16)]`. Char-valued enums read as a `u8` but tag as
/// `ENUM_CHAR_ID`; mark them with the `#[dbn(char)]` helper attribute.
#[proc_macro_derive(EnumField, attributes(dbn))]
pub fn derive_enum_field(input: TokenStream) -> TokenStream {
    enum_field::derive_impl(input)
}

/// Derive macro for the `StructLayoutDesc` trait, for a struct nested in records like
/// `BidAskPair`.
///
/// Places each fixed-width field at its `offset_of!` offset under the `v4::fields`
/// registry constant named after it. Fields beginning with `_` become padding. Also
/// implements `FieldType` for the struct and arrays of it, so registry constants can
/// name them.
///
/// Supports the `field_id = PATH` field attribute, which uses the registry entry
/// `PATH` instead of the one named after the field.
#[proc_macro_derive(StructLayoutDesc, attributes(dbn))]
pub fn derive_struct_layout_desc(input: TokenStream) -> TokenStream {
    layout_desc::derive_struct_impl(input)
}

/// Derive macro for CSV serialization. Supports the following `dbn` attributes:
/// - `c_char`: serializes the field as a `char`
/// - `encode_order`: overrides the position of the field in the CSV table
/// - `fixed_price`: serializes the field as fixed-price, with the output format
///   depending on `PRETTY_PX`
/// - `skip`: does not serialize the field
/// - `unix_nanos`: serializes the field as a UNIX timestamp, with the output format
///   depending on `PRETTY_TS`
///
/// Note: fields beginning with `_` will automatically be skipped, e.g. `_reserved`
/// isn't serialized.
#[proc_macro_derive(CsvSerialize, attributes(dbn))]
pub fn derive_csv_serialize(input: TokenStream) -> TokenStream {
    serialize::derive_csv_macro_impl(input)
}

/// Derive macro for JSON serialization.
///
/// Supports the following `dbn` attributes:
/// - `c_char`: serializes the field as a `char`
/// - `fixed_price`: serializes the field as fixed-price, with the output format
///   depending on `PRETTY_PX`
/// - `skip`: does not serialize the field
/// - `unix_nanos`: serializes the field as a UNIX timestamp, with the output format
///   depending on `PRETTY_TS`
///
/// Note: fields beginning with `_` will automatically be skipped, e.g. `_reserved`
/// isn't serialized.
#[proc_macro_derive(JsonSerialize, attributes(dbn))]
pub fn derive_json_serialize(input: TokenStream) -> TokenStream {
    serialize::derive_json_macro_impl(input)
}

/// Derive macro for field descriptions exposed to Python.
///
/// Supports the following `dbn` attributes:
/// - `c_char`: indicates the field dtype should be a single-character string rather
///   than an integer
/// - `encode_order`: overrides the position of the field in the ordered list
/// - `fixed_price`: indicates this is a fixed-precision field
/// - `skip`: indicates this field should be hidden
/// - `unix_nanos`: indicates this is a UNIX nanosecond timestamp field
#[proc_macro_derive(PyFieldDesc, attributes(dbn))]
pub fn derive_py_field_desc(input: TokenStream) -> TokenStream {
    py_field_desc::derive_impl(input)
}

/// Attribute macro that acts like a derive macro for `Debug` (with customization),
/// `Record`, `RecordMut`, `HasRType`, `PartialOrd`, `AsRef<[u8]>`, and
/// `RecordLayoutDesc`.
///
/// Expects 1 or more paths to `u8` constants that are the RTypes associated
/// with this record, optionally followed by:
/// - `layout = false`: skips `RecordLayoutDesc`
/// - `required(name, name = N, ...)`: the symbol fields every layout of the record
///   must have, listed in `REQUIRED_FIELDS`. `N` moves the field like `encode_order`
///
/// The layout places each field after the header at its `offset_of!` offset under the
/// `v4::fields` registry constant named after it, so the layout matches the compiled
/// struct.
///
/// Supports the following `dbn` field attributes:
/// - `c_char`: format the type as a `char` instead of as a numeric
/// - `encode_order(N)`: moves the field to position `N` of the layout's display order
/// - `field_id = PATH`: uses the registry entry `PATH` instead of the one named after
///   the field. A bare name resolves against `v4::fields`
/// - `fixed_price`: format the integer as a fixed-precision decimal
/// - `fmt_binary`: format as a binary
/// - `fmt_method`: try to format by calling the getter method with the same name as the
///   field. If the getter returns an error, the raw field value will be used
/// - `index_ts`: indicates this field is the primary timestamp for the record
/// - `skip`: won't be included in the `Debug` output
///
/// Note: attribute macros don't support helper attributes on their own. If not deriving
/// `CsvSerialize` or `JsonSerialize`, derive `DbnAttr` to use the `dbn` helper attribute
/// without a compiler error.
#[proc_macro_attribute]
pub fn dbn_record(attr: TokenStream, input: TokenStream) -> TokenStream {
    has_rtype::attribute_macro_impl(attr, input)
}

/// Derive macro for Arrow columnar serialization.
///
/// Generates an implementation of `ArrowSerialize` for the type, gated behind
/// `#[cfg(feature = "arrow")]`. Supports the same `dbn` attributes as
/// `CsvSerialize` and `JsonSerialize`.
///
/// Each field's schema entry and column come from one `ArrowField` implementer: the
/// marker type for the field's `dbn` attribute, if any, otherwise the field's type.
/// Those impls live in `dbn::encode::arrow::serialize`.
#[proc_macro_derive(ArrowSerialize, attributes(dbn))]
pub fn derive_arrow_serialize(input: TokenStream) -> TokenStream {
    arrow::derive_arrow_serialize_macro_impl(input)
}

/// Derive macro for Debug representations with the same extensions for DBN records
/// as `dbn_record`.
///
/// Supports the following `dbn` attributes:
/// - `c_char`: format the type as a `char` instead of as a numeric
/// - `fixed_price`: format the integer as a fixed-precision decimal
/// - `fmt_binary`: format as a binary
/// - `fmt_method`: try to format by calling the getter method with the same name as the
///   field. If the getter returns an error, the raw field value will be used
/// - `skip`: won't be included in the `Debug` output
///
/// Note: fields beginning with `_` will automatically be skipped, e.g. `_reserved`
/// isn't included in the `Debug` output.
#[proc_macro_derive(RecordDebug, attributes(dbn))]
pub fn derive_record_debug(input: TokenStream) -> TokenStream {
    debug::derive_impl(input)
}

/// Derive macro for Python-specific `__repr__` output.
///
/// Generates an implementation of `WritePyRepr` for the type.
/// Uses the `dbn` attributes.
#[proc_macro_derive(WritePyRepr, attributes(dbn))]
pub fn derive_write_py_repr(input: TokenStream) -> TokenStream {
    py_repr::derive_impl(input)
}

#[cfg(test)]
mod tests {
    #[test]
    fn ui() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/*.rs");
    }
}
