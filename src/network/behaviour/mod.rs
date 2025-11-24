use crate::network::{config::StorageBackend, store::NetabaseStore};
use libp2p::swarm::behaviour::toggle::Toggle;
use libp2p::{connection_limits, identity::Keypair, swarm::NetworkBehaviour, PeerId};
use netabase_store::traits::definition::NetabaseDefinitionTrait;

pub mod clone_impl;
pub mod sync_behaviour;

#[cfg(feature = "native")]
use libp2p::mdns;

#[cfg(feature = "paxos")]
use sync_behaviour::paxos::PaxosBehaviour;

#[derive(NetworkBehaviour)]
pub struct NetabaseBehaviour<D: NetabaseDefinitionTrait + Send + Sync + 'static>
where
    D: netabase_store::convert::ToIVec + serde::Serialize + for<'de> serde::Deserialize<'de>,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    /// Kademlia DHT behavior (disabled when Paxos is enabled)
    ///
    /// When paxos feature is disabled: This owns the store and provides DHT functionality
    /// When paxos feature is enabled: This is disabled (Toggle::from(None)) and DHT access
    /// goes through paxos.kad instead
    pub kad: Toggle<libp2p::kad::Behaviour<NetabaseStore<D>>>,
    pub identify: libp2p::identify::Behaviour,
    #[cfg(feature = "native")]
    pub mdns: libp2p::mdns::tokio::Behaviour,
    pub connection_limit: libp2p::connection_limits::Behaviour,
    /// Paxos consensus behavior with embedded Kademlia
    ///
    /// When enabled, this owns the store and provides both consensus and DHT functionality
    /// via its embedded kad field (paxos.kad)
    #[cfg(feature = "paxos")]
    pub paxos: Toggle<PaxosBehaviour<D>>,
}

impl<D: NetabaseDefinitionTrait + Send + Sync + 'static> NetabaseBehaviour<D>
where
    D: netabase_store::convert::ToIVec + serde::Serialize + for<'de> serde::Deserialize<'de>,
    D::Keys: netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    pub fn _new(keypair: &Keypair) -> Result<Self, crate::errors::Error> {
        Self::new_with_config(keypair, None, Default::default())
    }

    pub fn _new_with_name(
        keypair: &Keypair,
        name: Option<String>,
    ) -> Result<Self, crate::errors::Error> {
        Self::new_with_config(keypair, name, Default::default())
    }

    pub fn new_with_config(
        keypair: &Keypair,
        name: Option<String>,
        config: crate::network::config::NetabaseConfig,
    ) -> Result<Self, crate::errors::Error> {
        let backend = config.storage_backend;
        let pub_key = keypair.public();
        let peer_id = PeerId::from_public_key(&pub_key);

        // Create the store - this will be owned by either kad or paxos.kad
        #[cfg(feature = "native")]
        let store = if let Some(ref name) = name {
            NetabaseStore::<D>::new(backend, name)
                .map_err(|e| crate::errors::Error::Database(e.to_string()))?
        } else {
            // Use a default path based on peer_id
            let default_path = format!("./netabase_data/{}", peer_id);
            NetabaseStore::<D>::new(backend, &default_path)
                .map_err(|e| crate::errors::Error::Database(e.to_string()))?
        };

        #[cfg(all(feature = "wasm", not(feature = "native")))]
        let store = NetabaseStore::<D>::temp(backend)
            .map_err(|e| crate::errors::Error::Database(e.to_string()))?;

        let identify_config = libp2p::identify::Config::new("/newsnet/0.0.1".to_string(), pub_key);
        let identify = libp2p::identify::Behaviour::new(identify_config);

        #[cfg(feature = "native")]
        let mdns_config = mdns::Config::default();
        #[cfg(feature = "native")]
        let mdns = mdns::tokio::Behaviour::new(mdns_config, peer_id)?;

        let limits = connection_limits::ConnectionLimits::default();
        let connection_limit = connection_limits::Behaviour::new(limits);

        // Phase 5: Toggle pattern for kad/paxos
        // When paxos is enabled, it owns the store and kad is disabled
        // When paxos is disabled, kad owns the store
        #[cfg(feature = "paxos")]
        let (kad, paxos) = {
            use crate::network::behaviour::sync_behaviour::paxos::NodeId;

            // Convert PeerId to NodeId
            let node_id = NodeId::from(peer_id);

            // Create kad behaviour wrapping the store
            let kad_behaviour = libp2p::kad::Behaviour::new(peer_id.clone(), store);

            // Paxos owns the kad behaviour, top-level kad is disabled
            let paxos_behaviour = PaxosBehaviour::new(node_id, kad_behaviour);
            (
                Toggle::from(None), // kad disabled
                Toggle::from(Some(paxos_behaviour)) // paxos enabled with embedded kad
            )
        };

        #[cfg(not(feature = "paxos"))]
        let kad = {
            // kad owns the store, paxos is disabled
            let kad_behaviour = libp2p::kad::Behaviour::new(peer_id.clone(), store);
            Toggle::from(Some(kad_behaviour)) // kad enabled
        };

        Ok(Self {
            kad,
            identify,
            #[cfg(feature = "native")]
            mdns,
            connection_limit,
            #[cfg(feature = "paxos")]
            paxos,
        })
    }
}

// Separate impl block for helper methods that don't require D::Keys bound
// This allows command handlers to use kad()/kad_mut() without extra trait bounds
impl<D: NetabaseDefinitionTrait + Send + Sync + 'static> NetabaseBehaviour<D>
where
    D: netabase_store::convert::ToIVec + serde::Serialize + for<'de> serde::Deserialize<'de>,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    /// Get a reference to the Kademlia behavior
    ///
    /// Returns the kad behavior from the appropriate location:
    /// - When paxos is disabled: Returns self.kad
    /// - When paxos is enabled: Returns self.paxos.kad
    ///
    /// # Returns
    /// Option<&libp2p::kad::Behaviour<NetabaseStore<D>>>
    pub fn kad(&self) -> Option<&libp2p::kad::Behaviour<NetabaseStore<D>>> {
        // Try paxos.kad first (when paxos is enabled)
        #[cfg(feature = "paxos")]
        if let Some(paxos_behaviour) = self.paxos.as_ref() {
            return Some(&paxos_behaviour.store);
        }

        // Fall back to top-level kad (when paxos is disabled)
        self.kad.as_ref()
    }

    /// Get a mutable reference to the Kademlia behavior
    ///
    /// Returns the kad behavior from the appropriate location:
    /// - When paxos is disabled: Returns self.kad
    /// - When paxos is enabled: Returns self.paxos.kad
    ///
    /// # Returns
    /// Option<&mut libp2p::kad::Behaviour<NetabaseStore<D>>>
    pub fn kad_mut(&mut self) -> Option<&mut libp2p::kad::Behaviour<NetabaseStore<D>>> {
        // Try paxos.kad first (when paxos is enabled)
        #[cfg(feature = "paxos")]
        if let Some(paxos_behaviour) = self.paxos.as_mut() {
            return Some(&mut paxos_behaviour.store);
        }

        // Fall back to top-level kad (when paxos is disabled)
        self.kad.as_mut()
    }
}
