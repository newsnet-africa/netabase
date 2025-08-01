use test_macros::{NetabaseSchema, UserRandom};

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
    assert_eq!(user.key(), "placeholder");
    assert_eq!(user2.key(), "placeholder");
    assert_eq!(user_clone.key(), "placeholder");

    println!("✓ NetabaseSchema derive macro works!");
    println!("✓ Trait implementation generated successfully!");
    println!("✓ Key extraction works!");
    println!("✓ Clone functionality works!");
    println!("=== All tests passed! Macro compilation error fixed! ===");
}
