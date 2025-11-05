/// Unified store enum that wraps different backend types
/// This allows backend selection while maintaining a single type for libp2p RecordStore
use crate::network::config::StorageBackend;
use netabase_store::{
    databases::sled_store::SledStore, traits::definition::NetabaseDefinitionTrait,
};

#[cfg(feature = "native")]
use netabase_store::databases::redb_store::RedbStore;


/// Unified store that can use any supported backend
pub enum NetabaseStore<D>
where
    D: NetabaseDefinitionTrait,
    <D as strum::IntoDiscriminant>::Discriminant: netabase_store::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as strum::IntoDiscriminant>::Discriminant: netabase_store::traits::definition::NetabaseKeyDiscriminant,
{
    Sled(SledStore<D>),
    #[cfg(feature = "native")]
    Redb(RedbStore<D>),
}

impl<D> NetabaseStore<D>
where
    D: NetabaseDefinitionTrait + netabase_store::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: netabase_store::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as strum::IntoDiscriminant>::Discriminant: netabase_store::traits::definition::NetabaseKeyDiscriminant,
{
    /// Create a new store with the specified backend and path
    pub fn new(backend: StorageBackend, path: &str) -> anyhow::Result<Self> {
        match backend {
            StorageBackend::Sled => Ok(NetabaseStore::Sled(SledStore::new(path)?)),
            #[cfg(feature = "native")]
            StorageBackend::Redb => Ok(NetabaseStore::Redb(RedbStore::new(path)?)),
            #[cfg(feature = "wasm")]
            StorageBackend::IndexedDB => {
                // IndexedDB doesn't use file paths
                Err(anyhow::anyhow!(
                    "IndexedDB backend must be created with new_async"
                ))
            }
        }
    }

    /// Create a temporary store (used for testing)
    #[cfg(feature = "native")]
    pub fn temp(backend: StorageBackend) -> anyhow::Result<Self> {
        match backend {
            StorageBackend::Sled => Ok(NetabaseStore::Sled(SledStore::temp()?)),
            #[cfg(feature = "native")]
            StorageBackend::Redb => {
                use tempfile::NamedTempFile;
                let temp_file = NamedTempFile::new()?;
                let temp_path = temp_file.path();
                Ok(NetabaseStore::Redb(RedbStore::new(temp_path)?))
            }
            #[cfg(feature = "wasm")]
            StorageBackend::IndexedDB => Err(anyhow::anyhow!("IndexedDB temp not supported")),
        }
    }
}

// RecordStore implementations moved to netabase_store to satisfy orphan rules
// (RecordStore is from libp2p, SledStore/RedbStore are from netabase_store)
