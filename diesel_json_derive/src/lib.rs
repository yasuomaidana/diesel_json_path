use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr, Type};

/// Derives the `SqlFields` trait for a struct, generating SQL field accessor methods.
///
/// # Attributes
/// - `#[diesel_json(column = "col_name")]` — Specifies the root JSON column name (defaults to `"body"`).
/// - `#[json_path("some.nested.path")]` — Specifies a custom JSON path for a field (defaults to the field name).
/// - `#[sql_type]` — Reserved for future use.
///
/// # Generated Methods
/// For each named field in the struct, a static method `{field_name}_sql()` is generated,
/// returning a `diesel::expression::SqlLiteral` with the appropriate SQL type and JSON traversal expression.
///
/// # Example
/// ```rust
/// #[derive(SqlFields)]
/// #[diesel_json(column = "data")]
/// struct MyStruct {
///     #[json_path("user.age")]
///     age: i32,
///     name: String,
/// }
/// // Generates:
/// // MyStruct::age_sql()  -> SQL: (data->'user'->>'age')::int
/// // MyStruct::name_sql() -> SQL: data->>'name'
/// ```
#[proc_macro_derive(SqlFields, attributes(diesel_json, json_path, sql_type))]
pub fn sql_fields_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // 1. Get Root Column (default to "body")
    let root_column = input.attrs.iter()
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

    let methods = if let Data::Struct(data) = input.data {
        if let Fields::Named(fields) = data.fields {
            fields.named.iter().map(|f| {
                let field_name = f.ident.as_ref().unwrap();
                let method_name = quote::format_ident!("{}_sql", field_name);

                // 2. Get Path (default to field name)
                let path = f.attrs.iter()
                    .find(|a| a.path().is_ident("json_path"))
                    .and_then(|a| a.parse_args::<LitStr>().ok().map(|s| s.value()))
                    .unwrap_or_else(|| field_name.to_string());

                // 3. Determine Types and Casting
                let (diesel_type, pg_cast) = get_type_info(&f.ty);
                let sql_expr = generate_postgresql_json_expr(&root_column, &path, &diesel_type, pg_cast);

                quote! {
                    impl #struct_name {
                        pub fn #method_name() -> diesel::expression::SqlLiteral<#diesel_type> {
                            diesel::dsl::sql::<#diesel_type>(#sql_expr)
                        }
                    }
                }
            }).collect::<Vec<_>>()
        } else { vec![] }
    } else { vec![] };

    TokenStream::from(quote! { #(#methods)* })
}

fn get_type_info(ty: &Type) -> (proc_macro2::TokenStream, Option<&'static str>) {
    let mut current_ty = ty;
    let mut is_nullable = false;

    // Simple Option detection
    if let Type::Path(tp) = ty {
        if tp.path.segments.last().unwrap().ident == "Option" {
            is_nullable = true;
            // Extract T from Option<T>
            if let syn::PathArguments::AngleBracketed(args) = &tp.path.segments.last().unwrap().arguments {
                if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                    current_ty = inner;
                }
            }
        }
    }

    let type_name = if let Type::Path(tp) = current_ty {
        tp.path.segments.last().unwrap().ident.to_string()
    } else {
        "String".to_string()
    };

    let (base_diesel, cast) = match type_name.as_str() {
        "i32" => (quote!(diesel::sql_types::Integer), Some("int")),
        "i64" => (quote!(diesel::sql_types::BigInt), Some("bigint")),
        "f32" => (quote!(diesel::sql_types::Float), Some("real")),
        "f64" => (quote!(diesel::sql_types::Double), Some("double precision")),
        "bool" => (quote!(diesel::sql_types::Bool), Some("boolean")),
        "Value" => (quote!(diesel::sql_types::Jsonb), None),
        _ => (quote!(diesel::sql_types::Text), None),
    };

    if is_nullable {
        (quote!(diesel::sql_types::Nullable<#base_diesel>), cast)
    } else {
        (base_diesel, cast)
    }
}

fn generate_postgresql_json_expr(column: &str, path: &str, diesel_ty: &proc_macro2::TokenStream, cast: Option<&str>) -> String {
    let parts: Vec<&str> = path.split('.').collect();
    let mut sql = column.to_string();
    let is_jsonb = diesel_ty.to_string().contains("Jsonb");

    for (i, part) in parts.iter().enumerate() {
        let is_last = i == parts.len() - 1;
        let op = if is_last && !is_jsonb { "->>" } else { "->" };
        sql.push_str(&format!("{}'{}'", op, part));
    }

    if let Some(c) = cast {
        format!("({})::{}", sql, c)
    } else {
        sql
    }
}