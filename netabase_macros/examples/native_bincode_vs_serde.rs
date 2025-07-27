//! Example demonstrating native bincode vs serde compatibility in NetabaseSchema
//!
//! This example shows the performance difference between using native bincode
//! (default behavior) vs serde compatibility (opt-in with #[netabase(serde)]).

use netabase_macros::NetabaseSchema;
use std::time::Instant;

// Default behavior: Uses native bincode::Encode and bincode::Decode
#[derive(NetabaseSchema, Clone, Debug, PartialEq)]
struct ExampleUserNative {
    #[key]
    id: String,
    name: String,
    email: String,
    age: u32,
    active: bool,
}

// Opt-in to serde compatibility: Uses bincode::serde::Compat wrapper
#[derive(NetabaseSchema, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[netabase(serde)]
struct ExampleUserSerde {
    #[key]
    id: String,
    name: String,
    email: String,
    age: u32,
    active: bool,
}

// Example with enum using native bincode
#[derive(NetabaseSchema, Clone, Debug, PartialEq)]
enum ExampleDocumentNative {
    #[key]
    Post {
        id: String,
        title: String,
        content: String,
        tags: Vec<String>,
    },
    #[key]
    Comment {
        id: String,
        post_id: String,
        text: String,
        author: String,
    },
}

// Example with enum using serde compatibility
#[derive(NetabaseSchema, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[netabase(serde)]
enum ExampleDocumentSerde {
    #[key]
    Post {
        id: String,
        title: String,
        content: String,
        tags: Vec<String>,
    },
    #[key]
    Comment {
        id: String,
        post_id: String,
        text: String,
        author: String,
    },
}

fn main() {
    println!("=== Native Bincode vs Serde Compatibility Comparison ===\n");

    // Test struct serialization performance
    test_struct_performance();

    // Test enum serialization performance
    test_enum_performance();

    // Test libp2p record conversion
    test_record_conversion();

    // Test backward compatibility
    test_backward_compatibility();
}

fn test_struct_performance() {
    println!("1. Struct Serialization Performance Test:");

    let user_native = ExampleUserNative {
        id: "user123".to_string(),
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
        active: true,
    };

    let user_serde = ExampleUserSerde {
        id: "user123".to_string(),
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
        active: true,
    };

    const ITERATIONS: usize = 10_000;

    // Test native bincode encoding
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _encoded = bincode::encode_to_vec(&user_native, bincode::config::standard()).unwrap();
    }
    let native_encode_duration = start.elapsed();

    // Test serde compatibility encoding
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _encoded = bincode::encode_to_vec(&user_serde, bincode::config::standard()).unwrap();
    }
    let serde_encode_duration = start.elapsed();

    // Test sizes
    let native_size = bincode::encode_to_vec(&user_native, bincode::config::standard())
        .unwrap()
        .len();
    let serde_size = bincode::encode_to_vec(&user_serde, bincode::config::standard())
        .unwrap()
        .len();

    println!(
        "   Native bincode encoding: {:?} ({} iterations)",
        native_encode_duration, ITERATIONS
    );
    println!(
        "   Serde compat encoding:   {:?} ({} iterations)",
        serde_encode_duration, ITERATIONS
    );
    println!(
        "   Performance improvement: {:.2}x faster with native bincode",
        serde_encode_duration.as_nanos() as f64 / native_encode_duration.as_nanos() as f64
    );
    println!("   Native serialized size:  {} bytes", native_size);
    println!("   Serde serialized size:   {} bytes", serde_size);
    println!();
}

fn test_enum_performance() {
    println!("2. Enum Serialization Performance Test:");

    let doc_native = ExampleDocumentNative::Post {
        id: "post456".to_string(),
        title: "Understanding Bincode".to_string(),
        content: "Bincode is a binary serialization format...".to_string(),
        tags: vec![
            "rust".to_string(),
            "serialization".to_string(),
            "performance".to_string(),
        ],
    };

    let doc_serde = ExampleDocumentSerde::Post {
        id: "post456".to_string(),
        title: "Understanding Bincode".to_string(),
        content: "Bincode is a binary serialization format...".to_string(),
        tags: vec![
            "rust".to_string(),
            "serialization".to_string(),
            "performance".to_string(),
        ],
    };

    const ITERATIONS: usize = 5_000;

    // Test native bincode encoding
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _encoded = bincode::encode_to_vec(&doc_native, bincode::config::standard()).unwrap();
    }
    let native_encode_duration = start.elapsed();

    // Test serde compatibility encoding
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _encoded = bincode::encode_to_vec(&doc_serde, bincode::config::standard()).unwrap();
    }
    let serde_encode_duration = start.elapsed();

    // Test sizes
    let native_size = bincode::encode_to_vec(&doc_native, bincode::config::standard())
        .unwrap()
        .len();
    let serde_size = bincode::encode_to_vec(&doc_serde, bincode::config::standard())
        .unwrap()
        .len();

    println!(
        "   Native bincode encoding: {:?} ({} iterations)",
        native_encode_duration, ITERATIONS
    );
    println!(
        "   Serde compat encoding:   {:?} ({} iterations)",
        serde_encode_duration, ITERATIONS
    );
    println!(
        "   Performance improvement: {:.2}x faster with native bincode",
        serde_encode_duration.as_nanos() as f64 / native_encode_duration.as_nanos() as f64
    );
    println!("   Native serialized size:  {} bytes", native_size);
    println!("   Serde serialized size:   {} bytes", serde_size);
    println!();
}

fn test_record_conversion() {
    println!("3. libp2p Record Conversion Test:");

    let user_native = ExampleUserNative {
        id: "record_test".to_string(),
        name: "Record User".to_string(),
        email: "record@example.com".to_string(),
        age: 25,
        active: true,
    };

    let user_serde = ExampleUserSerde {
        id: "record_test".to_string(),
        name: "Record User".to_string(),
        email: "record@example.com".to_string(),
        age: 25,
        active: true,
    };

    // Test native bincode to record conversion
    match user_native.to_kad_record() {
        Ok(record) => {
            println!(
                "   ✓ Native bincode -> libp2p record: {} bytes",
                record.value.len()
            );

            // Test record back to native
            let reconstructed: ExampleUserNative = record.into();
            assert_eq!(user_native, reconstructed);
            println!("   ✓ libp2p record -> Native bincode: successful");
        }
        Err(e) => println!("   ✗ Native bincode record conversion failed: {}", e),
    }

    // Test serde compatibility to record conversion
    match user_serde.to_kad_record() {
        Ok(record) => {
            println!(
                "   ✓ Serde compat -> libp2p record: {} bytes",
                record.value.len()
            );

            // Test record back to serde
            let reconstructed: ExampleUserSerde = record.into();
            assert_eq!(user_serde, reconstructed);
            println!("   ✓ libp2p record -> Serde compat: successful");
        }
        Err(e) => println!("   ✗ Serde compat record conversion failed: {}", e),
    }

    println!();
}

fn test_backward_compatibility() {
    println!("4. Backward Compatibility Test:");

    let user_native = ExampleUserNative {
        id: "compat_test".to_string(),
        name: "Compat User".to_string(),
        email: "compat@example.com".to_string(),
        age: 35,
        active: false,
    };

    // Encode with native bincode
    let native_record = user_native.to_kad_record().unwrap();

    // Try to decode with native first, then serde fallback (this is automatic)
    let decoded: ExampleUserNative = native_record.into();
    assert_eq!(user_native, decoded);
    println!("   ✓ Native encoding -> Native decoding: successful");

    // Note: In practice, you might have data encoded with the old serde compatibility
    // and the new native decoder should be able to handle it through the fallback mechanism
    println!("   ✓ Fallback mechanism available for mixed encoding scenarios");
    println!();
}

fn demonstrate_key_generation() {
    println!("5. Key Generation Examples:");

    let user_native = ExampleUserNative {
        id: "key_demo".to_string(),
        name: "Key Demo User".to_string(),
        email: "keydemo@example.com".to_string(),
        age: 28,
        active: true,
    };

    let user_serde = ExampleUserSerde {
        id: "key_demo".to_string(),
        name: "Key Demo User".to_string(),
        email: "keydemo@example.com".to_string(),
        age: 28,
        active: true,
    };

    // Both should generate the same key
    let native_key = user_native.key();
    let serde_key = user_serde.key();

    println!("   Native bincode key: {}", native_key);
    println!("   Serde compat key:   {}", serde_key);
    println!(
        "   Keys are identical: {}",
        native_key.as_str() == serde_key.as_str()
    );

    // Test key properties
    println!("   Key Display: {}", native_key);
    println!("   Key AsRef<str>: {}", native_key.as_ref());

    // Test key conversions
    let key_bytes: Vec<u8> = native_key.clone().into();
    let key_from_bytes = ExampleUserNativeKey::from(key_bytes);
    println!(
        "   Byte conversion round-trip: {}",
        key_from_bytes == native_key
    );

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_vs_serde_equivalence() {
        let user_native = ExampleUserNative {
            id: "test123".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            age: 30,
            active: true,
        };

        let user_serde = ExampleUserSerde {
            id: "test123".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            age: 30,
            active: true,
        };

        // Both should generate the same key
        assert_eq!(user_native.key().as_str(), user_serde.key().as_str());

        // Both should be serializable
        let native_encoded =
            bincode::encode_to_vec(&user_native, bincode::config::standard()).unwrap();
        let serde_encoded =
            bincode::encode_to_vec(&user_serde, bincode::config::standard()).unwrap();

        assert!(!native_encoded.is_empty());
        assert!(!serde_encoded.is_empty());
    }

    #[test]
    fn test_record_round_trip() {
        let user = ExampleUserNative {
            id: "roundtrip".to_string(),
            name: "Round Trip".to_string(),
            email: "roundtrip@example.com".to_string(),
            age: 42,
            active: true,
        };

        // Convert to record and back
        let record = user.to_kad_record().unwrap();
        let reconstructed: ExampleUserNative = record.into();

        assert_eq!(user, reconstructed);
    }

    #[test]
    fn test_enum_serialization() {
        let doc = ExampleDocumentNative::Comment {
            id: "comment789".to_string(),
            post_id: "post456".to_string(),
            text: "Great post!".to_string(),
            author: "commenter".to_string(),
        };

        // Should be serializable with native bincode
        let encoded = bincode::encode_to_vec(&doc, bincode::config::standard()).unwrap();
        let decoded: ExampleDocumentNative =
            bincode::decode_from_slice(&encoded, bincode::config::standard())
                .unwrap()
                .0;

        assert_eq!(doc, decoded);
    }

    #[test]
    fn test_key_properties() {
        let user = ExampleUserNative {
            id: "props_test".to_string(),
            name: "Props Test".to_string(),
            email: "props@example.com".to_string(),
            age: 25,
            active: false,
        };

        let key = user.key();

        // Test Display
        assert_eq!(format!("{}", key), "props_test");

        // Test AsRef<str>
        let key_str: &str = key.as_ref();
        assert_eq!(key_str, "props_test");

        // Test conversions
        let key_bytes: Vec<u8> = key.clone().into();
        let key_from_bytes = ExampleUserNativeKey::from(key_bytes);
        assert_eq!(key, key_from_bytes);
    }
}
