use diesel_json_path_derive::SqlFields;

#[derive(SqlFields)]
#[diesel_json(column = 5)]
struct A {
    id: i32,
}

fn main() {}
