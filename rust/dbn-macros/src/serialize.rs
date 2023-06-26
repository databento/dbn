use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Ident, Meta};

pub fn derive_csv_macro_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(data_struct) = data {
        if let syn::Fields::Named(fields) = data_struct.fields {
            let serialize_header_iter = fields.named.iter().map(write_csv_header_token_stream);
            let serialize_field_iter = fields.named.iter().map(write_csv_field_token_stream);
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

                        #(#serialize_field_iter)*
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
            let field_iter = fields.named.iter().map(write_json_field_token_stream);
            return quote! {
                impl crate::encode::json::serialize::JsonSerialize for #ident {
                    fn to_json<J: crate::json_writer::JsonWriter, const PRETTY_PX: bool, const PRETTY_TS: bool>(
                        &self,
                        writer: &mut crate::json_writer::JsonObjectWriter<J>,
                    ) {
                        use crate::encode::json::serialize::WriteField;

                        #(#field_iter)*
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

fn write_csv_field_token_stream(field: &Field) -> TokenStream {
    let ident = field.ident.as_ref().unwrap();
    // ignore dummy fields
    if ident.to_string().starts_with('_') {
        return TokenStream::new();
    }
    if let Some(dbn_attr_id) = find_dbn_attr_id(field) {
        if dbn_attr_id == "unix_nanos" {
            quote! {
                crate::encode::csv::serialize::write_ts_field::<_, PRETTY_TS>(writer, self.#ident)?;
            }
        } else if dbn_attr_id == "fixed_price" {
            quote! {
                crate::encode::csv::serialize::write_px_field::<_, PRETTY_PX>(writer, self.#ident)?;
            }
        } else if dbn_attr_id == "c_char" {
            quote! {
                crate::encode::csv::serialize::write_c_char_field(writer, self.#ident)?;
            }
        } else if dbn_attr_id == "skip" {
            quote! {}
        } else {
            syn::Error::new(
                dbn_attr_id.span(),
                format!("Invalid attr `{dbn_attr_id}` passed to `#[dbn]`"),
            )
            .into_compile_error()
        }
    } else {
        quote! {
            self.#ident.write_field::<_, PRETTY_PX, PRETTY_TS>(writer)?;
        }
    }
}

fn write_json_field_token_stream(field: &Field) -> TokenStream {
    let ident = field.ident.as_ref().unwrap();
    // ignore dummy fields
    if ident.to_string().starts_with('_') {
        return TokenStream::new();
    }
    if let Some(dbn_attr_id) = find_dbn_attr_id(field) {
        if dbn_attr_id == "unix_nanos" {
            quote! {
                crate::encode::json::serialize::write_ts_field::<_, PRETTY_TS>(writer, stringify!(#ident), self.#ident);
            }
        } else if dbn_attr_id == "fixed_price" {
            quote! {
                crate::encode::json::serialize::write_px_field::<_, PRETTY_PX>(writer, stringify!(#ident), self.#ident);
            }
        } else if dbn_attr_id == "c_char" {
            quote! {
                crate::encode::json::serialize::write_c_char_field(writer, stringify!(#ident), self.#ident);
            }
        } else if dbn_attr_id == "skip" {
            quote! {}
        } else {
            syn::Error::new(
                dbn_attr_id.span(),
                format!("Invalid attr `{dbn_attr_id}` passed to `#[dbn]`"),
            )
            .into_compile_error()
        }
    } else {
        quote! {
            self.#ident.write_field::<_, PRETTY_PX, PRETTY_TS>(writer, stringify!(#ident));
        }
    }
}

fn find_dbn_attr_id(field: &Field) -> Option<Ident> {
    field.attrs.iter().find_map(|a| {
        if let Meta::List(ref meta_list) = a.meta {
            if meta_list.path.is_ident("dbn") {
                // parse contents, e.g `unix_nanos` `#[dbn(unix_nanos)]`
                return meta_list.parse_args().ok();
            }
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use syn::FieldsNamed;

    use super::*;

    #[test]
    fn test_skip_field() {
        let input = quote!({
                #[dbn(skip)]
                pub b: bool,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        assert_eq!(fields.named.len(), 1);
        let csv_generated = write_csv_field_token_stream(fields.named.first().unwrap());
        let json_generated = write_json_field_token_stream(fields.named.first().unwrap());
        assert!(csv_generated.is_empty());
        assert!(json_generated.is_empty());
    }

    #[test]
    fn test_skip_underscore_field() {
        let input = quote!({
                pub _a: bool,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        assert_eq!(fields.named.len(), 1);
        let csv_generated = write_csv_field_token_stream(fields.named.first().unwrap());
        let json_generated = write_json_field_token_stream(fields.named.first().unwrap());
        assert!(csv_generated.is_empty());
        assert!(json_generated.is_empty());
    }
}
