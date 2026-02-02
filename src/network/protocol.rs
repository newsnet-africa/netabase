//! Protocol messages for Netabase network communication.
//!
//! This module defines all message types used in the Netabase protocol,
//! including handshake, query, write, and sync operations.

use serde::{Deserialize, Serialize};

use crate::capabilities::{AuthorizationToken, Capability, CapabilitySignature};
use crate::primitives::{ConflictRank, NDimensionalRange, NodeId};
use crate::query::{QueryEntry, QueryError, QueryResponse, SecureQuery, WriteRequest, WriteResponse};

// =========================================================================
//  Top-Level Protocol Message
// =========================================================================

/// Top-level protocol message envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolMessage<PK, SK, T> {
    /// Handshake request
    HandshakeRequest(HandshakeRequest),
    
    /// Handshake response
    HandshakeResponse(HandshakeResponse),
    
    /// Query request
    Query(SecureQuery<PK, SK>),
    
    /// Query response
    QueryResponse(Result<QueryResponse<T>, QueryError>),
    
    /// Write request
    Write(WriteRequest<T, PK, SK>),
    
    /// Write response
    WriteResponse(WriteResponse),
    
    /// Sync request
    SyncRequest(SyncRequest<PK, SK>),
    
    /// Sync response
    SyncResponse(SyncResponse<T>),
    
    /// Capability grant
    GrantCapability(GrantCapabilityMessage<PK, SK>),
    
    /// Disconnect notification
    Disconnect(DisconnectMessage),
}

// =========================================================================
//  Handshake Protocol
// =========================================================================

/// Handshake request to establish connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandshakeRequest {
    /// Node sending the handshake
    pub from: NodeId,
    
    /// Protocol version
    pub protocol_version: u32,
    
    /// Supported features (bit flags)
    pub features: u64,
    
    /// Schema hash for compatibility checking
    pub schema_hash: [u8; 32],
    
    /// Nonce for replay protection
    pub nonce: u64,
    
    /// Timestamp
    pub timestamp: u64,
}

/// Handshake response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandshakeResponse {
    /// Node responding
    pub from: NodeId,
    
    /// Protocol version
    pub protocol_version: u32,
    
    /// Whether handshake is accepted
    pub accepted: bool,
    
    /// Optional rejection reason
    pub reason: Option<String>,
    
    /// Signature binding the response
    pub signature: CapabilitySignature,
}

// =========================================================================
//  Sync Protocol
// =========================================================================

/// Sync request to synchronize data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncRequest<PK, SK> {
    /// Range to sync
    pub range: NDimensionalRange<PK, SK>,
    
    /// Local fingerprint of the range
    pub local_fingerprint: Fingerprint,
    
    /// Capability authorizing sync
    pub capability: Capability<PK, SK>,
    
    /// Nonce
    pub nonce: u64,
    
    /// Timestamp
    pub timestamp: u64,
}

/// Sync response with data or fingerprints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncResponse<T> {
    /// Strategy used for sync
    pub strategy: SyncStrategy,
    
    /// Data if full sync
    pub entries: Vec<QueryEntry<T>>,
    
    /// Fingerprints if incremental sync
    pub fingerprints: Vec<RangeFingerprint>,
    
    /// Whether more data is available
    pub has_more: bool,
}

/// Sync strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStrategy {
    /// Send all data
    Full,
    
    /// Send fingerprints for incremental sync
    Incremental,
    
    /// No sync needed (fingerprints match)
    NoOp,
}

/// Merkle tree fingerprint of a range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fingerprint {
    /// Hash of the range
    pub hash: [u8; 32],
    
    /// Number of entries
    pub count: u64,
    
    /// Latest Lamport clock in range
    pub max_clock: u64,
}

/// Fingerprint of a sub-range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangeFingerprint {
    /// Start of the range
    pub start: Vec<u8>,
    
    /// End of the range
    pub end: Vec<u8>,
    
    /// Fingerprint
    pub fingerprint: Fingerprint,
}

// =========================================================================
//  Capability Management
// =========================================================================

/// Grant a capability to another node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrantCapabilityMessage<PK, SK> {
    /// The capability being granted
    pub capability: Capability<PK, SK>,
    
    /// Signature by the grantor
    pub signature: CapabilitySignature,
    
    /// Timestamp
    pub timestamp: u64,
}

// =========================================================================
//  Connection Management
// =========================================================================

/// Disconnect notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisconnectMessage {
    /// Node disconnecting
    pub from: NodeId,
    
    /// Reason for disconnect
    pub reason: DisconnectReason,
}

/// Reason for disconnection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisconnectReason {
    /// Clean shutdown
    Shutdown,
    
    /// Protocol version mismatch
    IncompatibleProtocol,
    
    /// Schema mismatch
    IncompatibleSchema,
    
    /// Rate limit exceeded
    RateLimited,
    
    /// Other reason
    Other(String),
}

// =========================================================================
//  Feature Flags
// =========================================================================

/// Protocol feature flags.
pub mod features {
    /// Support for sync protocol
    pub const SYNC: u64 = 1 << 0;
    
    /// Support for blob transfer
    pub const BLOBS: u64 = 1 << 1;
    
    /// Support for PAI (Private Area Intersection)
    pub const PAI: u64 = 1 << 2;
    
    /// Support for subscription rooms
    pub const SUBSCRIPTIONS: u64 = 1 << 3;
    
    /// Support for gossip
    pub const GOSSIP: u64 = 1 << 4;
}
