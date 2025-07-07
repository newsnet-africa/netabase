use std::path::Path;

use libp2p::{gossipsub::IdentityTransform, identity::Keypair, swarm::NetworkBehaviour, *};
use sled::Error;

use crate::database::SledStore;

#[derive(NetworkBehaviour)]
pub struct NetabaseBehaviour {
    kad: kad::Behaviour<SledStore>,
    identify: identify::Behaviour,
}

impl NetabaseBehaviour {
    pub fn new<P: AsRef<Path>>(storage_path: P, keypair: &Keypair) -> anyhow::Result<Self> {
        let peer_id = keypair.public().to_peer_id();
        let protocol = "NetabaseDummy".to_string(); //TODO: Obviosuly check to see if the protocol works the same everywhere and if I can use the same protocol for everything.
        let kad = kad::Behaviour::new(peer_id, SledStore::new(peer_id, storage_path)?);
        let identify = identify::Behaviour::new(identify::Config::new(protocol, keypair.public()));
        Ok(Self { kad, identify })
    }
}
