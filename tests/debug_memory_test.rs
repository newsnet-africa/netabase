//! Debug Memory Test
//!
//! A simple test to debug the libp2p QueryResult structure and understand
//! the correct field names for accessing record data.

use std::time::Duration;

use bincode::{Decode, Encode};
use log::{error, info};
use netabase::Netabase;
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::traits::NetabaseModel as NetabaseModelTrait;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

static INIT: std::sync::Once = std::sync::Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .format_timestamp_secs()
            .init();
    });
}

// Simple test schema
#[netabase_schema_module(DebugSchema, DebugKeys)]
mod debug_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(DebugMessageKey)]
    pub struct DebugMessage {
        #[key]
        pub id: u64,
        pub content: String,
    }
}

use debug_schema::{DebugMessage, DebugSchema};

#[tokio::test]
#[cfg(feature = "memory")]
async fn debug_query_result_structure() {
    init_logger();

    info!("🔍 Debug test: Understanding QueryResult structure");

    // Create memory-based netabase
    let mut netabase = Netabase::<DebugSchema>::new_with_memory().unwrap();
    netabase.start_swarm().await.unwrap();

    // Wait a moment for initialization
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Create test message
    let test_msg = DebugMessage {
        id: 1,
        content: "Debug test message".to_string(),
    };

    info!("📤 Storing test message: {:?}", test_msg);

    // Store the message
    match timeout(
        Duration::from_secs(10),
        netabase.put_record(test_msg.clone()),
    )
    .await
    {
        Ok(Ok(query_result)) => {
            info!("✅ Put operation successful");
            info!("📊 Put QueryResult structure: {:#?}", query_result);
        }
        Ok(Err(e)) => {
            error!("❌ Put operation failed: {:?}", e);
        }
        Err(_) => {
            error!("⏰ Put operation timed out");
        }
    }

    // Wait for propagation
    tokio::time::sleep(Duration::from_secs(3)).await;

    info!("📥 Attempting to retrieve test message");

    // Try to retrieve the message
    match timeout(Duration::from_secs(10), netabase.get_record(test_msg.key())).await {
        Ok(Ok(query_result)) => {
            info!("✅ Get operation successful");
            info!("📊 Get QueryResult structure: {:#?}", query_result);

            // Let's see what's inside the QueryResult
            match query_result {
                libp2p::kad::QueryResult::GetRecord(get_record_result) => {
                    info!("🔍 This is a GetRecord result");
                    match get_record_result {
                        Ok(get_record_ok) => {
                            info!("🔍 GetRecord OK structure: {:#?}", get_record_ok);
                            // This will show us the actual field names we need to use
                        }
                        Err(get_record_err) => {
                            info!("🔍 GetRecord error: {:#?}", get_record_err);
                        }
                    }
                }
                other => {
                    info!("🔍 Unexpected query result type: {:#?}", other);
                }
            }
        }
        Ok(Err(e)) => {
            error!("❌ Get operation failed: {:?}", e);
        }
        Err(_) => {
            error!("⏰ Get operation timed out");
        }
    }

    // Test provider functionality
    info!("🔄 Testing provider functionality");

    match timeout(
        Duration::from_secs(10),
        netabase.start_providing(test_msg.key()),
    )
    .await
    {
        Ok(Ok(query_result)) => {
            info!("✅ Start providing successful");
            info!("📊 Start providing QueryResult: {:#?}", query_result);
        }
        Ok(Err(e)) => {
            error!("❌ Start providing failed: {:?}", e);
        }
        Err(_) => {
            error!("⏰ Start providing timed out");
        }
    }

    // Wait for provider record propagation
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Get providers
    match timeout(
        Duration::from_secs(10),
        netabase.get_providers(test_msg.key()),
    )
    .await
    {
        Ok(Ok(query_result)) => {
            info!("✅ Get providers successful");
            info!("📊 Get providers QueryResult: {:#?}", query_result);

            match query_result {
                libp2p::kad::QueryResult::GetProviders(get_providers_result) => {
                    info!("🔍 This is a GetProviders result");
                    match get_providers_result {
                        Ok(get_providers_ok) => {
                            info!("🔍 GetProviders OK structure: {:#?}", get_providers_ok);
                            // This will show us the actual field names for providers
                        }
                        Err(get_providers_err) => {
                            info!("🔍 GetProviders error: {:#?}", get_providers_err);
                        }
                    }
                }
                other => {
                    info!("🔍 Unexpected query result type: {:#?}", other);
                }
            }
        }
        Ok(Err(e)) => {
            error!("❌ Get providers failed: {:?}", e);
        }
        Err(_) => {
            error!("⏰ Get providers timed out");
        }
    }

    // Cleanup
    netabase.stop_swarm().await.unwrap();
    info!("🏁 Debug test completed");
}
