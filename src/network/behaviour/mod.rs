use crate::network::config::StorageBackend;
use libp2p::{PeerId, connection_limits, identity::Keypair, swarm::NetworkBehaviour};
use netabase_store::{databases::sled_store::SledStore, traits::definition::{NetabaseDefinitionTrait, RecordStoreExt}};

pub mod clone_impl;
// Sync behaviour module commented out - not yet fully implemented
// pub mod sync_behaviour;

#[cfg(feature = "native")]
use libp2p::mdns;

// Kad behavior now works with generic Definition types via instance method dispatch
#[derive(NetworkBehaviour)]
pub struct NetabaseBehaviour<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>
where
    D: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: netabase_store::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as strum::IntoDiscriminant>::Discriminant: netabase_store::traits::definition::NetabaseKeyDiscriminant,
{
    pub kad: libp2p::kad::Behaviour<SledStore<D>>,
    pub identify: libp2p::identify::Behaviour,
    #[cfg(feature = "native")]
    pub mdns: libp2p::mdns::tokio::Behaviour,
    pub connection_limit: libp2p::connection_limits::Behaviour,
}

impl<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static> NetabaseBehaviour<D>
where
    D: netabase_store::convert::ToIVec,
    D::Keys: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: netabase_store::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as strum::IntoDiscriminant>::Discriminant: netabase_store::traits::definition::NetabaseKeyDiscriminant,
{
    pub fn _new(keypair: &Keypair) -> Result<Self, crate::errors::Error> {
        Self::new_with_config(keypair, None, StorageBackend::default())
    }

    pub fn _new_with_name(
        keypair: &Keypair,
        name: Option<String>,
    ) -> Result<Self, crate::errors::Error> {
        Self::new_with_config(keypair, name, StorageBackend::default())
    }

    pub fn new_with_config(
        keypair: &Keypair,
        name: Option<String>,
        _backend: StorageBackend,
    ) -> Result<Self, crate::errors::Error> {
        let pub_key = keypair.public();
        let peer_id = PeerId::from_public_key(&pub_key);

        // Create the store - currently using SledStore, backend parameter ignored
        #[cfg(feature = "native")]
        let store = if let Some(name) = name {
            SledStore::<D>::new(&name)
                .map_err(|e| crate::errors::Error::Database(e.to_string()))?
        } else {
            // Use a default path based on peer_id
            let default_path = format!("./netabase_data/{}", peer_id);
            SledStore::<D>::new(&default_path)
                .map_err(|e| crate::errors::Error::Database(e.to_string()))?
        };

        #[cfg(all(feature = "wasm", not(feature = "native")))]
        let store = SledStore::<D>::temp()
            .map_err(|e| crate::errors::Error::Database(e.to_string()))?;

        let kad = libp2p::kad::Behaviour::new(peer_id.clone(), store);
        let identify_config = libp2p::identify::Config::new("/newsnet/0.0.1".to_string(), pub_key);
        let identify = libp2p::identify::Behaviour::new(identify_config);

        #[cfg(feature = "native")]
        let mdns_config = mdns::Config::default();
        #[cfg(feature = "native")]
        let mdns = mdns::tokio::Behaviour::new(mdns_config, peer_id)?;

        let limits = connection_limits::ConnectionLimits::default();
        let connection_limit = connection_limits::Behaviour::new(limits);

        Ok(Self {
            kad,
            identify,
            #[cfg(feature = "native")]
            mdns,
            connection_limit,
        })
    }
}
