//! Derive macros for the `RecordLayoutDesc` and `StructLayoutDesc` layout traits.

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Field, FieldsNamed, Type};

use crate::{
    dbn_attr::{is_hidden, sort_by_encode_order, DbnFieldAttrs, RequiredField},
    utils::crate_name,
};

pub fn derive_struct_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_struct(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

pub fn expand_record(
    ident: &Ident,
    required: &[RequiredField],
    fields: &FieldsNamed,
) -> syn::Result<TokenStream> {
    let crate_name = crate_name();
    let name = ident.to_string();
    let placements = field_placements(&crate_name, &quote!(r), fields)?;
    let order = field_order(&crate_name, fields, required)?;
    let required = required.iter().map(|field| {
        let field_name = field.ident.to_string();
        let field_id = registry_path(&crate_name, &field.ident.clone().into());
        quote!((#field_id, #field_name))
    });
    Ok(quote! {
        impl #crate_name::layout::RecordLayoutDesc for #ident {
            const NAME: &'static str = #name;

            fn add_fields(r: &mut #crate_name::layout::RecordLayoutBuilder<'_>) {
                #(#placements)*
                r.order(&[#(#order),*]);
            }

            const REQUIRED_FIELDS: &'static [(
                #crate_name::v4::fields::Field<#crate_name::v4::types::Symbol>,
                &'static str,
            )] = &[#(#required),*];
        }
    })
}

fn expand_struct(input: &DeriveInput) -> syn::Result<TokenStream> {
    let crate_name = crate_name();
    let ident = &input.ident;
    let name = ident.to_string();
    let fields = named_fields(input)?;
    let placements = field_placements(&crate_name, &quote!(s), fields)?;
    let order = field_order(&crate_name, fields, &[])?;
    Ok(quote! {
        impl #crate_name::layout::StructLayoutDesc for #ident {
            const NAME: &'static str = #name;

            fn add_fields(s: &mut #crate_name::layout::StructLayoutBuilder<'_>) {
                #(#placements)*
                s.order(&[#(#order),*]);
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
        fn enum_matches_registry<E: #crate_name::v4::types::EnumField, S>(
            _field: #crate_name::v4::fields::Field<#crate_name::v4::types::OpenEnum<E>>,
            _get: fn(&S) -> &E,
        ) {
            const {
                assert!(::std::mem::size_of::<E>() == ::std::mem::size_of::<E::Storage>());
            }
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
                let check = if is_registry_storage(ty) {
                    quote_spanned! {field.span()=>
                        storage_matches_registry(#field_id, |rec: &Self| &rec.#field_ident);
                    }
                } else {
                    quote_spanned! {field.span()=>
                        enum_matches_registry(#field_id, |rec: &Self| &rec.#field_ident);
                    }
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

/// The field IDs in encode order including `required` fields.
fn field_order(
    crate_name: &TokenStream,
    fields: &FieldsNamed,
    required: &[RequiredField],
) -> syn::Result<Vec<TokenStream>> {
    let mut entries = Vec::new();
    for field in &fields.named {
        let entry = if is_hidden(field) {
            None
        } else {
            Some(registry_entry(field)?)
        };
        entries.push((DbnFieldAttrs::parse(field)?.encode_order, entry));
    }
    entries.extend(
        required
            .iter()
            .map(|field| (field.encode_order, Some(field.ident.clone().into()))),
    );
    Ok(sort_by_encode_order(entries)?
        .into_iter()
        .flatten()
        .map(|entry| {
            let field_id = registry_path(crate_name, &entry);
            quote!(#field_id.id())
        })
        .collect())
}

fn registry_entry(field: &Field) -> syn::Result<syn::Path> {
    Ok(DbnFieldAttrs::parse(field)?
        .field_id
        .unwrap_or_else(|| field.ident.clone().unwrap().into()))
}

fn registry_path(crate_name: &TokenStream, entry: &syn::Path) -> TokenStream {
    match entry.get_ident() {
        Some(field) => {
            let constant = format_ident!("{}", field.to_string().to_ascii_uppercase());
            quote!(#crate_name::v4::fields::#constant)
        }
        None => quote!(#entry),
    }
}

fn is_registry_storage(ty: &Type) -> bool {
    const PRIMITIVES: &[&str] = &[
        "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "f64", "c_char", "FlagSet",
    ];
    matches!(ty, Type::Path(path) if path.path.segments.last().is_some_and(|s| PRIMITIVES.iter().any(|p| s.ident == p)))
}

fn is_c_char(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().is_some_and(|s| s.ident == "c_char"))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn expand_record_of(input: &DeriveInput) -> syn::Result<TokenStream> {
        expand_record(
            &input.ident,
            &[RequiredField {
                ident: format_ident!("raw_symbol"),
                encode_order: Some(1),
            }],
            named_fields(input)?,
        )
    }

    #[rstest]
    #[case::record(expand_record_of, quote! {
        struct Rec {
            hd: RecordHeader,
            #[dbn(field_id = STATUS_ACTION)]
            action: u16,
            _reserved: [u8; 6],
            #[dbn(encode_order(0))]
            err: [c_char; 8],
            #[dbn(field_id = LEVELS_1)]
            levels: [BidAskPair; 1],
            security_update_action: SecurityUpdateAction,
            #[dbn(field_id = fields::TS_NOW)]
            ts_now: u64,
        }
    }, &[
        "layout :: RecordLayoutDesc for Rec",
        ":: dbn :: v4 :: fields :: STATUS_ACTION",
        "str_field (:: dbn :: v4 :: fields :: ERR",
        "struct_array_of :: < BidAskPair , 1 > (:: dbn :: v4 :: fields :: LEVELS_1 , \"levels\")",
        "REQUIRED_FIELDS",
        "(:: dbn :: v4 :: fields :: RAW_SYMBOL , \"raw_symbol\")",
        "enum_matches_registry (:: dbn :: v4 :: fields :: SECURITY_UPDATE_ACTION",
        "storage_matches_registry (fields :: TS_NOW",
        "r . at (:: std :: mem :: size_of :: < Self > ())",
        "r . order (& [:: dbn :: v4 :: fields :: ERR . id () , \
         :: dbn :: v4 :: fields :: RAW_SYMBOL . id () , :: dbn :: v4 :: fields :: HD . id () , \
         :: dbn :: v4 :: fields :: STATUS_ACTION . id () , :: dbn :: v4 :: fields :: LEVELS_1 . id () , \
         :: dbn :: v4 :: fields :: SECURITY_UPDATE_ACTION . id () , fields :: TS_NOW . id ()]) ;",
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
        // The record builder places the header itself
        assert!(!expanded.contains("\"hd\""), "{expanded}");
    }
}
