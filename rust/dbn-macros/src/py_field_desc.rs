use std::collections::VecDeque;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field};

use crate::dbn_attr::{
    find_dbn_attr_id, get_sorted_fields, is_hidden, C_CHAR_ATTR, FIXED_PRICE_ATTR, UNIX_NANOS_ATTR,
};

pub fn derive_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);
    let Data::Struct(data_struct) = data else {
        return syn::Error::new(ident.span(), "Can only derive PyFieldDesc for structs")
            .into_compile_error()
            .into()
    };
    let syn::Fields::Named(fields) = data_struct.fields else {
        return syn::Error::new(ident.span(), "Cannot derive PyFieldDesc for tuple struct")
            .into_compile_error()
            .into()
    };
    let sorted_fields = match get_sorted_fields(fields.clone()) {
        Ok(fields) => fields,
        Err(ts) => {
            return ts.into_compile_error().into();
        }
    };
    let raw_fields: VecDeque<_> = fields.named.into_iter().collect();
    let dtype_iter = raw_fields.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let f_type = &f.ty;
        if matches!(find_dbn_attr_id(f).unwrap_or_default(), Some(id) if id == C_CHAR_ATTR) {
            quote! {
                res.push((
                    stringify!(#ident).to_owned(),
                    "S1".to_owned(),
                ));
            }
        } else {
            quote! { res.extend(<#f_type>::field_dtypes(stringify!(#ident))); }
        }
    });
    let price_fields = fields_with_attr_ts(&raw_fields, FIXED_PRICE_ATTR, quote!(price_fields));
    let hidden_fields = hidden_fields(&raw_fields);
    let timestamp_fields =
        fields_with_attr_ts(&raw_fields, UNIX_NANOS_ATTR, quote!(timestamp_fields));
    let ordered_fields = sorted_fields.iter().filter(|f| !is_hidden(f)).map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let f_type = &f.ty;
        quote! {
            res.extend(<#f_type>::ordered_fields(stringify!(#ident)));
        }
    });

    quote! {
        impl crate::python::PyFieldDesc for #ident {
            fn field_dtypes(_field_name: &str) -> Vec<(String, String)> {
                let mut res =  Vec::new();
                #(#dtype_iter)*
                res
            }
            fn price_fields(_field_name: &str) -> Vec<String> {
                let mut res =  Vec::new();
                #price_fields
                res
            }
            fn hidden_fields(_field_name: &str) -> Vec<String> {
                let mut res =  Vec::new();
                #hidden_fields
                res
            }
            fn timestamp_fields(_field_name: &str) -> Vec<String> {
                let mut res =  Vec::new();
                #timestamp_fields
                res
            }
            fn ordered_fields(_field_name: &str) -> Vec<String> {
                let mut res =  Vec::new();
                #(#ordered_fields)*
                res
            }
        }
    }
    .into()
}

fn fields_with_attr_ts(fields: &VecDeque<Field>, attr: &str, method: TokenStream) -> TokenStream {
    let fields_iter = fields.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        if matches!(find_dbn_attr_id(f).unwrap_or_default(), Some(id) if id == attr) {
            quote! { res.push(stringify!(#ident).to_owned()); }
        } else {
            let f_type = &f.ty;
            quote! { res.extend(<#f_type>::#method(stringify!(#ident))); }
        }
    });
    quote! {
        #(#fields_iter)*
    }
}

fn hidden_fields(fields: &VecDeque<Field>) -> TokenStream {
    let fields_iter = fields.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        if is_hidden(f) {
            quote! { res.push(stringify!(#ident).to_owned()); }
        } else {
            let f_type = &f.ty;
            quote! { res.extend(<#f_type>::hidden_fields(stringify!(#ident))); }
        }
    });
    quote! {
        #(#fields_iter)*
    }
}
