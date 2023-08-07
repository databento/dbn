use proc_macro::TokenStream;

mod dbn_attr;
mod has_rtype;
mod py_field_desc;
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

/// Derive macro for CSV serialization. Supports the following `dbn` attributes:
/// - `c_char`: serializes the field as a `char`
/// - `fixed_price`: serializes the field as fixed-price, with the output format
///   depending on `PRETTY_PX`
/// - `skip`: does not serialize the field
/// - `unix_nanos`: serializes the field as a UNIX timestamp, with the output format
///   depending on `PRETTY_TS`
///
/// Note: fields beginning with `_` will automatically be skipped, e.g. `_dummy` isn't
/// serialized.
#[proc_macro_derive(CsvSerialize, attributes(dbn))]
pub fn derive_csv_serialize(input: TokenStream) -> TokenStream {
    serialize::derive_csv_macro_impl(input)
}

/// Derive macro for JSON serialization. Supports the following `dbn` attributes:
/// - `c_char`: serializes the field as a `char`
/// - `fixed_price`: serializes the field as fixed-price, with the output format
///   depending on `PRETTY_PX`
/// - `skip`: does not serialize the field
/// - `unix_nanos`: serializes the field as a UNIX timestamp, with the output format
///   depending on `PRETTY_TS`
///
/// Note: fields beginning with `_` will automatically be skipped, e.g. `_dummy` isn't
/// serialized.
#[proc_macro_derive(JsonSerialize, attributes(dbn))]
pub fn derive_json_serialize(input: TokenStream) -> TokenStream {
    serialize::derive_json_macro_impl(input)
}

/// Derive macro for field descriptions exposed to Python. Supports the following `dbn`
/// attributes:
/// - `fixed_price`: indicates this is a fixed-precision field
/// - `skip`: indicates this field should be hidden
/// - `unix_nanos`: indicates this is a UNIX nanosecond timestamp field
#[proc_macro_derive(PyFieldDesc, attributes(dbn))]
pub fn derive_py_field_desc(input: TokenStream) -> TokenStream {
    py_field_desc::derive_impl(input)
}

/// Attribute macro that acts like a derive macro for for `HasRType` and
/// `AsRef<[u8]>`.
///
/// Expects 1 or more paths to `u8` constants that are the RTypes associated
/// with this record.
#[proc_macro_attribute]
pub fn dbn_record(attr: TokenStream, input: TokenStream) -> TokenStream {
    has_rtype::attribute_macro_impl(attr, input)
}

#[cfg(test)]
mod tests {
    #[test]
    fn ui() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/*.rs");
    }
}
