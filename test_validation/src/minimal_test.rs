//! Minimal test to verify NetabaseSchema derive macro functionality

use bincode::{Decode, Encode};
use netabase::NetabaseSchema;

#[derive(NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
struct SimpleUser {
    #[key]
    id: String,
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_derive() {
        let user = SimpleUser {
            id: "test123".to_string(),
            name: "Test User".to_string(),
        };

        let key = user.key();
        assert_eq!(key.as_str(), "test123");
    }
}
