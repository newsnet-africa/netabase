use libp2p::{PeerId, identify, identity::Keypair, kad, mdns, swarm::NetworkBehaviour};

use crate::{
    config::{BehaviourConfig, KadStoreConfig},
    database::SledStore,
};

// Enum to hold different store types
pub enum StoreType {
    Memory(kad::store::MemoryStore),
    Sled(SledStore),
}

impl kad::store::RecordStore for StoreType {
    type RecordsIter<'a> = Box<dyn Iterator<Item = std::borrow::Cow<'a, kad::Record>> + 'a>;
    type ProvidedIter<'a> =
        Box<dyn Iterator<Item = std::borrow::Cow<'a, kad::ProviderRecord>> + 'a>;

    fn get(&self, k: &kad::RecordKey) -> Option<std::borrow::Cow<'_, kad::Record>> {
        match self {
            StoreType::Memory(store) => store.get(k),
            StoreType::Sled(store) => store.get(k),
        }
    }

    fn put(&mut self, r: kad::Record) -> kad::store::Result<()> {
        match self {
            StoreType::Memory(store) => store.put(r),
            StoreType::Sled(store) => store.put(r),
        }
    }

    fn remove(&mut self, k: &kad::RecordKey) {
        match self {
            StoreType::Memory(store) => store.remove(k),
            StoreType::Sled(store) => store.remove(k),
        }
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        match self {
            StoreType::Memory(store) => Box::new(store.records()),
            StoreType::Sled(store) => Box::new(store.records()),
        }
    }

    fn add_provider(&mut self, record: kad::ProviderRecord) -> kad::store::Result<()> {
        match self {
            StoreType::Memory(store) => store.add_provider(record),
            StoreType::Sled(store) => store.add_provider(record),
        }
    }

    fn providers(&self, key: &kad::RecordKey) -> Vec<kad::ProviderRecord> {
        match self {
            StoreType::Memory(store) => store.providers(key),
            StoreType::Sled(store) => store.providers(key),
        }
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        match self {
            StoreType::Memory(store) => Box::new(store.provided()),
            StoreType::Sled(store) => Box::new(store.provided()),
        }
    }

    fn remove_provider(&mut self, k: &kad::RecordKey, provider: &PeerId) {
        match self {
            StoreType::Memory(store) => store.remove_provider(k, provider),
            StoreType::Sled(store) => store.remove_provider(k, provider),
        }
    }
}

#[derive(NetworkBehaviour)]
pub struct NetabaseBehaviour {
    pub kad: kad::Behaviour<StoreType>,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
}

impl NetabaseBehaviour {
    pub fn new(key: &Keypair, config: &BehaviourConfig) -> anyhow::Result<Self> {
        let peer_id = PeerId::from_public_key(&key.public());

        let store = match config.store_config() {
            KadStoreConfig::MemoryStore {
                peer_id: store_peer_id,
                config: memory_config,
            } => {
                if let Some(memory_config) = memory_config {
                    StoreType::Memory(kad::store::MemoryStore::with_config(
                        *store_peer_id,
                        memory_config.clone(),
                    ))
                } else {
                    StoreType::Memory(kad::store::MemoryStore::new(*store_peer_id))
                }
            }
            KadStoreConfig::SledStore {
                path,
                config: sled_config,
            } => {
                if let Some(sled_config) = sled_config {
                    StoreType::Sled(SledStore::with_config(peer_id, path, sled_config.clone())?)
                } else {
                    StoreType::Sled(SledStore::new(peer_id, path)?)
                }
            }
        };

        let kad = if let Some(kad_config) = config.kad_config() {
            kad::Behaviour::with_config(peer_id, store, kad_config.clone())
        } else {
            kad::Behaviour::new(peer_id, store)
        };

        let mdns = if let Some(mdns_config) = config.mdns_config() {
            mdns::Behaviour::new(mdns_config.clone(), peer_id)?
        } else {
            mdns::Behaviour::new(mdns::Config::default(), peer_id)?
        };

        let identify = if let Some(identify_config) = config.identify_config() {
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

    pub fn new_test(key: &Keypair, test_number: usize) -> Self {
        let config = crate::config::DefaultBehaviourConfig::builder()
            .store_config(KadStoreConfig::sled_store(format!(
                "./test/database{test_number}"
            )))
            .protocol_version("/p2p/newsnet/0.0.0".to_string())
            .build()
            .expect("Default BehaviourConfig should be valid");

        Self::new(key, &config).expect("Test behaviour creation should succeed")
    }

    pub fn with_memory_store(key: &Keypair, peer_id: PeerId) -> anyhow::Result<Self> {
        let config = BehaviourConfig::with_memory_store(peer_id)
            .build()
            .expect("Memory store config should be valid");
        Self::new(key, &config)
    }

    pub fn with_sled_store<P: AsRef<str>>(key: &Keypair, path: P) -> anyhow::Result<Self> {
        let config = BehaviourConfig::with_sled_store(path)
            .build()
            .expect("Sled store config should be valid");
        Self::new(key, &config)
    }

    pub fn with_sled_store_config<P: AsRef<str>>(
        key: &Keypair,
        path: P,
        config: crate::database::SledStoreConfig,
    ) -> anyhow::Result<Self> {
        let behaviour_config = BehaviourConfig::with_sled_store_config(path, config)
            .build()
            .expect("Sled store config should be valid");
        Self::new(key, &behaviour_config)
    }

    pub fn with_memory_store_config(
        key: &Keypair,
        peer_id: PeerId,
        config: kad::store::MemoryStoreConfig,
    ) -> anyhow::Result<Self> {
        let behaviour_config = BehaviourConfig::with_memory_store_config(peer_id, config)
            .build()
            .expect("Memory store config should be valid");
        Self::new(key, &behaviour_config)
    }
}
