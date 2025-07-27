//! Example demonstrating the generated trait implementations for NetabaseSchema and NetabaseSchemaKey
//!
//! This example shows how to use the `#[derive(NetabaseSchema)]` macro to automatically generate
//! implementations for the NetabaseSchema and NetabaseSchemaKey traits, including:
//! - Key extraction from marked fields
//! - Conversion to/from libp2p::kad::Record
//! - Bincode serialization/deserialization
//! - Utility methods for Kademlia integration

use bincode::{Decode, Encode};
use netabase::netabase_trait::{NetabaseSchema as NetabaseSchemaTrait, NetabaseSchemaKey};
use netabase_macros::NetabaseSchema;

/// Example user struct with a single key field
#[derive(Clone, Debug, NetabaseSchema, Encode, Decode)]
pub struct User {
    #[key]
    pub id: String,
    pub name: String,
    pub email: String,
    pub age: u32,
}

/// Example product struct with a different key field
#[derive(Clone, Debug, NetabaseSchema, Encode, Decode)]
pub struct Product {
    #[key]
    pub sku: String,
    pub name: String,
    pub price: f64,
    pub category: String,
}

/// Example enum with key fields in variants
#[derive(Clone, Debug, NetabaseSchema, Encode, Decode)]
pub enum Document {
    #[key]
    Text {
        #[key]
        content_hash: String,
        content: String,
    },
    #[key]
    Image {
        #[key]
        image_hash: String,
        data: Vec<u8>,
    },
    #[key]
    Video {
        #[key]
        video_id: String,
        duration: u64,
    },
}

/// Example showing usage of the generated implementations
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== NetabaseSchema Trait Implementation Examples ===\n");

    // Example 1: User struct
    println!("1. User Example:");
    let user = User {
        id: "user123".to_string(),
        name: "Alice Smith".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
    };

    // Access the key using the generated trait implementation
    println!("   User key: {}", user.key().as_str());

    // Convert to Kademlia record
    let kad_record = user.to_kad_record()?;
    println!("   Kademlia key: {:?}", kad_record.key);
    println!("   Record value size: {} bytes", kad_record.value.len());

    // Get just the Kademlia key
    let kad_key = user.kad_key();
    println!("   Direct kad key: {:?}", kad_key);

    // Example 2: Product struct
    println!("\n2. Product Example:");
    let product = Product {
        sku: "PROD-001".to_string(),
        name: "Laptop".to_string(),
        price: 999.99,
        category: "Electronics".to_string(),
    };

    println!("   Product key: {}", product.key().as_str());
    println!("   Product kad key: {:?}", product.kad_key());

    // Example 3: Document enum
    println!("\n3. Document Enum Examples:");

    let text_doc = Document::Text {
        content_hash: "hash123".to_string(),
        content: "Hello, world!".to_string(),
    };
    println!("   Text document key: {}", text_doc.key().as_str());

    let image_doc = Document::Image {
        image_hash: "img456".to_string(),
        data: vec![0x89, 0x50, 0x4E, 0x47], // PNG header
    };
    println!("   Image document key: {}", image_doc.key().as_str());

    let video_doc = Document::Video {
        video_id: "vid789".to_string(),
        duration: 3600,
    };
    println!("   Video document key: {}", video_doc.key().as_str());

    // Example 4: Key generation
    println!("\n4. Key Generation Example:");
    let generated_key = UserKey::generate_key();
    println!("   Generated user key: {}", generated_key.as_str());
    println!("   Generated key length: {}", generated_key.as_str().len());

    // Example 5: Round-trip serialization
    println!("\n5. Serialization Round-trip Example:");

    // Convert user to kad record
    let user_record = user.to_kad_record()?;
    println!("   Original user: {:?}", user);

    // Convert back from record
    let recovered_user = User::from(user_record);
    println!("   Recovered user: {:?}", recovered_user);

    // Verify keys match
    assert_eq!(user.key().as_str(), recovered_user.key().as_str());
    println!("   ✓ Keys match after round-trip");

    // Example 6: Key type utilities
    println!("\n6. Key Type Utilities:");
    let key_from_string = UserKey::new("custom_key_123".to_string());
    println!("   Custom key: {}", key_from_string.as_str());

    let key_string = key_from_string.clone().into_string();
    println!("   Key as string: {}", key_string);

    println!("\n=== All examples completed successfully! ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_key_extraction() {
        let user = User {
            id: "test123".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            age: 25,
        };

        // The key should be based on the id field since it's marked with #[key]
        assert_eq!(user.key().as_str(), "test123");
    }

    #[test]
    fn test_product_key_extraction() {
        let product = Product {
            sku: "SKU-999".to_string(),
            name: "Test Product".to_string(),
            price: 19.99,
            category: "Test".to_string(),
        };

        // The key should be based on the sku field since it's marked with #[key]
        assert_eq!(product.key().as_str(), "SKU-999");
    }

    #[test]
    fn test_document_enum_keys() {
        let text = Document::Text {
            content_hash: "text_hash".to_string(),
            content: "content".to_string(),
        };

        let image = Document::Image {
            image_hash: "image_hash".to_string(),
            data: vec![1, 2, 3],
        };

        // Each variant should extract the key from its respective key field
        assert_eq!(text.key().as_str(), "text_hash");
        assert_eq!(image.key().as_str(), "image_hash");
    }

    #[test]
    fn test_key_generation() {
        let key1 = UserKey::generate_key();
        let key2 = UserKey::generate_key();

        // Generated keys should be different
        assert_ne!(key1.as_str(), key2.as_str());

        // Both should be non-empty
        assert!(!key1.as_str().is_empty());
        assert!(!key2.as_str().is_empty());
    }

    #[test]
    fn test_kad_record_conversion() -> Result<(), Box<dyn std::error::Error>> {
        let user = User {
            id: "record_test".to_string(),
            name: "Record User".to_string(),
            email: "record@example.com".to_string(),
            age: 35,
        };

        // Convert to kad record
        let record = user.to_kad_record()?;

        // The key should match
        assert_eq!(String::from_utf8(record.key.to_vec())?, user.key().as_str());

        // Should be able to deserialize back
        let recovered = User::from(record);
        assert_eq!(user.id, recovered.id);
        assert_eq!(user.name, recovered.name);
        assert_eq!(user.email, recovered.email);
        assert_eq!(user.age, recovered.age);

        Ok(())
    }

    #[test]
    fn test_key_type_utilities() {
        let original_string = "test_key_123".to_string();
        let key = UserKey::new(original_string.clone());

        // as_str should return the same content
        assert_eq!(key.as_str(), original_string);

        // into_string should return the same content
        assert_eq!(key.into_string(), original_string);
    }
}
