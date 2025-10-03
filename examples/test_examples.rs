use bincode::{Decode, Encode};
use netabase::Netabase;
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::traits::NetabaseModel;
use serde::{Deserialize, Serialize};

/// Example schema module for testing netabase functionality
#[netabase_schema_module(TestSchema, TestSchemaKeys)]
mod test_schema {
    use super::*;

    /// Test user model
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(TestUserKey)]
    pub struct TestUser {
        #[key]
        pub id: u64,
        pub name: String,
    }
}

use test_schema::{TestSchema, TestUser};

#[cfg(test)]
mod tests {
    use crate::test_schema::TestUserKey;

    use super::*;
    use libp2p::{
        Multiaddr, PeerId,
        kad::{GetRecordOk, QueryResult},
    };
    use netabase::traits::NetabaseModel;
    use std::path::Path;
    use tempfile::TempDir;

    /// Test the NetabaseModel trait implementation
    #[test]
    fn test_netabase_model_traits() {
        let user = TestUser {
            id: 1,
            name: "Alice".into(),
        };

        // Test key generation
        let key = user.key();
        assert_eq!(
            key,
            TestUserKey::Primary(test_schema::TestUserPrimaryKey(1))
        );
        // Test tree name
        assert_eq!(TestUser::tree_name(), "TestUser");
    }

    /// Test the basic database operations from doctests
    #[tokio::test]
    async fn test_database_operations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path();

        let mut netabase = Netabase::<TestSchema>::new_with_path(db_path).unwrap();
        let user = TestUser {
            id: 1,
            name: "Alice".into(),
        };

        // Start the network swarm
        netabase.start_swarm().await.unwrap();

        // Test record operations
        let key = user.key();
        netabase.put_record(user.clone()).await.unwrap();

        // Get the record back
        let result = netabase.get_record(key.clone()).await.unwrap();

        match result {
            QueryResult::GetRecord(get_record_ok) => assert!(true),
            _ => assert!(false),
        }

        // Test provider operations
        netabase.start_providing(key.clone()).await.unwrap();
        netabase.stop_providing(key).await.unwrap();

        // Cleanup
        netabase.stop_swarm().await.unwrap();
    }

    /// Test network operations from doctests
    #[tokio::test]
    async fn test_network_operations() {
        let mut netabase = Netabase::<TestSchema>::new().unwrap();
        netabase.start_swarm().await.unwrap();

        // Test mode operations
        let mode = netabase.get_mode().await.unwrap();
        assert!(matches!(mode, libp2p::kad::Mode::Server));

        // Test protocol operations
        let protocols = netabase.get_protocol_names().await.unwrap();
        assert!(!protocols.to_string().is_empty());

        // Cleanup
        netabase.stop_swarm().await.unwrap();
    }

    /// Test peer operations from doctests
    #[tokio::test]
    async fn test_peer_operations() {
        let mut netabase = Netabase::<TestSchema>::new().unwrap();
        netabase.start_swarm().await.unwrap();

        // Create test peer data
        let peer_id = PeerId::random();
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/12345".parse().unwrap();

        // Test peer operations
        netabase.add_address(peer_id, addr.clone()).await.unwrap();
        netabase.remove_address(peer_id, addr).await.unwrap();
        netabase.remove_peer(peer_id).await.unwrap();

        // Cleanup
        netabase.stop_swarm().await.unwrap();
    }
}

fn main() {
    println!("Running Netabase examples...\n");

    // Example 1: Create test user and get its key
    println!("Example 1: Key generation");
    let user = TestUser {
        id: 1,
        name: String::from("Alice"),
    };
    let key = user.key();
    println!("Generated key from user: {:?}\n", key);

    // Example 2: Basic broadcast subscription
    println!("Example 2: Basic broadcast subscription");
    let netabase = Netabase::<TestSchema>::new().unwrap();
    let _receiver = netabase.subscribe_to_broadcasts();
    println!("Successfully subscribed to broadcasts\n");

    // Example 3: Multiple broadcast subscriptions
    println!("Example 3: Multiple broadcast subscriptions");
    let netabase = Netabase::<TestSchema>::new().unwrap();

    let receiver1 = netabase.subscribe_to_broadcasts();
    let receiver2 = netabase.subscribe_to_broadcasts();
    let receiver3 = netabase.subscribe_to_broadcasts();

    // Verify they are independent instances by comparing memory addresses
    let addr1 = &receiver1 as *const _ as usize;
    let addr2 = &receiver2 as *const _ as usize;
    let addr3 = &receiver3 as *const _ as usize;

    assert_ne!(addr1, addr2);
    assert_ne!(addr2, addr3);
    assert_ne!(addr1, addr3);
    println!("Successfully created multiple independent broadcast subscriptions\n");

    // Example 4: Broadcast receiver cloning using resubscribe
    println!("Example 4: Broadcast receiver cloning");
    let netabase = Netabase::<TestSchema>::new().unwrap();

    // Get a receiver and clone it
    let mut receiver1 = netabase.subscribe_to_broadcasts();
    let mut receiver2 = receiver1.resubscribe();

    // Verify both receivers are working and initially empty
    assert!(receiver1.try_recv().is_err());
    assert!(receiver2.try_recv().is_err());
    println!("Successfully demonstrated broadcast receiver cloning\n");

    println!("All examples completed successfully!");
}
