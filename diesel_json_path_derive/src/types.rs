//! Mapping of Rust field types to Diesel SQL types and PostgreSQL casts.

use quote::quote;
use syn::{GenericArgument, PathArguments, PathSegment, Type};

/// `(rust type, diesel sql type, postgres cast)` for types without generics.
const SIMPLE: &[(&str, &str, Option<&str>)] = &[
    ("i8", "SmallInt", Some("smallint")),
    ("i16", "SmallInt", Some("smallint")),
    ("i32", "Integer", Some("int")),
    ("i64", "BigInt", Some("bigint")),
    ("u8", "SmallInt", Some("smallint")),
    ("u16", "Integer", Some("int")),
    // bigint for u32 avoids overflow around i32::MAX.
    ("u32", "BigInt", Some("bigint")),
    // numeric for u64 preserves the full unsigned range.
    ("u64", "Numeric", Some("numeric")),
    ("f32", "Float", Some("real")),
    ("f64", "Double", Some("double precision")),
    ("bool", "Bool", Some("boolean")),
    ("String", "Text", None),
    ("str", "Text", None),
    ("Uuid", "Uuid", Some("uuid")),
    ("NaiveDateTime", "Timestamp", Some("timestamp")),
    ("NaiveDate", "Date", Some("date")),
    ("NaiveTime", "Time", Some("time")),
    ("Decimal", "Numeric", Some("numeric")),
    ("Value", "Jsonb", None),
    ("Jsonb", "Jsonb", None),
];

/// `(rust type, first generic argument, diesel sql type, postgres cast)`.
const GENERIC: &[(&str, &str, &str, Option<&str>)] = &[
    ("DateTime", "Utc", "Timestamptz", Some("timestamptz")),
    ("Vec", "u8", "Binary", Some("bytea")),
];

/// What the macro knows about the type of one field.
#[derive(Debug, PartialEq)]
pub struct FieldType {
    /// Diesel SQL type name, `None` for nested structs.
    pub sql_type: Option<&'static str>,
    /// PostgreSQL cast applied to the extracted text.
    pub cast: Option<&'static str>,
    /// Last path segment of the type, used to name nested builders.
    pub name: String,
    /// Whether the field was wrapped in `Option`.
    pub is_option: bool,
}

/// Classifies the type of a field.
pub fn field_type(ty: &Type) -> FieldType {
    let (is_option, inner) = unwrap_option(ty);
    let known = known_type(inner);
    FieldType {
        sql_type: known.map(|(sql, _)| sql),
        cast: known.and_then(|(_, cast)| cast),
        name: display_name(inner),
        is_option,
    }
}

fn unwrap_option(ty: &Type) -> (bool, &Type) {
    if let Some(segment) = last_segment(ty)
        && segment.ident == "Option"
        && let Some(inner) = first_generic(segment)
    {
        return (true, inner);
    }

    (false, ty)
}

fn last_segment(ty: &Type) -> Option<&PathSegment> {
    match ty {
        Type::Path(tp) => tp.path.segments.last(),
        _ => None,
    }
}

fn display_name(ty: &Type) -> String {
    match last_segment(ty) {
        Some(segment) => segment.ident.to_string(),
        None => quote!(#ty).to_string().replace(' ', ""),
    }
}

fn known_type(ty: &Type) -> Option<(&'static str, Option<&'static str>)> {
    let segment = last_segment(ty)?;
    let ident = segment.ident.to_string();
    let simple = SIMPLE.iter().find(|(name, ..)| *name == ident);
    if let Some(&(_, sql, cast)) = simple {
        return Some((sql, cast));
    }
    let arg = first_generic(segment)
        .and_then(last_segment)
        .map(|s| s.ident.to_string());
    GENERIC
        .iter()
        .find(|(name, first, ..)| *name == ident && arg.as_deref() == Some(*first))
        .map(|&(_, _, sql, cast)| (sql, cast))
}

fn first_generic(segment: &PathSegment) -> Option<&Type> {
    match &segment.arguments {
        PathArguments::AngleBracketed(args) => match args.args.first() {
            Some(GenericArgument::Type(ty)) => Some(ty),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use syn::parse_quote;

    #[rstest]
    #[case(parse_quote!(i8), Some("SmallInt"), Some("smallint"))]
    #[case(parse_quote!(u64), Some("Numeric"), Some("numeric"))]
    #[case(parse_quote!(String), Some("Text"), None)]
    #[case(parse_quote!(serde_json::Value), Some("Jsonb"), None)]
    #[case(parse_quote!(chrono::DateTime<chrono::Utc>), Some("Timestamptz"), Some("timestamptz"))]
    #[case(parse_quote!(Vec<u8>), Some("Binary"), Some("bytea"))]
    #[case(parse_quote!(Vec<String>), None, None)]
    #[case(parse_quote!(DateTime<Local>), None, None)]
    #[case(parse_quote!(DateTime), None, None)]
    #[case(parse_quote!(Nested), None, None)]
    #[case(parse_quote!((i32, i32)), None, None)]
    fn maps_known_types(
        #[case] ty: Type,
        #[case] sql_type: Option<&str>,
        #[case] cast: Option<&str>,
    ) {
        let found = field_type(&ty);
        assert_eq!((found.sql_type, found.cast), (sql_type, cast));
    }

    #[rstest]
    #[case(parse_quote!(i32), false, "i32")]
    #[case(parse_quote!(Option<i32>), true, "i32")]
    #[case(parse_quote!(Option<a::Nested>), true, "Nested")]
    #[case(parse_quote!(Option), false, "Option")]
    #[case(parse_quote!(Option<'a>), false, "Option")]
    #[case(parse_quote!(&str), false, "&str")]
    #[case(parse_quote!([u8; 4]), false, "[u8;4]")]
    fn reports_option_and_name(#[case] ty: Type, #[case] is_option: bool, #[case] name: &str) {
        let found = field_type(&ty);
        assert_eq!((found.is_option, found.name.as_str()), (is_option, name));
    }
}
