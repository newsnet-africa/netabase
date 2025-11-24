use crate::network::store::NetabaseStore;
use libp2p::swarm::NetworkBehaviour;
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod communicator;
pub mod log_entry;
pub mod node_id;
pub mod state;

// Re-export key types
pub use node_id::NodeId;

// ============================================================================
// Newtypes for Paxos protocol fields
// ============================================================================

/// Round number in the Paxos protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RoundNum(pub u64);

impl RoundNum {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn next(&self) -> Self {
        Self(self.0.saturating_add(1))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

// Implement required traits for paxakos::Number
impl std::fmt::Display for RoundNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<RoundNum> for u128 {
    fn from(val: RoundNum) -> Self {
        val.0 as u128
    }
}

impl TryFrom<u128> for RoundNum {
    type Error = std::num::TryFromIntError;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        Ok(RoundNum(u64::try_from(value)?))
    }
}

impl TryFrom<usize> for RoundNum {
    type Error = std::num::TryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(RoundNum(u64::try_from(value)?))
    }
}

impl TryFrom<RoundNum> for usize {
    type Error = std::num::TryFromIntError;

    fn try_from(value: RoundNum) -> Result<Self, Self::Error> {
        usize::try_from(value.0)
    }
}

impl num_traits::Bounded for RoundNum {
    fn min_value() -> Self {
        RoundNum(u64::MIN)
    }

    fn max_value() -> Self {
        RoundNum(u64::MAX)
    }
}

impl num_traits::Num for RoundNum {
    type FromStrRadixErr = std::num::ParseIntError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        u64::from_str_radix(str, radix).map(RoundNum)
    }
}

impl num_traits::Zero for RoundNum {
    fn zero() -> Self {
        RoundNum(0)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl num_traits::One for RoundNum {
    fn one() -> Self {
        RoundNum(1)
    }
}

impl std::ops::Add for RoundNum {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        RoundNum(self.0.saturating_add(other.0))
    }
}

impl std::ops::Sub for RoundNum {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        RoundNum(self.0.saturating_sub(other.0))
    }
}

impl std::ops::Mul for RoundNum {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        RoundNum(self.0.saturating_mul(other.0))
    }
}

impl std::ops::Div for RoundNum {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        RoundNum(self.0 / other.0)
    }
}

impl std::ops::Rem for RoundNum {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        RoundNum(self.0 % other.0)
    }
}

/// Coordination number in the Paxos protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CoordNum(pub u64);

impl CoordNum {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn next(&self) -> Self {
        Self(self.0.saturating_add(1))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

// Implement required traits for paxakos::Number
impl std::fmt::Display for CoordNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<CoordNum> for u128 {
    fn from(val: CoordNum) -> Self {
        val.0 as u128
    }
}

impl TryFrom<u128> for CoordNum {
    type Error = std::num::TryFromIntError;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        Ok(CoordNum(u64::try_from(value)?))
    }
}

impl TryFrom<usize> for CoordNum {
    type Error = std::num::TryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(CoordNum(u64::try_from(value)?))
    }
}

impl TryFrom<CoordNum> for usize {
    type Error = std::num::TryFromIntError;

    fn try_from(value: CoordNum) -> Result<Self, Self::Error> {
        usize::try_from(value.0)
    }
}

impl num_traits::Bounded for CoordNum {
    fn min_value() -> Self {
        CoordNum(u64::MIN)
    }

    fn max_value() -> Self {
        CoordNum(u64::MAX)
    }
}

impl num_traits::Num for CoordNum {
    type FromStrRadixErr = std::num::ParseIntError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        u64::from_str_radix(str, radix).map(CoordNum)
    }
}

impl num_traits::Zero for CoordNum {
    fn zero() -> Self {
        CoordNum(0)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl num_traits::One for CoordNum {
    fn one() -> Self {
        CoordNum(1)
    }
}

impl std::ops::Add for CoordNum {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        CoordNum(self.0.saturating_add(other.0))
    }
}

impl std::ops::Sub for CoordNum {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        CoordNum(self.0.saturating_sub(other.0))
    }
}

impl std::ops::Mul for CoordNum {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        CoordNum(self.0.saturating_mul(other.0))
    }
}

impl std::ops::Div for CoordNum {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        CoordNum(self.0 / other.0)
    }
}

impl std::ops::Rem for CoordNum {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        CoordNum(self.0 % other.0)
    }
}

/// Log entry identifier using blake3 hash
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LogEntryId(pub blake3::Hash);

impl LogEntryId {
    pub fn new(hash: blake3::Hash) -> Self {
        Self(hash)
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(blake3::hash(bytes))
    }
}

// Manual PartialOrd/Ord implementations since blake3::Hash doesn't implement them
impl PartialOrd for LogEntryId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LogEntryId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.as_bytes().cmp(other.0.as_bytes())
    }
}

impl std::fmt::Display for LogEntryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_hex())
    }
}

// Custom Serialize/Deserialize for LogEntryId since blake3::Hash doesn't implement them
impl Serialize for LogEntryId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.0.as_bytes())
    }
}

impl<'de> Deserialize<'de> for LogEntryId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BytesVisitor;

        impl<'de> serde::de::Visitor<'de> for BytesVisitor {
            type Value = LogEntryId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a byte array of length 32")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.len() != 32 {
                    return Err(E::custom(format!("expected 32 bytes, got {}", v.len())));
                }
                let mut bytes = [0u8; 32];
                bytes.copy_from_slice(v);
                Ok(LogEntryId(blake3::Hash::from(bytes)))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut bytes = [0u8; 32];
                for (i, byte) in bytes.iter_mut().enumerate() {
                    *byte = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::custom(format!("expected 32 bytes, got {}", i)))?;
                }
                Ok(LogEntryId(blake3::Hash::from(bytes)))
            }
        }

        deserializer.deserialize_bytes(BytesVisitor)
    }
}

// LogEntryId gets Identifier automatically from paxakos's blanket impl
// since it implements Copy + Debug + Eq + Hash + Ord + Send + Sync + Unpin

/// Abstain vote message information (node chose not to vote)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbstainMsg;

/// Yea vote message information (affirmative vote)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct YeaMsg;

/// Nay vote message information (negative vote)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NayMsg;

// ============================================================================
// Paxos protocol messages
// ============================================================================

/// Paxos protocol request messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "E: Serialize", deserialize = "E: Deserialize<'de>"))]
pub enum PaxosRequest<E> {
    /// Prepare phase - leader election
    Prepare {
        round_num: RoundNum,
        coord_num: CoordNum,
    },
    /// Propose phase - propose a log entry
    Propose {
        round_num: RoundNum,
        coord_num: CoordNum,
        #[serde(with = "serde_arc")]
        log_entry: Arc<E>,
    },
    /// Commit phase - commit a log entry
    Commit {
        round_num: RoundNum,
        coord_num: CoordNum,
        #[serde(with = "serde_arc")]
        log_entry: Arc<E>,
    },
    /// Commit by ID phase - commit using log entry ID
    CommitById {
        round_num: RoundNum,
        coord_num: CoordNum,
        log_entry_id: LogEntryId,
    },
}

// Custom serialization module for Arc
mod serde_arc {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::sync::Arc;

    pub fn serialize<S, T>(arc: &Arc<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        arc.as_ref().serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Arc<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        T::deserialize(deserializer).map(Arc::new)
    }
}

/// Paxos protocol response messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "E: Serialize", deserialize = "E: Deserialize<'de>"))]
pub enum PaxosResponse<E> {
    /// Vote response for Prepare requests
    Vote(VoteMsg<E>),
    /// Acceptance response for Propose requests
    Acceptance(AcceptanceMsg<E>),
    /// Committed response for Commit/CommitById requests
    Committed,
}

/// Vote message response to a prepare request (leader election)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "E: Serialize", deserialize = "E: Deserialize<'de>"))]
pub enum VoteMsg<E> {
    /// The node voted for the candidate and provides a promise
    Given(PromiseMsg<E>),
    /// The node couldn't vote due to a conflict
    Conflicted(ConflictMsg<E>),
    /// The node abstained from voting
    Abstained(AbstainMsg),
}

/// Acceptance message response to a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "E: Serialize", deserialize = "E: Deserialize<'de>"))]
pub enum AcceptanceMsg<E> {
    /// The node accepted the proposal (Yea vote)
    Given(YeaMsg),
    /// The node couldn't accept due to a conflict
    Conflicted(ConflictMsg<E>),
    /// The node rejected the proposal (Nay vote)
    Refused(NayMsg),
}

/// A promise message not to accept certain proposals
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "E: Serialize", deserialize = "E: Deserialize<'de>"))]
pub struct PromiseMsg<E> {
    /// Conditions that must be honored
    pub conditions: Vec<ConditionMsg<E>>,
}

/// A condition message in a promise
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "E: Serialize", deserialize = "E: Deserialize<'de>"))]
pub struct ConditionMsg<E> {
    pub round_num: RoundNum,
    pub coord_num: CoordNum,
    #[serde(with = "serde_arc")]
    pub log_entry: Arc<E>,
}

/// Conflict message response indicating why a request was rejected
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "E: Serialize", deserialize = "E: Deserialize<'de>"))]
pub enum ConflictMsg<E> {
    /// Another node was elected leader with a higher coordination number
    Supplanted { coord_num: CoordNum },
    /// The round already converged on a log entry
    Converged {
        coord_num: CoordNum,
        #[serde(
            with = "option_tuple_arc",
            skip_serializing_if = "Option::is_none",
            default
        )]
        log_entry: Option<(CoordNum, Arc<E>)>,
    },
}

// Custom serialization for Option<(CoordNum, Arc<E>)>
mod option_tuple_arc {
    use super::CoordNum;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::sync::Arc;

    pub fn serialize<S, E>(
        opt: &Option<(CoordNum, Arc<E>)>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        E: Serialize,
    {
        match opt {
            Some((c, arc)) => Some((c, arc.as_ref())).serialize(serializer),
            None => None::<()>.serialize(serializer),
        }
    }

    pub fn deserialize<'de, D, E>(deserializer: D) -> Result<Option<(CoordNum, Arc<E>)>, D::Error>
    where
        D: Deserializer<'de>,
        E: Deserialize<'de>,
    {
        Option::<(CoordNum, E)>::deserialize(deserializer)
            .map(|opt| opt.map(|(c, e)| (c, Arc::new(e))))
    }
}

// ============================================================================
// PaxosBehaviour implementation
// ============================================================================

/// Paxos-enhanced behaviour that wraps Kademlia with consensus
///
/// The node_id is stored in a separate lazy_static to avoid NetworkBehaviour
/// derive macro issues with ignored fields.
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "PaxosEvent")]
pub struct PaxosBehaviour<D: NetabaseDefinitionTrait + Send + Sync + 'static>
{
    #[behaviour(rename = "store")]
    pub(crate) store: libp2p::kad::Behaviour<NetabaseStore<D>>,
    // TODO: Add request_response behaviour for Paxos messages
    // #[behaviour(rename = "request_response")]
    // request_response: libp2p::request_response::cbor::Behaviour<PaxosRequest<D>, PaxosResponse<D>>,
}

/// Event type for PaxosBehaviour
#[derive(Debug)]
pub enum PaxosEvent {
    Store(libp2p::kad::Event),
    // RequestResponse(libp2p::request_response::Event<PaxosRequest<D>, PaxosResponse<D>>),
}

// Manual Debug implementation since NetworkBehaviour doesn't auto-derive it
impl<D: NetabaseDefinitionTrait + Send + Sync + 'static> std::fmt::Debug for PaxosBehaviour<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PaxosBehaviour")
            .field("node_id", &self.node_id())
            .finish_non_exhaustive()
    }
}

impl From<libp2p::kad::Event> for PaxosEvent {
    fn from(event: libp2p::kad::Event) -> Self {
        PaxosEvent::Store(event)
    }
}

// Store node IDs separately to avoid derive macro issues
use std::sync::RwLock;
use std::collections::HashMap;
use once_cell::sync::Lazy;

static NODE_IDS: Lazy<RwLock<HashMap<String, NodeId>>> = Lazy::new(|| RwLock::new(HashMap::new()));

fn store_node_id(key: &str, node_id: NodeId) {
    NODE_IDS.write().unwrap().insert(key.to_string(), node_id);
}

fn get_node_id(key: &str) -> Option<NodeId> {
    NODE_IDS.read().unwrap().get(key).copied()
}

impl<D> PaxosBehaviour<D>
where
    D: NetabaseDefinitionTrait + Send + Sync + 'static,
{
    /// Create a new PaxosBehaviour
    pub fn new(
        node_id: NodeId,
        store: libp2p::kad::Behaviour<NetabaseStore<D>>,
    ) -> Self {
        // Store the node_id in our static map keyed by the peer_id
        let peer_id_str = node_id.peer_id().to_string();
        store_node_id(&peer_id_str, node_id);

        Self {
            store,
        }
    }

    /// Get the local node ID
    ///
    /// Note: This retrieves the NodeId that was stored during construction.
    /// Since libp2p's kad::Behaviour no longer exposes local_peer_id(),
    /// we retrieve it from our static storage.
    pub fn node_id(&self) -> NodeId {
        // For simplicity, we just get the stored NodeId
        // In practice, there's typically only one PaxosBehaviour per process
        NODE_IDS.read()
            .unwrap()
            .values()
            .next()
            .copied()
            .expect("NodeId should have been stored during construction")
    }

    /// Get the local peer ID (for compatibility with libp2p)
    pub fn peer_id(&self) -> libp2p::PeerId {
        self.node_id().peer_id()
    }

    // ========================================================================
    // TODO: Paxos helper methods will be implemented when request_response
    // behaviour is added. For now, all communication goes through the
    // Communicator trait implementation in communicator.rs
    // ========================================================================
}

// ============================================================================
// Error types
// ============================================================================

/// Error that can occur during commit operations
#[derive(Debug, thiserror::Error)]
pub enum CommitError {
    #[error("Log entry not found: {0}")]
    EntryNotFound(LogEntryId),

    #[error("Invalid round number: expected {expected:?}, got {actual:?}")]
    InvalidRound {
        expected: RoundNum,
        actual: RoundNum,
    },

    #[error("Invalid coordination number: expected {expected:?}, got {actual:?}")]
    InvalidCoordination {
        expected: CoordNum,
        actual: CoordNum,
    },

    #[error("Storage error: {0}")]
    Storage(String),
}
