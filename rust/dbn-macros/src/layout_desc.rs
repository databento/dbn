//! Derive macros for the `RecordLayoutDesc` and `StructLayoutDesc` layout traits.

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Field, FieldsNamed, Type};

use crate::{
    dbn_attr::{is_hidden, DbnFieldAttrs, DbnStructAttrs},
    utils::crate_name,
};

pub fn derive_record_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_record(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

pub fn derive_struct_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_struct(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn expand_record(input: &DeriveInput) -> syn::Result<TokenStream> {
    let crate_name = crate_name();
    let ident = &input.ident;
    let name = ident.to_string();
    let placements = field_placements(&crate_name, &quote!(r), named_fields(input)?)?;
    let required = DbnStructAttrs::parse(&input.attrs)?
        .required
        .into_iter()
        .map(|field| {
            let field_name = field.to_string();
            let field_id = registry_path(&crate_name, &field);
            quote!(r.symbol_field(#field_id, #field_name);)
        });
    Ok(quote! {
        impl #crate_name::layout::RecordLayoutDesc for #ident {
            const NAME: &'static str = #name;

            fn add_fields(r: &mut #crate_name::layout::RecordLayoutBuilder<'_>) {
                #(#placements)*
            }

            fn add_required_fields(r: &mut #crate_name::layout::RecordLayoutBuilder<'_>) {
                #(#required)*
            }
        }
    })
}

fn expand_struct(input: &DeriveInput) -> syn::Result<TokenStream> {
    let crate_name = crate_name();
    let ident = &input.ident;
    let name = ident.to_string();
    let placements = field_placements(&crate_name, &quote!(s), named_fields(input)?)?;
    Ok(quote! {
        impl #crate_name::layout::StructLayoutDesc for #ident {
            const NAME: &'static str = #name;

            fn add_fields(s: &mut #crate_name::layout::StructLayoutBuilder<'_>) {
                #(#placements)*
            }
        }
        impl #crate_name::v4::fields::FieldType for #ident {
            const TYPE_ID: u8 = #crate_name::v4::fields::STRUCT_ID;
        }
        impl<const N: usize> #crate_name::v4::fields::FieldType for [#ident; N] {
            const TYPE_ID: u8 = #crate_name::v4::fields::STRUCT_ID;
            const ARRAY_WIRE_SIZE: Option<u16> = {
                let size = ::std::mem::size_of::<#ident>() * N;
                assert!(size <= u16::MAX as usize, "struct array size exceeds u16");
                Some(size as u16)
            };
        }
    })
}

fn named_fields(input: &DeriveInput) -> syn::Result<&FieldsNamed> {
    if let Data::Struct(data) = &input.data {
        if let syn::Fields::Named(fields) = &data.fields {
            return Ok(fields);
        }
    }
    Err(syn::Error::new(
        input.span(),
        "layout descriptions can only be derived for structs with named fields",
    ))
}

fn field_placements(
    crate_name: &TokenStream,
    builder: &TokenStream,
    fields: &FieldsNamed,
) -> syn::Result<Vec<TokenStream>> {
    let mut placements = vec![quote! {
        fn storage_matches_registry<T: #crate_name::v4::fields::FixedWidth, S>(
            _field: #crate_name::v4::fields::Field<T>,
            _get: fn(&S) -> &T::Storage,
        ) {
        }
    }];
    for field in &fields.named {
        let field_ident = field.ident.as_ref().unwrap();
        let field_name = field_ident.to_string();
        if field_name == "hd" || is_hidden(field) {
            continue;
        }
        let entry = registry_entry(field)?;
        let field_id = registry_path(crate_name, &entry);
        let at = quote!(#builder.at(::std::mem::offset_of!(Self, #field_ident)));
        let ty = &field.ty;
        placements.push(match ty {
            Type::Array(array) if is_c_char(&array.elem) => quote! {
                #at.str_field(
                    #field_id,
                    #field_name,
                    u16::try_from(::std::mem::size_of::<#ty>()).expect("C string width exceeds u16"),
                );
            },
            Type::Array(array) => {
                let elem = &array.elem;
                let len = &array.len;
                quote!(#at.struct_array_of::<#elem, #len>(#field_id, #field_name);)
            }
            _ => {
                let check = quote_spanned! {field.span()=>
                    storage_matches_registry(#field_id, |rec: &Self| &rec.#field_ident);
                };
                quote! {
                    #check
                    #at.field(#field_id, #field_name);
                }
            }
        });
    }
    placements.push(quote!(#builder.at(::std::mem::size_of::<Self>());));
    Ok(placements)
}

fn registry_entry(field: &Field) -> syn::Result<Ident> {
    Ok(DbnFieldAttrs::parse(field)?
        .field_id
        .unwrap_or_else(|| field.ident.clone().unwrap()))
}

fn registry_path(crate_name: &TokenStream, field: &Ident) -> TokenStream {
    let constant = format_ident!("{}", field.to_string().to_ascii_uppercase());
    quote!(#crate_name::v4::fields::#constant)
}

fn is_c_char(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().is_some_and(|s| s.ident == "c_char"))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::record(expand_record, quote! {
        #[dbn(required(raw_symbol))]
        struct Rec {
            hd: RecordHeader,
            #[dbn(field_id = STATUS_ACTION)]
            action: u16,
            _reserved: [u8; 6],
            err: [c_char; 8],
            #[dbn(field_id = LEVELS_1)]
            levels: [BidAskPair; 1],
        }
    }, &[
        "layout :: RecordLayoutDesc for Rec",
        ":: dbn :: v4 :: fields :: STATUS_ACTION",
        "str_field (:: dbn :: v4 :: fields :: ERR",
        "struct_array_of :: < BidAskPair , 1 > (:: dbn :: v4 :: fields :: LEVELS_1 , \"levels\")",
        "r . symbol_field (:: dbn :: v4 :: fields :: RAW_SYMBOL , \"raw_symbol\")",
        "r . at (:: std :: mem :: size_of :: < Self > ())",
    ])]
    #[case::nested_struct(expand_struct, quote! {
        struct Pair {
            bid_px: i64,
            _reserved: [u8; 2],
        }
    }, &[
        "layout :: StructLayoutDesc for Pair",
        "FieldType for [Pair ; N]",
        "const NAME : & 'static str = \"Pair\"",
        "s . at (:: std :: mem :: offset_of ! (Self , bid_px)) . field (:: dbn :: v4 :: fields :: BID_PX , \"bid_px\")",
    ])]
    fn expands(
        #[case] expand: fn(&DeriveInput) -> syn::Result<TokenStream>,
        #[case] input: TokenStream,
        #[case] expected: &[&str],
    ) {
        let expanded = expand(&syn::parse2(input).unwrap()).unwrap().to_string();
        for snippet in expected {
            assert!(
                expanded.contains(snippet),
                "missing `{snippet}` in {expanded}"
            );
        }
        assert!(!expanded.contains("_reserved"), "{expanded}");
        assert!(!expanded.contains("fields :: HD"), "{expanded}");
    }
}
