//! Query executor for type-safe queries.
//!
//! This module provides execution logic for `SecureQuery` and related
//! query types with capability-based authorization.

use std::marker::PhantomData;

use crate::capabilities::{Capability, CapabilityError, Operation};
use crate::primitives::{ConflictRank, LamportClock, NDimensionalRange, NodeId};
use crate::query::messages::{
    QueryEntry, QueryError, QueryResponse, SecureQuery, WriteRequest, WriteResponse,
};

/// Result type for query operations.
pub type QueryResult<T> = Result<T, QueryError>;

/// Trait for executing secure queries.
///
/// Implementors provide the actual data access logic while this
/// trait handles authorization and validation.
pub trait QueryExecutor<PK, SK> {
    /// The data type returned by queries
    type Item;

    /// Execute a secure query after authorization.
    fn execute_query(
        &self,
        query: &SecureQuery<PK, SK>,
    ) -> QueryResult<QueryResponse<Self::Item>>;

    /// Execute a write request after authorization.
    fn execute_write(
        &mut self,
        write: &WriteRequest<Self::Item, PK, SK>,
    ) -> QueryResult<WriteResponse>;
}

/// Guard for validating queries before execution.
///
/// Guards can be chained to enforce multiple policies.
pub trait QueryGuard<PK, SK>: Send + Sync {
    /// Check if a query is valid.
    fn check_query(&self, query: &SecureQuery<PK, SK>) -> Result<(), QueryError>;
}

/// Separate trait for write validation (avoids generic issues with dyn).
pub trait WriteGuard<T, PK, SK>: Send + Sync {
    /// Check if a write is valid.
    fn check_write(&self, write: &WriteRequest<T, PK, SK>) -> Result<(), QueryError>;
}

/// Rate limiting guard.
pub struct RateLimitGuard {
    max_queries_per_second: u64,
    // In a real implementation, this would track state
}

impl RateLimitGuard {
    pub fn new(max_queries_per_second: u64) -> Self {
        Self {
            max_queries_per_second,
        }
    }
}

impl<PK, SK> QueryGuard<PK, SK> for RateLimitGuard {
    fn check_query(&self, _query: &SecureQuery<PK, SK>) -> Result<(), QueryError> {
        // TODO: Implement actual rate limiting
        // For now, always pass
        Ok(())
    }
}

impl<T, PK, SK> WriteGuard<T, PK, SK> for RateLimitGuard {
    fn check_write(&self, _write: &WriteRequest<T, PK, SK>) -> Result<(), QueryError> {
        // TODO: Implement actual rate limiting
        Ok(())
    }
}

/// Replay protection guard.
pub struct ReplayProtectionGuard {
    // In a real implementation, this would track seen nonces
    _phantom: PhantomData<()>,
}

impl ReplayProtectionGuard {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl Default for ReplayProtectionGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl<PK, SK> QueryGuard<PK, SK> for ReplayProtectionGuard {
    fn check_query(&self, query: &SecureQuery<PK, SK>) -> Result<(), QueryError> {
        // TODO: Check nonce against seen nonces
        // For now, just validate it's non-zero
        if query.nonce == 0 {
            return Err(QueryError::ReplayDetected);
        }
        Ok(())
    }
}

impl<T, PK, SK> WriteGuard<T, PK, SK> for ReplayProtectionGuard {
    fn check_write(&self, write: &WriteRequest<T, PK, SK>) -> Result<(), QueryError> {
        if write.nonce == 0 {
            return Err(QueryError::ReplayDetected);
        }
        Ok(())
    }
}

/// Timestamp validation guard.
pub struct TimestampGuard {
    max_skew_seconds: u64,
}

impl TimestampGuard {
    pub fn new(max_skew_seconds: u64) -> Self {
        Self { max_skew_seconds }
    }
}

impl<PK, SK> QueryGuard<PK, SK> for TimestampGuard {
    fn check_query(&self, query: &SecureQuery<PK, SK>) -> Result<(), QueryError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let diff = if now > query.timestamp {
            now - query.timestamp
        } else {
            query.timestamp - now
        };

        if diff > self.max_skew_seconds {
            return Err(QueryError::TimestampSkew);
        }

        Ok(())
    }
}

impl<T, PK, SK> WriteGuard<T, PK, SK> for TimestampGuard {
    fn check_write(&self, write: &WriteRequest<T, PK, SK>) -> Result<(), QueryError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let diff = if now > write.timestamp {
            now - write.timestamp
        } else {
            write.timestamp - now
        };

        if diff > self.max_skew_seconds {
            return Err(QueryError::TimestampSkew);
        }

        Ok(())
    }
}

/// Capability validation guard.
pub struct CapabilityGuard {
    root_owner: NodeId,
}

impl CapabilityGuard {
    pub fn new(root_owner: NodeId) -> Self {
        Self { root_owner }
    }
}

impl<PK, SK> QueryGuard<PK, SK> for CapabilityGuard
where
    PK: Clone,
    SK: Clone + PartialEq,
{
    fn check_query(&self, query: &SecureQuery<PK, SK>) -> Result<(), QueryError> {
        // Verify capability chain
        query
            .capability
            .verify_chain(&self.root_owner)
            .map_err(|e| match e {
                CapabilityError::Expired => QueryError::CapabilityExpired,
                CapabilityError::InvalidSignature => QueryError::InvalidSignature,
                _ => QueryError::MalformedCapability,
            })?;

        // Verify capability authorizes the query
        if !query
            .capability
            .authorizes(&Operation::Read, &query.range)
        {
            return Err(QueryError::Unauthorized);
        }

        Ok(())
    }
}

impl<T, PK, SK> WriteGuard<T, PK, SK> for CapabilityGuard
where
    PK: Clone,
    SK: Clone + PartialEq,
{
    fn check_write(&self, write: &WriteRequest<T, PK, SK>) -> Result<(), QueryError> {
        // Verify capability chain
        write
            .authorization
            .capability
            .verify_chain(&self.root_owner)
            .map_err(|e| match e {
                CapabilityError::Expired => QueryError::CapabilityExpired,
                CapabilityError::InvalidSignature => QueryError::InvalidSignature,
                _ => QueryError::MalformedCapability,
            })?;

        // Verify write operation is allowed
        if !write
            .authorization
            .capability
            .operation
            .includes(&Operation::Write)
        {
            return Err(QueryError::Unauthorized);
        }

        Ok(())
    }
}

/// Combined guard chain.
pub struct GuardChain<PK, SK> {
    guards: Vec<Box<dyn QueryGuard<PK, SK>>>,
}

impl<PK, SK> GuardChain<PK, SK> {
    pub fn new() -> Self {
        Self { guards: Vec::new() }
    }

    pub fn add_guard(mut self, guard: Box<dyn QueryGuard<PK, SK>>) -> Self {
        self.guards.push(guard);
        self
    }
}

impl<PK, SK> Default for GuardChain<PK, SK> {
    fn default() -> Self {
        Self::new()
    }
}

impl<PK, SK> QueryGuard<PK, SK> for GuardChain<PK, SK> {
    fn check_query(&self, query: &SecureQuery<PK, SK>) -> Result<(), QueryError> {
        for guard in &self.guards {
            guard.check_query(query)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::CapabilitySignature;
    use crate::primitives::{KeyRange, NodeIdRange, PathBuilder};

    #[test]
    fn test_rate_limit_guard() {
        let guard = RateLimitGuard::new(100);
        let query = create_test_query();
        assert!(guard.check_query(&query).is_ok());
    }

    #[test]
    fn test_replay_protection() {
        let guard = ReplayProtectionGuard::new();
        
        let mut query = create_test_query();
        query.nonce = 0;
        assert!(guard.check_query(&query).is_err());
        
        query.nonce = 1;
        assert!(guard.check_query(&query).is_ok());
    }

    #[test]
    fn test_timestamp_validation() {
        let guard = TimestampGuard::new(60); // 60 second skew
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut query = create_test_query();
        query.timestamp = now;
        assert!(guard.check_query(&query).is_ok());
        
        // Too far in future
        query.timestamp = now + 120;
        assert!(guard.check_query(&query).is_err());
    }

    #[test]
    fn test_guard_chain() {
        let chain = GuardChain::new()
            .add_guard(Box::new(ReplayProtectionGuard::new()))
            .add_guard(Box::new(TimestampGuard::new(60)));
        
        let query = create_test_query();
        assert!(chain.check_query(&query).is_ok());
    }

    fn create_test_query() -> SecureQuery<String, u16> {
        let range: NDimensionalRange<String, u16> = NDimensionalRange::new(
            NodeIdRange::All,
            KeyRange::prefix(PathBuilder::new().key("test").build()),
            vec![],
        );

        let capability = Capability::new_root(
            NodeId::from_bytes([1u8; 32]),
            NodeId::from_bytes([2u8; 32]),
            Operation::Read,
            range.clone(),
            u64::MAX,
        );

        SecureQuery {
            range,
            capability,
            nonce: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            signature: CapabilitySignature([0u8; 64]),
            continuation: None,
            limit: Some(100),
        }
    }
}
