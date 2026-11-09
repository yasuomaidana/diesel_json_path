use diesel_json_path_derive::SqlFields;

#[derive(SqlFields)]
#[diesel_json(table = "users")]
struct A {
    id: i32,
}

fn main() {}
