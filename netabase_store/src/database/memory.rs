//! Memory-based database implementation for WASM builds using libp2p::kad::store::MemoryStore
//!
//! This module provides a wrapper around libp2p's MemoryStore to integrate with
//! Netabase traits and work in WASM environments where sled is not available.

use std::collections::HashMap;
use std::marker::PhantomData;

#[cfg(feature = "libp2p")]
use libp2p::PeerId;
#[cfg(feature = "libp2p")]
use libp2p::kad::{
    ProviderRecord, Record, RecordKey,
    store::{MemoryStore, RecordStore, Result as RecordStoreResult},
};

use crate::errors::NetabaseError;
use crate::traits::NetabaseSchema;

#[cfg(feature = "libp2p")]
use crate::traits::NetabaseRecordStoreQuery;

/// Memory-based vector that mimics sled::IVec for compatibility
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryIVec(Vec<u8>);

impl MemoryIVec {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn as_ref(&self) -> &[u8] {
        &self.0
    }

    pub fn to_vec(self) -> Vec<u8> {
        self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<u8>> for MemoryIVec {
    fn from(vec: Vec<u8>) -> Self {
        Self(vec)
    }
}

impl From<&[u8]> for MemoryIVec {
    fn from(slice: &[u8]) -> Self {
        Self(slice.to_vec())
    }
}

impl From<String> for MemoryIVec {
    fn from(s: String) -> Self {
        Self(s.into_bytes())
    }
}

impl From<&str> for MemoryIVec {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }
}

impl AsRef<[u8]> for MemoryIVec {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::ops::Deref for MemoryIVec {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Compatibility implementations for trait bounds
impl TryInto<Vec<u8>> for MemoryIVec {
    type Error = NetabaseError;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        Ok(self.0)
    }
}

/// Enhanced memory database that wraps libp2p::kad::store::MemoryStore
pub struct NetabaseMemoryDatabase<M>
where
    M: NetabaseSchema,
{
    #[cfg(feature = "libp2p")]
    memory_store: MemoryStore,
    // In-memory storage for non-libp2p data organized by schema discriminants
    model_storage: HashMap<M::SchemaDiscriminants, HashMap<Vec<u8>, Vec<u8>>>,
    _phantom: PhantomData<M>,
}

impl<M> NetabaseMemoryDatabase<M>
where
    M: NetabaseSchema,
    M::SchemaDiscriminants: AsRef<str> + Clone + std::hash::Hash + Eq + strum::IntoEnumIterator,
{
    /// Create a new enhanced memory database
    pub fn new() -> Result<Self, NetabaseError> {
        #[cfg(feature = "libp2p")]
        let memory_store = MemoryStore::new(PeerId::random());

        let mut model_storage = HashMap::new();

        // Initialize storage for each schema discriminant
        for discriminant in M::all_schema_discriminants() {
            model_storage.insert(discriminant, HashMap::new());
        }

        Ok(Self {
            #[cfg(feature = "libp2p")]
            memory_store,
            model_storage,
            _phantom: PhantomData,
        })
    }

    /// Get a reference to the underlying libp2p memory store
    #[cfg(feature = "libp2p")]
    pub fn memory_store(&self) -> &MemoryStore {
        &self.memory_store
    }

    /// Get a mutable reference to the underlying libp2p memory store
    #[cfg(feature = "libp2p")]
    pub fn memory_store_mut(&mut self) -> &mut MemoryStore {
        &mut self.memory_store
    }

    /// Get a reference to a tree by discriminant (returns a reference to the HashMap)
    pub fn get_main_tree_by_discriminant(
        &self,
        schema_discriminant: &M::SchemaDiscriminants,
    ) -> Option<&HashMap<Vec<u8>, Vec<u8>>> {
        self.model_storage.get(schema_discriminant)
    }

    /// Get a mutable reference to a tree by discriminant
    pub fn get_main_tree_by_discriminant_mut(
        &mut self,
        schema_discriminant: &M::SchemaDiscriminants,
    ) -> Option<&mut HashMap<Vec<u8>, Vec<u8>>> {
        self.model_storage.get_mut(schema_discriminant)
    }

    /// Open a tree for a specific model
    pub fn open_tree_for_model<Model, ModelKey>(
        &self,
    ) -> Result<NetabaseMemoryTree<Model, ModelKey>, NetabaseError>
    where
        Model: crate::traits::NetabaseModel<Key = ModelKey>
            + TryFrom<MemoryIVec>
            + TryInto<MemoryIVec>,
        ModelKey:
            crate::traits::NetabaseModelKey + TryFrom<MemoryIVec> + TryInto<MemoryIVec> + Clone,
    {
        Ok(NetabaseMemoryTree {
            _phantom: PhantomData::<(Model, ModelKey)>,
        })
    }

    /// Get the main tree for a specific model type
    pub fn get_main_tree<Model, ModelKey>(
        &self,
    ) -> Result<NetabaseMemoryTree<Model, ModelKey>, NetabaseError>
    where
        Model: crate::traits::NetabaseModel<Key = ModelKey>
            + TryFrom<MemoryIVec>
            + TryInto<MemoryIVec>,
        ModelKey:
            crate::traits::NetabaseModelKey + TryFrom<MemoryIVec> + TryInto<MemoryIVec> + Clone,
    {
        self.open_tree_for_model()
    }

    /// Insert data directly into model storage
    fn insert_model_data(
        &mut self,
        discriminant: &M::SchemaDiscriminants,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, NetabaseError> {
        if let Some(tree) = self.model_storage.get_mut(discriminant) {
            Ok(tree.insert(key, value))
        } else {
            Err(NetabaseError::Database)
        }
    }

    /// Get data directly from model storage
    fn get_model_data(
        &self,
        discriminant: &M::SchemaDiscriminants,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, NetabaseError> {
        if let Some(tree) = self.model_storage.get(discriminant) {
            Ok(tree.get(key).cloned())
        } else {
            Err(NetabaseError::Database)
        }
    }

    /// Remove data from model storage
    fn remove_model_data(
        &mut self,
        discriminant: &M::SchemaDiscriminants,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, NetabaseError> {
        if let Some(tree) = self.model_storage.get_mut(discriminant) {
            Ok(tree.remove(key))
        } else {
            Err(NetabaseError::Database)
        }
    }
}

impl<M> Default for NetabaseMemoryDatabase<M>
where
    M: NetabaseSchema,
    M::SchemaDiscriminants: AsRef<str> + Clone + std::hash::Hash + Eq + strum::IntoEnumIterator,
{
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// Type-safe tree wrapper that operates on the memory database
pub struct NetabaseMemoryTree<Model, ModelKey>
where
    Model: crate::traits::NetabaseModel<Key = ModelKey>,
    ModelKey: crate::traits::NetabaseModelKey,
{
    _phantom: PhantomData<(Model, ModelKey)>,
}

impl<Model, ModelKey> NetabaseMemoryTree<Model, ModelKey>
where
    Model: crate::traits::NetabaseModel<Key = ModelKey>,
    ModelKey: crate::traits::NetabaseModelKey,
{
    /// Get a model from the tree (placeholder implementation)
    pub fn get(&self, _key: &ModelKey) -> Result<Option<Model>, NetabaseError> {
        // Placeholder implementation for WASM builds
        // In a real implementation, this would access a global storage
        Ok(None)
    }

    /// Get the number of items in the tree (placeholder implementation)
    pub fn len(&self) -> usize {
        0
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Iterator for memory tree - simplified version
pub struct NetabaseMemoryIter<Model, ModelKey>
where
    Model: crate::traits::NetabaseModel<Key = ModelKey>,
    ModelKey: crate::traits::NetabaseModelKey,
{
    entries: Vec<(Vec<u8>, Vec<u8>)>,
    position: usize,
    _phantom: PhantomData<(Model, ModelKey)>,
}

impl<Model, ModelKey> Iterator for NetabaseMemoryIter<Model, ModelKey>
where
    Model: crate::traits::NetabaseModel<Key = ModelKey> + TryFrom<MemoryIVec>,
    ModelKey: crate::traits::NetabaseModelKey + TryFrom<MemoryIVec>,
{
    type Item = Result<(ModelKey, Model), NetabaseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.entries.len() {
            let (key_bytes, model_bytes) = &self.entries[self.position];
            self.position += 1;

            let key_result = ModelKey::try_from(MemoryIVec::from(key_bytes.clone()))
                .map_err(|_| NetabaseError::Serialization);
            let model_result = Model::try_from(MemoryIVec::from(model_bytes.clone()))
                .map_err(|_| NetabaseError::Serialization);

            match (key_result, model_result) {
                (Ok(key), Ok(model)) => Some(Ok((key, model))),
                (Err(e), _) | (_, Err(e)) => Some(Err(e)),
            }
        } else {
            None
        }
    }
}

/// Simple iterator over raw memory data for compatibility with traits
pub struct MemoryIter {
    entries: Vec<(MemoryIVec, MemoryIVec)>,
    position: usize,
}

impl MemoryIter {
    pub fn new(entries: Vec<(MemoryIVec, MemoryIVec)>) -> Self {
        Self {
            entries,
            position: 0,
        }
    }

    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
            position: 0,
        }
    }
}

impl Iterator for MemoryIter {
    type Item = Result<(MemoryIVec, MemoryIVec), NetabaseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.entries.len() {
            let entry = self.entries[self.position].clone();
            self.position += 1;
            Some(Ok(entry))
        } else {
            None
        }
    }
}

// Compatibility type aliases for easier migration
pub type NetabaseIter<Model, ModelKey> = NetabaseMemoryIter<Model, ModelKey>;
pub type NetabaseTreeCompatible<Model, ModelKey> = NetabaseMemoryTree<Model, ModelKey>;

#[cfg(feature = "libp2p")]
impl<M> RecordStore for NetabaseMemoryDatabase<M>
where
    M: NetabaseSchema,
    M::SchemaDiscriminants: AsRef<str> + Clone + std::hash::Hash + Eq + strum::IntoEnumIterator,
{
    type RecordsIter<'a>
        = <MemoryStore as RecordStore>::RecordsIter<'a>
    where
        Self: 'a;
    type ProvidedIter<'a>
        = <MemoryStore as RecordStore>::ProvidedIter<'a>
    where
        Self: 'a;

    fn get(&self, key: &RecordKey) -> Option<std::borrow::Cow<'_, Record>> {
        self.memory_store.get(key)
    }

    fn put(&mut self, record: Record) -> RecordStoreResult<()> {
        self.memory_store.put(record)
    }

    fn remove(&mut self, key: &RecordKey) {
        self.memory_store.remove(key)
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        self.memory_store.records()
    }

    fn add_provider(&mut self, record: ProviderRecord) -> RecordStoreResult<()> {
        self.memory_store.add_provider(record)
    }

    fn providers(&self, key: &RecordKey) -> Vec<ProviderRecord> {
        self.memory_store.providers(key)
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        self.memory_store.provided()
    }

    fn remove_provider(&mut self, key: &RecordKey, provider: &PeerId) {
        self.memory_store.remove_provider(key, provider)
    }
}

#[cfg(feature = "libp2p")]
impl<M> crate::traits::NetabaseRecordStoreQuery<M> for NetabaseMemoryDatabase<M>
where
    M: NetabaseSchema,
    M::SchemaDiscriminants: AsRef<str> + Clone + std::hash::Hash + Eq + strum::IntoEnumIterator,
{
    fn schema_key_to_record_key(_key: &M::Keys) -> Result<libp2p::kad::RecordKey, NetabaseError> {
        // Placeholder implementation
        Err(NetabaseError::Database)
    }

    fn record_key_to_schema_key(
        _record_key: &libp2p::kad::RecordKey,
    ) -> Result<M::Keys, NetabaseError> {
        // Placeholder implementation
        Err(NetabaseError::Database)
    }

    fn get_schema_by_record_key(
        &self,
        _record_key: &libp2p::kad::RecordKey,
    ) -> Result<Option<M>, NetabaseError> {
        // Placeholder implementation
        Ok(None)
    }

    fn schema_to_record(_schema: &M) -> Result<libp2p::kad::Record, NetabaseError> {
        // Placeholder implementation
        Err(NetabaseError::Database)
    }

    fn record_to_schema(_record: &libp2p::kad::Record) -> Result<M, NetabaseError> {
        // Placeholder implementation
        Err(NetabaseError::Database)
    }
}
