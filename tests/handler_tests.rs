use std::sync::Once;
use std::time::Duration;

use bincode::{Decode, Encode};
use libp2p::{Multiaddr, PeerId, kad::Mode};
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

// Test schema for handler tests
#[netabase_schema_module(HandlerTestSchema, HandlerTestSchemaKeys)]
pub mod handler_test_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(HandlerTestUserKey)]
    pub struct HandlerTestUser {
        #[key]
        pub id: u64,
        pub name: String,
        pub email: String,
        pub handler_test_flag: bool,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(HandlerTestDataKey)]
    pub struct HandlerTestData {
        #[key]
        pub id: u64,
        pub content: String,
        pub metadata: std::collections::HashMap<String, String>,
    }
}

use handler_test_schema::{HandlerTestData, HandlerTestSchema, HandlerTestUser};

/// Generate a unique database path for each test to avoid conflicts
fn generate_unique_db_path() -> String {
    let uuid = Uuid::new_v4();
    format!("test_handler_db_{}", uuid.to_string().replace("-", "_"))
}

/// Create test user for handler testing
fn create_handler_test_user(id: u64) -> HandlerTestUser {
    HandlerTestUser {
        id,
        name: format!("Handler Test User {}", id),
        email: format!("handler_user{}@test.com", id),
        handler_test_flag: true,
    }
}

/// Create test data for handler testing
fn create_handler_test_data(id: u64) -> HandlerTestData {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("test_type".to_string(), "handler".to_string());
    metadata.insert("data_id".to_string(), id.to_string());

    HandlerTestData {
        id,
        content: format!("Handler test content {}", id),
        metadata,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_put_record_handler() {
    init_logger();

    let db_path = generate_unique_db_path();
    let mut netabase = Netabase::<HandlerTestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    // Give the swarm time to initialize
    tokio::time::sleep(Duration::from_millis(200)).await;

    let test_user = create_handler_test_user(1001);

    // Test the put_record handler
    let result = timeout(
        Duration::from_secs(5),
        netabase.put_record(test_user.clone()),
    )
    .await;

    match result {
        Ok(put_result) => {
            match put_result {
                Ok(query_result) => {
                    println!("Put record handler test successful: {:?}", query_result);

                    // Verify the query result type
                    match query_result {
                        libp2p::kad::QueryResult::PutRecord(put_record_result) => {
                            match put_record_result {
                                Ok(put_record_ok) => {
                                    println!("Put record OK: {:?}", put_record_ok);
                                }
                                Err(put_record_err) => {
                                    println!("Put record error: {:?}", put_record_err);
                                }
                            }
                        }
                        other => {
                            println!("Unexpected query result type: {:?}", other);
                        }
                    }
                }
                Err(e) => {
                    println!("Put record handler failed: {:?}", e);
                }
            }
        }
        Err(_) => {
            println!("Put record handler timed out - may be expected in single-node setup");
        }
    }

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_record_handler() {
    init_logger();

    let db_path = generate_unique_db_path();
    let mut netabase = Netabase::<HandlerTestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    let test_user = create_handler_test_user(1002);

    // First try to put a record
    let _ = timeout(
        Duration::from_secs(2),
        netabase.put_record(test_user.clone()),
    )
    .await;

    // Test the get_record handler using the user's key
    let result = timeout(Duration::from_secs(5), netabase.get_record(test_user.key())).await;

    match result {
        Ok(get_result) => {
            match get_result {
                Ok(query_result) => {
                    println!("Get record handler test successful: {:?}", query_result);

                    // Verify the query result type
                    match query_result {
                        libp2p::kad::QueryResult::GetRecord(get_record_result) => {
                            match get_record_result {
                                Ok(get_record_ok) => {
                                    println!("Get record OK: {:?}", get_record_ok);
                                }
                                Err(get_record_err) => {
                                    println!(
                                        "Get record error (may be expected): {:?}",
                                        get_record_err
                                    );
                                }
                            }
                        }
                        other => {
                            println!("Unexpected query result type: {:?}", other);
                        }
                    }
                }
                Err(e) => {
                    println!("Get record handler failed: {:?}", e);
                }
            }
        }
        Err(_) => {
            println!("Get record handler timed out");
        }
    }

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_start_providing_handler() {
    init_logger();

    let db_path = generate_unique_db_path();
    let mut netabase = Netabase::<HandlerTestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    let test_user = create_handler_test_user(1003);

    // Test the start_providing handler using the user's key
    let result = timeout(
        Duration::from_secs(5),
        netabase.start_providing(test_user.key()),
    )
    .await;

    match result {
        Ok(provide_result) => {
            match provide_result {
                Ok(query_result) => {
                    println!(
                        "Start providing handler test successful: {:?}",
                        query_result
                    );

                    // Verify the query result type
                    match query_result {
                        libp2p::kad::QueryResult::StartProviding(start_providing_result) => {
                            match start_providing_result {
                                Ok(start_providing_ok) => {
                                    println!("Start providing OK: {:?}", start_providing_ok);
                                }
                                Err(start_providing_err) => {
                                    println!("Start providing error: {:?}", start_providing_err);
                                }
                            }
                        }
                        other => {
                            println!("Unexpected query result type: {:?}", other);
                        }
                    }
                }
                Err(e) => {
                    println!("Start providing handler failed: {:?}", e);
                }
            }
        }
        Err(_) => {
            println!("Start providing handler timed out");
        }
    }

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_providers_handler() {
    init_logger();

    let db_path = generate_unique_db_path();
    let mut netabase = Netabase::<HandlerTestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    let test_user = create_handler_test_user(1004);

    // First try to start providing for the key
    let _ = timeout(
        Duration::from_secs(2),
        netabase.start_providing(test_user.key()),
    )
    .await;

    // Test the get_providers handler using the user's key
    let result = timeout(
        Duration::from_secs(5),
        netabase.get_providers(test_user.key()),
    )
    .await;

    match result {
        Ok(providers_result) => {
            match providers_result {
                Ok(query_result) => {
                    println!("Get providers handler test successful: {:?}", query_result);

                    // Verify the query result type
                    match query_result {
                        libp2p::kad::QueryResult::GetProviders(get_providers_result) => {
                            match get_providers_result {
                                Ok(get_providers_ok) => {
                                    println!("Get providers OK: {:?}", get_providers_ok);
                                }
                                Err(get_providers_err) => {
                                    println!(
                                        "Get providers error (may be expected): {:?}",
                                        get_providers_err
                                    );
                                }
                            }
                        }
                        other => {
                            println!("Unexpected query result type: {:?}", other);
                        }
                    }
                }
                Err(e) => {
                    println!("Get providers handler failed: {:?}", e);
                }
            }
        }
        Err(_) => {
            println!("Get providers handler timed out");
        }
    }

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_bootstrap_handler() {
    init_logger();

    let db_path = generate_unique_db_path();
    let mut netabase = Netabase::<HandlerTestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Test the bootstrap handler
    let result = timeout(Duration::from_secs(5), netabase.bootstrap()).await;

    match result {
        Ok(bootstrap_result) => {
            match bootstrap_result {
                Ok(query_result) => {
                    println!("Bootstrap handler test successful: {:?}", query_result);

                    // Verify the query result type
                    match query_result {
                        libp2p::kad::QueryResult::Bootstrap(bootstrap_result) => {
                            match bootstrap_result {
                                Ok(bootstrap_ok) => {
                                    println!("Bootstrap OK: {:?}", bootstrap_ok);
                                }
                                Err(bootstrap_err) => {
                                    println!(
                                        "Bootstrap error (expected with no peers): {:?}",
                                        bootstrap_err
                                    );
                                }
                            }
                        }
                        other => {
                            println!("Unexpected query result type: {:?}", other);
                        }
                    }
                }
                Err(e) => {
                    println!("Bootstrap handler failed (expected): {:?}", e);
                    // Bootstrap typically fails when no known peers exist
                }
            }
        }
        Err(_) => {
            println!("Bootstrap handler timed out");
        }
    }

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_mode_handlers() {
    init_logger();

    let db_path = generate_unique_db_path();
    let mut netabase = Netabase::<HandlerTestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Test get_mode handler
    let get_mode_result = timeout(Duration::from_secs(2), netabase.get_mode()).await;
    assert!(
        get_mode_result.is_ok(),
        "Get mode handler should not timeout"
    );

    let current_mode = get_mode_result.unwrap().unwrap();
    println!("Current mode from handler: {:?}", current_mode);

    // Test set_mode handler
    let new_mode = match current_mode {
        Mode::Client => Some(Mode::Server),
        Mode::Server => Some(Mode::Client),
    };

    let set_mode_result = timeout(Duration::from_secs(2), netabase.set_mode(new_mode)).await;
    assert!(
        set_mode_result.is_ok(),
        "Set mode handler should not timeout"
    );
    println!("Set mode handler completed successfully");

    // Verify the mode was changed
    let verify_mode_result = timeout(Duration::from_secs(2), netabase.get_mode()).await;
    assert!(
        verify_mode_result.is_ok(),
        "Verify mode handler should not timeout"
    );

    let verified_mode = verify_mode_result.unwrap().unwrap();
    println!("Verified mode from handler: {:?}", verified_mode);

    if let Some(expected_mode) = new_mode {
        assert_eq!(verified_mode, expected_mode, "Mode should have changed");
    }

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_protocol_names_handler() {
    init_logger();

    let db_path = generate_unique_db_path();
    let mut netabase = Netabase::<HandlerTestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Test get_protocol_names handler
    let result = timeout(Duration::from_secs(2), netabase.get_protocol_names()).await;
    assert!(
        result.is_ok(),
        "Get protocol names handler should not timeout"
    );

    let protocol = result.unwrap().unwrap();
    println!("Protocol name from handler: {:?}", protocol);

    // Verify the protocol name is a valid StreamProtocol
    let protocol_str = protocol.to_string();
    assert!(
        !protocol_str.is_empty(),
        "Protocol name should not be empty"
    );
    println!("Protocol string representation: {}", protocol_str);

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_peer_management_handlers() {
    init_logger();

    let db_path = generate_unique_db_path();
    let mut netabase = Netabase::<HandlerTestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Create test peer and address
    let test_peer = PeerId::random();
    let test_address: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();

    // Test add_address handler
    let add_result = timeout(
        Duration::from_secs(2),
        netabase.add_address(test_peer, test_address.clone()),
    )
    .await;
    assert!(add_result.is_ok(), "Add address handler should not timeout");

    let routing_update = add_result.unwrap().unwrap();
    println!("Add address handler result: {:?}", routing_update);

    // Test remove_address handler
    let remove_result = timeout(
        Duration::from_secs(2),
        netabase.remove_address(test_peer, test_address.clone()),
    )
    .await;
    assert!(
        remove_result.is_ok(),
        "Remove address handler should not timeout"
    );

    let remove_address_result = remove_result.unwrap().unwrap();
    println!("Remove address handler result: {:?}", remove_address_result);

    // Test remove_peer handler
    let remove_peer_result = timeout(Duration::from_secs(2), netabase.remove_peer(test_peer)).await;
    assert!(
        remove_peer_result.is_ok(),
        "Remove peer handler should not timeout"
    );

    let remove_peer_entry = remove_peer_result.unwrap().unwrap();
    println!("Remove peer handler result: {:?}", remove_peer_entry);

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_record_removal_handler() {
    init_logger();

    let db_path = generate_unique_db_path();
    let mut netabase = Netabase::<HandlerTestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    let test_user = create_handler_test_user(1005);

    // Test remove_record handler using the user's key
    let result = timeout(
        Duration::from_secs(2),
        netabase.remove_record(test_user.key()),
    )
    .await;
    assert!(result.is_ok(), "Remove record handler should not timeout");

    let remove_result = result.unwrap();
    assert!(
        remove_result.is_ok(),
        "Remove record handler should succeed"
    );
    println!("Remove record handler completed successfully");

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_stop_providing_handler() {
    init_logger();

    let db_path = generate_unique_db_path();
    let mut netabase = Netabase::<HandlerTestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    let test_user = create_handler_test_user(1006);

    // First start providing
    let _ = timeout(
        Duration::from_secs(2),
        netabase.start_providing(test_user.key()),
    )
    .await;

    // Test stop_providing handler using the user's key
    let result = timeout(
        Duration::from_secs(2),
        netabase.stop_providing(test_user.key()),
    )
    .await;
    assert!(result.is_ok(), "Stop providing handler should not timeout");

    let stop_result = result.unwrap();
    assert!(stop_result.is_ok(), "Stop providing handler should succeed");
    println!("Stop providing handler completed successfully");

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_multiple_record_types_handlers() {
    init_logger();

    let db_path = generate_unique_db_path();
    let mut netabase = Netabase::<HandlerTestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    let test_user = create_handler_test_user(2001);
    let test_data = create_handler_test_data(2001);

    // Test handlers with different record types
    let user_put_result = timeout(
        Duration::from_secs(3),
        netabase.put_record(test_user.clone()),
    )
    .await;

    let data_put_result = timeout(
        Duration::from_secs(3),
        netabase.put_record(test_data.clone()),
    )
    .await;

    println!("User put handler result: {:?}", user_put_result);
    println!("Data put handler result: {:?}", data_put_result);

    // Test get handlers with different key types using the models' keys
    let user_get_result =
        timeout(Duration::from_secs(3), netabase.get_record(test_user.key())).await;
    let data_get_result =
        timeout(Duration::from_secs(3), netabase.get_record(test_data.key())).await;

    println!("User get handler result: {:?}", user_get_result);
    println!("Data get handler result: {:?}", data_get_result);

    netabase.stop_swarm().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_handler_error_scenarios() {
    init_logger();

    let db_path = generate_unique_db_path();
    let mut netabase = Netabase::<HandlerTestSchema>::new_with_path(&db_path).unwrap();
    netabase.start_swarm().await.unwrap();

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Test handlers with non-existent data
    let non_existent_user = create_handler_test_user(99999);

    // Test get_record with non-existent key
    let get_result = timeout(
        Duration::from_secs(3),
        netabase.get_record(non_existent_user.key()),
    )
    .await;

    match get_result {
        Ok(result) => match result {
            Ok(query_result) => {
                println!("Get non-existent record handler result: {:?}", query_result);

                if let libp2p::kad::QueryResult::GetRecord(get_record_result) = query_result {
                    match get_record_result {
                        Ok(_) => {
                            println!("Unexpectedly found record for non-existent key");
                        }
                        Err(e) => {
                            println!("Expected error for non-existent record: {:?}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Get record handler error: {:?}", e);
            }
        },
        Err(_) => {
            println!("Get non-existent record timed out");
        }
    }

    // Test get_providers with non-existent key
    let providers_result = timeout(
        Duration::from_secs(3),
        netabase.get_providers(non_existent_user.key()),
    )
    .await;

    match providers_result {
        Ok(result) => match result {
            Ok(query_result) => {
                println!(
                    "Get providers for non-existent key result: {:?}",
                    query_result
                );
            }
            Err(e) => {
                println!("Get providers handler error: {:?}", e);
            }
        },
        Err(_) => {
            println!("Get providers for non-existent key timed out");
        }
    }

    // Test removing non-existent record
    let remove_result = timeout(
        Duration::from_secs(2),
        netabase.remove_record(non_existent_user.key()),
    )
    .await;

    assert!(
        remove_result.is_ok(),
        "Remove non-existent record should not timeout"
    );
    println!("Remove non-existent record handler completed successfully");

    netabase.stop_swarm().await.unwrap();
}
