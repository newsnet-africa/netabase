//! Simple test to verify From implementations work for schema conversion
//!
//! This test verifies that the derive_more From implementations are working
//! correctly for converting NetabaseModel instances to Schema enum variants.

use std::sync::Once;

use log::info;

mod shared_schema_lib;
use shared_schema_lib::{SenderData, SharedMultiProcessSchema, create_sender_data};

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format_timestamp_secs()
            .is_test(true)
            .init();
    });
}

#[test]
fn test_from_implementations() {
    init_logger();

    info!("🧪 Testing From implementations for schema conversion");

    // Create a SenderData instance
    let sender_data = create_sender_data(1, "Test message", "test_peer", 1);
    info!(
        "📝 Created SenderData: message_id = {}",
        sender_data.message_id
    );

    // Test From<SenderData> for SharedMultiProcessSchema
    let schema: SharedMultiProcessSchema = sender_data.clone().into();
    info!("✅ Successfully converted SenderData to Schema enum");

    // Verify the conversion worked correctly
    match schema {
        SharedMultiProcessSchema::SenderData(data) => {
            assert_eq!(data.message_id, 1);
            assert_eq!(data.content, "Test message");
            assert_eq!(data.sender_peer_id, "test_peer");
            info!("✅ Schema conversion verification passed");
        }
        _ => {
            panic!("❌ Schema conversion created wrong variant");
        }
    }

    info!("🎉 All From implementation tests passed!");
}

#[test]
fn test_all_schema_variants_from_conversion() {
    init_logger();

    info!("🧪 Testing From implementations for all schema variants");

    use shared_multiprocess_schema::{
        LateJoinerEvent, NetworkEvent, ReceiverEvent, create_late_joiner_event,
        create_network_event, create_receiver_event,
    };

    // Test SenderData
    let sender_data = create_sender_data(1, "Test", "peer1", 1);
    let _schema1: SharedMultiProcessSchema = sender_data.into();
    info!("✅ SenderData -> Schema conversion works");

    // Test NetworkEvent
    let network_event = create_network_event("test", "peer1", "details");
    let _schema2: SharedMultiProcessSchema = network_event.into();
    info!("✅ NetworkEvent -> Schema conversion works");

    // Test ReceiverEvent
    let receiver_event = create_receiver_event("peer1", "test", None, 1, "details");
    let _schema3: SharedMultiProcessSchema = receiver_event.into();
    info!("✅ ReceiverEvent -> Schema conversion works");

    // Test LateJoinerEvent
    let late_joiner_event = create_late_joiner_event("peer1", "test", 1, 5, "details");
    let _schema4: SharedMultiProcessSchema = late_joiner_event.into();
    info!("✅ LateJoinerEvent -> Schema conversion works");

    info!("🎉 All schema variant From conversions passed!");
}

#[test]
fn test_schema_serialization_roundtrip() {
    init_logger();

    info!("🧪 Testing schema serialization roundtrip with From conversion");

    // Create original data
    let original_data = create_sender_data(42, "Roundtrip test", "test_peer", 1);

    // Convert to schema using From
    let schema: SharedMultiProcessSchema = original_data.clone().into();

    // Serialize
    let serialized = bincode::encode_to_vec(&schema, bincode::config::standard())
        .expect("Serialization should work");
    info!("📦 Serialized schema to {} bytes", serialized.len());

    // Deserialize
    let (deserialized_schema, _): (SharedMultiProcessSchema, usize) =
        bincode::decode_from_slice(&serialized, bincode::config::standard())
            .expect("Deserialization should work");

    // Extract data back
    match deserialized_schema {
        SharedMultiProcessSchema::SenderData(data) => {
            assert_eq!(data.message_id, original_data.message_id);
            assert_eq!(data.content, original_data.content);
            assert_eq!(data.sender_peer_id, original_data.sender_peer_id);
            info!("✅ Roundtrip serialization verification passed");
        }
        _ => {
            panic!("❌ Deserialized wrong schema variant");
        }
    }

    info!("🎉 Schema serialization roundtrip test passed!");
}
