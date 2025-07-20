use std::path::Path;

use libp2p::{gossipsub::IdentityTransform, identity::Keypair, swarm::NetworkBehaviour, *};
use sled::Error;

use crate::database::SledStore;

#[derive(NetworkBehaviour)]
pub struct NetabaseBehaviour {
    pub kad: kad::Behaviour<SledStore>,
    pub identify: identify::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

impl Clone for NetabaseBehaviourEvent {
    fn clone(&self) -> Self {
        match self {
            NetabaseBehaviourEvent::Kad(kad_event) => {
                // kad events implement Clone
                NetabaseBehaviourEvent::Kad(kad_event.clone())
            }
            NetabaseBehaviourEvent::Identify(identify_event) => {
                // Handle each identify event variant manually
                match identify_event {
                    libp2p::identify::Event::Received {
                        connection_id,
                        peer_id,
                        info,
                    } => NetabaseBehaviourEvent::Identify(libp2p::identify::Event::Received {
                        connection_id: connection_id.clone(),
                        peer_id: peer_id.clone(),
                        info: info.clone(),
                    }),
                    libp2p::identify::Event::Sent {
                        connection_id,
                        peer_id,
                    } => NetabaseBehaviourEvent::Identify(libp2p::identify::Event::Sent {
                        connection_id: connection_id.clone(),
                        peer_id: peer_id.clone(),
                    }),
                    libp2p::identify::Event::Pushed {
                        connection_id,
                        peer_id,
                        info,
                    } => NetabaseBehaviourEvent::Identify(libp2p::identify::Event::Pushed {
                        connection_id: connection_id.clone(),
                        peer_id: peer_id.clone(),
                        info: info.clone(),
                    }),
                    libp2p::identify::Event::Error {
                        connection_id,
                        peer_id,
                        error: _,
                    } => {
                        // For the Error case, we create a placeholder error since the original is not cloneable
                        NetabaseBehaviourEvent::Identify(libp2p::identify::Event::Error {
                            connection_id: connection_id.clone(),
                            peer_id: peer_id.clone(),
                            error: libp2p::swarm::StreamUpgradeError::Apply(
                                libp2p::identify::UpgradeError::StreamClosed,
                            ),
                        })
                    }
                }
            }
            NetabaseBehaviourEvent::Mdns(mdns) => NetabaseBehaviourEvent::Mdns(mdns.clone()),
        }
    }
}

impl NetabaseBehaviour {
    pub fn new<P: AsRef<Path>>(storage_path: P, keypair: &Keypair) -> anyhow::Result<Self> {
        let local_peer_id = keypair.public().to_peer_id();
        let protocol = "NetabaseDummy".to_string(); //TODO: Obviously check to see if the protocol works the same everywhere and if I can use the same protocol for everything.
        let kad = kad::Behaviour::new(local_peer_id, SledStore::new(local_peer_id, storage_path)?);
        let identify = identify::Behaviour::new(identify::Config::new(protocol, keypair.public()));
        let mdns = mdns::Behaviour::new(mdns::Config::default(), local_peer_id)?;
        Ok(Self {
            kad,
            identify,
            mdns,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logging;
    use libp2p::identity::Keypair;

    #[test]
    fn test_netabase_behaviour_event_clone() {
        init_logging();
        let keypair = Keypair::generate_ed25519();
        let peer_id = keypair.public().to_peer_id();
        let connection_id = libp2p::swarm::ConnectionId::new_unchecked(1);

        // Test that we can clone the NetabaseBehaviourEvent with Identify variant
        let identify_event = NetabaseBehaviourEvent::Identify(libp2p::identify::Event::Sent {
            connection_id,
            peer_id,
        });

        let cloned = identify_event.clone();
        assert!(matches!(
            cloned,
            NetabaseBehaviourEvent::Identify(libp2p::identify::Event::Sent { .. })
        ));
    }
}
