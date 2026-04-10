# diesel_json_derive
Procedural macro crate for [`diesel_json_path`](https://docs.rs/diesel_json_path).
Provides the `#[derive(SqlFields)]` macro that generates type-safe Diesel SQL
expressions for accessing fields inside PostgreSQL JSONB columns.
## Usage
Add `diesel_json_path` (the user-facing crate) to your `Cargo.toml` — it
re-exports this macro along with all required Diesel and serde_json types.
See the [`diesel_json_path` documentation](https://docs.rs/diesel_json_path)
for full usage examples and the complete type-mapping reference.
