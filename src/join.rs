use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Expr, Ident, Token,
};

use crate::{capitalize_ident, s_err};

pub(crate) fn join_inner(input: TokenStream) -> syn::Result<TokenStream> {
    let _span = input.span();
    let args = syn::parse2::<Args>(input)?;

    let db = args.db;
    let key = args.key;
    let joins = args.joins;

    if joins.is_empty() {
        return Err(s_err(Span::call_site(), "the joins cannot be empty"));
    }

    let mut tuple_inner = quote! {};

    for (
        i,
        Join {
            r#struct,
            _arrow,
            primary_key,
        },
    ) in joins.iter().enumerate()
    {
        let capped_struct = capitalize_ident(r#struct);
        let leading = if joins.len() - 1 != i {
            quote! {,}
        } else {
            quote! {}
        };
        tuple_inner.extend(quote! {
            if let Ok(table) = #db.#r#struct.lock() {
                let filtered = table.iter().map(|(_, val)| val.clone()).filter(|val| val.#primary_key == #key).collect::<Vec<#capped_struct>>();
                if !filtered.is_empty() {
                    Some(filtered)
                } else {
                    None
                }
            } else {
                None
            }#leading
        })
    }

    Ok(quote! {
        (#tuple_inner)
    })
}

struct Args {
    db: Ident,
    key: Expr,
    joins: Vec<Join>,
}

struct Join {
    r#struct: Ident,
    _arrow: Token![=>],
    primary_key: Ident,
}

impl Parse for Join {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Join {
            r#struct: input.parse()?,
            _arrow: input.parse()?,
            primary_key: input.parse()?,
        })
    }
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let db = syn::Ident::parse(input)?;
        <syn::Token![,]>::parse(input)?;
        let key = syn::Expr::parse(input)?;
        <syn::Token![,]>::parse(input)?;

        let mut joins = Vec::new();
        while !input.is_empty() {
            joins.push(Join::parse(input)?);
            if !input.is_empty() {
                <syn::Token![,]>::parse(input)?;
            }
        }

        Ok(Self { db, key, joins })
    }
}
