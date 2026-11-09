//! Code generation for `#[derive(SqlFields)]`, independent of `proc_macro`.

use crate::attrs::{field_path, root_column};
use crate::types::{FieldType, field_type};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Data, DataStruct, DeriveInput, Field, Fields, Ident, Path, parse_str};

/// One struct field, ready to generate code for.
struct Spec {
    name: Ident,
    path: String,
    ty: FieldType,
}

impl Spec {
    fn new(field: &Field) -> syn::Result<Self> {
        Ok(Spec {
            name: field.ident.clone().expect("named field"),
            path: field_path(field)?,
            ty: field_type(&field.ty),
        })
    }

    /// Name of the generated method for a primitive field.
    fn sql_method(&self) -> Ident {
        format_ident!("{}_sql", self.name)
    }

    /// Name of the builder generated for a nested struct field.
    fn nested_builder(&self) -> Ident {
        format_ident!("{}PathBuilder", self.ty.name)
    }
}

/// Expands the derive input, turning every error into a `compile_error!`.
pub fn expand(input: TokenStream) -> TokenStream {
    try_expand(input).unwrap_or_else(|e| e.to_compile_error())
}

fn try_expand(input: TokenStream) -> syn::Result<TokenStream> {
    let input: DeriveInput = syn::parse2(input)?;
    let root = root_column(&input.attrs)?;
    let specs = named_fields(&input)?
        .iter()
        .map(Spec::new)
        .collect::<syn::Result<Vec<_>>>()?;

    let struct_name = &input.ident;
    let builder_name = format_ident!("{}PathBuilder", struct_name);
    let methods = specs.iter().map(builder_method);
    let shortcuts = specs.iter().map(static_shortcut);

    Ok(quote! {
        #[derive(Clone)]
        pub struct #builder_name {
            pub base_path: String,
        }

        impl #builder_name {
            #(#methods)*
        }

        impl #struct_name {
            pub fn sql_path_builder() -> #builder_name {
                #builder_name { base_path: #root.to_string() }
            }

            #(#shortcuts)*
        }
    })
}

fn named_fields(input: &DeriveInput) -> syn::Result<&Punctuated<Field, Comma>> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named),
            ..
        }) => Ok(&named.named),
        _ => Err(syn::Error::new_spanned(
            &input.ident,
            "SqlFields can only be derived for structs with named fields",
        )),
    }
}

/// The Diesel SQL type of a primitive field, `Nullable` when optional.
fn diesel_type(spec: &Spec, sql_type: &str) -> TokenStream {
    let path: Path = parse_str(&format!("diesel::sql_types::{sql_type}")).expect("sql type path");
    if spec.ty.is_option {
        quote! { diesel::sql_types::Nullable<#path> }
    } else {
        quote! { #path }
    }
}

fn builder_method(spec: &Spec) -> TokenStream {
    let path = &spec.path;
    let Some(sql_type) = spec.ty.sql_type else {
        let (method, nested) = (&spec.name, spec.nested_builder());
        return quote! {
            pub fn #method(&self) -> #nested {
                #nested { base_path: format!("{}->'{}'", self.base_path, #path) }
            }
        };
    };

    let (method, diesel_type) = (spec.sql_method(), diesel_type(spec, sql_type));
    let op = if sql_type == "Jsonb" { "->" } else { "->>" };
    let cast = match spec.ty.cast {
        Some(cast) => quote! { format!("({})::{}", sql, #cast) },
        None => quote! { sql },
    };
    quote! {
        pub fn #method(&self) -> diesel::expression::SqlLiteral<#diesel_type> {
            let sql = format!("{}{}'{}'", self.base_path, #op, #path);
            let sql_with_cast = #cast;
            diesel::dsl::sql::<#diesel_type>(&sql_with_cast)
        }
    }
}

fn static_shortcut(spec: &Spec) -> TokenStream {
    let Some(sql_type) = spec.ty.sql_type else {
        let (method, nested) = (&spec.name, spec.nested_builder());
        return quote! {
            pub fn #method() -> #nested {
                Self::sql_path_builder().#method()
            }
        };
    };

    let (method, diesel_type) = (spec.sql_method(), diesel_type(spec, sql_type));
    quote! {
        pub fn #method() -> diesel::expression::SqlLiteral<#diesel_type> {
            Self::sql_path_builder().#method()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn expanded(input: TokenStream) -> String {
        expand(input).to_string()
    }

    #[rstest]
    #[case(quote!(struct A { id: i32 }), "pub fn id_sql () -> diesel :: expression :: SqlLiteral < diesel :: sql_types :: Integer >")]
    #[case(quote!(struct A { id: Option<i32> }), "Nullable < diesel :: sql_types :: Integer >")]
    #[case(quote!(struct A { id: i32 }), "base_path : \"body\" . to_string ()")]
    #[case(quote!(#[diesel_json(column = "meta")] struct A { id: i32 }), "base_path : \"meta\" . to_string ()")]
    #[case(quote!(struct A { name: String }), "\"->>\"")]
    #[case(quote!(struct A { raw: serde_json::Value }), "\"->\"")]
    #[case(quote!(struct A { inner: Inner }), "pub fn inner (& self) -> InnerPathBuilder")]
    #[case(quote!(struct A { inner: Inner }), "pub fn inner () -> InnerPathBuilder")]
    #[case(quote!(struct A { #[json_path("x.y")] id: i32 }), "\"x.y\"")]
    fn generates_the_expected_code(#[case] input: TokenStream, #[case] fragment: &str) {
        assert!(expanded(input).contains(fragment));
    }

    #[rstest]
    #[case(quote!(struct A(i32);))]
    #[case(quote!(struct A;))]
    #[case(quote!(enum A { B }))]
    #[case(quote!(#[diesel_json(nope = "x")] struct A { id: i32 }))]
    #[case(quote!(struct A { #[json_path(5)] id: i32 }))]
    #[case(quote!(fn not_a_struct() {}))]
    fn reports_invalid_input_as_compile_errors(#[case] input: TokenStream) {
        assert!(expanded(input).contains("compile_error"));
    }
}
