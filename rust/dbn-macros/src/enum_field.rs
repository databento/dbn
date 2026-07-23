//! Derive macro for the `EnumField` trait used in DBN v4 dynamic field access.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput};

use crate::utils::crate_name;

pub fn derive_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    if !matches!(input.data, Data::Enum(_)) {
        return Err(syn::Error::new(
            input.span(),
            "EnumField can only be derived for enums",
        ));
    }
    let ident = &input.ident;
    let crate_name = crate_name();

    // `Repr` and `TYPE_ID`: char-valued enums read as a `u8` but tag as `ENUM_CHAR_ID`
    // (they can't be told apart from a numeric `u8` enum by the repr alone), so a
    // `#[dbn(char)]` marker distinguishes them. Everything else keys off `#[repr(..)]`.
    let (repr_ty, type_id) = if has_char_attr(input)? {
        (quote!(u8), quote!(ENUM_CHAR_ID))
    } else {
        match repr_ident(input)?.as_deref() {
            Some("u8") => (quote!(u8), quote!(ENUM_8_ID)),
            Some("u16") => (quote!(u16), quote!(ENUM_16_ID)),
            _ => return Err(syn::Error::new(
                input.span(),
                "EnumField requires `#[repr(u8)]` or `#[repr(u16)]`, or the `#[dbn(char)]` marker",
            )),
        }
    };

    Ok(quote! {
        impl #crate_name::v4::types::EnumField for #ident {
            type Repr = #repr_ty;
            const TYPE_ID: u8 = #crate_name::v4::fields::#type_id;
        }
    })
}

/// Whether the enum carries the `#[dbn(char)]` marker.
fn has_char_attr(input: &DeriveInput) -> syn::Result<bool> {
    let mut is_char = false;
    for attr in &input.attrs {
        if attr.path().is_ident("dbn") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("char") {
                    is_char = true;
                }
                Ok(())
            })?;
        }
    }
    Ok(is_char)
}

/// The primitive named in `#[repr(..)]`, if any (e.g. `u8`, `u16`).
fn repr_ident(input: &DeriveInput) -> syn::Result<Option<String>> {
    let mut repr = None;
    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                if let Some(id) = meta.path.get_ident() {
                    repr = Some(id.to_string());
                }
                Ok(())
            })?;
        }
    }
    Ok(repr)
}
