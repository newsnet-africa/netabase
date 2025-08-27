use libp2p::{PeerId, identify, identity::Keypair, kad, mdns, swarm::NetworkBehaviour};

use crate::database::SledStore;

#[derive(NetworkBehaviour)]
pub struct NetabaseBehaviour {
    pub kad: kad::Behaviour<SledStore>,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
}

impl NetabaseBehaviour {
    pub fn new_test(key: &Keypair, test_number: usize) -> Self {
        let peer_id = PeerId::from_public_key(&key.public());
        let kad = kad::Behaviour::new(
            peer_id,
            SledStore::new(peer_id, format!("./test/database{test_number}")).expect("Sled erruh"),
        );
        let mdns = mdns::Behaviour::new(mdns::Config::default(), peer_id).expect("Mdns Erruh");
        let identify = identify::Behaviour::new(identify::Config::new(
            "/p2p/newsnet/0.0.0".to_string(),
            key.public(),
        ));

        Self {
            kad,
            mdns,
            identify,
        }
    }
}
