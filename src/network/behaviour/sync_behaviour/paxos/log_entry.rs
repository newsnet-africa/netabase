use netabase_store::traits::definition::NetabaseDefinitionTrait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Log entry operations that can be applied to the Kademlia datastore
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "D: Serialize + serde::de::DeserializeOwned")]
pub enum LogEntry<D>
where
    D: NetabaseDefinitionTrait,
{
    /// Create operation - inserts a new model into the datastore
    Create(D),
    /// Delete operation - removes a model from the datastore
    Delete(D),
}

impl<D> LogEntry<D>
where
    D: NetabaseDefinitionTrait,
{
    /// Returns a reference to the inner model
    pub fn model(&self) -> &D {
        match self {
            LogEntry::Create(model) | LogEntry::Delete(model) => model,
        }
    }

    /// Checks if this is a create operation
    pub fn is_create(&self) -> bool {
        matches!(self, LogEntry::Create(_))
    }

    /// Checks if this is a delete operation
    pub fn is_delete(&self) -> bool {
        matches!(self, LogEntry::Delete(_))
    }
}

// Implement paxakos::LogEntry trait
impl<D> paxakos::LogEntry for LogEntry<D>
where
    D: NetabaseDefinitionTrait + Clone + Send + Sync + Unpin + 'static,
    D: Serialize,
    for<'de> D: Deserialize<'de>,
{
    type Id = super::LogEntryId;

    fn id(&self) -> Self::Id {
        // Serialize the log entry and hash it to get a unique ID
        // Use bincode's serde compatibility layer for types that implement Serialize
        let bytes = bincode::serde::encode_to_vec(self, bincode::config::standard())
            .expect("Failed to serialize log entry");
        super::LogEntryId::from_bytes(&bytes)
    }
}
