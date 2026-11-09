#![doc = include_str!("../README.md")]

mod attrs;
mod expand;
mod types;

use proc_macro::TokenStream;

/// Derives SQL JSON path helper APIs for a struct.
///
/// This macro reads field metadata from `#[json_path(...)]` and root metadata
/// from `#[diesel_json(...)]`, then generates a path builder and static
/// shortcut methods for Diesel SQL expressions.
#[proc_macro_derive(SqlFields, attributes(diesel_json, json_path, sql_type))]
pub fn sql_fields_derive(input: TokenStream) -> TokenStream {
    expand::expand(input.into()).into()
}
