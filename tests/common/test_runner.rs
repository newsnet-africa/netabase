//! Test runner that enforces single-threaded execution to avoid race conditions
//!
//! This module provides utilities to run tests sequentially, which is especially
//! important for Sled database tests that cannot handle concurrent access to the
//! same database files.

use log::{error, info, warn};
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Global semaphore to ensure only one full integration test runs at a time
static GLOBAL_TEST_SEMAPHORE: std::sync::LazyLock<Arc<Semaphore>> =
    std::sync::LazyLock::new(|| Arc::new(Semaphore::new(1)));

/// Test execution result
#[derive(Debug)]
pub enum TestResult {
    Passed(Duration),
    Failed(String, Duration),
    Timeout(Duration),
}

impl TestResult {
    pub fn is_success(&self) -> bool {
        matches!(self, TestResult::Passed(_))
    }

    pub fn duration(&self) -> Duration {
        match self {
            TestResult::Passed(d) | TestResult::Failed(_, d) | TestResult::Timeout(d) => *d,
        }
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            TestResult::Failed(msg, _) => Some(msg),
            TestResult::Timeout(_) => Some("Test timed out"),
            TestResult::Passed(_) => None,
        }
    }
}

/// Test execution configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub name: String,
    pub timeout: Duration,
    pub retry_count: usize,
    pub requires_isolation: bool,
}

impl TestConfig {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            timeout: Duration::from_secs(60),
            retry_count: 0,
            requires_isolation: true,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_retries(mut self, retry_count: usize) -> Self {
        self.retry_count = retry_count;
        self
    }

    pub fn without_isolation(mut self) -> Self {
        self.requires_isolation = false;
        self
    }
}

/// Single-threaded test runner for database tests
pub struct SingleThreadTestRunner {
    _permit: Option<tokio::sync::SemaphorePermit<'static>>,
}

impl SingleThreadTestRunner {
    /// Acquire exclusive access for running a test
    pub async fn acquire() -> Result<Self, String> {
        match GLOBAL_TEST_SEMAPHORE.acquire().await {
            Ok(permit) => {
                info!("🔒 Acquired exclusive test lock");
                Ok(Self {
                    _permit: Some(permit),
                })
            }
            Err(e) => Err(format!("Failed to acquire test lock: {}", e)),
        }
    }

    /// Run a single test with the given configuration
    pub async fn run_test<F, Fut>(&self, config: TestConfig, mut test_fn: F) -> TestResult
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<(), String>>,
    {
        info!("🧪 Starting test: {}", config.name);
        let start_time = Instant::now();

        let mut attempts = 0;
        let max_attempts = config.retry_count + 1;

        while attempts < max_attempts {
            if attempts > 0 {
                info!("🔄 Retry attempt {} for test: {}", attempts, config.name);
                // Wait a bit before retrying
                tokio::time::sleep(Duration::from_secs(2)).await;
            }

            let result = match tokio::time::timeout(config.timeout, test_fn()).await {
                Ok(Ok(())) => {
                    let duration = start_time.elapsed();
                    info!("✅ Test passed: {} (took {:?})", config.name, duration);
                    return TestResult::Passed(duration);
                }
                Ok(Err(e)) => {
                    attempts += 1;
                    if attempts < max_attempts {
                        warn!(
                            "❌ Test failed (attempt {}): {} - {}",
                            attempts, config.name, e
                        );
                        continue;
                    } else {
                        let duration = start_time.elapsed();
                        error!("❌ Test failed (final): {} - {}", config.name, e);
                        return TestResult::Failed(e, duration);
                    }
                }
                Err(_) => {
                    let duration = start_time.elapsed();
                    error!("⏰ Test timed out: {} (after {:?})", config.name, duration);
                    return TestResult::Timeout(duration);
                }
            };
        }

        unreachable!("Should not reach this point")
    }

    /// Run multiple tests sequentially
    pub async fn run_test_suite<F, Fut>(
        &self,
        suite_name: &str,
        tests: Vec<(TestConfig, F)>,
    ) -> Vec<TestResult>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<(), String>>,
    {
        info!(
            "📋 Starting test suite: {} ({} tests)",
            suite_name,
            tests.len()
        );
        let suite_start = Instant::now();
        let mut results = Vec::new();

        for (config, mut test_fn) in tests {
            let result = self.run_test(config, test_fn).await;
            results.push(result);

            // Small delay between tests to ensure cleanup
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        let suite_duration = suite_start.elapsed();
        let passed = results.iter().filter(|r| r.is_success()).count();
        let total = results.len();

        info!(
            "📊 Test suite completed: {} - {}/{} passed (took {:?})",
            suite_name, passed, total, suite_duration
        );

        if passed < total {
            error!("❌ Some tests failed in suite: {}", suite_name);
            for (i, result) in results.iter().enumerate() {
                if let Some(error) = result.error_message() {
                    error!("   Test {}: {}", i + 1, error);
                }
            }
        }

        results
    }
}

impl Drop for SingleThreadTestRunner {
    fn drop(&mut self) {
        if self._permit.is_some() {
            info!("🔓 Released exclusive test lock");
        }
    }
}

/// Macro to create a single-threaded test
#[macro_export]
macro_rules! single_threaded_test {
    ($test_name:ident, $timeout:expr, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            use $crate::common::test_runner::{SingleThreadTestRunner, TestConfig};

            let runner = SingleThreadTestRunner::acquire()
                .await
                .expect("Failed to acquire test runner");

            let config = TestConfig::new(stringify!($test_name)).with_timeout($timeout);

            let result = runner
                .run_test(config, || async move { $test_body.await })
                .await;

            assert!(
                result.is_success(),
                "Test failed: {:?}",
                result.error_message()
            );
        }
    };
}

/// Macro to create a single-threaded test with retries
#[macro_export]
macro_rules! single_threaded_test_with_retries {
    ($test_name:ident, $timeout:expr, $retries:expr, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            use $crate::common::test_runner::{SingleThreadTestRunner, TestConfig};

            let runner = SingleThreadTestRunner::acquire()
                .await
                .expect("Failed to acquire test runner");

            let config = TestConfig::new(stringify!($test_name))
                .with_timeout($timeout)
                .with_retries($retries);

            let result = runner
                .run_test(config, || async move { $test_body.await })
                .await;

            assert!(
                result.is_success(),
                "Test failed after {} retries: {:?}",
                $retries,
                result.error_message()
            );
        }
    };
}

/// Utility function to run a quick test without full isolation
pub async fn run_quick_test<F, Fut>(name: &str, test_fn: F) -> Result<(), String>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<(), String>>,
{
    info!("⚡ Running quick test: {}", name);
    let start = Instant::now();

    match tokio::time::timeout(Duration::from_secs(30), test_fn()).await {
        Ok(Ok(())) => {
            info!(
                "✅ Quick test passed: {} (took {:?})",
                name,
                start.elapsed()
            );
            Ok(())
        }
        Ok(Err(e)) => {
            error!("❌ Quick test failed: {} - {}", name, e);
            Err(e)
        }
        Err(_) => {
            error!("⏰ Quick test timed out: {}", name);
            Err("Test timed out".to_string())
        }
    }
}

/// Test execution statistics
#[derive(Debug, Default)]
pub struct TestStats {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub total_duration: Duration,
    pub longest_test: Option<(String, Duration)>,
    pub shortest_test: Option<(String, Duration)>,
}

impl TestStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_result(&mut self, test_name: &str, result: &TestResult) {
        self.total_tests += 1;
        self.total_duration += result.duration();

        match result {
            TestResult::Passed(_) => self.passed_tests += 1,
            TestResult::Failed(_, _) | TestResult::Timeout(_) => self.failed_tests += 1,
        }

        // Track longest and shortest tests
        let duration = result.duration();
        if let Some((_, longest_duration)) = &self.longest_test {
            if duration > *longest_duration {
                self.longest_test = Some((test_name.to_string(), duration));
            }
        } else {
            self.longest_test = Some((test_name.to_string(), duration));
        }

        if let Some((_, shortest_duration)) = &self.shortest_test {
            if duration < *shortest_duration {
                self.shortest_test = Some((test_name.to_string(), duration));
            }
        } else {
            self.shortest_test = Some((test_name.to_string(), duration));
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            self.passed_tests as f64 / self.total_tests as f64
        }
    }

    pub fn print_summary(&self) {
        info!("📈 Test Statistics:");
        info!("   Total Tests: {}", self.total_tests);
        info!("   Passed: {}", self.passed_tests);
        info!("   Failed: {}", self.failed_tests);
        info!("   Success Rate: {:.1}%", self.success_rate() * 100.0);
        info!("   Total Duration: {:?}", self.total_duration);

        if let Some((name, duration)) = &self.longest_test {
            info!("   Longest Test: {} ({:?})", name, duration);
        }

        if let Some((name, duration)) = &self.shortest_test {
            info!("   Shortest Test: {} ({:?})", name, duration);
        }

        if self.total_tests > 0 {
            let avg_duration = self.total_duration / self.total_tests as u32;
            info!("   Average Duration: {:?}", avg_duration);
        }
    }
}
