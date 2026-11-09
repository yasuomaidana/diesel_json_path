#![doc = include_str!("../README.md")]

#[doc(inline)]
pub use diesel_json_path_derive::SqlFields;

// Re-export diesel items so the macro has access to them
pub mod exports {
    pub use diesel;
    pub use serde_json;
}

/// Returns the version of this crate.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_matches_the_package() {
        assert_eq!(super::version(), env!("CARGO_PKG_VERSION"));
    }
}
