use std::{sync::Arc, time::Duration};

use libp2p::{
    self, PeerId, StreamProtocol, identify,
    identity::Keypair,
    kad::{self, store::MemoryStore},
    mdns::{self, Event},
    ping,
    swarm::{ConnectionId, NetworkBehaviour},
};

use super::database::LocalDatabase;

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "NetEvent")]
pub struct NetabaseBehaviour {
    pub kademlia: kad::Behaviour<LocalDatabase>,
    pub mdns: mdns::tokio::Behaviour,
    pub ping: ping::Behaviour,
}

#[derive(Clone, Debug)]
pub enum NetEvent {
    Ping {
        peer_id: PeerId,
        connection: ConnectionId,
        result: Option<Duration>,
    },
    Kad(kad::Event),
    Mdns(mdns::Event),
}

impl From<ping::Event> for NetEvent {
    fn from(value: ping::Event) -> Self {
        Self::Ping {
            peer_id: value.peer,
            connection: value.connection,
            result: value.result.ok(),
        }
    }
}

impl From<kad::Event> for NetEvent {
    fn from(value: kad::Event) -> Self {
        Self::Kad(value)
    }
}

impl From<mdns::Event> for NetEvent {
    fn from(value: mdns::Event) -> Self {
        Self::Mdns(value)
    }
}

impl NetabaseBehaviour {
    pub fn new(key: &Keypair) -> Self {
        let kademlia_config = kad::Config::new(StreamProtocol::new("/newsnet"));
        let mdns_config = mdns::Config::default();
        let identify_config = identify::Config::new("1.0.0".to_string(), key.public());

        let identify = identify::Behaviour::new(identify_config);

        let kademlia = kad::Behaviour::with_config(
            key.public().to_peer_id(),
            LocalDatabase::new(true, 100),
            // MemoryStore::new(key.public().to_peer_id()),
            kademlia_config,
        );
        let mdns = mdns::Behaviour::new(mdns_config, key.public().to_peer_id())
            .expect("Failed to create mdns");
        let ping = ping::Behaviour::default();
        Self {
            kademlia,
            mdns,
            ping,
        }
    }
}
