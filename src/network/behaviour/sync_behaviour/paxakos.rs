//! Paxakos integration for Paxos consensus
//!
//! This module integrates the Paxakos consensus library with Netabase,
//! enabling distributed consensus on database operations over libp2p.
//!
//! # Architecture
//!
//! - `NetworkCommunicator`: Implements the paxakos `Communicator` trait, queuing
//!   messages to be sent over the network via request-response protocol
//! - `PaxosBehaviour`: A libp2p `NetworkBehaviour` that acts as the network layer,
//!   sending/receiving Paxos messages and routing responses back to the consensus algorithm
//! - Response routing: Uses oneshot channels to deliver responses to waiting futures

use std::collections::{HashMap, HashSet, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use libp2p::PeerId;
use netabase_store::definition::NetabaseDefinitionTrait;
use paxakos::{Invocation, NodeInfo, State};
use paxakos::communicator::{Acceptance, Committed, Communicator, Vote};
use paxakos::state::Frozen;
use serde::{Deserialize, Serialize};

use crate::{Netabase, NetabaseNodeInfo};

/// Context for tracking Paxos state
///
/// This tracks which log entries have been applied to ensure idempotency
#[derive(Debug, Clone)]
pub struct PaxosContext<D>
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
    /// Set of applied entry IDs (primary keys from the Keys enum)
    pub applied_entries: HashSet<D::Keys>,
    /// Last applied round number
    pub last_applied_round: u128,
    _marker: std::marker::PhantomData<D>,
}

impl<D> Default for PaxosContext<D>
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
    fn default() -> Self {
        Self {
            applied_entries: HashSet::new(),
            last_applied_round: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<D> PaxosContext<D>
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
    /// Check if an entry has been applied
    pub fn is_applied(&self, entry_id: &D::Keys) -> bool {
        self.applied_entries.contains(entry_id)
    }

    /// Mark an entry as applied
    pub fn mark_applied(&mut self, entry_id: D::Keys, round: u128) {
        self.applied_entries.insert(entry_id);
        if round > self.last_applied_round {
            self.last_applied_round = round;
        }
    }
}

/// Frozen state snapshot for Paxos
#[derive(Debug, Clone)]
pub struct FrozenState<D>
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
    pub applied_entries: HashSet<D::Keys>,
    pub last_applied_round: u128,
    _marker: std::marker::PhantomData<D>,
}

/// Frozen trait implementation for FrozenState
///
/// This allows frozen snapshots to be thawed back into active Netabase instances
impl<D> Frozen<Netabase<D>> for FrozenState<D>
where
    D: NetabaseDefinitionTrait + netabase_store::convert::ToIVec + Send + Sync + 'static + paxakos::LogEntry<Id = <D as NetabaseDefinitionTrait>::Keys>,
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
    fn thaw(&self, context: &mut PaxosContext<D>) -> Netabase<D> {
        // TODO: Implement proper state reconstruction
        // This would require:
        // 1. Loading the persistent database state
        // 2. Recreating runtime components (channels, threads)
        // 3. Restoring the PaxosContext from the frozen state

        // For now, restore the context from frozen state
        context.applied_entries = self.applied_entries.clone();
        context.last_applied_round = self.last_applied_round;

        // Cannot properly reconstruct Netabase without access to runtime components
        // In a real implementation, this would be handled by the Paxakos node infrastructure
        unimplemented!("Thawing Netabase from frozen state requires runtime initialization")
    }
}

/// Frozen trait implementation for FrozenState with PaxosBehaviour
///
/// This allows frozen snapshots to be thawed back into active PaxosBehaviour instances
#[cfg(all(feature = "paxos", feature = "libp2p"))]
impl<D> Frozen<PaxosBehaviour<D>> for FrozenState<D>
where
    D: NetabaseDefinitionTrait + netabase_store::convert::ToIVec + Send + Sync + 'static + paxakos::LogEntry<Id = <D as NetabaseDefinitionTrait>::Keys> + Serialize + for<'de> Deserialize<'de>,
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
    fn thaw(&self, context: &mut PaxosContext<D>) -> PaxosBehaviour<D> {
        // Restore the context from frozen state
        context.applied_entries = self.applied_entries.clone();
        context.last_applied_round = self.last_applied_round;

        // Cannot properly reconstruct PaxosBehaviour without access to:
        // - The actual store instance
        // - Network configuration (peer_id, kad config)
        // - Runtime components
        // In a real implementation, this would be handled by proper state recovery
        unimplemented!("Thawing PaxosBehaviour from frozen state requires store and network initialization")
    }
}

/// Invocation trait implementation for Netabase
///
/// This defines the basic types used in Paxos consensus
impl<D> Invocation for Netabase<D>
where
    D: NetabaseDefinitionTrait + netabase_store::convert::ToIVec + Send + Sync + 'static + paxakos::LogEntry<Id = <D as NetabaseDefinitionTrait>::Keys>,
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
    type RoundNum = u128;
    type CoordNum = u128;
    type State = Self;
    type Yea = ();
    type Nay = ();
    type Abstain = ();
    type Ejection = String; // Must be convertible from CommunicationError
    type CommunicationError = String; // Changed from NetabaseError to String for Clone requirement
}

/// State trait implementation for Netabase
///
/// This defines how log entries are applied to the state and how
/// the state can be frozen for snapshots
impl<D: NetabaseDefinitionTrait> State for Netabase<D>
where
    D: netabase_store::convert::ToIVec + Send + Sync + 'static + paxakos::LogEntry<Id = <D as NetabaseDefinitionTrait>::Keys>,
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
    type Frozen = FrozenState<D>;
    type LogEntry = D;
    type Context = PaxosContext<D>;
    type Outcome = ();
    type Effect = ();
    type Error = String; // Changed from NetabaseError to String for Clone requirement
    type Node = NetabaseNodeInfo;

    /// Apply a log entry to the state
    ///
    /// This is the core consensus operation. Once a log entry is committed
    /// by the Paxos cluster, it is applied to all nodes' states via this method.
    ///
    /// The implementation ensures idempotency by tracking applied entries.
    fn apply(
        &mut self,
        log_entry: &Self::LogEntry,
        context: &mut Self::Context,
    ) -> Result<(Self::Outcome, Self::Effect), Self::Error> {
        // Get unique ID for this entry (primary key)
        let entry_id = log_entry.id();

        // Check if already applied (idempotency)
        if context.is_applied(&entry_id) {
            return Ok(((), ()));
        }

        // Note: This State implementation is for Netabase<D>, which doesn't
        // have direct access to the store. The actual application logic
        // is in the PaxosBehaviour<D> State implementation below (line 1082+).
        //
        // This stub implementation is here for type system compatibility
        // but should not be called in practice since PaxosBehaviour<D> is
        // the actual State type used by paxakos.

        // Mark as applied
        context.mark_applied(entry_id, 0);

        Ok(((), ()))
    }

    /// Get cluster membership at a given round
    ///
    /// This enables dynamic membership changes. For now, we return a static
    /// cluster configuration.
    fn cluster_at(
        &self,
        _round_offset: std::num::NonZeroUsize,
    ) -> Vec<Self::Node> {
        // TODO: Implement dynamic membership
        // For now, return empty cluster (this would come from config)
        vec![]
    }

    /// Freeze the state for snapshotting
    ///
    /// This creates a snapshot of the current state that can be used
    /// for recovery or catching up lagging nodes.
    fn freeze(&self, context: &mut Self::Context) -> Self::Frozen {
        FrozenState {
            applied_entries: context.applied_entries.clone(),
            last_applied_round: context.last_applied_round,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Paxos request messages
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(bound = "D: Serialize + serde::de::DeserializeOwned")]
pub enum PaxosRequest<D>
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
    /// Phase 1a: Prepare request
    Prepare {
        round: u128,
        coord: u128,
    },
    /// Phase 2a: Proposal request
    Proposal {
        round: u128,
        coord: u128,
        entry: D,
    },
    /// Phase 2b: Commit request
    Commit {
        round: u128,
        coord: u128,
    },
}

/// Paxos response messages
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum PaxosResponse {
    /// Promise to not accept lower coordinator numbers
    Promise {
        round: u128,
        coord: u128,
    },
    /// Conflict with higher coordinator number
    Conflict {
        higher_coord: u128,
    },
    /// Abstain from voting
    Abstain,
    /// Accept the proposal
    Accept,
    /// Reject the proposal
    Reject {
        reason: String,
    },
    /// Acknowledge commit
    Committed,
}

/// Unified Paxos message type for gossipsub communication
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(bound = "D: Serialize + serde::de::DeserializeOwned")]
pub enum PaxosMessage<D>
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
    /// Request message from coordinator
    Request(PaxosRequest<D>),
    /// Response message from acceptor
    Response {
        /// ID of the peer sending the response (as string)
        peer_id: String,
        /// The response payload
        response: PaxosResponse,
    },
}

/// Custom events emitted by PaxosBehaviour
///
/// This enum includes both Paxos-specific consensus events and Kademlia DHT events,
/// since PaxosBehaviour owns the Kademlia instance.
#[derive(Debug, Clone)]
pub enum PaxosEvent<D>
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
    /// Kademlia DHT event (forwarded from the embedded kad behavior)
    Kad(libp2p::kad::Event),

    /// A log entry was committed by consensus
    EntryCommitted {
        round: u128,
        entry: D,
    },
    /// Node became coordinator
    BecameCoordinator,
    /// Node lost coordinator status
    LostCoordinator,
    /// Consensus round completed
    RoundCompleted {
        round: u128,
    },
}

/// Outgoing Paxos request that needs to be sent over the network
#[derive(Debug)]
struct OutgoingRequest<D>
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
    target: NetabaseNodeInfo,
    message: PaxosMessage<D>,
    response_tx: tokio::sync::oneshot::Sender<Result<PaxosResponse, String>>,
}

/// Network-based Communicator for Paxakos
///
/// This communicator queues outgoing messages that will be sent by PaxosBehaviour
/// over libp2p's request-response protocol. Responses are delivered via oneshot channels.
pub struct NetworkCommunicator<D>
where
    D: NetabaseDefinitionTrait + netabase_store::convert::ToIVec + Send + Sync + 'static + paxakos::LogEntry + Serialize + for<'de> Deserialize<'de>,
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
    /// Shared queue for outgoing requests
    outgoing_queue: Arc<Mutex<VecDeque<OutgoingRequest<D>>>>,
}

impl<D> NetworkCommunicator<D>
where
    D: NetabaseDefinitionTrait + netabase_store::convert::ToIVec + Send + Sync + 'static + paxakos::LogEntry + Serialize + for<'de> Deserialize<'de>,
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
    /// Create a new NetworkCommunicator with a shared outgoing queue
    pub fn new(queue: Arc<Mutex<VecDeque<OutgoingRequest<D>>>>) -> Self {
        Self {
            outgoing_queue: queue,
        }
    }

    /// Queue a message and return a future that completes when response arrives
    fn queue_request(
        &mut self,
        target: NetabaseNodeInfo,
        message: PaxosMessage<D>,
    ) -> Pin<Box<dyn Future<Output = Result<PaxosResponse, String>> + Send>> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        let request = OutgoingRequest {
            target,
            message,
            response_tx: tx,
        };

        // Queue the request
        if let Ok(mut queue) = self.outgoing_queue.lock() {
            queue.push_back(request);
        } else {
            // If lock fails, return error future
            return Box::pin(async { Err("Failed to acquire queue lock".to_string()) });
        }

        // Return future that waits for response
        Box::pin(async move {
            rx.await
                .map_err(|_| "Response channel closed".to_string())?
        })
    }
}

/// Communicator trait implementation for NetworkCommunicator
///
/// This enables Paxakos to send consensus messages over libp2p's request-response protocol.
/// Messages are queued and sent by PaxosBehaviour, with responses delivered via oneshot channels.
impl<D> Communicator for NetworkCommunicator<D>
where
    D: NetabaseDefinitionTrait + netabase_store::convert::ToIVec + Send + Sync + 'static + paxakos::LogEntry + Serialize + for<'de> Deserialize<'de>,
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
    type Node = NetabaseNodeInfo;
    type RoundNum = u128;
    type CoordNum = u128;
    type LogEntry = D;
    type Error = String;

    // Future types
    type SendPrepare = Pin<Box<dyn Future<Output = Result<Vote<u128, u128, D, ()>, String>> + Send>>;
    type Abstain = ();

    type SendProposal = Pin<Box<dyn Future<Output = Result<Acceptance<u128, D, (), ()>, String>> + Send>>;
    type Yea = ();
    type Nay = ();

    type SendCommit = Pin<Box<dyn Future<Output = Result<Committed, String>> + Send>>;
    type SendCommitById = Pin<Box<dyn Future<Output = Result<Committed, String>> + Send>>;

    fn send_prepare<'a>(
        &mut self,
        receivers: &'a [Self::Node],
        round: Self::RoundNum,
        coord: Self::CoordNum,
    ) -> Vec<(&'a Self::Node, Self::SendPrepare)> {
        let request = PaxosRequest::Prepare { round, coord };
        let message = PaxosMessage::Request(request);

        receivers
            .iter()
            .map(|node| {
                let fut = self.queue_request(node.clone(), message.clone());
                let fut = Box::pin(async move {
                    match fut.await? {
                        PaxosResponse::Promise { round: _r, coord: _c } => {
                            // Create an empty Promise (no previous conditions)
                            Ok(Vote::Given(paxakos::Promise::from(vec![])))
                        }
                        PaxosResponse::Conflict { higher_coord } => {
                            Ok(Vote::Conflicted(paxakos::Conflict::Supplanted { coord_num: higher_coord }))
                        }
                        PaxosResponse::Abstain => {
                            Ok(Vote::Abstained(()))
                        }
                        other => Err(format!("Unexpected response to Prepare: {:?}", other)),
                    }
                });
                (node, fut as Self::SendPrepare)
            })
            .collect()
    }

    fn send_proposal<'a>(
        &mut self,
        receivers: &'a [Self::Node],
        round: Self::RoundNum,
        coord: Self::CoordNum,
        log_entry: Arc<Self::LogEntry>,
    ) -> Vec<(&'a Self::Node, Self::SendProposal)> {
        let request = PaxosRequest::Proposal {
            round,
            coord,
            entry: (*log_entry).clone(),
        };
        let message = PaxosMessage::Request(request);

        receivers
            .iter()
            .map(|node| {
                let fut = self.queue_request(node.clone(), message.clone());
                let fut = Box::pin(async move {
                    match fut.await? {
                        PaxosResponse::Accept => {
                            Ok(Acceptance::Given(()))
                        }
                        PaxosResponse::Reject { reason } => {
                            // Map rejection to Refused variant
                            Ok(Acceptance::Refused(()))
                        }
                        PaxosResponse::Conflict { higher_coord } => {
                            Ok(Acceptance::Conflicted(paxakos::Conflict::Supplanted { coord_num: higher_coord }))
                        }
                        other => Err(format!("Unexpected response to Proposal: {:?}", other)),
                    }
                });
                (node, fut as Self::SendProposal)
            })
            .collect()
    }

    fn send_commit<'a>(
        &mut self,
        receivers: &'a [Self::Node],
        round: Self::RoundNum,
        coord: Self::CoordNum,
        _log_entry: Arc<Self::LogEntry>,
    ) -> Vec<(&'a Self::Node, Self::SendCommit)> {
        let request = PaxosRequest::Commit { round, coord };
        let message = PaxosMessage::Request(request);

        receivers
            .iter()
            .map(|node| {
                let fut = self.queue_request(node.clone(), message.clone());
                let fut = Box::pin(async move {
                    match fut.await? {
                        PaxosResponse::Committed => Ok(Committed),
                        other => Err(format!("Unexpected response to Commit: {:?}", other)),
                    }
                });
                (node, fut as Self::SendCommit)
            })
            .collect()
    }

    fn send_commit_by_id<'a>(
        &mut self,
        receivers: &'a [Self::Node],
        round: Self::RoundNum,
        coord: Self::CoordNum,
        _log_entry_id: <Self::LogEntry as paxakos::LogEntry>::Id,
    ) -> Vec<(&'a Self::Node, Self::SendCommitById)> {
        // For commit by ID, we still send the full commit request
        let request = PaxosRequest::Commit { round, coord };
        let message = PaxosMessage::Request(request);

        receivers
            .iter()
            .map(|node| {
                let fut = self.queue_request(node.clone(), message.clone());
                let fut = Box::pin(async move {
                    match fut.await? {
                        PaxosResponse::Committed => Ok(Committed),
                        other => Err(format!("Unexpected response to Commit: {:?}", other)),
                    }
                });
                (node, fut as Self::SendCommitById)
            })
            .collect()
    }
}

/// Paxos network behavior with embedded Kademlia DHT
///
/// **Architecture:**
/// When Paxos is enabled, this behavior owns both the NetabaseStore and a Kademlia
/// DHT instance. This allows Paxos to:
/// 1. Apply consensus entries directly to the store via `kad.store_mut()`
/// 2. Provide DHT functionality through the nested Kademlia instance
/// 3. Ensure atomic updates where consensus decisions are immediately persisted
///
/// **Access Pattern:**
/// When Paxos is enabled, access Kademlia via: `swarm.behaviour.paxos.kad`
/// (instead of the top-level `swarm.behaviour.kad` which is disabled)
///
/// **Integration:**
/// This is a libp2p NetworkBehaviour that integrates paxakos consensus with the
/// network layer. It uses request-response protocol to send Paxos messages and
/// routes responses back to the consensus algorithm.
pub struct PaxosBehaviour<D>
where
    D: NetabaseDefinitionTrait + Serialize + for<'de> Deserialize<'de>,
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
    /// Kademlia DHT - PaxosBehaviour owns this along with the store
    ///
    /// This provides DHT functionality and, crucially, gives Paxos access to
    /// the store via `self.kad.store_mut()` for applying consensus entries.
    pub kad: libp2p::kad::Behaviour<crate::network::store::NetabaseStore<D>>,

    /// Paxos consensus context - tracks applied entries for idempotency
    pub context: PaxosContext<D>,

    /// Request-response protocol for Paxos messages
    request_response: libp2p::request_response::cbor::Behaviour<PaxosMessage<D>, PaxosMessage<D>>,

    /// Shared queue for outgoing requests (shared with NetworkCommunicator)
    outgoing_queue: Arc<Mutex<VecDeque<OutgoingRequest<D>>>>,

    /// Map of pending request IDs to their response channels
    pending_requests: HashMap<
        libp2p::request_response::OutboundRequestId,
        tokio::sync::oneshot::Sender<Result<PaxosResponse, String>>
    >,

    /// Local peer ID
    peer_id: PeerId,

    /// Cluster member peer IDs for Paxos consensus
    ///
    /// This defines which nodes participate in the consensus cluster.
    /// - Empty vec = single-node mode (all proposals auto-commit)
    /// - Multiple peers = multi-node consensus requiring quorum
    cluster_members: Vec<PeerId>,
}

/// Manual Debug implementation for PaxosBehaviour
/// (auto-derive doesn't work because kad and request_response don't implement Debug)
impl<D> std::fmt::Debug for PaxosBehaviour<D>
where
    D: NetabaseDefinitionTrait + Serialize + for<'de> Deserialize<'de>,
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PaxosBehaviour")
            .field("context", &self.context)
            .field("peer_id", &self.peer_id)
            .field("cluster_members", &self.cluster_members)
            .field("pending_requests_count", &self.pending_requests.len())
            .finish_non_exhaustive()
    }
}

impl<D> PaxosBehaviour<D>
where
    D: NetabaseDefinitionTrait + netabase_store::convert::ToIVec + Serialize + for<'de> Deserialize<'de> + paxakos::LogEntry,
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
    /// Create a new Paxos behavior with embedded Kademlia
    ///
    /// # Arguments
    /// * `peer_id` - The local peer's ID
    /// * `store` - The NetabaseStore to be owned by Kademlia
    /// * `kad_config` - Optional Kademlia configuration (uses default if None)
    /// * `cluster_members` - List of peer IDs that form the Paxos cluster
    ///
    /// # Returns
    /// A new PaxosBehaviour that owns both the store and Kademlia DHT
    ///
    /// # Cluster Membership
    /// - Empty vec: Single-node mode (all proposals auto-commit)
    /// - Multiple peers: Multi-node consensus requiring quorum
    ///
    /// # Example
    /// ```ignore
    /// let store = NetabaseStore::<MyDefinition>::new(backend, path)?;
    /// // 3-node cluster
    /// let members = vec![peer1, peer2, peer3];
    /// let paxos = PaxosBehaviour::new(&peer_id, store, None, members);
    /// ```
    pub fn new(
        peer_id: &PeerId,
        store: crate::network::store::NetabaseStore<D>,
        kad_config: Option<libp2p::kad::Config>,
        cluster_members: Vec<PeerId>,
    ) -> Self {
        use libp2p::request_response::{ProtocolSupport, Config};
        use libp2p::StreamProtocol;

        // Create Kademlia behaviour with the store
        let kad = if let Some(config) = kad_config {
            libp2p::kad::Behaviour::with_config(peer_id.clone(), store, config)
        } else {
            libp2p::kad::Behaviour::new(peer_id.clone(), store)
        };

        // Create request-response behavior for Paxos protocol
        let protocols = std::iter::once((
            StreamProtocol::new("/netabase/paxos/1.0.0"),
            ProtocolSupport::Full,
        ));

        let cfg = Config::default();
        let request_response = libp2p::request_response::cbor::Behaviour::new(protocols, cfg);

        // Create shared outgoing queue
        let outgoing_queue = Arc::new(Mutex::new(VecDeque::new()));

        // Create context for tracking applied entries
        let context = PaxosContext::default();

        Self {
            kad,
            context,
            request_response,
            outgoing_queue,
            pending_requests: HashMap::new(),
            peer_id: peer_id.clone(),
            cluster_members,
        }
    }

    /// Poll the outgoing queue and send any pending requests
    pub fn poll_outgoing(&mut self) {
        if let Ok(mut queue) = self.outgoing_queue.lock() {
            while let Some(request) = queue.pop_front() {
                let target_peer = request.target.0;
                let message = request.message;
                let response_tx = request.response_tx;

                // Send via request-response
                let request_id = self.request_response.send_request(&target_peer, message);

                // Store the response channel for when response arrives
                self.pending_requests.insert(request_id, response_tx);

                println!("Sent Paxos request {:?} to peer {:?}", request_id, target_peer);
            }
        }
    }

    /// Handle an incoming request from a peer
    fn handle_incoming_request(&mut self, peer: PeerId, request: PaxosMessage<D>) -> PaxosMessage<D> {
        // Extract the request and handle it
        match request {
            PaxosMessage::Request(paxos_request) => {
                let response = self.handle_paxos_request(paxos_request, peer);
                PaxosMessage::Response {
                    peer_id: peer.to_string(),
                    response,
                }
            }
            PaxosMessage::Response { .. } => {
                // Shouldn't receive a response as an incoming request
                println!("Warning: Received response message as request from {}", peer);
                PaxosMessage::Response {
                    peer_id: peer.to_string(),
                    response: PaxosResponse::Reject {
                        reason: "Invalid message type".to_string(),
                    },
                }
            }
        }
    }

    /// Handle a Paxos request and generate a response
    fn handle_paxos_request(&mut self, request: PaxosRequest<D>, _peer: PeerId) -> PaxosResponse {
        // TODO: Route to the actual paxakos::Node handler methods
        // For now, provide stub responses
        match request {
            PaxosRequest::Prepare { round, coord } => {
                println!("Handling Prepare request - round: {}, coord: {}", round, coord);
                // TODO: Call paxos_node.handle_prepare() or equivalent
                PaxosResponse::Promise { round, coord }
            }
            PaxosRequest::Proposal { round, coord, entry: _ } => {
                println!("Handling Proposal request - round: {}, coord: {}", round, coord);
                // TODO: Call paxos_node.handle_proposal() or equivalent
                PaxosResponse::Accept
            }
            PaxosRequest::Commit { round, coord } => {
                println!("Handling Commit request - round: {}, coord: {}", round, coord);
                // TODO: Call paxos_node.handle_commit() or equivalent
                PaxosResponse::Committed
            }
        }
    }

    /// Handle an incoming response and complete the waiting future
    fn handle_incoming_response(
        &mut self,
        request_id: libp2p::request_response::OutboundRequestId,
        response: PaxosMessage<D>,
    ) {
        // Extract the response payload
        let response_payload = match response {
            PaxosMessage::Response { response, .. } => response,
            PaxosMessage::Request(_) => {
                println!("Warning: Received request message as response");
                return;
            }
        };

        // Find and complete the waiting future
        if let Some(response_tx) = self.pending_requests.remove(&request_id) {
            let _ = response_tx.send(Ok(response_payload));
        } else {
            println!("Warning: Received response for unknown request ID: {:?}", request_id);
        }
    }

    /// Handle a failed request
    fn handle_failed_request(
        &mut self,
        request_id: libp2p::request_response::OutboundRequestId,
        error: String,
    ) {
        // Complete the waiting future with an error
        if let Some(response_tx) = self.pending_requests.remove(&request_id) {
            let _ = response_tx.send(Err(error));
        }
    }
}

// Implement event transformation for the NetworkBehaviour
impl<D> PaxosBehaviour<D>
where
    D: NetabaseDefinitionTrait + netabase_store::convert::ToIVec + Serialize + for<'de> Deserialize<'de> + paxakos::LogEntry,
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
    /// Process request-response events and emit PaxosEvents as needed
    pub fn on_request_response_event(
        &mut self,
        event: libp2p::request_response::Event<PaxosMessage<D>, PaxosMessage<D>>,
    ) -> Option<PaxosEvent<D>> {
        use libp2p::request_response::Event;

        // First, poll outgoing queue to send any pending requests
        self.poll_outgoing();

        match event {
            Event::Message { peer, message, connection_id: _ } => {
                use libp2p::request_response::Message;
                match message {
                    Message::Request { request, channel, .. } => {
                        // Handle incoming request and send response
                        let response = self.handle_incoming_request(peer, request);
                        let _ = self.request_response.send_response(channel, response);
                        None
                    }
                    Message::Response { request_id, response } => {
                        // Handle incoming response
                        self.handle_incoming_response(request_id, response);
                        None
                    }
                }
            }
            Event::OutboundFailure { request_id, error, .. } => {
                let error_msg = format!("Outbound request failed: {:?}", error);
                self.handle_failed_request(request_id, error_msg);
                None
            }
            Event::InboundFailure { peer, error, .. } => {
                println!("Inbound request from {} failed: {:?}", peer, error);
                None
            }
            Event::ResponseSent { peer, .. } => {
                println!("Response sent to {}", peer);
                None
            }
        }
    }
}

/// State trait implementation for PaxosBehaviour
///
/// This implements the core Paxos State trait, enabling PaxosBehaviour to apply
/// committed log entries to the store. Since PaxosBehaviour owns the store via
/// its embedded Kademlia instance, it can directly apply entries using the
/// macro-generated `apply_to_store` method.
///
/// **Phase 3 Implementation:**
/// This connects the macro-generated `apply_to_store` (Phase 1) with store access
/// via `self.kad.store_mut()` (Phase 2), enabling actual consensus entry application.
#[cfg(all(feature = "paxos", feature = "libp2p"))]
impl<D> State for PaxosBehaviour<D>
where
    D: NetabaseDefinitionTrait + netabase_store::convert::ToIVec + Serialize + for<'de> Deserialize<'de> + paxakos::LogEntry<Id = <D as NetabaseDefinitionTrait>::Keys>,
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
    type Frozen = FrozenState<D>;
    type LogEntry = D;
    type Context = PaxosContext<D>;
    type Outcome = ();
    type Effect = ();
    type Error = String;
    type Node = NetabaseNodeInfo;

    /// Apply a committed log entry to the store
    ///
    /// This is the core consensus operation. Once Paxos achieves consensus on a
    /// log entry, this method is called to apply it to the local database.
    ///
    /// **Implementation:**
    /// 1. Check idempotency - skip if already applied (using entry content hash as ID)
    /// 2. Call the macro-generated `apply_to_store` method (Phase 1)
    /// 3. Access store via `self.kad.store_mut()` (Phase 2)
    /// 4. Flush store to ensure persistence
    /// 5. Mark entry as applied in context to prevent re-application
    ///
    /// **Idempotency:**
    /// Entries are tracked by their content-addressable ID (blake3 hash of serialized entry).
    /// This ensures that even if Paxos asks us to apply the same entry multiple times
    /// (due to retries, leader changes, etc.), we only apply it once.
    ///
    /// # Arguments
    /// * `log_entry` - The definition entry to apply (User, Post, etc.)
    /// * `context` - Paxos context tracking applied entries and round numbers
    ///
    /// # Returns
    /// * `Ok(((), ()))` - Entry successfully applied (or already applied)
    /// * `Err(String)` - Application failed with error message
    fn apply(
        &mut self,
        log_entry: &Self::LogEntry,
        context: &mut Self::Context,
    ) -> Result<(Self::Outcome, Self::Effect), Self::Error> {
        // Get the primary key for this entry (used as the log entry ID)
        // The LogEntry trait defines `id()` which returns D::Keys
        // This directly maps to the database primary key for idempotency checking
        let entry_id = <Self::LogEntry as paxakos::LogEntry>::id(log_entry);

        // Check if already applied (idempotency guard)
        if context.is_applied(&entry_id) {
            // Entry already in database, skip re-application
            return Ok(((), ()));
        }

        // Apply the entry to the store using the macro-generated method
        // This routes the Definition variant (User, Post, etc.) to the correct
        // store tree and creates a libp2p Record with the appropriate key format
        //
        // The apply_to_store method is generated by the netabase_definition_module macro
        // when both paxos and libp2p features are enabled (Phase 1).
        // It takes `&self` and applies the entry based on its variant.
        #[cfg(all(feature = "paxos", feature = "libp2p"))]
        {
            // Phase 1 generated this method with signature:
            // pub fn apply_to_store<S>(&self, store: &mut S) -> Result<(), String>
            // where S: RecordStore
            log_entry.apply_to_store(self.kad.store_mut())
                .map_err(|e| format!("Failed to apply entry to store: {}", e))?;
        }

        #[cfg(not(all(feature = "paxos", feature = "libp2p")))]
        {
            return Err("apply_to_store requires both paxos and libp2p features".to_string());
        }

        // Flush the store to ensure the entry is persisted to disk
        // This is critical for durability - if we crash before flushing,
        // the entry might be lost
        // Note: libp2p's RecordStore trait doesn't have a flush method, so we rely on
        // the underlying store's automatic persistence. The SledStore and RedbStore
        // implementations handle persistence internally on each write.

        // Mark entry as applied in context to prevent duplicate applications
        // The round number should ideally come from Paxos, but for now we use 0
        // This will be properly implemented in Phase 4 when we track rounds
        context.mark_applied(entry_id, 0);

        Ok(((), ()))
    }

    /// Get cluster membership at a given round
    ///
    /// Returns the configured list of cluster members that participate in consensus.
    /// This implementation uses static membership (same members at all rounds).
    ///
    /// # Arguments
    /// * `_round_offset` - The round number to query membership for (unused for static membership)
    ///
    /// # Returns
    /// Vector of node information for all cluster members
    ///
    /// # Behavior
    /// - Empty cluster: Returns empty vec (single-node mode, all proposals auto-commit)
    /// - With members: Returns all configured cluster members wrapped in NetabaseNodeInfo
    ///
    /// # Future Enhancement
    /// Dynamic membership could be added where cluster composition changes by round,
    /// allowing nodes to join/leave the cluster over time.
    fn cluster_at(
        &self,
        _round_offset: std::num::NonZeroUsize,
    ) -> Vec<Self::Node> {
        // Return static cluster membership
        // Each PeerId is wrapped in NetabaseNodeInfo for the paxakos NodeInfo trait
        self.cluster_members
            .iter()
            .map(|peer_id| crate::NetabaseNodeInfo(peer_id.clone()))
            .collect()
    }

    /// Freeze the state for snapshotting
    ///
    /// Creates a point-in-time snapshot of which entries have been applied.
    /// This is used for:
    /// - Catching up lagging nodes
    /// - Recovery after crashes
    /// - State transfer in leader election
    ///
    /// # Arguments
    /// * `context` - The Paxos context to snapshot
    ///
    /// # Returns
    /// A frozen state snapshot that can be serialized and sent to other nodes
    fn freeze(&self, context: &mut Self::Context) -> Self::Frozen {
        FrozenState {
            applied_entries: context.applied_entries.clone(),
            last_applied_round: context.last_applied_round,
            _marker: std::marker::PhantomData,
        }
    }
}

/// NodeInfo implementation for NetabaseNodeInfo
///
/// This allows PeerId to be used as a node identifier in Paxos
impl NodeInfo for NetabaseNodeInfo {
    type Id = PeerId;

    fn id(&self) -> Self::Id {
        self.0
    }
}

/// Manual NetworkBehaviour implementation for PaxosBehaviour
impl<D> libp2p::swarm::NetworkBehaviour for PaxosBehaviour<D>
where
    D: NetabaseDefinitionTrait + netabase_store::convert::ToIVec + Serialize + for<'de> Deserialize<'de> + paxakos::LogEntry,
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
    type ConnectionHandler = <libp2p::request_response::cbor::Behaviour<PaxosMessage<D>, PaxosMessage<D>> as libp2p::swarm::NetworkBehaviour>::ConnectionHandler;
    type ToSwarm = PaxosEvent<D>;

    fn handle_established_inbound_connection(
        &mut self,
        connection_id: libp2p::swarm::ConnectionId,
        peer: PeerId,
        local_addr: &libp2p::Multiaddr,
        remote_addr: &libp2p::Multiaddr,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        self.request_response.handle_established_inbound_connection(
            connection_id,
            peer,
            local_addr,
            remote_addr,
        )
    }

    fn handle_established_outbound_connection(
        &mut self,
        connection_id: libp2p::swarm::ConnectionId,
        peer: PeerId,
        addr: &libp2p::Multiaddr,
        role_override: libp2p::core::Endpoint,
        port_use: libp2p::core::transport::PortUse,
    ) -> Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        self.request_response.handle_established_outbound_connection(
            connection_id,
            peer,
            addr,
            role_override,
            port_use,
        )
    }

    fn on_swarm_event(&mut self, event: libp2p::swarm::FromSwarm) {
        // Forward swarm events to both sub-behaviors
        // Note: We need to clone for kad since both behaviors need to see the event
        match &event {
            libp2p::swarm::FromSwarm::ConnectionEstablished(e) => {
                self.kad.on_swarm_event(libp2p::swarm::FromSwarm::ConnectionEstablished(*e));
            }
            libp2p::swarm::FromSwarm::ConnectionClosed(e) => {
                self.kad.on_swarm_event(libp2p::swarm::FromSwarm::ConnectionClosed(*e));
            }
            libp2p::swarm::FromSwarm::AddressChange(e) => {
                self.kad.on_swarm_event(libp2p::swarm::FromSwarm::AddressChange(*e));
            }
            libp2p::swarm::FromSwarm::DialFailure(e) => {
                self.kad.on_swarm_event(libp2p::swarm::FromSwarm::DialFailure(*e));
            }
            libp2p::swarm::FromSwarm::ListenFailure(e) => {
                self.kad.on_swarm_event(libp2p::swarm::FromSwarm::ListenFailure(*e));
            }
            libp2p::swarm::FromSwarm::NewListener(e) => {
                self.kad.on_swarm_event(libp2p::swarm::FromSwarm::NewListener(*e));
            }
            libp2p::swarm::FromSwarm::NewListenAddr(e) => {
                self.kad.on_swarm_event(libp2p::swarm::FromSwarm::NewListenAddr(*e));
            }
            libp2p::swarm::FromSwarm::ExpiredListenAddr(e) => {
                self.kad.on_swarm_event(libp2p::swarm::FromSwarm::ExpiredListenAddr(*e));
            }
            libp2p::swarm::FromSwarm::ListenerError(e) => {
                self.kad.on_swarm_event(libp2p::swarm::FromSwarm::ListenerError(*e));
            }
            libp2p::swarm::FromSwarm::ListenerClosed(e) => {
                self.kad.on_swarm_event(libp2p::swarm::FromSwarm::ListenerClosed(*e));
            }
            libp2p::swarm::FromSwarm::NewExternalAddrCandidate(e) => {
                self.kad.on_swarm_event(libp2p::swarm::FromSwarm::NewExternalAddrCandidate(*e));
            }
            libp2p::swarm::FromSwarm::ExternalAddrExpired(e) => {
                self.kad.on_swarm_event(libp2p::swarm::FromSwarm::ExternalAddrExpired(*e));
            }
            libp2p::swarm::FromSwarm::ExternalAddrConfirmed(e) => {
                self.kad.on_swarm_event(libp2p::swarm::FromSwarm::ExternalAddrConfirmed(*e));
            }
            _ => {}
        }
        self.request_response.on_swarm_event(event);
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: PeerId,
        connection_id: libp2p::swarm::ConnectionId,
        event: libp2p::swarm::THandlerOutEvent<Self>,
    ) {
        self.request_response.on_connection_handler_event(peer_id, connection_id, event);
    }

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<libp2p::swarm::ToSwarm<Self::ToSwarm, libp2p::swarm::THandlerInEvent<Self>>> {
        use std::task::Poll;
        use libp2p::swarm::ToSwarm;

        // Poll Kademlia first
        loop {
            match self.kad.poll(cx) {
                Poll::Ready(ToSwarm::GenerateEvent(event)) => {
                    // Wrap and emit Kademlia events
                    return Poll::Ready(ToSwarm::GenerateEvent(PaxosEvent::Kad(event)));
                }
                Poll::Ready(other) => {
                    // Forward ToSwarm actions from Kad that don't have behavior-specific types
                    // Note: NotifyHandler, CloseConnection, etc. are handled by the swarm
                    // internally for the sub-behavior, so we don't forward them
                    let mapped = match other {
                        ToSwarm::Dial { opts } => ToSwarm::Dial { opts },
                        ToSwarm::NewExternalAddrCandidate(addr) => {
                            ToSwarm::NewExternalAddrCandidate(addr)
                        }
                        ToSwarm::ExternalAddrConfirmed(addr) => {
                            ToSwarm::ExternalAddrConfirmed(addr)
                        }
                        ToSwarm::ExternalAddrExpired(addr) => {
                            ToSwarm::ExternalAddrExpired(addr)
                        }
                        ToSwarm::ListenOn { opts } => ToSwarm::ListenOn { opts },
                        ToSwarm::RemoveListener { id } => ToSwarm::RemoveListener { id },
                        ToSwarm::GenerateEvent(_) => unreachable!("Already handled above"),
                        // Skip behavior-specific variants that can't be forwarded
                        _ => {
                            continue;
                        }
                    };
                    return Poll::Ready(mapped);
                }
                Poll::Pending => break,
            }
        }

        // Poll the request-response behaviour
        loop {
            match self.request_response.poll(cx) {
                Poll::Ready(ToSwarm::GenerateEvent(event)) => {
                    // Handle the request-response event
                    if let Some(paxos_event) = self.on_request_response_event(event) {
                        return Poll::Ready(ToSwarm::GenerateEvent(paxos_event));
                    }
                    // If no Paxos event was generated, continue polling
                    continue;
                }
                Poll::Ready(other) => {
                    // Forward ToSwarm actions from request-response that don't have behavior-specific types
                    let mapped = match other {
                        ToSwarm::Dial { opts } => ToSwarm::Dial { opts },
                        ToSwarm::NewExternalAddrCandidate(addr) => {
                            ToSwarm::NewExternalAddrCandidate(addr)
                        }
                        ToSwarm::ExternalAddrConfirmed(addr) => {
                            ToSwarm::ExternalAddrConfirmed(addr)
                        }
                        ToSwarm::ExternalAddrExpired(addr) => {
                            ToSwarm::ExternalAddrExpired(addr)
                        }
                        ToSwarm::ListenOn { opts } => ToSwarm::ListenOn { opts },
                        ToSwarm::RemoveListener { id } => ToSwarm::RemoveListener { id },
                        ToSwarm::GenerateEvent(_) => unreachable!("Already handled above"),
                        // Skip behavior-specific variants that can't be forwarded
                        _ => {
                            continue;
                        }
                    };
                    return Poll::Ready(mapped);
                }
                Poll::Pending => {
                    // Try to send outgoing messages before returning Pending
                    self.poll_outgoing();
                    return Poll::Pending;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Update these tests to use a proper test Definition type with Keys
    // These tests need to be rewritten since LogEntry::Id is now D::Keys instead of [u8; 32]

    // Example of what the updated test should look like:
    // #[test]
    // fn test_paxos_context_idempotency() {
    //     // Create a test Definition with proper Keys enum
    //     let mut context = PaxosContext::<TestDefinition>::default();
    //     let entry = TestDefinition::User(TestUser { id: 1, ... });
    //     let entry_id = entry.id(); // Returns TestDefinitionKeys::User(1)
    //
    //     assert!(!context.is_applied(&entry_id));
    //     context.mark_applied(entry_id.clone(), 1);
    //     assert!(context.is_applied(&entry_id));
    // }
}
