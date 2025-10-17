//! Kademlia DHT Tests Module
//!
//! This module contains tests specific to Kademlia DHT functionality including
//! peer discovery, data sharing, provider functionality, and interprocess
//! communication. These tests verify the core DHT operations and network
//! behavior of the Netabase system.

pub mod kademlia_interprocess_messaging;
pub mod kademlia_memory_test;

// Re-export test functions for external access
pub use kademlia_interprocess_messaging::*;
pub use kademlia_memory_test::*;
