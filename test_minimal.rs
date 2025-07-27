//! Minimal test to debug the Encode/Decode implementation

use bincode::{Decode, Encode};
use netabase::netabase_trait::{NetabaseSchema, NetabaseSchemaKey};
use netabase_macros::NetabaseSchema;

/// Minimal test struct
#[derive(Clone, Debug, NetabaseSchema, serde::Serialize, serde::Deserialize)]
struct TestUser {
    #[key]
    id: String,
    name: String,
}

fn main() {
    let user = TestUser {
        id: "test123".to_string(),
        name: "Test User".to_string(),
    };

    println!("User: {:?}", user);
    println!("User key: {:?}", user.key());

    // Test that the key can generate itself
    let generated_key = TestUserKey::generate_key();
    println!("Generated key: {:?}", generated_key);

    // Test bincode serialization
    let encoded = bincode::encode_to_vec(&user, bincode::config::standard()).unwrap();
    println!("Encoded user: {} bytes", encoded.len());

    let decoded: TestUser = bincode::decode_from_slice(&encoded, bincode::config::standard())
        .unwrap()
        .0;
    println!("Decoded user: {:?}", decoded);
}
