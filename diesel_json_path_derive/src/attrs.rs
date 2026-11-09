//! Parsing of the `#[diesel_json(...)]` and `#[json_path(...)]` attributes.

use syn::{Attribute, Field, LitStr};

/// Column used when no `#[diesel_json(column = "...")]` is given.
const DEFAULT_COLUMN: &str = "body";

/// Returns the JSON column named by `#[diesel_json(column = "...")]`.
pub fn root_column(attrs: &[Attribute]) -> syn::Result<String> {
    let mut column = DEFAULT_COLUMN.to_string();
    for attr in attrs.iter().filter(|a| a.path().is_ident("diesel_json")) {
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("column") {
                return Err(meta.error("unsupported key, expected `column = \"...\"`"));
            }
            column = meta.value()?.parse::<LitStr>()?.value();
            Ok(())
        })?;
    }
    Ok(column)
}

/// Returns the JSON key of a field: `#[json_path("...")]` or the field name.
pub fn field_path(field: &Field) -> syn::Result<String> {
    match field.attrs.iter().find(|a| a.path().is_ident("json_path")) {
        Some(attr) => Ok(attr.parse_args::<LitStr>()?.value()),
        None => Ok(field
            .ident
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;
    use rstest::rstest;
    use syn::parse::Parser;
    use syn::{DeriveInput, parse_quote};

    #[rstest]
    #[case(parse_quote!(struct A;), "body")]
    #[case(parse_quote!(#[diesel_json(column = "meta")] struct A;), "meta")]
    fn root_column_reads_the_attribute(#[case] input: DeriveInput, #[case] expected: &str) {
        assert_eq!(root_column(&input.attrs).unwrap(), expected);
    }

    #[rstest]
    #[case(parse_quote!(#[diesel_json(table = "x")] struct A;))]
    #[case(parse_quote!(#[diesel_json(column = 5)] struct A;))]
    fn root_column_rejects_invalid_attributes(#[case] input: DeriveInput) {
        assert!(root_column(&input.attrs).is_err());
    }

    #[rstest]
    #[case(quote!(plain: i32), "plain")]
    #[case(quote!(#[json_path("a.b")] plain: i32), "a.b")]
    fn field_path_reads_the_attribute(#[case] tokens: TokenStream, #[case] expected: &str) {
        let field = Parser::parse2(Field::parse_named, tokens).unwrap();
        assert_eq!(field_path(&field).unwrap(), expected);
    }

    #[test]
    fn field_path_rejects_a_non_string_path() {
        let tokens = quote!(#[json_path(5)] plain: i32);
        let field = Parser::parse2(Field::parse_named, tokens).unwrap();
        assert!(field_path(&field).is_err());
    }
}
