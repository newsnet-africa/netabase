use serde::{Serialize, Deserialize, de::DeserializeOwned};
use netabase_store::{
    prelude::{NetabaseDefinition, NetabaseModel},
    traits::registery::models::NetabaseModelKeys,
};
use strum::IntoDiscriminant;
use crate::node::{
    capabilities::{Capability, CapabilityRange},
    primitives::Signature,
};

// =========================================================================
//  1. Database Queries (CRUD & Fetching)
// =========================================================================

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(bound = "M::Keys: NetabaseModelKeys<D, M>, <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned, <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned")]
pub enum DatabaseQuery<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
{
    /// Get a single record by its Primary Key
    Get {
        key: <M::Keys as NetabaseModelKeys<D, M>>::Primary,
    },
    
    /// Get records matching a Secondary Key value
    GetBySecondary {
        key: <M::Keys as NetabaseModelKeys<D, M>>::Secondary,
    },
    
    /// Check if a record exists (Lightweight HEAD request)
    Exists {
        key: <M::Keys as NetabaseModelKeys<D, M>>::Primary,
    },

    /// List records within a range (Start..End)
    Range {
        start: Option<<M::Keys as NetabaseModelKeys<D, M>>::Primary>,
        end: Option<<M::Keys as NetabaseModelKeys<D, M>>::Primary>,
        limit: Option<u32>,
    },
    
    /// Get specific blob data associated with a record
    GetBlob {
        key: <M::Keys as NetabaseModelKeys<D, M>>::Primary,
        field_index: u8, // Index of the blob field in the model
    },
}

// =========================================================================
//  2. Network Queries (Sync, Discovery, Metadata)
// =========================================================================

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(bound = "D::SubscriptionKeysDiscriminant: Serialize + DeserializeOwned")]
pub enum NetworkQuery<D: NetabaseDefinition> 
where 
    D::Discriminant: std::fmt::Debug + 'static 
{
    /// Request the Merkle Root hash for a specific subscription/model to check consistency
    GetMerkleRoot {
        subscription: D::SubscriptionKeysDiscriminant,
    },

    /// Request a sync bucket (for reconciliation)
    GetSyncBucket {
        subscription: D::SubscriptionKeysDiscriminant,
        bucket_index: u32,
    },

    /// Discovery: Ask a peer for other providers of this definition
    FindProviders {
        limit: u8,
    },
    
    /// Handshake/Auth: Challenge-Response initiation
    ChallengeRequest {
        nonce: u64,
    }
}

// =========================================================================
//  3. The Secure Envelope (Wire Format)
// =========================================================================

/// A wrapper that binds a Query to the Capability that authorizes it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(bound = "Capability<D, M>: Serialize + DeserializeOwned, Q: Serialize + DeserializeOwned")]
pub struct QueryEnvelope<D, M, Q>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: std::fmt::Debug + Serialize + DeserializeOwned + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    D::SubscriptionKeysDiscriminant: Serialize + DeserializeOwned + std::fmt::Debug + Clone + PartialEq + Eq,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    /// The actual query (Database or Network)
    pub payload: Q,

    /// The capability proving permission to execute this query
    /// Must be signed by the Data Owner (or a chain leading to them)
    pub proof: Capability<D, M>,

    /// A signature OF THIS ENVELOPE by the `proof.issued_to` (The Caller).
    /// This proves the caller currently holds the private key for the capability they are presenting.
    pub caller_signature: Signature,
    
    /// Replay protection
    pub nonce: u64,
    pub timestamp: u64,
}

// =========================================================================
//  4. Unified Query Enum
// =========================================================================

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(bound = "DatabaseQuery<D, M>: Serialize + DeserializeOwned, NetworkQuery<D>: Serialize + DeserializeOwned")]
pub enum QueryType<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    D::SubscriptionKeysDiscriminant: Serialize + DeserializeOwned + std::fmt::Debug + Clone + PartialEq + Eq,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    Database(DatabaseQuery<D, M>),
    Network(NetworkQuery<D>),
}

// =========================================================================
//  5. Query Results
// =========================================================================

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(bound = "M: Serialize + DeserializeOwned")]
pub enum DatabaseQueryResult<M> {
    Record(Option<M>),
    Exists(bool),
    Range(Vec<M>),
    Blob(Vec<u8>),
}