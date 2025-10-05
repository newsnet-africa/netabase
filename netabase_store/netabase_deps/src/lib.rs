//! # Netabase Dependencies
//!
//! This crate provides hygienic re-exports of all dependencies used by the Netabase macros.
//! It ensures that procedural macros can reference external dependencies without requiring
//! users to manually import them.
//!
//! This crate is an implementation detail and should not be used directly by end users.

/// Re-exports of all dependencies used by generated macro code.
/// This ensures macro hygiene - users don't need to manually import these dependencies.
pub mod __private {
    //! Private re-exports for macro hygiene. Do not use directly.
    //!
    //! All Netabase macros automatically use these dependencies, making them completely
    //! hygienic. You can use `#[derive(NetabaseModel)]` without any manual imports!

    /// Serialization library for binary encoding/decoding
    pub use bincode;

    /// Derive macro utilities
    pub use derive_more;

    /// Serialization framework
    pub use serde;

    /// Embedded database
    pub use sled;

    /// Enum utilities and derive macros
    pub use strum;

    /// Standard library re-exports
    pub use std;
}

// Also provide direct access for convenience
pub use bincode;
pub use derive_more;
pub use serde;
pub use sled;
pub use strum;
