//! Shared Schema Library for Multi-Process Tests
//!
//! This module defines a unified schema that all multi-process tests
//! (sender, receiver, late_joiner) can use to ensure compatibility
//! when communicating over the network via Kademlia DHT.
//!
//! All tests should import and use this schema to avoid serialization
//! errors when processes try to exchange data.

use bincode::{Decode, Encode};
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::traits::NetabaseModel as NetabaseModelTrait;
use serde::{Deserialize, Serialize};

/// Generate current timestamp in seconds
pub fn current_timestamp_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// Unified test schema for multi-process communication
#[netabase_schema_module(SharedMultiProcessSchema, SharedMultiProcessKeys)]
pub mod shared_multiprocess_schema {
    use super::*;

    /// Data sent by sender processes
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(SenderDataKey)]
    pub struct SenderData {
        #[key]
        pub message_id: u64,
        pub content: String,
        pub sender_peer_id: String,
        pub receiver_peer_id: Option<String>,
        pub timestamp: u64,
        pub message_type: String,
        pub payload_size: usize,
        pub sequence_number: u32,
        pub sender_process_id: String,
    }

    /// General network events
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(NetworkEventKey)]
    pub struct NetworkEvent {
        #[key]
        pub event_id: String,
        pub event_type: String,
        pub peer_id: String,
        pub timestamp: u64,
        pub details: String,
    }

    /// Events recorded by receiver processes
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(ReceiverEventKey)]
    pub struct ReceiverEvent {
        #[key]
        pub event_id: String,
        pub receiver_peer_id: String,
        pub event_type: String,
        pub sender_peer_id: Option<String>,
        pub message_count: u32,
        pub timestamp: u64,
        pub details: String,
    }

    /// Events recorded by late joiner processes
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(LateJoinerEventKey)]
    pub struct LateJoinerEvent {
        #[key]
        pub event_id: String,
        pub joiner_peer_id: String,
        pub event_type: String,
        pub cycle_number: u32,
        pub messages_found: u32,
        pub timestamp: u64,
        pub details: String,
    }
}

// Re-export the schema and types for easy use
pub use shared_multiprocess_schema::{
    LateJoinerEvent, NetworkEvent, ReceiverEvent, SenderData, SharedMultiProcessKeys,
    SharedMultiProcessSchema,
};

/// Helper functions for creating test data

/// Create test data message
pub fn create_sender_data(
    message_id: u64,
    content: &str,
    sender_peer_id: &str,
    sequence_number: u32,
) -> SenderData {
    let process_id = format!("sender_process_{}", std::process::id());
    SenderData {
        message_id,
        content: content.to_string(),
        sender_peer_id: sender_peer_id.to_string(),
        receiver_peer_id: None,
        timestamp: current_timestamp_secs(),
        message_type: "broadcast_data".to_string(),
        payload_size: content.len(),
        sequence_number,
        sender_process_id: process_id,
    }
}

/// Create network event record
pub fn create_network_event(event_type: &str, peer_id: &str, details: &str) -> NetworkEvent {
    NetworkEvent {
        event_id: format!("{}_{}", event_type, current_timestamp_secs()),
        event_type: event_type.to_string(),
        peer_id: peer_id.to_string(),
        timestamp: current_timestamp_secs(),
        details: details.to_string(),
    }
}

/// Create receiver event record
pub fn create_receiver_event(
    event_type: &str,
    receiver_peer_id: &str,
    sender_peer_id: Option<&str>,
    message_count: u32,
    details: &str,
) -> ReceiverEvent {
    ReceiverEvent {
        event_id: format!("recv_{}_{}", event_type, current_timestamp_secs()),
        receiver_peer_id: receiver_peer_id.to_string(),
        event_type: event_type.to_string(),
        sender_peer_id: sender_peer_id.map(|s| s.to_string()),
        message_count,
        timestamp: current_timestamp_secs(),
        details: details.to_string(),
    }
}

/// Create late joiner event record
pub fn create_late_joiner_event(
    event_type: &str,
    joiner_peer_id: &str,
    cycle_number: u32,
    messages_found: u32,
    details: &str,
) -> LateJoinerEvent {
    LateJoinerEvent {
        event_id: format!("late_{}_{}", event_type, current_timestamp_secs()),
        joiner_peer_id: joiner_peer_id.to_string(),
        event_type: event_type.to_string(),
        cycle_number,
        messages_found,
        timestamp: current_timestamp_secs(),
        details: details.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_schema_creation() {
        // Test that we can create all schema types
        let sender_data = create_sender_data(1, "test message", "sender123", 1);
        let network_event = create_network_event("test", "peer123", "test details");
        let receiver_event = create_receiver_event(
            "message_received",
            "receiver123",
            Some("sender123"),
            1,
            "received test",
        );
        let late_joiner_event =
            create_late_joiner_event("cycle_complete", "joiner123", 1, 5, "found 5 messages");

        assert_eq!(sender_data.message_id, 1);
        assert_eq!(network_event.event_type, "test");
        assert_eq!(receiver_event.message_count, 1);
        assert_eq!(late_joiner_event.cycle_number, 1);
    }
}
