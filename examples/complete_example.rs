//! Complete example demonstrating the NetabaseSchema derive macro
//!
//! This example shows how to use the NetabaseSchema derive macro to automatically
//! generate all the necessary trait implementations for working with libp2p Kademlia
//! records, including the automatic From<libp2p::kad::Record> implementation.

use bincode::{Decode, Encode};
use netabase::netabase_trait::NetabaseSchema as NetabaseSchemaInterface;
use netabase_macros::NetabaseSchema;

/// A simple user struct with automatic NetabaseSchema implementation
#[derive(Clone, Debug, NetabaseSchema, Encode, Decode)]
struct User {
    #[key]
    id: String,
    name: String,
    email: String,
    age: u32,
}

/// A product enum showing different variants with key fields
#[derive(Clone, Debug, NetabaseSchema, Encode, Decode)]
enum Product {
    Digital {
        #[key]
        sku: String,
        name: String,
        download_url: String,
        price: f64,
    },
    Physical {
        #[key]
        barcode: String,
        name: String,
        weight: f32,
        dimensions: (f32, f32, f32),
        price: f64,
    },
    Service {
        #[key]
        service_id: String,
        name: String,
        duration_hours: u32,
        price: f64,
    },
}

/// A blog post with automatic key generation
#[derive(Clone, Debug, NetabaseSchema, Encode, Decode)]
struct BlogPost {
    #[key]
    slug: String,
    title: String,
    content: String,
    author: String,
    published_at: u64, // Unix timestamp
    tags: Vec<String>,
}

fn main() {
    println!("=== NetabaseSchema Derive Macro Example ===\n");

    // Example 1: Basic struct usage
    basic_struct_example();

    // Example 2: Enum usage
    enum_example();

    // Example 3: Record conversion
    record_conversion_example();

    // Example 4: Key operations
    key_operations_example();

    println!("\n=== All examples completed successfully! ===");
}

fn basic_struct_example() {
    println!("1. Basic Struct Example:");
    println!("   Creating users with different IDs...");

    let user1 = User {
        id: "user_123".to_string(),
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        age: 28,
    };

    let user2 = User {
        id: "user_456".to_string(),
        name: "Bob Smith".to_string(),
        email: "bob@example.com".to_string(),
        age: 35,
    };

    // Each user has its own unique key based on its ID
    let key1 = user1.key();
    let key2 = user2.key();

    println!("   User 1 key: {}", key1.as_str());
    println!("   User 2 key: {}", key2.as_str());
    println!("   Keys are different: {}", key1.as_str() != key2.as_str());

    // Keys are properly generated per instance
    assert_eq!(key1.as_str(), "user_123");
    assert_eq!(key2.as_str(), "user_456");
    assert_ne!(key1.as_str(), key2.as_str());

    println!("   ✅ Basic struct example passed!\n");
}

fn enum_example() {
    println!("2. Enum Example:");
    println!("   Creating different product variants...");

    let digital = Product::Digital {
        sku: "SOFT_001".to_string(),
        name: "Premium Software".to_string(),
        download_url: "https://example.com/download/soft001".to_string(),
        price: 99.99,
    };

    let physical = Product::Physical {
        barcode: "123456789012".to_string(),
        name: "Wireless Headphones".to_string(),
        weight: 0.25,
        dimensions: (20.0, 15.0, 8.0),
        price: 149.99,
    };

    let service = Product::Service {
        service_id: "CONSULT_001".to_string(),
        name: "Software Consultation".to_string(),
        duration_hours: 2,
        price: 200.00,
    };

    // Each variant has its own key based on its key field
    let digital_key = digital.key();
    let physical_key = physical.key();
    let service_key = service.key();

    println!("   Digital product key: {}", digital_key.as_str());
    println!("   Physical product key: {}", physical_key.as_str());
    println!("   Service product key: {}", service_key.as_str());

    // Verify keys match the key fields
    assert_eq!(digital_key.as_str(), "SOFT_001");
    assert_eq!(physical_key.as_str(), "123456789012");
    assert_eq!(service_key.as_str(), "CONSULT_001");

    println!("   ✅ Enum example passed!\n");
}

fn record_conversion_example() {
    println!("3. Record Conversion Example:");
    println!("   Testing automatic From<libp2p::kad::Record> implementation...");

    let original_post = BlogPost {
        slug: "my-first-post".to_string(),
        title: "My First Blog Post".to_string(),
        content: "This is the content of my first blog post...".to_string(),
        author: "Jane Doe".to_string(),
        published_at: 1640995200, // Jan 1, 2022
        tags: vec!["rust".to_string(), "programming".to_string()],
    };

    println!("   Original post key: {}", original_post.key().as_str());

    // Convert to libp2p kad record (using the generated method)
    let record = original_post
        .to_kad_record()
        .expect("Failed to convert to record");

    println!("   Record key (bytes): {:?}", record.key.to_vec());
    println!("   Record value size: {} bytes", record.value.len());

    // Convert back from record using the auto-generated From implementation
    let recovered_post = BlogPost::from(record);

    println!("   Recovered post key: {}", recovered_post.key().as_str());

    // Verify the round-trip conversion worked
    assert_eq!(original_post.slug, recovered_post.slug);
    assert_eq!(original_post.title, recovered_post.title);
    assert_eq!(original_post.content, recovered_post.content);
    assert_eq!(original_post.author, recovered_post.author);
    assert_eq!(original_post.published_at, recovered_post.published_at);
    assert_eq!(original_post.tags, recovered_post.tags);
    assert_eq!(original_post.key().as_str(), recovered_post.key().as_str());

    println!("   ✅ Record conversion example passed!\n");
}

fn key_operations_example() {
    println!("4. Key Operations Example:");
    println!("   Testing key-related functionality...");

    let user = User {
        id: "key_test_user".to_string(),
        name: "Key Test User".to_string(),
        email: "keytest@example.com".to_string(),
        age: 30,
    };

    // Get the key
    let key = user.key();
    println!("   User key: {}", key.as_str());

    // Test key methods
    println!("   Key as string: {}", key.as_str());
    println!("   Key into string: {}", key.clone().into_string());

    // Create a new key manually
    let manual_key = UserKey::new("manual_key".to_string());
    println!("   Manual key: {}", manual_key.as_str());

    // Generate a random key
    let generated_key = UserKey::generate_key();
    println!("   Generated key: {}", generated_key.as_str());

    // Test key conversion to libp2p record key
    let record_key = libp2p::kad::RecordKey::from(key.clone());
    println!("   Record key (bytes): {:?}", record_key.to_vec());

    // Test key recovery from record
    let dummy_record = libp2p::kad::Record {
        key: record_key,
        value: vec![],
        publisher: None,
        expires: None,
    };
    let recovered_key = UserKey::from(dummy_record);
    println!("   Recovered key: {}", recovered_key.as_str());

    // Verify key round-trip
    assert_eq!(key.as_str(), recovered_key.as_str());

    println!("   ✅ Key operations example passed!\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_key_generation() {
        let user1 = User {
            id: "test1".to_string(),
            name: "Test User 1".to_string(),
            email: "test1@example.com".to_string(),
            age: 25,
        };

        let user2 = User {
            id: "test2".to_string(),
            name: "Test User 2".to_string(),
            email: "test2@example.com".to_string(),
            age: 30,
        };

        assert_eq!(user1.key().as_str(), "test1");
        assert_eq!(user2.key().as_str(), "test2");
        assert_ne!(user1.key().as_str(), user2.key().as_str());
    }

    #[test]
    fn test_product_enum_keys() {
        let digital = Product::Digital {
            sku: "TEST_SKU".to_string(),
            name: "Test Product".to_string(),
            download_url: "https://test.com".to_string(),
            price: 10.0,
        };

        let physical = Product::Physical {
            barcode: "TEST_BARCODE".to_string(),
            name: "Test Physical".to_string(),
            weight: 1.0,
            dimensions: (1.0, 1.0, 1.0),
            price: 20.0,
        };

        assert_eq!(digital.key().as_str(), "TEST_SKU");
        assert_eq!(physical.key().as_str(), "TEST_BARCODE");
        assert_ne!(digital.key().as_str(), physical.key().as_str());
    }

    #[test]
    fn test_record_round_trip() {
        let original = BlogPost {
            slug: "test-post".to_string(),
            title: "Test Post".to_string(),
            content: "Test content".to_string(),
            author: "Test Author".to_string(),
            published_at: 1234567890,
            tags: vec!["test".to_string()],
        };

        let record = original.to_kad_record().unwrap();
        let recovered = BlogPost::from(record);

        assert_eq!(original.slug, recovered.slug);
        assert_eq!(original.title, recovered.title);
        assert_eq!(original.content, recovered.content);
        assert_eq!(original.author, recovered.author);
        assert_eq!(original.published_at, recovered.published_at);
        assert_eq!(original.tags, recovered.tags);
        assert_eq!(original.key().as_str(), recovered.key().as_str());
    }

    #[test]
    fn test_key_serialization() {
        let user = User {
            id: "serialize_test".to_string(),
            name: "Serialize Test".to_string(),
            email: "serialize@test.com".to_string(),
            age: 25,
        };

        let key = user.key();

        // Test bincode serialization
        let encoded = bincode::encode_to_vec(&key, bincode::config::standard()).unwrap();
        let decoded: UserKey = bincode::decode_from_slice(&encoded, bincode::config::standard())
            .unwrap()
            .0;

        assert_eq!(key.as_str(), decoded.as_str());
    }

    #[test]
    fn test_automatic_from_implementation() {
        let user = User {
            id: "from_test".to_string(),
            name: "From Test".to_string(),
            email: "from@test.com".to_string(),
            age: 25,
        };

        // This uses the automatically generated From<libp2p::kad::Record> implementation
        let record = libp2p::kad::Record::from(user.clone());
        let recovered = User::from(record);

        assert_eq!(user.id, recovered.id);
        assert_eq!(user.name, recovered.name);
        assert_eq!(user.email, recovered.email);
        assert_eq!(user.age, recovered.age);
    }
}
