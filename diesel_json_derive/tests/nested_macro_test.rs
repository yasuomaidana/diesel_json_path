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

struct A{
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
