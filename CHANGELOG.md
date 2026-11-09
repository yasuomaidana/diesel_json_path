# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Quality workflow: CI, directives and pre-commit hook.
- trybuild compile-fail tests for invalid attributes.
- `diesel_json_path::version()`.

### Changed

- The derive is split into modules around a testable `expand()`.
- Invalid attributes and non-struct inputs now produce compile errors.
- Tests use `rstest`; coverage is 100% of lines.

## [0.1.0]

### Added

- Initial release: derive macro generating typed SQL expressions for nested JSON fields in PostgreSQL with Diesel.
