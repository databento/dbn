use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    ExprPath, ItemStruct, Token,
};

use crate::dbn_attr::{find_dbn_attr_args, INDEX_TS_ATTR};

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
    let raw_index_ts = get_raw_index_ts(&input_struct).unwrap_or_else(|e| e.into_compile_error());
    let rtypes = args.args.iter();
    let crate_name = crate::utils::crate_name();
    quote! (
        #input_struct

        impl #crate_name::record::Record for #record_type {
            fn header(&self) -> &#crate_name::record::RecordHeader {
                &self.hd
            }
            #raw_index_ts
        }

        impl #crate_name::record::RecordMut for #record_type {
            fn header_mut(&mut self) -> &mut #crate_name::record::RecordHeader {
                &mut self.hd
            }
        }

        impl #crate_name::record::HasRType for #record_type {
            #[allow(deprecated)]
            fn has_rtype(rtype: u8) -> bool {
                matches!(rtype, #(#rtypes)|*)
            }
        }

        impl AsRef<[u8]> for #record_type {
            fn as_ref(&self) -> &[u8] {
                unsafe { ::std::slice::from_raw_parts(self as *const #record_type as *const u8, ::std::mem::size_of::<#record_type>()) }
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
    )
    .into()
}

struct Args {
    args: Vec<ExprPath>,
    span: Span,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args = Punctuated::<ExprPath, Token![,]>::parse_terminated(input)?;
        Ok(Args {
            args: args.into_iter().collect(),
            span: input.span(),
        })
    }
}

fn get_raw_index_ts(input_struct: &ItemStruct) -> syn::Result<TokenStream> {
    let mut index_ts_fields = Vec::new();
    for field in input_struct.fields.iter() {
        if find_dbn_attr_args(field)?
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
}
