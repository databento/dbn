use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token, ExprPath, Ident, ItemStruct, LitBool, Token,
};

use crate::dbn_attr::{get_sorted_fields, DbnFieldAttrs, RequiredField, INDEX_TS_ATTR};

pub fn attribute_macro_impl(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(attr as Args);
    if args.args.is_empty() {
        return syn::Error::new(
            args.span,
            "Need to specify at least one rtype to match against",
        )
        .into_compile_error()
        .into();
    }
    let input_struct = parse_macro_input!(input as ItemStruct);
    let record_type = &input_struct.ident;
    let header = match get_header_type(&input_struct) {
        Ok(header) => header,
        Err(e) => return e.into_compile_error().into(),
    };
    let raw_index_ts = get_raw_index_ts(&input_struct).unwrap_or_else(|e| e.into_compile_error());
    let rtypes = args.args.iter();
    let crate_name = crate::utils::crate_name();
    let impl_debug = crate::debug::record_debug_impl(&input_struct);
    let impl_py_repr = get_py_repr_impl(&input_struct);
    let impl_layout = if args.layout {
        match &input_struct.fields {
            syn::Fields::Named(fields) => {
                crate::layout_desc::expand_record(record_type, &args.required, fields)
                    .unwrap_or_else(|e| e.into_compile_error())
            }
            _ => syn::Error::new(input_struct.span(), "A record needs named fields")
                .into_compile_error(),
        }
    } else {
        quote!()
    };
    quote! (
        #input_struct

        impl #crate_name::record::Record for #record_type {
            type Header = #header;

            fn record_size(&self) -> usize {
                self.hd.record_size()
            }

            fn rtype(&self) -> #crate_name::Result<#crate_name::RType> {
                self.hd.rtype()
            }

            fn raw_rtype(&self) -> u16 {
                u16::from(self.hd.rtype)
            }

            fn publisher_id(&self) -> u16 {
                self.hd.publisher_id
            }

            fn publisher(&self) -> #crate_name::Result<#crate_name::Publisher> {
                self.hd.publisher()
            }

            fn instrument_id(&self) -> u64 {
                u64::from(self.hd.instrument_id)
            }

            fn raw_ts_event(&self) -> u64 {
                self.hd.ts_event
            }

            #raw_index_ts
        }

        impl #crate_name::record::RecordMut for #record_type {
            fn header_mut(&mut self) -> &mut #header {
                &mut self.hd
            }
        }

        impl #crate_name::record::HasRType for #record_type {
            #[allow(deprecated)]
            fn has_rtype(rtype: u16) -> bool {
                #(rtype == u16::from(#rtypes))||*
            }
        }

        impl AsRef<[u8]> for #record_type {
            fn as_ref(&self) -> &[u8] {
                unsafe { #crate_name::record::record_as_u8_slice(self) }
            }
        }

        impl std::cmp::PartialOrd for #record_type {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                use #crate_name::record::Record;
                if self.raw_index_ts() == #crate_name::UNDEF_TIMESTAMP || other.raw_index_ts() == #crate_name::UNDEF_TIMESTAMP {
                    None
                } else {
                    Some(self.raw_index_ts().cmp(&other.raw_index_ts()))
                }
            }
        }

        #impl_debug

        #impl_py_repr

        #impl_layout
    )
    .into()
}

fn get_py_repr_impl(input_struct: &ItemStruct) -> TokenStream {
    let syn::Fields::Named(fields) = &input_struct.fields else {
        return quote!();
    };
    let sorted_fields = match get_sorted_fields(fields.clone()) {
        Ok(fields) => fields,
        Err(e) => return e.into_compile_error(),
    };
    let impl_body = crate::py_repr::py_repr_impl(&input_struct.ident, &sorted_fields);
    quote! {
        #[cfg(feature = "python")]
        #impl_body
    }
}

pub(crate) struct Args {
    args: Vec<ExprPath>,
    layout: bool,
    required: Vec<RequiredField>,
    span: Span,
}

enum Arg {
    RType(ExprPath),
    Layout(LitBool),
    Required(Punctuated<RequiredField, Token![,]>),
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident) && (input.peek2(Token![=]) || input.peek2(token::Paren)) {
            let key: Ident = input.parse()?;
            if key == "layout" {
                input.parse::<Token![=]>()?;
                return Ok(Self::Layout(input.parse()?));
            }
            if key == "required" {
                let content;
                parenthesized!(content in input);
                return Ok(Self::Required(Punctuated::parse_terminated(&content)?));
            }
            return Err(syn::Error::new(
                key.span(),
                "unrecognized dbn_record argument",
            ));
        }
        Ok(Self::RType(input.parse()?))
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Args {
            args: Vec::new(),
            layout: true,
            required: Vec::new(),
            span: input.span(),
        };
        for arg in Punctuated::<Arg, Token![,]>::parse_terminated(input)? {
            match arg {
                Arg::RType(rtype) => args.args.push(rtype),
                Arg::Layout(layout) => args.layout = layout.value,
                Arg::Required(required) => args.required.extend(required),
            }
        }
        if !args.layout && !args.required.is_empty() {
            return Err(syn::Error::new(
                args.required[0].ident.span(),
                "`required` has no effect with `layout = false`",
            ));
        }
        Ok(args)
    }
}

/// The header type a record begins with, which is the type of its `hd` field.
fn get_header_type(input_struct: &ItemStruct) -> syn::Result<&syn::Type> {
    input_struct
        .fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|i| i == "hd"))
        .map(|f| &f.ty)
        .ok_or_else(|| syn::Error::new(input_struct.span(), "A record needs an `hd` header field"))
}

fn get_raw_index_ts(input_struct: &ItemStruct) -> syn::Result<TokenStream> {
    let mut index_ts_fields = Vec::new();
    for field in input_struct.fields.iter() {
        if DbnFieldAttrs::parse(field)?
            .flags
            .iter()
            .any(|id| id == INDEX_TS_ATTR)
        {
            index_ts_fields.push(field.ident.as_ref().unwrap())
        }
    }
    match index_ts_fields.len() {
        0 => Ok(quote!()),
        1 => {
            let index_ts = index_ts_fields[0];
            Ok(quote!(
                fn raw_index_ts(&self) -> u64 {
                    self.#index_ts
                }
            ))
        }
        _ => Err(syn::Error::new(
            input_struct.span(),
            "Only one field can be marked index_ts",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_single() {
        let input = quote!(rtype::MBO);
        let args = syn::parse2::<Args>(input).unwrap();
        assert_eq!(args.args.len(), 1);
    }

    #[test]
    fn parse_args_multiple() {
        let input = quote!(rtype::MBO, rtype::OHLC);
        let args = syn::parse2::<Args>(input).unwrap();
        assert_eq!(args.args.len(), 2);
    }

    #[test]
    fn parse_args_empty() {
        let input = quote!();
        let args = syn::parse2::<Args>(input).unwrap();
        assert!(args.args.is_empty());
    }

    #[test]
    fn parse_args_layout() {
        let args = syn::parse2::<Args>(quote!(rtype::MBO)).unwrap();
        assert!(args.layout);
        let args = syn::parse2::<Args>(quote!(rtype::GAP, layout = false)).unwrap();
        assert_eq!(args.args.len(), 1);
        assert!(!args.layout);
        assert!(syn::parse2::<Args>(quote!(rtype::GAP, shape = false)).is_err());
    }

    #[test]
    fn parse_args_required() {
        let args = syn::parse2::<Args>(quote!(
            rtype::SYMBOL_MAPPING,
            required(stype_in_symbol = 2, stype_out_symbol)
        ))
        .unwrap();
        assert_eq!(args.args.len(), 1);
        let required: Vec<_> = args
            .required
            .iter()
            .map(|field| (field.ident.to_string(), field.encode_order))
            .collect();
        assert_eq!(
            required,
            [
                ("stype_in_symbol".to_owned(), Some(2)),
                ("stype_out_symbol".to_owned(), None)
            ]
        );
        assert!(syn::parse2::<Args>(quote!(
            rtype::INSTRUMENT_DEF,
            required(raw_symbol),
            layout = false
        ))
        .is_err());
    }
}
