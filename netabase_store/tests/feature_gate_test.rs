use bincode::{Decode, Encode};
use netabase_macros::NetabaseModel;

#[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq)]
#[key_name(TestUserKey)]
pub struct TestUser {
    #[key]
    pub id: u64,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use netabase_store::traits::NetabaseModel;

    #[test]
    fn test_basic_functionality_always_works() {
        // This should always work regardless of feature flags
        let user = TestUser {
            id: 1,
            name: "Test User".to_string(),
        };

        // Basic functionality should work
        let key = user.key();
        assert_eq!(key, TestUserKey::Primary(TestUserPrimaryKey(1)));
    }

    #[cfg(feature = "libp2p")]
    #[test]
    fn test_libp2p_functionality_when_enabled() {
        use netabase_store::traits::NetabaseSchemaLibp2p;

        let user = TestUser {
            id: 42,
            name: "LibP2P User".to_string(),
        };

        // libp2p Record conversion should work when feature is enabled
        let record = user.to_record().unwrap();
        assert!(!record.value.is_empty());
        assert!(!record.key.to_vec().is_empty());

        let decoded = TestUser::from_record(record).unwrap();
        assert_eq!(user, decoded);
    }

    #[cfg(feature = "libp2p")]
    #[test]
    fn test_libp2p_key_functionality_when_enabled() {
        use netabase_store::traits::NetabaseKeysLibp2p;

        let key = TestUserKey::Primary(TestUserPrimaryKey(99));

        // libp2p RecordKey conversion should work when feature is enabled
        let record_key = key.to_record_key().unwrap();
        assert!(!record_key.to_vec().is_empty());

        let decoded = TestUserKey::from_record_key(record_key).unwrap();
        assert_eq!(key, decoded);
    }

    #[cfg(feature = "libp2p")]
    #[test]
    fn test_generated_tryinto_impls_with_libp2p() {
        use std::convert::TryInto;

        let user = TestUser {
            id: 200,
            name: "TryInto Test User".to_string(),
        };

        // libp2p TryInto should only be available with feature
        let record: libp2p::kad::Record = user.clone().try_into().unwrap();
        let decoded: TestUser = record.try_into().unwrap();
        assert_eq!(user, decoded);

        let key = TestUserKey::Primary(TestUserPrimaryKey(200));
        let record_key: libp2p::kad::RecordKey = key.clone().try_into().unwrap();
        let decoded_key: TestUserKey = record_key.try_into().unwrap();
        assert_eq!(key, decoded_key);
    }

    #[cfg(not(feature = "libp2p"))]
    #[test]
    fn test_no_libp2p_functionality_without_feature() {
        // This test ensures that the code compiles and works without libp2p feature.
        // The main thing is that this test can run without libp2p dependencies.
        let user = TestUser {
            id: 100,
            name: "No LibP2P User".to_string(),
        };

        // Basic model functionality should always work
        let key = user.key();
        assert_eq!(key, TestUserKey::Primary(TestUserPrimaryKey(100)));

        // This test passing means libp2p is properly feature-gated
        assert_eq!(user.name, "No LibP2P User");
    }
}
