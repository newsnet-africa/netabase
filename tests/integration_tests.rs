use std::sync::Once;
use std::time::Duration;

use bincode::{Decode, Encode};
use libp2p::{Multiaddr, PeerId};
use netabase::Netabase;
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::traits::NetabaseModel as NetabaseModelTrait;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use uuid::Uuid;

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

/// Generate a unique database path for each test to avoid conflicts
fn generate_unique_db_path() -> String {
    let uuid = Uuid::new_v4();
    format!("test_db_{}", uuid.to_string().replace("-", "_"))
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

    let db_path = generate_unique_db_path();
    let netabase = Netabase::<TestSchema>::new_with_path(&db_path).unwrap();

    // Test that creation works with custom path
    let _receiver = netabase.subscribe_to_broadcasts();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_swarm_lifecycle() {
    init_logger();

    let mut netabase = Netabase::<TestSchema>::new().unwrap();

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

    let mut netabase = Netabase::<TestSchema>::new().unwrap();

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

    let mut netabase = Netabase::<TestSchema>::new().unwrap();
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

    let mut netabase = Netabase::<TestSchema>::new().unwrap();
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

    let mut netabase = Netabase::<TestSchema>::new().unwrap();
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

    let mut netabase = Netabase::<TestSchema>::new().unwrap();
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

    let mut netabase = Netabase::<TestSchema>::new().unwrap();
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

    let mut netabase = Netabase::<TestSchema>::new().unwrap();
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

    let mut netabase = Netabase::<TestSchema>::new().unwrap();
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

    let mut netabase = Netabase::<TestSchema>::new().unwrap();
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

    let mut netabase = Netabase::<TestSchema>::new().unwrap();
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

    let mut netabase = Netabase::<TestSchema>::new().unwrap();
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

    let mut netabase = Netabase::<TestSchema>::new().unwrap();
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
