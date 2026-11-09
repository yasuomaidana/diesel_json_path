use diesel_json_path_derive::SqlFields;

#[derive(SqlFields)]
struct A(i32);

fn main() {}
