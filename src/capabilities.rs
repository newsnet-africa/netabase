//! Capability management for network-based model access control.
//!
//! This module provides types for managing capabilities in distributed systems,
//! enabling fine-grained access control for models across peers.
//!
//! # Overview
//!
//! The [`Capability`] type is a generic, type-safe capability that ties access
//! rights to specific model types within a definition. This enables compile-time
//! verification that capabilities are used with the correct models.
//!
//! For the underlying Meadowcap capability system (delegation, areas, private
//! interest intersection), see [`crate::data::network::capability`].
//!
//! # Example
//!
//! ```rust
//! use netabase::capabilities::AccessMode;
//!
//! // AccessMode determines what operations are allowed
//! let read_only = AccessMode::Read;
//! let read_write = AccessMode::ReadWrite;
//!
//! // Check what each mode allows using pattern matching
//! assert!(matches!(read_only, AccessMode::Read));
//! assert!(matches!(read_write, AccessMode::ReadWrite));
//!
//! // Or check equality
//! assert_eq!(read_only, AccessMode::Read);
//! assert_ne!(read_only, AccessMode::ReadWrite);
//! ```
//!
//! For typed `Capability<D, M>` usage with `can_read()` and `can_write()` methods,
//! see the `example` crate's tests which demonstrate capabilities with concrete
//! definition and model types.

use crate::data::store::network::NetworkDefinition;
use serde::{Deserialize, Serialize};

/// A capability grants access rights to a specific model type within a definition.
///
/// Capabilities can be shared, delegated, and verified across peers in a
/// peer-to-peer network.
///
/// # Type Parameters
///
/// - `D`: The network definition that contains the model
/// - `M`: The specific model type this capability grants access to
///
/// # Usage
///
/// This type requires a full `NetworkDefinition` and `NetabaseModel` to be instantiated.
/// See the `example` crate's `networking_capabilities.rs` tests for concrete usage:
///
/// ```text
/// // With a concrete definition and model:
/// let cap = Capability::<MyDefinition, User>::new_read();
/// assert!(cap.can_read());
/// assert!(!cap.can_write());
///
/// let rw_cap = Capability::<MyDefinition, User>::new_read_write();
/// assert!(rw_cap.can_read());
/// assert!(rw_cap.can_write());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capability<D: NetworkDefinition, M>
where
    <D as netabase_store::strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    M: netabase_store::prelude::NetabaseModel<D>,
    M::Keys: std::fmt::Debug + Clone + Eq,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Secondary as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Relational as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Blob as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Libp2p as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// The access mode for this capability
    pub mode: AccessMode,
    
    /// Marker to tie this capability to specific types
    #[serde(skip)]
    _phantom: std::marker::PhantomData<(D, M)>,
}

/// Access mode for capabilities
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccessMode {
    /// Read-only access
    Read,
    /// Read and write access
    ReadWrite,
}

impl<D: NetworkDefinition, M> Capability<D, M>
where
    <D as netabase_store::strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    M: netabase_store::prelude::NetabaseModel<D>,
    M::Keys: std::fmt::Debug + Clone + Eq,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Secondary as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Relational as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Blob as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Libp2p as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// Create a new read-only capability
    pub fn new_read() -> Self {
        Self {
            mode: AccessMode::Read,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Create a new read-write capability
    pub fn new_read_write() -> Self {
        Self {
            mode: AccessMode::ReadWrite,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Check if this capability allows reads
    pub fn can_read(&self) -> bool {
        matches!(self.mode, AccessMode::Read | AccessMode::ReadWrite)
    }
    
    /// Check if this capability allows writes
    pub fn can_write(&self) -> bool {
        matches!(self.mode, AccessMode::ReadWrite)
    }
}

impl<D: NetworkDefinition, M> Default for Capability<D, M>
where
    <D as netabase_store::strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    M: netabase_store::prelude::NetabaseModel<D>,
    M::Keys: std::fmt::Debug + Clone + Eq,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Secondary as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Relational as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Blob as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
    <<<M as netabase_store::prelude::NetabaseModel<D>>::Keys as netabase_store::traits::registry::models::NetabaseModelKeys<D, M>>::Libp2p as netabase_store::strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn default() -> Self {
        Self::new_read()
    }
}
