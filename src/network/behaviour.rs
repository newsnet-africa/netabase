use std::path::Path;

use libp2p::{gossipsub::IdentityTransform, identity::Keypair, swarm::NetworkBehaviour, *};
use sled::Error;

use crate::database::SledStore;

/// Cloneable version of libp2p::identify::Event
#[derive(Clone, Debug)]
pub enum CloneableIdentifyEvent {
    /// Identification information has been received from a peer.
    Received {
        /// Identifier of the connection.
        connection_id: libp2p::swarm::ConnectionId,
        /// The peer that has been identified.
        peer_id: libp2p::PeerId,
        /// The information provided by the peer.
        info: libp2p::identify::Info,
    },
    /// Identification information of the local node has been sent to a peer
    /// in response to an identification request.
    Sent {
        /// Identifier of the connection.
        connection_id: libp2p::swarm::ConnectionId,
        /// The peer that the information has been sent to.
        peer_id: libp2p::PeerId,
    },
    /// Identification information of the local node has been actively pushed to a peer.
    Pushed {
        /// Identifier of the connection.
        connection_id: libp2p::swarm::ConnectionId,
        /// The peer that the information has been sent to.
        peer_id: libp2p::PeerId,
        /// The full Info struct we pushed to the remote peer.
        info: libp2p::identify::Info,
    },
    /// Error while attempting to identify the remote.
    Error {
        /// Identifier of the connection.
        connection_id: libp2p::swarm::ConnectionId,
        /// The peer with whom the error originated.
        peer_id: libp2p::PeerId,
        /// String representation of the error that occurred.
        error: String,
    },
}

impl From<libp2p::identify::Event> for CloneableIdentifyEvent {
    fn from(event: libp2p::identify::Event) -> Self {
        match event {
            libp2p::identify::Event::Received {
                connection_id,
                peer_id,
                info,
            } => CloneableIdentifyEvent::Received {
                connection_id,
                peer_id,
                info,
            },
            libp2p::identify::Event::Sent {
                connection_id,
                peer_id,
            } => CloneableIdentifyEvent::Sent {
                connection_id,
                peer_id,
            },
            libp2p::identify::Event::Pushed {
                connection_id,
                peer_id,
                info,
            } => CloneableIdentifyEvent::Pushed {
                connection_id,
                peer_id,
                info,
            },
            libp2p::identify::Event::Error {
                connection_id,
                peer_id,
                error,
            } => CloneableIdentifyEvent::Error {
                connection_id,
                peer_id,
                error: format!("{:?}", error),
            },
        }
    }
}

#[derive(NetworkBehaviour)]
pub struct NetabaseBehaviour {
    kad: kad::Behaviour<SledStore>,
    identify: identify::Behaviour,
}

impl NetabaseBehaviour {
    pub fn new<P: AsRef<Path>>(storage_path: P, keypair: &Keypair) -> anyhow::Result<Self> {
        let peer_id = keypair.public().to_peer_id();
        let protocol = "NetabaseDummy".to_string(); //TODO: Obviously check to see if the protocol works the same everywhere and if I can use the same protocol for everything.
        let kad = kad::Behaviour::new(peer_id, SledStore::new(peer_id, storage_path)?);
        let identify = identify::Behaviour::new(identify::Config::new(protocol, keypair.public()));
        Ok(Self { kad, identify })
    }
}
