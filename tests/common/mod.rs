//! Common test utilities and shared functionality
//!
//! This module provides shared utilities for all test modules including:
//! - Logger initialization
//! - Test data creation helpers
//! - Database cleanup utilities
//! - Test synchronization primitives

use log::{info, warn};
use std::sync::Once;
use std::time::Duration;
use tempfile::TempDir;

static INIT: Once = Once::new();

/// Initialize logger for tests with appropriate filtering
pub fn init_test_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format_timestamp_secs()
            .is_test(true)
            .init();
    });
}

/// Initialize logger with debug level for verbose testing
pub fn init_debug_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .format_timestamp_secs()
            .is_test(true)
            .init();
    });
}

/// Create a temporary directory for test databases
pub fn create_temp_db_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

/// Generate unique test identifier based on current timestamp
pub fn generate_test_id() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Generate current timestamp in seconds
pub fn current_timestamp_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Generate current timestamp in milliseconds
pub fn current_timestamp_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Standard timeout duration for test operations
pub const TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Short timeout for quick operations
pub const SHORT_TIMEOUT: Duration = Duration::from_secs(5);

/// Long timeout for complex operations
pub const LONG_TIMEOUT: Duration = Duration::from_secs(60);

/// Standard wait time for DHT propagation
pub const DHT_PROPAGATION_WAIT: Duration = Duration::from_secs(5);

/// Wait time for peer discovery
pub const PEER_DISCOVERY_WAIT: Duration = Duration::from_secs(10);

/// Cleanup database directory and log the operation
pub fn cleanup_db_dir(temp_dir: &TempDir) {
    let path = temp_dir.path();
    info!("🧹 Cleaning up test database directory: {:?}", path);

    if path.exists() {
        match std::fs::remove_dir_all(path) {
            Ok(_) => info!("✅ Successfully cleaned up database directory"),
            Err(e) => warn!("⚠️ Failed to cleanup database directory: {}", e),
        }
    }
}

/// Wait for a condition with timeout and periodic checking
pub async fn wait_for_condition<F, Fut>(
    condition: F,
    timeout: Duration,
    check_interval: Duration,
    description: &str,
) -> Result<(), String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if condition().await {
            info!("✅ Condition met: {}", description);
            return Ok(());
        }

        tokio::time::sleep(check_interval).await;
    }

    Err(format!("❌ Timeout waiting for condition: {}", description))
}

/// Test result tracking for comprehensive test reporting
#[derive(Debug, Default)]
pub struct TestResults {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

impl TestResults {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_pass(&mut self) {
        self.total_tests += 1;
        self.passed += 1;
    }

    pub fn add_fail(&mut self, error: String) {
        self.total_tests += 1;
        self.failed += 1;
        self.errors.push(error);
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            self.passed as f64 / self.total_tests as f64
        }
    }

    pub fn print_summary(&self) {
        info!("📊 Test Results Summary:");
        info!("   Total: {}", self.total_tests);
        info!("   Passed: {}", self.passed);
        info!("   Failed: {}", self.failed);
        info!("   Success Rate: {:.1}%", self.success_rate() * 100.0);

        if !self.errors.is_empty() {
            info!("❌ Errors:");
            for (i, error) in self.errors.iter().enumerate() {
                info!("   {}: {}", i + 1, error);
            }
        }
    }
}

/// Test node configuration for multi-node tests
#[derive(Debug, Clone)]
pub struct TestNodeConfig {
    pub name: String,
    pub port: u16,
    pub bootstrap_peers: Vec<String>,
}

impl TestNodeConfig {
    pub fn new(name: &str, port: u16) -> Self {
        Self {
            name: name.to_string(),
            port,
            bootstrap_peers: Vec::new(),
        }
    }

    pub fn with_bootstrap_peers(mut self, peers: Vec<String>) -> Self {
        self.bootstrap_peers = peers;
        self
    }
}

/// Generate test node configurations for multi-node testing
pub fn generate_test_node_configs(count: usize, base_port: u16) -> Vec<TestNodeConfig> {
    (0..count)
        .map(|i| TestNodeConfig::new(&format!("node_{}", i), base_port + i as u16))
        .collect()
}

pub mod test_runner;
