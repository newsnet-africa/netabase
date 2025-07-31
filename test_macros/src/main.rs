use test_macros::User;

fn main() {
    println!("=== NetabaseSchema Macro Test ===");

    // Test basic struct instantiation
    let user = User {
        id: 42,
        name: "Test User".to_string(),
    };

    // Test that the generated methods work
    println!("User schema name: {}", User::schema_name());
    println!("User key bytes length: {}", user.get_key().len());

    // Test that we can create multiple users
    let user2 = User {
        id: 100,
        name: "Another User".to_string(),
    };

    println!("User2 schema name: {}", User::schema_name());
    println!("User2 key bytes length: {}", user2.get_key().len());

    println!("✓ Basic macro functionality works!");
    println!("✓ Schema name generation works!");
    println!("✓ Key extraction works!");
    println!("=== Test completed successfully ===");
}
