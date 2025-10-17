//! Integration Tests Module
//!
//! This module contains integration tests that verify the interaction between
//! multiple components of the Netabase system. These tests focus on end-to-end
//! functionality and component integration.

pub mod integration_tests;
pub mod multi_process_tests;

// Re-export test functions for external access
pub use integration_tests::*;
pub use multi_process_tests::*;
