use diesel_json_path::SqlFields;
use diesel::prelude::*;

diesel::table! {
    users (id) {
        id -> Int4,
        metadata -> Jsonb,
    }
}

#[derive(SqlFields)]
#[diesel_json(column = "metadata")]
struct UserProfile {
    // defaults to metadata->>'id'::int
    id: i32,
    // metadata->'settings'->>'theme'
    #[json_path("settings.theme")]
    theme: Option<String>,
}

#[test]
fn test_macro_generation() {
    let query = users::table
        .select(UserProfile::id_sql())
        .filter(UserProfile::theme_sql().eq("dark"));

    let sql = diesel::debug_query::<diesel::pg::Pg, _>(&query).to_string();
    assert!(sql.contains("(metadata->>'id')::int"));
    assert!(sql.contains("metadata->'settings'->>'theme'"));
}