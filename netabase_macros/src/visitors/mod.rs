//! Visitors module for netabase macro processing
//!
//! This module contains the core logic for validating and finding schemas within Rust code.
//! It's organized with clear separation of concerns:
//!
//! - `schema_validator`: Validates individual schemas for compliance
//! - `key_finder`: Validates and extracts key information from schemas
//! - `schema_finder`: Finds valid schemas within modules using the validators
//! - `utils`: Common data structures and utilities

pub mod key_finder;
pub mod schema_finder;
pub mod schema_validator;
pub mod utils;

// Re-export commonly used types for convenience
// Note: These are used internally by the macro implementation
pub use key_finder::{KeyInfoBuilder, KeyValidator};
pub use schema_finder::{SchemaFinder, SchemaType};
pub use schema_validator::{SchemaValidator, ValidationError, ValidationResult};
pub use utils::{KeyInfo, KeyType, SchemaInfo, SchemaInfoBuilder};
