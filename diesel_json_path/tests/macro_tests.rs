use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types::SingleValue;
use diesel_json_path::SqlFields;
use rstest::rstest;
use serde_json::Value;

table! {
    users (id) {
        id -> Int4,
        metadata -> Jsonb,
    }
}

#[allow(dead_code)]
#[derive(SqlFields)]
#[diesel_json(column = "metadata")]
struct UserProfile {
    // Basic types
    id: i32,
    score: i64,
    rating: f32,
    balance: f64,
    is_active: bool,
    username: String,
    login_count: u32,
    total_bytes: u64,

    // JSON type
    raw_data: Value,
    raw_data_qualified: serde_json::Value,

    // Nested structures
    settings: UserSettings,

    // Optional types (should produce Nullable)
    nickname: Option<String>,
}

#[allow(dead_code)]
#[derive(SqlFields)]
struct UserSettings {
    #[json_path("color_theme")] // Testing custom path mapping within a nested struct
    theme: String,
    notifications_enabled: bool,
    advanced: Option<AdvancedSettings>,
}

#[allow(dead_code)]
#[derive(SqlFields)]
struct AdvancedSettings {
    beta_features: bool,
    level: i32,
}

fn render<ST: SingleValue>(expr: SqlLiteral<ST>) -> String {
    diesel::debug_query::<diesel::pg::Pg, _>(&users::table.select(expr)).to_string()
}

#[rstest]
#[case::int(render(UserProfile::id_sql()), "(metadata->>'id')::int")]
#[case::bigint(render(UserProfile::score_sql()), "(metadata->>'score')::bigint")]
#[case::real(render(UserProfile::rating_sql()), "(metadata->>'rating')::real")]
#[case::double(
    render(UserProfile::balance_sql()),
    "(metadata->>'balance')::double precision"
)]
#[case::boolean(
    render(UserProfile::is_active_sql()),
    "(metadata->>'is_active')::boolean"
)]
#[case::text_has_no_cast(render(UserProfile::username_sql()), "metadata->>'username'")]
#[case::u32_is_bigint(
    render(UserProfile::login_count_sql()),
    "(metadata->>'login_count')::bigint"
)]
#[case::u64_is_numeric(
    render(UserProfile::total_bytes_sql()),
    "(metadata->>'total_bytes')::numeric"
)]
#[case::jsonb(render(UserProfile::raw_data_sql()), "metadata->'raw_data'")]
#[case::jsonb_qualified(
    render(UserProfile::raw_data_qualified_sql()),
    "metadata->'raw_data_qualified'"
)]
#[case::optional(render(UserProfile::nickname_sql()), "metadata->>'nickname'")]
#[case::nested_custom_path(render(UserProfile::settings().theme_sql()), "metadata->'settings'->>'color_theme'")]
#[case::nested_bool(render(UserProfile::settings().notifications_enabled_sql()), "(metadata->'settings'->>'notifications_enabled')::boolean")]
#[case::nested_twice_bool(render(UserProfile::settings().advanced().beta_features_sql()), "(metadata->'settings'->'advanced'->>'beta_features')::boolean")]
#[case::nested_twice_int(render(UserProfile::settings().advanced().level_sql()), "(metadata->'settings'->'advanced'->>'level')::int")]
fn generates_the_expected_sql(#[case] sql: String, #[case] expected: &str) {
    assert!(sql.contains(expected), "{sql} does not contain {expected}");
}
