use serde::{Deserialize, Serialize};

use crate::capabilities::{AuthorizationToken, Capability, CapabilitySignature, Operation};
use crate::primitives::{
    ConflictRank, KeyRange, LamportClock, NDimensionalRange, NodeId, PathRange,
    SecondaryKeyRange,
};

// =========================================================================
//  Query Messages - Type-Safe Network Protocol
// =========================================================================

/// A secure query with capability-based authorization.
///
/// Every query includes:
/// - The query range (what data is requested)
/// - A capability proving permission to query that range
/// - A nonce for replay protection
/// - A timestamp for freshness checking
/// - A signature binding all the above
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecureQuery<PK, SK> {
    /// The N-dimensional range being queried
    pub range: NDimensionalRange<PK, SK>,
    
    /// Capability authorizing this query
    pub capability: Capability<PK, SK>,
    
    /// Monotonic nonce for replay protection
    pub nonce: u64,
    
    /// Unix timestamp of the query
    pub timestamp: u64,
    
    /// Signature over (range || capability || nonce || timestamp)
    pub signature: CapabilitySignature,
    
    /// Optional pagination cursor
    pub continuation: Option<ContinuationToken>,
    
    /// Maximum results to return
    pub limit: Option<u32>,
}

/// Opaque pagination token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuationToken(pub Vec<u8>);

/// Response to a secure query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryResponse<T> {
    /// The queried data
    pub entries: Vec<QueryEntry<T>>,
    
    /// Whether more data is available
    pub has_more: bool,
    
    /// Token for fetching next page
    pub continuation: Option<ContinuationToken>,
    
    /// Node that responded
    pub responder: NodeId,
    
    /// Signature by responder over the response
    pub signature: CapabilitySignature,
}

/// An entry returned from a query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryEntry<T> {
    /// The author of this entry
    pub author: NodeId,
    
    /// The entry's conflict resolution rank
    pub rank: ConflictRank,
    
    /// Lamport clock for causality tracking
    pub lamport: LamportClock,
    
    /// The actual data
    pub data: T,
    
    /// Hash of the data for integrity
    pub data_hash: [u8; 32],
}

/// Query error responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryError {
    /// Rate limit exceeded
    RateLimited { retry_after_ms: u64 },
    
    /// Replay attack detected (nonce reused)
    ReplayDetected,
    
    /// Clock skew too large
    TimestampSkew,
    
    /// Invalid signature
    InvalidSignature,
    
    /// Malformed capability
    MalformedCapability,
    
    /// Capability doesn't authorize this query
    Unauthorized,
    
    /// Capability has expired
    CapabilityExpired,
    
    /// Query range exceeds capability scope
    OutOfScope,
    
    /// Internal execution error
    ExecutionError { message: String },
    
    /// Unknown/unexpected error
    Unknown { message: String },
}

// =========================================================================
//  Write Operations
// =========================================================================

/// A write request with authorization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteRequest<T, PK, SK> {
    /// The entry to write
    pub entry: QueryEntry<T>,
    
    /// Authorization token proving write permission
    pub authorization: AuthorizationToken<PK, SK>,
    
    /// Nonce for replay protection
    pub nonce: u64,
    
    /// Timestamp
    pub timestamp: u64,
}

/// Response to a write request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteResponse {
    /// Write successful
    Ok {
        /// The final rank after conflict resolution
        rank: ConflictRank,
    },
    
    /// Write rejected due to conflict
    Conflict {
        /// The existing entry's rank (which won)
        existing_rank: ConflictRank,
    },
    
    /// Write failed
    Error(QueryError),
}

// =========================================================================
//  Sync Protocol Messages
// =========================================================================

/// Announce an area of interest for synchronization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindAreaOfInterest<PK, SK> {
    /// The area we want to sync
    pub area: NDimensionalRange<PK, SK>,
    
    /// Capability proving we're authorized to access this area
    pub capability: Capability<PK, SK>,
}

/// Send a fingerprint for a range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendFingerprint<PK, SK> {
    /// The range this fingerprint covers
    pub range: NDimensionalRange<PK, SK>,
    
    /// BLAKE3 hash of all entries in range
    pub fingerprint: [u8; 32],
    
    /// Number of entries in range
    pub entry_count: u64,
    
    /// Maximum Lamport clock in range
    pub max_lamport: LamportClock,
}

/// Request a subrange split.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestSubrange<PK, SK> {
    /// The range to split
    pub range: NDimensionalRange<PK, SK>,
    
    /// Which dimension to split on
    pub split_by: SplitDimension,
}

/// Dimension to split a range on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitDimension {
    /// Split by subspace (author)
    Subspace,
    
    /// Split by primary key
    PrimaryKey,
    
    /// Split by a specific secondary key
    SecondaryKey(u16), // Discriminant
    
    /// Split by Lamport clock
    Lamport,
}

/// Announce dropping interest in an area.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnounceDropInterest<PK, SK> {
    pub area: NDimensionalRange<PK, SK>,
}

// =========================================================================
//  Legacy Database Queries (For Migration)
// =========================================================================

/// Legacy query types - to be migrated to SecureQuery.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[deprecated(note = "Use SecureQuery instead")]
pub enum LegacyDatabaseQuery<PK, SK> {
    /// Get by primary key
    Get { key: PK },
    
    /// Get by secondary key
    GetBySecondary { key: SK },
    
    /// Check existence
    Exists { key: PK },
    
    /// Range query
    Range {
        start: Option<PK>,
        end: Option<PK>,
        limit: Option<u32>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{NodeIdRange, PathBuilder};

    #[test]
    fn test_secure_query_creation() {
        let range: NDimensionalRange<String, u16> = NDimensionalRange::new(
            NodeIdRange::All,
            KeyRange::prefix(PathBuilder::new().key("users").build()),
            vec![],
        );

        let capability = Capability::new_root(
            NodeId::from_bytes([1u8; 32]),
            NodeId::from_bytes([2u8; 32]),
            Operation::Read,
            range.clone(),
            u64::MAX,
        );

        let query = SecureQuery {
            range,
            capability,
            nonce: 1,
            timestamp: 1234567890,
            signature: CapabilitySignature([0u8; 64]),
            continuation: None,
            limit: Some(100),
        };

        assert_eq!(query.nonce, 1);
        assert_eq!(query.limit, Some(100));
    }

    #[test]
    fn test_query_entry() {
        let entry: QueryEntry<String> = QueryEntry {
            author: NodeId::from_bytes([1u8; 32]),
            rank: ConflictRank::basic(1),
            lamport: LamportClock::new(10, [1u8; 8]),
            data: "test data".to_string(),
            data_hash: [0u8; 32],
        };

        assert_eq!(entry.data, "test data");
    }
}