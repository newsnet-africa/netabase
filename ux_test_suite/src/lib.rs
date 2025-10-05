//! # Netabase UX Test Suite
//!
//! This crate provides comprehensive testing of the user experience for Netabase
//! macro hygiene and dependency auto-export functionality. It validates that users
//! can use Netabase macros without manually importing external dependencies.
//!
//! ## Test Categories
//!
//! 1. **Hygiene Tests** - Validate macros work without any manual imports
//! 2. **Convenience Tests** - Test re-exported dependencies work correctly
//! 3. **Integration Tests** - Full workflow testing with all features
//! 4. **Compilation Tests** - Ensure generated code compiles cleanly
//! 5. **Real World Scenarios** - Test common usage patterns
//!
//! ## Macro Hygiene Validation
//!
//! The core principle being tested is that users should be able to write:
//!
//! ```rust
//! use netabase_macros::NetabaseModel;
//!
//! #[derive(NetabaseModel)]
//! #[key_name(UserKey)]
//! pub struct User {
//!     #[key]
//!     pub id: u64,
//!     pub name: String,
//! }
//! ```
//!
//! Without needing to manually import `serde`, `bincode`, `strum`, etc.

pub mod hygiene_test_simple;
pub mod test_helpers;
pub mod test_models;

pub use hygiene_test_simple::*;
/// Re-export commonly used test utilities
pub use test_helpers::*;
pub use test_models::*;

/// Test result type for consistent error handling
pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Macro to assert that a type implements a trait without importing the trait
#[macro_export]
macro_rules! assert_trait_implemented {
    ($type:ty, $trait:path) => {
        const _: fn() = || {
            fn assert_impl<T: $trait>() {}
            assert_impl::<$type>();
        };
    };
}

/// Test configuration for different scenarios
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub name: String,
    pub description: String,
    pub validate_hygiene: bool,
    pub validate_functionality: bool,
    pub validate_networking: bool,
}

impl TestConfig {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            validate_hygiene: true,
            validate_functionality: true,
            validate_networking: false,
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn hygiene_only(mut self) -> Self {
        self.validate_hygiene = true;
        self.validate_functionality = false;
        self.validate_networking = false;
        self
    }

    pub fn with_networking(mut self) -> Self {
        self.validate_networking = true;
        self
    }
}

/// Test execution framework
pub struct TestRunner {
    config: TestConfig,
}

impl TestRunner {
    pub fn new(config: TestConfig) -> Self {
        Self { config }
    }

    pub fn run<F>(&self, test_fn: F) -> TestResult
    where
        F: FnOnce(&TestConfig) -> TestResult,
    {
        println!("Running test: {}", self.config.name);
        if !self.config.description.is_empty() {
            println!("Description: {}", self.config.description);
        }

        let result = test_fn(&self.config);

        match &result {
            Ok(_) => println!("✓ Test passed: {}", self.config.name),
            Err(e) => println!("✗ Test failed: {}: {}", self.config.name, e),
        }

        result
    }
}

/// Utility to measure compilation times and memory usage
pub struct CompilationMetrics {
    pub compile_time_ms: u128,
    pub generated_code_size: usize,
}

impl CompilationMetrics {
    pub fn measure<F, T>(f: F) -> (T, CompilationMetrics)
    where
        F: FnOnce() -> T,
    {
        let start = std::time::Instant::now();
        let result = f();
        let compile_time_ms = start.elapsed().as_millis();

        (
            result,
            CompilationMetrics {
                compile_time_ms,
                generated_code_size: 0, // Would need proc-macro expansion info
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = TestConfig::new("test")
            .with_description("A test")
            .hygiene_only();

        assert_eq!(config.name, "test");
        assert_eq!(config.description, "A test");
        assert!(config.validate_hygiene);
        assert!(!config.validate_functionality);
        assert!(!config.validate_networking);
    }

    #[test]
    fn test_runner_basic() {
        let config = TestConfig::new("basic_test");
        let runner = TestRunner::new(config);

        let result = runner.run(|_| Ok(()));
        assert!(result.is_ok());
    }
}
