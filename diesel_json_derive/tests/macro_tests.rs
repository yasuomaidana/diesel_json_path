use std::any::Any;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Integer;
use diesel_json_derive::SqlFields;

table! {
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
    let id_expr = UserProfile::id_sql();
    let theme_expr = UserProfile::theme_sql();

    let id_sql =
        diesel::debug_query::<Pg, _>(&users::table.select(id_expr)).to_string();
    let theme_sql =
        diesel::debug_query::<Pg, _>(&users::table.select(theme_expr)).to_string();

    assert_eq!(
        id_sql,
        "SELECT (metadata->>'id')::int FROM \"users\" -- binds: []"
    );

    assert!(theme_sql.contains("metadata->'settings'->>'theme'"));
}
