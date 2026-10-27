use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field};

use crate::{
    dbn_attr::{
        find_dbn_serialize_attr, get_sorted_fields, is_hidden, C_CHAR_ATTR, FIXED_PRICE_ATTR,
        UNIX_NANOS_ATTR,
    },
    utils::crate_name,
};

pub fn derive_arrow_serialize_macro_impl(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(data_struct) = data {
        if let syn::Fields::Named(fields) = data_struct.fields {
            let crate_name = crate_name();
            let fields = match get_sorted_fields(fields) {
                Ok(fields) => fields,
                Err(ts) => {
                    return ts.into_compile_error().into();
                }
            };
            let (fields, columns): (Vec<_>, Vec<_>) = match fields
                .iter()
                .filter(|field| !is_hidden(field))
                .map(field_token_streams)
                .collect::<syn::Result<Vec<_>>>()
            {
                Ok(fields) => fields.into_iter().unzip(),
                Err(e) => return e.into_compile_error().into(),
            };

            return quote! {
                #[cfg(feature = "arrow")]
                impl #crate_name::encode::arrow::serialize::ArrowSerialize for #ident {
                    fn arrow_fields(
                        pretty_px: bool,
                        fields: &mut Vec<::arrow::datatypes::Field>,
                    ) {
                        use #crate_name::encode::arrow::serialize::{self as arrow_ser, ArrowField};
                        #(#fields)*
                    }

                    fn arrow_columns<R>(
                        rows: &[R],
                        get: impl Fn(&R) -> &Self + Copy,
                        pretty_px: bool,
                        columns: &mut Vec<::arrow::array::ArrayRef>,
                    ) {
                        use #crate_name::encode::arrow::serialize::{self as arrow_ser, ArrowField};
                        #(#columns)*
                    }
                }
            }
            .into();
        }
    }
    syn::Error::new(ident.span(), "Can only derive ArrowSerialize for structs")
        .into_compile_error()
        .into()
}

/// Returns the statements pushing `field`'s Arrow fields and columns. Both go through
/// the same `ArrowField` implementer, so the schema and the data can't disagree.
fn field_token_streams(field: &Field) -> syn::Result<(TokenStream, TokenStream)> {
    let ident = field.ident.as_ref().unwrap();
    let name = ident.to_string();
    let field_type = &field.ty;
    let column_type = match find_dbn_serialize_attr(field)? {
        Some(attr) if attr == UNIX_NANOS_ATTR => quote! { arrow_ser::UnixNanos },
        Some(attr) if attr == FIXED_PRICE_ATTR => quote! { arrow_ser::FixedPrice },
        Some(attr) if attr == C_CHAR_ATTR => quote! { arrow_ser::CChar },
        Some(attr) => {
            return Err(syn::Error::new(
                attr.span(),
                format!("Invalid attr `{attr}` passed to `#[dbn]`"),
            ))
        }
        None => quote! { <#field_type as ArrowField> },
    };
    Ok((
        quote! { #column_type::fields(#name, pretty_px, fields); },
        quote! { #column_type::columns(rows, |r| &get(r).#ident, pretty_px, columns); },
    ))
}
