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

fn assert_integer_expr<E: Expression<SqlType = Integer>>(_e: &E) {}
#[test]
fn test_macro_generation() {
    let id_expr = UserProfile::id_sql();
    let theme_expr = UserProfile::settings().theme_sql();

    let id_sql = diesel::debug_query::<Pg, _>(&users::table.select(id_expr)).to_string();
    let theme_sql = diesel::debug_query::<Pg, _>(&users::table.select(theme_expr)).to_string();

    assert_eq!(
        id_sql,
        "SELECT (metadata->>'id')::int FROM \"users\" -- binds: []"
    );

    assert_integer_expr(&UserProfile::id_sql());

    println!("Generated SQL for theme: {}", theme_sql);
    assert!(theme_sql.contains("metadata->'settings'->>'theme'"));
}

#[test]
fn test_macro_generation2() {
    let id_expr = UserProfile2::id_sql();

    let id_sql = diesel::debug_query::<Pg, _>(&users::table.select(id_expr)).to_string();

    assert_eq!(
        id_sql,
        "SELECT (body->>'id')::int FROM \"users\" -- binds: []"
    );

    assert_integer_expr(&UserProfile2::id_sql());
}
