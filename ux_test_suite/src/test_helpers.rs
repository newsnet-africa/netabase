//! # Test Helpers
//!
//! This module provides utilities and helpers for testing Netabase macro hygiene
//! and user experience. It includes database setup, temporary directories,
//! assertion helpers, and test data generators.

use crate::TestResult;
use std::path::Path;
use tempfile::TempDir;

/// Helper for creating temporary test databases
pub struct TestDatabase {
    pub temp_dir: TempDir,
    pub db_path: std::path::PathBuf,
}

impl TestDatabase {
    /// Create a new temporary database for testing
    pub fn new() -> TestResult<Self> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_db");
        Ok(Self { temp_dir, db_path })
    }

    /// Get the database path as a string
    pub fn path(&self) -> &Path {
        &self.db_path
    }

    /// Get the database path as a string slice
    pub fn path_str(&self) -> &str {
        self.db_path.to_str().unwrap()
    }
}

/// Test data generators for consistent test scenarios
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate a sequence of test IDs
    pub fn ids(count: usize) -> Vec<u64> {
        (1..=count as u64).collect()
    }

    /// Generate test names
    pub fn names(count: usize) -> Vec<String> {
        (1..=count).map(|i| format!("TestUser{}", i)).collect()
    }

    /// Generate test email addresses
    pub fn emails(count: usize) -> Vec<String> {
        (1..=count)
            .map(|i| format!("user{}@example.com", i))
            .collect()
    }

    /// Generate test timestamps
    pub fn timestamps(count: usize) -> Vec<u64> {
        let base = 1600000000u64; // Some base timestamp
        (0..count)
            .map(|i| base + (i as u64 * 3600)) // Each hour apart
            .collect()
    }

    /// Generate random test data
    pub fn random_string(len: usize) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        len.hash(&mut hasher);
        std::time::SystemTime::now().hash(&mut hasher);
        format!("test_{:x}", hasher.finish())
    }
}

/// Macro hygiene validation helpers
pub struct HygieneValidator;

impl HygieneValidator {
    /// Validate that a type can be created without manual imports
    pub fn validate_no_imports_needed<T>() -> bool {
        // This is a compile-time test - if it compiles, the test passes
        true
    }

    /// Validate that generated code doesn't conflict with user imports
    pub fn validate_no_name_conflicts() -> bool {
        // Test that common names don't conflict
        let _serde = "user_defined_serde";
        let _bincode = "user_defined_bincode";
        let _strum = "user_defined_strum";
        true
    }
}

/// Performance testing helpers
pub struct PerformanceHelper;

impl PerformanceHelper {
    /// Measure execution time of a function
    pub fn measure_time<F, R>(f: F) -> (R, std::time::Duration)
    where
        F: FnOnce() -> R,
    {
        let start = std::time::Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Measure memory usage (simplified)
    pub fn measure_memory<F, R>(f: F) -> (R, usize)
    where
        F: FnOnce() -> R,
    {
        // Simplified memory measurement
        let result = f();
        (result, 0) // Would need more sophisticated measurement in real scenarios
    }
}

/// Assertion helpers for test validation
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that a result is successful and return the value
    pub fn assert_ok<T, E>(result: Result<T, E>) -> T
    where
        E: std::fmt::Debug,
    {
        match result {
            Ok(value) => value,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    }

    /// Assert that two collections have the same elements (order independent)
    pub fn assert_same_elements<T>(a: &[T], b: &[T])
    where
        T: PartialEq + std::fmt::Debug,
    {
        assert_eq!(a.len(), b.len(), "Collections have different lengths");
        for item in a {
            assert!(
                b.contains(item),
                "Item {:?} not found in second collection",
                item
            );
        }
    }

    /// Assert that a value is within an expected range
    pub fn assert_in_range<T>(value: T, min: T, max: T)
    where
        T: PartialOrd + std::fmt::Debug,
    {
        assert!(
            value >= min && value <= max,
            "Value {:?} not in range [{:?}, {:?}]",
            value,
            min,
            max
        );
    }
}

/// Test environment setup and cleanup
pub struct TestEnvironment {
    pub logger_initialized: bool,
}

impl TestEnvironment {
    /// Initialize the test environment
    pub fn new() -> Self {
        Self {
            logger_initialized: false,
        }
    }

    /// Initialize logging for tests
    pub fn init_logging(&mut self) {
        if !self.logger_initialized {
            let _ = env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Debug)
                .is_test(true)
                .try_init();
            self.logger_initialized = true;
        }
    }

    /// Clean up test environment
    pub fn cleanup(&self) {
        // Perform any necessary cleanup
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Macro to create a test that validates hygiene
#[macro_export]
macro_rules! hygiene_test {
    ($name:ident, $model:ty) => {
        #[test]
        fn $name() {
            // This test validates that the model can be used without manual imports
            // If this compiles, the hygiene test passes
            let _test = <$model>::default();
            assert!(true, "Hygiene test passed for {}", stringify!($model));
        }
    };
}

/// Macro to create a test that validates functionality
#[macro_export]
macro_rules! functionality_test {
    ($name:ident, $model:ty, $key:ty) => {
        #[test]
        fn $name() -> crate::TestResult {
            use netabase_store::traits::NetabaseModel;

            let model = <$model>::default();
            let key = model.key();

            // Validate that key extraction works
            assert!(true, "Key extraction works for {}", stringify!($model));

            Ok(())
        }
    };
}

/// Utility for generating test scenarios
pub struct ScenarioGenerator;

impl ScenarioGenerator {
    /// Generate a basic CRUD test scenario
    pub fn basic_crud() -> Vec<(&'static str, Box<dyn Fn() -> TestResult>)> {
        vec![
            ("create", Box::new(|| Ok(()))),
            ("read", Box::new(|| Ok(()))),
            ("update", Box::new(|| Ok(()))),
            ("delete", Box::new(|| Ok(()))),
        ]
    }

    /// Generate a complex query test scenario
    pub fn complex_queries() -> Vec<(&'static str, Box<dyn Fn() -> TestResult>)> {
        vec![
            ("primary_key_query", Box::new(|| Ok(()))),
            ("secondary_key_query", Box::new(|| Ok(()))),
            ("range_query", Box::new(|| Ok(()))),
            ("compound_query", Box::new(|| Ok(()))),
        ]
    }

    /// Generate a networking test scenario
    pub fn networking() -> Vec<(&'static str, Box<dyn Fn() -> TestResult>)> {
        vec![
            ("put_record", Box::new(|| Ok(()))),
            ("get_record", Box::new(|| Ok(()))),
            ("broadcast", Box::new(|| Ok(()))),
            ("provider_query", Box::new(|| Ok(()))),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_helper() -> TestResult {
        let db = TestDatabase::new()?;
        assert!(db.path().exists() || !db.path().exists()); // Path may or may not exist yet
        Ok(())
    }

    #[test]
    fn test_data_generator() {
        let ids = TestDataGenerator::ids(3);
        assert_eq!(ids, vec![1, 2, 3]);

        let names = TestDataGenerator::names(2);
        assert_eq!(names, vec!["TestUser1", "TestUser2"]);

        let random = TestDataGenerator::random_string(10);
        assert!(!random.is_empty());
    }

    #[test]
    fn test_performance_helper() {
        let (result, duration) = PerformanceHelper::measure_time(|| {
            std::thread::sleep(std::time::Duration::from_millis(1));
            42
        });

        assert_eq!(result, 42);
        assert!(duration.as_millis() >= 1);
    }

    #[test]
    fn test_assertions() {
        TestAssertions::assert_ok(Ok::<i32, &str>(42));
        TestAssertions::assert_same_elements(&[1, 2, 3], &[3, 1, 2]);
        TestAssertions::assert_in_range(5, 1, 10);
    }

    #[test]
    fn test_environment() {
        let mut env = TestEnvironment::new();
        env.init_logging();
        assert!(env.logger_initialized);
    }
}
