use libp2p::{
    PeerId, connection_limits, identify,
    identity::{Keypair, PublicKey},
    swarm::NetworkBehaviour,
};
use netabase_store::{
    database::NetabaseSledDatabase, errors::NetabaseError, traits::NetabaseSchema,
};
pub mod clone_impl;

#[derive(NetworkBehaviour)]
pub struct NetabaseBehaviour<S: NetabaseSchema> {
    pub kad: libp2p::kad::Behaviour<NetabaseSledDatabase<S>>,
    pub identify: libp2p::identify::Behaviour,
    pub mdns: libp2p::mdns::tokio::Behaviour,
    pub connection_limit: libp2p::connection_limits::Behaviour,
}

impl<S: NetabaseSchema> NetabaseBehaviour<S> {
    pub fn new(keypair: &Keypair) -> Result<Self, crate::errors::Error> {
        Self::new_with_name(keypair, None)
    }

    pub fn new_with_name(
        keypair: &Keypair,
        name: Option<String>,
    ) -> Result<Self, crate::errors::Error> {
        let pub_key = keypair.public();
        let peer_id = PeerId::from_public_key(&pub_key);

        let database = if let Some(name) = name {
            NetabaseSledDatabase::<S>::new_with_path(&name)?
        } else {
            NetabaseSledDatabase::<S>::new()?
        };

        let kad = libp2p::kad::Behaviour::new(peer_id.clone(), database);
        let identify_config = libp2p::identify::Config::new("/newsnet/0.0.1".to_string(), pub_key);
        let identify = libp2p::identify::Behaviour::new(identify_config);
        let mdns_config = libp2p::mdns::Config::default();
        let mdns = libp2p::mdns::Behaviour::new(mdns_config, peer_id)?;
        let limits = libp2p::connection_limits::ConnectionLimits::default();
        let connection_limit = libp2p::connection_limits::Behaviour::new(limits);

        Ok(Self {
            kad,
            identify,
            mdns,
            connection_limit,
        })
    }
}
