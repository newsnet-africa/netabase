//! Schema Compatibility Test for Multi-Process Communication
//!
//! This test validates that the shared schema can be serialized and deserialized
//! consistently across different compilation contexts to debug the inter-process
//! schema conversion errors.

use std::collections::HashMap;
use std::sync::Once;

use bincode::{Decode, Encode};
use log::{error, info};

use crate::shared_schema_lib::{
    LateJoinerEvent, NetworkEvent, ReceiverEvent, SenderData, SharedMultiProcessSchema,
    create_late_joiner_event, create_network_event, create_receiver_event, create_sender_data,
};

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .format_timestamp_secs()
            .is_test(true)
            .init();
    });
}

/// Test data creation functions using shared helper functions
fn create_test_sender_data() -> SenderData {
    create_sender_data(12345, "Test message content", "12D3KooWTestSender", 1)
}

fn create_test_network_event() -> NetworkEvent {
    create_network_event(
        "peer_discovered",
        "12D3KooWTestPeer",
        "Test peer discovery event",
    )
}

fn create_test_receiver_event() -> ReceiverEvent {
    create_receiver_event(
        "data_received",
        "12D3KooWTestReceiver",
        Some("12D3KooWTestSender"),
        5,
        "Test receiver event",
    )
}

fn create_test_late_joiner_event() -> LateJoinerEvent {
    create_late_joiner_event(
        "late_joiner_discovery",
        "12D3KooWTestLateJoiner",
        3,
        10,
        "Test late joiner event",
    )
}

/// Test direct bincode serialization/deserialization
fn test_bincode_roundtrip<T>(item: &T, type_name: &str) -> Result<(), String>
where
    T: Encode + Decode<()> + std::fmt::Debug + PartialEq,
{
    info!("Testing bincode roundtrip for {}", type_name);

    // Serialize
    let encoded = bincode::encode_to_vec(item, bincode::config::standard())
        .map_err(|e| format!("Failed to encode {}: {:?}", type_name, e))?;

    info!("Encoded {} to {} bytes", type_name, encoded.len());
    info!(
        "Encoded bytes (hex): {}",
        hex::encode(&encoded[..std::cmp::min(encoded.len(), 64)])
    );

    // Deserialize
    let (decoded, _) = bincode::decode_from_slice::<T, _>(&encoded, bincode::config::standard())
        .map_err(|e| format!("Failed to decode {}: {:?}", type_name, e))?;

    // Verify equality
    if item == &decoded {
        info!("✅ Bincode roundtrip successful for {}", type_name);
        Ok(())
    } else {
        error!(
            "❌ Bincode roundtrip failed for {}: original != decoded",
            type_name
        );
        Err(format!("Data mismatch for {}", type_name))
    }
}

/// Test schema enum conversion
fn test_schema_conversion<T>(item: T, type_name: &str) -> Result<(), String>
where
    T: Into<SharedMultiProcessSchema>
        + TryFrom<SharedMultiProcessSchema>
        + std::fmt::Debug
        + PartialEq,
    <T as TryFrom<SharedMultiProcessSchema>>::Error: std::fmt::Debug,
{
    info!("Testing schema conversion for {}", type_name);

    // Convert to schema enum
    let schema: SharedMultiProcessSchema = item.into();
    info!("Converted {} to schema enum: {:?}", type_name, schema);

    // Test bincode roundtrip of schema enum
    let encoded = bincode::encode_to_vec(&schema, bincode::config::standard())
        .map_err(|e| format!("Failed to encode schema for {}: {:?}", type_name, e))?;

    info!("Encoded schema {} to {} bytes", type_name, encoded.len());

    let (decoded_schema, _) = bincode::decode_from_slice::<SharedMultiProcessSchema, _>(
        &encoded,
        bincode::config::standard(),
    )
    .map_err(|e| format!("Failed to decode schema for {}: {:?}", type_name, e))?;

    // Verify schema roundtrip
    if schema == decoded_schema {
        info!("✅ Schema enum roundtrip successful for {}", type_name);
    } else {
        error!("❌ Schema enum roundtrip failed for {}", type_name);
        return Err(format!("Schema enum mismatch for {}", type_name));
    }

    // Convert back to original type
    let _recovered: T = decoded_schema
        .try_into()
        .map_err(|e| format!("Failed to convert schema back to {}: {:?}", type_name, e))?;

    info!("✅ Schema conversion successful for {}", type_name);
    Ok(())
}

#[tokio::test]
async fn test_schema_compatibility() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();

    info!("🧪 Starting Schema Compatibility Test");
    info!("======================================");

    // Create test instances
    let sender_data = create_test_sender_data();
    let network_event = create_test_network_event();
    let receiver_event = create_test_receiver_event();
    let late_joiner_event = create_test_late_joiner_event();

    info!("📊 Created test instances:");
    info!("  • SenderData: {:?}", sender_data);
    info!("  • NetworkEvent: {:?}", network_event);
    info!("  • ReceiverEvent: {:?}", receiver_event);
    info!("  • LateJoinerEvent: {:?}", late_joiner_event);

    // Test 1: Direct bincode serialization
    info!("\n🔍 Testing direct bincode serialization...");
    test_bincode_roundtrip(&sender_data, "SenderData")?;
    test_bincode_roundtrip(&network_event, "NetworkEvent")?;
    test_bincode_roundtrip(&receiver_event, "ReceiverEvent")?;
    test_bincode_roundtrip(&late_joiner_event, "LateJoinerEvent")?;

    // Test 2: Schema enum conversion
    info!("\n🔄 Testing schema enum conversion...");
    test_schema_conversion(sender_data.clone(), "SenderData")?;
    test_schema_conversion(network_event.clone(), "NetworkEvent")?;
    test_schema_conversion(receiver_event.clone(), "ReceiverEvent")?;
    test_schema_conversion(late_joiner_event.clone(), "LateJoinerEvent")?;

    // Test 3: Cross-type schema compatibility
    info!("\n🔀 Testing cross-type schema compatibility...");
    let mut schema_map = HashMap::new();

    // Convert all items to schema and store
    schema_map.insert("sender", SharedMultiProcessSchema::from(sender_data));
    schema_map.insert("network", SharedMultiProcessSchema::from(network_event));
    schema_map.insert("receiver", SharedMultiProcessSchema::from(receiver_event));
    schema_map.insert(
        "late_joiner",
        SharedMultiProcessSchema::from(late_joiner_event),
    );

    // Serialize all schemas
    for (name, schema) in &schema_map {
        let encoded = bincode::encode_to_vec(schema, bincode::config::standard())?;
        let (decoded, _) = bincode::decode_from_slice::<SharedMultiProcessSchema, _>(
            &encoded,
            bincode::config::standard(),
        )?;

        if schema == &decoded {
            info!("✅ Cross-type compatibility OK for {}", name);
        } else {
            error!("❌ Cross-type compatibility FAILED for {}", name);
            return Err(format!("Cross-type compatibility failed for {}", name).into());
        }
    }

    // Test 4: Simulated inter-process data exchange
    info!("\n📡 Testing simulated inter-process data exchange...");

    // Simulate sender creating a NetworkEvent (like peer discovery)
    let sender_network_event = create_network_event(
        "peer_discovered",
        "12D3KooWTestPeer",
        "mDNS peer discovery from receiver",
    );

    // Convert to schema and serialize (simulating network transmission)
    let schema = SharedMultiProcessSchema::from(sender_network_event.clone());
    let transmitted_bytes = bincode::encode_to_vec(&schema, bincode::config::standard())?;

    info!("Simulated transmission: {} bytes", transmitted_bytes.len());
    info!(
        "Transmitted bytes (hex): {}",
        hex::encode(&transmitted_bytes[..std::cmp::min(transmitted_bytes.len(), 64)])
    );

    // Deserialize on receiver side
    let (received_schema, _) = bincode::decode_from_slice::<SharedMultiProcessSchema, _>(
        &transmitted_bytes,
        bincode::config::standard(),
    )?;

    // Convert back to NetworkEvent
    let received_event: NetworkEvent = received_schema
        .try_into()
        .map_err(|e| format!("Failed to convert received schema to NetworkEvent: {:?}", e))?;

    if sender_network_event == received_event {
        info!("✅ Simulated inter-process exchange successful");
    } else {
        error!("❌ Simulated inter-process exchange failed");
        error!("Original: {:?}", sender_network_event);
        error!("Received: {:?}", received_event);
        return Err("Inter-process exchange simulation failed".into());
    }

    info!("\n🎉 All schema compatibility tests passed!");
    info!("The shared schema is working correctly in this compilation context.");
    info!("The inter-process errors must be caused by something else.");

    Ok(())
}

#[tokio::test]
async fn test_problematic_record_patterns() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();

    info!("🔍 Testing problematic record patterns from logs");
    info!("================================================");

    // Test the exact patterns that were failing in the logs
    let problematic_patterns = vec![
        ("peer_discovered_1759886391", "peer_discovered"),
        ("late_joiner_discovery_1759886132", "late_joiner_discovery"),
        ("late_joiner_discovery_1759886415", "late_joiner_discovery"),
    ];

    for (event_id, event_type) in problematic_patterns {
        info!("Testing pattern: {} -> {}", event_id, event_type);

        let test_event = if event_type.starts_with("late_joiner") {
            // Create LateJoinerEvent
            let late_joiner_event = create_late_joiner_event(
                event_type,
                "12D3KooWTestLateJoiner",
                1,
                0,
                "Late joiner discovered existing peer",
            );
            SharedMultiProcessSchema::from(late_joiner_event)
        } else {
            // Create NetworkEvent
            let network_event = create_network_event(
                event_type,
                "12D3KooWTestPeer",
                "mDNS peer discovery from receiver",
            );
            SharedMultiProcessSchema::from(network_event)
        };

        // Test serialization/deserialization
        let encoded = bincode::encode_to_vec(&test_event, bincode::config::standard())?;
        let (decoded, _) = bincode::decode_from_slice::<SharedMultiProcessSchema, _>(
            &encoded,
            bincode::config::standard(),
        )?;

        if test_event == decoded {
            info!("✅ Pattern {} works correctly", event_id);
        } else {
            error!("❌ Pattern {} failed", event_id);
            return Err(format!("Pattern {} failed", event_id).into());
        }
    }

    info!("✅ All problematic patterns work correctly in isolation");

    Ok(())
}
