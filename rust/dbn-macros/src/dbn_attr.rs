//! Common functionality for working with the `#[dbn(...)]` attribute
//! macros.

use std::collections::{HashSet, VecDeque};

use proc_macro2::Ident;
use syn::{
    parenthesized, punctuated::Punctuated, spanned::Spanned, token, Attribute, Field, FieldsNamed,
    Meta, Token,
};

pub const C_CHAR_ATTR: &str = "c_char";
pub const ENCODE_ORDER_ATTR: &str = "encode_order";
pub const FIELD_ID_ATTR: &str = "field_id";
pub const FIXED_PRICE_ATTR: &str = "fixed_price";
pub const FMT_BINARY: &str = "fmt_binary";
pub const FMT_METHOD: &str = "fmt_method";
pub const INDEX_TS_ATTR: &str = "index_ts";
pub const REQUIRED_ATTR: &str = "required";
pub const SKIP_ATTR: &str = "skip";
pub const UNIX_NANOS_ATTR: &str = "unix_nanos";

/// Parses and sorts the fields of a DBN record according to the order specified with `dbn`
/// attributes.
pub fn get_sorted_fields(fields: FieldsNamed) -> syn::Result<VecDeque<Field>> {
    let mut fields: VecDeque<_> = fields.named.into_iter().collect();
    let mut encode_orders = HashSet::new();
    let mut encode_order_fields = Vec::new();
    for field in fields.iter() {
        if let Some(encode_order) = DbnFieldAttrs::parse(field)?.encode_order {
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
    encode_order_fields.sort_by_key(|field| field.0);
    for (encode_order, prioritized_field) in encode_order_fields {
        let idx = fields
            .iter()
            .position(|f| f.ident == prioritized_field.ident)
            .expect("to find field");
        fields.remove(idx).expect("to find field");
        if encode_order >= fields.len() {
            fields.push_back(prioritized_field);
        } else {
            fields.insert(encode_order, prioritized_field);
        }
    }
    Ok(fields)
}

#[derive(Debug, Default)]
pub struct DbnFieldAttrs {
    pub flags: Vec<Ident>,
    pub encode_order: Option<usize>,
    pub field_id: Option<Ident>,
}

impl DbnFieldAttrs {
    pub fn parse(field: &Field) -> syn::Result<Self> {
        let mut attrs = Self::default();
        let Some(meta_list) = field.attrs.iter().find_map(|attr| match &attr.meta {
            Meta::List(meta_list) if meta_list.path.is_ident("dbn") => Some(meta_list),
            _ => None,
        }) else {
            return Ok(attrs);
        };
        // parse contents, e.g `unix_nanos` from `#[dbn(unix_nanos)]` or `#[dbn(unix_nanos, encode_order)]`
        meta_list.parse_nested_meta(|meta| {
            if meta.path.is_ident(ENCODE_ORDER_ATTR) {
                attrs.encode_order = Some(if meta.input.peek(token::Paren) {
                    let content;
                    parenthesized!(content in meta.input);
                    content.parse::<syn::LitInt>()?.base10_parse()?
                } else {
                    0
                });
                Ok(())
            } else if meta.path.is_ident(FIELD_ID_ATTR) {
                attrs.field_id = Some(meta.value()?.parse()?);
                Ok(())
            } else if let Some(i) = meta.path.get_ident() {
                if i == C_CHAR_ATTR
                    || i == FIXED_PRICE_ATTR
                    || i == FMT_BINARY
                    || i == FMT_METHOD
                    || i == INDEX_TS_ATTR
                    || i == SKIP_ATTR
                    || i == UNIX_NANOS_ATTR
                {
                    attrs.flags.push(i.clone());
                    Ok(())
                } else {
                    Err(meta.error(format!("unrecognized dbn attr argument {i}")))
                }
            } else {
                Err(meta.error("unrecognized dbn attr"))
            }
        })?;
        Ok(attrs)
    }
}

#[derive(Debug, Default)]
pub struct DbnStructAttrs {
    pub required: Vec<Ident>,
}

impl DbnStructAttrs {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut parsed = Self::default();
        for attr in attrs.iter().filter(|attr| attr.path().is_ident("dbn")) {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(REQUIRED_ATTR) {
                    let content;
                    parenthesized!(content in meta.input);
                    parsed
                        .required
                        .extend(Punctuated::<Ident, Token![,]>::parse_terminated(&content)?);
                    Ok(())
                } else {
                    Err(meta.error("unrecognized dbn attr"))
                }
            })?;
        }
        Ok(parsed)
    }
}

pub fn is_hidden(field: &Field) -> bool {
    let ident = field.ident.as_ref().unwrap();
    ident.to_string().starts_with('_')
        || DbnFieldAttrs::parse(field)
            .unwrap_or_default()
            .flags
            .iter()
            .any(|id| id == SKIP_ATTR)
}

pub fn find_dbn_debug_attr(field: &Field) -> syn::Result<Option<Ident>> {
    let mut args: Vec<_> = DbnFieldAttrs::parse(field)?
        .flags
        .into_iter()
        .filter(|id| {
            id == C_CHAR_ATTR
                || id == FIXED_PRICE_ATTR
                || id == FMT_BINARY
                || id == FMT_METHOD
                || id == UNIX_NANOS_ATTR
        })
        .collect();
    match args.len() {
        0 => Ok(None),
        1 => Ok(Some(args.pop().unwrap())),
        _ => Err(syn::Error::new(
            field.span(),
            "Passed incompatible format arguments to dbn attr",
        )),
    }
}

pub fn find_dbn_serialize_attr(field: &Field) -> syn::Result<Option<Ident>> {
    let mut args: Vec<_> = DbnFieldAttrs::parse(field)?
        .flags
        .into_iter()
        .filter(|id| id == C_CHAR_ATTR || id == FIXED_PRICE_ATTR || id == UNIX_NANOS_ATTR)
        .collect();
    match args.len() {
        0 => Ok(None),
        1 => Ok(Some(args.pop().unwrap())),
        _ => Err(syn::Error::new(
            field.span(),
            "Passed incompatible serialization arguments to dbn attr",
        )),
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::FieldsNamed;

    use super::*;

    #[test]
    fn encode_order_blank() {
        let input = quote!({
            pub b: bool,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        assert_eq!(fields.named.len(), 1);
        assert!(DbnFieldAttrs::parse(fields.named.first().unwrap())
            .unwrap()
            .encode_order
            .is_none());
    }

    #[test]
    fn encode_order_none() {
        let input = quote!({
            #[dbn(c_char)]
            pub f: c_char,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        assert_eq!(fields.named.len(), 1);
        assert!(DbnFieldAttrs::parse(fields.named.first().unwrap())
            .unwrap()
            .encode_order
            .is_none());
    }

    #[test]
    fn encode_order_only() {
        let input = quote!({
            #[dbn(encode_order(3))]
            pub b: bool,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        assert_eq!(fields.named.len(), 1);
        assert_eq!(
            DbnFieldAttrs::parse(fields.named.first().unwrap())
                .unwrap()
                .encode_order
                .unwrap(),
            3
        );
    }

    #[test]
    fn encode_order_first() {
        let input = quote!({
            #[dbn(encode_order(2), unix_nanos)]
            pub ts: u64,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        assert_eq!(fields.named.len(), 1);
        assert_eq!(
            DbnFieldAttrs::parse(fields.named.first().unwrap())
                .unwrap()
                .encode_order
                .unwrap(),
            2
        );
    }

    #[test]
    fn encode_order_last() {
        let input = quote!({
            #[dbn(unix_nanos, encode_order(4))]
            pub ts: u64,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        assert_eq!(fields.named.len(), 1);
        assert_eq!(
            DbnFieldAttrs::parse(fields.named.first().unwrap())
                .unwrap()
                .encode_order
                .unwrap(),
            4
        );
    }

    #[test]
    fn parses_every_argument_kind() {
        let input = quote!({
            #[dbn(encode_order(1), field_id = STATUS_ACTION, fmt_method)]
            pub action: u16,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        let field = fields.named.first().unwrap();
        let attrs = DbnFieldAttrs::parse(field).unwrap();
        assert_eq!(attrs.field_id.unwrap(), "STATUS_ACTION");
        assert_eq!(attrs.encode_order, Some(1));
        assert_eq!(attrs.flags, [FMT_METHOD]);
    }

    #[test]
    fn encode_order_default() {
        let input = quote!({
            #[dbn(unix_nanos, encode_order)]
            pub ts: u64,
        });
        let fields = syn::parse2::<FieldsNamed>(input).unwrap();
        assert_eq!(fields.named.len(), 1);
        assert_eq!(
            DbnFieldAttrs::parse(fields.named.first().unwrap())
                .unwrap()
                .encode_order
                .unwrap(),
            0
        );
    }
}
