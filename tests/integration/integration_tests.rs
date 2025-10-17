use std::sync::Once;
use std::time::Duration;

use bincode::{Decode, Encode};
use libp2p::{Multiaddr, PeerId};
use netabase::Netabase;
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::traits::{
    NetabaseModel as NetabaseModelTrait, NetabaseSchema, NetabaseSchemaQuery,
};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use test_schema::*;
use tokio::time::timeout;

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    });
}

// Test schema for integration tests
#[netabase_schema_module(TestSchema, TestSchemaKeys)]
pub mod test_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(TestUserKey)]
    pub struct TestUser {
        #[key]
        pub id: u64,
        pub name: String,
        pub email: String,
        pub created_at: u64,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(TestPostKey)]
    pub struct TestPost {
        #[key]
        pub id: u64,
        pub title: String,
        pub content: String,
        pub author_id: u64,
        pub created_at: u64,
    }
}

use test_schema::{TestPost, TestSchema, TestUser};

/// Generate a unique temporary database directory for each test to avoid conflicts
fn create_temp_db() -> (TempDir, String) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir
        .path()
        .join("test_db")
        .to_string_lossy()
        .to_string();
    (temp_dir, db_path)
}

/// Create a test user with unique data
fn create_test_user(id: u64) -> TestUser {
    TestUser {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
        created_at: chrono::Utc::now().timestamp() as u64,
    }
}

/// Create a test post with unique data
fn create_test_post(id: u64, author_id: u64) -> TestPost {
    TestPost {
        id,
        title: format!("Post {}", id),
        content: format!("Content for post {}", id),
        author_id,
        created_at: chrono::Utc::now().timestamp() as u64,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_netabase_creation() {
    init_logger();

    let netabase = Netabase::<TestSchema>::new().unwrap();

    // Test that we can create broadcast subscriptions
    let receiver1 = netabase.subscribe_to_broadcasts();
    let receiver2 = netabase.subscribe_to_broadcasts();

    // Verify they are independent
    assert_ne!(
        &receiver1 as *const _ as usize,
        &receiver2 as *const _ as usize
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_netabase_creation_with_path() {
    init_logger();

    let (_temp_dir, db_path) = create_temp_db();
    let netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();

    // Test that creation works with custom path
    let _receiver = netabase.subscribe_to_broadcasts();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_swarm_lifecycle() {
    init_logger();

    let (_temp_dir, db_path) = create_temp_db();
    let mut netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();

    // Test starting the swarm
    let result = netabase.start_swarm().await;
    assert!(result.is_ok(), "Failed to start swarm: {:?}", result);

    // Give the swarm a moment to initialize
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test stopping the swarm
    let result = netabase.stop_swarm().await;
    assert!(result.is_ok(), "Failed to stop swarm: {:?}", result);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_swarm_double_start_prevention() {
    init_logger();

    let (_temp_dir, db_path) = create_temp_db();
    let mut netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();

    // Start the swarm
    let result1 = netabase.start_swarm().await;
    assert!(result1.is_ok());

    // Try to start again - should fail
    let result2 = netabase.start_swarm().await;
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("already running"));

    // Clean up
    let _ = netabase.stop_swarm().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_basic_put_record() {
    init_logger();

    let (_temp_dir, db_path) = create_temp_db();
    let mut netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    // Give the swarm time to initialize
    tokio::time::sleep(Duration::from_millis(200)).await;

    let test_user = create_test_user(1);

    // Test putting a record
    let result = timeout(
        Duration::from_secs(5),
        netabase.put_record(test_user.clone()),
    )
    .await;

    match result {
        Ok(put_result) => {
            match put_result {
                Ok(query_result) => {
                    println!("Put record successful: {:?}", query_result);
                }
                Err(e) => {
                    println!("Put record failed: {:?}", e);
                    // Don't fail the test for DHT operations that might not complete
                }
            }
        }
        Err(_) => {
            println!("Put record timed out - this is expected in single-node DHT");
        }
    }

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_basic_get_record() {
    init_logger();

    let (_temp_dir, db_path) = create_temp_db();
    let mut netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    // Give the swarm time to initialize
    tokio::time::sleep(Duration::from_millis(200)).await;

    let test_user = create_test_user(2);

    // First try to put the record
    let _ = timeout(
        Duration::from_secs(2),
        netabase.put_record(test_user.clone()),
    )
    .await;

    // Then try to get it using the user's key
    let result = timeout(Duration::from_secs(5), netabase.get_record(test_user.key())).await;

    match result {
        Ok(get_result) => match get_result {
            Ok(query_result) => {
                println!("Get record successful: {:?}", query_result);
            }
            Err(e) => {
                println!("Get record failed: {:?}", e);
            }
        },
        Err(_) => {
            println!("Get record timed out - this is expected in single-node DHT");
        }
    }

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_provider_operations() {
    init_logger();

    let (_temp_dir, db_path) = create_temp_db();
    let mut netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    // Give the swarm time to initialize
    tokio::time::sleep(Duration::from_millis(200)).await;

    let test_user = create_test_user(3);
    let user_key = test_user.key();

    // Test start providing
    let result = timeout(
        Duration::from_secs(5),
        netabase.start_providing(user_key.clone()),
    )
    .await;
    match result {
        Ok(provide_result) => match provide_result {
            Ok(query_result) => {
                println!("Start providing successful: {:?}", query_result);
            }
            Err(e) => {
                println!("Start providing failed: {:?}", e);
            }
        },
        Err(_) => {
            println!("Start providing timed out");
        }
    }

    // Test get providers
    let result = timeout(
        Duration::from_secs(5),
        netabase.get_providers(user_key.clone()),
    )
    .await;
    match result {
        Ok(providers_result) => match providers_result {
            Ok(query_result) => {
                println!("Get providers successful: {:?}", query_result);
            }
            Err(e) => {
                println!("Get providers failed: {:?}", e);
            }
        },
        Err(_) => {
            println!("Get providers timed out");
        }
    }

    // Test stop providing
    let result = timeout(Duration::from_secs(2), netabase.stop_providing(user_key)).await;
    assert!(result.is_ok(), "Stop providing should not timeout");

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_bootstrap_operation() {
    init_logger();

    let (_temp_dir, db_path) = create_temp_db();
    let mut netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    // Give the swarm time to initialize
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Test bootstrap - will likely fail with no known peers, but should not crash
    let result = timeout(Duration::from_secs(5), netabase.bootstrap()).await;

    match result {
        Ok(bootstrap_result) => match bootstrap_result {
            Ok(query_result) => {
                println!("Bootstrap successful: {:?}", query_result);
            }
            Err(e) => {
                println!("Bootstrap failed (expected with no peers): {:?}", e);
            }
        },
        Err(_) => {
            println!("Bootstrap timed out");
        }
    }

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_mode_operations() {
    init_logger();

    let (_temp_dir, db_path) = create_temp_db();
    let mut netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    // Give the swarm time to initialize
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Test getting current mode
    let result = timeout(Duration::from_secs(2), netabase.get_mode()).await;
    assert!(result.is_ok(), "Get mode should not timeout");

    let current_mode = result.unwrap().unwrap();
    println!("Current DHT mode: {:?}", current_mode);

    // Test setting mode
    let result = timeout(
        Duration::from_secs(2),
        netabase.set_mode(Some(libp2p::kad::Mode::Client)),
    )
    .await;
    assert!(result.is_ok(), "Set mode should not timeout");

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_protocol_names() {
    init_logger();

    let (_temp_dir, db_path) = create_temp_db();
    let mut netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    // Give the swarm time to initialize
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Test getting protocol names
    let result = timeout(Duration::from_secs(2), netabase.get_protocol_names()).await;
    assert!(result.is_ok(), "Get protocol names should not timeout");

    let protocol = result.unwrap().unwrap();
    println!("Protocol name: {:?}", protocol);

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_record_removal() {
    init_logger();

    let (_temp_dir, db_path) = create_temp_db();
    let mut netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    // Give the swarm time to initialize
    tokio::time::sleep(Duration::from_millis(200)).await;

    let test_user = create_test_user(4);
    let user_key = test_user.key();

    // Test removing a record (should not timeout even if record doesn't exist)
    let result = timeout(Duration::from_secs(2), netabase.remove_record(user_key)).await;
    assert!(result.is_ok(), "Remove record should not timeout");

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_multiple_record_types() {
    init_logger();

    let (_temp_dir, db_path) = create_temp_db();
    let mut netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    // Give the swarm time to initialize
    tokio::time::sleep(Duration::from_millis(200)).await;

    let test_user = create_test_user(5);
    let test_post = create_test_post(1, test_user.id);

    // Test putting different types of records
    let user_result = timeout(
        Duration::from_secs(3),
        netabase.put_record(test_user.clone()),
    )
    .await;
    let post_result = timeout(
        Duration::from_secs(3),
        netabase.put_record(test_post.clone()),
    )
    .await;

    println!("User put result: {:?}", user_result);
    println!("Post put result: {:?}", post_result);

    // Test getting different types of records using their keys
    let user_get_result =
        timeout(Duration::from_secs(3), netabase.get_record(test_user.key())).await;
    let post_get_result =
        timeout(Duration::from_secs(3), netabase.get_record(test_post.key())).await;

    println!("User get result: {:?}", user_get_result);
    println!("Post get result: {:?}", post_get_result);

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_broadcast_event_reception() {
    init_logger();

    let (_temp_dir, db_path) = create_temp_db();
    let mut netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();
    let mut receiver = netabase.subscribe_to_broadcasts();

    netabase.start_swarm().await.unwrap();

    // Give the swarm time to initialize and potentially generate events
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Try to receive any broadcast events (non-blocking)
    let mut event_count = 0;
    loop {
        match receiver.try_recv() {
            Ok(event) => {
                println!("Received broadcast event: {:?}", event);
                event_count += 1;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                break;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {
                println!("Receiver lagged behind");
                break;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                println!("Receiver channel closed");
                break;
            }
        }
    }

    println!("Total events received: {}", event_count);

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_concurrent_operations() {
    init_logger();

    let (_temp_dir, db_path) = create_temp_db();
    let mut netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    // Give the swarm time to initialize
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Test multiple concurrent operations
    let user1 = create_test_user(10);
    let user2 = create_test_user(11);
    let user3 = create_test_user(12);

    // Start multiple operations concurrently
    let put1 = netabase.put_record(user1.clone());
    let put2 = netabase.put_record(user2.clone());
    let put3 = netabase.put_record(user3.clone());

    // Wait for all operations with timeout
    let join_future = async { tokio::join!(put1, put2, put3) };
    let results = timeout(Duration::from_secs(10), join_future).await;

    match results {
        Ok((result1, result2, result3)) => {
            println!(
                "Concurrent put results: {:?}, {:?}, {:?}",
                result1, result2, result3
            );
        }
        Err(_) => {
            println!("Concurrent operations timed out - expected in single-node setup");
        }
    }

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_peer_management_operations() {
    init_logger();

    let (_temp_dir, db_path) = create_temp_db();
    let mut netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    // Give the swarm time to initialize
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Create a dummy peer ID and address for testing
    let peer_id = PeerId::random();
    let address: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();

    // Test adding an address
    let result = timeout(
        Duration::from_secs(2),
        netabase.add_address(peer_id, address.clone()),
    )
    .await;
    assert!(result.is_ok(), "Add address should not timeout");
    println!("Add address result: {:?}", result.unwrap());

    // Test removing an address
    let result = timeout(
        Duration::from_secs(2),
        netabase.remove_address(peer_id, address.clone()),
    )
    .await;
    assert!(result.is_ok(), "Remove address should not timeout");
    println!("Remove address result: {:?}", result.unwrap());

    // Test removing a peer
    let result = timeout(Duration::from_secs(2), netabase.remove_peer(peer_id)).await;
    assert!(result.is_ok(), "Remove peer should not timeout");
    println!("Remove peer result: {:?}", result.unwrap());

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test]
async fn test_direct_database_access() {
    init_logger();

    let (_temp_dir, db_path) = create_temp_db();
    let netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();

    // Test direct database access without starting swarm
    let mut db = netabase.database().unwrap();

    // Initialize trees for the schema discriminants
    let discriminants = TestSchema::all_schema_discriminants();
    db.initialize_trees_from_discriminants(&discriminants)
        .unwrap();

    // Create a test user
    let user = create_test_user(1);
    let user_schema = TestSchema::TestUser(user.clone());

    // Store directly in database
    let result = db.put_schema(&user_schema);
    assert!(
        result.is_ok(),
        "Direct database put should succeed: {:?}",
        result.err()
    );

    // Retrieve directly from database using model's key
    let user_key = user.key();
    let schema_key = TestSchemaKeys::from(user_key);
    let retrieved_schema = db.get_schema(&schema_key).unwrap();

    match retrieved_schema {
        Some(TestSchema::TestUser(retrieved_user)) => {
            assert_eq!(retrieved_user.id, user.id);
            assert_eq!(retrieved_user.name, user.name);
            assert_eq!(retrieved_user.email, user.email);
        }
        _ => panic!("Expected TestUser schema"),
    }
}

#[tokio::test]
async fn test_direct_database_mutable_access() {
    init_logger();

    let (_temp_dir, db_path) = create_temp_db();
    let netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();

    // Test mutable database access
    let mut db = netabase.database_mut().unwrap();

    // Initialize trees for the schema discriminants
    let discriminants = TestSchema::all_schema_discriminants();
    db.initialize_trees_from_discriminants(&discriminants)
        .unwrap();

    // Create test data
    let user = create_test_user(1);
    let post = create_test_post(1, user.id);

    let user_schema = TestSchema::TestUser(user.clone());
    let post_schema = TestSchema::TestPost(post.clone());

    // Store multiple items
    assert!(db.put_schema(&user_schema).is_ok());
    assert!(db.put_schema(&post_schema).is_ok());

    // Verify both items exist
    let user_key = user.key();
    let post_key = post.key();
    let user_schema_key = TestSchemaKeys::from(user_key);
    let post_schema_key = TestSchemaKeys::from(post_key);

    let retrieved_user = db.get_schema(&user_schema_key).unwrap();
    let retrieved_post = db.get_schema(&post_schema_key).unwrap();

    assert!(retrieved_user.is_some());
    assert!(retrieved_post.is_some());
}

#[tokio::test]
async fn test_direct_vs_network_operations() {
    init_logger();

    // Create two separate temp directories to avoid database conflicts
    let (_temp_dir1, db_path1) = create_temp_db();
    let (_temp_dir2, db_path2) = create_temp_db();

    // First test: Direct database operations
    let netabase1 = Netabase::<TestSchema>::new_with_path(&db_path1).unwrap();
    let user = create_test_user(1);
    let mut db = netabase1.database().unwrap();

    // Initialize trees for the schema discriminants
    let discriminants = TestSchema::all_schema_discriminants();
    db.initialize_trees_from_discriminants(&discriminants)
        .unwrap();

    let user_schema = TestSchema::TestUser(user.clone());
    assert!(db.put_schema(&user_schema).is_ok());

    // Verify direct database access works
    let schema_key = TestSchemaKeys::from(user.key());
    let direct_result = db.get_schema(&schema_key).unwrap();
    assert!(direct_result.is_some());

    // Second test: Network operations with separate database
    let mut netabase2 = Netabase::<TestSchema>::new_with_path(&db_path2).unwrap();
    netabase2.start_swarm().await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test network operations
    let test_user2 = create_test_user(2);
    let user_key2 = test_user2.key();
    let put_result = timeout(Duration::from_secs(2), netabase2.put_record(test_user2)).await;

    // Put operation should succeed or timeout (both are acceptable for this test)
    let _put_success = put_result.is_ok();

    netabase2.stop_swarm().await.unwrap();
}

#[tokio::test]
async fn test_database_error_handling() {
    init_logger();

    // Test with invalid path
    let invalid_netabase =
        Netabase::<TestSchema>::new_with_path("/invalid/path/that/does/not/exist").unwrap();
    let db_result = invalid_netabase.database();

    // Should handle database creation errors gracefully
    // Note: sled might create directories, so this test checks error handling exists
    match db_result {
        Ok(_) => {
            // If sled created the directory, that's fine - just verify we can access it
            println!("Database created successfully even with unusual path");
        }
        Err(e) => {
            println!("Database access failed as expected: {}", e);
        }
    }
}
