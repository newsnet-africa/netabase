// pub mod record_store;  // Commented out due to API incompatibility with current traits
pub mod sled;
pub mod wrappers;

// Re-export the enhanced database as the primary implementation
pub use sled::{NetabaseIter, NetabaseSledDatabase, NetabaseSledTree, NetabaseTreeCompatible};
