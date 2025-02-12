use libp2p::{
    self, StreamProtocol,
    identity::Keypair,
    kad::{self, store::MemoryStore},
    mdns::{self, Event},
    ping,
    swarm::NetworkBehaviour,
};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "NetEvent")]
pub struct NetabaseBehaviour {
    pub event_kad: kad::Behaviour<MemoryStore>,
    pub mentions_kad: kad::Behaviour<MemoryStore>,
    pub gkg_kad: kad::Behaviour<MemoryStore>,
    pub mdns: mdns::tokio::Behaviour,
}

#[derive(Clone, Debug)]
pub enum NetEvent {
    Test,
    Kad(kad::Event),
    Mdns(mdns::Event),
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
        let events_kad_config = kad::Config::new(StreamProtocol::new("/newsnet/events"));
        let mentions_kad_config = kad::Config::new(StreamProtocol::new("/newsnet/mentions"));
        let gkg_kad_config = kad::Config::new(StreamProtocol::new("/newsnet/gkg"));
        let mdns_config = mdns::Config::default();

        let event_kad = kad::Behaviour::with_config(
            key.public().to_peer_id(),
            MemoryStore::new(key.public().to_peer_id()),
            events_kad_config,
        );
        let mentions_kad = kad::Behaviour::with_config(
            key.public().to_peer_id(),
            MemoryStore::new(key.public().to_peer_id()),
            mentions_kad_config,
        );
        let gkg_kad = kad::Behaviour::with_config(
            key.public().to_peer_id(),
            MemoryStore::new(key.public().to_peer_id()),
            gkg_kad_config,
        );
        let mdns = mdns::Behaviour::new(mdns_config, key.public().to_peer_id())
            .expect("Failed to create mdns");
        let ping = ping::Behaviour::default();
        Self {
            event_kad,
            mentions_kad,
            gkg_kad,
            mdns,
        }
    }
}
