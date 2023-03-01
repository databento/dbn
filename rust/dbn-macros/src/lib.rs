use proc_macro::TokenStream;

/// Dummy derive macro to get around `cfg_attr` incompatibility of several
/// of pyo3's attribute macros. See <https://github.com/PyO3/pyo3/issues/780>.
///
/// `MockPyo3` is an invented trait.
#[proc_macro_derive(MockPyo3, attributes(pyo3))]
pub fn derive_mock_pyo3(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}
