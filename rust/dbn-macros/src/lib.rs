use proc_macro::TokenStream;

mod serialize;

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
