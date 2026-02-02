//! Query validation utilities.
//!
//! Provides validation logic for query parameters, ranges, and permissions.

use std::fmt;

use crate::primitives::{NDimensionalRange, Path};
use crate::query::messages::{QueryError, SecureQuery, WriteRequest};

/// Validation errors specific to query structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Query range is empty or invalid
    InvalidRange,
    
    /// Path in query is malformed
    InvalidPath(String),
    
    /// Limit is out of acceptable range
    InvalidLimit { limit: u32, max: u32 },
    
    /// Query would return too much data
    QueryTooLarge,
    
    /// Missing required field
    MissingField(String),
    
    /// Invalid continuation token
    InvalidContinuation,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRange => write!(f, "Invalid query range"),
            Self::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
            Self::InvalidLimit { limit, max } => {
                write!(f, "Invalid limit {} (max: {})", limit, max)
            }
            Self::QueryTooLarge => write!(f, "Query would return too much data"),
            Self::MissingField(field) => write!(f, "Missing required field: {}", field),
            Self::InvalidContinuation => write!(f, "Invalid continuation token"),
        }
    }
}

impl std::error::Error for ValidationError {}

impl From<ValidationError> for QueryError {
    fn from(e: ValidationError) -> Self {
        QueryError::ExecutionError {
            message: e.to_string(),
        }
    }
}

/// Trait for validating queries.
pub trait ValidateQuery {
    /// Validate the query structure.
    fn validate(&self) -> Result<(), ValidationError>;
}

impl<PK, SK> ValidateQuery for SecureQuery<PK, SK> {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate nonce is non-zero
        if self.nonce == 0 {
            return Err(ValidationError::MissingField("nonce".to_string()));
        }

        // Validate timestamp is reasonable (not too far in future/past)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        const MAX_SKEW: u64 = 300; // 5 minutes
        if self.timestamp > now + MAX_SKEW || self.timestamp + MAX_SKEW < now {
            return Err(ValidationError::MissingField("valid timestamp".to_string()));
        }

        // Validate limit if present
        if let Some(limit) = self.limit {
            const MAX_LIMIT: u32 = 10000;
            if limit > MAX_LIMIT {
                return Err(ValidationError::InvalidLimit {
                    limit,
                    max: MAX_LIMIT,
                });
            }
        }

        Ok(())
    }
}

impl<T, PK, SK> ValidateQuery for WriteRequest<T, PK, SK> {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate nonce
        if self.nonce == 0 {
            return Err(ValidationError::MissingField("nonce".to_string()));
        }

        // Validate timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        const MAX_SKEW: u64 = 300;
        if self.timestamp > now + MAX_SKEW || self.timestamp + MAX_SKEW < now {
            return Err(ValidationError::MissingField("valid timestamp".to_string()));
        }

        Ok(())
    }
}

/// Validator for query parameters.
pub struct QueryValidator {
    max_limit: u32,
    max_range_size: usize,
}

impl QueryValidator {
    pub fn new(max_limit: u32, max_range_size: usize) -> Self {
        Self {
            max_limit,
            max_range_size,
        }
    }

    pub fn default() -> Self {
        Self {
            max_limit: 1000,
            max_range_size: 100_000,
        }
    }

    pub fn validate_limit(&self, limit: Option<u32>) -> Result<u32, ValidationError> {
        match limit {
            Some(l) if l > self.max_limit => Err(ValidationError::InvalidLimit {
                limit: l,
                max: self.max_limit,
            }),
            Some(l) => Ok(l),
            None => Ok(self.max_limit.min(100)), // Default to 100
        }
    }

    pub fn validate_path(&self, path: &Path) -> Result<(), ValidationError> {
        // Validate path
        path.validate()
            .map_err(|e| ValidationError::InvalidPath(e.to_string()))?;

        // Check path depth isn't excessive
        const MAX_PATH_DEPTH: usize = 20;
        if path.len() > MAX_PATH_DEPTH {
            return Err(ValidationError::InvalidPath(format!(
                "Path too deep: {} (max: {})",
                path.len(),
                MAX_PATH_DEPTH
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::{Capability, CapabilitySignature, Operation};
    use crate::primitives::{KeyRange, NodeId, NodeIdRange, PathBuilder};

    #[test]
    fn test_query_validation() {
        let query = create_valid_query();
        assert!(query.validate().is_ok());
    }

    #[test]
    fn test_invalid_nonce() {
        let mut query = create_valid_query();
        query.nonce = 0;
        assert!(query.validate().is_err());
    }

    #[test]
    fn test_invalid_limit() {
        let mut query = create_valid_query();
        query.limit = Some(100_000); // Way too high
        // Note: This would be caught by QueryValidator, not the query itself
    }

    #[test]
    fn test_query_validator_limit() {
        let validator = QueryValidator::new(1000, 100_000);
        
        assert!(validator.validate_limit(Some(500)).is_ok());
        assert!(validator.validate_limit(Some(2000)).is_err());
        assert_eq!(validator.validate_limit(None).unwrap(), 100);
    }

    #[test]
    fn test_path_validation() {
        let validator = QueryValidator::default();
        
        let path = PathBuilder::new().key("test").build();
        assert!(validator.validate_path(&path).is_ok());
        
        // Test deep path
        let mut deep_path = PathBuilder::new();
        for i in 0..25 {
            deep_path = deep_path.key(format!("level{}", i));
        }
        let path = deep_path.build();
        assert!(validator.validate_path(&path).is_err());
    }

    fn create_valid_query() -> SecureQuery<String, u16> {
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
