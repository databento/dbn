use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Field, ItemStruct};

use crate::{
    dbn_attr::{
        find_dbn_debug_attr, is_hidden, C_CHAR_ATTR, FIXED_PRICE_ATTR, FMT_BINARY, FMT_METHOD,
    },
    utils::crate_name,
};

pub fn record_debug_impl(input_struct: &ItemStruct) -> TokenStream {
    let record_type = &input_struct.ident;
    let field_iter = input_struct
        .fields
        .iter()
        .map(|f| format_field(f).unwrap_or_else(|e| e.into_compile_error()));
    quote! {
        impl ::std::fmt::Debug for #record_type {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let mut debug_struct = f.debug_struct(stringify!(#record_type));
                #(#field_iter)*
                debug_struct.finish()
            }
        }
    }
}

pub fn derive_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);
    let input_struct = parse_macro_input!(input as ItemStruct);
    let record_type = &input_struct.ident;
    let field_iter = input_struct
        .fields
        .iter()
        .map(|f| format_field(f).unwrap_or_else(|e| e.into_compile_error()));
    quote! {
        impl ::std::fmt::Debug for #record_type {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let mut debug_struct = f.debug_struct(stringify!(#record_type));
                #(#field_iter)*
                debug_struct.finish()
            }
        }
    }
    .into()
}

fn format_field(field: &Field) -> syn::Result<TokenStream> {
    let ident = field.ident.as_ref().unwrap();
    if is_hidden(field) {
        return Ok(quote!());
    }
    Ok(match find_dbn_debug_attr(field)? {
        Some(id) if id == C_CHAR_ATTR => {
            quote! { debug_struct.field(stringify!(#ident), &(self.#ident as u8 as char)); }
        }
        Some(id) if id == FIXED_PRICE_ATTR => {
            let crate_name = crate_name();
            quote! { debug_struct.field(stringify!(#ident), &#crate_name::pretty::Px(self.#ident)); }
        }
        Some(id) if id == FMT_BINARY => {
            // format as `0b00101010`
            quote! { debug_struct.field(stringify!(#ident), &format_args!("{:#010b}", &self.#ident)); }
        }
        Some(id) if id == FMT_METHOD => {
            // Try to use method to format, otherwise fallback on raw value
            return Ok(quote! {
                match self.#ident() {
                    Ok(s) => debug_struct.field(stringify!(#ident), &s),
                    Err(_) => debug_struct.field(stringify!(#ident), &self.#ident),
                };
            });
        }
        _ => quote! { debug_struct.field(stringify!(#ident), &self.#ident); },
    })
}
