use libp2p::{
    self, StreamProtocol,
    identity::Keypair,
    kad::{self, store::MemoryStore},
    mdns, ping,
    swarm::NetworkBehaviour,
};

#[derive(NetworkBehaviour)]
pub struct NetabaseBehaviour {
    pub event_kad: kad::Behaviour<MemoryStore>,
    pub mentions_kad: kad::Behaviour<MemoryStore>,
    pub gkg_kad: kad::Behaviour<MemoryStore>,
    pub mdns: mdns::tokio::Behaviour,
    pub ping: ping::Behaviour,
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
            ping,
        }
    }
}
