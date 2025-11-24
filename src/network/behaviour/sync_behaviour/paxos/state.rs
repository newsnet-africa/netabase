use super::{log_entry::LogEntry, NodeId, PaxosBehaviour, RoundNum};
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};
use paxakos::cluster::Cluster;
use paxakos::state::Frozen as PaxakosFrozen;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Frozen snapshot of the datastore state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "D: Serialize + serde::de::DeserializeOwned")]
pub struct Frozen<D>
where
    D: NetabaseDefinitionTrait,
{
    /// All entries in the datastore at the time of freezing
    pub entries: Vec<(Vec<u8>, Vec<u8>)>,
    /// Metadata for reconstruction
    pub metadata: HashMap<String, Vec<u8>>,
    /// Phantom data to use the D type parameter
    #[serde(skip)]
    _phantom: std::marker::PhantomData<D>,
}

impl<D> Frozen<D>
where
    D: NetabaseDefinitionTrait,
{
    /// Create a new frozen snapshot
    pub fn new(entries: Vec<(Vec<u8>, Vec<u8>)>, metadata: HashMap<String, Vec<u8>>) -> Self {
        Self {
            entries,
            metadata,
            _phantom: std::marker::PhantomData,
        }
    }
}

// Implement paxakos::Frozen trait with comprehensive bounds
impl<D> PaxakosFrozen<PaxosBehaviour<D>> for Frozen<D>
where
    D: NetabaseDefinitionTrait + RecordStoreExt + Clone + Send + Sync + 'static,
    D: netabase_store::convert::ToIVec + Serialize + Unpin,
    for<'d> D: Deserialize<'d>,
    <D as strum::IntoDiscriminant>::Discriminant:
        netabase_store::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as strum::IntoDiscriminant>::Discriminant:
        netabase_store::traits::definition::NetabaseKeyDiscriminant,
    <D as NetabaseDefinitionTrait>::Keys: netabase_store::convert::ToIVec + Clone + Serialize + Unpin,
    for<'d> <D as NetabaseDefinitionTrait>::Keys: Deserialize<'d>,
{
    fn thaw(&self, _context: &mut Context) -> PaxosBehaviour<D> {
        // This cannot be properly implemented without external dependencies.
        // The caller should use a builder pattern or factory instead.
        panic!("Frozen::thaw() is not supported - use PaxosBehaviourBuilder instead");
    }
}

/// Application-specific context (can be extended later if needed)
#[derive(Debug, Default)]
pub struct Context {
    // Placeholder for future context data (e.g., working directory, temp storage, etc.)
}

/// Outcome of applying a log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Outcome {
    /// Entry was successfully applied
    Success,
    /// Entry was a duplicate and was ignored
    Duplicate,
}

/// Effect information from applying a log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    /// A model was created
    Created { key: Vec<u8> },
    /// A model was deleted
    Deleted { key: Vec<u8> },
}

/// State error types
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Failed to serialize/deserialize: {0}")]
    Serialization(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Invalid model data: {0}")]
    InvalidModel(String),
}

// Implement State trait for PaxosBehaviour
impl<D> paxakos::State for PaxosBehaviour<D>
where
    D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static,
    D: netabase_store::convert::ToIVec + Clone + Serialize + Unpin,
    for<'d> D: Deserialize<'d>,
    <D as strum::IntoDiscriminant>::Discriminant:
        netabase_store::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as strum::IntoDiscriminant>::Discriminant:
        netabase_store::traits::definition::NetabaseKeyDiscriminant,
    <D as NetabaseDefinitionTrait>::Keys: netabase_store::convert::ToIVec + Clone + Serialize + Unpin,
    for<'d> <D as NetabaseDefinitionTrait>::Keys: Deserialize<'d>,
{
    type Frozen = Frozen<D>;
    type LogEntry = LogEntry<D>;
    type Context = Context;
    type Outcome = Outcome;
    type Effect = Effect;
    type Error = StateError;
    type Node = NodeId;

    /// Apply a log entry to the Kademlia datastore
    fn apply(
        &mut self,
        log_entry: &Self::LogEntry,
        _context: &mut Self::Context,
    ) -> Result<(Self::Outcome, Self::Effect), Self::Error> {
        match log_entry {
            LogEntry::Create(model) => {
                // Serialize the model to bytes for storage
                let config = bincode::config::standard();
                let key = bincode::encode_to_vec(model, config)
                    .map_err(|e| StateError::Serialization(e.to_string()))?;

                // Create a libp2p Record
                let record = libp2p::kad::Record {
                    key: libp2p::kad::RecordKey::new(&key),
                    value: key.clone(),
                    publisher: None,
                    expires: None,
                };

                // Store in Kademlia's record store
                use libp2p::kad::store::RecordStore;
                self.store
                    .store_mut()
                    .put(record)
                    .map_err(|e| StateError::Storage(format!("{:?}", e)))?;

                Ok((
                    Outcome::Success,
                    Effect::Created { key: key.to_vec() },
                ))
            }
            LogEntry::Delete(model) => {
                // Serialize the model to get its key
                let config = bincode::config::standard();
                let key = bincode::encode_to_vec(model, config)
                    .map_err(|e| StateError::Serialization(e.to_string()))?;

                // Remove from Kademlia's record store
                use libp2p::kad::store::RecordStore;
                let record_key = libp2p::kad::RecordKey::new(&key);
                self.store.store_mut().remove(&record_key);

                Ok((
                    Outcome::Success,
                    Effect::Deleted { key: key.to_vec() },
                ))
            }
        }
    }

    /// Create a frozen snapshot of the datastore
    fn freeze(&self, _context: &mut Self::Context) -> Self::Frozen {
        // TODO: Implement proper record iteration
        // The libp2p kad::Behaviour doesn't expose direct access to iterate all records
        // in the store. We need to either:
        // 1. Keep a separate index of all stored records
        // 2. Use kad queries to retrieve records (but this is async)
        // 3. Wait for libp2p API improvements
        //
        // For now, return an empty snapshot
        let entries: Vec<(Vec<u8>, Vec<u8>)> = vec![];

        Frozen {
            entries,
            metadata: HashMap::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Return the cluster membership at a given round
    fn cluster_at(&self, _round: std::num::NonZeroUsize) -> Vec<Self::Node> {
        // TODO: Implement proper cluster membership management
        // For now, return an empty vector
        // This should be replaced with actual membership logic based on discovered peers
        vec![]
    }
}

impl<D> PaxosBehaviour<D>
where
    D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static,
    D: netabase_store::convert::ToIVec + Clone + Serialize + Unpin,
    for<'d> D: Deserialize<'d>,
    <D as strum::IntoDiscriminant>::Discriminant:
        netabase_store::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as strum::IntoDiscriminant>::Discriminant:
        netabase_store::traits::definition::NetabaseKeyDiscriminant,
    <D as NetabaseDefinitionTrait>::Keys: netabase_store::convert::ToIVec + Clone + Serialize + Unpin,
    for<'d> <D as NetabaseDefinitionTrait>::Keys: Deserialize<'d>,
{
    /// Thaw a frozen snapshot back into the datastore
    pub fn thaw(&mut self, frozen: Frozen<D>) -> Result<(), StateError> {
        use libp2p::kad::store::RecordStore;

        // Iterate over all entries in the frozen snapshot and insert them
        for (key, value) in frozen.entries {
            let record = libp2p::kad::Record {
                key: libp2p::kad::RecordKey::new(&key),
                value,
                publisher: None,
                expires: None,
            };

            self.store
                .store_mut()
                .put(record)
                .map_err(|e| StateError::Storage(format!("{:?}", e)))?;
        }

        Ok(())
    }
}
