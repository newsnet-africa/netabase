//! Node management module
//!
//! The NetabaseNode represents a single participant in the network,
//! managing subscriptions, connections, and the handshake lifecycle.

use std::collections::HashMap;

use libp2p::PeerId;

use crate::data::{
    network::handshake::{
        keys::StaticKeyPair,
        state::{EstablishedConnection, HandshakeState},
    },
    store::network::NetworkDefinition,
};

use self::metadata::{NodePrivateMetadata, NodePublicMetadata, PeerConnection};
use self::subscription::SubscriptionRoom;

pub mod metadata;
pub mod subscription;

/// A netabase node
pub struct NetabaseNode<D: NetworkDefinition>
where
    D::Discriminant: std::fmt::Debug + 'static,
{
    /// Public metadata (can be shared with peers)
    pub public_metadata: NodePublicMetadata,
    /// Private metadata (never transmitted)
    private_metadata: NodePrivateMetadata,
    /// Subscriptions this node participates in
    rooms: Vec<SubscriptionRoom<D>>,
    /// Active connections to peers
    connections: HashMap<PeerId, PeerConnection>,
    /// Pending handshakes
    pending_handshakes: HashMap<PeerId, HandshakeState>,
}

impl<D: NetworkDefinition> NetabaseNode<D>
where
    D::Discriminant: std::fmt::Debug + 'static + Clone,
{
    /// Create a new node with generated keys
    pub fn new(peer_id: PeerId) -> Self {
        let static_keys = StaticKeyPair::generate();
        let public_metadata = NodePublicMetadata::new(peer_id, static_keys.public_key().clone());
        let private_metadata = NodePrivateMetadata::new(static_keys);

        Self {
            public_metadata,
            private_metadata,
            rooms: Vec::new(),
            connections: HashMap::new(),
            pending_handshakes: HashMap::new(),
        }
    }

    /// Create a node with existing keys
    pub fn with_keys(peer_id: PeerId, static_keys: StaticKeyPair) -> Self {
        let public_metadata = NodePublicMetadata::new(peer_id, static_keys.public_key().clone());
        let private_metadata = NodePrivateMetadata::new(static_keys);

        Self {
            public_metadata,
            private_metadata,
            rooms: Vec::new(),
            connections: HashMap::new(),
            pending_handshakes: HashMap::new(),
        }
    }

    /// Get our peer ID
    pub fn peer_id(&self) -> &PeerId {
        &self.public_metadata.peer_id
    }

    /// Add a subscription room
    pub fn add_subscription(&mut self, room: SubscriptionRoom<D>) {
        // Add to subscription interests
        self.private_metadata.add_interest(metadata::SubscriptionInterest {
            subscription_hash: room.interest_hash(),
            table_hashes: room.table_hashes(),
            has_write_capability: room.is_owned(),
        });

        // Update public commitment
        self.public_metadata
            .subscription_commitments
            .push(room.interest_hash());

        self.rooms.push(room);
    }

    /// Get subscription rooms
    pub fn subscriptions(&self) -> &[SubscriptionRoom<D>] {
        &self.rooms
    }

    /// Initiate a handshake with a peer
    pub fn initiate_handshake(&mut self, peer_id: PeerId) -> HandshakeState {
        let state = HandshakeState::new_initiator(
            self.private_metadata.static_keys.clone(),
            self.private_metadata.interest_hashes(),
        );
        
        self.pending_handshakes.insert(peer_id.clone(), state.clone());
        state
    }

    /// Accept a handshake from a peer
    pub fn accept_handshake(&mut self, peer_id: PeerId) -> HandshakeState {
        let state = HandshakeState::new_responder(
            self.private_metadata.static_keys.clone(),
            self.private_metadata.interest_hashes(),
        );

        self.pending_handshakes.insert(peer_id.clone(), state.clone());
        state
    }

    /// Get pending handshake for a peer
    pub fn get_handshake(&mut self, peer_id: &PeerId) -> Option<&mut HandshakeState> {
        self.pending_handshakes.get_mut(peer_id)
    }

    /// Complete a handshake and establish connection
    pub fn complete_handshake(
        &mut self,
        peer_id: PeerId,
        connection: EstablishedConnection,
    ) {
        self.pending_handshakes.remove(&peer_id);
        self.connections.insert(peer_id, PeerConnection::new(connection));
    }

    /// Get an established connection
    pub fn get_connection(&self, peer_id: &PeerId) -> Option<&PeerConnection> {
        self.connections.get(peer_id)
    }

    /// Get mutable reference to connection
    pub fn get_connection_mut(&mut self, peer_id: &PeerId) -> Option<&mut PeerConnection> {
        self.connections.get_mut(peer_id)
    }

    /// Check if we have an active connection to a peer
    pub fn is_connected(&self, peer_id: &PeerId) -> bool {
        self.connections
            .get(peer_id)
            .map(|c| c.is_connected())
            .unwrap_or(false)
    }

    /// Get all connected peers
    pub fn connected_peers(&self) -> impl Iterator<Item = &PeerId> {
        self.connections
            .iter()
            .filter(|(_, c)| c.is_connected())
            .map(|(p, _)| p)
    }

    /// Disconnect from a peer
    pub fn disconnect(&mut self, peer_id: &PeerId) {
        self.connections.remove(peer_id);
        self.pending_handshakes.remove(peer_id);
    }
}
