use std::path::Path;

use libp2p::{gossipsub::IdentityTransform, identity::Keypair, swarm::NetworkBehaviour, *};
use sled::Error;

use crate::database::SledStore;

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "NetabaseBehaviourEvent")]
pub struct NetabaseBehaviour {
    kad: kad::Behaviour<SledStore>,
    identify: identify::Behaviour,
}

#[derive(Clone, Debug)]
pub enum NetabaseBehaviourEvent
where
    kad::Behaviour<SledStore>: ::libp2p::swarm::derive_prelude::NetworkBehaviour,
    identify::Behaviour: ::libp2p::swarm::derive_prelude::NetworkBehaviour,
{
    Kad(<kad::Behaviour<SledStore> as ::libp2p::swarm::derive_prelude::NetworkBehaviour>::ToSwarm),
    Identify(<identify::Behaviour as ::libp2p::swarm::derive_prelude::NetworkBehaviour>::ToSwarm),
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

impl From<identify::Event> for NetabaseBehaviourEvent {
    fn from(event: identify::Event) -> Self {
        Self::Identify(event)
    }
}

impl From<kad::Event> for NetabaseBehaviourEvent {
    fn from(event: kad::Event) -> Self {
        Self::Kad(event)
    }
}
