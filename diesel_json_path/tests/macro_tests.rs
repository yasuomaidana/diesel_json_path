use diesel::prelude::*;
use diesel_json_path::SqlFields;
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

#[test]
fn test_recursive_path_and_all_supported_types() {
    // 1. Test basic types and their specific casts
    let query_i32 = users::table.select(UserProfile::id_sql());
    let sql_i32 = diesel::debug_query::<diesel::pg::Pg, _>(&query_i32).to_string();
    assert!(sql_i32.contains("(metadata->>'id')::int"));

    let query_i64 = users::table.select(UserProfile::score_sql());
    let sql_i64 = diesel::debug_query::<diesel::pg::Pg, _>(&query_i64).to_string();
    assert!(sql_i64.contains("(metadata->>'score')::bigint"));

    let query_f32 = users::table.select(UserProfile::rating_sql());
    let sql_f32 = diesel::debug_query::<diesel::pg::Pg, _>(&query_f32).to_string();
    assert!(sql_f32.contains("(metadata->>'rating')::real"));

    let query_f64 = users::table.select(UserProfile::balance_sql());
    let sql_f64 = diesel::debug_query::<diesel::pg::Pg, _>(&query_f64).to_string();
    assert!(sql_f64.contains("(metadata->>'balance')::double precision"));

    let query_bool = users::table.select(UserProfile::is_active_sql());
    let sql_bool = diesel::debug_query::<diesel::pg::Pg, _>(&query_bool).to_string();
    assert!(sql_bool.contains("(metadata->>'is_active')::boolean"));

    let query_string = users::table.select(UserProfile::username_sql());
    let sql_string = diesel::debug_query::<diesel::pg::Pg, _>(&query_string).to_string();
    assert!(sql_string.contains("metadata->>'username'")); // Strings shouldn't have casts

    let query_u32 = users::table.select(UserProfile::login_count_sql());
    let sql_u32 = diesel::debug_query::<diesel::pg::Pg, _>(&query_u32).to_string();
    assert!(sql_u32.contains("(metadata->>'login_count')::bigint"));

    let query_u64 = users::table.select(UserProfile::total_bytes_sql());
    let sql_u64 = diesel::debug_query::<diesel::pg::Pg, _>(&query_u64).to_string();
    assert!(sql_u64.contains("(metadata->>'total_bytes')::numeric"));

    // 2. Test JSONB value type (uses -> instead of ->> and no cast)
    let query_value = users::table.select(UserProfile::raw_data_sql());
    let sql_value = diesel::debug_query::<diesel::pg::Pg, _>(&query_value).to_string();
    assert!(sql_value.contains("metadata->'raw_data'"));

    let query_value_qualified = users::table.select(UserProfile::raw_data_qualified_sql());
    let sql_value_qualified =
        diesel::debug_query::<diesel::pg::Pg, _>(&query_value_qualified).to_string();
    assert!(sql_value_qualified.contains("metadata->'raw_data_qualified'"));

    // 3. Test Option / Nullable types
    let query_option = users::table.select(UserProfile::nickname_sql());
    let sql_option = diesel::debug_query::<diesel::pg::Pg, _>(&query_option).to_string();
    assert!(sql_option.contains("metadata->>'nickname'"));

    // 4. Test Recursive path builders (1 level deep)
    let query_nested_1 = users::table.select(UserProfile::settings().theme_sql());
    let sql_nested_1 = diesel::debug_query::<diesel::pg::Pg, _>(&query_nested_1).to_string();
    assert!(sql_nested_1.contains("metadata->'settings'->>'color_theme'"));

    let query_nested_bool =
        users::table.select(UserProfile::settings().notifications_enabled_sql());
    let sql_nested_bool = diesel::debug_query::<diesel::pg::Pg, _>(&query_nested_bool).to_string();
    assert!(sql_nested_bool.contains("(metadata->'settings'->>'notifications_enabled')::boolean"));

    // 5. Test Recursive path builders (2 levels deep)
    let query_nested_2 =
        users::table.select(UserProfile::settings().advanced().beta_features_sql());
    let sql_nested_2 = diesel::debug_query::<diesel::pg::Pg, _>(&query_nested_2).to_string();
    assert!(sql_nested_2.contains("(metadata->'settings'->'advanced'->>'beta_features')::boolean"));

    let query_nested_3 = users::table.select(UserProfile::settings().advanced().level_sql());
    let sql_nested_3 = diesel::debug_query::<diesel::pg::Pg, _>(&query_nested_3).to_string();
    assert!(sql_nested_3.contains("(metadata->'settings'->'advanced'->>'level')::int"));
}
