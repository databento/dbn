use std::collections::{HashSet, VecDeque};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parenthesized, parse_macro_input, token, Data, DeriveInput, Field, FieldsNamed, Ident, Meta,
};

pub fn derive_csv_macro_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(data_struct) = data {
        if let syn::Fields::Named(fields) = data_struct.fields {
            let fields = match get_sorted_fields(fields) {
                Ok(fields) => fields,
                Err(ts) => {
                    return ts.into_compile_error().into();
                }
            };
            let serialize_header_iter = fields.iter().map(write_csv_header_token_stream);
            let serialize_fields = fields
                .iter()
                .map(write_csv_field_token_stream)
                .collect::<syn::Result<Vec<_>>>()
                .unwrap_or_else(|e| vec![syn::Error::to_compile_error(&e)]);
            return quote! {
                impl crate::encode::csv::serialize::CsvSerialize for #ident {
                    fn serialize_header<W: ::std::io::Write>(writer: &mut ::csv::Writer<W>) -> ::csv::Result<()> {
                        use crate::encode::csv::serialize::WriteField;

                        #(#serialize_header_iter)*
                        Ok(())
                    }

                    fn serialize_to<W: ::std::io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
                        &self,
                        writer: &mut ::csv::Writer<W>
                    ) -> ::csv::Result<()> {
                        use crate::encode::csv::serialize::WriteField;

                        #(#serialize_fields)*
                        Ok(())
                    }
                }
            }
            .into();
        }
    }
    syn::Error::new(ident.span(), "Can only derive CsvSerialize for structs")
        .into_compile_error()
        .into()
}

pub fn derive_json_macro_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(data_struct) = data {
        if let syn::Fields::Named(fields) = data_struct.fields {
            let fields = match get_sorted_fields(fields) {
                Ok(fields) => fields,
                Err(ts) => {
                    return ts.into_compile_error().into();
                }
            };
            let serialize_fields = fields
                .iter()
                .map(write_json_field_token_stream)
                .collect::<syn::Result<Vec<_>>>()
                .unwrap_or_else(|e| vec![syn::Error::to_compile_error(&e)]);
            return quote! {
                impl crate::encode::json::serialize::JsonSerialize for #ident {
                    fn to_json<J: crate::json_writer::JsonWriter, const PRETTY_PX: bool, const PRETTY_TS: bool>(
                        &self,
                        writer: &mut crate::json_writer::JsonObjectWriter<J>,
                    ) {
                        use crate::encode::json::serialize::WriteField;

                        #(#serialize_fields)*
                    }
                }
            }
            .into();
        }
    }
    syn::Error::new(ident.span(), "Can only derive JsonSerialize for structs")
        .into_compile_error()
        .into()
}

fn get_sorted_fields(fields: FieldsNamed) -> syn::Result<VecDeque<Field>> {
    let mut fields: VecDeque<_> = fields.named.into_iter().collect();
    let mut encode_orders = HashSet::new();
    let mut encode_order_fields = Vec::new();
    for field in fields.iter() {
        if let Some(encode_order) = find_encode_order_attr(field)? {
            if !encode_orders.insert(encode_order) {
                // Already existed
                return Err(syn::Error::new_spanned(
                    field,
                    format!("Specified duplicate encode order `{encode_order}` for field"),
                ));
            }
            encode_order_fields.push((encode_order, field.clone()));
        }
    }
    encode_order_fields.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
    for (encode_order, prioritized_field) in encode_order_fields {
        let idx = fields
            .iter()
            .position(|f| f.ident == prioritized_field.ident)
            .expect("to find field");
        fields.remove(idx).expect("Field to be at index");
        fields.insert(encode_order, prioritized_field);
    }
    Ok(fields)
}

fn write_csv_header_token_stream(field: &Field) -> TokenStream {
    let ident = field.ident.as_ref().unwrap();
    let field_type = &field.ty;
    // ignore dummy fields
    if ident.to_string().starts_with('_') {
        return TokenStream::new();
    }
    quote! {
        <#field_type>::write_header(writer, stringify!(#ident))?;
    }
}

fn write_csv_field_token_stream(field: &Field) -> syn::Result<TokenStream> {
    let ident = field.ident.as_ref().unwrap();
    // ignore dummy fields
    if ident.to_string().starts_with('_') {
        return Ok(quote! {});
    }
    if let Some(dbn_attr_id) = find_dbn_attr_id(field)? {
        if dbn_attr_id == "unix_nanos" {
            Ok(quote! {
                crate::encode::csv::serialize::write_ts_field::<_, PRETTY_TS>(writer, self.#ident)?;
            })
        } else if dbn_attr_id == "fixed_price" {
            Ok(quote! {
                crate::encode::csv::serialize::write_px_field::<_, PRETTY_PX>(writer, self.#ident)?;
            })
        } else if dbn_attr_id == "c_char" {
            Ok(quote! {
                crate::encode::csv::serialize::write_c_char_field(writer, self.#ident)?;
            })
        } else if dbn_attr_id == "skip" {
            Ok(quote! {})
        } else {
            Err(syn::Error::new(
                dbn_attr_id.span(),
                format!("Invalid attr `{dbn_attr_id}` passed to `#[dbn]`"),
            ))
        }
    } else {
        Ok(quote! {
            self.#ident.write_field::<_, PRETTY_PX, PRETTY_TS>(writer)?;
        })
    }
}

fn write_json_field_token_stream(field: &Field) -> syn::Result<TokenStream> {
    let ident = field.ident.as_ref().unwrap();
    // ignore dummy fields
    if ident.to_string().starts_with('_') {
        return Ok(quote! {});
    }
    if let Some(dbn_attr_id) = find_dbn_attr_id(field)? {
        if dbn_attr_id == "unix_nanos" {
            Ok(quote! {
                crate::encode::json::serialize::write_ts_field::<_, PRETTY_TS>(writer, stringify!(#ident), self.#ident);
            })
        } else if dbn_attr_id == "fixed_price" {
            Ok(quote! {
                crate::encode::json::serialize::write_px_field::<_, PRETTY_PX>(writer, stringify!(#ident), self.#ident);
            })
        } else if dbn_attr_id == "c_char" {
            Ok(quote! {
                crate::encode::json::serialize::write_c_char_field(writer, stringify!(#ident), self.#ident);
            })
        } else if dbn_attr_id == "skip" {
            Ok(quote! {})
        } else {
            Err(syn::Error::new(
                dbn_attr_id.span(),
                format!("Invalid attr `{dbn_attr_id}` passed to `#[dbn]`"),
            ))
        }
    } else {
        Ok(quote! {
            self.#ident.write_field::<_, PRETTY_PX, PRETTY_TS>(writer, stringify!(#ident));
        })
    }
}

fn find_dbn_attr_id(field: &Field) -> syn::Result<Option<Ident>> {
    for attr in field.attrs.iter() {
        if let Meta::List(ref meta_list) = attr.meta {
            if meta_list.path.is_ident("dbn") {
                let mut ident = None;
                // parse contents, e.g `unix_nanos` from `#[dbn(unix_nanos)]` or `#[dbn(unix_nanos, encode_order)]`
                meta_list.parse_nested_meta(|meta| {
                    // Ignore encode_order here
                    if meta.path.is_ident("encode_order") {
                        // Still need to parse (N) here to consume it
                        if meta.input.peek(token::Paren) {
                            let content;
                            parenthesized!(content in meta.input);
                            let _lit: syn::LitInt = content.parse()?;
                        }
                    } else if let Some(i) = meta.path.get_ident() {
                        ident = Some(i.clone());
                    }
                    Ok(())
                })?;
                return Ok(ident);
            }
        }
    }
    Ok(None)
}

fn find_encode_order_attr(field: &Field) -> syn::Result<Option<usize>> {
    for attr in field.attrs.iter() {
        if let Meta::List(ref meta_list) = attr.meta {
            if meta_list.path.is_ident("dbn") {
                let mut encode_order = None;
                meta_list.parse_nested_meta(|meta| {
                    // #[dbn(encode_order)] or #[dbn(encode_order(1))]
                    if meta.path.is_ident("encode_order") {
                        if meta.input.peek(token::Paren) {
                            let content;
                            parenthesized!(content in meta.input);
                            let lit: syn::LitInt = content.parse()?;
                            let n: usize = lit.base10_parse()?;
                            encode_order = Some(n);
                        } else {
                            // defaults to 0
                            encode_order = Some(0)
                        }
                    }
                    Ok(())
                })?;
                return Ok(encode_order);
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use syn::FieldsNamed;

    use super::*;

    #[test]
    fn skip_field() {
        let input = quote!({
                #[dbn(skip)]
                pub b: bool,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        assert_eq!(fields.named.len(), 1);
        let csv_generated = write_csv_field_token_stream(fields.named.first().unwrap()).unwrap();
        let json_generated = write_json_field_token_stream(fields.named.first().unwrap()).unwrap();
        assert!(csv_generated.is_empty());
        assert!(json_generated.is_empty());
    }

    #[test]
    fn skip_underscore_field() {
        let input = quote!({
                pub _a: bool,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        assert_eq!(fields.named.len(), 1);
        let csv_generated = write_csv_field_token_stream(fields.named.first().unwrap()).unwrap();
        let json_generated = write_json_field_token_stream(fields.named.first().unwrap()).unwrap();
        assert!(csv_generated.is_empty());
        assert!(json_generated.is_empty());
    }

    #[test]
    fn find_encode_order_attr_blank() {
        let input = quote!({
            pub b: bool,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        assert_eq!(fields.named.len(), 1);
        assert!(find_encode_order_attr(&fields.named.first().unwrap())
            .unwrap()
            .is_none());
    }

    #[test]
    fn find_encode_order_attr_none() {
        let input = quote!({
            #[dbn(c_char)]
            pub f: c_char,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        assert_eq!(fields.named.len(), 1);
        assert!(find_encode_order_attr(&fields.named.first().unwrap())
            .unwrap()
            .is_none());
    }

    #[test]
    fn find_encode_order_attr_only() {
        let input = quote!({
            #[dbn(encode_order(3))]
            pub b: bool,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        assert_eq!(fields.named.len(), 1);
        assert_eq!(
            find_encode_order_attr(&fields.named.first().unwrap())
                .unwrap()
                .unwrap(),
            3
        );
    }

    #[test]
    fn find_encode_order_attr_first() {
        let input = quote!({
            #[dbn(encode_order(2), unix_nanos)]
            pub ts: u64,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        assert_eq!(fields.named.len(), 1);
        assert_eq!(
            find_encode_order_attr(&fields.named.first().unwrap())
                .unwrap()
                .unwrap(),
            2
        );
    }

    #[test]
    fn find_encode_order_attr_last() {
        let input = quote!({
            #[dbn(unix_nanos, encode_order(4))]
            pub ts: u64,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        assert_eq!(fields.named.len(), 1);
        assert_eq!(
            find_encode_order_attr(&fields.named.first().unwrap())
                .unwrap()
                .unwrap(),
            4
        );
    }

    #[test]
    fn find_encode_order_attr_default() {
        let input = quote!({
            #[dbn(unix_nanos, encode_order)]
            pub ts: u64,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        assert_eq!(fields.named.len(), 1);
        assert_eq!(
            find_encode_order_attr(&fields.named.first().unwrap())
                .unwrap()
                .unwrap(),
            0
        );
    }
}
