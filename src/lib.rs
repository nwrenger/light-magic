#![doc = include_str!("../README.md")]

use std::fmt;

use proc_macro as pc;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Ident, Token, Type,
};

fn s_err(span: proc_macro2::Span, msg: impl fmt::Display) -> syn::Error {
    syn::Error::new(span, msg)
}
/// Creates the Database struct with the fitting logic
///
/// ## Functions
/// - `insert_#table`
/// - `get_#table`
/// - `delete_#table`
/// - `search_#table`
///
/// ## Example
/// ```rs
/// db! {
///     // `users` is the table name
///     user => { id: usize, name: String, kind: String },
///     // `{...}` is the table data
///     permission => { user_name: String, level: Level },
///     // the first field, like here `user_name`, is the `primary_key`
///     criminal => { user_name: String, entry: String }
/// }
/// ```
#[proc_macro]
pub fn db(input: pc::TokenStream) -> pc::TokenStream {
    match db_inner(input.into()) {
        Ok(result) => result.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn db_inner(input: TokenStream) -> syn::Result<TokenStream> {
    let _span = input.span();
    let args = syn::parse2::<Args>(input)?;

    let tables = args.tables;

    let mut db_tables = TokenStream::new();
    let mut db_functions = TokenStream::new();
    let mut structs = TokenStream::new();

    for Table { name, fields } in tables {
        if fields.is_empty() {
            return Err(s_err(name.span(), "the fields of a table cannot be empty"));
        }
        let struct_ident = capitalize_ident(&name);
        let matches = generate_matches(&fields);
        structs.extend(quote! {
            #[derive(Debug, Clone)]
            struct #struct_ident { #fields }

            impl #struct_ident {
                #matches
            }
        });
        let key = &fields[0].ty;
        let key_name = &fields[0].name;
        db_tables.extend(quote! {
            /// The Table, never access this directly and use the functions on the `Database`
            #name: Arc<Mutex<BTreeMap<#key, #struct_ident>>>,
        });
        let insert_name = pre_extend_ident(&name, "insert_");
        let get_name = pre_extend_ident(&name, "get_");
        let delete_name = pre_extend_ident(&name, "delete_");
        let search_name = pre_extend_ident(&name, "search_");
        db_functions.extend(quote! {
            /// Insert data, works in parallel
            fn #insert_name(&self, value: #struct_ident) {
                if let Ok(mut table) = self.#name.lock() {
                    table.insert(value.#key_name.clone(), value);
                }
            }
            /// Get data, works in parallel
            fn #get_name(&self, #key_name: &#key) -> Option<#struct_ident> {
                if let Ok(table) = self.#name.lock() {
                    table.get(#key_name).cloned()
                } else {
                    None
                }
            }
            /// Delete data, works in parallel
            fn #delete_name(&self, #key_name: &#key) {
                if let Ok(mut table) = self.#name.lock() {
                    table.remove(#key_name);
                }
            }
            /// Search the data, works in parallel
            fn #search_name(&self, search: &str) -> Vec<#struct_ident> {
                if let Ok(table) = self.#name.lock() {
                    table.iter().map(|(_, val)| val.clone()).filter(|val| val.matches(search)).collect()
                } else {
                    vec![]
                }
            }
        })
    }

    Ok(quote! {
        use std::collections::BTreeMap;
        use std::sync::{Arc, Mutex};

        #[derive(Default)]
        /// The Database Struct
        pub struct Database {
            #db_tables
        }

        impl Database {
            /// Make a new Database Instance
            fn new() -> Self {
                Database::default()
            }

            #db_functions
        }

        #structs
    })
}

fn generate_matches(fields: &Punctuated<Field, Token![,]>) -> proc_macro2::TokenStream {
    let matches = fields.iter().map(|field| {
        let field_name = &field.name;
        quote! {
            let val = format!("{:?}", self.#field_name);
            if val.contains(query) {
                return true;
            }
        }
    });

    quote! {
        pub fn matches(&self, query: &str) -> bool {
            #(#matches)*
            false
        }
    }
}

fn capitalize_ident(ident: &Ident) -> Ident {
    let capitalized: String = ident
        .to_string()
        .split('_')
        .map(|part| {
            let mut c = part.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect();

    Ident::new(&capitalized, Span::call_site())
}

fn pre_extend_ident(ident: &Ident, pre_extend: &str) -> Ident {
    Ident::new(
        &(pre_extend.to_string() + &ident.to_string()),
        Span::call_site(),
    )
}

struct Args {
    tables: Vec<Table>,
}

struct Table {
    name: Ident,
    fields: Punctuated<Field, Token![,]>,
}

struct Field {
    name: Ident,
    _colon_token: Token![:],
    ty: Type,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Field {
            name: input.parse()?,
            _colon_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let ty = &self.ty;
        tokens.extend(quote! { #name: #ty })
    }
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut tables = Vec::new();

        while !input.is_empty() {
            let ident = syn::Ident::parse(input)?;
            <syn::Token![=>]>::parse(input)?;

            let content;
            syn::braced!(content in input);
            let parsed_fields = content.parse_terminated(Field::parse, Token![,])?;
            tables.push(Table {
                name: ident,
                fields: parsed_fields,
            });

            // `,` is optional
            if !input.is_empty() && input.peek(syn::Token![,]) {
                <syn::Token![,]>::parse(input)?;
            }
        }

        Ok(Self { tables })
    }
}
