use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, LitStr, Type, parse_macro_input};

#[proc_macro_derive(SqlFields, attributes(diesel_json, json_path, sql_type))]
pub fn sql_fields_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let builder_name = format_ident!("{}PathBuilder", struct_name);

    let root_column = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("diesel_json"))
        .and_then(|a| {
            let mut col = None;
            let _ = a.parse_nested_meta(|meta| {
                if meta.path.is_ident("column") {
                    let value: LitStr = meta.value()?.parse()?;
                    col = Some(value.value());
                }
                Ok(())
            });
            col
        })
        .unwrap_or_else(|| "body".to_string());

    let fields_data = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields
                .named
                .iter()
                .map(|f| {
                    let field_name = f.ident.as_ref().unwrap();
                    let path = f
                        .attrs
                        .iter()
                        .find(|a| a.path().is_ident("json_path"))
                        .and_then(|a| a.parse_args::<LitStr>().ok().map(|s| s.value()))
                        .unwrap_or_else(|| field_name.to_string());

                    let (base_diesel_type, pg_cast, inner_ty_name, is_option) =
                        get_field_details(&f.ty);
                    (
                        field_name.clone(),
                        path,
                        base_diesel_type,
                        pg_cast,
                        inner_ty_name,
                        is_option,
                    )
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let builder_methods = fields_data.iter().map(|(field_name, path, base_diesel_type, pg_cast, inner_ty_name, is_option)| {
        if let Some(base_diesel_type) = base_diesel_type {
            // Primitive field: generate a `_sql()` method.
            let method_name = format_ident!("{}_sql", field_name);
            let diesel_type = if *is_option {
                quote! { diesel::sql_types::Nullable<#base_diesel_type> }
            } else {
                quote! { #base_diesel_type }
            };

            let final_op = if base_diesel_type.to_string().contains("Jsonb") { "->" } else { "->>" };

            let cast_expr = if let Some(c) = pg_cast {
                quote! { format!("({})::{}", sql, #c) }
            } else {
                quote! { sql }
            };

            quote! {
                pub fn #method_name(&self) -> diesel::expression::SqlLiteral<#diesel_type> {
                    let sql = format!("{}{}'{}'", self.base_path, #final_op, #path);
                    let sql_with_cast = #cast_expr;
                    diesel::dsl::sql::<#diesel_type>(&sql_with_cast)
                }
            }
        } else {
            // Nested struct field: generate a builder-returning method.
            let method_name = field_name;
            let nested_builder_name = format_ident!("{}PathBuilder", inner_ty_name);

            quote! {
                pub fn #method_name(&self) -> #nested_builder_name {
                    #nested_builder_name { base_path: format!("{}->'{}'", self.base_path, #path) }
                }
            }
        }
    });

    let static_shortcuts = fields_data.iter().map(
        |(field_name, _, base_diesel_type, _, inner_ty_name, is_option)| {
            if let Some(base_diesel_type) = base_diesel_type {
                let method_name = format_ident!("{}_sql", field_name);
                let diesel_type = if *is_option {
                    quote! { diesel::sql_types::Nullable<#base_diesel_type> }
                } else {
                    quote! { #base_diesel_type }
                };
                quote! {
                    pub fn #method_name() -> diesel::expression::SqlLiteral<#diesel_type> {
                        let builder = Self::sql_path_builder();
                        builder.#method_name()
                    }
                }
            } else {
                let method_name = field_name;
                let nested_builder_name = format_ident!("{}PathBuilder", inner_ty_name);
                quote! {
                    pub fn #method_name() -> #nested_builder_name {
                        let builder = Self::sql_path_builder();
                        builder.#method_name()
                    }
                }
            }
        },
    );

    let expanded = quote! {
        #[derive(Clone)]
        pub struct #builder_name {
            pub base_path: String,
        }

        impl #builder_name {
            #(#builder_methods)*
        }

        impl #struct_name {
            pub fn sql_path_builder() -> #builder_name {
                #builder_name { base_path: #root_column.to_string() }
            }

            #(#static_shortcuts)*
        }
    };

    TokenStream::from(expanded)
}

fn get_field_details(
    ty: &Type,
) -> (
    Option<proc_macro2::TokenStream>,
    Option<&'static str>,
    String,
    bool,
) {
    let (is_option, inner_ty) = if let Type::Path(tp) = ty {
        if let Some(segment) = tp.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        (true, inner.clone())
                    } else {
                        (false, ty.clone())
                    }
                } else {
                    (false, ty.clone())
                }
            } else {
                (false, ty.clone())
            }
        } else {
            (false, ty.clone())
        }
    } else {
        (false, ty.clone())
    };

    let type_name = if let Type::Path(tp) = &inner_ty {
        tp.path.segments.last().unwrap().ident.to_string()
    } else {
        quote!(#inner_ty).to_string().replace(' ', "")
    };

    let (base_diesel, cast) = match type_name.as_str() {
        "i32" => (Some(quote!(diesel::sql_types::Integer)), Some("int")),
        "i64" => (Some(quote!(diesel::sql_types::BigInt)), Some("bigint")),
        "f32" => (Some(quote!(diesel::sql_types::Float)), Some("real")),
        "f64" => (
            Some(quote!(diesel::sql_types::Double)),
            Some("double precision"),
        ),
        "bool" => (Some(quote!(diesel::sql_types::Bool)), Some("boolean")),
        "String" => (Some(quote!(diesel::sql_types::Text)), None),
        "Value" | "Jsonb" => (Some(quote!(diesel::sql_types::Jsonb)), None),
        _ => (None, None),
    };

    (base_diesel, cast, type_name, is_option)
}
