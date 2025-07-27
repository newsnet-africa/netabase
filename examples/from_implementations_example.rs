//! Comprehensive example demonstrating all From implementations for NetabaseSchema
//!
//! This example showcases the bidirectional conversion capabilities between:
//! - Schema types and libp2p::kad::Record
//! - Key types and libp2p::kad::RecordKey
//! - Round-trip conversions and practical usage patterns

use bincode::{Decode, Encode};
use netabase::netabase_trait::{NetabaseSchema as NetabaseSchemaTrait, NetabaseSchemaKey};
use netabase_macros::NetabaseSchema;

/// Example user schema with string key
#[derive(Clone, Debug, PartialEq, NetabaseSchema, Encode, Decode)]
struct User {
    #[key]
    id: String,
    name: String,
    email: String,
    age: u32,
}

/// Example product schema with numeric key
#[derive(Clone, Debug, PartialEq, NetabaseSchema, Encode, Decode)]
struct Product {
    #[key]
    sku: u64,
    name: String,
    price: f64,
    category: String,
}

/// Example document enum with variant-specific keys
#[derive(Clone, Debug, PartialEq, NetabaseSchema, Encode, Decode)]
enum Document {
    Text {
        #[key]
        content_hash: String,
        content: String,
        word_count: usize,
    },
    Image {
        #[key]
        image_id: String,
        format: String,
        size_bytes: u64,
    },
    Video {
        #[key]
        video_id: u64,
        duration_seconds: u32,
        resolution: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== NetabaseSchema From Implementations Demo ===\n");

    // Example 1: Basic From conversions with User
    demo_basic_conversions()?;

    // Example 2: Enum conversions
    demo_enum_conversions()?;

    // Example 3: Key-only conversions
    demo_key_conversions()?;

    // Example 4: Round-trip conversions
    demo_round_trip_conversions()?;

    // Example 5: Practical Kademlia usage patterns
    demo_kademlia_patterns()?;

    // Example 6: Bulk operations
    demo_bulk_operations()?;

    println!("\n=== All From implementation examples completed successfully! ===");
    Ok(())
}

/// Example 1: Basic From conversions between schema types and Records
fn demo_basic_conversions() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Basic From Conversions:");

    let user = User {
        id: "user_001".to_string(),
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        age: 28,
    };

    // Convert User to Record using From trait
    println!("   Converting User to Record...");
    let record: libp2p::kad::Record = user.clone().into();

    // Verify the conversion
    let key_string = String::from_utf8(record.key.to_vec())?;
    println!("   Record key: {}", key_string);
    println!("   Record value size: {} bytes", record.value.len());
    assert_eq!(key_string, user.key().as_str());

    // Convert Record back to User using From trait
    println!("   Converting Record back to User...");
    let recovered_user: User = record.into();

    // Verify round-trip integrity
    assert_eq!(user, recovered_user);
    println!("   ✓ Round-trip conversion successful");
    println!("   Original: {:?}", user);
    println!("   Recovered: {:?}\n", recovered_user);

    Ok(())
}

/// Example 2: From conversions with enum types
fn demo_enum_conversions() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Enum From Conversions:");

    let text_doc = Document::Text {
        content_hash: "abc123def456".to_string(),
        content: "This is a sample document for testing purposes.".to_string(),
        word_count: 9,
    };

    let image_doc = Document::Image {
        image_id: "img_789".to_string(),
        format: "PNG".to_string(),
        size_bytes: 1024 * 512, // 512KB
    };

    let video_doc = Document::Video {
        video_id: 42,
        duration_seconds: 3600, // 1 hour
        resolution: "1920x1080".to_string(),
    };

    // Convert each enum variant to Record
    let text_record: libp2p::kad::Record = text_doc.clone().into();
    let image_record: libp2p::kad::Record = image_doc.clone().into();
    let video_record: libp2p::kad::Record = video_doc.clone().into();

    // Verify keys match expected values
    println!(
        "   Text document key: {}",
        String::from_utf8(text_record.key.to_vec())?
    );
    println!(
        "   Image document key: {}",
        String::from_utf8(image_record.key.to_vec())?
    );
    println!(
        "   Video document key: {}",
        String::from_utf8(video_record.key.to_vec())?
    );

    // Convert back and verify
    let recovered_text: Document = text_record.into();
    let recovered_image: Document = image_record.into();
    let recovered_video: Document = video_record.into();

    assert_eq!(text_doc, recovered_text);
    assert_eq!(image_doc, recovered_image);
    assert_eq!(video_doc, recovered_video);
    println!("   ✓ All enum variant conversions successful\n");

    Ok(())
}

/// Example 3: Key-only conversions
fn demo_key_conversions() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Key-Only Conversions:");

    // Create custom keys
    let user_key = UserKey::new("custom_user_123".to_string());
    let product_key = ProductKey::new("SKU-999".to_string());

    // Convert keys to RecordKey using From trait
    let user_record_key: libp2p::kad::RecordKey = user_key.clone().into();
    let product_record_key: libp2p::kad::RecordKey = product_key.clone().into();

    // Verify conversions
    println!(
        "   User key: {} -> RecordKey: {}",
        user_key.as_str(),
        String::from_utf8(user_record_key.to_vec())?
    );

    println!(
        "   Product key: {} -> RecordKey: {}",
        product_key.as_str(),
        String::from_utf8(product_record_key.to_vec())?
    );

    // Test with generated keys
    let generated_key = UserKey::generate_key();
    let generated_record_key: libp2p::kad::RecordKey = generated_key.clone().into();

    println!(
        "   Generated key: {} -> RecordKey: {}",
        generated_key.as_str(),
        String::from_utf8(generated_record_key.to_vec())?
    );

    println!("   ✓ Key conversions successful\n");

    Ok(())
}

/// Example 4: Round-trip conversion patterns
fn demo_round_trip_conversions() -> Result<(), Box<dyn std::error::Error>> {
    println!("4. Round-trip Conversion Patterns:");

    let product = Product {
        sku: 12345,
        name: "High-Performance Laptop".to_string(),
        price: 1299.99,
        category: "Electronics".to_string(),
    };

    // Method 1: Using to_kad_record() method
    let record1 = product.to_kad_record()?;

    // Method 2: Using From trait
    let record2: libp2p::kad::Record = product.clone().into();

    // Both methods should produce equivalent results
    assert_eq!(record1.key, record2.key);
    assert_eq!(record1.value, record2.value);
    println!("   ✓ Both conversion methods produce identical results");

    // Convert back using From trait
    let recovered1: Product = record1.into();
    let recovered2: Product = record2.into();

    // Verify both recovered products are identical to original
    assert_eq!(product, recovered1);
    assert_eq!(product, recovered2);
    println!("   ✓ Round-trip conversions preserve data integrity");

    // Test key conversion round-trip
    let original_key = product.key().clone();
    let record_key: libp2p::kad::RecordKey = original_key.clone().into();
    let key_from_record = ProductKey::from(libp2p::kad::Record {
        key: record_key.clone(),
        value: vec![],
        publisher: None,
        expires: None,
    });

    println!("   Original key: {}", original_key.as_str());
    println!("   Key from record: {}", key_from_record.as_str());
    println!("   ✓ Key round-trip successful\n");

    Ok(())
}

/// Example 5: Practical Kademlia DHT usage patterns
fn demo_kademlia_patterns() -> Result<(), Box<dyn std::error::Error>> {
    println!("5. Practical Kademlia DHT Usage Patterns:");

    let users = vec![
        User {
            id: "user_001".to_string(),
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
        },
        User {
            id: "user_002".to_string(),
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
        },
        User {
            id: "user_003".to_string(),
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
            age: 35,
        },
    ];

    // Pattern 1: Storing data in DHT
    println!("   Storing users in DHT format:");
    let mut records = Vec::new();
    for user in &users {
        let record: libp2p::kad::Record = user.clone().into();
        let key_str = String::from_utf8(record.key.to_vec())?;
        println!("     Stored user '{}' with key: {}", user.name, key_str);
        records.push(record);
    }

    // Pattern 2: Querying by key
    println!("   \n   Querying DHT by key:");
    let query_key = UserKey::new("user_002".to_string());
    let query_record_key: libp2p::kad::RecordKey = query_key.into();

    // Find matching record (simulated DHT lookup)
    if let Some(found_record) = records.iter().find(|r| r.key == query_record_key) {
        let found_user: User = found_record.clone().into();
        println!("     Found user: {}", found_user.name);
    }

    // Pattern 3: Bulk retrieval and conversion
    println!("   \n   Bulk retrieval from DHT:");
    let retrieved_users: Vec<User> = records.into_iter().map(|record| record.into()).collect();

    println!("     Retrieved {} users from DHT", retrieved_users.len());
    for user in &retrieved_users {
        println!("       - {} ({})", user.name, user.email);
    }

    println!("   ✓ Kademlia patterns demonstrated successfully\n");
    Ok(())
}

/// Example 6: Bulk operations and performance patterns
fn demo_bulk_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("6. Bulk Operations and Performance Patterns:");

    // Generate test data
    let documents: Vec<Document> = (0..5)
        .map(|i| match i % 3 {
            0 => Document::Text {
                content_hash: format!("text_hash_{}", i),
                content: format!("Sample text content number {}", i),
                word_count: 5 + i,
            },
            1 => Document::Image {
                image_id: format!("img_{:03}", i),
                format: "JPEG".to_string(),
                size_bytes: 1024 * (100 + i as u64),
            },
            _ => Document::Video {
                video_id: i as u64,
                duration_seconds: 60 * (i as u32 + 1),
                resolution: "1280x720".to_string(),
            },
        })
        .collect();

    // Bulk conversion to records
    println!("   Converting {} documents to records...", documents.len());
    let records: Vec<libp2p::kad::Record> =
        documents.iter().cloned().map(|doc| doc.into()).collect();

    // Show record information
    for (i, record) in records.iter().enumerate() {
        let key_str = String::from_utf8(record.key.to_vec())?;
        println!(
            "     Record {}: key={}, size={} bytes",
            i,
            key_str,
            record.value.len()
        );
    }

    // Bulk conversion back to documents
    println!("   Converting records back to documents...");
    let recovered_documents: Vec<Document> =
        records.into_iter().map(|record| record.into()).collect();

    // Verify data integrity
    assert_eq!(documents.len(), recovered_documents.len());
    for (original, recovered) in documents.iter().zip(recovered_documents.iter()) {
        assert_eq!(original, recovered);
    }

    println!(
        "   ✓ Bulk operations completed with {} items",
        documents.len()
    );

    // Key generation performance
    println!("   \n   Key generation performance test:");
    let start_time = std::time::Instant::now();
    let generated_keys: Vec<UserKey> = (0..100).map(|_| UserKey::generate_key()).collect();
    let generation_time = start_time.elapsed();

    println!(
        "     Generated {} unique keys in {:?}",
        generated_keys.len(),
        generation_time
    );

    // Verify all keys are unique
    let mut key_set = std::collections::HashSet::new();
    for key in &generated_keys {
        assert!(
            key_set.insert(key.as_str().to_string()),
            "Duplicate key found!"
        );
    }
    println!("     ✓ All generated keys are unique");

    println!("   ✓ Performance tests completed\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_from_implementations() -> Result<(), Box<dyn std::error::Error>> {
        let user = User {
            id: "test_user".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            age: 25,
        };

        // Test User -> Record -> User
        let record: libp2p::kad::Record = user.clone().into();
        let recovered_user: User = record.into();
        assert_eq!(user, recovered_user);

        // Test Key -> RecordKey
        let key = UserKey::new("test_key".to_string());
        let record_key: libp2p::kad::RecordKey = key.clone().into();
        let key_bytes = record_key.to_vec();
        let key_string = String::from_utf8(key_bytes)?;
        assert_eq!(key.as_str(), key_string);

        Ok(())
    }

    #[test]
    fn test_enum_from_implementations() -> Result<(), Box<dyn std::error::Error>> {
        let doc = Document::Text {
            content_hash: "test_hash".to_string(),
            content: "Test content".to_string(),
            word_count: 2,
        };

        // Test Document -> Record -> Document
        let record: libp2p::kad::Record = doc.clone().into();
        let recovered_doc: Document = record.into();
        assert_eq!(doc, recovered_doc);

        Ok(())
    }
}
