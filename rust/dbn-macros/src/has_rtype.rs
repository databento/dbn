use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    ExprPath, ItemStruct, Token,
};

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
    let rtypes = args.args.iter();
    let crate_name = crate::utils::crate_name();
    quote! (
        #input_struct

        impl #crate_name::record::HasRType for #record_type {
            #[allow(deprecated)]
            fn has_rtype(rtype: u8) -> bool {
                matches!(rtype, #(#rtypes)|*)
            }

            fn header(&self) -> &#crate_name::record::RecordHeader {
                &self.hd
            }

            fn header_mut(&mut self) -> &mut #crate_name::record::RecordHeader {
                &mut self.hd
            }
        }

        impl AsRef<[u8]> for #record_type {
            fn as_ref(&self) -> &[u8] {
                unsafe { ::std::slice::from_raw_parts(self as *const #record_type as *const u8, ::std::mem::size_of::<#record_type>()) }
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
