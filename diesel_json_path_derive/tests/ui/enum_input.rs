use diesel_json_path_derive::SqlFields;

#[derive(SqlFields)]
enum A {
    B,
}

fn main() {}
