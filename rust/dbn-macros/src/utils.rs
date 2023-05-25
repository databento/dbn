use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::FoundCrate;
use quote::quote;

pub fn crate_name() -> TokenStream {
    match proc_macro_crate::crate_name("dbn").expect("dbn crate in Cargo.toml") {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( ::#ident )
        }
    }
}
