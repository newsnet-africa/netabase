//! Netabase Test Library
//!
//! This module provides common utilities and organized test structure for all Netabase tests.
//! It ensures proper test isolation, single-threaded execution for database tests,
//! and comprehensive logging for debugging.
//!
//! Test Organization:
//! - `common/`: Shared utilities and test helpers
//! - `unit/`: Unit tests for individual components
//! - `integration/`: Integration tests for multi-component functionality
//! - `kademlia/`: Kademlia DHT specific tests
//! - `sled/`: Sled database persistence tests
//!
//! Usage:
//! ```rust
//! use crate::common::{init_test_logger, TestResults, TEST_TIMEOUT};
//! use crate::common::test_runner::{SingleThreadTestRunner, TestConfig};
//! ```

pub mod common;

// Re-export commonly used items for convenience
pub use common::{
    DHT_PROPAGATION_WAIT, LONG_TIMEOUT, PEER_DISCOVERY_WAIT, SHORT_TIMEOUT, TEST_TIMEOUT,
    TestNodeConfig, TestResults, cleanup_db_dir, create_temp_db_dir, current_timestamp_millis,
    current_timestamp_secs, generate_test_id, generate_test_node_configs, init_debug_logger,
    init_test_logger, wait_for_condition,
};

pub use common::test_runner::{
    SingleThreadTestRunner, TestConfig, TestResult, TestStats, run_quick_test,
};

// Test execution macros for easy single-threaded test creation
#[macro_export]
macro_rules! sled_test {
    ($test_name:ident, $test_body:block) => {
        #[tokio::test]
        async fn $test_name() {
            use $crate::common::test_runner::{SingleThreadTestRunner, TestConfig};
            use $crate::common::{LONG_TIMEOUT, init_debug_logger};

            init_debug_logger();

            let runner = SingleThreadTestRunner::acquire()
                .await
                .expect("Failed to acquire test runner");

            let config = TestConfig::new(stringify!($test_name)).with_timeout(LONG_TIMEOUT);

            let result = runner
                .run_test(config, || async move {
                    $test_body;
                    Ok(())
                })
                .await;

            assert!(
                result.is_success(),
                "Test failed: {:?}",
                result.error_message()
            );
        }
    };
}

#[macro_export]
macro_rules! integration_test {
    ($test_name:ident, $test_body:block) => {
        #[tokio::test]
        async fn $test_name() {
            use $crate::common::{TEST_TIMEOUT, init_test_logger};

            init_test_logger();

            let test_result: Result<(), String> = async move {
                $test_body;
                Ok(())
            }
            .await;

            assert!(test_result.is_ok(), "Test failed: {:?}", test_result.err());
        }
    };
}

#[macro_export]
macro_rules! unit_test {
    ($test_name:ident, $test_body:block) => {
        #[tokio::test]
        async fn $test_name() {
            use $crate::common::{SHORT_TIMEOUT, init_test_logger};

            init_test_logger();

            let test_result: Result<(), String> = async move {
                $test_body;
                Ok(())
            }
            .await;

            assert!(test_result.is_ok(), "Test failed: {:?}", test_result.err());
        }
    };
}

/// Helper function to setup test environment
pub fn setup_test_environment() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        // Initialize logging
        init_debug_logger();

        log::info!("🧪 Test environment initialized");
    });
}

/// Trait for test fixtures that need cleanup
pub trait TestFixture {
    fn setup(&mut self) -> Result<(), String>;
    fn teardown(&mut self) -> Result<(), String>;
}

/// Generic test runner that handles setup/teardown
pub async fn run_with_fixture<T, F, Fut>(mut fixture: T, test_fn: F) -> Result<(), String>
where
    T: TestFixture,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    fixture.setup()?;

    let result = test_fn().await;

    if let Err(e) = fixture.teardown() {
        log::warn!("⚠️ Fixture teardown failed: {}", e);
    }

    result
}
