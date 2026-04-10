# Diesel JSON Path

A Rust crate that provides a procedural macro to generate strongly-typed SQL expressions for accessing nested JSON fields in PostgreSQL using Diesel ORM.

## Overview

`diesel_json_path` simplifies working with PostgreSQL's JSONB columns when using Diesel. Instead of manually constructing JSON path expressions as raw SQL strings, this crate provides a derive macro that generates type-safe, compile-checked accessors for your JSON structures.

### Key Features

- **Type-Safe JSON Access**: Generate compile-time checked SQL expressions for JSON field access
- **Nested JSON Support**: Seamlessly navigate nested JSON structures with builder-pattern methods
- **Automatic Type Casting**: Primitive types are automatically cast to appropriate PostgreSQL types (e.g., `i32` → `int`, `String` → `text`)
- **Option Support**: Optional fields generate expressions that wrap results in `Nullable` SQL types
- **Zero Runtime Overhead**: All SQL generation happens at compile time
- **PostgreSQL Native**: Leverages PostgreSQL's native JSON operators (`->`, `->>`)

## Components

### `diesel_json_derive`

A procedural macro crate that provides the `SqlFields` derive macro. This is where the code generation happens.

### `diesel_json_path`

The main crate that re-exports the `SqlFields` derive macro and provides convenient access to Diesel and serde_json types.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
diesel = { version = "2.3", features = ["postgres"] }
diesel_json_path = "0.1"
```

## Usage

### Basic Example

Define your JSON structure using the `#[derive(SqlFields)]` macro:

```rust,ignore
use diesel_json_path::SqlFields;
use diesel::prelude::*;

table! {
    users (id) {
        id -> Int4,
        metadata -> Jsonb,
    }
}

#[derive(SqlFields)]
#[diesel_json(column = "metadata")]
struct Settings {
    theme: String,
}

#[derive(SqlFields)]
#[diesel_json(column = "metadata")]
struct UserProfile {
    id: i32,
    settings: Option<Settings>,
}

// Generate SQL expressions
fn get_user_theme(conn: &mut PgConnection) -> QueryResult<Vec<Option<String>>> {
    users::table
        .select(UserProfile::settings().theme_sql())
        .load::<Option<String>>(conn)
}
```

### Macro Attributes

#### `#[diesel_json(column = "...")]`

Specifies the root JSON column name. Defaults to `"body"` if not specified.

```rust,ignore
#[derive(SqlFields)]
#[diesel_json(column = "metadata")]
struct MyStruct {
    field: String,
}
```

#### `#[json_path("...")]`

Specifies a custom JSON path for a field. If omitted, the field name is used as the path.

```rust,ignore
#[derive(SqlFields)]
#[diesel_json(column = "data")]
struct Config {
    #[json_path("settings.theme")]
    theme: Option<String>,
}
```

### Generated Methods

For each field in your struct, the macro generates:

1. **Builder Method** (instance method): Returns a path builder for navigation
   ```rust,ignore
   pub fn settings(&self) -> SettingsPathBuilder {
       SettingsPathBuilder { base_path: "metadata->'settings'".to_string() }
   }
   ```

2. **SQL Method** (instance method): Returns a SQL expression for primitive types
   ```rust,ignore
   pub fn theme_sql(&self) -> diesel::expression::SqlLiteral<Nullable<Text>> {
       // Generates: (metadata->'settings'->>'theme')
   }
   ```

3. **Static Shortcut** (static method): Convenience method bypassing builder
   ```rust,ignore
   pub fn theme_sql() -> diesel::expression::SqlLiteral<Nullable<Text>> {
       // Directly accessible without building
   }
   ```

### Type Mapping

The macro automatically maps Rust types to PostgreSQL types with appropriate casting:

| Rust Type | PostgreSQL Type | SQL Cast |
|-----------|-----------------|----------|
| `i8`, `i16`, `u8` | SmallInt | `::smallint` |
| `i32`, `u16` | Integer | `::int` |
| `i64`, `u32` | BigInt | `::bigint` |
| `u64` | Numeric | `::numeric` |
| `f32` | Float | `::real` |
| `f64` | Double | `::double precision` |
| `bool` | Bool | `::boolean` |
| `String`, `str` | Text | (no cast) |
| `Uuid` | Uuid | `::uuid` |
| `NaiveDateTime` | Timestamp | `::timestamp` |
| `NaiveDate` | Date | `::date` |
| `NaiveTime` | Time | `::time` |
| `DateTime<Utc>` | Timestamptz | `::timestamptz` |
| `Decimal` | Numeric | `::numeric` |
| `Vec<u8>` | Binary | `::bytea` |
| `serde_json::Value`, `Jsonb` | Jsonb | (no cast) |

### Nullable Fields

When a field is wrapped in `Option<T>`, the generated SQL expression automatically uses `Nullable<T>`:

```rust,ignore
#[derive(SqlFields)]
#[diesel_json(column = "data")]
struct Profile {
    age: Option<i32>,  // Generates Nullable<Integer>
}
```

## Generated SQL Examples

Given this structure:

```rust,ignore
#[derive(SqlFields)]
#[diesel_json(column = "metadata")]
struct UserProfile {
    id: i32,
    settings: Option<Settings>,
}

#[derive(SqlFields)]
#[diesel_json(column = "metadata")]
struct Settings {
    theme: String,
}
```

The macro generates these SQL expressions:

- `UserProfile::id_sql()` → `(metadata->>'id')::int`
- `UserProfile::settings().theme_sql()` → `(metadata->'settings'->>'theme')`

## Advanced Usage

### Nested Structures

Navigate deeply nested JSON with type safety:

```rust,ignore
#[derive(SqlFields)]
#[diesel_json(column = "config")]
struct AppConfig {
    database: Option<DatabaseConfig>,
}

#[derive(SqlFields)]
#[diesel_json(column = "config")]
struct DatabaseConfig {
    host: String,
    port: Option<i32>,
}

// Build complex paths
let host_expr = AppConfig::database().host_sql();
let port_expr = AppConfig::database().port_sql();
```

### Custom JSON Paths

Override the default field-based path:

```rust,ignore
#[derive(SqlFields)]
#[diesel_json(column = "data")]
struct User {
    #[json_path("profile.info.name")]
    full_name: String,
    
    #[json_path("settings.ui.theme")]
    theme: String,
}
```

## Testing

Run the test suite:

```bash
cargo test
```

The tests are located in:
- `diesel_json_derive/tests/macro_tests.rs` - Macro generation tests
- `diesel_json_path/tests/macro_tests.rs` - Integration tests

## Design Notes

### Compilation Strategy

- The macro parses struct definitions at compile time
- Generates SQL construction code as proc-macro output
- All type checking happens via Diesel's type system at compile time
- Zero runtime overhead for path construction

### SQL Generation

- Uses PostgreSQL's native JSON operators:
  - `->` for JSON extraction (returns JSON)
  - `->>` for text extraction (returns text)
- Automatically applies type casting for primitive types
- Properly formats JSON paths for nested access

### Type Safety

The macro leverages Diesel's type system to ensure:
- Expressions have correct `SqlType`
- Only compatible types can be selected
- Compilation fails early for type mismatches

## Limitations

- Currently PostgreSQL-specific (uses `->`, `->>`, and PostgreSQL type casting)
- Requires explicit struct definition; cannot introspect runtime JSON
- Fields must map to concrete types (no arbitrary JSON values)
- Limited to single JSON column per struct (though you can define multiple structs for different columns)

## Contributing

Contributions are welcome! Please ensure:
- All tests pass: `cargo test`
- Code is formatted: `cargo fmt`
- No clippy warnings: `cargo clippy`

## License

Licensed under the MIT License. See LICENSE file for details.

## Related Projects

- [Diesel](https://diesel.rs/) - A safe, extensible ORM for Rust
- [PostgreSQL JSON Functions](https://www.postgresql.org/docs/current/functions-json.html) - PostgreSQL JSON documentation
- [serde_json](https://docs.rs/serde_json/) - JSON serialization/deserialization

## Examples

See the test files for complete working examples:
- Basic field access
- Nested structure navigation
- Multiple JSON columns
- Type casting and nullability

## Troubleshooting

### "Cannot find macro in module"

Ensure you've imported the macro:
```rust,ignore
use diesel_json_path::SqlFields;
```

### Type mismatch errors

Verify that:
1. The field type matches the actual PostgreSQL type in your JSON
2. `Option<T>` is used for nullable fields
3. Custom `#[json_path]` points to the correct JSON structure

### SQL not generating expected cast

Check the type mapping table above. If your type isn't listed, you may need to use a generic `serde_json::Value` or submit a feature request.

## Roadmap

Potential future improvements:
- Support for other databases (MySQL, SQLite)
- Validation of JSON paths at compile time
- Builder pattern for query construction
- Automatic deserialization helpers
