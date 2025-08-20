use test_macros::{UserRandom, UserRandomKey, netabase_trait::NetabaseSchema};

fn main() {
    println!("=== NetabaseSchema Macro Test ===");

    // Test basic struct instantiation
    let user = UserRandom {
        id: 42,
        name: "Test User".to_string(),
        another: "Additional Data".to_string(),
    };

    // Test that the NetabaseSchema trait is implemented
    println!("User key: {}", user.key());

    // Test cloning functionality
    let user_clone = user.clone();
    println!("Cloned user key: {}", user_clone.key());
    println!(
        "Original user ID: {}, Cloned user ID: {}",
        user.id, user_clone.id
    );

    // Test that we can create multiple users
    let user2 = UserRandom {
        id: 100,
        name: "Another User".to_string(),
        another: "More Data".to_string(),
    };

    println!("User2 key: {}", user2.key());

    // Verify that the macro-generated implementation works
    let expected_key1 = UserRandomKey("42".to_string());
    let expected_key2 = UserRandomKey("100".to_string());
    assert_eq!(user.key(), expected_key1);
    assert_eq!(user2.key(), expected_key2);
    assert_eq!(user_clone.key(), expected_key1);

    println!("✓ NetabaseSchema derive macro works!");
    println!("✓ Trait implementation generated successfully!");
    println!("✓ Key extraction works!");
    println!("✓ Clone functionality works!");
    println!("=== All tests passed! Macro compilation error fixed! ===");
}
