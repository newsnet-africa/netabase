//! Node metadata types for P2P networking.
//!
//! This module defines types for representing node identity and subscription
//! information in the Netabase peer-to-peer network.
//!
//! # Key Types
//!
//! - `NodeMetadata`: Complete metadata for a network node
//! - `PublicNodeData`: Public identity information (node ID, public key)
//! - `NodePublicKey`: Ed25519 public key for a node
//! - `SubscriptionRoom`: A topic/room the node is subscribed to
//!
//! # Feature Flag
//!
//! Most types in this module require the `libp2p` feature to be enabled.

#[cfg(feature = "libp2p")]
use libp2p::PeerId;
use serde::{Serialize, Deserialize};

use netabase_store::traits::registry::definition::NetworkDefinition;

#[cfg(feature = "libp2p")]
pub struct NodeMetadata<D: NetworkDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    node_pub: PublicNodeData,
    rooms: Vec<SubscriptionRoom<D>>,
}

#[cfg(feature = "libp2p")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicNodeData {
    pub node_id: PeerId,
    pub public_key: NodePublicKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodePublicKey(pub [u8; 32]);

pub struct SubscriptionRoomKey;

#[cfg(feature = "libp2p")]
pub struct SubscriptionRoom<D: NetworkDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    pub subscription: D::SubscriptionKeysDiscriminant,
    pub room_key: Option<SubscriptionRoomKey>,
    pub root_node: PublicNodeData,
    pub capabilities: D::DefinitionCapabilities,
}
