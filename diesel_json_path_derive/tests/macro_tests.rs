use diesel::expression::SqlLiteral;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Integer, SingleValue};
use diesel_json_path_derive::SqlFields;
use rstest::rstest;

table! {
    users (id) {
        id -> Int4,
        metadata -> Jsonb,
    }
}

#[allow(dead_code)]
#[derive(SqlFields)]
#[diesel_json(column = "metadata")]
struct Settings {
    theme: String,
}

#[allow(dead_code)]
#[derive(SqlFields)]
#[diesel_json(column = "metadata")]
struct UserProfile {
    // defaults to metadata->>'id'::int
    id: i32,
    // metadata->'settings'->>'theme'
    // #[json_path("settings.theme")]
    settings: Option<Settings>,
}

#[allow(dead_code)]
#[derive(SqlFields)]
struct UserProfile2 {
    // defaults to body->>'id'::int
    id: i32,
    // metadata->'settings'->>'theme'
    #[json_path("settings.theme")]
    theme: Option<String>,
}

#[allow(dead_code)]
#[derive(SqlFields)]
#[diesel_json(column = "metadata")]
struct UnsignedProfile {
    count: u32,
}

fn assert_integer_expr<E: Expression<SqlType = Integer>>(_e: &E) {}

fn render<ST: SingleValue>(expr: SqlLiteral<ST>) -> String {
    diesel::debug_query::<Pg, _>(&users::table.select(expr)).to_string()
}

#[rstest]
#[case::default_column(
    render(UserProfile2::id_sql()),
    "SELECT (body->>'id')::int FROM \"users\" -- binds: []"
)]
#[case::custom_column(
    render(UserProfile::id_sql()),
    "SELECT (metadata->>'id')::int FROM \"users\" -- binds: []"
)]
#[case::unsigned_is_bigint(
    render(UnsignedProfile::count_sql()),
    "SELECT (metadata->>'count')::bigint FROM \"users\" -- binds: []"
)]
fn generates_exact_sql(#[case] sql: String, #[case] expected: &str) {
    assert_eq!(sql, expected);
}

#[test]
fn nested_path_and_custom_json_path() {
    assert!(render(UserProfile::settings().theme_sql()).contains("metadata->'settings'->>'theme'"));
    assert!(render(UserProfile2::theme_sql()).contains("body->>'settings.theme'"));
}

#[test]
fn expression_sql_types_are_typed() {
    assert_integer_expr(&UserProfile::id_sql());
    assert_integer_expr(&UserProfile2::id_sql());
}
