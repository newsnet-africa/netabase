//! Unit Tests Module
//!
//! This module contains unit tests for individual components of the Netabase system.
//! These tests focus on testing isolated functionality and individual methods/functions
//! without requiring complex setup or external dependencies.

pub mod handler_tests;

// Re-export test functions for external access
pub use handler_tests::*;
