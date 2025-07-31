//! Code generators for netabase macros
//!
//! This module contains code generation logic for various aspects of the netabase system.
//! Each submodule focuses on generating specific types of code:
//!
//! - `key_struct`: Generates key extraction and manipulation code
//! - `from_traits`: Generates conversion traits and implementations
//! - `trait_impls`: Generates NetabaseSchema and NetabaseSchemaKey trait implementations
//!
//! The generators work with the validated schema information from the visitors
//! to produce the final code that gets inserted into the user's crate.

pub mod from_traits;
pub mod key_struct;
pub mod trait_impls;

// Re-export commonly used generator functionality
pub use from_traits::*;
pub use key_struct::*;
pub use trait_impls::*;
