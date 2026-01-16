//! Python-specific `__repr__` implementation for `dbn_record` macro.

use std::collections::VecDeque;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Field, ItemStruct};

use crate::{
    dbn_attr::{
        find_dbn_debug_attr, get_sorted_fields, is_hidden, C_CHAR_ATTR, FIXED_PRICE_ATTR,
        FMT_METHOD, UNIX_NANOS_ATTR,
    },
    utils::crate_name,
};

pub fn derive_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_struct = parse_macro_input!(input as ItemStruct);
    let syn::Fields::Named(fields) = &input_struct.fields else {
        return quote!().into();
    };
    let sorted_fields = match get_sorted_fields(fields.clone()) {
        Ok(fields) => fields,
        Err(e) => return e.into_compile_error().into(),
    };
    py_repr_impl(&input_struct.ident, &sorted_fields).into()
}

pub fn py_repr_impl(ident: &syn::Ident, sorted_fields: &VecDeque<Field>) -> TokenStream {
    let crate_name = crate_name();
    let type_name = ident.to_string();

    let field_writes_with_sep: Vec<TokenStream> = sorted_fields
        .iter()
        .enumerate()
        .filter_map(|(i, f)| {
            let fw = format_field(f, &crate_name)?;
            Some(if i == 0 {
                fw
            } else {
                quote! { write!(s, ", ")?; #fw }
            })
        })
        .collect();

    quote! {
        impl #crate_name::python::repr::WritePyRepr for #ident {
            fn write_py_repr(&self, s: &mut String) -> ::std::fmt::Result {
                use ::std::fmt::Write;

                s.push_str(concat!(#type_name, "("));
                #(#field_writes_with_sep)*
                s.push(')');
                Ok(())
            }
        }
    }
}

fn format_field(field: &Field, crate_name: &TokenStream) -> Option<TokenStream> {
    let ident = field.ident.as_ref()?;
    let field_name = ident.to_string();
    if is_hidden(field) {
        return None;
    }

    let attr = find_dbn_debug_attr(field).ok().flatten();
    let field_ty = &field.ty;

    Some(match attr {
        Some(id) if id == C_CHAR_ATTR => {
            quote! {
                write!(s, concat!(#field_name, "='{}'"), self.#ident as u8 as char)?;
            }
        }
        Some(id) if id == FMT_METHOD => {
            quote! {
                #crate_name::python::repr::fmt_enum_method(
                    s,
                    #field_name,
                    || self.#ident(),
                )?;
            }
        }
        Some(id) if id == FIXED_PRICE_ATTR => {
            quote! {
                #crate_name::python::repr::fmt_px(s, #field_name, self.#ident)?;
            }
        }
        Some(id) if id == UNIX_NANOS_ATTR => {
            quote! {
                #crate_name::python::repr::fmt_ts(s, #field_name, self.#ident)?;
            }
        }
        _ => {
            quote! {
                if !<#field_ty as #crate_name::python::repr::WritePyRepr>::SHOULD_FLATTEN {
                    write!(s, concat!(#field_name, "="))?;
                }
                self.#ident.write_py_repr(s)?;
            }
        }
    })
}
