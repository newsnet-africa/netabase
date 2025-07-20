//! Example demonstrating how to use numbered test functionality
//!
//! This example shows how to run tests with specific test numbers to use
//! different temp directories (tmp0, tmp1, tmp2, etc.)

use netabase::{get_test_temp_dir, test_database, test_network};

#[tokio::main]
async fn main() {
    println!("NetaBase Test Runner - Demonstrating numbered test functionality");
    println!("================================================================");

    // Initialize logging
    netabase::init_logging();

    println!("\n1. Running tests with default temp directory (tmp0):");
    println!("   - Default behavior when no test number is specified");
    println!("   - Uses: {}", get_test_temp_dir(None));

    // Run database tests with default temp directory (tmp0)
    println!("\n   Running database tests with default temp directory...");
    test_database::test_put_get_remove_record_numbered(0);
    println!("   ✓ put_get_remove_record test completed with tmp0");

    test_database::test_add_get_remove_provider_numbered(0);
    println!("   ✓ add_get_remove_provider test completed with tmp0");

    println!("\n2. Running tests with custom temp directories:");

    // Run tests with test number 1 (tmp1)
    println!("   - Test number 1 uses: {}", get_test_temp_dir(Some(1)));
    test_database::test_provided_numbered(1);
    println!("   ✓ provided test completed with tmp1");

    // Run tests with test number 2 (tmp2)
    println!("   - Test number 2 uses: {}", get_test_temp_dir(Some(2)));
    test_database::test_update_provider_numbered(2);
    println!("   ✓ update_provider test completed with tmp2");

    // Run tests with test number 3 (tmp3)
    println!("   - Test number 3 uses: {}", get_test_temp_dir(Some(3)));
    test_database::test_update_provided_numbered(3);
    println!("   ✓ update_provided test completed with tmp3");

    // Run tests with test number 4 (tmp4)
    println!("   - Test number 4 uses: {}", get_test_temp_dir(Some(4)));
    test_database::test_max_providers_per_key_numbered(4);
    println!("   ✓ max_providers_per_key test completed with tmp4");

    // Run tests with test number 5 (tmp5)
    println!("   - Test number 5 uses: {}", get_test_temp_dir(Some(5)));
    test_database::test_max_provided_keys_numbered(5);
    println!("   ✓ max_provided_keys test completed with tmp5");

    println!("\n3. Running network test with custom temp directory:");
    println!("   - Test number 6 uses: {}", get_test_temp_dir(Some(6)));
    test_network::test_swarm_numbered(6).await;
    println!("   ✓ swarm test completed with tmp6");

    println!("\n================================================================");
    println!("All tests completed successfully!");
    println!("\nTemp directories used:");
    println!("  - tmp0: Default tests");
    println!("  - tmp1: provided test");
    println!("  - tmp2: update_provider test");
    println!("  - tmp3: update_provided test");
    println!("  - tmp4: max_providers_per_key test");
    println!("  - tmp5: max_provided_keys test");
    println!("  - tmp6: swarm test");

    println!("\nYou can now run individual tests with specific numbers:");
    println!("  netabase::test_database::test_put_get_remove_record_numbered(7);");
    println!("  netabase::test_network::test_swarm_numbered(8).await;");
}
