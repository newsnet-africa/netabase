//! Query protocol traits.
//!
//! Defines the core traits for the query protocol, enabling different
//! implementations while maintaining type safety and security.

use crate::primitives::{ConflictRank, NodeId};
use crate::query::messages::{
    QueryEntry, QueryError, QueryResponse, SecureQuery, WriteRequest, WriteResponse,
};

/// Result type for query protocol operations.
pub type QueryResult<T> = Result<T, QueryError>;

/// The query protocol trait.
///
/// Implementors provide the networking layer for executing queries
/// across the distributed system.
pub trait QueryProtocol<PK, SK> {
    /// The data type returned by queries
    type Item;

    /// Send a query to a specific peer.
    fn query_peer(
        &self,
        peer: NodeId,
        query: SecureQuery<PK, SK>,
    ) -> QueryResult<QueryResponse<Self::Item>>;

    /// Broadcast a query to all peers in a topic.
    fn broadcast_query(
        &self,
        topic: &str,
        query: SecureQuery<PK, SK>,
    ) -> QueryResult<Vec<QueryResponse<Self::Item>>>;

    /// Send a write request to the authoritative peer.
    fn send_write(
        &self,
        peer: NodeId,
        write: WriteRequest<Self::Item, PK, SK>,
    ) -> QueryResult<WriteResponse>;
}

/// Trait for types that can handle incoming queries.
pub trait QueryHandler<PK, SK> {
    /// The data type this handler works with
    type Item;

    /// Handle an incoming query.
    fn handle_query(
        &self,
        from: NodeId,
        query: SecureQuery<PK, SK>,
    ) -> QueryResult<QueryResponse<Self::Item>>;

    /// Handle an incoming write request.
    fn handle_write(
        &mut self,
        from: NodeId,
        write: WriteRequest<Self::Item, PK, SK>,
    ) -> QueryResult<WriteResponse>;
}

/// Trait for data stores that support querying.
pub trait QueryableStore<PK, SK> {
    /// The data type stored
    type Item;

    /// Query for entries matching a range.
    fn query_range(
        &self,
        range: &crate::primitives::NDimensionalRange<PK, SK>,
        limit: Option<u32>,
    ) -> QueryResult<Vec<QueryEntry<Self::Item>>>;

    /// Write an entry with conflict resolution.
    fn write_entry(
        &mut self,
        entry: QueryEntry<Self::Item>,
    ) -> QueryResult<WriteResponse>;

    /// Get an entry by exact match.
    fn get_entry(
        &self,
        author: &NodeId,
        primary_key: &PK,
    ) -> QueryResult<Option<QueryEntry<Self::Item>>>;
}

/// Trait for conflict resolution.
pub trait ConflictResolver {
    /// The data type being resolved
    type Item;

    /// Resolve a conflict between two entries.
    ///
    /// Returns the winning entry, or a merged entry if applicable.
    fn resolve(
        &self,
        local: &QueryEntry<Self::Item>,
        remote: &QueryEntry<Self::Item>,
    ) -> QueryEntry<Self::Item>;

    /// Check if an entry supersedes another.
    fn supersedes(
        &self,
        candidate: &QueryEntry<Self::Item>,
        current: &QueryEntry<Self::Item>,
    ) -> bool {
        candidate.rank > current.rank
    }
}

/// Default rank-based conflict resolver.
pub struct RankResolver<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> RankResolver<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Default for RankResolver<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> ConflictResolver for RankResolver<T> {
    type Item = T;

    fn resolve(
        &self,
        local: &QueryEntry<Self::Item>,
        remote: &QueryEntry<Self::Item>,
    ) -> QueryEntry<Self::Item> {
        if remote.rank > local.rank {
            remote.clone()
        } else {
            local.clone()
        }
    }
}

/// Subscription manager trait.
pub trait SubscriptionManager<PK, SK> {
    /// Subscribe to a range of data.
    fn subscribe(
        &mut self,
        range: crate::primitives::NDimensionalRange<PK, SK>,
    ) -> QueryResult<SubscriptionHandle>;

    /// Unsubscribe from a range.
    fn unsubscribe(&mut self, handle: SubscriptionHandle) -> QueryResult<()>;

    /// Get all active subscriptions.
    fn subscriptions(&self) -> Vec<SubscriptionHandle>;
}

/// Handle for a subscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionHandle(pub u64);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::LamportClock;

    #[test]
    fn test_rank_resolver() {
        let resolver = RankResolver::<String>::new();

        let entry1 = QueryEntry {
            author: NodeId::from_bytes([1u8; 32]),
            rank: ConflictRank::basic(10),
            lamport: LamportClock::new(5, [1u8; 8]),
            data: "old".to_string(),
            data_hash: [0u8; 32],
        };

        let entry2 = QueryEntry {
            author: NodeId::from_bytes([1u8; 32]),
            rank: ConflictRank::basic(20),
            lamport: LamportClock::new(10, [1u8; 8]),
            data: "new".to_string(),
            data_hash: [0u8; 32],
        };

        let winner = resolver.resolve(&entry1, &entry2);
        assert_eq!(winner.data, "new");
        assert!(resolver.supersedes(&entry2, &entry1));
    }

    #[test]
    fn test_subscription_handle() {
        let handle1 = SubscriptionHandle(1);
        let handle2 = SubscriptionHandle(1);
        let handle3 = SubscriptionHandle(2);

        assert_eq!(handle1, handle2);
        assert_ne!(handle1, handle3);
    }
}
