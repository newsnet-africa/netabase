#![feature(duration_constructors_lite)]
#![feature(mpmc_channel)]

use rand::Rng;

pub mod config;
pub mod database;
pub mod network;

//TODO:
// 1. Read over generated code
// 2. Implement missing functions
// 3. Start integrating the database with the network
// 4. So much clean up:
//    - Commenting and documentation
//    - Remove unused imports
//    - Refactor code for better readability and maintainability
//    - Add more comprehensive error handling
//    - Improve test coverage
// 5. Fix testing and make it cohesive lol
// 6. NB!!! Make sure that the message passing and monitoring of the swarm is working correctly and completed in implementation:
//    - Implement message passing between nodes
//    - Monitor the health and status of the swarm
// NB!! - This is mostly so that there is an exposeable api to the netabase that can easily be used by the frontend

/// Initialize logging for tests across the library.
/// This function ensures that logging is initialized only once using `std::sync::Once`
/// to prevent multiple initialization attempts when running multiple tests.
///
/// The logger is configured with:
/// - Test mode enabled (`.is_test(true)`)
/// - Info level filtering to show relevant debug information during tests
/// - Graceful handling of re-initialization attempts
pub fn init_logging() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .try_init();
    });
}

/// Get a deterministic or random suffix for temp directories.
/// If NETABASE_TEST_SEED environment variable is set, it will use that seed
/// to generate a deterministic suffix. Otherwise, it generates a random suffix.
///
/// This enables consistent paths across machines and test sessions when needed.
pub fn get_deterministic_or_random_suffix(seed: Option<u64>) -> u32 {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    if let Some(inner_seed) = seed {
        StdRng::seed_from_u64(inner_seed).random::<u32>()
    } else {
        let mut rng = rand::rng(); // Create a random number generator
        rng.random::<u32>() // Generate a random u32
    }
}

pub fn get_test_temp_dir(test_number: Option<u64>, seed: Option<u64>) -> String {
    format!(
        "./test_tmp/tmp{}_{}",
        test_number.unwrap_or(0),
        get_deterministic_or_random_suffix(seed)
    )
}

pub fn get_test_temp_dir_str(suffix: Option<&str>, seed: Option<u64>) -> String {
    format!(
        "./test_tmp/tmp{}_{}",
        suffix.unwrap_or("0"),
        get_deterministic_or_random_suffix(seed)
    )
}

/// Test helper functions for numbered database tests
pub mod test_database {
    use crate::database::tests::*;

    pub fn test_put_get_remove_record_numbered(test_number: u64) {
        put_get_remove_record_with_number(Some(test_number));
    }

    pub fn test_add_get_remove_provider_numbered(test_number: u64) {
        add_get_remove_provider_with_number(Some(test_number));
    }

    pub fn test_provided_numbered(test_number: u64) {
        provided_with_number(Some(test_number));
    }

    pub fn test_update_provider_numbered(test_number: u64) {
        update_provider_with_number(Some(test_number));
    }

    pub fn test_update_provided_numbered(test_number: u64) {
        update_provided_with_number(Some(test_number));
    }

    pub fn test_max_providers_per_key_numbered(test_number: u64) {
        max_providers_per_key_with_number(Some(test_number));
    }

    pub fn test_max_provided_keys_numbered(test_number: u64) {
        max_provided_keys_with_number(Some(test_number));
    }
}

/// Test helper functions for numbered network tests
pub mod test_network {
    use crate::network::{behaviour::NetabaseBehaviourEvent, swarm::generate_swarm};

    #[tokio::test]
    async fn test_swarm() {
        test_swarm_with_number(Some(8)).await;
    }

    async fn test_swarm_with_number(test_number: Option<u64>) {
        crate::init_logging();
        let temp_dir = crate::get_test_temp_dir(test_number, None);
        let mut swarm = generate_swarm(&temp_dir).expect("Swarm Generation error");

        swarm
            .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().expect("Parse error"))
            .expect("Listen error");

        // Simple test that just verifies swarm creation and listening works
        log::info!("Swarm test completed for test number: {:?}", test_number);
    }
}
