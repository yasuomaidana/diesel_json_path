use diesel_json_path_derive::SqlFields;

#[derive(SqlFields)]
struct A {
    #[json_path(5)]
    id: i32,
}

fn main() {}
