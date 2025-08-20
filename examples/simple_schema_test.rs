use bincode::{Decode, Encode};
use netabase::NetabaseSchema;
use serde::{Deserialize, Serialize};

// Test basic NetabaseSchema derive without schema attribute
#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
pub struct SimpleUser {
    #[key]
    pub id: u64,
    pub name: String,
}

#[derive(Serialize, Deserialize, NetabaseSchema, Clone, Debug, PartialEq, Encode, Decode)]
pub struct SimpleMessage {
    #[key]
    pub msg_id: String,
    pub content: String,
}

fn main() {
    println!("=== Simple Schema Test ===");

    // Test that the NetabaseSchema trait is properly implemented
    let user = SimpleUser {
        id: 42,
        name: "Test User".to_string(),
    };

    let message = SimpleMessage {
        msg_id: "msg_001".to_string(),
        content: "Hello, World!".to_string(),
    };

    // Test key generation
    println!("User key: {}", user.key());
    println!("Message key: {}", message.key());

    // Test that cloning works
    let user_clone = user.clone();
    println!("Cloned user key: {}", user_clone.key());

    // Test that the schema derivation worked properly
    assert_eq!(user.key().to_string(), "42");
    assert_eq!(message.key().to_string(), "msg_001");

    println!("✓ Schema attribute macro test completed successfully!");
}
