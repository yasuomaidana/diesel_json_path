//! Runs the generated expressions against a real PostgreSQL database.
//!
//! Needs `DATABASE_URL`; run with `cargo test -- --ignored` (CI does).

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use diesel::deserialize::FromSqlRow;
use diesel::expression::SqlLiteral;
use diesel::pg::Pg;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_types::{HasSqlType, SingleValue};
use diesel_json_path::SqlFields;
use rstest::{fixture, rstest};
use serde_json::{Value, json};
use std::str::FromStr;
use uuid::Uuid;

table! {
    djp_documents (id) {
        id -> Int4,
        body -> Jsonb,
    }
}

#[allow(dead_code)]
#[derive(SqlFields)]
struct Inner {
    level: i32,
}

#[allow(dead_code)]
#[derive(SqlFields)]
struct Doc {
    small: i16,
    int: i32,
    big: i64,
    unsigned: u64,
    real: f32,
    double: f64,
    flag: bool,
    text: String,
    id: Uuid,
    at: NaiveDateTime,
    day: NaiveDate,
    time: NaiveTime,
    stamp: DateTime<Utc>,
    bytes: Vec<u8>,
    raw: Value,
    missing: Option<i32>,
    inner: Inner,
}

fn document() -> Value {
    json!({
        "small": 7, "int": 42, "big": 9_000_000_000_i64, "unsigned": 18446744073709551615_u64,
        "real": 1.5, "double": 2.25, "flag": true, "text": "hello",
        "id": "67e55044-10b1-426f-9247-bb680e5fe0c8",
        "at": "2024-05-06T07:08:09", "day": "2024-05-06", "time": "07:08:09",
        "stamp": "2024-05-06T07:08:09+00:00", "bytes": "\\x0102",
        "raw": {"k": [1, 2]}, "missing": null, "inner": {"level": 3}
    })
}

#[fixture]
fn conn() -> PgConnection {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut conn = PgConnection::establish(&url).expect("connect to postgres");
    conn.begin_test_transaction().expect("test transaction");
    diesel::sql_query(
        "CREATE TEMP TABLE djp_documents (id SERIAL PRIMARY KEY, body JSONB NOT NULL)",
    )
    .execute(&mut conn)
    .expect("create table");
    diesel::insert_into(djp_documents::table)
        .values(djp_documents::body.eq(document()))
        .execute(&mut conn)
        .expect("insert row");
    conn
}

fn fetch<ST, T>(conn: &mut PgConnection, expr: SqlLiteral<ST>) -> T
where
    ST: SingleValue + 'static,
    Pg: HasSqlType<ST>,
    T: FromSqlRow<ST, Pg> + 'static,
{
    djp_documents::table
        .select(expr)
        .first(conn)
        .expect("query")
}

#[rstest]
#[ignore = "needs DATABASE_URL"]
fn integers_floats_and_booleans(mut conn: PgConnection) {
    assert_eq!(fetch::<_, i16>(&mut conn, Doc::small_sql()), 7);
    assert_eq!(fetch::<_, i32>(&mut conn, Doc::int_sql()), 42);
    assert_eq!(fetch::<_, i64>(&mut conn, Doc::big_sql()), 9_000_000_000);
    assert_eq!(fetch::<_, f32>(&mut conn, Doc::real_sql()), 1.5);
    assert_eq!(fetch::<_, f64>(&mut conn, Doc::double_sql()), 2.25);
    assert!(fetch::<_, bool>(&mut conn, Doc::flag_sql()));
}

#[rstest]
#[ignore = "needs DATABASE_URL"]
fn text_numeric_and_uuid(mut conn: PgConnection) {
    assert_eq!(fetch::<_, String>(&mut conn, Doc::text_sql()), "hello");
    let unsigned = BigDecimal::from_str("18446744073709551615").unwrap();
    assert_eq!(
        fetch::<_, BigDecimal>(&mut conn, Doc::unsigned_sql()),
        unsigned
    );
    let id = Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap();
    assert_eq!(fetch::<_, Uuid>(&mut conn, Doc::id_sql()), id);
}

#[rstest]
#[ignore = "needs DATABASE_URL"]
fn dates_times_and_bytes(mut conn: PgConnection) {
    let day = NaiveDate::from_ymd_opt(2024, 5, 6).unwrap();
    let time = NaiveTime::from_hms_opt(7, 8, 9).unwrap();
    assert_eq!(fetch::<_, NaiveDate>(&mut conn, Doc::day_sql()), day);
    assert_eq!(fetch::<_, NaiveTime>(&mut conn, Doc::time_sql()), time);
    assert_eq!(
        fetch::<_, NaiveDateTime>(&mut conn, Doc::at_sql()),
        day.and_time(time)
    );
    let stamp = fetch::<_, DateTime<Utc>>(&mut conn, Doc::stamp_sql());
    assert_eq!(stamp.naive_utc(), day.and_time(time));
    assert_eq!(fetch::<_, Vec<u8>>(&mut conn, Doc::bytes_sql()), vec![1, 2]);
}

#[rstest]
#[ignore = "needs DATABASE_URL"]
fn json_optional_and_nested(mut conn: PgConnection) {
    assert_eq!(
        fetch::<_, Value>(&mut conn, Doc::raw_sql()),
        json!({"k": [1, 2]})
    );
    assert_eq!(fetch::<_, Option<i32>>(&mut conn, Doc::missing_sql()), None);
    assert_eq!(fetch::<_, i32>(&mut conn, Doc::inner().level_sql()), 3);
}
