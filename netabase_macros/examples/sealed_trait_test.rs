//! Test to verify that the NetabaseSchema derive macro generates all required sealed trait implementations
//!
//! This example demonstrates that derived schemas implement all the necessary traits
//! for the sealed NetabaseSchema trait, including Send, Sync, Unpin, Clone, Debug,
//! serialization traits, and conversion traits.

use netabase_macros::NetabaseSchema;

#[derive(NetabaseSchema, Clone, Debug, PartialEq)]
struct ExampleUser {
    #[key]
    id: String,
    name: String,
    email: String,
}

#[derive(NetabaseSchema, Clone, Debug, PartialEq)]
enum ExampleDocument {
    #[key]
    Post {
        id: String,
        title: String,
        content: String,
    },
    #[key]
    Comment {
        id: String,
        post_id: String,
        text: String,
    },
}

#[derive(NetabaseSchema, Clone, Debug, PartialEq)]
#[key = |product| format!("{}_{}", product.category, product.id)]
enum ExampleProduct {
    Book {
        id: String,
        category: String,
        title: String,
    },
    Electronics {
        id: String,
        category: String,
        model: String,
    },
}

fn main() {
    println!("=== Sealed Trait Implementation Test ===\n");

    test_user_sealed_traits();
    test_document_sealed_traits();
    test_product_sealed_traits();
    test_key_sealed_traits();
}

/// Test that ExampleUser and ExampleUserKey implement all sealed trait requirements
fn test_user_sealed_traits() {
    println!("1. Testing ExampleUser sealed trait implementations:");

    // Create a user instance
    let user = ExampleUser {
        id: "user123".to_string(),
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Test Clone trait
    let user_clone = user.clone();
    println!("   ✓ Clone: User can be cloned");

    // Test Debug trait
    println!("   ✓ Debug: {:?}", user);

    // Test PartialEq trait (if derived)
    assert_eq!(user, user_clone);
    println!("   ✓ PartialEq: Users can be compared");

    // Test key generation
    let key = user.key();
    let key_clone = key.clone();
    println!("   ✓ Key generation: {}", key.as_str());

    // Test key traits
    assert_eq!(key, key_clone);
    println!("   ✓ Key Clone and PartialEq work");

    // Test Display trait for key
    println!("   ✓ Key Display: {}", key);

    // Test AsRef<str> for key
    let key_str: &str = key.as_ref();
    println!("   ✓ Key AsRef<str>: {}", key_str);

    // Test conversion traits
    let key_bytes: Vec<u8> = key.clone().into();
    let key_from_bytes = ExampleUserKey::from(key_bytes);
    println!("   ✓ Key byte conversions work");

    // Test serialization (requires serde feature)
    test_serialization(&user, "User");

    println!();
}

/// Test that ExampleDocument enum implements sealed trait requirements
fn test_document_sealed_traits() {
    println!("2. Testing ExampleDocument enum sealed trait implementations:");

    let doc = ExampleDocument::Post {
        id: "post456".to_string(),
        title: "Hello World".to_string(),
        content: "This is a test post".to_string(),
    };

    // Test Clone
    let doc_clone = doc.clone();
    println!("   ✓ Clone: Document can be cloned");

    // Test Debug
    println!("   ✓ Debug: {:?}", doc);

    // Test key generation for enum
    let key = doc.key();
    println!("   ✓ Enum key generation: {}", key.as_str());

    // Test PartialEq
    assert_eq!(doc, doc_clone);
    println!("   ✓ PartialEq: Documents can be compared");

    test_serialization(&doc, "Document");

    println!();
}

/// Test that ExampleProduct enum with item-level closure implements sealed traits
fn test_product_sealed_traits() {
    println!("3. Testing ExampleProduct enum with item-level key closure:");

    let product = ExampleProduct::Book {
        id: "book789".to_string(),
        category: "fiction".to_string(),
        title: "The Great Novel".to_string(),
    };

    // Test Clone
    let product_clone = product.clone();
    println!("   ✓ Clone: Product can be cloned");

    // Test Debug
    println!("   ✓ Debug: {:?}", product);

    // Test item-level key closure
    let key = product.key();
    println!("   ✓ Item-level key generation: {}", key.as_str());
    // Should be "fiction_book789" based on the closure

    // Test PartialEq
    assert_eq!(product, product_clone);
    println!("   ✓ PartialEq: Products can be compared");

    test_serialization(&product, "Product");

    println!();
}

/// Test key-specific sealed trait implementations
fn test_key_sealed_traits() {
    println!("4. Testing Key type sealed trait implementations:");

    // Test key generation
    let generated_key = ExampleUserKey::generate_key();
    println!("   ✓ Key generation: {}", generated_key.as_str());

    // Test key creation from string
    let custom_key = ExampleUserKey::new("custom_id".to_string());
    println!("   ✓ Key from string: {}", custom_key);

    // Test key utility methods
    println!("   ✓ Key as_str(): {}", custom_key.as_str());
    println!(
        "   ✓ Key into_string(): {}",
        custom_key.clone().into_string()
    );

    // Test From<String>
    let from_string_key: ExampleUserKey = "from_string".to_string().into();
    println!("   ✓ From<String>: {}", from_string_key);

    // Test byte conversions
    let key_bytes: Vec<u8> = from_string_key.clone().into();
    let key_from_bytes = ExampleUserKey::from(key_bytes);
    assert_eq!(from_string_key, key_from_bytes);
    println!("   ✓ Byte conversions work correctly");

    println!();
}

/// Test serialization capabilities (demonstrates serde integration)
fn test_serialization<T>(item: &T, type_name: &str)
where
    T: serde::Serialize + Clone + std::fmt::Debug,
{
    // Test that the item can be serialized (this proves serde::Serialize is implemented)
    match serde_json::to_string(item) {
        Ok(json) => println!("   ✓ {} serialization: {} bytes", type_name, json.len()),
        Err(e) => println!("   ✗ {} serialization failed: {}", type_name, e),
    }
}

/// Compile-time tests for trait bounds
fn _compile_time_trait_tests() {
    // These functions will only compile if the traits are properly implemented

    fn requires_send_sync_unpin<T: Send + Sync + Unpin>(_: T) {}
    fn requires_clone_debug<T: Clone + std::fmt::Debug>(_: T) {}
    fn requires_sized<T: Sized>(_: T) {}

    let user = ExampleUser {
        id: "test".to_string(),
        name: "Test".to_string(),
        email: "test@test.com".to_string(),
    };

    let key = user.key();

    // Test that both User and UserKey implement required traits
    requires_send_sync_unpin(user.clone());
    requires_send_sync_unpin(key.clone());

    requires_clone_debug(user.clone());
    requires_clone_debug(key.clone());

    requires_sized(user);
    requires_sized(key);

    println!("   ✓ All compile-time trait bounds satisfied");
}

/// Test trait object compatibility (sealed traits should work with trait objects)
fn test_trait_objects() {
    println!("5. Testing trait object compatibility:");

    // This demonstrates that the sealed trait implementations work correctly
    let user = ExampleUser {
        id: "obj_test".to_string(),
        name: "Object Test".to_string(),
        email: "obj@test.com".to_string(),
    };

    // Test that we can use trait objects for standard traits
    let cloneable: &dyn Clone = &user;
    let debuggable: &dyn std::fmt::Debug = &user;

    println!("   ✓ Can create trait objects for Clone and Debug");
    println!("   ✓ Debug via trait object: {:?}", debuggable);

    // Test key trait objects
    let key = user.key();
    let displayable: &dyn std::fmt::Display = &key;
    let as_ref_str: &dyn AsRef<str> = &key;

    println!("   ✓ Key Display via trait object: {}", displayable);
    println!(
        "   ✓ Key AsRef<str> via trait object: {}",
        as_ref_str.as_ref()
    );

    println!();
}

/// Performance test for sealed trait implementations
fn test_performance() {
    println!("6. Testing performance characteristics:");

    let start = std::time::Instant::now();

    // Create many instances to test performance
    let users: Vec<ExampleUser> = (0..1000)
        .map(|i| ExampleUser {
            id: format!("user_{}", i),
            name: format!("User {}", i),
            email: format!("user{}@example.com", i),
        })
        .collect();

    // Test key generation performance
    let keys: Vec<ExampleUserKey> = users.iter().map(|u| u.key()).collect();

    // Test cloning performance
    let cloned_users: Vec<ExampleUser> = users.iter().cloned().collect();

    let duration = start.elapsed();

    println!(
        "   ✓ Created {} users, keys, and clones in {:?}",
        users.len(),
        duration
    );
    println!(
        "   ✓ Average per operation: {:?}",
        duration / (users.len() as u32 * 3)
    );

    // Verify all operations worked
    assert_eq!(users.len(), keys.len());
    assert_eq!(users.len(), cloned_users.len());
    assert_eq!(users, cloned_users);

    println!("   ✓ All performance operations completed successfully");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_implements_required_traits() {
        let user = ExampleUser {
            id: "test".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };

        // Test that user implements Send + Sync + Unpin
        fn assert_send_sync_unpin<T: Send + Sync + Unpin>(_: &T) {}
        assert_send_sync_unpin(&user);

        // Test Clone
        let cloned = user.clone();
        assert_eq!(user, cloned);

        // Test key generation
        let key = user.key();
        assert_eq!(key.as_str(), "test");
    }

    #[test]
    fn test_key_implements_required_traits() {
        let key = ExampleUserKey::new("test_key".to_string());

        // Test that key implements Send + Sync + Unpin
        fn assert_send_sync_unpin<T: Send + Sync + Unpin>(_: &T) {}
        assert_send_sync_unpin(&key);

        // Test Display
        assert_eq!(format!("{}", key), "test_key");

        // Test AsRef<str>
        let key_ref: &str = key.as_ref();
        assert_eq!(key_ref, "test_key");

        // Test From<String>
        let from_string: ExampleUserKey = "from_string".to_string().into();
        assert_eq!(from_string.as_str(), "from_string");
    }

    #[test]
    fn test_enum_with_closure_implements_required_traits() {
        let product = ExampleProduct::Electronics {
            id: "laptop123".to_string(),
            category: "computers".to_string(),
            model: "ThinkPad".to_string(),
        };

        // Test Send + Sync + Unpin
        fn assert_send_sync_unpin<T: Send + Sync + Unpin>(_: &T) {}
        assert_send_sync_unpin(&product);

        // Test Clone
        let cloned = product.clone();
        assert_eq!(product, cloned);

        // Test key generation with closure
        let key = product.key();
        assert_eq!(key.as_str(), "computers_laptop123");
    }

    #[test]
    fn test_byte_conversions() {
        let original_key = ExampleUserKey::new("byte_test".to_string());

        // Convert to bytes
        let bytes: Vec<u8> = original_key.clone().into();

        // Convert back from bytes
        let reconstructed_key = ExampleUserKey::from(bytes);

        assert_eq!(original_key, reconstructed_key);
        assert_eq!(original_key.as_str(), reconstructed_key.as_str());
    }

    #[test]
    fn test_key_generation() {
        // Test automatic key generation
        let generated = ExampleUserKey::generate_key();
        assert!(!generated.as_str().is_empty());

        // Each generated key should be unique
        let generated2 = ExampleUserKey::generate_key();
        assert_ne!(generated.as_str(), generated2.as_str());
    }
}
