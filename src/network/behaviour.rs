use libp2p::{PeerId, identify, identity::Keypair, kad, mdns, swarm::NetworkBehaviour};

use crate::{config::BehaviourConfig, database::SledStore};

#[derive(NetworkBehaviour)]
pub struct NetabaseBehaviour {
    pub kad: kad::Behaviour<SledStore>,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
}

impl NetabaseBehaviour {
    /// Create a new NetabaseBehaviour with custom configuration
    pub fn new(key: &Keypair, config: &BehaviourConfig) -> anyhow::Result<Self> {
        let peer_id = PeerId::from_public_key(&key.public());
        // Create Kademlia behaviour with optional custom config
        let kad = if let Some(kad_config) = &config.kad_config() {
            kad::Behaviour::with_config(
                peer_id,
                SledStore::new(peer_id, &config.database_path())?,
                kad_config.clone(),
            )
        } else {
            kad::Behaviour::new(peer_id, SledStore::new(peer_id, &config.database_path())?)
        };

        // Create mDNS behaviour with optional custom config
        let mdns = if let Some(mdns_config) = &config.mdns_config() {
            mdns::Behaviour::new(mdns_config.clone(), peer_id)?
        } else {
            mdns::Behaviour::new(mdns::Config::default(), peer_id)?
        };

        // Create Identify behaviour with optional custom config
        let identify = if let Some(identify_config) = &config.identify_config() {
            identify::Behaviour::new(identify_config.clone())
        } else {
            identify::Behaviour::new(
                identify::Config::new(config.protocol_version().to_string(), key.public())
                    .with_agent_version(config.agent_version().to_string()),
            )
        };

        Ok(Self {
            kad,
            mdns,
            identify,
        })
    }

    /// Create a new NetabaseBehaviour for testing (backward compatibility)
    pub fn new_test(key: &Keypair, test_number: usize) -> Self {
        let config = BehaviourConfig::builder()
            .database_path(format!("./test/database{test_number}"))
            .protocol_version("/p2p/newsnet/0.0.0".to_string())
            .build()
            .expect("Default BehaviourConfig should be valid");

        Self::new(key, &config).expect("Test behaviour creation should succeed")
    }
}
