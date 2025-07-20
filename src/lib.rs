#![feature(duration_constructors_lite)]
#![feature(mpmc_channel)]
pub mod database;
pub mod network;

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

pub fn get_test_temp_dir(test_number: Option<u32>) -> String {
    use rand::Rng;
    match test_number {
        Some(num) => format!("./tmp{}_{}", num, rand::rng().random::<u32>()),
        None => format!("./tmp0_{}", rand::rng().random::<u32>()),
    }
}

pub fn get_test_temp_dir_str(suffix: Option<&str>) -> String {
    use rand::Rng;
    match suffix {
        Some(s) => format!("./tmp{}_{}", s, rand::rng().random::<u32>()),
        None => format!("./tmp0_{}", rand::rng().random::<u32>()),
    }
}

pub fn get_test_temp_dir_with_default(test_number: Option<u32>) -> String {
    use rand::Rng;
    match test_number {
        Some(num) => format!("./tmp{}_{}", num, rand::rng().random::<u32>()),
        None => format!("./tmp0_{}", rand::rng().random::<u32>()),
    }
}

/// Test helper functions for numbered database tests
pub mod test_database {
    use crate::database::tests::*;

    pub fn test_put_get_remove_record_numbered(test_number: u32) {
        put_get_remove_record_with_number(Some(test_number));
    }

    pub fn test_add_get_remove_provider_numbered(test_number: u32) {
        add_get_remove_provider_with_number(Some(test_number));
    }

    pub fn test_provided_numbered(test_number: u32) {
        provided_with_number(Some(test_number));
    }

    pub fn test_update_provider_numbered(test_number: u32) {
        update_provider_with_number(Some(test_number));
    }

    pub fn test_update_provided_numbered(test_number: u32) {
        update_provided_with_number(Some(test_number));
    }

    pub fn test_max_providers_per_key_numbered(test_number: u32) {
        max_providers_per_key_with_number(Some(test_number));
    }

    pub fn test_max_provided_keys_numbered(test_number: u32) {
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

    pub async fn test_swarm_numbered(test_number: u32) {
        test_swarm_with_number(Some(test_number)).await;
    }

    async fn test_swarm_with_number(test_number: Option<u32>) {
        crate::init_logging();
        let temp_dir = crate::get_test_temp_dir(test_number);
        let mut swarm = generate_swarm(&temp_dir).expect("Swarm Generation error");
        let (_tx, mut _rx) = tokio::sync::broadcast::channel::<NetabaseBehaviourEvent>(1);

        swarm
            .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().expect("Parse error"))
            .expect("Listen error");

        // Simple test that just verifies swarm creation and listening works
        log::info!("Swarm test completed for test number: {:?}", test_number);
    }
}
