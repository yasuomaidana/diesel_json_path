#![doc = include_str!("../README.md")]

#[doc(inline)]
pub use diesel_json_path_derive::SqlFields;

// Re-export diesel items so the macro has access to them
pub mod exports {
    pub use diesel;
    pub use serde_json;

}