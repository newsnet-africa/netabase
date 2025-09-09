use netabase::netabase_trait::NetabaseSchema;
use netabase_macros::NetabaseSchema;
use netabase_macros::NetabaseSchemaKey;
use netabase_macros::schema_module;
use std::fmt::Debug;

#[schema_module(MyRegistry)]
pub mod registry {
    use bincode::Decode;
    use bincode::Encode;
    use netabase::netabase_trait::NetabaseSchema;
    use netabase_macros::NetabaseSchema;
    use netabase_macros::NetabaseSchemaKey;
    // Simple non-generic struct to test basic functionality
    #[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
    pub struct SimpleRecord {
        #[key]
        pub record_id: u64,
        pub name: String,
        pub description: Option<String>,
    }

    // Another simple struct with string key
    #[derive(NetabaseSchema, Encode, Decode, Clone, Debug)]
    pub struct UserRecord {
        #[key]
        pub user_id: String,
        pub username: String,
        pub email: String,
        pub created_at: u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_record() {
        let record = SimpleRecord {
            record_id: 1,
            name: "test".to_string(),
            description: Some("A test record".to_string()),
        };

        let key = record.key();
        assert_eq!(key.0, 1);
        println!("SimpleRecord key: {:?}", key);
    }

    #[test]
    fn test_user_record() {
        let user = UserRecord {
            user_id: "user123".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            created_at: 1234567890,
        };

        let key = user.key();
        assert_eq!(key.0, "user123");
        println!("UserRecord key: {:?}", key);
    }

    #[test]
    fn test_record_serialization() {
        let record = SimpleRecord {
            record_id: 42,
            name: "serialization test".to_string(),
            description: None,
        };

        // Test that the record can be converted to and from a kad::Record
        let kad_record: Result<macro_exports::__netabase_libp2p_kad::Record, _> =
            record.clone().try_into();
        assert!(
            kad_record.is_ok(),
            "Should be able to convert to kad::Record"
        );

        if let Ok(kad_record) = kad_record {
            let restored: Result<SimpleRecord, _> = kad_record.try_into();
            assert!(
                restored.is_ok(),
                "Should be able to convert back to SimpleRecord"
            );

            if let Ok(restored_record) = restored {
                assert_eq!(restored_record.record_id, record.record_id);
                assert_eq!(restored_record.name, record.name);
                assert_eq!(restored_record.description, record.description);
            }
        }
    }

    #[test]
    fn test_key_serialization() {
        let record = SimpleRecord {
            record_id: 100,
            name: "key test".to_string(),
            description: Some("Testing key serialization".to_string()),
        };

        let key = record.key();

        // Test that the key can be converted to and from a kad::RecordKey
        let kad_key: Result<macro_exports::__netabase_libp2p_kad::RecordKey, _> =
            key.clone().try_into();
        assert!(
            kad_key.is_ok(),
            "Should be able to convert key to kad::RecordKey"
        );

        if let Ok(kad_key) = kad_key {
            let restored_key: Result<SimpleRecordKey, _> = kad_key.try_into();
            assert!(
                restored_key.is_ok(),
                "Should be able to convert back to SimpleRecordKey"
            );

            if let Ok(restored_key) = restored_key {
                assert_eq!(restored_key.0, key.0);
            }
        }
    }
}
