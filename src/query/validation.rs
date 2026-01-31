use netabase_store::{
    prelude::{NetabaseDefinition, NetabaseModel},
    traits::registery::models::NetabaseModelKeys,
};
use strum::IntoDiscriminant;
use serde::{Serialize, Deserialize, de::DeserializeOwned};

use crate::node::{
    capabilities::{Capability, CapabilityRange, PathRange, CapabilityPermission},
    primitives::Operation,
};

use super::messages::{DatabaseQuery, NetworkQuery, QueryType};

/// Errors that can occur during query validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    ExpiredCapability,
    InvalidSignature,
    InsufficientPermissions { required: Operation, actual: Operation },
    OutOfScope { required_key: String },
    UnsupportedQueryType,
}

/// Trait to extract validation requirements from a query
pub trait ValidateQuery<D, M>
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
    /// Check if this specific query is allowed by the provided capability
    fn validate(&self, capability: &Capability<D, M>) -> Result<(), ValidationError>;
}

// ----------------------------------------------------------------------------
// Validation for Database Queries
// ----------------------------------------------------------------------------

impl<D, M> ValidateQuery<D, M> for DatabaseQuery<D, M>
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
    fn validate(&self, capability: &Capability<D, M>) -> Result<(), ValidationError> {
        // 1. Check permissions (All DB queries here are Reads)
        // If we add Put/Delete, we'd switch on self here.
        let required_op = Operation::Read;
        let cap_op = capability.resource.operation_type();
        
        // Simple hierarchy: Read is base. Write implies Read? 
        // For strictness, let's assume Write implies Read in this logic, 
        // or the CapabilityPermission implementation handles `is_subset_of`.
        // But here we check the raw Operation enum.
        if cap_op != required_op && cap_op != Operation::Write && cap_op != Operation::Mint {
             return Err(ValidationError::InsufficientPermissions { 
                 required: required_op, 
                 actual: cap_op 
            });
        }

        // 2. Check Range/Scope
        let cap_range = capability.resource.range();
        
        match self {
            DatabaseQuery::Get { key } | DatabaseQuery::Exists { key } | DatabaseQuery::GetBlob { key, .. } => {
                validate_key_in_range(key, cap_range)
            },
            DatabaseQuery::Range { start, end, .. } => {
                // Both start and end must be in range
                if let Some(s) = start {
                    validate_key_in_range(s, cap_range)?;
                }
                if let Some(e) = end {
                    validate_key_in_range(e, cap_range)?;
                }
                Ok(())
            }
        }
    }
}

// Helper to check if a specific key is covered by the CapabilityRange
fn validate_key_in_range<D, M>(
    key: &<M::Keys as NetabaseModelKeys<D, M>>::Primary,
    range: &CapabilityRange<D, M>
) -> Result<(), ValidationError>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    match range {
        CapabilityRange::FullTable => Ok(()),
        CapabilityRange::PrimaryRange(path_range) => {
            match path_range {
                PathRange::PathPrefix(_) => {
                    // Prefix check requires key serialization or knowing the key structure matches prefix.
                    // For safety, we can conservatively allow if prefix is empty, or deny if we can't check.
                    // Ideally, we'd serialize 'key' to bytes and check starts_with.
                    // Since we have Serialize bound:
                    if let Ok(key_bytes) = netabase_store::postcard::to_allocvec(key) {
                        if let PathRange::PathPrefix(crate::store::primitives::EntryPath(prefix)) = path_range {
                            if key_bytes.starts_with(prefix) {
                                return Ok(());
                            }
                        }
                    }
                    Err(ValidationError::OutOfScope { required_key: format!("{:?}", key) })
                },
                PathRange::Range { start, end } => {
                    if key >= start && key <= end {
                        Ok(())
                    } else {
                        Err(ValidationError::OutOfScope { required_key: format!("{:?}", key) })
                    }
                }
            }
        },
        CapabilityRange::SecondaryRange(_) => {
            // Cannot validate Primary Key against Secondary Range without index lookup
            // For now, fail safe.
            Err(ValidationError::OutOfScope { required_key: "Cannot validate PK against Secondary Capability".into() })
        }
    }
}

// ----------------------------------------------------------------------------
// Validation for Network Queries
// ----------------------------------------------------------------------------

impl<D, M> ValidateQuery<D, M> for NetworkQuery<D>
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
    fn validate(&self, capability: &Capability<D, M>) -> Result<(), ValidationError> {
        // Network queries usually require broad access (like FullTable) or matching subscription
        match self {
            NetworkQuery::GetMerkleRoot { subscription } | NetworkQuery::GetSyncBucket { subscription, .. } => {
                // Must have Read permission
                // Must match subscription topic in capability (if cap is scoped to topic)
                if capability.subscription != *subscription {
                     return Err(ValidationError::OutOfScope { required_key: "Subscription Mismatch".into() });
                }
                Ok(())
            },
            NetworkQuery::FindProviders { .. } => Ok(()), // Generally public? Or requires minimal cap?
            NetworkQuery::ChallengeRequest { .. } => Ok(()), // Handshake is pre-auth usually
        }
    }
}
