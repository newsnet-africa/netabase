/// Unified store enum that wraps different backend types
/// This allows backend selection while maintaining a single type for libp2p RecordStore
use crate::network::config::StorageBackend;
use netabase_store::{
    databases::sled_store::SledStore, traits::definition::NetabaseDefinitionTrait,
};

#[cfg(feature = "native")]
use netabase_store::databases::redb_store::RedbStore;

#[cfg(feature = "libp2p")]
use libp2p::kad::{
    store::{Error, RecordStore},
    ProviderRecord, Record, RecordKey,
};
#[cfg(feature = "libp2p")]
use libp2p::PeerId;
#[cfg(feature = "libp2p")]
use std::borrow::Cow;

/// Unified store that can use any supported backend
pub enum NetabaseStore<D>
where
    D: NetabaseDefinitionTrait,
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
{
    Sled(SledStore<D>),
    #[cfg(feature = "native")]
    Redb(RedbStore<D>),
}

impl<D> NetabaseStore<D>
where
    D: NetabaseDefinitionTrait + netabase_store::convert::ToIVec,
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

// Implement RecordStore for the unified store by delegating to the wrapped type
#[cfg(feature = "libp2p")]
impl<D> RecordStore for NetabaseStore<D>
where
    D: NetabaseDefinitionTrait,
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
    type RecordsIter<'a>
        = Box<dyn Iterator<Item = Cow<'a, Record>> + 'a>
    where
        Self: 'a;

    type ProvidedIter<'a>
        = Box<dyn Iterator<Item = Cow<'a, ProviderRecord>> + 'a>
    where
        Self: 'a;

    fn get(&self, k: &RecordKey) -> Option<Cow<'_, Record>> {
        match self {
            NetabaseStore::Sled(store) => store.get(k),
            #[cfg(feature = "native")]
            NetabaseStore::Redb(store) => store.get(k),
        }
    }

    fn put(&mut self, r: Record) -> libp2p::kad::store::Result<()> {
        match self {
            NetabaseStore::Sled(store) => store.put(r),
            #[cfg(feature = "native")]
            NetabaseStore::Redb(store) => store.put(r),
        }
    }

    fn remove(&mut self, k: &RecordKey) {
        match self {
            NetabaseStore::Sled(store) => store.remove(k),
            #[cfg(feature = "native")]
            NetabaseStore::Redb(store) => store.remove(k),
        }
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        match self {
            NetabaseStore::Sled(store) => Box::new(store.records()),
            #[cfg(feature = "native")]
            NetabaseStore::Redb(store) => Box::new(store.records()),
        }
    }

    fn add_provider(&mut self, record: ProviderRecord) -> libp2p::kad::store::Result<()> {
        match self {
            NetabaseStore::Sled(store) => store.add_provider(record),
            #[cfg(feature = "native")]
            NetabaseStore::Redb(store) => store.add_provider(record),
        }
    }

    fn providers(&self, key: &RecordKey) -> Vec<ProviderRecord> {
        match self {
            NetabaseStore::Sled(store) => store.providers(key),
            #[cfg(feature = "native")]
            NetabaseStore::Redb(store) => store.providers(key),
        }
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        match self {
            NetabaseStore::Sled(store) => Box::new(store.provided()),
            #[cfg(feature = "native")]
            NetabaseStore::Redb(store) => Box::new(store.provided()),
        }
    }

    fn remove_provider(&mut self, key: &RecordKey, provider: &PeerId) {
        match self {
            NetabaseStore::Sled(store) => store.remove_provider(key, provider),
            #[cfg(feature = "native")]
            NetabaseStore::Redb(store) => store.remove_provider(key, provider),
        }
    }
}
