use serde::{Serialize, Deserialize, de::DeserializeOwned};
use libp2p::PeerId;
use netabase_store::{
    prelude::{NetabaseDefinition, NetabaseModel},
    traits::registery::models::NetabaseModelKeys,
};
use strum::IntoDiscriminant;

use crate::node::primitives::{NamespaceId, Operation, Signature, SubspaceId};
use crate::node::metadata::{PublicNodeData, NodePublicKey};
use crate::store::primitives::EntryPath;
use crate::node::key::NetabasePath;

pub type CapabilityExpiration = u64;
pub type CapabilitySignature = Signature;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(bound = "K: Serialize + DeserializeOwned")]
pub enum PathRange<K> {
    PathPrefix(EntryPath),
    Range {
        start: K,
        end: K
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(bound = "M::Keys: NetabaseModelKeys<D, M>, <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned, <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned")]
pub enum CapabilityRange<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    FullTable,
    PrimaryRange(PathRange<<M::Keys as NetabaseModelKeys<D, M>>::Primary>),
    SecondaryRange(PathRange<<M::Keys as NetabaseModelKeys<D, M>>::Secondary>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(bound = "CapabilityRange<D, M>: Serialize + DeserializeOwned")]
pub enum CapabilityPermission<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    Read(CapabilityRange<D, M>),
    Write(CapabilityRange<D, M>),
    Mint(CapabilityRange<D, M>),
    Store(CapabilityRange<D, M>),
}

impl<D, M> CapabilityPermission<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    pub fn range(&self) -> &CapabilityRange<D, M> {
        match self {
            Self::Read(r) | Self::Write(r) | Self::Mint(r) | Self::Store(r) => r,
        }
    }
    
    pub fn operation_type(&self) -> Operation {
        match self {
            Self::Read(_) => Operation::Read,
            Self::Write(_) => Operation::Write,
            Self::Mint(_) => Operation::Mint,
            Self::Store(_) => Operation::Store,
        }
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Read(a), Self::Read(b)) => a.is_subset_of(b),
            (Self::Write(a), Self::Write(b)) => a.is_subset_of(b),
            (Self::Mint(a), Self::Mint(b)) => a.is_subset_of(b),
            (Self::Store(a), Self::Store(b)) => a.is_subset_of(b),
            (Self::Read(a), Self::Write(b)) => a.is_subset_of(b),
            _ => false,
        }
    }
}

impl<D, M> CapabilityRange<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    pub fn is_subset_of(&self, other: &Self) -> bool {
        match (other, self) {
            (Self::FullTable, _) => true,
            (Self::PrimaryRange(parent_r), Self::PrimaryRange(child_r)) => {
                parent_r.includes(child_r)
            },
            (Self::SecondaryRange(parent_r), Self::SecondaryRange(child_r)) => {
                parent_r.includes(child_r)
            },
            _ => false,
        }
    }
}

impl<K: PartialOrd + Eq> PathRange<K> {
    pub fn includes(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::PathPrefix(p1), Self::PathPrefix(p2)) => {
                p2.0.starts_with(&p1.0)
            },
            (Self::PathPrefix(p1), Self::Range { .. }) => {
                if p1.0.is_empty() { return true; }
                false 
            },
            (Self::Range { start: s1, end: e1 }, Self::Range { start: s2, end: e2 }) => {
                s1 <= s2 && e1 >= e2
            },
            _ => false,
        }
    }
}

/// Capability with Type Safety
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(bound = "CapabilityPermission<D, M>: Serialize + DeserializeOwned")]
pub struct Capability<D, M>
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
    pub subscription: D::SubscriptionKeysDiscriminant,
    pub owner: PublicNodeData,
    pub granted_by: PublicNodeData,
    pub issued_to: PublicNodeData,
    pub resource: CapabilityPermission<D, M>,
    pub expiry: CapabilityExpiration,
    pub signature: CapabilitySignature,
    
    // Chain support
    pub delegation: Option<Box<Capability<D, M>>>,
}

impl<D, M> Capability<D, M>
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
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.expiry < now
    }

    pub fn verify_chain(&self, root_owner_key: &NodePublicKey) -> bool {
        if self.is_expired() {
            return false;
        }

        if &self.owner.public_key != root_owner_key {
             return false;
        }

        if let Some(parent) = &self.delegation {
            if !parent.verify_chain(root_owner_key) {
                return false;
            }

            if parent.owner != self.owner {
                return false;
            }

            if parent.issued_to != self.granted_by {
                return false;
            }

            if !self.resource.is_subset_of(&parent.resource) {
                return false;
            }

            true
        } else {
            if self.granted_by != self.owner {
                return false;
            }
            true
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl<D, M> NetabasePath for CapabilityRange<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    fn to_path(&self) -> Vec<u8> {
        match self {
            CapabilityRange::FullTable => Vec::new(),
            CapabilityRange::PrimaryRange(PathRange::PathPrefix(p)) => p.0.clone(),
            CapabilityRange::PrimaryRange(PathRange::Range { start, .. }) => {
                 if let Ok(p_bytes) = netabase_store::postcard::to_allocvec(start) {
                     p_bytes
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        }
    }
}