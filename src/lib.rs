#![doc = include_str!("../README.md")]

mod db;
mod join;

use proc_macro as pc;
use proc_macro2::Span;
use std::fmt;
use syn::Ident;

/// Printing errors at the current span
fn s_err(span: proc_macro2::Span, msg: impl fmt::Display) -> syn::Error {
    syn::Error::new(span, msg)
}

/// Capitalizing ident
///
/// # Example
///
/// `user_name` => `UserName`
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

/// Pre-Extending ident
///
/// # Example
///
/// `pre_extend_ident(ident, "extend_")` => `extend_ident`
fn pre_extend_ident(ident: &Ident, pre_extend: &str) -> Ident {
    Ident::new(
        &(pre_extend.to_string() + &ident.to_string()),
        Span::call_site(),
    )
}

/// Creates the Database struct with the fitting logic
///
/// ## Functions
/// - `add_#table`
/// - `get_#table`
/// - `edit_#table`
/// - `delete_#table`
/// - `search_#table`
///
/// ## Example
/// ```
/// use light_magic::db;
///
/// db! {
///     // `users` is the table name
///     // `{...}` is the table data
///     // the first field, like here `name`, is the `primary_key`
///     user => { id: usize, name: String, kind: String },
///     // using [...] after the table name you can add your own derives
///     // like here `PartialEq`
///     criminal: [PartialEq] => { user_name: String, entry: String }
/// }
/// ```
#[proc_macro]
pub fn db(input: pc::TokenStream) -> pc::TokenStream {
    match db::db_inner(input.into()) {
        Ok(result) => result.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

/// Joins Data in the Database together
///
/// ## Example
/// ```
/// use light_magic::{db, join};
///
/// db! {
///     user => { id: usize, name: String, kind: String },
///     criminal => { user_name: String, entry: String }
/// }
///
/// let db = Database::new();
/// /// Firstly specify the to db which should be used, then the key,
/// /// and lastly the joined items with the field which should be joined
/// let joined = join!(db, "Nils", user => name, criminal => user_name);
/// ```
#[proc_macro]
pub fn join(input: pc::TokenStream) -> pc::TokenStream {
    match join::join_inner(input.into()) {
        Ok(result) => result.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
