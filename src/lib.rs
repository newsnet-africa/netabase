#![feature(impl_trait_in_assoc_type)]

//! # Netabase
//!
//! A distributed database built on top of sled with libp2p integration.
//!
//! This crate re-exports the core functionality from `netabase_store`.

pub use netabase_store::*;
pub mod network;
